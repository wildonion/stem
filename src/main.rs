



mod stream;
mod schemas;
mod mathista;


use schemas::Neuron;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use actix::prelude::*;
use serde_json::json;
use tokio::{net::unix::SocketAddr, task};
use stemplugins::*;



/* 
    Onion lang pipeline
    1 => read source code
    2 => use lexer to generate tokens or token stream from the source code strings and chars
    3 => use parser to generate ast statements from token stream
    4 => build a program from ast and extract bytecodes or shellcode from that 
    5 => execute bytecode/shellcode on vm using opcodes
*/


#[actix_web::main] // use actix_web main context since we have actors
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    stemlib::run().await;

    Ok(())

}