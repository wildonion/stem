

/*  ========================================================================================

        Neuron Actor DSL

            neuron onion{
                {
                    cells: 25,
                    shellcodeConfig: {
                        shellcode: 0x...,
                        encryption: aes256
                    },
                    nestedData: {
                        key: value
                    }
                }
            }

            every neuron actor must be able to do streaming either locally or remotely 
            by using one of the following syntax:
            stream!{
                local stream over onion: // backed by jobq mpsc
                    data -> | () => { 
                        // Send message logic
                    }),
                    data <- | () => { 
                        // Receive message logic
                    });
                
                remote stream over onion:  // backed by rmq
                    data -> | () => { 
                        // Send message logic
                    }),
                    data <- | () => { 
                        // Receive message logic
                    });
            }

    brokering is all about queueing, sending and receiving messages way more faster, 
    safer and reliable than a simple eventloop or a tcp based channel. 
    all brokers contains message/task/job queue to handle communication between services
    asyncly, they've been designed specially for decoupling tight long running services 
    due to their durability nature like giving predicting output of an ai model service
    while the ai model is working on other input prediction in the background we can 
    receive the old outputs and pass them through the brokers to receive them in some 
    http service for responding the caller.   
    In rmq producer sends message to exchange the a consumer can bind its queue to 
    the exchange to receive the messages, routing key determines the pattern of receiving 
    messages inside the bounded queue from the exchange 
    In kafka producer sends messages to topic the consumer can receives data from 
    the topic, Rmq adds an extra layer on top of msg handling logic which is creating 
    queue per each consumer.
    offset in kafka is an strategy which determines the way of tracking the sequential 
    order of receiving messages by kafka topics it's like routing key in rmq 
    in kafka you should create consumer and producer separately but in rmq everything is 
    started from a channel, we'll create a channel to declare the queue, exchange, consumer 
    and producer in that channel, channel is a thread that can manage multiple connection 
    to the broker through a single tcp connection.

    BROKER TYPES: (preferred stack: RMQ + RPC + WebSocket + ShortPollingJobId)
        → REDIS PUBSUB => light task queue
        → KAFKA        => high latency hight throughput
            -ˋˏ✄┈┈┈┈
            >_ topic contains messages allows consumers to consume from topics each topic 
            can be divided into multiple partitions for example a topic might have 10 
            partitions, each message can have its own unique partition key which specifies 
            to which partition the message will go. Offset can be assigned to each message
            within a partition it specifies the position of the message in that partition
            it's useful for the consumers to resume consuming where they've left.
            a consumer can commit the message after processing it which tells kafka that
            the consumer has received and processed the message completely.
            single consumer consumes messages from specific partitions but in group of 
            consumers kafka ensures that each partition is consumed by only one consumer 
            within the group like if a topic with 4 partitions and 2 consumers are in the 
            same group, Kafka will assign 2 partitions to each consumer:

                                                                 ----------> partition-key1 queue(m1, m2, m3, ...) - all messages with key1
             ---------                       ------------       |
            |consumer1| <-----consume-----> |Kafka Broker| <-----topic-----> partition-key3 queue(m1, m2, m3, ...) - all messages with key3
             ---------                      ------------ |      |
                |_______partition1&2______________|      |      |----------> partition-key2 queue(m1, m2, m3, ...) - all messages with key2
             ---------                            |      |      |
            |consumer2|                           |      |       ----------> partition-key4 queue(m1, m2, m3, ...) - all messages with key4
             ---------                            |      |
                 |                                |   ---------
                  --------------partition3&4------   | producer|
                                                      ---------
            it's notable that committing the offset too early, instead, might cause message 
            loss, since upon recovery the consumer will start from the next message, skipping 
            the one where the failure occurred.

        → RMQ          => low latency low throughput
            -ˋˏ✄┈┈┈┈
            >_ the consuming task has been started by sending the ConsumeNotif message 
            to this actor which will execute the streaming loop over the queue in 
            either the notif consumer actor context itself or the tokio spawn thread:

                notif consumer -----Q(Consume Payload)-----> Exchange -----notif CQRS writer-----> cache/store on Redis & db

            -ˋˏ✄┈┈┈┈
            >_ the producing task has been started by sending the ProduceNotif message 
            to this actor which will execute the publishing process to the exchange 
            in either the notif producer actor context itself or the tokio spawn thread:

                notif producer -----payload-----> Exchange
            
            -ˋˏ✄┈┈┈┈
            >_ client uses a short polling technique or websocket streaming to fetch notifications 
            for an specific owner from redis or db, this is the best solution to implement a
            realtiming strategy on the client side to fetch what's happening on the 
            server side in realtime.

             _________                                      _________
            | server1 | <------- RMQ notif broker -------> | server2 |
             ---------                                      ---------
                | ws                                          | ws
                 ------------------- client ------------------
    ======================================================================================== 
*/

