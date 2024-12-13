

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::sync::{Arc, Condvar};
use std::thread::{self, JoinHandle};
use crate::*;
use clap::error;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Connection;
use interfaces::{Crypter, ObjectStorage};
use salvo::{FlowCtrl, Router};
use sha2::digest::generic_array::arr;
use sha2::digest::Output;
use stemlib::dto::{Neuron, TransmissionMethod, *};
use stemlib::messages::*;
use stemlib::interfaces::OnionStream;
use tokio::io::AsyncReadExt;
use wallexerr::misc::SecureCellConfig; // import the interface to use the on() method on the Neuron instance
use stemlib::dsl::*;



#[tokio::test]
pub async fn onionEnv(){


    //============== a ctx has an environment and containers
    // environment: executor and agents / executor: runner

    let entityContainer = Container{
        service: Arc::new(EntityDto), 
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

    // start both containers as actors
    let entityContainerActor = entityContainer.start();
    let uploadDriverContainerActor = uploadDriverContainer.start();
    
    // entityContainerActor wants to talk with the uploadDriverContainerActor
    entityContainerActor.send(
        TalkToContainer{
            msg: MsgType::Serve, 
            container: uploadDriverContainerActor.clone().recipient()
        }
    ).await;
    let ctx = AppContext::new().await
        .pushContainer(entityContainerActor)
        .pushContainer(uploadDriverContainerActor);

    // get the first contianer actor
    let c1 = ctx.containers.get(1).unwrap();

    // neuron agents
    let getAgents = ctx.env.agents;
    let agents = getAgents.lock().await;

    let mut neuron1 = agents.get(0).unwrap().to_owned();
    let mut neuron = agents.get(1).unwrap().to_owned();

    
    // redisConn must be mutable and since we're moving it into another thread 
    // we should make it safe to gets moved and mutated using Arc and Mutex
    let arcedRedisConn = setupRedis().await;
    let clonnedRedisConn = arcedRedisConn.clone();

    
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


}


pub async fn setupRedis() -> Arc<tokio::sync::Mutex<Connection>>{
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

    Arc::new(tokio::sync::Mutex::new(redisConn))
}