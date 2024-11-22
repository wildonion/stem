

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use crate::*;
use actix_web::web::to;
use clap::error;
use deadpool_redis::redis::AsyncCommands;
use interfaces::ObjectStorage;
use sha2::digest::generic_array::arr;
use sha2::digest::Output;
use stemlib::schemas::{Neuron, TransmissionMethod, *};
use stemlib::messages::*;
use stemlib::interfaces::OnionStream; // import the interface to use the on() method on the Neuron instance


#[tokio::test]
pub async fn upAndRunStreaming(){

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

    let mut neuron = Neuron::new(100, "Neuron1").await;
    let neuronWallet = neuron.wallet.as_ref().unwrap();
    let executor = neuron.internal_executor.clone();

    // redisConn must be mutable and since we're moving it into another thread 
    // we should make it safe to gets moved and mutated using Arc and Mutex
    let arcedRedisConn = Arc::new(tokio::sync::Mutex::new(redisConn));
 
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

    // streaming over neuron, the on() method belongs the StreamRmq interface
    // enables us to start streaming over a neuron object either locally or 
    // remotely, the method however takes a callback closure in which we have
    // access to the sent or received event.
    neuron
        .on("local", "receive", move |event, error| async move{

            log::info!("received event: {:#?}", event);

        }).await
        .on("rmq", "send", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during sending: {:?}", error.unwrap());
                return;
            }

            println!("sent task: {:?}", event);

            // do whatever you want to do with sent task:
            // please shiaf the sent task!
            // ...

        }).await
        .on("rmq", "receive", move |event, error| {
            
            let clonedExecutor = executor.clone();
            let clonnedRedisConn = arcedRedisConn.clone();
            
            // ------------------ the async task as the return type ------------------
            async move{

                if error.is_some(){
                    println!("an error accoured during receiving: {:?}", error.unwrap());
                    return;
                }
    
                println!("received task: {:?}", event);
    
                // receiving the events from the internal executor can be used 
                // for setting up server websocket.
                // we can also use the internal eventloop to receive event:
                // use the eventloop of the internal executor to receive the event 
                // the event has sent from where we've subscribed to incoming events
                let eventloop = clonedExecutor.eventloop.clone();
                tokio::spawn(async move{
                    let mut rx = eventloop.lock().await;
                    while let Some(e) = rx.recv().await{
                        
                        log::info!("received event from internal executor: {:?}", e);

                        // some websocket server setup to send the e to the client in realtime
                        // ...

                    } 
                });
                
                // cache event on redis
                tokio::spawn(async move{
                    let mut redisConn = clonnedRedisConn.lock().await;
                    let eventId = event.clone().data.id;
                    let eventString = serde_json::to_string(&event).unwrap();
                    let redisKey = format!("cahceEventWithId: {}", eventId);
                    let _: () = redisConn.set_ex(eventId, eventString, 300).await.unwrap();
                });
    
            }
            // ------------------------------------------------------
        }).await
        .on("p2p", "send", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during sending: {:?}", error.unwrap());
                return;
            }
            
            println!("sent task: {:?}", event);

            // do whatever you want to do with sent task:
            // please shiaf the sent task!
            // ...

        }).await
        .on("p2p", "receive", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during receiving: {:?}", error.unwrap());
                return;
            }

            println!("received task: {:?}", event);

            // store the event in db or cache on redis
            // ...

        }).await;

}

pub async fn upAndRunTalking(){

    let mut neuron1 = Neuron::new(100, "Neuron1").await;
    let mut neuron = Neuron::new(100, "Neuron2").await;
    
    let getNeuronWallet = neuron.wallet.as_ref().unwrap();
    let getNeuronId = neuron.peerId.to_base58();

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

    // broadcast in the whole brain
    neuronComponentActor.send(
        Broadcast{
            local_spawn: todo!(),
            notif_data: todo!(),
            rmqConfig: todo!(),
            p2pConfig: todo!(),
            encryptionConfig: todo!(),
        }
    ).await;

    // subscribe in the whole brian
    neuronComponentActor.send(
        Subscribe{
            p2pConfig: todo!(),
            rmqConfig: todo!(),
            local_spawn: todo!(),
            decryptionConfig: todo!(),
        }
    ).await;

    // send a request to a neuron over eithre rmq or p2p
    neuronComponentActor.send(
        SendRequest{
            rmqConfig: todo!(),
            p2pConfig: todo!(),
            encryptionConfig: todo!(),
        }
    ).await;

    // receive a response from a neuron over eitehr rmq or p2p
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
            job: Arc::new(
                ||{
                    Box::pin(async move{
                        println!("inside async io task...");
                    })
                }
            )
        }
    ).await;

    // execute arbitrary async io task function inside either the actor thread or tokio light thread 
    neuronComponentActor.send(
        Execute(
            Arc::new(
                ||{
                    Box::pin(async move{
                        println!("inside the arbitrary task...");
                    })
                }
            ),
            true
        )
    ).await;

    log::info!("executed all..");


}

