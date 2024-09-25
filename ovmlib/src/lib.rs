


use std::collections::HashMap;
use std::thread;
use std::{sync::{Arc, Weak, RwLock}, cell::RefCell};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

mod cruft;
mod task;
mod actor;

/* ---------------------------------- see also gvm.spec

    TODO:
        build stem with ovm (crypter based communication and footprinting over notif broker actor protocols)
        ipfs p2p and graph algos
        read source code -> tokens -> ast -> compile (convert ast into bytecode/shellcode) -> vm (execute bytecode/shellcode)
        all hooper protocols (p2p onion) in notifbrokeractor component with rmq rpc supports 
        sending shellcode through the notifbrokeractor and hooper like hack bank fintech router with ovm 
    ovm and compiler for onion lang with p2p protocol like adnl for: 
        file sharing 
        vpn like v2ray and proxy or listener like ngron
        gateway loadbalancer ingress
        onchain broker 
    infra by:
        vm,shard,replica,p2p::wrtc,udp,quic,tcp,ws,noise,gossipsub,kdht,lightthread and hooper streamer
        actor addr,chan,arc,mutex,select,spawn,eventloop,cronscheduler,send,sync,static, static lazy arc mutex
        generate hash of a seed with the highest entropy for rng keypair 
        crypter to create wallet for each neuron actor and hooper streamer (wannacry crypter)
        on ton using circom and noir, zk verifier contract, shifting and binary ops and thecry repo for shellcode injection stem on TON tact
    backed by NotifBrokerActorComponent:
        each neuron component is an actor like a smart contract that can talk either 
            in clientside with shortpolling jobId and ws channels
            in realtime remotely through brokers(redis, rmq, kafka, rpc: json, protobuf, capnproto)
            and locally with msg sending pattern (address, mpsc, mailbox)
            and between other actors remotely with rpc in req-rep manner using rmq rpc queues
    tools: wallexerr, macrocosm, stem
    concepts:
        dsl and macrocosm and feature based for ovm and atomic transaction: ALL OR NONE with atomic syncing using Mutex & Channels 
        default type param, default const in struct, let ONION = const{}, Arc<Mutex< to mutate arced value vs Rc<RefCell< to mutate Rced value
        interfaces and traits for poly, stat dyn dispatch, access types through a single interface, Any trait, dep injection
        if you don't care about the result of the io task await on the spawn otherwise use mutex or chans and let the task gets executed in the background thread
        spawn(async move{handleMsg().await}) in the background light thread (none blocking) without awaiting: thread per each task
        Box::pin(async move{}) and Arc<Fn()->R> where R: Future + Send + Sync
        eventloop with loop{select!{}} and while let Some(job) = rx.recv().await{} inside the actor.rs of the ovm 
        cron(time, ctx, redis pubsub exp key), select!{} awaiting, arc, mutex, timeout, Box::pin(async{}), condvar, jobq chan send recv
*/


pub fn parse(){

    fn getUser<F1: std::future::Future<Output=()>, R: Send + Sync + 'static + std::future::Future<Output = ()>>(
        f1: F1,
        f2: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
        f3: std::sync::Arc<dyn Fn() -> R + Send + Sync + 'static> 
    ){}
    fn setUser(){}
    let func = setUser as *const (); // () in C style means function pointer in Rust is fn

}

struct Compiler{

}

struct Vm{

}