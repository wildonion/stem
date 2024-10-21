// src/stream.rs
use tokio::sync::mpsc;
use serde_json::Value;

// Function to simulate sending data to the neuron and running a callback
pub async fn local_stream_send(data: Value, sender: mpsc::Sender<Value>, callback: impl Fn(Value)) {
    println!("Sending data to neuron: {:?}", data);
    if sender.send(data.clone()).await.is_ok() {
        println!("Data sent successfully!");
        callback(data);  // Run the callback with access to the data
    } else {
        println!("Failed to send data.");
    }
}

// Function to simulate receiving data from the neuron and running a callback
pub async fn local_stream_receive(receiver: &mut mpsc::Receiver<Value>, callback: impl Fn(Value)) {
    if let Some(data) = receiver.recv().await {
        println!("Data received from neuron: {:?}", data);
        callback(data);  // Run the callback with access to the data
    } else {
        println!("Failed to receive data.");
    }
}
