

// dsl for neuron actor using macrocosm crate 
// -----------------------------------
/* Neuron Actor DSL Features

    tools: stem, GPT tryout section in ovm.spec, desktop books for neuroscience and information theory
    service logic: 
        ratelimiting, caching and interval execution with redis pubsub key exp chan
        api gateway, redis, pg, elastic, crypter and services: orders, rest management, paytx, reservation and searching, auth,
        develop apis using salvo http3 certed(ws realtiming over rmq streamer, jobId shortpolliong, middlewares, express based syntax, app ctx);
        talking between services using neuron component actor (run intervally, pubsub streaming and rpc using rmq/p2p, executor eventloop jobq chan for local message passing, lightthread)
        dockerizing and ci/cd pipeline for automatic deployment and version bumping and publishing the crate 
        atomic purchasing, booking and execution for tx order object (prevent double spending) and a product using select, spawn, mutex and executor eventloop jobq chan
        tokio time loop spawn, select, executor eventloop jobq chan, timeout, mutex, rwlock, signal
    features:
        vm using macro, neuron codec, supports ws, and jobId with CronScheduler for shortpolling 
        remote talking through p2p and rpc rmq req-rep queue
        p2p docs https://docs.ipfs.tech/concepts/libp2p/ , https://docs.libp2p.io/
        p2p based like adnl file sharing, vpn, gatewat, loadbalancer, proxy, ingress listener like ngrok and v2ray
        p2p based like adnl onchain broker stock engine (find peers which are behind nat over wan)
    main concepts:
            use Box::pin(async move{}) or Arc::pin(async move{})  to return async future object in none async context that you can't await on future objects
            make everything cloneable and break the cycle of self ref types using Arc and store on the heap using Box 
            mutex, channel, spawn, select, trait closure for poly, dep inj, dyn and stat dispatch,
            future objects in form of dyn dispatch with Box::pin(async move{}) or a job in closure return type with Arc::new(||async move{})
            raft,dag,mdp,adjmat,merkletree,shard,replica,p2p::wrtc,udp,quic,tcp,ws,noise,gossipsub,kdht,swarm,lightthread
            atomic,chan,arc,mutex,select,spawn,eventloop,CronScheduler,send,sync,static, static lazy arc mutex
            thread_local, actor id or address, std::alloc, jemalloc, GlobalAlloc, bumpalo and r3bl_rs_utils arena as a global allocator
            zero copy, null pointer optimiser, unique storage key, dsl and macrocosm and feature based for ovm
            atomic transaction: ALL OR NONE with atomic syncing using static lazy Arc Mutex & Channels 
            default type param, default const in struct, let ONION = const{}, Arc<Mutex< to mutate arced value vs Rc<RefCell< to mutate Rced value
            interfaces and traits for poly, stat dyn dispatch, access types through a single interface, Any trait, dep injection
            if you don't care about the result of the io task don't await on the spawn otherwise use static lazy arc mutex or chans and let the task gets executed in the background thread
            spawn(async move{handleMsg().await}) in the background light thread (none blocking) without awaiting: thread per each task
            Box::pin(async move{}), Arc::pin(async move{}) and Arc<Fn() -> R> where R: Future + Send + Sync
            eventloop with spawn(async move{loop{select!{}}}) and spawn(async move{while let Some(job) = rx.recv().await{}}) inside the actor.rs of the ovm 
            CronScheduler(time, ctx, redis pubsub exp key), select!{} awaiting, arc, mutex, timeout, Box::pin(async{}), Arc::pin(async move{}), condvar, jobq chan send recv
    

        neuron onion{
            {
                cells: 25,
                shellcodeConfig: {
                    shellcode: 0x...,
                    encryption: aes256
                },
                nestedData: {
                    key: value
                }
            }
        }

        every neuron actor must be able to do streaming either locally or remotely 
        by using one of the following syntax:
        stream!{
            local stream over onion: // backed by jobq mpsc
                data -> | () => { 
                    // Send message logic
                }),
                data <- | () => { 
                    // Receive message logic
                });
            
            remote stream over onion:  // backed by rmq
                data -> | () => { 
                    // Send message logic
                }),
                data <- | () => { 
                    // Receive message logic
                });
        }

*/


#[macro_export]
macro_rules! neuron {
    () => {

    };
}

#[macro_export]
macro_rules! stream {
    () => {
        
    };
}