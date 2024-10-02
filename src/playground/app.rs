


use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use core::task;
use std::error::Error;
use std::task::Context;
use std::thread;
use std::{collections::HashMap, sync::atomic::AtomicUsize};
use std::net::SocketAddr;
use deadpool_redis::redis::{AsyncCommands, RedisResult};
use futures::sink::Buffer;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use tokio::net::TcpStream;
use deadpool_redis::{Config as DeadpoolRedisConfig, Runtime as DeadPoolRedisRuntime};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use consts::SECURECELLCONFIG_TCPWALLET;
use consts::gen_random_chars;


pub mod tcp_chan_enc;
pub mod consts;
pub mod orex;


// store users along with their tcp streaming channels
pub static ID_TRACKER: AtomicUsize = AtomicUsize::new(1);
pub static USERS_TCP_STREAM: Lazy<Arc<Mutex<HashMap<usize, 
    (std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<String>>>, 
     tokio::sync::mpsc::Sender<String>)>>>> =
    Lazy::new(|| {
        Arc::new(Mutex::new(
            HashMap::new()
        ))
    }
);
pub static ONLINE_USERS: Lazy<Arc<Mutex<HashMap<String, usize>>>> = 
    Lazy::new(|| {
        let users = HashMap::default();
        Arc::new(Mutex::new(
            users
        ))
    }
); 


