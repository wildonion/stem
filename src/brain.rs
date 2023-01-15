



mod schemas;
mod mathista;
use schemas::Neuron;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use riker::actors::*;
use riker::system::ActorSystem;
use riker_patterns::ask::*; //// used to ask any actor to give us the info about or update the state of its guarded type 
use daemon;









#[tokio::main(flavor="multi_thread", worker_threads=10)] //// use the tokio multi threaded runtime by spawning 10 threads
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{ //// bounding the type that is caused to error to Error, Send and Sync traits to be shareable between threads and have static lifetime across threads and awaits; Box is an smart pointer which has valid lifetime for what's inside of it, we're putting the error part of the Result inside the Box since we have no idea about the size of the error or the type that caused this error happened at compile time thus we have to take a reference to it but without defining a specific lifetime
    

    
    daemon::bpf_loader().await;


    // near, cloudflare and shuttle are serverless:
    //      - write contract or serverless or faas methods in rust then compile to wasm
    //      - deploy using cli to the runtime server like coiniXerr node 
    //      - high performence proxy like pingora and k8s will balance the requests  
    //      - load the deployed code in js or the rust and call its methods
    //
    //// near will load the wasm contract inside its nodes which is
    //// written in rust to change the state of the blockchain
    //// whenever one of the contract method gets called from the js
    //// like funding an account once the fund() method gets called 
    //// from the contract.
    //
    //// the reason that near contract gets compiled to wasm is because 
    //// they can be loaded inside the browsers and also they have 
    //// no access to socket and std libs thus they secured, immutable and 
    //// can not communicate with outside world.
    //
    //// the reason that solana contract gets compiled to .so is because 
    //// they can be loaded from the linux kernel which is blazingly 
    //// fast also from the browsers, a json RPC call must be invoked 
    //// with a contract method name to the RPC server on the runtime node 
    //// in which it can load the .so contract which has bee deployed 
    //// that contains the BPF bytecode in it and can call the method name
    //// inside the incoming RPC request to change the state of the blockchain.

    // https://github.com/wildonion/cs-concepts
    // https://github.com/alordash/newton-fractal
    // https://github.com/Patryk27/shorelark/ -> GA, NN and WASM
    // https://crates.io/crates/wasmtime
    // https://wasmer.io/
    // TODO - building to wasm using wasmer and wastime   
    // ...




    Ok(())



}
