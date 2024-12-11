

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::sync::{Arc, Condvar};
use std::thread::{self, JoinHandle};
use crate::*;
use actix_web::web::to;
use actix_web::Handler;
use clap::error;
use deadpool_redis::redis::AsyncCommands;
use interfaces::{Crypter, ObjectStorage};
use salvo::Router;
use sha2::digest::generic_array::arr;
use sha2::digest::Output;
use stemlib::dto::{Neuron, TransmissionMethod, *};
use stemlib::messages::*;
use stemlib::interfaces::OnionStream;
use tokio::io::AsyncReadExt;
use wallexerr::misc::SecureCellConfig; // import the interface to use the on() method on the Neuron instance
use stemlib::dsl::*;



#[tokio::test]
pub async fn testNeuronActor(){

    use deadpool_redis::{Config as DeadpoolRedisConfig, Runtime as DeadPoolRedisRuntime};
    let redisPassword = "geDteDd0Ltg2135FJYQ6rjNYHYkGQa70";
    let redisHost = "0.0.0.0";
    let redisPort = 6379;
    let redisUsername = "";
    let redis_conn_url = if !redisPassword.is_empty(){
        format!("redis://:{}@{}:{}", redisPassword, redisHost, redisPort)
    } else if !redisPassword.is_empty() && !redisUsername.is_empty(){
        format!("redis://{}:{}@{}:{}", redisUsername, redisPassword, redisHost, redisPort)
    } else{
        format!("redis://{}:{}", redisHost, redisPort)
    };
    let redis_pool_cfg = DeadpoolRedisConfig::from_url(&redis_conn_url);
    let redis_pool = Arc::new(redis_pool_cfg.create_pool(Some(DeadPoolRedisRuntime::Tokio1)).unwrap()); 

    let Ok(redisConn) = redis_pool.get().await else{
        panic!("can't get redis connection from the pool");
    };
    
    // redisConn must be mutable and since we're moving it into another thread 
    // we should make it safe to gets moved and mutated using Arc and Mutex
    let arcedRedisConn = Arc::new(tokio::sync::Mutex::new(redisConn));
    let clonnedRedisConn = arcedRedisConn.clone();

    let mut neuron1 = Neuron::new(100, "Neuron1").await;
    let mut neuron = Neuron::new(100, "Neuron2").await;
    
    let getNeuronWallet = neuron.wallet.as_ref().unwrap();
    let getNeuronId = neuron.peerId.to_base58();

    let neuronWallet = neuron.wallet.as_ref().unwrap();
    let executor = neuron.internal_executor.clone();

    /* --------------------------
        execution thread process for solving future:
        await on async task suspend it to get the result but won't block thread 
        means the light thread can continue executing other tasks
        future objects are being done in the background awaiting on or polling  
        them tells runtime that we need the result if the future was ready he sends the 
        result to the caller otherwise it forces the thread to get another task 
        from the eventloop to execute it meanwhile the future is being solved, 
        this allows to execute tasks in a none blocking manner 
    */
    neuron.runInterval(|| async move{
        println!("i'm running every 10 seconds, with retries of 12 and timeout 0");
    }, 10, 12, 0).await;

    // -----------------------------------------------------------------
    // ------- sending message through actor mailbox eventloop receiver:
    // by default actors run on the system arbiter thread using 
    // its eventloop, we can run multiple instances of an actor 
    // in parallel with SyncArbiter. 
    // actor mailbox is the eventloop receiver of actor jobq mpsc channel
    // which receive messages and execute them in a light thread or process 

    // starting the neuron actor 
    let neuronComponentActor = neuron.clone().start();
    
    // sending update state message
    neuronComponentActor.send(
        UpdateState{new_state: 1}
    ).await;

    // send shutdown message to the neuron
    neuronComponentActor.send(ShutDown).await;

    // send payload remotely using the neuron actor
    neuronComponentActor.send(
        InjectPayload{
            payload: String::from("0x01ff").as_bytes().to_vec(), 
            method: TransmissionMethod::Remote(String::from("p2p-synapse"))
        }
    ).await;

    // broadcast
    neuronComponentActor.send(
        Broadcast{
            local_spawn: todo!(),
            notif_data: todo!(),
            rmqConfig: todo!(),
            p2pConfig: todo!(),
            encryptionConfig: todo!(),
        }
    ).await;

    // subscribe with callback execution process
    neuronComponentActor.send(
        Subscribe{
            p2pConfig: todo!(),
            rmqConfig: todo!(),
            local_spawn: todo!(),
            // this is a callback that will be executed per each received event
            callback: Arc::new(|event| Box::pin({
                
                // clone before going into the async move{} scope
                let clonnedRedisConn = clonnedRedisConn.clone();
                async move{

                    /* ------------------------------------------------------------
                    event is the received event, we can send the event in here
                    through gRPC or RPC to another service or cache it, 
                    for example:
                    we're receiving a massive of transactions through subsription 
                    process, for each tx we'll send it to the wallet service 
                    through gRPC or cache it on redis
                    */
                        
                    //    ... 
    
                    // cache event on redis inside the callback
                    tokio::spawn(async move{
                        let mut redisConn = clonnedRedisConn.lock().await;
                        let eventId = event.clone().data.id;
                        let eventString = serde_json::to_string(&event).unwrap();
                        let redisKey = format!("cahceEventWithId: {}", eventId);
                        let _: () = redisConn.set_ex(eventId, eventString, 300).await.unwrap();
                    });
                    /* ------------------------------------------------------------ */
    
                }
            })),
            decryptionConfig: todo!(),
        }
    ).await.unwrap();

    // send a request to a neuron over eithre rmq or p2p (req, res model)
    neuronComponentActor.send(
        SendRequest{
            rmqConfig: todo!(),
            p2pConfig: todo!(),
            encryptionConfig: todo!(),
        }
    ).await;

    // receive a response from a neuron over eitehr rmq or p2p (req, res model)
    let getResponse = neuronComponentActor.send(
        ReceiveResposne{
            rmqConfig: todo!(),
            p2pConfig: todo!(),
            decryptionConfig: todo!(),
        }
    ).await;
    let Ok(resp) = getResponse else{
        panic!("can't receive response from the neuron");
    };
    let res = resp.0.await; // await on the pinned box so the future can gets executed

    // talking between local actors
    let neuronComponentActor1 = neuron1.start().recipient();
    neuronComponentActor
        .send(TalkTo{
            neuron: neuronComponentActor1,
            message: String::from("hello from neuronComponentActor")
        }).await;


    // execute an async io task inside the neuron actor thread priodically
    neuronComponentActor.send(
        ExecutePriodically{
            period: 40, // every 40 seconds
            job: task!{
                {
                    println!("inside async io task...");
                }
            }
        }
    ).await;

    // execute arbitrary async io task function inside either the actor thread or tokio light thread 
    neuronComponentActor.send(
        Execute(
            task!(
                { // block logic 
                    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
                    tx.send(String::from("")).await;
                    while let Some(data) = rx.recv().await{
                        log::info!("received data in task!()");
                    }
                } 
            ),
            true // local spawn, set to true if we want to execute the task inside the actor thread
        )
    ).await;

    log::info!("executed all..");


}

