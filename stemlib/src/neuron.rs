


/*  ========================================================================================
    
    What is a Neuron?
        every neuron is an actor component object or a process or a light thread, the building 
        block of everything it's an smart object on its own which can communicate locally and 
        remotely with other neurons, every neuron contains a metadata field which carries 
        info about the actual object and informations that are being passed through the 
        synapses protocols. 

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

use std::error::Error;
use std::future::Future;
use std::process::Output;
use actor::workerthreadpool::sync::NoneAsyncThreadPool;
use interfaces::ServiceExt;
use task::Task;
use task::TaskStatus;
use tokio::spawn;
use tokio::sync::TryLockError;
use tokio::sync::MutexGuard;
use tokio::task::JoinHandle;
use tx::Transaction;
use wallexerr::misc::SecureCellConfig;
use interfaces::{Crypter};
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

// this is the internal executor, it's a job or task queue eventloop
// it has a buffer of Event objets, thread safe eventloop receiver and 
// a sender to send data to its channel, this is the backbone of actor
// worker objects they talk locally through the following pattern sicne
// they have isolated state there is no mutex at all mutating the state
// would be done through message sending pattern.
// threadpool executor eventloop: 
//      sender, 
//      Vec<JoinHanlde<()>>, 
//      while let Some(task) = rx.lock().await.recv().await{ spawn(task); }
#[derive(Clone)]
pub struct InternalExecutor<Event>{
    pub id: String, 
    pub buffer: Buffer<Event>,
    pub sender: tokio::sync::mpsc::Sender<Event>,
    // what does the eventloop do? it receives Event inside tokio::spawn and execute them in the background
    pub eventloop: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Event>>> 
}

// a job contains the io task 
pub struct Job<J: Clone, S>
where J: std::future::Future<Output = ()> + Send + Sync + 'static{
    pub id: String, 
    pub task: Task<J, S>
}

// Pin, Arc, Box are smart pointers Pin pin the pointee into the ram 
// at an stable position which won't allow the type to gets moved.
struct Task0<R, F: Fn() -> std::pin::Pin<Arc<R>> + Send + Sync + 'static> where 
R: std::future::Future<Output=()> + Send + Sync + 'static{
    pub job: Arc<F>
}

// a runner runs a job in its context
pub struct Runner<J: Clone, S>
where J: std::future::Future<Output = ()> + Send + Sync + 'static{
    pub id: String,
    pub job: Job<J, S>
}

// an event contains the offset in the cluster, execution status and the data
#[derive(Clone, Debug, Default)]
pub struct Event{
    pub data: EventData,
    pub status: EventStatus,
    pub offset: u64,
}

#[derive(Clone, Debug, Default)]
pub enum EventStatus{
    #[default]
    Committed,
    Executed,
    Halted
}

// a buffer contains a thread safe vector of Events
#[derive(Clone)]
pub struct Buffer<E>{ // eg: Buffer<Event>
    pub events: std::sync::Arc<tokio::sync::Mutex<Vec<E>>>,
    pub size: usize
}

#[derive(Clone, Debug)]
pub enum StreamError{
    Sender(String),
    Receiver(String)
}

#[derive(Clone, Debug)]
pub enum NeuronError{
    Runner(String),
    Job(String),
    Buffer(String),
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
pub struct UpdateState{
    pub new_state: u8
}

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct BanCry{
    pub cmd: String,
    pub tx: Transaction,
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
    pub local_spawn: bool, // either spawn in actor context or tokio threadpool
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
pub struct RmqConfig{
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct AppService{

}

#[derive(Clone)]
pub struct NeuronActor{
    pub rmqConfig: Option<RmqConfig>,
    pub wallet: Option<wallexerr::misc::Wallet>, /* -- a cryptography indentifier for each neuron -- */
    pub metadata: Option<serde_json::Value>, /* -- json object contains the actual info of an object which is being carried by this neuron -- */
    pub internal_executor: InternalExecutor<Event>, /* -- eventloop sender and thread safe receiver, potentially we can use the actor msg sending pattern as well -- */
    pub transactions: Option<std::sync::Arc<tokio::sync::Mutex<Vec<Transaction>>>>, /* -- all neuron atomic transactions -- */
    pub internal_worker: Option<std::sync::Arc<tokio::sync::Mutex<Worker>>>, /* -- an internal lighthread worker -- */
    pub internal_locker: Option<std::sync::Arc<tokio::sync::Mutex<()>>>, /* -- internal locker -- */
    pub internal_none_async_threadpool: std::sync::Arc<Option<NoneAsyncThreadPool>>, /* -- internal none async threadpool -- */
    pub signal: std::sync::Arc<std::sync::Condvar>, /* -- the condition variable signal for this neuron -- */
    pub dependency: std::sync::Arc<dyn ServiceExt<Model = Self>>, /* -- inject any type that impls the ServiceExt trait as dependency injection -- */
    pub contract: Option<Contract>, // circom and noir for zk verifier contract (TODO: use crypter)
    pub state: u8
}

impl<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S> Runner<J, S>{
    pub async fn execute(&mut self){
        let job = &self.job;
    }
}

impl Event{

    pub async fn process(&mut self){
        
    }

}

// ------------------------------
// implementnig the internal executor methods, it has an eventloop
// in its core which receives event coming from its channel constantly
// it then execute them along with the passed in callback in the run() method
impl InternalExecutor<Event>{
    
    pub fn new(buffer: Buffer<Event>) -> Self{
        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(100);
        let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
        Self { id: Uuid::new_v4().to_string(), buffer, sender: tx, eventloop: rx }
    }

    pub async fn spawn(&self, event: Event) -> Result<Self, tokio::sync::mpsc::error::SendError<Event>>{
        let sender = self.clone().sender;
        if let Err(e) = sender.send(event).await{
            return Err(e);
        }
        Ok(self.clone())
    }

