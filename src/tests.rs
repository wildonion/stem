

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::thread::JoinHandle;
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

#[tokio::test]
pub async fn upAndRunEventLoopExecutor(){

    // ================= the executor
    // eventloop executor is an actor object which has two methods
    // run and spawn in which the eventloop receiver will be started
    // receiving io tasks and execute them in a tokio threads and for 
    // the spawn method the io task will be sent to the channel.
    
    use tokio::sync::mpsc;
    use tokio::spawn;

    struct Task1<T: Fn() -> R + Send + Sync + 'static, R>
    where R: Future<Output = ()> + Send + Sync + 'static{
        pub taks: Arc<T> 
    } 

    type Io = std::sync::Arc<dyn Fn() -> 
            std::pin::Pin<Box<dyn std::future::Future<Output=()> 
            + Send + Sync + 'static>> 
            + Send + Sync + 'static>;
            
    // eventloop
    #[derive(Clone)]
    struct OnionActor{
        pub sender: tokio::sync::mpsc::Sender<Task>, 
        pub receiver: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Task>>>,
    }

    // the OnionActor structure has two spawn and run logics to send and
    // receive io jobs from the channel so we could execute them inside 
    // the background light thread using tokio runtime and scheduler
    trait EventLoopExt{
        async fn spawn(&self, task: Task);
        async fn run<R, F: Fn(Task) -> R + Send + Sync + 'static>(&mut self, cb: F) where R: std::future::Future<Output=()> + Send + Sync + 'static;
    }

    impl EventLoopExt for OnionActor{
        async fn run<R, F: Fn(Task) -> R + Send + Sync + 'static>(&mut self, cb: F) where R: std::future::Future<Output=()> + Send + Sync + 'static{
            let this = self.clone();
            let arcedCallback = std::sync::Arc::new(cb);
            // receiving inside a light thread
            go!{
                {
                    let clonedArcedCallback = arcedCallback.clone();
                    let getRx = this.clone().receiver;
                    let mut rx = getRx.lock().await;
                    while let Some(task) = rx.recv().await{
                        println!("received task from eventloop, executing in a thread of the eventloop");
                        // executing the callback inside a light thread
                        spawn(
                            clonedArcedCallback(task) // when we run the callback closure trait it returns future object as its return type 
                        );
                    }
                }
            };
        }
        
        async fn spawn(&self, task: Task) {
            let this = self.clone();
            let sender = this.sender;
            go!{
                {
                    sender.send(task).await;
                }
            };
        }
    }

    impl OnionActor{
        pub fn new() -> Self{
            let (tx, mut rx) = tokio::sync::mpsc::channel::<Task>(100);            
            Self{
                sender: tx,
                receiver: std::sync::Arc::new(
                    tokio::sync::Mutex::new(
                        rx
                    )
                ),
            }
        }
    }
    
    // task struct with not generic; it needs to pin the future object
    // no need to have output for the future, we'll be using channels
    // Arc<F>, F: Fn() -> R + Send + Sync + 'static where R: Future<Output=()> + Send + Sync + 'static
    // use Box<dyn or Arc<dyn for dynamic dispatch and R: Trait for static dispatch
    // in dynamic dispatch the instance of the type whose impls the trait must be wrapped into the Arc or Box
    #[derive(Clone)]
    struct Task{
        pub job: Io 
    }

    impl Task{
        pub fn new() -> Self{
            Self { job: task!{
                {
                    println!("executing an intensive io...");
                }
            } }
        }
    }

    pub trait OnionService{
        type Service; 
        async fn runner(&mut self);
        async fn stop(&self);
        async fn spawner(&self, task: Task);
    }

    impl OnionService for OnionActor{
        type Service = Self;
        async fn spawner(&self, task: Task) {
            self.spawn(task).await; // spawn io task into the eventloop thread
        }
        async fn stop(&self) {
            
        }
        async fn runner(&mut self){
            // run() method takes a callback with the received task as its param
            // inside the method we'll start receiving from the channel then pass
            // the received task to the callback, inside the callback however the
            // task is being exeucted inside another light thread 
            self.run(|task| async move{ 
                println!("inside the callback, executing received task");
                let job = task.job;
                go!(job);
                go!(
                    || Box::pin(async move{
                        println!("executing an io task inside light thread");
                    })
                );
            }).await;
        } 
    }
    
    let mut eventLoopService = OnionActor::new();
    eventLoopService.spawner(Task::new()).await;
    eventLoopService.runner().await;

}

