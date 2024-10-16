
// -----------------------------------
/* Neuron Actor DSL using Macros

    TODOs: dsl.rs, select!{}, publishToRmq() and consumeFromRmq() methods, 
    TODOs: stem.spec, express framework, publish stemlib to crate
    tools: 
        macrocosm crate, GPT tryout section in stem.spec, 
        desktop books for neuroscience and information theory
    features:
        local talking with jobq chan eventloop receiver mailbox
        remote talking through p2p and rpc rmq req-rep queue:
            p2p docs https://docs.ipfs.tech/concepts/libp2p/ , https://docs.libp2p.io/
            p2p based like adnl file sharing, vpn like tor and v2ray, firewall, proxy and dns over neuron actor, gateway, loadbalancer, ingress listener like ngrok
            p2p based like adnl onchain broker stock engine (find peers which are behind nat over wan)
    main concepts:
        dyn dispatch and dep injection with Arc::new(TypeImplsTrait{}) Box::new(TypeImplsTrait{}) Arc::pin(async move{}), Box::pin(async move{})
        stat dyn dispatch, dep injecton and binding to trait, Box::pin(async move{}), Arc::pin(async move{}), Arc<Fn() -> R> where R: Future
        Err(CstmError::new()) || ? on object of type CstmError who impls Error, From, Display traits || Err(Box::new(CstmError::new()))
        use Box::pin(async move{}) or Arc::pin(async move{})  to return async future object in none async context that you can't await on future objects
        make everything cloneable and break the cycle of self ref types using Arc and store on the heap using Box 
        mutex, channel, spawn, select, trait closure for poly, dep inj, dyn and stat dispatch,
        future objects in form of dyn dispatch with Box::pin(async move{}) or a job in closure return type with Arc::new(||async move{})
        raft,dag,mdp,adjmat,merkletree,shard,replica,p2p::wrtc,udp,quic,tcp,ws,noise,gossipsub,kdht,swarm,lightthread
        atomic,chan,arc,mutex,select,spawn,eventloop,CronScheduler,send,sync,static, static lazy arc mutex app ctx + send + sync
        thread_local, actor id or address, std::alloc, jemalloc, GlobalAlloc, bumpalo and r3bl_rs_utils arena as a global allocator
        zero copy, null pointer optimiser, unique storage key, dsl and macrocosm and feature based for stem
        atomic transaction: ALL OR NONE with atomic syncing using static lazy Arc Mutex & Channels 
        default type param, default const in struct, let ONION = const{}, Arc<Mutex< to mutate arced value vs Rc<RefCell< to mutate Rced value
        interfaces and traits for poly, stat dyn dispatch, access types through a single interface, Any trait, dep injection
        if you don't care about the result of the io task don't await on the spawn otherwise use static lazy arc mutex or chans and let the task gets executed in the background thread
        spawn(async move{handleMsg().await}) in the background light thread (none blocking) without awaiting: thread per each task
        Box::pin(async move{}), Arc::pin(async move{}) and Arc<Fn() -> R> where R: Future + Send + Sync
        eventloop with spawn(async move{loop{select!{}}}) and spawn(async move{while let Some(job) = rx.recv().await{}}) inside the actor.rs of the stem 
        CronScheduler(time, ctx, redis pubsub exp key), select!{} awaiting, arc, mutex, timeout, Box::pin(async{}), Arc::pin(async move{}), condvar, jobq chan send recv

        proc ones can only be used on top of functions and impl blocks to extend 
        the function body and add some vars into it at compile time:
            use proc macro with attrs to handle automatic task spawning and jobq chan creations
            use decl macro to build dsl

        struct ProcessStruct;

        // also handle multiple passing params to function using macros with omitting the useless ones
        // do this:    
        #[processExt]
        impl ProcessStruct{
            async fn start(&self){}
            async fn stop(&self){}
        }
        // instead of:
        impl processExt for ProcessStruct{} 


        // when a function is annotated with distribute 
        // means it can distribute itself among networks
        #[distribute]
        pub async fn injectShellcodeLogic(){
        }

*/

use crate::*;

// build instance from neuron actor
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
        println!("run me every 10 seconds, with retries of 12 and timeout 0");
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

// a neuron is an actor, an isolated state talks locally and remotely through 
// eventloop jobq channel and rpc rmq + p2p
#[macro_export]
macro_rules! neuron {
    () => {
        {
            
        }
    };
}

// stream is tool helps to start streaming over a neuron actor 
// either locally or remotely  
#[macro_export]
macro_rules! stream {
    () => {
        {

        }
    };
}

// layer contains one or more neurons inside itself, multiple 
// layers form an onion brain
#[macro_export]
macro_rules! layer {
    () => {
        {

        }
    };
}