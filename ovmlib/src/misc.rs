

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