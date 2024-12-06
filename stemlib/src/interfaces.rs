

use std::error::Error;
use crate::messages::*;
use crate::impls::*;
use crate::dto::*;
use wallexerr::misc::{SecureCellConfig, Wallet};
use crate::*;



pub trait ShaHasher{
    fn hash(&mut self);
}

pub trait Crypter{
    fn encrypt(&mut self, secure_cell_config: &mut SecureCellConfig);
    fn decrypt(&mut self, secure_cell_config: &mut SecureCellConfig);
}

pub trait OnionStream{
    type Model;
    async fn on<R: std::future::Future<Output = ()> + Send + Sync + 'static, 
        F: Clone + Fn(Event, Option<StreamError>) -> R + Send + Sync + 'static>
        (&mut self, streamer: &str, eventType: &str, callback: F) -> Self::Model;
}

pub trait ServiceExt: Send + Sync + 'static{
    fn start(&mut self);
    fn status(&self);
}

pub trait ServiceExt1{
    fn startService(&mut self);
    fn stopService(&mut self);
}


pub trait ObjectStorage{ // the trait supports polymorphism over the fId

    type Driver;
    async fn save(&mut self);
    async fn getFile(&mut self, fId: String) -> &[u8];
    // comapare the current checksum against the passed in file 
    // this is useful to detect steghided pictures and files
    fn checksum(&mut self, file: &mut [u8]) -> bool; 
}