use task::Task;
use task::TaskStatus;
use tokio::sync::TryLockError;
use tokio::sync::MutexGuard;
use tx::Transaction;
use wallexerr::misc::SecureCellConfig;
use misc::{runInterval, Crypter};
use crate::*;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct CryptoConfig{
    pub secret: String,
    pub passphrase: String,
    pub unique_key: String
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum ActionType{ // all the action type that causes the notif to get fired
    #[default]
    ProductPurchased, // or minted
    Zerlog,
    EventCreated,
    EventExpired,
    EventLocked,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct EventData{
    pub id: String,
    pub receiver_info: String,
    pub action_data: serde_json::Value, // you can map this to any structure you know!
    pub actioner_info: String,
    pub action_type: ActionType,
    pub fired_at: i64,
    pub is_seen: bool,
}

#[derive(Clone)]
pub struct Worker{
    pub id: String,
    pub thread: std::sync::Arc<tokio::task::JoinHandle<()>>,
}

#[derive(Clone)]
pub struct InternalExecutor<Event>{
    pub id: String, 
    pub buffer: Buffer<Event>,
    pub sender: tokio::sync::mpsc::Sender<Event>,
    // receive Event inside tokio::spawn and execute them in the background
    pub eventloop: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Event>>> 
}

pub struct Job<J: Clone, S>
where J: std::future::Future<Output = ()> + Send + Sync + 'static{
    pub id: String, 
    pub task: Task<J, S>
}

pub struct Runner<J: Clone, S>
where J: std::future::Future<Output = ()> + Send + Sync + 'static{
    pub id: String,
    pub job: Job<J, S>
}

#[derive(Clone, Debug)]
pub struct Event{
    pub data: EventData,
    pub status: EventStatus,
    pub offset: u64,
}

#[derive(Clone, Debug)]
pub enum EventStatus{
    Committed,
    Executed,
    Halted
}

#[derive(Clone)]
pub struct Buffer<E>{ // eg: Buffer<Event>
    pub events: std::sync::Arc<tokio::sync::Mutex<Vec<E>>>,
    pub size: usize
}

