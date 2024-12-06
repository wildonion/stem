


/* ------------------------------- implementations: 
    schemas implementations 
*/


use deadpool_lapin::lapin::options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions};
use deadpool_lapin::lapin::types::FieldTable;
use deadpool_lapin::lapin::{Connection as LapinConnection, ConnectionProperties};
use deadpool_lapin::{Config, Manager, Pool as LapinDeadPool, Runtime};
use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    BasicProperties,
};
use futures::StreamExt;
use log4rs::append;
use rayon::string;
use uuid::timestamp::context;
use wallexerr::misc::Wallet;
use crate::*;
use crate::messages::*;
use crate::dto::*;
use crate::interfaces::*;


impl Drop for Neuron{
    fn drop(&mut self) {
        let this = self.clone();
        tokio::spawn(async move{
            let getInternalWorker = &this.internal_worker;
            if getInternalWorker.is_some(){
                let mut internalWorker = getInternalWorker.clone().unwrap();
                let mut unloackedInternalWorker = internalWorker.lock().await;
                (*unloackedInternalWorker).thread.abort(); // abort the thread handler, the tokio threads are future based thread so we can easily abort them
            }
        });
    }
}

impl Neuron{
    
    pub async fn new(bufferSize: usize, name: &str) -> Self{
        Neuron{
            name: name.to_string(),
            /* *********************************************************** */
            /* **************** BUILDING SYNAPSE PROTOCOL **************** */
            /* *********************************************************** */
            synProt: { 
                // building the swarm object with our network behaviour contains our synapse protocol
                let mut swarm = SwarmBuilder::with_new_identity()
                .with_tokio()
                .with_tcp(
                    tcp::Config::default(), 
                    noise::Config::new,
                    yamux::Config::default
                )
                .unwrap()
                .with_quic()
                .with_behaviour(|key|{
                    Ok(NeuronBehaviour::new(key.clone()))
                })
                .unwrap()
                .with_swarm_config(|c| c.with_idle_connection_timeout(tokio::time::Duration::from_secs(60)))
                .build();

                // as soon as the actor is started the swarm eventloop will be executed
                // and set to be listened on different ports.
                // a shareable, cloneable and thread safe p2p swarm object containing 
                // all the network behaviours
                SynapseProtocol{
                    swarm: std::sync::Arc::new(
                        tokio::sync::Mutex::new(
                            swarm
                        )
                    )
                }
                    
            },
            /* *********************************************************** */
            /* *********************************************************** */
            /* *********************************************************** */
            peerId: {
                let edkeys = Keypair::generate_ed25519();
                let local_peer_id = PeerId::from_public_key(&edkeys.public());
                local_peer_id
            },
            wallet: Some(wallexerr::misc::Wallet::new_ed25519()), // wallexerr wallet for this neuron
            internal_executor: InternalExecutor::new(
                Buffer{ events: std::sync::Arc::new(tokio::sync::Mutex::new(vec![
                    Event{ data: EventData::default(), status: EventStatus::Committed, offset: 0 }
                ])), size: bufferSize }
            ), 
            metadata: None,
            internal_worker: None,
            transactions: None,
            internal_locker: None,
            internal_none_async_threadpool: Arc::new(None),
            signal: std::sync::Arc::new(std::sync::Condvar::new()),
            contract: None,
            rmqPool: {
                let ymlConfigFile = tokio::fs::read_to_string("cfg.yml").await;
                let Ok(config) = ymlConfigFile else{
                    panic!("found no YML config file!");
                };
                let cfg = serde_yml::from_str::<YmlConfig>(&config).unwrap();

                let rmq_addr = format!("amqp://{}:{}@{}:{}", cfg.rmqUsername, cfg.rmqPassword, cfg.rmqHost, cfg.rmqPort);
                let mut cfg = Config::default();
                cfg.url = Some(rmq_addr);
                let lapin_pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
                Arc::new(lapin_pool)
            },
            dependency: std::sync::Arc::new(AppService{}),
            state: 0 // this can be mutated by sending the update state message to the actor
        }
    }