    pub async fn run<F, R: std::future::Future<Output = ()> + Send + Sync + 'static, >(&self, callback: F)
    where F: Clone + Fn(Event, Option<StreamError>) -> R + Send + Sync + 'static{
        let get_rx = self.clone().eventloop;
        let mut rx = get_rx.try_lock();
        
        let cloned_callback = callback.clone();
        if rx.is_err(){
            let error = rx.unwrap_err();
            tokio::spawn(async move{
                cloned_callback(
                    Event::default(), 
                    Some(StreamError::Receiver(error.source().unwrap().to_string()))
                ).await; // calling callback with the passed in received event
            });
        } else{
            let mut actualRx = rx.unwrap(); 
            while let Some(event) = actualRx.recv().await{
                
                let event_for_first_tokio = event.clone();
                let cloned_event = event.clone();
                let cloned_callback = callback.clone();
    
                tokio::spawn(async move{
                    event_for_first_tokio.clone().process().await;
                });

                // executing the eventloop in the background
                tokio::spawn(async move{
                    cloned_callback(cloned_event, None).await; // calling callback with the passed in received event
                });
            }
        }

    }


}

impl NeuronActor{

    /* -------------------------------------------------------------
        since futures are object safe traits hence they have all traits 
        features we can pass them to the functions in an static or dynamic 
        dispatch way using Arc or Box or impl Future or even as the return 
        type of a closure trait method, so a future task would be one of 
        the following types: 

            1) Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>
            2) Pin<Arc<dyn Future<Output = ()> + Send + Sync + 'static>
            3) Arc<dyn Fn() -> Pin<Arc<R>> + Send + Sync + 'static> where R: Future<Output = ()> + Send + Sync + 'static
            4) Box<dyn Fn() -> Pin<Box<R>> + Send + Sync + 'static> where R: Future<Output = ()> + Send + Sync + 'static
            5) Arc<Mutex<dyn Fn() -> Pin<Arc<R>> + Send + Sync + 'static>> where R: Future<Output = ()> + Send + Sync + 'static
            6) job: F where F: Future<Output = ()> + Send + Sync + 'static
            7) param: impl Future<Output = ()> + Send + Sync + 'static

        NOTE: mutex requires the type to be Sized and since traits are not sized at 
        compile time we should annotate them with dyn keyword and put them behind a 
        pointer with a valid lifetime like Box or Arc smart pointers so for the 
        mutexed_job we must wrap the whole mutex inside an Arc or annotate it with 
        something like &'valid Mutex<dyn Fn() -> R + Send + Sync + 'static>
        the reason is that Mutex is a guard and not an smart pointer which can hanlde 
        an automatic pointer with lifetime, using of this syntax however would be in 
        a rare case when we want to mutate the closure!
    */
    // task is a closure that returns a future object 
    pub async fn runInterval<M, R, O>(&mut self, task: M, period: u64, retries: u8, timeout: u64)
    where M: Fn() -> R + Send + Sync + 'static,
            R: std::future::Future<Output = O> + Send + Sync + 'static,
    {
        let arced_task = std::sync::Arc::new(task);

        // the future task must gets executed within the period of timeout
        // otherwise it gets cancelled, dropped and aborted
        if timeout != 0{
            tokio::time::timeout(
                tokio::time::Duration::from_secs(timeout),
                arced_task()
            ).await;
        }        

        tokio::spawn(async move{
            let mut int = tokio::time::interval(tokio::time::Duration::from_secs(period));
            int.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            let mut attempts = 0;
            loop{
                if retries == 0{
                    continue;
                }
                if attempts >= retries{
                    break;
                }
                int.tick().await;
                arced_task().await;
                attempts += 1;
            }
        });

    }