/*  ====================================================================================
       REALTIME NOTIF EVENT STREAMING DESIGN PATTERN (README files inside docs folder)
    ====================================================================================
                   MAKE SURE YOU'VE STARTED CONSUMERS BEFORE PRODUCING
                   THEY MUST BE READY FOR CONSUMING WHILE PRODUCERS ARE
                   SENDING MESSAGES TO THE BROKER. USUALLY RUN THE PRODUCER
                   USING CLI AND THE CONSUMER AT STARTUP.

    NotifBrokerActor is the worker of handling the process of publishing and consuming 
    messages through rmq, redis and kafka, talking to the NotifBrokerActor can be done 
    by sending it a message contains the setup either to publish or consume something 
    to and from an specific broker, so generally it's a sexy actor to produce/consume 
    messages from different type of brokers it uses RMQ, Redis and Kafka to produce and 
    consume massive messages in realtime, kindly it supports data AES256 encryption 
    through producing messages to the broker. we can send either producing or consuming 
    message to this actor to start producing or consuming in the background.

    how capnpc works?
    in RPC every method call is a round trip in networking, canpnp pack all calls together 
    in only one round trip, it uses the promise pipelining feature which every call is a 
    future object which can be solved by awaiting in which it returns all the results from 
    all the calls sent to the server it's like `foo().bar().end()` takes only 1 round trip 
    which by awaiting on them it returns all the result from the server, it can call methods 
    without waiting just take a round trip. call results are returned to the client before 
    the request even arrives at the server, this is the feature of promise it's a place 
    holder for the result of each call and once we await on them all the results will be 
    arrived in one round trip.

    ************************************************************************************
    it's notable that for realtime push notif streaming we MUST start consuming from
    the specified broker passed in to the message structure when talking with actor, in
    a place where the application logic which is likely a server is being started.
    ************************************************************************************
    
    ====================================================================================
*/

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct SendRpcMessage{ // used to send rpc request through rmq queue, good for actor communication directly through rpc backed by rmq
    pub reqQueue: String,
    pub repQueue: String,
    pub payload: String, // json string maybe! we'll convert it to u8 bytes eventually
}

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct PublishNotifToRmq{
    pub local_spawn: bool, // either spawn in actor context or tokio threadpool
    pub notif_data: EventData,
    pub exchange_name: String,
    pub exchange_type: String,
    pub routing_key: String,
    pub encryptionConfig: Option<CryptoConfig>,
}

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct ConsumeNotifFromRmq{ // we'll create a channel then start consuming by binding a queue to the exchange
    /* -ˋˏ✄┈┈┈┈ 
        following queue gets bounded to the passed in exchange type with its 
        routing key, when producer wants to produce notif data it sends them 
        to the exchange with a known routing key, any queue that is bounded 
        to that exchange routing key will be filled up with messages coming 
        from the producer and they stay in there until a consumer read them
    */
    pub queue: String,
    pub exchange_name: String,
    /* -ˋˏ✄┈┈┈┈ 
        routing_key is pattern for the exchange to route the messages to the 
        bounded queue.
        multiple producers can send their messages to a single exchange but 
        each of with different routing keys.
        any queue that is bounded to the exchange routing key will receive 
        all the messages that follows the pattern inside the routing_key.
        a message can be sent from producer to an exchange in a topic way with 
        an sepecific routing key which tells the exchange this is the way of 
        receiving messages that a bounded queue does since we might have 
        sent messages to the same exchange with multiple different routing 
        keys per each message and for a queue that is bounded to the exchange 
        with the passed in routing key can only receives the messages that 
        follow the pattern in the selected routing key. so the routing key in 
        consumer is the patterns for this queue to tell exchange to what 
        messages this queue is interested in:

        1) producer produces messages and send them to the exchange with an specific routing key
        2) a consumer create its own queue and bind it to the exchange with the bind key that 
           is interested to receive the message from the exchange based on that key.
        3) it's notable that a queue can be bounded to multiple exchange at the same time 
           it allows to receive different messages based on each exchange routing key.
                                                                                                                 --------          ---------
                                                                                                                | queue1 | <----- |consumer1|
                                                                        ------> routing_key1 <---------------------------          ---------
                                                                       |                                            
        producer1 ----------                                       -----------------> routing_key0  .........        
                            |____ messages > routing_key1 ------> | exchange1|                                                
                             ____ messages > routing_key4 ------>  -----------------> routing_key2  .........                                   
                            |                                          |                                --------        -----------
       producer2 -----------                                           |                               | queue2 | <----| consumer2 |
                                                                        ------> routing_key4 <------------------        -----------
                                                                     ----------                             |
                                                                    | exchange2| -----bind(routing_key)-----
                                                                     ----------
    */
    pub routing_key: String, // patterns for this queue to tell exchange what messages this queue is interested in
    pub tag: String,
    pub redis_cache_exp: u64,
    pub local_spawn: bool, // either spawn in actor context or tokio threadpool
    pub store_in_db: bool,
    pub decryptionConfig: Option<CryptoConfig>
}

#[derive(Clone)]
pub enum ContractType{
    Verifier,
    Transmitter(Addr<NeuronActor>),
}

#[derive(Clone)]
pub struct Contract{
    pub cont_type: ContractType,
    pub opcode: u8,
}

#[derive(Clone)]
pub struct NeuronActor{
    pub wallet: Option<wallexerr::misc::Wallet>,
    pub metadata: Option<serde_json::Value>,
    pub internal_executor: InternalExecutor<Event>, // sender and receiver, potentiall we can use the actor msg sending pattern
    pub transactions: Option<std::sync::Arc<tokio::sync::Mutex<Vec<Transaction>>>>,
    pub internal_worker: Option<std::sync::Arc<tokio::sync::Mutex<Worker>>>,
    pub contract: Option<Contract>, // circom and noir for zk verifier contract
}