    pub async fn sendRpcRequest(&mut self, rpcConfig: RmqRequestConfig, encConfig: Option<CryptoConfig>){

    }

    pub async fn sendP2pRequest(&mut self, p2pConfig: P2pRequestConfig, encConfig: Option<CryptoConfig>){

    }

    pub async fn receiveRpcResponse(&mut self, rpcConfig: RmqResponseConfig, decConfig: Option<CryptoConfig>) -> String{

        todo!()
    }

    pub async fn receiveP2pResponse(&mut self, p2pConfig: P2pResponseConfig, decConfig: Option<CryptoConfig>)  -> String{

        todo!()
    }

    // in swarm event loop we'll handle all swarm network behaviours 
    // including stream, kademlia, gossipsub and request response events
    pub async fn startP2pSwarmEventLoop(&mut self){
        
        // https://github.com/libp2p/rust-libp2p/blob/master/examples/distributed-key-value-store/src/main.rs
        // https://github.com/libp2p/rust-libp2p/blob/master/examples/file-sharing/src/network.rs
        // https://github.com/libp2p/rust-libp2p/blob/master/examples/chat/src/main.rs
        // https://github.com/libp2p/rust-libp2p/blob/master/examples/stream/src/main.rs
        let mut getSwarm = self.clone().synProt.clone().swarm;
        let mut swarm = getSwarm.lock().await;
        swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
        
        // listen on both udp and tcp
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap()).unwrap();
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        // TODO 
        // swarm eventloop is a loop that waits for incoming events of
        // kademlia, gossipsub and request response ones and we can handle 
        // them in realtime as they're coming, process various events emitted
        // by the swarm
        // ...

