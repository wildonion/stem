


use std::collections::HashMap;
use std::thread;
use std::{sync::{Arc, Weak, RwLock}, cell::RefCell};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

mod cruft;
mod task;
mod actor;



/* 
    read source code 
    convert into tokens
    convert into ast 
    compile (convert ast into bytecode/shellcode)
    vm (execute bytecode/shellcode)
    use bytecode or shellcode to execute it since it's hard to read and find any malware in it


    actor based lang, compiler, vm, p2p protocol talking
    neuron controller component actor (cron scheduling, receive msg, execute task in background thread, talk remotely)
    dsl and macrocosm and feature based for ovm 
    neuron actor talking with rpc (g and capnp) and mpsc job/message/tasks q eventloop, spawn task into the neuron light thread
    rmq,redis,kafka long running task then use short polling or ws 
    build p2p stock market and fintech tx object (see tx.rs)
    crypter to create wallet for each neuron actor and hooper streamer
    condvar, jobq, chan, spawn, send, recv, eventloop, select keywords
    onion lang, compiler and vm supports: actor, spawn, channels, mutex and select, send receive and jobq eventloop
    vm,shard,replica,p2p::wrtc,udp,quic,tcp,ws,noise,gossipsub,kdht,lightthread,neuron actor::addr,chan,mutex,select,spawn,eventloop
    tcp based jobq channels like rpc, rmq, redis, kafka
    add thread safe graph definition for p2p onion protocols based on rpc for talking between outside neurons  
    on ton using circom and noir, zk verifier contract, shifting and binary ops and thecry repo for shellcode injection stem on TON tact
    a p2p protocol like adnl to run file sharing, listener, gateway, ingress, proxy, load balancer, vpn (v2ray) using ngrok
*/


pub fn parse(){

    fn getUser(){}
    let func = getUser as *const (); // () in C style means function pointer in Rust is fn

}

pub fn tokenize(){

}

struct Compiler{

}

struct Vm{

}