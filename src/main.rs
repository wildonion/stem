use rand_chacha::rand_core::le;





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{


    async fn selectEventLoop(){

        println!("running async tasks using eventloop...");

        use tokio::{spawn, select, sync::Mutex};
        use std::sync::Arc;
        
        spawn(async move{
            
            let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
            let mutexed_receiver = Arc::new(Mutex::new(rx));
            
            let send_task = async move{
                for i in 0..10{
                    tx.send(format!("wildonion-{}", i)).await;
                    println!("sent! {i:}");
                }
            }; 

            let cloned_mutexed_receiver = mutexed_receiver.clone();
            let receive_task = async move{
                while let Some(data) = cloned_mutexed_receiver.lock().await.recv().await{
            
                    println!("received {data:}");
                    
                }
            };

            // pin the task into the ram to prevent moving 
            // cause we want to use it inside the loop and 
            // in Rust types will be moved often
            tokio::pin!(send_task, receive_task); 

            // eventloop using select!{}, put the select inside a loop, makes 
            // long running hanlder which handles the async tasks as soon as 
            // one of them completes
            loop{
                select! {
                    _ = &mut receive_task => {
                        // The receive task is completed, break the loop
                        println!("Receive task completed.");
                        break; // break in here to cancel other tasks 
                    },
                    _ = &mut send_task => {
                        // Execute some logic after the sending task has executed
                        println!("Send task completed.");
                        /*
                            `async fn` resumed after completion:
                            occurs when an async task is awaited multiple times after it has already completed 
                            we should break after sending to exit from the eventloop and don't allow the 
                            eventloop to await on the task again
                        */
                        // sleep to avoid awaiting multiple times inside the loop
                        tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;
                    },
                }
            }

        });



        // since the spawn is doing its job in the background to see what's happening inside
        // or logs of it, we'll wait in here for seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    }

    /* 
        in Rust by default we should await on futures cause they don't do anything by
        default unless we poll them it's not like nodejs which can be executed in
        the background without awaiting on them, doing so tells runtime that suspend the 
        task execution but don't block the current thread but let that execute other 
        tasks, notify caller once the future is completed. for io tasks it would be 
        better to await on them in a ligh thread, this can be done by sending the future
        to the spawn() method, tokio moves the future into a light thread and returns 
        a joinhandle which can be awaited if we need to have some result from the spawned
        scope, would be great if we allow the future to be executed in the background 
        light thread (light thread per task) and use channels or static lazy arc mutex 
        to get results. the task passed into to the tokio::spawn() will be awaited by 
        the tokio inside a light thread.
        
        following will log nothing unless we await on the getMe() method:
        async fn getMe(){
            println!("in future object"); 
        }
        getMe();

        desired output from the following function:
            inside the selectEventLoop function
            sent! 0
            sent! 1
            sent! 2
            sent! 3
            sent! 4
            sent! 5
            sent! 6
            sent! 7
            sent! 8
            sent! 9
            Send task completed.
            received wildonion-0
            received wildonion-1
            received wildonion-2
            received wildonion-3
            received wildonion-4
            received wildonion-5
            received wildonion-6
            received wildonion-7
            received wildonion-8
            received wildonion-9
            Receive task completed.
    */
    selectEventLoop().await;

    ovmlib::parse();

    Ok(())

}