impl<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S> Runner<J, S>{
    pub async fn execute(&mut self){

    }
}

impl Event{

    pub async fn process(&mut self){

    }

}

impl InternalExecutor<Event>{
    
    pub fn new(buffer: Buffer<Event>) -> Self{
        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(100);
        let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
        Self { id: Uuid::new_v4().to_string(), buffer, sender: tx, eventloop: rx }
    }

    pub async fn spawn(&self, event: Event) -> Self{
        let sender = self.clone().sender;
        sender.send(event).await;
        self.clone()
    }

    pub async fn run<F, R: std::future::Future<Output = ()> + Send + Sync + 'static, >(&self, callback: F)
    where F: Clone + Fn(Event) -> R + Send + Sync + 'static{
        let get_rx = self.clone().eventloop;
        let mut rx = get_rx.lock().await;
        
        while let Some(mut event) = rx.recv().await{
            
            let event_for_first_tokio = event.clone();
            let cloned_event = event.clone();
            let cloned_callback = callback.clone();

            tokio::spawn(async move{
                event_for_first_tokio.clone().process().await;
            });
            tokio::spawn(async move{
                cloned_callback(cloned_event).await; // calling callback with the passed in received event
            });
        }
    }

}

impl NeuronActor{
    
    // see EventLoop struct in src/playground/app.rs
    pub async fn on<R: std::future::Future<Output = ()> + Send + Sync + 'static, 
        F: Clone + Fn(Event) -> R + Send + Sync + 'static>(&mut self, streamer: &str, eventType: &str, callback: F) -> Self{
        
        let get_internal_executor = self.clone().internal_executor;
        let get_events = get_internal_executor.buffer.events.lock().await;
        let cloned_get_internal_executor = get_internal_executor.clone();
        let last_event = get_events.last().unwrap();
        
        // in order to use * or deref mark on last_event the Copy trait must be implemented 
        // for the type since the Copy is not implemented for heap data types thus we should 
        // use clone() method on them to return the owned type.
        let owned_las_event = last_event.clone();

        match eventType{
            "send" => {
                
                match streamer{
                    "local" => {
                        // sending in the background
                        let first_token_last_event = owned_las_event.clone();
                        
                        // spawn an event for the executor
                        tokio::spawn(async move{
                            cloned_get_internal_executor.spawn(first_token_last_event).await;
                        });
                        
                        // executing the callback in the background thread
                        tokio::spawn(callback(owned_las_event.to_owned()));
                        self.clone()
                    },
                    "rmq" => {
                        self.clone()
                    },
                    _ => {
                        log::error!("unknown streamer!");
                        self.clone()
                    }
                }
            },
            "receive" => {
                
                match streamer{
                    "local" => {
                        // receiving in the background 
                        tokio::spawn(async move{
                            cloned_get_internal_executor.run(callback).await;
                        });
                        self.clone()
                    },
                    "rmq" => {
                        self.clone()
                    },
                    _ => {
                        log::error!("unknown streamer!");
                        self.clone()
                    }
                }

            },
            _ => {
                log::info!("unknown event type!");
                self.clone()
            }
        }

    }

    pub async fn publishToRmq(&self, data: &str, exchange: &str, routing_key: &str, 
        exchange_type: &str, secureCellConfig: SecureCellConfig, redisUniqueKey: &str){

            // TODO
            // use self.on() method

    }

    pub async fn consumeFromRmq(&self, exp_seconds: u64,
        consumer_tag: &str, queue: &str, 
        binding_key: &str, exchange: &str,
        store_in_db: bool,
        decryptionConfig: Option<CryptoConfig>
    ){

            // TODO
            // send consumed data to the internal streamer channel
            // use self.on() method
    }

    // if we can acquire the lock means the lock is free
    // and is not being in used by another process which 
    // causes the second process to be awaited until the 
    // lock gets freed.
    pub async fn execute<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S>
        (&mut self, mut runner: Runner<J, S>) where S: Send + Sync + 'static{

        if self.internal_worker.is_none(){
            return;
        }

        let get_internal_worker = self.internal_worker.clone().unwrap();
        let worker_lock = get_internal_worker.try_lock();

        // if the lock is busy reject the execution
        if worker_lock.is_err(){
            return;
        }

        let mut worker = worker_lock.unwrap();

        // update the worker thread with a new runner 
        (*worker).thread = std::sync::Arc::new(
            tokio::spawn(async move{
                runner.execute().await;
            })
        );
    }

}

