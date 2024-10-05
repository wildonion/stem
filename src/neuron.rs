// src/neuron.rs
use tokio::sync::mpsc;
use serde_json::Value;
use crate::types::{Wallet, Transaction};

pub struct Neuron {
    pub name: String,
    pub cells: i64,
    pub metadata: Value,
    pub wallet: Option<Wallet>,
    pub transaction: Option<Transaction>,
    pub sender: mpsc::Sender<Value>,
    pub receiver: mpsc::Receiver<Value>,
}

impl Neuron {
    pub fn new(name: String, cells: i64, metadata: Value) -> (Self, mpsc::Sender<Value>) {
        let (tx, rx) = mpsc::channel(100);  // MPSC channel for message passing
        let neuron = Neuron {
            name,
            cells,
            metadata,
            wallet: None,
            transaction: None,
            sender: tx.clone(),
            receiver: rx,
        };
        (neuron, tx)
    }

    pub async fn run(&mut self) {
        println!("Neuron '{}' with {} cells is running.", self.name, self.cells);
        while let Some(message) = self.receiver.recv().await {
            println!("Neuron {} received message: {:?}", self.name, message);
            // Process the incoming message and potentially modify wallet or transaction
        }
    }
}
