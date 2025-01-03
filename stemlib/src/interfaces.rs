

use std::error::Error;
use crate::messages::*;
use crate::impls::*;
use crate::dto::*;
use wallexerr::misc::{SecureCellConfig, Wallet};
use crate::*;
use salvo::Router;


pub trait ShaHasher{
    fn hashMe(&mut self);
}

pub trait Crypter{
    fn encrypt(&mut self, secure_cell_config: &mut SecureCellConfig);
    fn decrypt(&mut self, secure_cell_config: &mut SecureCellConfig);
}

pub trait OnionStream{
    type Channel;
    async fn on<R: std::future::Future<Output = ()> + Send + Sync + 'static, 
        F: Clone + Fn(Event, Option<StreamError>) -> R + Send + Sync + 'static>
        (&mut self, streamer: &str, eventType: &str, callback: F) -> Self;
}

/// a distributed object storage interface supports object and instances and files (video, audio and image)
/// it uses a distributed hash table to store and retreive objects
pub trait ObjectStorage{ // it can be any bytes io or &[u8], an encoded instance of an struct or a file

    /// save the object to the storage returns object Id
    async fn store(&mut self) -> String;
    /// load the object from the storage as u8 bytes
    async fn fetch(key: &str) -> Vec<u8>;
    /// comapare the current checksum against the passed in object id, this is useful to detect steghided object
    fn checksum(&mut self, objId: &str) -> bool; 
}

pub trait Service: Send + Sync + 'static{
    // build router tree for the current dto
    // the trait is generic over any router 
    fn startService(&self, host: &str, port: u16);
}