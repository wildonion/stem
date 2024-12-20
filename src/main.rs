



mod tests;

use dto::Neuron;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use actix::prelude::*;
use serde_json::json;
use tokio::{net::unix::SocketAddr, task};
use stemplugins::*;
use stemlib::*;



#[actix_web::main] // use actix_web main context since we have actors
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    
    Ok(())

}