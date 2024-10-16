


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
use std::process::Output;

use task::Task;
use task::TaskStatus;
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
    pub cmd: String
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
    pub wallet: Option<wallexerr::misc::Wallet>, /* -- a cryptography indentifier for each neuron -- */
    pub metadata: Option<serde_json::Value>, /* -- json object contains the actual info of an object which is being carried by this neuron -- */
    pub internal_executor: InternalExecutor<Event>, /* -- eventloop sender and thread safe receiver, potentially we can use the actor msg sending pattern as well -- */
    pub transactions: Option<std::sync::Arc<tokio::sync::Mutex<Vec<Transaction>>>>, /* -- all neuron atomic transactions -- */
    pub internal_worker: Option<std::sync::Arc<tokio::sync::Mutex<Worker>>>, /* -- an internal lighthread worker -- */
    pub internal_locker: Option<std::sync::Arc<tokio::sync::Mutex<()>>>, /* -- internal locker -- */
    pub signal: std::sync::Arc<std::sync::Condvar>, /* -- the condition variable signal for this neuron -- */
    pub contract: Option<Contract>, // circom and noir for zk verifier contract (TODO: use crypter)
    pub state: u8
}


impl<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S> Runner<J, S>{
    pub async fn execute(&mut self){

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

    }

    pub async fn consumeFromRmq(&self, exp_seconds: u64,
        consumer_tag: &str, queue: &str, 
        binding_key: &str, exchange: &str,
        store_in_db: bool,
        decryptionConfig: Option<CryptoConfig>
    ){

            // this can be used for sending the received message to some ws server 
            let internal_executor = &self.internal_executor;

            // TODO
            // also send consumed data to the streamer channel of the internal executor 
            // use self.on() method
            // also handle rmq rpc and p2p in here 
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
        log::info!("execute...");
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
        // dependency injection and dynamic dispatching is done by:
        // Arc::pin(async move{}) or Box::pin(async move{})
        // Pin<Arc<dyn Trait>> or Pin<Box<dyn Trait>>
        // R: Future<Output=()> + Send + Sync + 'static
        use std::{pin::Pin, sync::Arc};
        type Callback<R> = Arc<dyn Fn() -> Pin<Arc<R>> + Send + Sync + 'static>;

        // use effect: interval task execution but use chan to fill the data
        // use effect takes an async closure
        struct useEffect<'valid, E, F, R>(pub F, &'valid [E]) where 
            E: Send + Sync + 'static + Clone,
            F: Fn() -> R + Send + Sync + 'static,
            R: std::future::Future<Output = ()> + Send + Sync + 'static;

        let router = String::from("/login"); // the effect on variable
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

        let BanCry { cmd } = msg;
        match cmd.as_str(){
            "executeTransaction" => {

            },
            _ => {

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