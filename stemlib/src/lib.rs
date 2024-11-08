

/*  ========================================================================================
    
    What is a Neuron?
        a neuron is an actor component object or a process or a light thread, the building 
        block of everything it's an smart object on its own which can communicate locally and 
        remotely with other neurons through RMQ, P2P or MPSC JobQ channel, every neuron contains 
        a metadata field which carries info about the actual object and informations that are 
        being passed through the synapses protocols. 

    brokering is all about queueing, sending and receiving messages way more faster, 
    safer and reliable than a simple eventloop or a tcp based channel. 
    all brokers contains message/task/job queue to handle communication between services
    asyncly, they've been designed specially for decoupling tight long running services 
    due to their durability nature like giving predicting output of an ai model service
    while the ai model is working on other input prediction in the background we can 
    receive the old outputs and pass them through the brokers to receive them in some 
    http service for responding the caller.   
    In rmq producer sends message to exchange the a consumer can bind its queue to 
    the exchange to receive the messages, routing key determines the pattern of receiving 
    messages inside the bounded queue from the exchange 
    In kafka producer sends messages to topic the consumer can receives data from 
    the topic, Rmq adds an extra layer on top of msg handling logic which is creating 
    queue per each consumer.
    offset in kafka is an strategy which determines the way of tracking the sequential 
    order of receiving messages by kafka topics it's like routing key in rmq 
    in kafka you should create consumer and producer separately but in rmq everything is 
    started from a channel, we'll create a channel to declare the queue, exchange, consumer 
    and producer in that channel, channel is a thread that can manage multiple connection 
    to the broker through a single tcp connection.

    BROKER TYPES: (preferred stack: RMQ + RPC + WebSocket + ShortPollingJobId)
        → REDIS PUBSUB => light task queue
        → KAFKA        => high latency hight throughput
            -ˋˏ✄┈┈┈┈
            >_ topic contains messages allows consumers to consume from topics each topic 
            can be divided into multiple partitions for example a topic might have 10 
            partitions, each message can have its own unique partition key which specifies 
            to which partition the message will go. Offset can be assigned to each message
            within a partition it specifies the position of the message in that partition
            it's useful for the consumers to resume consuming where they've left.
            a consumer can commit the message after processing it which tells kafka that
            the consumer has received and processed the message completely.
            single consumer consumes messages from specific partitions but in group of 
            consumers kafka ensures that each partition is consumed by only one consumer 
            within the group like if a topic with 4 partitions and 2 consumers are in the 
            same group, Kafka will assign 2 partitions to each consumer:

                                                                 ----------> partition-key1 queue(m1, m2, m3, ...) - all messages with key1
             ---------                       ------------       |
            |consumer1| <-----consume-----> |Kafka Broker| <-----topic-----> partition-key3 queue(m1, m2, m3, ...) - all messages with key3
             ---------                       ----------- |      |
                |_______partition1&2______________|      |      |----------> partition-key2 queue(m1, m2, m3, ...) - all messages with key2
             ---------                            |      |      |
            |consumer2|                           |      |       ----------> partition-key4 queue(m1, m2, m3, ...) - all messages with key4
             ---------                            |      |
                 |                                |   ---------
                  --------------partition3&4------   | producer|
                                                      ---------
            it's notable that committing the offset too early, instead, might cause message 
            loss, since upon recovery the consumer will start from the next message, skipping 
            the one where the failure occurred.

        → RMQ          => low latency low throughput
            -ˋˏ✄┈┈┈┈
            >_ the consuming task has been started by sending the ConsumeNotif message 
            to this actor which will execute the streaming loop over the queue in 
            either the notif consumer actor context itself or the tokio spawn thread:

                notif consumer -----Q(Consume Payload)-----> Exchange -----notif CQRS writer-----> cache/store on Redis & db

            -ˋˏ✄┈┈┈┈
            >_ the producing task has been started by sending the ProduceNotif message 
            to this actor which will execute the publishing process to the exchange 
            in either the notif producer actor context itself or the tokio spawn thread:

                notif producer -----payload-----> Exchange
            
            -ˋˏ✄┈┈┈┈
            >_ client uses a short polling technique or websocket streaming to fetch notifications 
            for an specific owner from redis or db, this is the best solution to implement a
            realtiming strategy on the client side to fetch what's happening on the 
            server side in realtime.

             _________                                      _________
            | server1 | <------- RMQ notif broker -------> | server2 |
             ---------                                      ---------
                | ws                                          | ws
                 ------------------- client ------------------
    ======================================================================================== 
*/