pub async fn onionEnv(){

    // trait based service container (proxy design pattern)

    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::pin::Pin;
    use tokio::sync::mpsc::{Sender, Receiver};

    // ======== io jobs and tasks
    pub type Io = Arc<dyn Fn() -> Pin<Box<dyn std::future::Future<Output = ()> 
        + Send + Sync + 'static>> 
        + Send + Sync + 'static>;

    // io event
    pub type IoEvent = Arc<dyn Fn(Event<String>) -> Pin<Box<dyn std::future::Future<Output = ()> 
        + Send + Sync + 'static>> 
        + Send + Sync + 'static>;
    
    struct Task<R, T: Fn() -> R + Send + Sync + 'static> where
        R: Future<Output=()> + Send + Sync + 'static{
        pub task: Arc<T>,
        pub io: Io,
    }
    
    #[derive(Clone)]
    struct Event<T>{
        pub datum: T,
        pub timestamp: i64
    }
    
    #[derive(Clone)]
    struct Buffer<G>{
        pub data: Arc<Mutex<Vec<G>>>,
        pub size: usize
    }

    #[derive(Clone)]
    struct PipeLine{
        pub tags: Vec<String>,
        pub pid: String,
        pub job: Job
    }
    
    // job node
    #[derive(Clone)]
    struct Job{
        pub taks: Io,
        pub parent: Arc<Job>, // use Arc to break the cycle
        pub children: Arc<Mutex<Vec<Job>>>
    }
    
    #[derive(Clone)]
    enum RunnerStatus{
        Started,
        Halted,
        Executed
    }
    
    #[derive(Clone)]
    struct Executor{
        pub runner: RunnerActorThreadPoolEventLoop,
    }

    impl Environment{
        pub fn new(envName: String) -> Self{
            Self { env_name: envName, 
            executor: Executor { runner: RunnerActorThreadPoolEventLoop::new(10) } }
        }
    }
    
    #[derive(Clone)]
    struct Environment{
        pub env_name: String,
        pub executor: Executor,
    }

    // ============== the app context
    #[derive(Clone)]
    struct AppContext{
        pub containers: Vec<Addr<Container>>,
        pub env: Environment
    }
    impl AppContext{
        pub fn new() -> Self{
            Self { containers: vec![], env: Environment::new(String::from("onionEvn013")) }
        }
        pub fn pushContainer(&mut self, container: Addr<Container>) -> Self{
            let Self{containers, env} = self;
            containers.push(container);
            Self{containers: containers.to_vec(), env: env.clone() }
        }
    }
    #[derive(Clone)]
    struct UserDto;
    trait Service{
        // build router tree for the current dto
        fn buildRouters(&mut self) -> Router;
    }
    // ============== implementations
    impl Service for UserDto{
        fn buildRouters(&mut self) -> Router{
            let router = Router::new();
            // possibly post and get routers
            // ...
            router
        }
    }
    impl Service for MinIoDriver{
        fn buildRouters(&mut self) -> Router {
            let router = Router::new();
            // possibly post and get routers
            // ...
            router      
        }
    }
    impl Service for LocalFileDriver{
        fn buildRouters(&mut self) -> Router {
            let router = Router::new();
            // possibly post and get routers
            // ...
            router      
        }
    }

    // ============== the container actors
    #[derive(Clone)]
    struct Container{
        // Arc is a reference-counted smart pointer used for thread-safe shared ownership of data
        // Arc makes the whole service field cloneable cause the container must be cloneable 
        // to return updated context when pushing new container into its vector 
        service: Arc<dyn Service>, // dependency injection
        id: String,
        // a service must have host and port
        host: String,
        port: u16
    }
    let user = UserDto;
    let userContainer = Container{
        service: Arc::new(user), 
        id: Uuid::new_v4().to_string(),
        host: String::from("0.0.0.0"),
        port: 2875
    }; // create a container for this service

    let uploadDriverContainer = Container{
        service: Arc::new(LocalFileDriver{
            content: {
                // calling the save() of the interface on the driver instance
                // we can do this since the interface is implemented for the struct
                // and we can override the methods
                let mut file = tokio::fs::File::open("Data.json").await.unwrap();
                let mut buffer = vec![];
                let readBytes = file.read_buf(&mut buffer).await.unwrap();
                let mut secureCellConfig = SecureCellConfig::default();
                buffer.encrypt(&mut secureCellConfig);
                Arc::new(buffer)
            }, 
            path: String::from("here.txt")
        }),
        id: Uuid::new_v4().to_string(),
        host: String::from("0.0.0.0"),
        port: 8375
    };

    // buffer now contains the encrypted data, no need to mutex the buffer 
    // cause we don't want to modify the buffer, calling save() method requires 
    // the type to implement the ObjectStorage trait
    // let mut fileDriver = uploadDriverContainer.service;
    // fileDriver.save().await;

    // actor setup
    impl Actor for Container{
        type Context = Context<Self>;
        fn started(&mut self, ctx: &mut Self::Context) {
            println!("the container {} started", self.id);
        }
    }
    #[derive(Message, Clone, Debug)]
    #[rtype(result = "()")]
    pub struct TalkTo{
        pub msg: MsgType,
        pub container: Recipient<WakeUp> // to talk to the container we should send a WakeUp message
    }
    #[derive(Message, Clone, Debug)]
    #[rtype(result = "()")]
    struct WakeUp{
        pub msg: MsgType
    }
    #[derive(Clone, Debug)]
    enum MsgType{
        Serve,
        Stop
    }
    impl actix::Handler<TalkTo> for Container{ // use this to send the wake up message to a container
        type Result = ();
        fn handle(&mut self, msg: TalkTo, ctx: &mut Self::Context) -> Self::Result {
            let TalkTo { msg, container } = msg.clone();
            go!{
                {
                    container.send(WakeUp { msg }).await;
                }
            }
        }
    }
    impl actix::Handler<WakeUp> for Container{ // use this to wake up a container
        type Result = ();
        fn handle(&mut self, msg: WakeUp, ctx: &mut Self::Context) -> Self::Result {
            let WakeUp { msg } = msg.clone();
            match msg{
                MsgType::Serve => {
                    let host = self.host.clone();
                    let port = self.port.clone();
                },
                MsgType::Stop => {
                    ctx.stop();
                },
                _ => {
                    log::error!("not supported command for the container");
                }
            }

        }
    }
    let userContainerActor = userContainer.start();
    let uploadDriverContainerActor = uploadDriverContainer.start();
    userContainerActor.send(
        TalkTo{
            msg: MsgType::Serve, 
            container: uploadDriverContainerActor.clone().recipient()
        }
    ).await;
    let ctx = AppContext::new()
        .pushContainer(userContainerActor)
        .pushContainer(uploadDriverContainerActor);

    // ============== the runner actor threadpool
    #[derive(Clone)]
    struct Worker{
        pub thread: Arc<tokio::task::JoinHandle<()>>,
        pub id: String
    }

    impl Worker{
        pub fn new(id: String, receiver: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Message>>>) -> Self{
            
            let clonedId = id.clone();
            let thread = tokio::spawn(async move{
                let mut getReceiver = receiver.lock().await;
                while let Some(msg) = getReceiver.recv().await{
                    log::info!("worker {} received task", clonedId);
                    match msg{
                        Message::Task(job) => {
                            job().await;
                        },
                        Message::Terminate => {
                            println!("terminating worker");
                            break;
                        },
                        Message::EventBuffer(buffer) => {
                            let getData = buffer.data;
                            let mut dataVector = getData.lock().await;
                            while !dataVector.is_empty(){
                                let getEvent = dataVector.pop();
                                if getEvent.is_some(){
                                    let event = getEvent.unwrap();
                                    let job = event.datum;
                                    go!{
                                        {
                                            job().await;
                                        }
                                    }
                                }
                            }

                        },
                    }
                }
            });

            // returning the built thread which has a receiver receiving constantly
            Self { thread: Arc::new(thread), id, }
        }
    }

    enum Message{
        EventBuffer(Buffer<Event<Io>>),
        Task(Io),
        Terminate
    }

    #[derive(Clone)]
    struct RunnerActorThreadPoolEventLoop{
        pub workers: Vec<Worker>,
        pub buffer: Buffer<Event<Io>>,
        pub sender: tokio::sync::mpsc::Sender<Message>,
        pub eventLoop: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Message>>>,
        pub pods: Vec<PipeLine>,
        pub id: String,
        pub status: RunnerStatus
    }

    // terminating all workers when the instance is going to be dropped
    impl Drop for RunnerActorThreadPoolEventLoop{
        fn drop(&mut self) {
            println!("sending terminate message");
            for worker in &self.workers{ // Worker doesn't implement Clone, we're borrowing it
                let clonedSender = self.sender.clone();
                // send the terminate message in the background thread
                tokio::spawn(async move{
                    clonedSender.send(Message::Terminate).await;
                });
            }
            println!("shutting down all workers");
            for worker in &self.workers{ // Worker doesn't implement Clone, we're borrowing it
                worker.thread.abort(); // abort the future object
            }
        }    
    } 

    impl RunnerActorThreadPoolEventLoop{
        pub fn new(size: usize) -> Self{

            let (tx, rx) = tokio::sync::mpsc::channel::<Message>(100);
            let eventLoop = Arc::new(tokio::sync::Mutex::new(rx));
            
            Self{ buffer: Buffer { data: Arc::new(Mutex::new(vec![])), size: 100 }, workers: {
                (0..size)
                .into_iter()
                .map(|idx| {
                    Worker::new(Uuid::new_v4().to_string(), eventLoop.clone())
                })
                .collect::<Vec<Worker>>()
            }, sender: tx,  eventLoop, pods: vec![], 
                id: Uuid::new_v4().to_string(), status: RunnerStatus::Started }
        }
        pub async fn spawn(&self, msg: Message){
            let sender = self.sender.clone();
            sender.send(msg).await;
        }
        pub async fn push(&mut self, job: Io){
            let buffer = self.buffer.clone(); // clone to prevent the self from moving
            let mut getData = buffer.data.lock().await;
            (*getData).push(Event { datum: job, timestamp: chrono::Local::now().timestamp() });
        }

    }

}