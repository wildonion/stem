


use crate::*;
use crate::dsl::*;

#[tokio::test]
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
        internal_locker: None,
        internal_none_async_threadpool: Arc::new(None),
        signal: std::sync::Arc::new(std::sync::Condvar::new()),
        contract: None,
        state: 0 // this can be mutated by sending the update state message to the actor
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
    neuron.runInterval(|| async move{
        println!("i'm running every 10 seconds, with retries of 12 and timeout 0");
    }, 10, 12, 0).await;

    // streaming over neuron, the on() method belongs the StreamRmq interface
    // enables us to start streaming over a neuron object either locally or 
    // remotely, the method however takes a callback closure in which we have
    // access to the sent or received event.
    neuron
        .on("local", "receive", move |event, error| async move{

            if error.is_some(){
                println!("an error accoured during receiving: {:?}", error.unwrap());
                return;
            }

            println!("received task: {:?}", event);

            // do whatever you want to do with received task:
            // storing in db or cache on redis 
            // ...


        }).await
        .on("rmq", "send", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during sending: {:?}", error.unwrap());
                return;
            }
            
            println!("sent task: {:?}", event);

            // do whatever you want to do with sent task:
            // please shiaf the sent task!
            // ...

        }).await;
    
    // starting the neuron actor 
    let nueronComponentActor = neuron.clone().start();
    
    // sending update state message
    nueronComponentActor.send(UpdateState{new_state: 1}).await;


}