use crate::messages::*;
use crate::schemas::*;
use crate::impls::*;


/*  ====================================================================================
       REALTIME NOTIF EVENT STREAMING DESIGN PATTERN (README files inside docs folder)
    ====================================================================================
                   MAKE SURE YOU'VE STARTED CONSUMERS BEFORE PRODUCING
                   THEY MUST BE READY FOR CONSUMING WHILE PRODUCERS ARE
                   SENDING MESSAGES TO THE BROKER. USUALLY RUN THE PRODUCER
                   USING CLI AND THE CONSUMER AT STARTUP.

    NotifBrokerActor is the worker of handling the process of publishing and consuming 
    messages through rmq, redis and kafka, talking to the NotifBrokerActor can be done 
    by sending it a message contains the setup either to publish or consume something 
    to and from an specific broker, so generally it's a sexy actor to produce/consume 
    messages from different type of brokers it uses RMQ, Redis and Kafka to produce and 
    consume massive messages in realtime, kindly it supports data AES256 encryption 
    through producing messages to the broker. we can send either producing or consuming 
    message to this actor to start producing or consuming in the background.

    how capnpc works?
    in RPC every method call is a round trip in networking, canpnp pack all calls together 
    in only one round trip, it uses the promise pipelining feature which every call is a 
    future object which can be solved by awaiting in which it returns all the results from 
    all the calls sent to the server it's like `foo().bar().end()` takes only 1 round trip 
    which by awaiting on them it returns all the result from the server, it can call methods 
    without waiting just take a round trip. call results are returned to the client before 
    the request even arrives at the server, this is the feature of promise it's a place 
    holder for the result of each call and once we await on them all the results will be 
    arrived in one round trip.
    ************************************************************************************
    it's notable that for realtime push notif streaming we MUST start consuming from
    the specified broker passed in to the message structure when talking with actor, in
    a place where the application logic which is likely a server is being started.
    ************************************************************************************
    
    ====================================================================================
*/

use std::collections::HashMap;
use std::thread;
use std::{sync::{Arc, Weak, RwLock}, cell::RefCell};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use actix::{prelude::*, spawn};
use actix::Handler as ActixMessageHandler;
use uuid::Uuid;
use std::sync::atomic::AtomicU64;
use interfaces::OnionStream;
use crate::dsl::*;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;
use libp2p::{kad, PeerId, Swarm, SwarmBuilder};
use libp2p::{tcp, yamux, gossipsub, mdns, noise,
    swarm::NetworkBehaviour, swarm::SwarmEvent,
    request_response::{self, ProtocolSupport, OutboundRequestId, ResponseChannel, 
        Event as P2pReqResEvent, Codec, Config},
};
use libp2p::identity::Keypair;
use std::error::Error;
use std::future::Future;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::process::Output;
use actor::workerthreadpool::sync::NoneAsyncThreadPool;
use interfaces::ServiceExt;
use task::Task;
use task::TaskStatus;
use tokio::sync::TryLockError;
use tokio::sync::MutexGuard;
use tokio::task::JoinHandle;
use tx::Transaction;
use wallexerr::misc::SecureCellConfig;
use interfaces::{Crypter, ShaHasher};
use clap::{Parser, Subcommand};

const APP_NAME: &str = "STEM";

pub mod cruft;
pub mod task;
pub mod actor;
pub mod tx;
pub mod interfaces;
pub mod dsl;
pub mod schemas; 
pub mod messages;
pub mod impls;
pub mod handlers;