#[tokio::test]
pub async fn saveFile(){

    // TODO 
    // download manager like idm with pause and resume 
    // streaming over file chunk using while let some, stream traits and simd ops

    // --------------------------------------------
    // simulating the pause and resume process
    // deref pointers using *: this can be Box, Arc and Mutex 
    // --------------------------------------------
    let fileStatus = std::sync::Mutex::new(String::from("pause"));
    let signal = std::sync::Condvar::new();
    let sigstat = Arc::new((signal, fileStatus));
    let clonedSigStat = sigstat.clone();

    // the caller thread gets blocked until it able to acquire the mutex 
    // since at the time of acquireing the mutex might be busy by another thread
    // hence to avoid data races we should block the requester thread until the 
    // mutex is freed to be used.
    // we can use mutex to update it globally even inside a thread
    std::thread::spawn(move || {
        let (sig, stat) = &*clonedSigStat; // deref the Arc smart pointer then borrow it to prevent moving
        let mut lock = stat.lock().unwrap();
        *lock = String::from("resume");
        sig.notify_one();
    });
    
    let (sig, stat) = &*sigstat; // deref the Arc smart pointer then borrow it to prevent moving
    let mut getStat = stat.lock().unwrap();
    while *getStat != String::from("resume"){
        // wait by blocking the thread until the getState gets updated
        // and if it was still paused we can wait to be resumed
        getStat = sig.wait(getStat).unwrap(); // the result of the wait is the updated version of the getStat
    }

    // --------------------------------------------

    // calling the save() of the interface on the driver instance
    // we can do this since the interface is implemented for the struct
    // and we can override the methods
    let mut file = tokio::fs::File::open("Data.json").await.unwrap();
    let mut buffer = vec![];
    let readBytes = file.read_buf(&mut buffer).await.unwrap();
    let mut secureCellConfig = SecureCellConfig::default();
    buffer.encrypt(&mut secureCellConfig);

    // buffer now contains the encrypted data, no need to mutex the buffer 
    // if we want to modify the buffer we should use the Mutex
    let mut driver = MinIoDriver{content: Arc::new(buffer)};
    driver.save().await;

    let arr = vec![1, 2, 3, 4, 5, 4, 6, 2];
    let mut map = HashMap::new();
    for idx in 0..arr.len(){ // O(n)

        // use hashmap to keep track of the rep elems
        let keyval = map.get_key_value(&arr[idx]);
        if let Some((key, val)) = keyval{
            let mut rawVal = *val;
            rawVal += 1;
            map.insert(*key, rawVal);   
        } else{
            map.insert(arr[idx], 0);
        }

        // OR

        map
            .entry(arr[idx])
            .and_modify(|rep| *rep += 1)
            .or_insert(arr[idx]);
    }



}

pub async fn sexchangeRunner(){

    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::pin::Pin;
    use tokio::sync::mpsc::{Sender, Receiver};

    type Io = Arc<dyn Fn() -> Pin<Box<dyn Future<Output=()> + Send + Sync + 'static>> + Send + Sync + 'static >; 
    
    struct Task<R, T: Fn() -> R + Send + Sync + 'static> where
        R: Future<Output=()> + Send + Sync + 'static{
        pub task: Arc<T>
    }
    
    struct Event<T>{
        pub datum: T,
        pub timestamp: i64
    }
    
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
    
    struct Executor<G>{
        pub sender: Arc<Sender<G>>,
        pub receiver: Arc<Mutex<Receiver<G>>>,
        pub runner: RunnerActorThreadPoolEventLoop,
    }
    
    struct Environment<G>{
        pub env_name: String,
        pub executor: Executor<G>,
    }
    
    enum OrderType{
        Bid,
        Ask
    }
    
    struct Order{
        pub otype: OrderType,
        pub quantity: u64,
        pub amount: u64,
        pub timestamp: i64
    }
    
    // spawn a light thread per each crypto type to find the match
    // cause we have lock which is nicer to do it in a light thread
    struct BookEngine{
        pub orders: Arc<Mutex<std::collections::BTreeMap<String, Order>>>,
    }


    // ============== the app context
    #[derive(Clone)]
    struct AppContext{
        pub containers: Vec<Addr<Container>>,
        pub runner: RunnerActorThreadPoolEventLoop,
    }
    impl AppContext{
        pub fn new() -> Self{
            Self { containers: vec![], runner: RunnerActorThreadPoolEventLoop::new(10) }
        }
        pub fn pushContainer(&mut self, container: Addr<Container>) -> Self{
            let Self{containers, runner} = self;
            containers.push(container);
            Self{containers: containers.to_vec(), runner: runner.clone() }
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

    // ============== the container actors
    #[derive(Clone)]
    struct Container{
        // Arc is a reference-counted smart pointer used for thread-safe shared ownership of data
        // Arc makes the whole service field cloneable cause the container must be cloneable 
        // to return updated context when pushing new container into its vector 
        service: Arc<dyn Service>, 
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
        service: Arc::new(MinIoDriver{content: Arc::new(vec![])}),
        id: Uuid::new_v4().to_string(),
        host: String::from("0.0.0.0"),
        port: 8375
    };

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
            tokio::spawn(async move{
                container.send(WakeUp { msg }).await;
            });
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
                        }
                    }
                }
            });

            Self { thread: Arc::new(thread), id, }
        }
    }

    enum Message{
        Task(Io),
        Terminate
    }

    #[derive(Clone)]
    struct RunnerActorThreadPoolEventLoop{
        pub workers: Vec<Worker>,
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

            let workers = 
            (0..size)
                .into_iter()
                .map(|idx| {
                    Worker::new(Uuid::new_v4().to_string(), eventLoop.clone())
                })
                .collect::<Vec<Worker>>();

            
            Self{ workers, sender: tx,  eventLoop, pods: vec![], 
                id: Uuid::new_v4().to_string(), status: RunnerStatus::Started }
        }
        pub async fn spawn(&self, msg: Message){
            let sender = self.sender.clone();
            sender.send(msg).await;
        }
    }


    // implement #[event] on top of the RunnerActorThreadPoolEventLoop
    // deploy ctx as serverless object using BFP
    // ...

    // notify all train about the pause and arrival time of a train
    // don't pass stations
    // if a train was stopped longer than the pauseTime block all other trains (inside tunnel or at station)
    // use this model for sexchange
    struct Train{
        pub arrivalTime: u64,
        pub pauseTime: u64,
        pub status: std::sync::Condvar
    }

    struct TrainQueue{
        pub queue: std::collections::VecDeque<Train>, // a ring buffer queue
    }
    
}