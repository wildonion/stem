

use wallexerr::misc::{SecureCellConfig, Wallet};
use crate::*;


// task is a closure that returns a future object 
pub async fn runInterval<M, R, O>(task: M, period: u64, mut retries: u8)
    where M: Fn() -> R + Send + Sync + 'static,
            R: std::future::Future<Output = O> + Send + Sync + 'static,
{
    let arced_task = std::sync::Arc::new(task);
    tokio::spawn(async move{
        let mut int = tokio::time::interval(tokio::time::Duration::from_secs(period));
        int.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        let mut attempts = 0;
        loop{
            if retries == 0{
                continue;
            }
            if attempts >= retries{
                break;
            }
            int.tick().await;
            arced_task().await;
            attempts += 1;
        }
    });
}

/* -------------------------------------------------------------
    since futures are object safe trait hence they have all traits 
    features we can pass them to the functions in an static or dynamic 
    dispatch way using Arc or Box or impl Future or event as the return 
    type of a closure trait method, so a future task would be:

        1) std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>
        2) Arc<dyn Fn() -> R + Send + Sync + 'static> where R: std::future::Future<Output = ()> + Send + Sync + 'static
        3) Box<dyn Fn() -> R + Send + Sync + 'static> where R: std::future::Future<Output = ()> + Send + Sync + 'static
        4) Arc<Mutex<dyn Fn() -> R + Send + Sync + 'static>> where R: std::future::Future<Output = ()> + Send + Sync + 'static
        5) F: std::future::Future<Output = ()> + Send + Sync + 'static
        6) param: impl std::future::Future<Output = ()> + Send + Sync + 'static

    NOTE: mutex requires the type to be Sized and since traits are 
    not sized at compile time we should annotate them with dyn keyword
    and put them behind a pointer with valid lifetime or Box and Arc smart pointers
    so for the mutexed_job we must wrap the whole mutex inside an Arc or annotate it
    with something like &'valid tokio::sync::Mutex<dyn Fn() -> R + Send + Sync + 'static>
    the reason is that Mutex is a guard and not an smart pointer which can hanlde 
    an automatic pointer with lifetime 
*/
pub async fn startCronScheduler<F, R, T: Fn() -> R + Send + Sync + 'static>(period: u64, 
    // make the future cloneable in each iteration and tokio scope 
    // as well as safe to be shared between threads
    task: T) where // make the closure trait shareable and cloneable
        R: std::future::Future<Output = ()> + Send + Sync + 'static{

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
    tokio::spawn(async move{
        runInterval(|| async move{
            println!("i'm executing intervally in the background thread ...");
        }, period, 10)
        .await;
    });
}

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