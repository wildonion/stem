// src/types.rs
#[derive(Debug)]
pub struct Wallet {
    pub id: String,
    pub balance: f64,
}

#[derive(Debug)]
pub struct Transaction {
    pub tx_id: String,
    pub amount: f64,
}