        // use select inside a loop to control multiple execution flow of the app 
        // during runtime
    }

    // gossipsub publish 
    pub async fn p2pPublish(&mut self, p2pConfig: P2pPublishConfig){

        let P2pPublishConfig { topic, peerId, message, synProt } = p2pConfig;
        
        let mut getSwarm = synProt.swarm;
        let mut swarm = getSwarm.lock().await;
        let topic = gossipsub::IdentTopic::new(topic);
        
        // this can be inside the swarm eventloop and publishing the message constantly 
        let msgId = swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.as_bytes()).unwrap();

        log::info!("message has published with Id [{}]", std::str::from_utf8(&msgId.0).unwrap());

    }   

    // gossipsub consume
    pub async fn p2pConsume(&mut self, p2pConfig: P2pConsumeConfig){

        let P2pConsumeConfig { topic, synProt } = p2pConfig;

        let mut getSwarm = synProt.swarm;
        let mut swarm = getSwarm.lock().await;
        let topic = gossipsub::IdentTopic::new(topic);
        swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

        // we'll log the received and subscribed messages inside the swarm eventloop

    }

    pub async fn rmqPublish(&self, data: &str, exchange: &str, routing_key: &str, 
        exchange_type: &str, secureCellConfig: SecureCellConfig, redisUniqueKey: &str){

        // these are must be converted into String first to make longer lifetime 
        // cause &str can't get moved into tokio spawn as its lifetime it's not 
        // static the tokio spawn lives longer than the &str and the &str gets 
        // dropped out of the ram once the function is finished with executing
        let exchange = exchange.to_string();
        let routing_key = routing_key.to_string();
        let exchange_type = exchange_type.to_string();
        let data = data.to_string();
        let redisUniqueKey = redisUniqueKey.to_string();
        let getPool = self.clone().rmqPool.get().await;
        if getPool.is_err(){
            panic!("can't get rmq connection from the pool");
        }
        let conn = getPool.unwrap();

        tokio::spawn(async move{
            // -ˋˏ✄┈┈┈┈ creating a channel in this thread
            match conn.create_channel().await{
                Ok(chan) => {

                    let mut ex_options = ExchangeDeclareOptions::default();
                    ex_options.auto_delete = true; // the exchange can only be deleted automatically if all bindings are deleted
                    ex_options.durable = true; // durability is the ability to restore data on node shutdown

                    // -ˋˏ✄┈┈┈┈ creating exchange
                    /* 
                        you should set the auto_delete flag to True when declaring the exchange. This will 
                        automatically delete the exchange when all channels are done with it.
                        Keep in mind that this means that it will stay as long as there is an active binding 
                        to the exchange. If you delete the binding, or queue, the exchange will be deleted.
                        if you need to keep the queue, but not the exchange you can remove the binding once 
                        you are done publishing. This should automatically remove the exchange.
                        so when all bindings (queues and exchanges) get deleted the exchange will be deleted.
                    */
                    match chan
                        .exchange_declare(&exchange, {
                            match exchange_type.as_str(){
                                "fanout" => deadpool_lapin::lapin::ExchangeKind::Fanout,
                                "direct" => deadpool_lapin::lapin::ExchangeKind::Direct,
                                "headers" => deadpool_lapin::lapin::ExchangeKind::Headers,
                                _ => deadpool_lapin::lapin::ExchangeKind::Topic,
                            }
                        },
                        ex_options, FieldTable::default()
                        )
                        .await
                        {
                            Ok(ok) => {ok},
                            Err(e) => {
                                log::error!("can't create exchange");
                            }

                        };

                    /* =================================================================================== */
                    /* ================================ PRODUCING PROCESS ================================ */
                    /* =================================================================================== */
                    // async task: publish messages to exchange in the background in a 
                    // lightweight thread of execution
                    tokio::spawn(async move{
                                    
                        // -ˋˏ✄┈┈┈┈ publishing to exchange from this channel,
                        // later consumer bind its queue to this exchange and its
                        // routing key so messages go inside its queue, later they 
                        // can be consumed from the queue by the consumer.
                        // in direct exchange ("") it has assumed that the queue
                        // name is the same as the routing key name.
                        use deadpool_lapin::lapin::options::BasicPublishOptions;
                        let payload = data.as_bytes();
                        match chan
                            .basic_publish(
                                &exchange, // messages go in there
                                &routing_key, // the way that message gets routed to the queue based on a unique routing key
                                BasicPublishOptions::default(),
                                payload, // this is the ProduceNotif data,
                                BasicProperties::default().with_content_type("application/json".into()),
                            )
                            .await
                            {
                                Ok(pc) => {
                                    let get_confirmation = pc.await;
                                    let Ok(confirmation) = get_confirmation else{
                                        log::error!("can't get confirmation");
                                        return; // needs to terminate the caller in let else pattern
                                    };

                                    if confirmation.is_ack(){
                                        log::info!("publisher confirmation is acked, payload has sent");
                                    }

                                },
                                Err(e) => {
                                    log::error!("can't get publisher confirmation");
                                }
                            }
                    });

                },
                Err(e) => {
                    log::error!("can't create the channel");
                    return;
                }
            }
        });

    }

    pub async fn rmqConsume(&self,
        consumer_tag: &str, queue: &str, callback: IoEvent,
        binding_key: &str, exchange: &str,
        decryptionConfig: Option<CryptoConfig>
    ){

        // this can be used for sending the received message to some ws server 
        let internal_executor = &self.internal_executor;
        let intex_sender = internal_executor.clone().sender;
        let exchange = exchange.to_string();
        let getPool = self.clone().rmqPool.get().await;
        if getPool.is_err(){
            panic!("can't get rmq connection from the pool");
        }
        let conn = getPool.unwrap();

        let create_channel = conn.create_channel().await;
        match create_channel{
            Ok(chan) => {

                let mut q_options = QueueDeclareOptions::default();
                q_options.durable = true; // durability is the ability to restore data on node shutdown

                // -ˋˏ✄┈┈┈┈ making a queue inside the broker per each consumer, 
                let create_queue = chan
                    .queue_declare(
                        &queue,
                        q_options,
                        FieldTable::default(),
                    )
                    .await;

                let Ok(q) = create_queue else{
                    log::error!("can't create queue");
                    return; // terminate the caller
                };

                // binding the queue to the exchange routing key to receive messages it interested in
                /* -ˋˏ✄┈┈┈┈ 
                    if the exchange is not direct or is fanout or topic we should bind the 
                    queue to the exchange to consume the messages from the queue. binding 
                    the queue to the passed in exchange, if the exchange is direct every 
                    queue that is created is automatically bounded to it with a routing key 
                    which is the same as the queue name, the direct exchange is "" and 
                    rmq doesn't allow to bind any queue to that manually
                */
                match chan
                    .queue_bind(q.name().as_str(), &exchange, &binding_key, // using binding key specifies how we're going to receive messages from the exchange (the way of receiving)
                        QueueBindOptions::default(), FieldTable::default()
                    )
                    .await
                    { // trying to bind the queue to the exchange
                        Ok(_) => {},
                        Err(e) => {
                            log::error!("can't bind the queue");
                            return; // terminate the caller
                        }
                    }

                // since &str is not lived long enough to be passed to the tokio spawn
                // if it was static it would be good however we're converting them to
                // String to pass the String version of them to the tokio spawn scope
                let cloned_consumer_tag = consumer_tag.to_string();
                let cloned_queue = queue.to_string();

                /* =================================================================================== */
                /* ================================ CONSUMING PROCESS ================================ */
                /* =================================================================================== */
                // start consuming in the background in a lightweight thread of execution
                // receiving is considered to be none blocking which won't block the current thread. 
                tokio::spawn(async move{

                    // -ˋˏ✄┈┈┈┈ consuming from the queue owned by this consumer
                    match chan
                        .basic_consume(
                            /* -ˋˏ✄┈┈┈┈
                                the queue that is bounded to the exchange to receive messages based on the routing key
                                since the queue is already bounded to the exchange and its routing key, it only receives 
                                messages from the exchange that matches and follows the passed in routing pattern like:
                                message routing key "orders.processed" might match a binding with routing key "orders.#
                                if none the messages follow the pattern then the queue will receive no message from the 
                                exchange based on that pattern! although messages are inside the exchange.
                            */
                            &cloned_queue, 
                            &cloned_consumer_tag, // custom consumer name
                            BasicConsumeOptions::default(), 
                            FieldTable::default()
                        )
                        .await
                        {
                            Ok(mut consumer) => {
                                while let Some(delivery) = consumer.next().await{
                                    match delivery{
                                        Ok(delv) => {
                                            
                                            log::info!("[*] received delivery from queue#<{}>", q.name());
                                            
                                            let consumedBuffer = delv.data.clone(); 
                                            let clonedIntExSender = intex_sender.clone();
                                            let clonedCallback = callback.clone();
                                            
                                            // handle the message in the background light thread asyncly and concurrently
                                            // by spawning a light thread for the async task
                                            tokio::spawn(async move{
                                                // either decrypted or the raw data as string
                                                log::info!("[*] received data: {:?}", hex::encode(&consumedBuffer));
                                                
                                                // we've assumed that the received data is not encrypted
                                                // decoding the string data into the Event structure (convention)
                                                let get_event = serde_json::from_slice::<Event>(&consumedBuffer);
                                                match get_event{
                                                    Ok(event) => {

                                                        log::info!("[*] deserialized data: {:?}", event);

                                                        // send the subscribed message to the internal executor channel 
                                                        // so we could receive it inside the callback or other scopes like 
                                                        // handling rmq messages over websocket, we'll use the receiver 
                                                        // inside the websocket server.
                                                        clonedIntExSender.send(event.clone()).await;

                                                        // execute the callback in here
                                                        tokio::spawn(clonedCallback(event));

                                                    },
                                                    Err(e) => {
                                                        log::error!("can't deserialize consumed data");
                                                    }
                                                }
                                            
                                            });
                                        },
                                        Err(e) => {
                                            log::error!("can't get msg delivery");
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                log::error!("can't get the consumer");
                                return;
                            }
                        }
                    });
            },
            Err(e) => {
                log::error!("can't create channel!");
                return;
            }
        }

    }

    //==================================================================================
    /* -------------------------------------------------------------
                    to store on heap use arc box or &
        since futures are object safe traits hence they have all traits 
        features we can pass them to the functions in an static or dynamic 
        dispatch way using Arc or Box or impl Future or even as the return 
        type of a closure trait method, so a future task would be one of 
        the following types: 
        note that Arcing and Boxing closure traits as generic would 
        need no dyn keyword behind the trait or generic type also we 
        can't await on an Arced future! use number 10 instead which is 
        having future object as separate type without using generics.
        number1 is not a safe io task to be sharead between threads
        number10 is a safe thread io task tha can be sharead between threads

            1) task: Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>
            2) task: Pin<Arc<dyn Future<Output = ()> + Send + Sync + 'static>
            3) task: Arc<dyn Fn() -> Pin<Arc<R>> + Send + Sync + 'static> where R: Future<Output = ()> + Send + Sync + 'static
            4) task: Box<dyn Fn() -> Pin<Box<R>> + Send + Sync + 'static> where R: Future<Output = ()> + Send + Sync + 'static
            5) task: Arc<Mutex<dyn Fn() -> Pin<Arc<R>> + Send + Sync + 'static>> where R: Future<Output = ()> + Send + Sync + 'static
            6) task: F where F: Future<Output = ()> + Send + Sync + 'static
            7) task: impl Future<Output = ()> + Send + Sync + 'static
            8) task: Arc<F> where F: Fn() -> R + Send + Sync + 'static where R: Future<Output = ()> + Send + Sync + 'static
            9) task: Box<F> where F: Fn() -> R + Send + Sync + 'static where R: Future<Output = ()> + Send + Sync + 'static
            10) task: Arc<dyn Fn() -> Pin<Box<dyn Future<Output=()> + Send + Sync + 'static>>> => Arc::new(||Box::pin(async move{}))

        NOTE: mutex requires the type to be Sized and since traits are not sized at 
        compile time we should annotate them with dyn keyword and put them behind a 
        pointer with a valid lifetime like Box or Arc smart pointers so for the 
        mutexed_job we must wrap the whole mutex inside an Arc or annotate it with 
        something like &'valid Mutex<dyn Fn() -> R + Send + Sync + 'static>
        the reason is that Mutex is a guard and not an smart pointer which can hanlde 
        an automatic pointer with lifetime, using of this syntax however would be in 
        a rare case when we want to mutate the closure!
    */
    // task is a closure that returns a future object 
    pub async fn runInterval<M, R, O>(&mut self, task: M, period: u64, retries: u8, timeout: u64)
    where M: Fn() -> R + Send + Sync + 'static,
            R: std::future::Future<Output = O> + Send + Sync + 'static,
    {
        let arced_task = std::sync::Arc::new(task);

        // the future task must gets executed within the period of timeout
        // otherwise it gets cancelled, dropped and aborted
        if timeout != 0{
            tokio::time::timeout(
                tokio::time::Duration::from_secs(timeout),
                arced_task() // this task gets executed with the passed in timeout
            ).await;
        }

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

    pub async fn na_getTask<F, R>(
        task1: F,
        task2: Arc<dyn Fn() -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>>, // without using generic
        task: impl std::future::Future<Output = ()> + Send + Sync + 'static
    ) where F: Fn() -> R + Send + Sync + 'static, R: std::future::Future<Output=()> + Send + Sync + 'static{
        
        let job = task2();
        job.await;
        let arcedTask = std::sync::Arc::new(task1);
        arcedTask().await;

        task.await;
    }

    pub async fn na_runInterval<M, R, O>(task: M, period: u64, retries: u8, timeout: u64)
    where M: Fn() -> R + Send + Sync + 'static,
            R: std::future::Future<Output = O> + Send + Sync + 'static,
    {
        let arced_task = std::sync::Arc::new(task);

        // the future task must gets executed within the period of timeout
        // otherwise it gets cancelled, dropped and aborted
        if timeout != 0{
            tokio::time::timeout(
                tokio::time::Duration::from_secs(timeout),
                arced_task()
            ).await;
        }        

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

    // if we can acquire the lock means the lock is free
    // and is not being in used by another process which 
    // causes the second process to be awaited until the 
    // lock gets freed.
    pub async fn execute<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S>
        (&mut self, mut runner: Runner<J, S>) where S: Send + Sync + 'static{

        if self.internal_worker.is_none(){
            return;
        }

        let get_internal_worker = self.internal_worker.clone().unwrap();
        let worker_lock = get_internal_worker.try_lock();

        // if the lock is busy reject the execution
        if worker_lock.is_err(){
            return;
        }

        let mut worker = worker_lock.unwrap();

        // update the worker thread with a new runner 
        (*worker).thread = std::sync::Arc::new(
            tokio::spawn(async move{
                runner.execute().await;
            })
        );
    }

    pub async fn executekMeIntervally(&mut self){
        log::info!("executing...");
    }

    /* -------------------------------------------------------
        an example function to show how we can execute jobs 
        using an eventloop in the background thread.
        a job can be arced version of a closure trait which 
        returns a future object as its body or a pinned version
        of the whole closure trait, a pinned version
        of a boxed of a future trait object or binding the 
        generic to the closure trait which returns a future
        object as its body.
        for both Arc<dyn Trait> or Box<dyn Trait> the trait 
        must be object safe.
    */
    pub async fn AllInOneInternalExecutor<J, R>(&mut self, job: J) where J: Fn() -> R + Send + Sync +'static, 
    R: std::future::Future<Output = ()> + Send + Sync + 'static{

        let arcedJob = std::sync::Arc::new(job);
        let (tx, mut eventloop) = 
            tokio::sync::mpsc::channel::<std::sync::Arc<J>>(100);

        // schedule a job to pop it out from the eventloop queue to gets executed in the background 
        // thread or send a message to the actor worker thread to execute a job like start consuming 
        // or receiving streaming in a light thread like stream.on or while let some

        // step1)
        // execute an interval in the background thread, this would
        // execute a sending task every 5 seconds in the background thread.
        let mut this = self.clone();
        tokio::spawn(async move{
            // run a task intervally in the background light thread
            this.runInterval(move || {
                // ----------------------------------------------------------------
                // -------- clone before going into the async move{} scope
                // ----------------------------------------------------------------
                // we must clone them inside the closure body 
                // before sending them inside the tokio spawn
                let cloned_tx = tx.clone();
                let arcedJob = arcedJob.clone();
                // this is the closure body which is of type future object
                async move{
                    cloned_tx.send(arcedJob).await;
                }
            }, 5, 0, 0).await;
        });

        // step2)
        // start receiving or streaming over the eventloop 
        // of the channel to receive jobs and execute them
        // in another light thread in the background
        tokio::spawn(async move{
            while let Some(job) = eventloop.recv().await{
                tokio::spawn(job());
            }
        });

    }

}



impl<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S> Runner<J, S>{
    pub async fn execute(&mut self){
        let job = &self.job;

        // execute the job in a light thread
        // ...
    }
}

impl Event{

    pub async fn process(&mut self){
        
    }

}


// ------------------------------
// implementnig the internal executor methods, it has an eventloop
// in its core which receives event coming from its channel constantly
// it then execute them along with the passed in callback in the run() method
impl InternalExecutor<Event>{
    
    pub fn new(buffer: Buffer<Event>) -> Self{
        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(100);
        let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
        Self { id: Uuid::new_v4().to_string(), buffer, sender: tx, eventloop: rx }
    }

    pub async fn spawn(&self, event: Event) -> Result<Self, tokio::sync::mpsc::error::SendError<Event>>{
        let sender = self.clone().sender;
        if let Err(e) = sender.send(event).await{
            return Err(e);
        }
        Ok(self.clone())
    }

    pub async fn run<F, R: std::future::Future<Output = ()> + Send + Sync + 'static, >(&self, callback: F)
    where F: Clone + Fn(Event, Option<StreamError>) -> R + Send + Sync + 'static{
        let get_rx = self.clone().eventloop;
        let mut rx = get_rx.try_lock();
        
        let cloned_callback = callback.clone();
        if rx.is_err(){
            let error = rx.unwrap_err();
            tokio::spawn(async move{
                cloned_callback(
                    Event::default(), 
                    Some(StreamError::Receiver(error.source().unwrap().to_string()))
                ).await; // calling callback with the passed in received event
            });
        } else{
            let mut actualRx = rx.unwrap(); 
            while let Some(event) = actualRx.recv().await{
                
                let event_for_first_tokio = event.clone();
                let cloned_event = event.clone();
                let cloned_callback = callback.clone();
    
                tokio::spawn(async move{
                    event_for_first_tokio.clone().process().await;
                });

                // executing the eventloop in the background
                tokio::spawn(async move{
                    cloned_callback(cloned_event, None).await; // calling callback with the passed in received event
                });
            }
        }

    }


}


// building a new network behaviour with kademlia and gossipsub protocols
impl NeuronBehaviour{
    pub fn new(key: Keypair) -> Self{

        let peer_id = key.clone().public().to_peer_id();
        let memory_store = MemoryStore::new(peer_id.clone());
        let kad = kad::Behaviour::new(peer_id, memory_store);

        // use hash of the message as the message id
        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };
        let gossipsubConfig = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(tokio::time::Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict) // enforce message signing when a peer sends it
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))
            .unwrap();

        let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsubConfig,
        ).unwrap();

        // all the behaviours need to build the synapse network protocol
        Self{
            kademlia: kad,
            gossipsub,
        }

    }

}