    pub async fn publishToRmq(&self, data: &str, exchange: &str, routing_key: &str, 
        exchange_type: &str, secureCellConfig: SecureCellConfig, redisUniqueKey: &str){

            // TODO
            // use self.on() method
            // also handle rmq rpc and p2p in here
            // also handle file sharing with rmq  

            /*
                let zerlog_producer_actor = self.clone().zerlog_producer_actor;
                let this = self.clone();

                // these are must be converted into String first to make longer lifetime 
                // cause &str can't get moved into tokio spawn as its lifetime it's not 
                // static the tokio spawn lives longer than the &str and the &str gets 
                // dropped out of the ram once the function is finished with executing
                let exchange = exchange.to_string();
                let routing_key = routing_key.to_string();
                let exchange_type = exchange_type.to_string();
                let data = data.to_string();
                let redisUniqueKey = redisUniqueKey.to_string();

                tokio::spawn(async move{

                    let storage = this.clone().app_storage.clone();
                    let rmq_pool = storage.as_ref().unwrap().get_lapin_pool().await.unwrap();
                    let redis_pool = storage.as_ref().unwrap().get_redis_pool().await.unwrap();
                    

                    // store the config on redis for future validation in consumer
                    let mut redis_conn = redis_pool.get().await.unwrap(); // TODO - handle the error properly
                    let redisKey = format!("Rmq_SecureCellConfig-{}", redisUniqueKey);
                    let configString = serde_json::to_string(&secureCellConfig).unwrap();
                    let _: () = redis_conn.set(redisKey, configString).await.unwrap();


                    // trying to ge a connection from the pool
                    match rmq_pool.get().await{
                        Ok(pool) => {

                            // -ˋˏ✄┈┈┈┈ creating a channel in this thread
                            match pool.create_channel().await{
                                Ok(chan) => {

                                    let mut ex_options = ExchangeDeclareOptions::default();
                                    ex_options.auto_delete = true; // the exchange can only be deleted automatically if all bindings are deleted
                                    ex_options.durable = true; // durability is the ability to restore data on node shutdown

                                    // -ˋˏ✄┈┈┈┈ creating exchange
                                    /* 
                                        you should set the auto_delete flag to True when declaring the exchange. This will 
                                        automatically delete the exchange when all channels are done with it.
                                        Keep in mind that this means that it will stay as long as there is an active binding 
                                        to the exchange. If you delete the binding, or queue, the exchange will be deleted.
                                        if you need to keep the queue, but not the exchange you can remove the binding once 
                                        you are done publishing. This should automatically remove the exchange.
                                        so when all bindings (queues and exchanges) get deleted the exchange will be deleted.
                                    */
                                    match chan
                                        .exchange_declare(&exchange, {
                                            match exchange_type.as_str(){
                                                "fanout" => deadpool_lapin::lapin::ExchangeKind::Fanout,
                                                "direct" => deadpool_lapin::lapin::ExchangeKind::Direct,
                                                "headers" => deadpool_lapin::lapin::ExchangeKind::Headers,
                                                _ => deadpool_lapin::lapin::ExchangeKind::Topic,
                                            }
                                        },
                                        ex_options, FieldTable::default()
                                        )
                                        .await
                                        {
                                            Ok(ok) => {ok},
                                            Err(e) => {
                                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                let e_string = &e.to_string();
                                                let error_content = e_string.as_bytes().to_vec();
                                                let mut error_instance = HoopoeErrorResponse::new(
                                                    *constants::STORAGE_IO_ERROR_CODE, // error code
                                                    error_content, // error content
                                                    ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                    "NotifBrokerActor.exchange_declare", // method
                                                    Some(&zerlog_producer_actor)
                                                ).await;
                                                
                                            }

                                        };

                                    /* =================================================================================== */
                                    /* ================================ PRODUCING PROCESS ================================ */
                                    /* =================================================================================== */
                                    // async task: publish messages to exchange in the background in a lightweight thread of execution
                                    tokio::spawn(async move{
                                        
                                        // -ˋˏ✄┈┈┈┈ publishing to exchange from this channel,
                                        // later consumer bind its queue to this exchange and its
                                        // routing key so messages go inside its queue, later they 
                                        // can be consumed from the queue by the consumer.
                                        // in direct exchange ("") it has assumed that the queue
                                        // name is the same as the routing key name.
                                        use deadpool_lapin::lapin::options::BasicPublishOptions;
                                        let payload = data.as_bytes();
                                        match chan
                                            .basic_publish(
                                                &exchange, // messages go in there
                                                &routing_key, // the way that message gets routed to the queue based on a unique routing key
                                                BasicPublishOptions::default(),
                                                payload, // this is the ProduceNotif data,
                                                BasicProperties::default().with_content_type("application/json".into()),
                                            )
                                            .await
                                            {
                                                Ok(pc) => {
                                                    let get_confirmation = pc.await;
                                                    let Ok(confirmation) = get_confirmation else{
                                                        use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                        let error_content_ = get_confirmation.unwrap_err();
                                                        let e_string = &error_content_.to_string();
                                                        let error_content = e_string.as_bytes().to_vec();
                                                        let mut error_instance = HoopoeErrorResponse::new(
                                                            *constants::STORAGE_IO_ERROR_CODE, // error code
                                                            error_content, // error content
                                                            ErrorKind::Storage(crate::error::StorageError::Rmq(error_content_)), // error kind
                                                            "NotifBrokerActor.get_confirmation", // method
                                                            Some(&zerlog_producer_actor)
                                                        ).await;

                                                        return; // needs to terminate the caller in let else pattern
                                                    };

                                                    if confirmation.is_ack(){
                                                        log::info!("publisher confirmation is acked");
                                                    }

                                                },
                                                Err(e) => {
                                                    use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                    let error_content = &e.to_string();
                                                    let error_content = error_content.as_bytes().to_vec();
                                                    let mut error_instance = HoopoeErrorResponse::new(
                                                        *constants::STORAGE_IO_ERROR_CODE, // error code
                                                        error_content, // error content
                                                        ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                        "NotifBrokerActor.basic_publish", // method
                                                        Some(&zerlog_producer_actor)
                                                    ).await;

                                                }
                                            }
                                    });

                                },
                                Err(e) => {
                                    use crate::error::{ErrorKind, HoopoeErrorResponse};
                                    let error_content = &e.to_string();
                                    let error_content = error_content.as_bytes().to_vec();
                                    let mut error_instance = HoopoeErrorResponse::new(
                                        *constants::STORAGE_IO_ERROR_CODE, // error code
                                        error_content, // error content
                                        ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                        "NotifBrokerActor.create_channel", // method
                                        Some(&zerlog_producer_actor)
                                    ).await;

                                }
                            }
                        },
                        Err(e) => {

                            use crate::error::{ErrorKind, HoopoeErrorResponse};
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();
                            let mut error_instance = HoopoeErrorResponse::new(
                                *constants::STORAGE_IO_ERROR_CODE, // error code
                                error_content, // error content
                                ErrorKind::Storage(crate::error::StorageError::RmqPool(e)), // error kind
                                "NotifBrokerActor.produce_pool", // method
                                Some(&zerlog_producer_actor)
                            ).await;

                        }
                    };
                    
                }); 
            */

    }

    pub async fn consumeFromRmq(&self,
        consumer_tag: &str, queue: &str, 
        binding_key: &str, exchange: &str,
        decryptionConfig: Option<CryptoConfig>
    ){

            // this can be used for sending the received message to some ws server 
            let internal_executor = &self.internal_executor;

            // TODO
            // also send consumed data to the streamer channel of the internal executor 
            // use self.on() method
            // also handle rmq rpc and p2p in here
            // also handle file sharing with rmq 

            /* 
                let storage = self.app_storage.clone();
                let rmq_pool = storage.as_ref().unwrap().get_lapin_pool().await.unwrap();
                let redis_pool = storage.unwrap().get_redis_pool().await.unwrap();
                let notif_mutator_actor = self.notif_mutator_actor.clone();
                let zerlog_producer_actor = self.clone().zerlog_producer_actor;
                let notif_data_sender = self.notif_broker_ws_sender.clone();
                
                match rmq_pool.get().await{
                    Ok(conn) => {

                        let create_channel = conn.create_channel().await;
                        match create_channel{
                            Ok(chan) => {

                                let mut q_options = QueueDeclareOptions::default();
                                q_options.durable = true; // durability is the ability to restore data on node shutdown

                                // -ˋˏ✄┈┈┈┈ making a queue inside the broker per each consumer, 
                                let create_queue = chan
                                    .queue_declare(
                                        &queue,
                                        q_options,
                                        FieldTable::default(),
                                    )
                                    .await;

                                let Ok(q) = create_queue else{
                                    let e = create_queue.unwrap_err();
                                    use crate::error::{ErrorKind, HoopoeErrorResponse};
                                    let error_content = &e.to_string();
                                    let error_content = error_content.as_bytes().to_vec();
                                    let mut error_instance = HoopoeErrorResponse::new(
                                        *constants::STORAGE_IO_ERROR_CODE, // error code
                                        error_content, // error content
                                        ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                        "NotifBrokerActor.queue_declare", // method
                                        Some(&zerlog_producer_actor)
                                    ).await;
                                    return; // terminate the caller
                                };

                                // binding the queue to the exchange routing key to receive messages it interested in
                                /* -ˋˏ✄┈┈┈┈ 
                                    if the exchange is not direct or is fanout or topic we should bind the 
                                    queue to the exchange to consume the messages from the queue. binding 
                                    the queue to the passed in exchange, if the exchange is direct every 
                                    queue that is created is automatically bounded to it with a routing key 
                                    which is the same as the queue name, the direct exchange is "" and 
                                    rmq doesn't allow to bind any queue to that manually
                                */
                                match chan
                                    .queue_bind(q.name().as_str(), &exchange, &binding_key, 
                                        QueueBindOptions::default(), FieldTable::default()
                                    )
                                    .await
                                    { // trying to bind the queue to the exchange
                                        Ok(_) => {},
                                        Err(e) => {
                                            use crate::error::{ErrorKind, HoopoeErrorResponse};
                                            let error_content = &e.to_string();
                                            let error_content = error_content.as_bytes().to_vec();
                                            let mut error_instance = HoopoeErrorResponse::new(
                                                *constants::STORAGE_IO_ERROR_CODE, // error code
                                                error_content, // error content
                                                ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                "NotifBrokerActor.queue_bind", // method
                                                Some(&zerlog_producer_actor)
                                            ).await;
                                            return; // terminate the caller
                                        }
                                    }

                                // since &str is not lived long enough to be passed to the tokio spawn
                                // if it was static it would be good however we're converting them to
                                // String to pass the String version of them to the tokio spawn scope
                                let cloned_consumer_tag = consumer_tag.to_string();
                                let cloned_queue = queue.to_string();
                                let cloned_notif_data_sender_channel = notif_data_sender.clone();

                                /* =================================================================================== */
                                /* ================================ CONSUMING PROCESS ================================ */
                                /* =================================================================================== */
                                // start consuming in the background in a lightweight thread of execution
                                // receiving is considered to be none blocking which won't block the current thread. 
                                tokio::spawn(async move{

                                    // try to create the secure cell config and 
                                    // do passphrase and secret key validation logic before
                                    // consuming messages
                                    let secure_cell_config_uniqueKey = if let Some(mut config) = decryptionConfig.clone(){
                                
                                        config.secret = hex::encode(config.secret);
                                        config.passphrase = hex::encode(config.passphrase);

                                        // return the loaded instance from redis
                                        let secureCellConfig = SecureCellConfig{
                                            secret_key: config.clone().secret,
                                            passphrase: config.clone().passphrase,
                                            data: vec![],
                                        };

                                        (secureCellConfig, config.unique_key)

                                    } else{
                                        (SecureCellConfig::default(), String::from(""))
                                    };

                                    // -ˋˏ✄┈┈┈┈ consuming from the queue owned by this consumer
                                    match chan
                                        .basic_consume(
                                            /* -ˋˏ✄┈┈┈┈
                                                the queue that is bounded to the exchange to receive messages based on the routing key
                                                since the queue is already bounded to the exchange and its routing key, it only receives 
                                                messages from the exchange that matches and follows the passed in routing pattern like:
                                                message routing key "orders.processed" might match a binding with routing key "orders.#
                                                if none the messages follow the pattern then the queue will receive no message from the 
                                                exchange based on that pattern! although messages are inside the exchange.
                                            */
                                            &cloned_queue, 
                                            &cloned_consumer_tag, // custom consumer name
                                            BasicConsumeOptions::default(), 
                                            FieldTable::default()
                                        )
                                        .await
                                    {
                                        Ok(mut consumer) => {
                                            // stream over consumer to receive data from the queue
                                            while let Some(delivery) = consumer.next().await{
                                                match delivery{
                                                    Ok(delv) => {

                                                        /* WE SHOULD DO THE VALIDATION IN EVERY ITERATION */
                                                        let mut secure_cell_config = secure_cell_config_uniqueKey.0.clone();
                                                        let redisUniqueKey = secure_cell_config_uniqueKey.clone().1;
                                                        let mut redis_conn = redis_pool.get().await.unwrap(); // TODO - handle the error properly
                                                        let redisKey = format!("Rmq_SecureCellConfig-{}", redisUniqueKey);
                                                        let isKeyThere: bool = redis_conn.exists(&redisKey).await.unwrap();
                                                        if isKeyThere{
                                                            let getConfig: String = redis_conn.get(redisKey).await.unwrap();
                                                            let mut redisConfig = serde_json::from_str::<SecureCellConfig>(&getConfig).unwrap();
                                                            // validating...
                                                            if 
                                                                redisConfig.passphrase != secure_cell_config.passphrase || 
                                                                redisConfig.secret_key != secure_cell_config.secret_key
                                                                {
                                                                    log::error!("invalid secure cell config, CHANNEL IS NOT SAFE!");
                                                                    return;
                                                            }
                                                            
                                                        } else{
                                                            log::error!("Rmq configs are not set properly on Redis");
                                                        }

                                                        log::info!("[*] received delivery from queue#<{}>", q.name());
                                                        let consumedBuffer = delv.data.clone();
                                                        let hexed_data = std::str::from_utf8(&consumedBuffer).unwrap();
                                                        let mut payload = hexed_data.to_string();

                                                        let cloned_notif_data_sender_channel = cloned_notif_data_sender_channel.clone();
                                                        let redis_pool = redis_pool.clone();
                                                        let notif_mutator_actor = notif_mutator_actor.clone();
                                                        let cloned_zerlog_producer_actor = zerlog_producer_actor.clone();

                                                        // handle the message in the background light thread asyncly and concurrently
                                                        // by spawning a light thread for the async task
                                                        tokio::spawn(async move{

                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>
                                                            // ===>>>===>>>===>>>===>>>===>>> data decryption logic ===>>>===>>>===>>>===>>>===>>>
                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>
                                                            // if we have a config means the data has been encrypted
                                                            let finalData = if 
                                                                !secure_cell_config.clone().secret_key.is_empty() && 
                                                                !secure_cell_config.clone().passphrase.is_empty(){
                                                                    
                                                                // after calling decrypt method has changed and now contains raw string
                                                                // payload must be mutable since the method mutate the content after decrypting
                                                                payload.decrypt(&mut secure_cell_config);

                                                                // payload is now raw string which can be decoded into the NotifData structure
                                                                payload

                                                            } else{
                                                                // no decryption config is needed, just return the raw data
                                                                // there would be no isse with decoding this into NotifData
                                                                log::error!("secure_cell_config is empty, data is not encrypted");
                                                                payload
                                                            };
                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>
                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>
                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>

                                                            // either decrypted or the raw data as string
                                                            log::info!("[*] received data: {}", finalData);
                                                            
                                                            // decoding the string data into the NotifData structure (convention)
                                                            let get_notif_event = serde_json::from_str::<NotifData>(&finalData);
                                                            match get_notif_event{
                                                                Ok(notif_event) => {

                                                                    log::info!("[*] deserialized data: {:?}", notif_event);

                                                                    // =================================================================
                                                                    /* -------------------------- send to mpsc channel for ws streaming
                                                                    // =================================================================
                                                                        this is the most important part in here, we're slightly sending the data
                                                                        to downside of a jobq mpsc channel, the receiver eventloop however will 
                                                                        receive data in websocket handler which enables us to send realtime data 
                                                                        received from RMQ to the browser through websocket server: RMQ over websocket
                                                                        once we receive the data from the mpsc channel in websocket handler we'll 
                                                                        send it to the browser through websocket channel.
                                                                    */
                                                                    if let Err(e) = cloned_notif_data_sender_channel.send(finalData).await{
                                                                        log::error!("can't send notif data to websocket channel due to: {}", e.to_string());
                                                                    }

                                                                    // =============================================================================
                                                                    // ------------- if the cache on redis flag was activated we then store on redis
                                                                    // =============================================================================
                                                                    if exp_seconds != 0{
                                                                        match redis_pool.get().await{
                                                                            Ok(mut redis_conn) => {

                                                                                // key: String::from(notif_receiver.id) | value: Vec<NotifData>
                                                                                let redis_notif_key = format!("notif_owner:{}", &notif_event.receiver_info);
                                                                                
                                                                                // -ˋˏ✄┈┈┈┈ extend notifs
                                                                                let get_events: RedisResult<String> = redis_conn.get(&redis_notif_key).await;
                                                                                let events = match get_events{
                                                                                    Ok(events_string) => {
                                                                                        let get_messages = serde_json::from_str::<Vec<NotifData>>(&events_string);
                                                                                        match get_messages{
                                                                                            Ok(mut messages) => {
                                                                                                messages.push(notif_event.clone());
                                                                                                messages
                                                                                            },
                                                                                            Err(e) => {
                                                                                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                                                                let error_content = &e.to_string();
                                                                                                let error_content_ = error_content.as_bytes().to_vec();
                                                                                                let mut error_instance = HoopoeErrorResponse::new(
                                                                                                    *constants::CODEC_ERROR_CODE, // error code
                                                                                                    error_content_, // error content
                                                                                                    ErrorKind::Codec(crate::error::CodecError::Serde(e)), // error kind
                                                                                                    "NotifBrokerActor.decode_serde_redis", // method
                                                                                                    Some(&cloned_zerlog_producer_actor)
                                                                                                ).await;
                                                                                                return; // terminate the caller
                                                                                            }
                                                                                        }
                                                                        
                                                                                    },
                                                                                    Err(e) => {
                                                                                        // we can't get the key means this is the first time we're creating the key
                                                                                        // or the key is expired already, we'll create a new key either way and put
                                                                                        // the init message in there.
                                                                                        let init_message = vec![
                                                                                            notif_event.clone()
                                                                                        ];

                                                                                        init_message

                                                                                    }
                                                                                };

                                                                                // -ˋˏ✄┈┈┈┈ caching the notif event in redis with expirable key
                                                                                // chaching in redis is an async task which will be executed 
                                                                                // in the background with an expirable key
                                                                                tokio::spawn(async move{
                                                                                    let events_string = serde_json::to_string(&events).unwrap();
                                                                                    let is_key_there: bool = redis_conn.exists(&redis_notif_key.clone()).await.unwrap();
                                                                                    if is_key_there{ // update only the value
                                                                                        let _: () = redis_conn.set(&redis_notif_key.clone(), &events_string).await.unwrap();
                                                                                    } else{ // initializing a new expirable key containing the new notif data
                                                                                        /*
                                                                                            make sure you won't get the following error:
                                                                                            called `Result::unwrap()` on an `Err` value: MISCONF: Redis is configured to 
                                                                                            save RDB snapshots, but it's currently unable to persist to disk. Commands that
                                                                                            may modify the data set are disabled, because this instance is configured to 
                                                                                            report errors during writes if RDB snapshotting fails (stop-writes-on-bgsave-error option). 
                                                                                            Please check the Redis logs for details about the RDB error. 
                                                                                            SOLUTION: restart redis :)
                                                                                        */
                                                                                        let _: () = redis_conn.set_ex(&redis_notif_key.clone(), &events_string, exp_seconds).await.unwrap();
                                                                                    }
                                                                                });

                                                                            },
                                                                            Err(e) => {
                                                                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                                                let error_content = &e.to_string();
                                                                                let error_content_ = error_content.as_bytes().to_vec();
                                                                                let mut error_instance = HoopoeErrorResponse::new(
                                                                                    *constants::STORAGE_IO_ERROR_CODE, // error code
                                                                                    error_content_, // error content
                                                                                    ErrorKind::Storage(crate::error::StorageError::RedisPool(e)), // error kind
                                                                                    "NotifBrokerActor.redis_pool", // method
                                                                                    Some(&cloned_zerlog_producer_actor)
                                                                            ).await;
                                                                                return; // terminate the caller
                                                                            }
                                                                        }
                                                                    }

                                                                    // =============================================================================
                                                                    // ------------- if the store in db flag was activated we then store in database
                                                                    // =============================================================================
                                                                    if store_in_db{
                                                                        // -ˋˏ✄┈┈┈┈ store notif in db by sending message to the notif mutator actor worker
                                                                        // sending StoreNotifEvent message to the notif event mutator actor
                                                                        // spawning the async task of storing data in db in the background
                                                                        // worker of lightweight thread of execution using tokio threadpool
                                                                        tokio::spawn(
                                                                            {
                                                                                let cloned_message = notif_event.clone();
                                                                                let cloned_mutator_actor = notif_mutator_actor.clone();
                                                                                let zerlog_producer_actor = cloned_zerlog_producer_actor.clone();
                                                                                async move{
                                                                                    match cloned_mutator_actor
                                                                                        .send(StoreNotifEvent{
                                                                                            message: cloned_message,
                                                                                            local_spawn: true
                                                                                        })
                                                                                        .await
                                                                                        {
                                                                                            Ok(_) => { () },
                                                                                            Err(e) => {
                                                                                                let source = &e.to_string(); // we know every goddamn type implements Error trait, we've used it here which allows use to call the source method on the object
                                                                                                let err_instance = crate::error::HoopoeErrorResponse::new(
                                                                                                    *MAILBOX_CHANNEL_ERROR_CODE, // error hex (u16) code
                                                                                                    source.as_bytes().to_vec(), // text of error source in form of utf8 bytes
                                                                                                    crate::error::ErrorKind::Actor(crate::error::ActixMailBoxError::Mailbox(e)), // the actual source of the error caused at runtime
                                                                                                    &String::from("NotifBrokerActor.notif_mutator_actor.send"), // current method name
                                                                                                    Some(&zerlog_producer_actor)
                                                                                                ).await;
                                                                                                return;
                                                                                            }
                                                                                        }
                                                                                }
                                                                            }
                                                                        );
                                                                    }

                                                                },
                                                                Err(e) => {
                                                                    log::error!("[!] can't deserialized into NotifData struct");
                                                                    use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                                    let error_content = &e.to_string();
                                                                    let error_content_ = error_content.as_bytes().to_vec();
                                                                    let mut error_instance = HoopoeErrorResponse::new(
                                                                        *constants::CODEC_ERROR_CODE, // error code
                                                                        error_content_, // error content
                                                                        ErrorKind::Codec(crate::error::CodecError::Serde(e)), // error kind
                                                                        "NotifBrokerActor.decode_serde", // method
                                                                        Some(&cloned_zerlog_producer_actor)
                                                                    ).await;
                                                                    return; // terminate the caller
                                                                }
                                                            }
                                                        });

                                                        // acking here means we have processed payload successfully
                                                        match delv.ack(BasicAckOptions::default()).await{
                                                            Ok(ok) => { /* acked */ },
                                                            Err(e) => {
                                                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                                let error_content = &e.to_string();
                                                                let error_content = error_content.as_bytes().to_vec();
                                                                let mut error_instance = HoopoeErrorResponse::new(
                                                                    *constants::STORAGE_IO_ERROR_CODE, // error code
                                                                    error_content, // error content
                                                                    ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                                    "NotifBrokerActor.consume_ack", // method
                                                                    Some(&zerlog_producer_actor)
                                                                ).await;
                                                                return; // terminate the caller
                                                            }
                                                        }
                            
                                                    },
                                                    Err(e) => {
                                                        use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                        let error_content = &e.to_string();
                                                        let error_content = error_content.as_bytes().to_vec();
                                                        let mut error_instance = HoopoeErrorResponse::new(
                                                            *constants::STORAGE_IO_ERROR_CODE, // error code
                                                            error_content, // error content
                                                            ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                            "NotifBrokerActor.consume_getting_delivery", // method
                                                            Some(&zerlog_producer_actor)
                                                        ).await;
                                                        return; // terminate the caller 
                                                    }
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            use crate::error::{ErrorKind, HoopoeErrorResponse};
                                            let error_content = &e.to_string();
                                            let error_content = error_content.as_bytes().to_vec();
                                            let mut error_instance = HoopoeErrorResponse::new(
                                                *constants::STORAGE_IO_ERROR_CODE, // error code
                                                error_content, // error content
                                                ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                "NotifBrokerActor.consume_basic_consume", // method
                                                Some(&zerlog_producer_actor)
                                            ).await;
                                            return; // terminate the caller 
                                        }
                                    }

                                });

                            },
                            Err(e) => {
                                the
                            },
                            Err(e) => {
                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                let error_content = &e.to_string();
                                let error_content = error_content.as_bytes().to_vec();
                                let mut error_instance = HoopoeErrorResponse::new(
                                    *constants::STORAGE_IO_ERROR_CODE, // error code
                                    error_content, // error content
                                    ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                    "NotifBrokerActor.consume_create_channel", // method
                                    Some(&zerlog_producer_actor)
                                ).await;
                                return; // terminate the caller   
                            }
                        }

                    },
                    Err(e) => {

                    }
                    Err(e) => {
                        use crate::error::{ErrorKind, HoopoeErrorResponse};
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();
                        let mut error_instance = HoopoeErrorResponse::new(
                            *constants::STORAGE_IO_ERROR_CODE, // error code
                            error_content, // error content
                            ErrorKind::Storage(crate::error::StorageError::RmqPool(e)), // error kind
                            "NotifBrokerActor.consume_pool", // method
                            Some(&zerlog_producer_actor)
                        ).await;
                        return; // terminate the caller
                    }
                };
            */
            
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

    pub async fn executekMeIntervally(&mut self){
        log::info!("executing...");
    }

    /* -------------------------------------------------------
        an example function to show how we can execute jobs 
        using an eventloop in the background thread.
        a job can be arced version of a closure trait which 
        returns a future object as its body or a pinned version
        of the whole closure trait, a pinned version
        of a boxed of a future trait object or binding the 
        generic to the closure trait which returns a future
        object as its body.
        for both Arc<dyn Trait> or Box<dyn Trait> the trait 
        must be object safe.
    */
    pub async fn AllInOneInternalExecutor<J, R>(&mut self, job: J) where J: Fn() -> R + Send + Sync +'static, 
    R: std::future::Future<Output = ()> + Send + Sync + 'static{

        trait Interface{}
        struct Job{}
        impl Interface for Job{}

        // arcing like boxing the object safe trait use for thread safe dynamic dispatching and dep injection
        let interfaces: std::sync::Arc<dyn Interface> = std::sync::Arc::new(Job{});

        // boxing trait for dynamic dispatching and dep ibjection
        let interfaces1: Box<dyn Interface> = Box::new(Job{});

        // arcing the mutexed object safe trait for dynamic dispatching, dep injection and mutating it safely
        let interfaces2: std::sync::Arc<tokio::sync::Mutex<dyn Interface>> = std::sync::Arc::new(tokio::sync::Mutex::new(Job{}));

        // we can't have the following cause in order to pin a type the size must be known
        // at compile time thus pin can't have an object safe trait for pinning it at stable 
        // position inside the ram without knowing the size 
        // let interfaces3: std::sync::Arc<std::pin::Pin<dyn Interface>> = std::sync::Arc::pin(Job{});
        // we can't pin a trait directly into the ram, instead we must pin their
        // boxed or arced version like Arc<dyn Future> or Box<dyn Future> into 
        // the ram: Arc::pin(async move{}) or Box::pin(async move{})

        // job however is an async io task which is a future object
        type IoJob<R> = Arc<dyn Fn() -> R + Send + Sync + 'static>;
        type IoJob1<R> = std::pin::Pin<Arc<dyn Fn() -> R + Send + Sync + 'static>>;
        type IoJob2 = std::pin::Pin<Arc<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>;
        type IoJob3 = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>;
        type Tasks = Vec<std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>>;
        type Task = Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>;
        type Task1 = std::sync::Arc<dyn FnOnce() 
            -> std::pin::Pin<std::sync::Arc<dyn Future<Output = ()> + Send + Sync + 'static>>>;
        // dependency injection and dynamic dispatching is done by:
        // Arc::pin(async move{}) or Box::pin(async move{})
        // Pin<Arc<dyn Trait>> or Pin<Box<dyn Trait>>
        // R: Future<Output=()> + Send + Sync + 'static
        use std::{pin::Pin, sync::Arc};
        type Callback<R> = Arc<dyn Fn() -> Pin<Arc<R>> + Send + Sync + 'static>; // an arced closure trait which returns an arced pinned future trait object

        async fn push<C, R>(topic: &str, event: Event, payload: &[u8], c: C) where 
        C: Fn() -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send + Sync + 'static{
            let arcedCallback = Arc::new(c);
            // spawn the task in the background thread, don't 
            // await on it let the task spawned into the tokio 
            // thread gets awaited by the tokio ligh thread
            spawn(async move{
                arcedCallback().await;
            });
        }

        // use effect: interval task execution but use chan to fill the data
        // use effect takes an async closure
        struct useEffect<'valid, E, F, R>(pub F, &'valid [E]) where 
            E: Send + Sync + 'static + Clone,
            F: Fn() -> R + Send + Sync + 'static,
            R: std::future::Future<Output = ()> + Send + Sync + 'static;

        let router = String::from("/login"); // this variable gets effected
        self.runInterval(move || {
            let clonedRouter = router.clone();
            let clonedRouter1 = router.clone();
            async move{
                let state = useEffect(
                    move || {
                        let clonedRouter1 = clonedRouter1.clone();
                        async move{

                            // some condition to affect on router
                            // ...

                            use tokio::{spawn, sync::mpsc::channel};
                            let (tx, rx) = channel(100);
                            spawn(async move{
                                tx.send(clonedRouter1).await;
                            });
            
                        }
                    }, &[clonedRouter]
                );
            }
        }, 10, 10, 0).await;

        
        /* the error relates to compiling traits with static dispatch approach:
        mismatched types
            expected `async` block `{async block@stemlib/src/neuron.rs:661:27: 661:37}`
            found `async` block `{async block@stemlib/src/neuron.rs:661:41: 661:51}`
            no two async blocks, even if identical, have the same type
            consider pinning your async block and casting it to a trait object 
        */
        // let tasks1 = vec![async move{}, async move{}];
        // vector of async tasks pinned into the ram, every async move{} considerred to be a different type
        // which is kninda static dispatch or impl Future logic, boxing their pinned object into the ram 
        // bypass this allows us to access multiple types through a single interface using dynamic dispatch.
        // since the type in dynamic dispatch will be specified at runtime. 
        let tasks: Tasks = vec![Box::pin(async move{}), Box::pin(async move{})]; // future as separate objects must gets pinned into the ram  
        let futDependency: Task = Box::new(async move{});

        let arcedJob = std::sync::Arc::new(job);
        let (tx, mut eventloop) = 
            tokio::sync::mpsc::channel::<std::sync::Arc<J>>(100);

        // schedule a job to pop it out from the eventloop queue to gets executed in the background 
        // thread or send a message to the actor worker thread to execute a job like start consuming 
        // or receiving streaming in a light thread like stream.on or while let some

        // step1)
        // execute an interval in the background thread, this would
        // execute a sending task every 5 seconds in the background thread.
        let mut this = self.clone();
        tokio::spawn(async move{
            // run a task intervally in the background light thread
            this.runInterval(move || {
                // we must clone them inside the closure body 
                // before sending them inside the tokio spawn
                let cloned_tx = tx.clone();
                let arcedJob = arcedJob.clone();
                // this is the closure body which is of type future object
                async move{
                    cloned_tx.send(arcedJob).await;
                }
            }, 5, 0, 0).await;
        });

        // step2)
        // start receiving or streaming over the eventloop 
        // of the channel to receive jobs
        tokio::spawn(async move{
            while let Some(job) = eventloop.recv().await{
                tokio::spawn(async move{
                    job().await
                });
            }
        });

    }

}