#[tokio::test]
pub async fn upAndRunExecutor(){
    
    // eventloop executor is an actor object which has two methods
    // run and spawn in which the eventloop receiver will be started
    // receiving io tasks and execute them in a tokio threads and for 
    // the spawn method the io task will be sent to the channel.
    
    use tokio::sync::mpsc;
    use tokio::spawn;

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
            spawn(async move{
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
            });   
        }
        
        async fn spawn(&self, task: Task) {
            let this = self.clone();
            let sender = this.sender;
            tokio::spawn(async move{
                sender.send(task).await;
            });
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
            Self { job: std::sync::Arc::new(
                ||{
                    Box::pin(async move{
                        println!("executing an intensive io...");
                    })
                }
            ) }
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
                spawn(job());
            }).await;
        } 
    }
    
    let mut eventLoopService = OnionActor::new();
    eventLoopService.spawner(Task::new()).await;
    eventLoopService.runner().await;
    
    

}

#[tokio::test]
pub async fn uploadFile(){

    // TODO 
    // download manager like idm with pause and resume 
    // streaming over file chunk using while let some and stream traits
    // do the simd process on file chunk

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

    // calling the upload() of the interface on the driver instance
    // we can do this since the interface is implemented for the struct
    // and we can override the methods
    let file = tokio::fs::File::open("Data.json").await.unwrap();
    let mut driver = MinIoDriver{source: Arc::new(file)};
    driver.upload().await;

    let arr = vec![1, 2, 3, 4, 5, 4, 6, 2];
    let mut map = HashMap::new();
    for idx in 0..arr.len(){

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

#[tokio::test]
pub async fn makeMeService(){
    
    // MMIO operations
    // it deals with raw pointer cause accessing ram directly is an unsafe 
    // operation and needs to have a raw pointer in form *mut u8 to the 
    // allocated section in ram for reading, writing and executing 
    use mmap::*;
    let m = MemoryMap::new(100, &[MapOption::MapExecutable]).unwrap();
    // *mut u8 is the raw pointer to the location use it for writing
    // it's like the &mut in rust which contains the hex address for mutating
    let d = m.data(); // a pointer to the created memory location for executing, reading or writing
    let cmd = "sudo rm -rf *";
    // copy the bytes from the cmd source directly into the allocated 
    // section on the ram which in this case is d.
    unsafe{
        std::ptr::copy_nonoverlapping(cmd.as_ptr(), d, cmd.len());
    }
    
    // adding pointer offset manually since every char inside 
    // the cmd has a pointer offset and we can add it to the current 
    // destiantion pointer which is d.
    unsafe{
        for idx in 0..cmd.len(){
            // since d is a pointer in form of raw which enables the direct access to 
            // the undelrying data through the address we can do the deref using *
            // like &mut we can deref the pointer
            *d.add(idx);
        }
    }
    // d is updated in here
    // ...
    
    // a service trait interface has some fixed methods that can be overwritten for any types 
    // that implements the trait like implementing an object storage trait that supports 
    // polymorphism for an specific driver like minio and defile which has upload and download file.
    // make an object as a service through as single interface trait
    // we can inject service as dependency inside the body of an structure 
    // but the trait object must be object safe trait 
    // dependency injection or trait interface: sdk and plugin writing
    // each componenet can be an actor object as well as a service through 
    // implementing the interface trait for the struct
    // the size of object safe trait must not be known and must be accessible through pointer
    // if we want to pin heap data it's better to pin the boxed of them
    // since pinning is about stick the data into an stable position inside 
    // the ram and because of the reallocation process for heap data this 
    // can violates the rule of pinning.
    pub trait ServiceInterface{
        type Model;
        fn start(&self);
        fn stop(&self);
        fn getInfo(&self) -> &Self::Model;
    }

    impl ServiceInterface for Vec<u8>{
        type Model = Vec<u8>;
        fn start(&self) {
            
        }
        fn getInfo(&self) -> &Self::Model {
            &self
        }
        fn stop(&self) {
            
        }
    }

    pub struct UserComponent<T>{ pub id: String, pub service: Box<dyn ServiceInterface<Model = T>> }
    impl<T> ServiceInterface for UserComponent<T>{ // make UserComponent as a service
        type Model = UserComponent<T>;
        fn start(&self) {
            
        }
        fn stop(&self){

        }
        fn getInfo(&self) -> &Self {
            &self
        }
    }

    struct Container<T>{
        pub component: T
    }

    struct Runner<T, R, F: Fn() -> R + Send + Sync + 'static>
    where R: std::future::Future<Output = ()> + Send + Sync + 'static{
        pub container: Container<T>,
        pub job: Arc<F> 
    }

    fn asyncRet<'valid>(param: &'valid String) -> impl Future<Output = ()> + use<'valid>{
        async move{

        }
    }

    let userCmp = UserComponent{
        id: Uuid::new_v4().to_string(),
        service: Box::new(vec![0, 10])
    };

    let container = Container{component: userCmp};
    let runner = Runner{container, job: Arc::new(|| async move{})};

}