


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
use interfaces::OnionStream;
use crate::dsl::*;


const APP_NAME: &str = "STEM";

mod cruft;
mod task;
mod actor;
mod neuron;
mod tx;
mod interfaces;
mod dsl;
mod test;


pub async fn upAndRun(){
    log::info!("up and running from stemlib...");
}