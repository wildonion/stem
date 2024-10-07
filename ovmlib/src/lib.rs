


use std::collections::HashMap;
use std::thread;
use std::{sync::{Arc, Weak, RwLock}, cell::RefCell};
use neuron::{Buffer, Event, EventData, EventStatus, InternalExecutor, NeuronActor, UpdateState, Worker};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use actix::{prelude::*, spawn};
use actix::Handler as ActixMessageHandler;
use uuid::Uuid;
use std::sync::atomic::AtomicU64;


const APP_NAME: &str = "OVM";

mod cruft;
mod task;
mod actor;
mod neuron;
mod tx;
mod misc;
mod dsl;

pub async fn upAndRun(){

    let mut neuron = NeuronActor{
        wallet: Some(wallexerr::misc::Wallet::new_ed25519()),
        internal_executor: InternalExecutor::new(
            Buffer{ events: std::sync::Arc::new(tokio::sync::Mutex::new(vec![
                Event{ data: EventData::default(), status: EventStatus::Committed, offset: 0 }
            ])), size: 100 }
        ),
        metadata: None,
        internal_worker: None,
        transactions: None,
        contract: None,
        state: 0
    };

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
    neuron.runInterval(||async move{
        println!("run me every 10 seconds, with retries of 12 and timeout 0");
    }, 10, 12, 0).await;

    // streaming over neuron
    neuron
        .on("local", "receive", move |event, error| async move{

            if error.is_some(){
                println!("an error accoured in receiving: {:?}", error.unwrap());
            }

            // event is the received event
            // ...
            println!("received task >> {:?}", event);

        }).await
        .on("rmq", "send", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured in sending: {:?}", error.unwrap());
            }
            
            // event is the sent event
            // ...
            println!("sent task >> {:?}", event);

        }).await;

    
    // starting the neuron actor 
    let neuron_actor = neuron.start();
    
    // send update state message
    neuron_actor.send(UpdateState{new_state: 1}).await;


}