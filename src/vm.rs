// src/vm.rs
use crate::neuron::Neuron;
use tokio::sync::mpsc;
use serde_json::Value;

pub struct VirtualMachine {
    bytecode: Vec<u8>,
    pc: usize, // Program counter
}

impl VirtualMachine {
    pub fn new(bytecode: Vec<u8>) -> Self {
        VirtualMachine { bytecode, pc: 0 }
    }

    pub async fn run(&mut self) {
        while self.pc < self.bytecode.len() {
            match self.bytecode[self.pc] {
                0x01 => self.execute_neuron_creation().await, // Neuron creation
                0x02 => self.execute_stream_send().await,     // Send data
                0x03 => self.execute_stream_receive().await,  // Receive data
                _ => {}
            }
            self.pc += 1;
        }
    }

    async fn execute_neuron_creation(&mut self) {
        println!("Executing Neuron Creation Opcode");
        // Example metadata for neuron creation
        let metadata = serde_json::json!({
            "version": "1.0",
            "description": "Neuron metadata from VM"
        });

        // Neuron creation logic (You can customize the name and cell count as needed)
        let (mut neuron, sender) = Neuron::new("vm_neuron".to_string(), 10, metadata);

        // Spawn the neuron actor to run in a separate task
        tokio::spawn(async move {
            neuron.run().await;
        });

        // Store the sender for future stream interactions if necessary
        self.pc += 1; // Move to next instruction
    }

    async fn execute_stream_send(&mut self) {
        println!("Executing Send Stream Opcode");

        // Example data to send
        let data = serde_json::json!({
            "key": "vm_value"
        });

        // Assuming you have the neuron sender stored in a way accessible here
        // Simulate sending data to the neuron
        // Note: You should have a reference to `mpsc::Sender<Value>` here to send messages
        if let Some(sender) = self.get_neuron_sender() {
            crate::stream::local_stream_send(data.clone(), sender, |data| {
                println!("Callback after sending data from VM: {:?}", data);
            })
            .await;
        }

        self.pc += 1; // Move to next instruction
    }

    async fn execute_stream_receive(&mut self) {
        println!("Executing Receive Stream Opcode");

        // Assuming you have a neuron receiver accessible here
        if let Some(receiver) = self.get_neuron_receiver() {
            crate::stream::local_stream_receive(receiver, |data| {
                println!("Callback after receiving data in VM: {:?}", data);
            })
            .await;
        }

        self.pc += 1; // Move to next instruction
    }

    // Helper function to retrieve neuron sender (you might need to store senders in a map)
    fn get_neuron_sender(&self) -> Option<mpsc::Sender<Value>> {
        // Return the mpsc sender for the neuron here
        None // Example, replace with actual logic
    }

    // Helper function to retrieve neuron receiver (you might need to store receivers in a map)
    fn get_neuron_receiver(&self) -> Option<&mut mpsc::Receiver<Value>> {
        // Return the mpsc receiver for the neuron here
        None // Example, replace with actual logic
    }
}
