



mod schemas;
mod mathista;
use schemas::Neuron;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use daemon;
use actix::prelude::*;








#[tokio::main(flavor="multi_thread", worker_threads=10)] //// use the tokio multi threaded runtime by spawning 10 threads
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{ //// bounding the type that is caused to error to Error, Send and Sync traits to be shareable between threads and have static lifetime across threads and awaits; Box is an smart pointer which has valid lifetime for what's inside of it, we're putting the error part of the Result inside the Box since we have no idea about the size of the error or the type that caused this error happened at compile time thus we have to take a reference to it but without defining a specific lifetime
    


    // https://github.com/mozilla/cbindgen -> generate c bindings and .so from rust code
    // https://github.com/wildonion/cs-concepts
    // https://github.com/alordash/newton-fractal
    // https://github.com/Patryk27/shorelark/ -> GA, NN and WASM
    // https://crates.io/crates/wasmtime
    // https://wasmer.io/
    // TODO - building to wasm using wasmer and wastime   
    // ...




    Ok(())



}
