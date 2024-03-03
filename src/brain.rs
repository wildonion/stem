



mod schemas;
mod mathista;
use schemas::Neuron;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use daemon;
use actix::prelude::*;
mod nodify;








#[tokio::main(flavor="multi_thread", worker_threads=10)] //// use the tokio multi threaded runtime by spawning 10 threads

/* 
    if we want to use Result<(), impl std::error::Error + Send + Sync + 'static>
    as the return type of the error part, the exact error type instance must be 
    sepecified also the Error trait must be implemented for the error type (impl 
    Error for ErrorType{}) since we're implementing the Error trait for the error 
    type in return type which insists that the instance of the type implements the 
    Error trait. by returning a boxed error trait we're returning the Error trait 
    as a heap object behind a valid pointer which handles all error type at runtime, 
    this is the solution to return traits as an object cause we don't know what type 
    causes the error at runtiem and is the implementor of the Error trait which 
    forces us to return the trait as the error itself and since traits are dynamically
    sized we can't treat them as a typed object directly we must put them behind 
    pointer like &'valid dyn Trait or box them to send them on the heap, also by 
    bounding the Error trait to Send + Sync + 'static we'll make it sefable, sendable 
    and shareable to move it between different scopes and threads.
*/
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{
    


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
