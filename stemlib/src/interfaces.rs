

use std::error::Error;
use neuron::StreamError;
use wallexerr::misc::{SecureCellConfig, Wallet};
use crate::*;


pub trait Crypter{
    fn encrypt(&mut self, secure_cell_config: &mut SecureCellConfig);
    fn decrypt(&mut self, secure_cell_config: &mut SecureCellConfig);
}

// used for en(de)crypting data in form of string
impl Crypter for String{
    fn decrypt(&mut self, secure_cell_config: &mut SecureCellConfig){
       
        // encrypt convert the raw string into hex encrypted thus
        // calling decrypt method on the hex string returns the 
        // raw string
        secure_cell_config.data = hex::decode(&self).unwrap();
        match Wallet::secure_cell_decrypt(secure_cell_config){ // passing the redis secure_cell_config instance
            Ok(data) => {

                // update the self by converting the data into string format from its utf8
                *self = std::str::from_utf8(&data).unwrap().to_string();

                secure_cell_config.data = data;
            },
            Err(e) => {

                // don't update data field in secure_cell_config instance
                // the encrypted data remains the same as before.
            }
        };

    }
    fn encrypt(&mut self, secure_cell_config: &mut SecureCellConfig){

        // use the self as the input data to be encrypted
        secure_cell_config.data = self.clone().as_bytes().to_vec();
        
        match Wallet::secure_cell_encrypt(secure_cell_config){
            Ok(encrypted) => {
                
                let stringified_data = hex::encode(&encrypted);
                
                // update the self or the string with the hex encrypted data
                *self = stringified_data;

                // update the data field with the encrypted content bytes
                secure_cell_config.data = encrypted; 

            },
            Err(e) => {
                
                // don't update data field in secure_cell_config instance
                // the raw data remains the same as before.
            }
        };

    }

}


pub trait OnionStream{
    type Model;
    async fn on<R: std::future::Future<Output = ()> + Send + Sync + 'static, 
        F: Clone + Fn(Event, Option<StreamError>) -> R + Send + Sync + 'static>
        (&mut self, streamer: &str, eventType: &str, callback: F) -> Self::Model;
}

impl OnionStream for NeuronActor{
    type Model = NeuronActor;
    async fn on<R: std::future::Future<Output = ()> + Send + Sync + 'static, 
            F: Clone + Fn(Event, Option<StreamError>) -> R + Send + Sync + 'static>
            (&mut self, streamer: &str, eventType: &str, callback: F) -> Self::Model {
        
        // execute callback instead of directly caching and storing the received 
        // or sent events on redis or inside db , the process can be done inside 
        // the callback instead of handling it in here
        
        let get_internal_executor = &self.clone().internal_executor;
        let get_events = get_internal_executor.buffer.events.lock().await;
        let cloned_get_internal_executor = get_internal_executor.clone();
        let last_event = get_events.last().unwrap();

        // in order to use * or deref mark on last_event the Copy trait must be implemented 
        // for the type since the Copy is not implemented for heap data types thus we should 
        // use clone() method on them to return the owned type.
        let owned_las_event = last_event.clone();

        match eventType{
            "send" => {
                
                match streamer{
                    "local" => {
                        // sending in the background
                        let first_token_last_event = owned_las_event.clone();
                        
                        // spawn an event for the executor, this would send the event into the channel
                        // in the background lightweight thread
                        tokio::spawn(async move{
                            match cloned_get_internal_executor.spawn(first_token_last_event).await{
                                Ok(this) => {
                                    tokio::spawn(
                                        callback(
                                            owned_las_event.to_owned(), 
                                            None
                                        )
                                    );
                                },
                                Err(e) => {
                                    tokio::spawn(
                                        callback(
                                            owned_las_event.to_owned(), 
                                            Some(StreamError::Sender(e.source().unwrap().to_string()))
                                        )
                                    );
                                }
                            }
                        });

                        self.clone()

                    },
                    "rmq" => {
                        self.clone()
                    },
                    _ => {
                        log::error!("unknown streamer!");
                        self.clone()
                    }
                }
            },
            "receive" => {
                
                match streamer{
                    "local" => {
                        // running the eventloop to receive event streams from the channel 
                        // this would be done in the background lightweight thread, we've passed
                        // the callback to execute it in there
                        tokio::spawn(async move{
                            cloned_get_internal_executor.run(callback).await;
                        });
                        self.clone()
                    },
                    "rmq" => {
                        self.clone()
                    },
                    _ => {
                        log::error!("unknown streamer!");
                        self.clone()
                    }
                }

            },
            _ => {
                log::info!("unknown event type!");
                self.clone()
            }
        }

    }
    
}