// Error part is an object safe trait which will be dispatched dynamically
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    let redis_password = "";
    let redis_host = "";
    let redis_port = "";
    let redis_username = "";
    let redis_conn_url = if !redis_password.is_empty(){
        format!("redis://:{}@{}:{}", redis_password, redis_host, redis_port)
    } else if !redis_password.is_empty() && !redis_username.is_empty(){
        format!("redis://{}:{}@{}:{}", redis_username, redis_password, redis_host, redis_port)
    } else{
        format!("redis://{}:{}", redis_host, redis_port)
    };


    let redis_pool_cfg = DeadpoolRedisConfig::from_url(&redis_conn_url);
    let redis_pool = redis_pool_cfg.create_pool(Some(DeadPoolRedisRuntime::Tokio1)).unwrap(); 
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8753").await.unwrap();
    tokio::spawn(async move{
        while let Ok((mut stream, addr)) = listener.accept().await{

            let get_id_tracker = ID_TRACKER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            
            let online_users = ONLINE_USERS.clone();
            let mut get_online_users = online_users.lock().await;

            // try to find an existing user with this address
            // otherwise insert it into the map
            *get_online_users.entry(
                addr.to_string()
            ).and_modify(|v| { *v; } /* keep the old id */ )
            .or_insert(get_id_tracker);
            
            let cloned_redis_pool = redis_pool.clone();
            // try to connect the user to a random online one
            tokio::spawn(async move{
                let stream = std::sync::Arc::new(tokio::sync::Mutex::new(stream));
                // execute the connect logic inside another light io thread since 
                // the logic contains some async locking mechanism as well as io processing
                // it's better not to suspend any async function for the locking process 
                // and gets done in a separate thread
                connectMe(stream, addr, cloned_redis_pool).await;
            });
        }
    }); 

    // since the socket stream needs to be mutable hence moving it 
    // between threads requires to be Arced and Mutexed to use it 
    // mutably in a safe manner inside other threads.
    pub async fn connectMe(mut current_user_stream: std::sync::Arc<tokio::sync::Mutex<TcpStream>>, 
        current_user: SocketAddr, redis_pool: deadpool_redis::Pool){ 

        let users_streams = USERS_TCP_STREAM.clone();
        let mut get_users_streams = users_streams.lock().await;

        let online_users = ONLINE_USERS.clone();
        let mut get_online_users = online_users.lock().await;
        let cloned_current_user_stream = current_user_stream.clone();


        let mut redis_conn = redis_pool.get().await.unwrap();
        let get_connected_users: String = redis_conn.get("connectedUsers").await.unwrap();
        let mut decoded_connected_users = serde_json::from_str::<
                HashMap<String, usize>
            >(&get_connected_users)
            .unwrap();

        let mut map_keys = get_online_users
            .clone()
            .into_iter()
            .map(|(user, id)| user)
            .collect::<Vec<String>>();

        map_keys.shuffle(&mut thread_rng());
        let mut found_user: String = String::from("");
        let mut found_id: usize = 0;

        for user in map_keys.clone(){

            let id = get_online_users.get(&user).unwrap();

            let mut redis_conn = redis_pool.get().await.unwrap();
            let get_connected_users: String = redis_conn.get("connectedUsers").await.unwrap();
            let mut decoded_connected_users = serde_json::from_str::<
                    HashMap<String, usize>
                >(&get_connected_users)
                .unwrap();

            // the user to connect to must not be the current user as well as
            // must not on redis in during the 2 mins period
            // a user can't connect to a user which was connected 2 mins ago
            if user == current_user.to_string() || decoded_connected_users.contains_key(&user){
                continue;
            } else{
                
                // start chatting with the first found user
                found_id = *id;
                found_user = user;
                break;
            }
        }        

        // store the user on redis, for 2 mins he won't be able to
        // connect to the previous user
        if !found_user.is_empty() && found_id != 0{
            decoded_connected_users.insert(found_user, found_id);
        }
        let encoded_connected_user = serde_json::to_string(&decoded_connected_users).unwrap();
        let _: () = redis_conn.set_ex("connectedUsers", &encoded_connected_user, 120).await.unwrap();
        
        // receive msg bytes from the user tcp stream channel
        let getUserSender = get_users_streams.get(&found_id).unwrap().1.clone();
        let getUserReceiver = get_users_streams.get(&found_id).unwrap().0.clone();
        let cloned_current_user_stream = cloned_current_user_stream.clone();

        tokio::spawn(async move{
            
            let mut buff = vec![];
            let mut getStream = cloned_current_user_stream.lock().await;

            while let Ok(rcvd_bytes) = getStream.read(&mut buff).await{
                if rcvd_bytes == 0{
                    getStream.shutdown().await; // shutdown the stream, disconnect the connection
                }
                let current_user_msg = std::str::from_utf8(&buff[..rcvd_bytes]).unwrap();
                // send the msg bytes of the current user to the connected user channel
                // connected user will use his receiver to receive the msg 
                getUserSender.send(current_user_msg.to_string()).await;

                let cloned_getUserReceiver = getUserReceiver.clone();
                let mut getReceiver = cloned_getUserReceiver.lock().await;
                // receive the connected user (user2) msg in here and send it through the 
                // current user (user1) tcp stream channel to the current user 
                while let Some(connected_user_msg) = getReceiver.recv().await{
                    getStream.write_all(connected_user_msg.as_bytes()).await;
                }
            }
        });

    }

    pub async fn disconnectMe(mut current_user_stream: std::sync::Arc<tokio::sync::Mutex<TcpStream>>, 
        user_id: usize, current_user: SocketAddr, redis_pool: deadpool_redis::Pool){
        
        let cloned_current_user_stream = current_user_stream.clone();
        // lockcing as an async task inside a light io thread
        tokio::spawn(async move{
            let mut getStream = cloned_current_user_stream.lock().await;
            getStream.shutdown().await; // close the current user tcp streaming channel

            // try to remove the user from online users
            let online_users = ONLINE_USERS.clone();
            let mut get_online_users = online_users.lock().await;
            (*get_online_users).remove(&current_user.to_string()).unwrap();

        });

    }
    
    // ---====---====---====---====---====---====---====---====---====---====---====---====
    
    // an eventloop is a thread safe mpsc receiver queue
    #[derive(Clone)]
    struct EventLoop<T: Clone + Send + Sync + 'static>{
        // a thread safe receiver queue
        pub event: T,
        pub sender: tokio::sync::mpsc::Sender<T>,
        pub queue: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<T>>>,
    }

    impl<T: Clone + Send + Sync + 'static> EventLoop<T>{
        pub async fn on<F, R>(&mut self, event_name: &str, triggerer: F) 
            where F: Fn(T) -> R + Send + Sync + 'static + Clone, // bind to 'static and clone so we can move it into the tokio spawn scope
            R: std::future::Future<Output = ()> + Send + Sync
            {
                match event_name{
                    "receive" => {
                        println!("[*] triggering {:?} event", event_name);
                        let mut get_queue = self.queue.lock().await;
                        while let Some(event) = get_queue.recv().await{
                            // triger the event in the background thread
                            let _triggerer = triggerer.clone();
                            tokio::spawn(async move{
                                _triggerer(event).await;
                            });
                        }
                    },
                    "send" => {
                        let _triggerer = triggerer.clone();
                        let event = self.event.clone();
                        tokio::spawn(async move{
                            _triggerer(event).await;
                        });
                    },
                    _ => {

                    }
                }
            }
        
        pub async fn cronScheduling<F, R>(&mut self, func: F, period: u64) 
            where F: Fn() -> R + Send + Sync + 'static + Clone,
                R: std::future::Future<Output = ()> + Send + Sync + 'static
            {

                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(period));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                
                // defining an async context or future object
                let fut = async move{
                    let cloned_task = func.clone();
                    loop{
                        interval.tick().await;
                        // since the closure returns a future we can run it 
                        // in the background tokio thread
                        tokio::spawn(func()); // the closure however returns a future 
                    }
                };

                // execute the future object in the background thread worker 
                // without waiting for the result
                tokio::spawn(fut);

            }
        
    }

    #[derive(Clone, Debug)]
    struct BufferEvent{
        pub data: std::sync::Arc<tokio::sync::Mutex<Vec<u8>>>
    }
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct EventData{
        pub owner: String,
        pub data: String,
        pub recv_time: i64
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel::<BufferEvent>(100);
    let mut eventloop = EventLoop::<BufferEvent>{
        event: BufferEvent { data: Arc::new(Mutex::new(String::from("wildonion").as_bytes().to_vec())) },
        sender: tx.clone(),
        queue: std::sync::Arc::new(tokio::sync::Mutex::new(rx))
    };

    /* -------------------------------------
        .await; suspend the task and tell runtime i need the result, if the
        result wasn't ready runtime continue executing other task by poping 
        them out from the eventloop and suspend the awaited task in there until 
        the future completes and result becomes ready, not awaiting means we 
        don't care about the result, runtime executes task in the background 
        light io thread without having any suspention hence we could continue 
        with the rest of the code, thus if you need the result of async task
        like sending it to channel, await on it, HOWEVER this won't block the 
        light thread.
        also we could use tokio::select to control the execution flow of 
        the app in async context and get the result of whatever the async
        task has solved sooner than the other. 
        tokio::spawn() is a place where async task can be executed by the runtime
        scheduler, it's a lightweight thread of execution where async tasks 
        will be awaited in there without blocking the thread.
    */
    // handle receive event
    tokio::spawn(
        {
            let mut eventloop = eventloop.clone();
            async move{
                // once we receive a buffer event we'll be decoding it 
                // into EventData structure
                eventloop.on("receive", |e| async move{
                    let get_event_data = e.data.lock().await;
                    let event_data = serde_json::
                        from_slice::<EventData>(&get_event_data)
                        .unwrap();
                    println!("[*] received event: {:?}", event_data);
            
                }).await;
            }
        }
    );

    // handle send event 
    tokio::spawn(
        {
            let mut eventloop = eventloop.clone();
            async move{
                let sender = eventloop.clone().sender;
                eventloop.on("send", move |e| 
                    {   
                        // needs to clone the sender before moving into the async move{} scopes
                        // cause the sender has been captured by the inner closure thus we should
                        // clone it again here before getting captured by the async move closuer
                        let cloned_sender = sender.clone();
                        async move{
                            cloned_sender.clone().send(e.clone()).await;
                            println!("[*] sent event: {:#?}", e);
                        }
                    }
                ).await;
            }
        }
    );

    async fn selectEventLoop(){

        println!("running async tasks using eventloop...");

        use tokio::{spawn, select, sync::Mutex};
        use std::sync::Arc;

        let mut int = tokio::time::interval(tokio::time::Duration::from_secs(10));

        /* 
            every tokio spawn task would run in the background thread
            the light thread would await on the task in its own context
            but we shouldn't await on the spawned thread, using of channels 
            and mutex would be great
            runtime schedule async tasks to execute them over time but futures 
            do nothing if we want to run them in background put them inside 
            background light thread and await in there, awaiting on them suspend 
            their execution until runtime gives us the result it doesn't mean other 
            tasks stop executing during the await process cause awaiting is not 
            a blocking manner
        */
        // ticking in the background every 10 seconds
        tokio::spawn(async move{
            loop{
                int.tick().await;
                println!("ticked 10 seconds");
            } 
        });

        tokio::spawn(async move{
            for i in 1..30{
                println!("Doing work: {}", i);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });


        // sleep 20 seconds greater than the tick to see 2 elapse of the ticker
        // to see what's happening inside each tokio spawn light thread
        tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;

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

    Ok(())

}