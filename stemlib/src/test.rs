


use neuron::{AppService, Broadcast, InjectPayload, ReceiveP2pResponse, ReceiveRpcResponse, RmqConfig, SendP2pRequest, SendRpcRequest, ShutDown, StartP2pSwarEventLoop, Subscribe};
use crate::*;
use crate::dsl::*;


pub async fn upAndRunStreaming(){
    
    let (mut neuron, synprot) = NeuronActor::new(100,
        Some(RmqConfig{ 
            host: String::from("0.0.0.0"), 
            port: 5672, 
            username: String::from("rabbitmq"), 
            password: String::from("geDteDd0Ltg2135FJYQ6rjNYHYkGQa70") 
        }),
    ).await;

    let getNeuronWallet = neuron.wallet.as_ref().unwrap();

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
    neuron.runInterval(|| async move{
        println!("i'm running every 10 seconds, with retries of 12 and timeout 0");
    }, 10, 12, 0).await;

    // streaming over neuron, the on() method belongs the StreamRmq interface
    // enables us to start streaming over a neuron object either locally or 
    // remotely, the method however takes a callback closure in which we have
    // access to the sent or received event.
    neuron
        .on("local", "receive", move |event, error| async move{

            if error.is_some(){
                println!("an error accoured during receiving: {:?}", error.unwrap());
                return;
            }

            println!("received task: {:?}", event);

            // do whatever you want to do with received task:
            // storing in db or cache on redis 
            // ...


        }).await
        .on("rmq", "send", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during sending: {:?}", error.unwrap());
                return;
            }
            
            println!("sent task: {:?}", event);

            // do whatever you want to do with sent task:
            // please shiaf the sent task!
            // ...

        }).await
        .on("rmq", "receive", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during receiving: {:?}", error.unwrap());
                return;
            }

            println!("received task: {:?}", event);

            // store the event in db or cache on redis
            // ...

        }).await
        .on("p2p", "send", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during sending: {:?}", error.unwrap());
                return;
            }
            
            println!("sent task: {:?}", event);

            // do whatever you want to do with sent task:
            // please shiaf the sent task!
            // ...

        }).await
        .on("p2p", "receive", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during receiving: {:?}", error.unwrap());
                return;
            }

            println!("received task: {:?}", event);

            // store the event in db or cache on redis
            // ...

        }).await;

}

pub async fn upAndRunTalking(){

    let (mut neuron, synprot) = NeuronActor::new(100,
        Some(RmqConfig{ 
            host: String::from("0.0.0.0"), 
            port: 5672, 
            username: String::from("rabbitmq"), 
            password: String::from("geDteDd0Ltg2135FJYQ6rjNYHYkGQa70") 
        }),
    ).await;

    
    let getNeuronWallet = neuron.wallet.as_ref().unwrap();
    let getNeuronId = neuron.peerId.to_base58();
    
    // ------- sending message through actor mailbox eventloop receiver:
    // actor mailbox is the eventloop receiver of actor jobq mpsc channel
    // which receive messages and execute them in a light thread or process 

    // starting the neuron actor 
    let neuronComponentActor = neuron.clone().start();
    
    // sending update state message
    neuronComponentActor.send(
        UpdateState{new_state: 1}
    ).await;

    // send shutdown message to the neuron
    neuronComponentActor.send(ShutDown).await;

    // send payload remotely using the neuron actor
    neuronComponentActor.send(
        InjectPayload{
            payload: String::from("0x01ff").as_bytes().to_vec(), 
            method: neuron::TransmissionMethod::Remote(String::from("p2p-synapse"))
        }
    ).await;

    // send rpc request to another neuron
    neuronComponentActor.send(
        SendRpcRequest{
            local_spawn: todo!(),
            notif_data: todo!(),
            requestQueue: todo!(),
            correlationId: todo!(),
            encryptionConfig: todo!(),
        }
    ).await;

    // receive rpc responses from another neuron
    neuronComponentActor.send(
        ReceiveRpcResponse{
            local_spawn: todo!(),
            notif_data: todo!(),
            requestQueue: todo!(),
            encryptionConfig: todo!(),
        }
    ).await;

    // broadcast in the whole brain
    neuronComponentActor.send(
        Broadcast{
            local_spawn: todo!(),
            notif_data: todo!(),
            rmqConfig: todo!(),
            p2pConfig: todo!(),
            encryptionConfig: todo!(),
        }
    ).await;

    // subscribe in the whole brian
    neuronComponentActor.send(
        Subscribe{
            p2pConfig: todo!(),
            rmqConfig: todo!(),
            local_spawn: todo!(),
            decryptionConfig: todo!(),
        }
    ).await;

    // start the p2p swarm event loop 
    neuronComponentActor.send(
        StartP2pSwarEventLoop{
            synprot
        }
    ).await;

    // send a p2p based request
    neuronComponentActor.send(
        SendP2pRequest{
            message: todo!(),
            peerId: todo!()
        }
    ).await;

    // receive a p2p based response
    neuronComponentActor.send(
        ReceiveP2pResponse
    ).await;

    log::info!("executed all..");


}