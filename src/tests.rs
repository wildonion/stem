

use std::sync::Arc;
use crate::*;
use stemlib::schemas::{Neuron, TransmissionMethod, *};
use stemlib::messages::*;
use stemlib::interfaces::OnionStream;



pub async fn upAndRunStreaming(){
    
    let mut neuron = Neuron::new(100, "Neuron1").await;
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
            
            // ------------------ the async task ------------------
            async move{

                if error.is_some(){
                    println!("an error accoured during receiving: {:?}", error.unwrap());
                    return;
                }
    
                println!("received task: {:?}", event);
    
                // we can also use the internal eventloop to receive event:
                // use the eventloop of the internal executor to receive the event 
                // the event has sent from where we've subscribed to incoming events
                let eventloop = clonedExecutor.eventloop.clone();
                tokio::spawn(async move{
                    let mut rx = eventloop.lock().await;
                    while let Some(e) = rx.recv().await{
                        
                        log::info!("received event: {:?}", e);

                        // ...

                    } 
                });

                
                // cache event on redis
                // ...
    
    
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
        Execute{
            period: 40,
            job: Arc::new(
                ||{
                    Box::pin(async move{
                        println!("inside async io task...");
                    })
                }
            )
        }
    ).await;

    log::info!("executed all..");


}

pub async fn upAndRunExecutor(){
    use tokio::sync::mpsc;
    use tokio::spawn;

    type Io = std::sync::Arc<dyn Fn() -> 
            std::pin::Pin<Box<dyn std::future::Future<Output=()> 
            + Send + Sync + 'static>> 
            + Send + Sync + 'static>;
            
    // eventloop
    #[derive(Clone)]
    struct EventLoop{
        pub sender: tokio::sync::mpsc::Sender<Task>, 
        pub receiver: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Task>>>
    }
    
    impl EventLoop{
        pub fn new() -> Self{
            let (tx, mut rx) = tokio::sync::mpsc::channel::<Task>(100);            
            Self{
                sender: tx,
                receiver: std::sync::Arc::new(
                    tokio::sync::Mutex::new(
                        rx
                    )
                )
            }
        }
        pub async fn spawn(&mut self, task: Task){
            let sender = self.clone().sender;
            sender.send(task).await;
        }
        pub async fn run<R, F: Fn(Task) -> R + Send + Sync + 'static>(&mut self, cb: F)
        where R: std::future::Future<Output=()> + Send + Sync + 'static{
            let this = self.clone();
            let arcedCallback = std::sync::Arc::new(cb);
            spawn(async move{
                let clonedArcedCallback = arcedCallback.clone();
                let getRx = this.clone().receiver;
                let mut rx = getRx.lock().await;
                while let Some(task) = rx.recv().await{
                    println!("received task from eventloop, executing in a thread of the eventloop");
                    spawn(
                        clonedArcedCallback(task)
                    );
                }
            });
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
    
    
    let mut eventLoop = EventLoop::new();
    
    let task = Task{
        job: std::sync::Arc::new(
            ||{
                Box::pin(async move{
                    println!("executing an intensive io...");
                })
            }
        )
    };
    eventLoop.spawn(task).await;
    
    eventLoop.run(|task| async move{
        println!("inside the callback, playing with received task");
        let job = task.job;
        spawn(job());
    }).await;
}