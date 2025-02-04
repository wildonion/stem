
use futures::stream::{self, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use core::time;
use std::collections::{HashMap, VecDeque};
use std::default;
use std::future::Future;
use std::sync::{Arc, Condvar};
use std::thread::{self, park, JoinHandle};
use crate::*;
use clap::error;
use crypter::wallet::ed25519;
use deadpool_lapin::lapin::protocol::channel;
use deadpool_redis::redis::{AsyncCommands, RedisError};
use deadpool_redis::Connection;
use interfaces::{Crypter, ObjectStorage, PubSub, ShaHasher};
use is_type::Is;
use salvo::{FlowCtrl, Router};
use sha2::digest::generic_array::arr;
use sha2::digest::Output;
use stemlib::dto::{Neuron, TransmissionMethod, *};
use stemlib::messages::*;
use stemlib::interfaces::Channel;
use tokio::sync::Mutex;
use wallexerr::misc::SecureCellConfig; // import the interface to use the on() method on the Neuron instance
use stemlib::dsl::*;
use stemlib::misc::setupRedis;



#[tokio::test]
async fn streamObjStrg(){
    
    let mut map: HashMap::<String, Vec<u8>> = HashMap::new(); // an in memory object storage using hash map
    // upload streaming
    let mut file = tokio::fs::File::open("video.mp4").await.unwrap();
    let chunkSize = 10; // 10 bytes
    let mut buf = Vec::with_capacity(chunkSize);
    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    let mut streamBuf = vec![]; 
    let objId = streamBuf.store().await;

    let getFileObject = map.get_mut("video.mp4");
    let mut fileObj = vec![];
    if getFileObject.is_some(){
        fileObj = getFileObject.unwrap().clone();
    } else{
        map.insert("vide.mp4".to_string(), vec![]);
    }
 
    let mut file = tokio::fs::File::create("file.mp4").await.unwrap();
    // read 10 bytes per each iteration then send each 
    // chunk into the channel
    let clonedTx = tx.clone();
    loop{
        let readBytes = file.read(&mut buf).await.unwrap(); // read 10 bytes of the whole file
        if readBytes == 0{
            break;
        }

        // process on those 10 bytes
        // 1) filling the object inside the OBJ STORAGE chunk by chunk
        let mut key = String::from("some random bytes");
        key.hashMe();
        let mut pass = String::from("some random high secure bytes");
        pass.hashMe();
        // encrypt the buffer
        buf.encrypt(&mut SecureCellConfig{ 
            secret_key: key, 
            passphrase: pass, 
            data: vec![] // this gets filled inside the trait implementation
        });
        fileObj.extend_from_slice(&buf); // pass the encrypted buffer
        // 2) send the buffer to the channel to save it in file later
        tx.send(buf.clone()).await;
        // 3) push the buffer into the stream buffer
        streamBuf.push(buf.clone());
        // 4) write into file as they're coming
        file.write_all(&buf).await;
    }

    let mut streamer = stream::iter(streamBuf);

    while let Some(chunk) = rx.recv().await{
        // write to file as they're coming from the channel
        file.write_all(&chunk).await;
    }

    while let Some(chunk) = streamer.next().await{
        // also we can write the byte into file in here
        // ...
    }

}


// ================================================================================
// ================================================================================
// ================================================================================
// an actor based design pattern to create dto as a service container 
// and deploy them as a serverless obejct through their service

/* 
    Actor(Container(Service(Dto))):
        we want to deploy the Dto as a serverless object which is a of type 
        Service trait and wrapped by Container which is an actor based component 
    - a dto can be registred as a service inside a container 
    - a container has an id, host and port for the related service
    - a container is an actor component that allows us to talk with other actor component; container talking
        ex: a container can send an MsgType::Serve message to another contianer 
            to start the second container service on the defined host and port
    - a container can receive and send messages from/to other containers and different parts of the app
    - a container componenet can talk with other component by sending message
    
    components are container actor workers which contains a dto as a service (not necessarily) 
    they can talk locally and remotely using .on() methods with each other through message 
    sending pattern and can be deployed as a server less object like user or otp container 
    have its own set of routers and a deployable service an so on for notification container.
*/
// ================================================================================
// ================================================================================
// ================================================================================
pub async fn onionEnv(){ 

    // ==============================================================================
    // ======================= STEP 1) CREATE CONTAINER COMPONENTS AND THEIR SERVICES
    // ==============================================================================
    // each container is a different component inside the app like we have otp service
    // actor responsible for sending otp , rate limiter service actor responsible for 
    // handling rate limits, each container component can transfer data between different
    // thread and parts of the app through message sending logics like pubsub and mpsc
    let mut webhookHandlerComponent = Container{
        service: Arc::new(WebHookHandler),
        id: Uuid::new_v4().to_string(),
        chanConfig: ChanConfig{chanType: String::from("mpsc")},
        requests: Arc::new(vec![]), // requests to this container so far
        host: String::from("0.0.0.0"), // the server of webhook hanlder
        port: 2879,
    };

    let mut otpComponent = Container{
        service: Arc::new(Otp),
        id: Uuid::new_v4().to_string(),
        chanConfig: ChanConfig{chanType: String::from("p2p")},
        requests: Arc::new(vec![]), // requests to this container so far
        host: String::from("0.0.0.0"),
        port: 2877,
    };

    let mut rateLimiterComponent = Container{
        service: Arc::new(RateLimiter),
        id: Uuid::new_v4().to_string(),
        chanConfig: ChanConfig{chanType: String::from("pubsub")},
        requests: Arc::new(vec![]), // requests to this container so far
        host: String::from("0.0.0.0"),
        port: 2870,
    };
    
    let mut walletComponent = Container{
        service: Arc::new(WalletDto), // object safe trait for dependency injection, WalletDto impls the Service trait
        id: Uuid::new_v4().to_string(),
        chanConfig: ChanConfig{chanType: String::from("ws")},
        requests: Arc::new(vec![]), // requests to this container so far
        host: String::from("0.0.0.0"),
        port: 2875,
    };

    let uploadDriverComponent = Container{
        service: Arc::new(LocalFileDriver{
            content: {
                // calling the save() of the interface on the driver instance
                // we can do this since the interface is implemented for the struct
                // and we can override the methods
                let mut file = tokio::fs::File::open("Data.json").await.unwrap();
                let mut buffer = vec![];
                let readBytes = file.read_buf(&mut buffer).await.unwrap();
                let mut secureCellConfig = SecureCellConfig{ // don't use default cause we'll face invalid param
                    secret_key: hex::encode("secret"),
                    passphrase: hex::encode("passphrase"),
                    data: vec![],
                };
                buffer.encrypt(&mut secureCellConfig);
                Arc::new(buffer)
            }, 
            path: String::from("here.txt")
        }),
        id: Uuid::new_v4().to_string(),
        chanConfig: ChanConfig{chanType: String::from("http")},
        host: String::from("0.0.0.0"),
        port: 8375,
        requests: Arc::new(vec![]) // requests to this container so far
    };

    let mut clonedWalletComponent = walletComponent.clone();
    tokio::spawn(async move{

        // streaming over contanier is also possible, generally this sugar syntax is better
        // than sending message using .send() method, currently based on the channel confit 
        // the channel type for walletComponent is ws which means we're sending ws event through
        // switch to a new channel using walltComponent.switchChannel("p2p"); method
        clonedWalletComponent.on("recv", |mut event, error| async move{

            if error.is_some(){
                log::error!("error has happened: {:?}", error.unwrap());
            }
            log::info!("sent event: {:?}", event);

            // do the redis operations inside a new thread cause, 
            // call the object storage methods inside a new thread
            tokio::spawn(async move{

                log::info!("executing callback for received event: ... ");
                // ======================
                // ex) using object storage to store and fetch data
                let receivedData = serde_json::from_value::<String>(event.clone().data.action_data).unwrap();
                let objectId = event.store().await;
                let object = String::fetch(&objectId).await;

                // downloading file from object storage)
                // streaming over chunks, having them as future object  
                let mut objStreamer = String::fetchStream(&objectId).await;
                let mut file = tokio::fs::File::create("path.txt").await.unwrap();
                let mut buffer = vec![];
                while let Some(d) = objStreamer.next().await{
                    let b = d.unwrap();
                    file.write_all(&b).await; // write to disk chunk by chunk
                    // encrypted chunk
                    b.to_vec().encrypt(
                        &mut SecureCellConfig{ 
                            secret_key: String::from("secret"), 
                            passphrase: String::from("pass"), 
                            data: vec![] 
                        }
                    );
                    // receive a chunk from the channel and append it to the buffer
                    buffer.extend_from_slice(b.to_vec().as_slice()); 
                }
                file.flush().await.unwrap();
                // ======================

                // ...
            });

        }).await;

    });

    // ======================================================
    // ======================= STEP 2) START CONTAINER ACTOR 
    // ======================================================
    // start both containers as actors
    let walletComponentActor = walletComponent.start();
    let uploadDriverComponentActor = uploadDriverComponent.start();
    
    // =========================================================================================
    // ======================= STEP 3) TALK TO EACH CONTAINER THROUGH SENDING MESSAGE USING MPSC
    // ========================================================================================= 
    // walletComponentActor wants to talk with the uploadDriverComponentActor
    walletComponentActor.send(
        TalkToContainer{
            msg: MsgType::Serve, 
            container: uploadDriverComponentActor.clone().recipient()
        }
    ).await.unwrap();

    walletComponentActor.send(
        TalkToContainer{
            msg: MsgType::Stop, 
            container: uploadDriverComponentActor.clone().recipient()
        }
    ).await.unwrap();

    // send an event data to the uploadDriverComponentActor 
    walletComponentActor.send(
        TalkToContainer{
            msg: MsgType::Event(Event::default()),
            container: uploadDriverComponentActor.clone().recipient()
        }
    ).await.unwrap();

    let underlyingService = walletComponentActor.send(
        GetServiceInfo
    ).await;

    let task = Arc::new(||{
        Box::pin(async move{
            log::info!("a heavy task..");
        })
    });
    let mut time = tokio::time::interval(tokio::time::Duration::from_secs(10));
    tokio::spawn(async move{
        loop{
            time.tick().await;
            task().await;
        }
    });

    // ================================================================================
    // ======================= STEP 4) BUIL APP CONTEXT AND PUSH THE CONTAINERS INTO IT
    // ================================================================================ 
    // a dto is a copmponent that can be used to model an antity and interact with the core 
    // of the entity including db calls and updating its state; we can convert a dto into a 
    // service to host it on an address and port by adding it inside a container as a service 
    // trait object, each container is also an actor which can communicate internally with 
    // other container through message sending. 
    // push the containers into the app context
    let ctx = AppContext::new().await
        .pushContainer(walletComponentActor.clone())
        .pushContainer(uploadDriverComponentActor.clone());
    
    // get the first contianer actor
    let containers = ctx.getContainers();
    let c1 = containers[1].clone();
    let clonedC1 = c1.clone();
    
    //============== testing container actor local message passing
    // deploy the container service in the background thread
    // it starts its service on the specified host and port  
    go!{
        {
            clonedC1.send(Deploy).await.unwrap(); // wallet dto model starts an http server, it can by any server overwritten in Service trait methods
        }
    }

    // execute an async io task priodically
    c1.clone().send(
        ExecutePriodically{
            period: 40, // every 40 seconds
            job: task!{
                {
                    // we can check the status of the task
                    println!("i'm being executed every 40 seconds...");
                }
            }
        }
    ).await.unwrap();

    // execute arbitrary async io task function inside either the actor thread or tokio light thread 
    c1.clone().send(
        Execute(
            task!(
                { // block logic 
                    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
                    tx.send(String::from("wildonion sender")).await;
                    while let Some(data) = rx.recv().await{
                        log::info!("received data in task!() > {:?}", data);
                    }
                } 
            ),
            true // local spawn, set to true if we want to execute the task inside the actor thread
        )
    ).await.unwrap();

    // keep the app up so the dto services can be in a constant execution state
    loop{}

    // ============================================================================
    // ============================== OBJECT STORAGE ==============================
    // ============================================================================
    // event dto instance tests
    // let mut event = Event::default();
    // let objId = event.store().await; // cache the event instance
    // let stringEvent = Event::fetch(&objId).await; // fetch the object from the storage, it's a byte array and can be anything files and instances
    // let mut event = serde_json::from_slice::<Event>(&stringEvent).unwrap(); // decode the fetched object into the Event struct
    // event.on("rmq", "send", |event, error| async move{ // start sending event streams with executing callback
    //     if let Some(err) = error{
    //         log::error!("the error in sending event");
    //     }
    //     log::info!("executing callback");
    // }).await;

    // // storing file on object storage
    // let mut file = {
    //     // calling the save() of the interface on the driver instance
    //     // we can do this since the interface is implemented for the struct
    //     // and we can override the methods
    //     let mut file = tokio::fs::File::open("Data.json").await.unwrap();
    //     let mut buffer = vec![];
    //     let readBytes = file.read_buf(&mut buffer).await.unwrap();
    //     let mut secureCellConfig = SecureCellConfig{ // don't use default cause we'll face invalid param
    //         secret_key: hex::encode("secret"),
    //         passphrase: hex::encode("passphrase"),
    //         data: vec![],
    //     };
    //     buffer.encrypt(&mut secureCellConfig); // encrypt the buffer or the file content
    //     buffer
    // };

    // // store the encrypted file bytes on redis and return the object id 
    // let id = file.store().await;
    // let mut fileBuffer = Vec::<u8>::fetch(&id).await; // fetch the stored object into the vector of u8 bytes
    // // now we can store the bytes in a file
    // let mut file1 = tokio::fs::File::create("saved.txt").await.unwrap();
    // file1.write(&mut fileBuffer).await;

    // // file chunk streaming
    // let chunkSize = 5;
    // let (tx, mut stream) = tokio::sync::mpsc::channel(100);
    // for b in (0..file.len()).step_by(chunkSize){
    //     let mut end = b + chunkSize; // get from b up to b + chunkSize
    //     if end > file.len(){ // if we reach the end of the bytes
    //         end = file.len(); // the end would be the last element since we've reached the last elem
    //     }
    //     let chunk = &file[b..end];
    //     // TODO - encode each chunk using a codec
    //     // ...
    //     tx.send(chunk.to_vec()).await; // this can be any channel (ws, tcp, p2p)
    // }
    // // gather the whole chunks to form the buffer
    // tokio::spawn(async move{
    //     let mut buffer = vec![];
    //     while let Some(chunk) = stream.recv().await{ // receiving the bytes from a channel, this can be a tcp based channel
    //         // extending the buffer with the received bytes
    //         buffer.extend(chunk);
    //     }
    //     // we have a fullfilled buffer in here contains the file bytes
    // });

    // ============================================


    // // ===========================================================================
    // // ============================== NEURON AGENTS ==============================
    // // ===========================================================================
    // // neuron stemlib is an agent used to build an actor worker bot which can be
    // // a server and client to send and receive messages remotely and locally
    
    // let getAgents = ctx.env.agents;
    // let agents = getAgents.lock().await;

    // let mut errorTracer = agents.get(0).unwrap().to_owned(); // it's an error tracer and can be used to send runtime errors to rmq queue
    // let mut neuron = agents.get(1).unwrap().to_owned();

    // // redisConn must be mutable and since we're moving it into another thread 
    // // we should make it safe to gets moved and mutated using Arc and Mutex
    // let redisPool = setupRedis().await;
    // let Ok(pool) = redisPool else{
    //     return;
    // };

    // let clonedRedisPool = pool.clone();
    // let redisConn = clonedRedisPool.get().await.unwrap();
    // let clonedRedisConn = Arc::new(tokio::sync::Mutex::new(redisConn));

    
    // let getNeuronWallet = neuron.wallet.as_ref().unwrap();
    // let getNeuronId = neuron.peerId.to_base58();

    // let neuronWallet = neuron.wallet.as_ref().unwrap();
    // let executor = neuron.internal_executor.clone();

    // /* --------------------------
    //     execution thread process for solving future:
    //     await on async task suspend it to get the result but won't block thread 
    //     means the light thread can continue executing other tasks
    //     future objects are being done in the background awaiting on or polling  
    //     them tells runtime that we need the result if the future was ready he sends the 
    //     result to the caller otherwise it forces the thread to get another task 
    //     from the eventloop to execute it meanwhile the future is being solved, 
    //     this allows to execute tasks in a none blocking manner 
    // */
    // neuron.runInterval(|| async move{
    //     println!("i'm running every 10 seconds, with retries of 12 and timeout 0");
    // }, 10, 12, 0).await;


    // // --------------------------
    // // ------- sending message through actor mailbox eventloop receiver:
    // // by default actors run on the system arbiter thread using 
    // // its eventloop, we can run multiple instances of an actor 
    // // in parallel with SyncArbiter. 
    // // actor mailbox is the eventloop receiver of actor jobq mpsc channel
    // // which receive messages and execute them in a light thread or process 
  
    // // starting the neuron actor 
    // let neuronComponentActor = neuron.clone().start();
    
    // // sending update state message
    // neuronComponentActor.send(
    //     UpdateState{new_state: 1}
    // ).await;

    // // send shutdown message to the neuron
    // neuronComponentActor.send(ShutDown).await;

    // // send payload remotely using the neuron actor
    // neuronComponentActor.send(
    //     InjectPayload{
    //         payload: String::from("0x01ff").as_bytes().to_vec(), 
    //         method: TransmissionMethod::Remote(String::from("p2p-synapse"))
    //     }
    // ).await;

    // // broadcast
    // neuronComponentActor.send(
    //     Broadcast{
    //         local_spawn: todo!(),
    //         notif_data: todo!(),
    //         rmqConfig: todo!(),
    //         p2pConfig: todo!(),
    //         encryptionConfig: todo!(),
    //     }
    // ).await;

    // // subscribe with callback execution process
    // neuronComponentActor.send(
    //     Subscribe{
    //         p2pConfig: todo!(),
    //         rmqConfig: todo!(),
    //         local_spawn: todo!(),
    //         // this is a callback that will be executed per each received event
    //         callback: Arc::new(|event| Box::pin({

    //             // clone before going into the async move{} scope
    //             let clonedRedisConn = clonedRedisConn.clone();

    //             async move{

    //                 /* ------------------------------------------------------------
    //                 event is the received event, we can send the event in here
    //                 through gRPC or RPC to another service or cache it, 
    //                 for example:
    //                 we're receiving a massive of transactions through subsription 
    //                 process, for each tx we'll send it to the wallet service 
    //                 through gRPC or cache it on redis
    //                 */
  
    //                 //    ... 
    
    //                 // cache event on redis inside the callback
    //                 tokio::spawn(async move{
    //                     let mut redisConn = clonedRedisConn.lock().await;
    //                     let eventId = event.clone().data.id;
    //                     let eventString = serde_json::to_string(&event).unwrap();
    //                     let redisKey = format!("cahceEventWithId: {}", eventId);
    //                     let _: () = redisConn.set_ex(eventId, eventString, 300).await.unwrap(); // cache for 5 mins
    //                 });
    //                 /* ------------------------------------------------------------ */
    
    //             }
    //         })),
    //         decryptionConfig: todo!(),
    //     }
    // ).await.unwrap();

    // // send a request to a neuron over eithre rmq or p2p (req, res model)
    // neuronComponentActor.send(
    //     SendRequest{
    //         rmqConfig: todo!(),
    //         p2pConfig: todo!(),
    //         encryptionConfig: todo!(),
    //     }
    // ).await;

    // // receive a response from a neuron over eitehr rmq or p2p (req, res model)
    // let getResponse = neuronComponentActor.send(
    //     ReceiveResposne{
    //         rmqConfig: todo!(),
    //         p2pConfig: todo!(),
    //         decryptionConfig: todo!(),
    //     }
    // ).await;
    // let Ok(resp) = getResponse else{
    //     panic!("can't receive response from the neuron");
    // };
    // let res = resp.0.await; // await on the pinned box so the future can gets executed

    // // talking between local actors
    // let neuronComponentActor1 = errorTracer.start().recipient();
    // neuronComponentActor
    //     .send(TalkTo{
    //         neuron: neuronComponentActor1,
    //         message: String::from("hello from neuronComponentActor")
    //     }).await;


    // // execute an async io task inside the neuron actor thread priodically
    // neuronComponentActor.send(
    //     ExecutePriodically{
    //         period: 40, // every 40 seconds
    //         job: task!{
    //             {
    //                 println!("inside async io task...");
    //             }
    //         }
    //     }
    // ).await;

    // // execute arbitrary async io task function inside either the actor thread or tokio light thread 
    // neuronComponentActor.send(
    //     Execute(
    //         task!(
    //             { // block logic 
    //                 let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    //                 tx.send(String::from("")).await;
    //                 while let Some(data) = rx.recv().await{
    //                     log::info!("received data in task!()");
    //                 }
    //             } 
    //         ),
    //         true // local spawn, set to true if we want to execute the task inside the actor thread
    //     )
    // ).await;


}