impl Actor for NeuronActor{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        
        ctx.run_interval(std::time::Duration::from_secs(10), |actor, ctx|{

            let mut this = actor.clone(); 
            tokio::spawn(async move{
                this.executekMeIntervally().await;
            });

        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        log::error!("neuron actor has stopped");        
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

        let this = self.clone();

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
                local_spawn,
                decryptionConfig

            } = msg.clone(); // the unpacking pattern is always matched so if let ... is useless
        
        let this = self.clone();
        
        // spawn the future in the background into the given actor context thread
        // by doing this we're executing the future inside the actor thread since
        // every actor has its own thread of execution.
        if local_spawn{
            async move{
                this.consumeFromRmq(
                    &tag, 
                    &queue, 
                    &routing_key, 
                    &exchange_name,
                    decryptionConfig
                ).await;
            }
            .into_actor(self) // convert the future into an actor future of type NotifBrokerActor
            .spawn(ctx); // spawn the future object into this actor context thread
        } else{ // spawn the future in the background into the tokio lightweight thread
            tokio::spawn(async move{
                this.consumeFromRmq(
                    &tag, 
                    &queue, 
                    &routing_key, 
                    &exchange_name,
                    decryptionConfig
                ).await;
            });
        }
        return; // terminate the caller

    }

}

