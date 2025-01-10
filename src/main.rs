



mod tests;

use dto::Neuron;
use serde::{Serialize, Deserialize};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;
use actix::prelude::*;
use serde_json::json;
use tokio::{net::unix::SocketAddr, task};
use stemplugins::*;
use stemlib::*;
use std::env;



#[actix_web::main] // use actix_web main context since we have actors
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    env::set_var("RUST_LOG", "trace");
        env_logger::init();
        
        // logging
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE) // higher than trace like debug, info, warn
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    
    tests::onionEnv().await;
    
    Ok(())

}