impl Actor for NeuronActor{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        log::error!("neuron actor has stopped");        
    }
}

impl ActixMessageHandler<SendRpcMessage> for NeuronActor{
    type Result = ();
    fn handle(&mut self, msg: SendRpcMessage, ctx: &mut Self::Context) -> Self::Result {
        
        ()
    }
}

/* ********************************************************************************* */
/* ***************************** PRODUCE NOTIF HANDLER ***************************** */
/* ********************************************************************************* */
impl ActixMessageHandler<PublishNotifToRmq> for NeuronActor{
    
    type Result = ();
    fn handle(&mut self, msg: PublishNotifToRmq, ctx: &mut Self::Context) -> Self::Result {;

        // unpacking the notif data
        let PublishNotifToRmq { 
                exchange_name,
                exchange_type,
                routing_key,
                local_spawn,
                notif_data,
                encryptionConfig,

            } = msg.clone();
        
        let this = self.clone();
        
        let mut stringData = serde_json::to_string(&notif_data).unwrap();
        let mut scc = SecureCellConfig::default();
        let mut ruk = String::from(""); 

        let finalData = if encryptionConfig.is_some(){
                        
            let CryptoConfig{ secret, passphrase, unique_key } = encryptionConfig.clone().unwrap();
            let mut secure_cell_config = &mut wallexerr::misc::SecureCellConfig{
                secret_key: hex::encode(secret),
                passphrase: hex::encode(passphrase),
                data: vec![],
            };
            
            scc = secure_cell_config.clone();
            ruk = unique_key;

            // after calling encrypt method stringData has changed and contains the hex encrypted data
            stringData.encrypt(secure_cell_config);
            
            stringData // contains aes256 encrypted data in hex format

        } else{
            stringData
        };

        // spawn the future in the background into the given actor context thread
        // by doing this we're executing the future inside the actor thread since
        // every actor has its own thread of execution.
        if local_spawn{
            async move{
                this.publishToRmq(&finalData, &exchange_name, &routing_key, &exchange_type, scc, &ruk).await;
            }
            .into_actor(self) // convert the future into an actor future of type NotifBrokerActor
            .spawn(ctx); // spawn the future object into this actor context thread
        } else{ // spawn the future in the background into the tokio lightweight thread
            tokio::spawn(async move{
                this.publishToRmq(&finalData, &exchange_name, &routing_key, &exchange_type, scc, &ruk).await;
            });
        }
        
        return;
        
    }

}

impl ActixMessageHandler<ConsumeNotifFromRmq> for NeuronActor{
    
    type Result = ();
    fn handle(&mut self, msg: ConsumeNotifFromRmq, ctx: &mut Self::Context) -> Self::Result {

        // unpacking the consume data
        let ConsumeNotifFromRmq { 
                queue, 
                tag,
                exchange_name,
                routing_key,
                redis_cache_exp,
                local_spawn,
                store_in_db,
                decryptionConfig

            } = msg.clone(); // the unpacking pattern is always matched so if let ... is useless
        
        let this = self.clone();
        
        // spawn the future in the background into the given actor context thread
        // by doing this we're executing the future inside the actor thread since
        // every actor has its own thread of execution.
        if local_spawn{
            async move{
                this.consumeFromRmq(
                    redis_cache_exp, 
                    &tag, 
                    &queue, 
                    &routing_key, 
                    &exchange_name,
                    store_in_db,
                    decryptionConfig
                ).await;
            }
            .into_actor(self) // convert the future into an actor future of type NotifBrokerActor
            .spawn(ctx); // spawn the future object into this actor context thread
        } else{ // spawn the future in the background into the tokio lightweight thread
            tokio::spawn(async move{
                this.consumeFromRmq(
                    redis_cache_exp, 
                    &tag, 
                    &queue, 
                    &routing_key, 
                    &exchange_name,
                    store_in_db,
                    decryptionConfig
                ).await;
            });
        }
        return; // terminate the caller

    }

}