// handler for handling send update state message to this actor to update the state
// this ensures the actor isolation stays safe and secure cause there is no direct 
// mutating it's handled only by sending message to the actor and the actor receives
// it from its mailbox and runs it accordingly. 
impl ActixMessageHandler<UpdateState> for NeuronActor{
    type Result = ();
    fn handle(&mut self, msg: UpdateState, ctx: &mut Self::Context) -> Self::Result {
        self.state = msg.new_state;
    }
}

impl ActixMessageHandler<BanCry> for NeuronActor{
    type Result = ();
    fn handle(&mut self, msg: BanCry, ctx: &mut Self::Context) -> Self::Result{

        let BanCry { cmd, tx } = msg;
        match cmd.as_str(){
            "executeTransaction" => {
                let this = self.clone();
                let task = async move{
                    let getNeuronTransactions = &this.transactions;
                    if getNeuronTransactions.is_some(){
                        let neuronTransactions = getNeuronTransactions.as_ref().unwrap();
                        let mut lockedNeuronTransactions = neuronTransactions.lock().await;
                        let tx = Transaction::new(
                            tx,
                            Uuid::new_v4().to_string().as_str(), 
                            100.0, 
                            "0x01", 
                            "0x00", 
                            2.5, 
                            String::from("some data").as_bytes()
                        ).await;
                        (*lockedNeuronTransactions).push(tx);
                    } else{
                        log::error!("[!] actor has no transactions");
                    }
                };
                spawn(task); // spawn the task of pushing tx into the neuron transactions in the background thread
            },
            _ => {
                log::error!("[!] invalid command for bancry!");
            }
        }
        
    }
}

impl Drop for NeuronActor{
    fn drop(&mut self) {
        let this = self.clone();
        tokio::spawn(async move{
            let getInternalWorker = &this.internal_worker;
            if getInternalWorker.is_some(){
                let mut internalWorker = getInternalWorker.clone().unwrap();
                let mut unloackedInternalWorker = internalWorker.lock().await;
                (*unloackedInternalWorker).thread.abort(); // abort the thread handler, the tokio threads are future based thread so we can easily abort them
            }
        });
    }
}