


use std::collections::HashMap;
use std::thread;
use std::{sync::{Arc, Weak, RwLock}, cell::RefCell};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

mod cruft;
mod task;
mod actor;
mod neuron;


type AssistId = String;
struct Run<R, O, C: Fn() -> R + Send + Sync + 'static> 
where R: std::future::Future<Output = O>, O: Send + Sync + 'static{
    pub assistId: AssistId,
    pub function: std::sync::Arc<C>
}


/* ---------------------------------- see also gvm.spec

    TRY WITH GPT 01

    read source code -> tokens -> ast -> compile (convert ast into bytecode/shellcode) -> vm (execute bytecode/shellcode)
    every neuron in stem in onion lang is a NotifBrokerActorComponent object which supports:
        running and executing onion functions on the vm inside the background light thread of tokio spawn like executing serverless functions on the cloud
        crypter wallexerr wallet (hight entropy hashed seed for rng keypair) for encryption footprinting through circom and noir, zk verifier contract
        keypair in form of base64/58 or hex and encrypting media using base64 or aes256 
        sending encrypted neuron shellcode or bytecode remotely to other neuron actors 
        a websocket mpsc sender to send received notif to websocket server
        talking remotely through p2p, rmq rpc req-rep queue, kafka and redis
        talking locally through mpsc mailbox jobq eventloop arc mutex receiver and actor address
        data codec with json, protofbuf and capnp
        realtiming in client using JobId and CronScheduler actor (correlationId) shortpolling and websocket
        dsl macrocosm for stem framework (stream!{}.on(), contract!{}, product!{}, atomic_synapse!{}) and component!{} macro to generate NotifBrokerActorComponent instance
        p2p based like adnl file sharing, vpn, gatewat, loadbalancer, proxy, ingress listener like ngrok and v2ray
        p2p based like adnl onchain broker stock engine (find peers which are behind nat over wan)
        creating stream eventloop for an sepecifc neuron by starting a NotifBrokerActorComponent actor backed by different brokers:
        let stream = stream(neuron_id, "rmq") // stream backed by rmq (redis, kafka)
            .on("receive", |data| async move{

            })
            .on("send", |data| async move{

            });
    tools: crypter, wallexerr, hoopoe, macrocosm, stem, shifting and binary ops using thecry repo
    tools: desktop books for neuroscience and information theory, GPT tryout section in ovm.spec
    main concepts:
        raft,dag,mdp,adjmat,merkletree,shard,replica,p2p::wrtc,udp,quic,tcp,ws,noise,gossipsub,kdht,swarm,lightthread
        atomic,chan,arc,mutex,select,spawn,eventloop,CronScheduler,send,sync,static, static lazy arc mutex
        thread_local, actor id or address, std::alloc, jemalloc, GlobalAlloc, bumpalo and r3bl_rs_utils arena as a global allocator
        zero copy, null pointer optimiser, unique storage key, dsl and macrocosm and feature based for ovm
        atomic transaction: ALL OR NONE with atomic syncing using static lazy Arc Mutex & Channels 
        default type param, default const in struct, let ONION = const{}, Arc<Mutex< to mutate arced value vs Rc<RefCell< to mutate Rced value
        interfaces and traits for poly, stat dyn dispatch, access types through a single interface, Any trait, dep injection
        if you don't care about the result of the io task don't await on the spawn otherwise use static lazy arc mutex or chans and let the task gets executed in the background thread
        spawn(async move{handleMsg().await}) in the background light thread (none blocking) without awaiting: thread per each task
        Box::pin(async move{}) and Arc<Fn()->R> where R: Future + Send + Sync
        eventloop with spawn(async move{loop{select!{}}}) and spawn(async move{while let Some(job) = rx.recv().await{}}) inside the actor.rs of the ovm 
        CronScheduler(time, ctx, redis pubsub exp key), select!{} awaiting, arc, mutex, timeout, Box::pin(async{}), condvar, jobq chan send recv
*/


pub fn parse(){

}

struct Compiler{

}

struct Vm{

}