impl Actor for Neuron{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {

        let pid = ctx.address(); // actor or process address or unique id

        // start swarm event loop in the background thread
        let mut this = self.clone();
        tokio::spawn(async move{
            this.startP2pSwarmEventLoop().await
        });

        ctx.run_interval(std::time::Duration::from_secs(10), |actor, ctx|{

            let mut this = actor.clone(); 
            tokio::spawn(async move{
                this.executekMeIntervally().await;
            });

        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        log::error!("neuron actor has stopped working");        
    }
    
}

impl ObjectStorage for MinIoDriver{

    type Driver = Self;
    async fn save(&mut self) {
        
    }
    async fn getFile(&mut self, fId: String) -> &[u8] {
        let file = &[0];
        file
    }

    fn checksum(&mut self, file: &mut [u8]) -> bool {
        true
    }
}

impl ObjectStorage for SeaFileDriver{

    type Driver = Self;
    async fn save(&mut self) {
        
    }
    async fn getFile(&mut self, fId: String) -> &[u8] {
        let file = &[0];
        file
    }

    fn checksum(&mut self, file: &mut [u8]) -> bool {
        true
    }
}

impl ObjectStorage for DigiSpaces{
    
    type Driver = Self;
    async fn save(&mut self){
        
    }
    async fn getFile(&mut self, fId: String) -> &[u8]{
        let file = &[0];
        file
    }
    
