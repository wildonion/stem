


use std::collections::HashMap;
use std::thread;
use std::{sync::{Arc, Weak, RwLock}, cell::RefCell};
use neuron::{Buffer, Event, EventData, EventStatus, InternalExecutor, NeuronActor, Worker};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use actix::prelude::*;
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

pub async fn someMethod(){

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
    };


    // streaming over neuron
    neuron
        .on("local", "receive", move |event| async move{

            // event is the received event
            // ...
            println!("received task >> {:?}", event);

        }).await
        .on("rmq", "send", move |event| async move{
            
            // event is the sent event
            // ...
            println!("sent task >> {:?}", event);

        }).await;

    
    // starting the neuron actor 
    let neuron_actor = neuron.start();
    
    // now we could send message to the neuron actor
    // to start consuming / producing or talk remotely through rmq rpc
    // ...


}