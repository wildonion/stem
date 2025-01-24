

use std::error::Error;
use crate::messages::*;
use crate::impls::*;
use crate::dto::*;
use bytes::Bytes;
use tokio::sync::mpsc::Receiver;
use wallexerr::misc::{SecureCellConfig, Wallet};
use crate::*;
use salvo::Router;
use tokio::sync::Mutex;


pub trait ShaHasher{
    fn hashMe(&mut self);
}

pub trait Crypter{
    fn encrypt(&mut self, secure_cell_config: &mut SecureCellConfig);
    fn decrypt(&mut self, secure_cell_config: &mut SecureCellConfig);
}

pub trait Channel{
    async fn on<F, R>(&mut self, eventType: &str, callback: F) -> Self
    where F: Fn(Event, Option<ChanError>) -> R + Send + Sync + 'static, 
    R: Future<Output = ()> + Send + Sync + 'static;
}

/// a distributed object storage interface supports object and instances and files (video, audio and image)
/// it uses a distributed hash table to store and retreive objects
pub trait ObjectStorage{ // it can be any bytes io or &[u8], an encoded instance of an struct or a file

    /// save the object to the storage returns object Id
    async fn store(&mut self) -> String;
    /// load the object from the storage as u8 bytes
    async fn fetch(key: &str) -> Vec<u8>;
    /// load the object from the storage by streaming over its chunk
    async fn fetchChunk(key: &str) -> impl Stream<Item = Result<Bytes, deadpool_redis::redis::RedisError>>;
    /// load the object into the ram and send each chunk to the channel
    async fn fetchChunkChan(key: &str) -> Arc<Mutex<Receiver<Vec<u8>>>>;
    /// comapare the current checksum against the passed in object id, this is useful to detect steghided object
    fn checksum(&mut self, objId: &str) -> bool; 
}

pub trait Service: Send + Sync + 'static{ // don't inheritence from Serialize and Deserialize cause it can't be object safe trait
    // build router tree for the current dto
    // the trait is generic over any router 
    fn startService(&self, host: &str, port: u16);
    fn getServiceInfo(&self) -> String;
}

pub trait PubSub: Send + Sync + 'static{
    async fn subscribe(&mut self, topic: &str) -> Arc<Mutex<Receiver<String>>>;
    async fn publish(&self, topic: &str, data: &str);
}

/// an state machine to set and get state
pub trait Fsm{
    type Engine; // redis or hashMap or btreeMap
    async fn setState(&mut self, key: &str, state: &str);
    async fn getState(&mut self, key: &str) -> Result<String, FsmEngineError>;
}