    fn checksum(&mut self, file: &mut [u8]) -> bool {
        true
    }
}

impl ObjectStorage for LocalFileDriver{
    
    type Driver = Self;
    async fn getFile(&mut self, fId: String) -> &[u8] {
        let file = &[0];
        file
    }
    async fn save(&mut self) {
        
    }
    fn checksum(&mut self, file: &mut [u8]) -> bool {
        true
    }
}

// make C3 as an isolated actor worker (message passing, interval execution in light thread) 
impl Actor for C3{
    type Context = Context<Self>;
}

// impl the interface for any T
// impl<T> ServiceExt1 for T{
//     fn startService(&mut self) {}
//     fn stopService(&mut self) {}
// }
impl ServiceExt1 for C3{
    fn startService(&mut self) {}
    fn stopService(&mut self) {}
}

impl OnionStream for Neuron{
    type Model = Neuron;

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
        let owned_last_event = last_event.clone();

        match eventType{
            "send" => {

                match streamer{
                    "local" => { // use internal executor channel and eventloop receiver
                        // sending in the background
                        let first_token_last_event = owned_last_event.clone();
                        
                        // spawn an event for the executor, this would send the event into the channel
                        // in the background lightweight thread
                        tokio::spawn(async move{
                            match cloned_get_internal_executor.spawn(first_token_last_event).await{
                                Ok(this) => {
                                    tokio::spawn(
                                        callback(
                                            owned_last_event.to_owned(), 
                                            None
                                        )
                                    );
                                },
                                Err(e) => {
                                    tokio::spawn(
                                        callback(
                                            owned_last_event.to_owned(), 
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
                    },
                }
            },
            "receive" => {
                match streamer{
                    "local" => { // use internal executor channel and eventloop receiver
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

// used for en(de)crypting data in form of string
impl Crypter for String{
    fn decrypt(&mut self, secure_cell_config: &mut SecureCellConfig){

        // encrypt() method converts the raw string into hex encrypted thus
        // calling decrypt method on the hex string returns the 
        // raw string, we should decode the hex into the encrypted bytes
        // then feed the decryptor to decrypt the bytes and return the
        // raw bytes then convert the raw bytes into the string.
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
                
                // convert the encrypted bytes into the hex
                let stringified_data = hex::encode(&encrypted);
                
                // update the self or the string with the hex encrypted data
                *self = stringified_data;

                // update the data field with the encrypted content bytes
                // the method won't update the data field in its logic
                secure_cell_config.data = encrypted; 

            },
            Err(e) => {
                
                // don't update data field in secure_cell_config instance
                // the raw data remains the same as before.
            }
        };

    }

}

/* -------------------------------------------------------
    impl Crypter for &[u8]{} issues:
    since all the function types and params will be dropped as soon as the 
    function gets executed thus we can't take a reference to any of those type
    and update the self with an slice!
    
    *self = encrypted.clone().as_slice();
    
    encrypted.clone() creates a temporary cloned value.
    as_slice() creates another temporary reference to that clone.
    once this line ends, the temporary value is dropped, leaving *self pointing to invalid memory.

    NOTE: don't use slice as musch as you can use Vec<u8>

*/
impl Crypter for Vec<u8>{
    fn decrypt(&mut self, secure_cell_config: &mut SecureCellConfig) {
        // self refers to the hex bytes of the encrypted content
        secure_cell_config.data = hex::decode(&self).unwrap();
        match Wallet::secure_cell_decrypt(secure_cell_config){ // passing the redis secure_cell_config instance
            Ok(data) => {

                // update the self
                *self = data.clone();

                secure_cell_config.data = data;
            },
            Err(e) => {

                // don't update data field in secure_cell_config instance
                // the encrypted data remains the same as before.
            }
        };
    }
    fn encrypt(&mut self, secure_cell_config: &mut SecureCellConfig) {
        
        secure_cell_config.data = self.clone().to_vec(); // self is the &[u8] content
        match Wallet::secure_cell_encrypt(secure_cell_config){ // passing the redis secure_cell_config instance
            Ok(encrypted) => {
                
                // we've got the encrypted data which can be used to write into the file
                *self = encrypted.clone();
                
                secure_cell_config.data = encrypted; // data is the encrypted content utf8 bytes
            },
            Err(e) => {

                // don't update data field in secure_cell_config instance
                // the encrypted data remains the same as before.
            }
        };
        
    }
}

impl ShaHasher for String{
    fn hash(&mut self) {
        let hashed = Wallet::generate_sha256_from(&self);
        *self = hex::encode(hashed);
    }
}

impl ServiceExt for AppService{
    fn start(&mut self) {
        
    }
    fn status(&self) {
        
    }
}
