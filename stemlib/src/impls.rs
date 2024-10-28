


/* ------------------------------- implementations: 
    schemas implementations 
*/

use crate::*;
use crate::messages::*;
use crate::schemas::*;
use crate::interfaces::*;


impl Drop for NeuronActor{
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

impl NeuronActor{

    pub async fn new(bufferSize: usize, rmqConfig: Option<RmqConfig>) -> Self{
        NeuronActor{
            synProt: {
                // building the swarm object with our network behaviour contains our protocol
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

                SynapseProtocol{
                    swarm: std::sync::Arc::new(
                        tokio::sync::Mutex::new(
                            swarm
                        )
                    )
                }
                    
            },
            peerId: {
                let edkeys = Keypair::generate_ed25519();
                let local_peer_id = PeerId::from_public_key(&edkeys.public());
                local_peer_id
            },
            wallet: Some(wallexerr::misc::Wallet::new_ed25519()),
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
            rmqConfig,
            dependency: std::sync::Arc::new(AppService{}),
            state: 0 // this can be mutated by sending the update state message to the actor
        }
    }

    pub async fn sendRpcRequest(&mut self, rpcConfig: RmqRequestConfig, encConfig: Option<CryptoConfig>){

    }

    pub async fn sendP2pRequest(&mut self, p2pConfig: P2pRequestConfig, encConfig: Option<CryptoConfig>){

    }

    pub async fn receiveRpcResponse(&mut self, rpcConfig: RmqResponseConfig, encConfig: Option<CryptoConfig>) -> String{

        todo!()
    }

    pub async fn receiveP2pResponse(&mut self, p2pConfig: P2pResponseConfig, encConfig: Option<CryptoConfig>)  -> String{

        todo!()
    }

    // in swarm event loop we'll handle all swarm network behaviours 
    // including stream, kademlia, gossipsub and request response events
    pub async fn startP2pSwarmEventLoop(&mut self, synProt: SynapseProtocol){
        
        // https://github.com/libp2p/rust-libp2p/blob/master/examples/distributed-key-value-store/src/main.rs
        // https://github.com/libp2p/rust-libp2p/blob/master/examples/chat/src/main.rs
        let mut getSwarm = synProt.swarm;
        let mut swarm = getSwarm.lock().await;

        swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap()).unwrap();
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        // TODO 
        // swarm eventloop is a loop that waits for incoming events to 
        // like kademlia, gossipsub and request response events
        // ...


    }

    pub async fn p2pPublish(&mut self, p2pConfig: P2pPublishConfig){

        let P2pPublishConfig { topic, peerId, message, synProt } = p2pConfig;
        
        let mut getSwarm = synProt.swarm;
        let mut swarm = getSwarm.lock().await;
        let topic = gossipsub::IdentTopic::new(topic);
        let msgId = swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.as_bytes()).unwrap();
        
        // TODO - use self.on()
        // ...

    }   

    pub async fn p2pConsume(&mut self, p2pConfig: P2pConsumeConfig){

        let P2pConsumeConfig { topic, synProt } = p2pConfig;

        let mut getSwarm = synProt.swarm;
        let mut swarm = getSwarm.lock().await;
        let topic = gossipsub::IdentTopic::new(topic);
        swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

        // TODO - use self.on()
        // ...

    }

    pub async fn rmqPublish(&self, data: &str, exchange: &str, routing_key: &str, 
        exchange_type: &str, secureCellConfig: SecureCellConfig, redisUniqueKey: &str){

            // TODO
            // use self.on() method 

            /*
                let zerlog_producer_actor = self.clone().zerlog_producer_actor;
                let this = self.clone();

                // these are must be converted into String first to make longer lifetime 
                // cause &str can't get moved into tokio spawn as its lifetime it's not 
                // static the tokio spawn lives longer than the &str and the &str gets 
                // dropped out of the ram once the function is finished with executing
                let exchange = exchange.to_string();
                let routing_key = routing_key.to_string();
                let exchange_type = exchange_type.to_string();
                let data = data.to_string();
                let redisUniqueKey = redisUniqueKey.to_string();

                tokio::spawn(async move{

                    let storage = this.clone().app_storage.clone();
                    let rmq_pool = storage.as_ref().unwrap().get_lapin_pool().await.unwrap();
                    let redis_pool = storage.as_ref().unwrap().get_redis_pool().await.unwrap();
                    

                    // store the config on redis for future validation in consumer
                    let mut redis_conn = redis_pool.get().await.unwrap(); // TODO - handle the error properly
                    let redisKey = format!("Rmq_SecureCellConfig-{}", redisUniqueKey);
                    let configString = serde_json::to_string(&secureCellConfig).unwrap();
                    let _: () = redis_conn.set(redisKey, configString).await.unwrap();


                    // trying to ge a connection from the pool
                    match rmq_pool.get().await{
                        Ok(pool) => {

                            // -ˋˏ✄┈┈┈┈ creating a channel in this thread
                            match pool.create_channel().await{
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
                                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                let e_string = &e.to_string();
                                                let error_content = e_string.as_bytes().to_vec();
                                                let mut error_instance = HoopoeErrorResponse::new(
                                                    *constants::STORAGE_IO_ERROR_CODE, // error code
                                                    error_content, // error content
                                                    ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                    "NotifBrokerActor.exchange_declare", // method
                                                    Some(&zerlog_producer_actor)
                                                ).await;
                                                
                                            }

                                        };

                                    /* =================================================================================== */
                                    /* ================================ PRODUCING PROCESS ================================ */
                                    /* =================================================================================== */
                                    // async task: publish messages to exchange in the background in a lightweight thread of execution
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
                                                        use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                        let error_content_ = get_confirmation.unwrap_err();
                                                        let e_string = &error_content_.to_string();
                                                        let error_content = e_string.as_bytes().to_vec();
                                                        let mut error_instance = HoopoeErrorResponse::new(
                                                            *constants::STORAGE_IO_ERROR_CODE, // error code
                                                            error_content, // error content
                                                            ErrorKind::Storage(crate::error::StorageError::Rmq(error_content_)), // error kind
                                                            "NotifBrokerActor.get_confirmation", // method
                                                            Some(&zerlog_producer_actor)
                                                        ).await;

                                                        return; // needs to terminate the caller in let else pattern
                                                    };

                                                    if confirmation.is_ack(){
                                                        log::info!("publisher confirmation is acked");
                                                    }

                                                },
                                                Err(e) => {
                                                    use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                    let error_content = &e.to_string();
                                                    let error_content = error_content.as_bytes().to_vec();
                                                    let mut error_instance = HoopoeErrorResponse::new(
                                                        *constants::STORAGE_IO_ERROR_CODE, // error code
                                                        error_content, // error content
                                                        ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                        "NotifBrokerActor.basic_publish", // method
                                                        Some(&zerlog_producer_actor)
                                                    ).await;

                                                }
                                            }
                                    });

                                },
                                Err(e) => {
                                    use crate::error::{ErrorKind, HoopoeErrorResponse};
                                    let error_content = &e.to_string();
                                    let error_content = error_content.as_bytes().to_vec();
                                    let mut error_instance = HoopoeErrorResponse::new(
                                        *constants::STORAGE_IO_ERROR_CODE, // error code
                                        error_content, // error content
                                        ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                        "NotifBrokerActor.create_channel", // method
                                        Some(&zerlog_producer_actor)
                                    ).await;

                                }
                            }
                        },
                        Err(e) => {

                            use crate::error::{ErrorKind, HoopoeErrorResponse};
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();
                            let mut error_instance = HoopoeErrorResponse::new(
                                *constants::STORAGE_IO_ERROR_CODE, // error code
                                error_content, // error content
                                ErrorKind::Storage(crate::error::StorageError::RmqPool(e)), // error kind
                                "NotifBrokerActor.produce_pool", // method
                                Some(&zerlog_producer_actor)
                            ).await;

                        }
                    };
                    
                }); 
            */

    }

    pub async fn rmqConsume(&self,
        consumer_tag: &str, queue: &str, 
        binding_key: &str, exchange: &str,
        decryptionConfig: Option<CryptoConfig>
    ){

            // this can be used for sending the received message to some ws server 
            let internal_executor = &self.internal_executor;

            // TODO
            // also send consumed data to the streamer channel of the internal executor 
            // use self.on() method

            /* 
                let storage = self.app_storage.clone();
                let rmq_pool = storage.as_ref().unwrap().get_lapin_pool().await.unwrap();
                let redis_pool = storage.unwrap().get_redis_pool().await.unwrap();
                let notif_mutator_actor = self.notif_mutator_actor.clone();
                let zerlog_producer_actor = self.clone().zerlog_producer_actor;
                let notif_data_sender = self.notif_broker_ws_sender.clone();
                
                match rmq_pool.get().await{
                    Ok(conn) => {

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
                                    let e = create_queue.unwrap_err();
                                    use crate::error::{ErrorKind, HoopoeErrorResponse};
                                    let error_content = &e.to_string();
                                    let error_content = error_content.as_bytes().to_vec();
                                    let mut error_instance = HoopoeErrorResponse::new(
                                        *constants::STORAGE_IO_ERROR_CODE, // error code
                                        error_content, // error content
                                        ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                        "NotifBrokerActor.queue_declare", // method
                                        Some(&zerlog_producer_actor)
                                    ).await;
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
                                    .queue_bind(q.name().as_str(), &exchange, &binding_key, 
                                        QueueBindOptions::default(), FieldTable::default()
                                    )
                                    .await
                                    { // trying to bind the queue to the exchange
                                        Ok(_) => {},
                                        Err(e) => {
                                            use crate::error::{ErrorKind, HoopoeErrorResponse};
                                            let error_content = &e.to_string();
                                            let error_content = error_content.as_bytes().to_vec();
                                            let mut error_instance = HoopoeErrorResponse::new(
                                                *constants::STORAGE_IO_ERROR_CODE, // error code
                                                error_content, // error content
                                                ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                "NotifBrokerActor.queue_bind", // method
                                                Some(&zerlog_producer_actor)
                                            ).await;
                                            return; // terminate the caller
                                        }
                                    }

                                // since &str is not lived long enough to be passed to the tokio spawn
                                // if it was static it would be good however we're converting them to
                                // String to pass the String version of them to the tokio spawn scope
                                let cloned_consumer_tag = consumer_tag.to_string();
                                let cloned_queue = queue.to_string();
                                let cloned_notif_data_sender_channel = notif_data_sender.clone();

                                /* =================================================================================== */
                                /* ================================ CONSUMING PROCESS ================================ */
                                /* =================================================================================== */
                                // start consuming in the background in a lightweight thread of execution
                                // receiving is considered to be none blocking which won't block the current thread. 
                                tokio::spawn(async move{

                                    // try to create the secure cell config and 
                                    // do passphrase and secret key validation logic before
                                    // consuming messages
                                    let secure_cell_config_uniqueKey = if let Some(mut config) = decryptionConfig.clone(){
                                
                                        config.secret = hex::encode(config.secret);
                                        config.passphrase = hex::encode(config.passphrase);

                                        // return the loaded instance from redis
                                        let secureCellConfig = SecureCellConfig{
                                            secret_key: config.clone().secret,
                                            passphrase: config.clone().passphrase,
                                            data: vec![],
                                        };

                                        (secureCellConfig, config.unique_key)

                                    } else{
                                        (SecureCellConfig::default(), String::from(""))
                                    };

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
                                            // stream over consumer to receive data from the queue
                                            while let Some(delivery) = consumer.next().await{
                                                match delivery{
                                                    Ok(delv) => {

                                                        /* WE SHOULD DO THE VALIDATION IN EVERY ITERATION */
                                                        let mut secure_cell_config = secure_cell_config_uniqueKey.0.clone();
                                                        let redisUniqueKey = secure_cell_config_uniqueKey.clone().1;
                                                        let mut redis_conn = redis_pool.get().await.unwrap(); // TODO - handle the error properly
                                                        let redisKey = format!("Rmq_SecureCellConfig-{}", redisUniqueKey);
                                                        let isKeyThere: bool = redis_conn.exists(&redisKey).await.unwrap();
                                                        if isKeyThere{
                                                            let getConfig: String = redis_conn.get(redisKey).await.unwrap();
                                                            let mut redisConfig = serde_json::from_str::<SecureCellConfig>(&getConfig).unwrap();
                                                            // validating...
                                                            if 
                                                                redisConfig.passphrase != secure_cell_config.passphrase || 
                                                                redisConfig.secret_key != secure_cell_config.secret_key
                                                                {
                                                                    log::error!("invalid secure cell config, CHANNEL IS NOT SAFE!");
                                                                    return;
                                                            }
                                                            
                                                        } else{
                                                            log::error!("Rmq configs are not set properly on Redis");
                                                        }

                                                        log::info!("[*] received delivery from queue#<{}>", q.name());
                                                        let consumedBuffer = delv.data.clone();
                                                        let hexed_data = std::str::from_utf8(&consumedBuffer).unwrap();
                                                        let mut payload = hexed_data.to_string();

                                                        let cloned_notif_data_sender_channel = cloned_notif_data_sender_channel.clone();
                                                        let redis_pool = redis_pool.clone();
                                                        let notif_mutator_actor = notif_mutator_actor.clone();
                                                        let cloned_zerlog_producer_actor = zerlog_producer_actor.clone();

                                                        // handle the message in the background light thread asyncly and concurrently
                                                        // by spawning a light thread for the async task
                                                        tokio::spawn(async move{

                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>
                                                            // ===>>>===>>>===>>>===>>>===>>> data decryption logic ===>>>===>>>===>>>===>>>===>>>
                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>
                                                            // if we have a config means the data has been encrypted
                                                            let finalData = if 
                                                                !secure_cell_config.clone().secret_key.is_empty() && 
                                                                !secure_cell_config.clone().passphrase.is_empty(){
                                                                    
                                                                // after calling decrypt method has changed and now contains raw string
                                                                // payload must be mutable since the method mutate the content after decrypting
                                                                payload.decrypt(&mut secure_cell_config);

                                                                // payload is now raw string which can be decoded into the NotifData structure
                                                                payload

                                                            } else{
                                                                // no decryption config is needed, just return the raw data
                                                                // there would be no isse with decoding this into NotifData
                                                                log::error!("secure_cell_config is empty, data is not encrypted");
                                                                payload
                                                            };
                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>
                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>
                                                            // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>

                                                            // either decrypted or the raw data as string
                                                            log::info!("[*] received data: {}", finalData);
                                                            
                                                            // decoding the string data into the NotifData structure (convention)
                                                            let get_notif_event = serde_json::from_str::<NotifData>(&finalData);
                                                            match get_notif_event{
                                                                Ok(notif_event) => {

                                                                    log::info!("[*] deserialized data: {:?}", notif_event);

                                                                    // =================================================================
                                                                    /* -------------------------- send to mpsc channel for ws streaming
                                                                    // =================================================================
                                                                        this is the most important part in here, we're slightly sending the data
                                                                        to downside of a jobq mpsc channel, the receiver eventloop however will 
                                                                        receive data in websocket handler which enables us to send realtime data 
                                                                        received from RMQ to the browser through websocket server: RMQ over websocket
                                                                        once we receive the data from the mpsc channel in websocket handler we'll 
                                                                        send it to the browser through websocket channel.
                                                                    */
                                                                    if let Err(e) = cloned_notif_data_sender_channel.send(finalData).await{
                                                                        log::error!("can't send notif data to websocket channel due to: {}", e.to_string());
                                                                    }

                                                                    // =============================================================================
                                                                    // ------------- if the cache on redis flag was activated we then store on redis
                                                                    // =============================================================================
                                                                    if exp_seconds != 0{
                                                                        match redis_pool.get().await{
                                                                            Ok(mut redis_conn) => {

                                                                                // key: String::from(notif_receiver.id) | value: Vec<NotifData>
                                                                                let redis_notif_key = format!("notif_owner:{}", &notif_event.receiver_info);
                                                                                
                                                                                // -ˋˏ✄┈┈┈┈ extend notifs
                                                                                let get_events: RedisResult<String> = redis_conn.get(&redis_notif_key).await;
                                                                                let events = match get_events{
                                                                                    Ok(events_string) => {
                                                                                        let get_messages = serde_json::from_str::<Vec<NotifData>>(&events_string);
                                                                                        match get_messages{
                                                                                            Ok(mut messages) => {
                                                                                                messages.push(notif_event.clone());
                                                                                                messages
                                                                                            },
                                                                                            Err(e) => {
                                                                                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                                                                let error_content = &e.to_string();
                                                                                                let error_content_ = error_content.as_bytes().to_vec();
                                                                                                let mut error_instance = HoopoeErrorResponse::new(
                                                                                                    *constants::CODEC_ERROR_CODE, // error code
                                                                                                    error_content_, // error content
                                                                                                    ErrorKind::Codec(crate::error::CodecError::Serde(e)), // error kind
                                                                                                    "NotifBrokerActor.decode_serde_redis", // method
                                                                                                    Some(&cloned_zerlog_producer_actor)
                                                                                                ).await;
                                                                                                return; // terminate the caller
                                                                                            }
                                                                                        }
                                                                        
                                                                                    },
                                                                                    Err(e) => {
                                                                                        // we can't get the key means this is the first time we're creating the key
                                                                                        // or the key is expired already, we'll create a new key either way and put
                                                                                        // the init message in there.
                                                                                        let init_message = vec![
                                                                                            notif_event.clone()
                                                                                        ];

                                                                                        init_message

                                                                                    }
                                                                                };

                                                                                // -ˋˏ✄┈┈┈┈ caching the notif event in redis with expirable key
                                                                                // chaching in redis is an async task which will be executed 
                                                                                // in the background with an expirable key
                                                                                tokio::spawn(async move{
                                                                                    let events_string = serde_json::to_string(&events).unwrap();
                                                                                    let is_key_there: bool = redis_conn.exists(&redis_notif_key.clone()).await.unwrap();
                                                                                    if is_key_there{ // update only the value
                                                                                        let _: () = redis_conn.set(&redis_notif_key.clone(), &events_string).await.unwrap();
                                                                                    } else{ // initializing a new expirable key containing the new notif data
                                                                                        /*
                                                                                            make sure you won't get the following error:
                                                                                            called `Result::unwrap()` on an `Err` value: MISCONF: Redis is configured to 
                                                                                            save RDB snapshots, but it's currently unable to persist to disk. Commands that
                                                                                            may modify the data set are disabled, because this instance is configured to 
                                                                                            report errors during writes if RDB snapshotting fails (stop-writes-on-bgsave-error option). 
                                                                                            Please check the Redis logs for details about the RDB error. 
                                                                                            SOLUTION: restart redis :)
                                                                                        */
                                                                                        let _: () = redis_conn.set_ex(&redis_notif_key.clone(), &events_string, exp_seconds).await.unwrap();
                                                                                    }
                                                                                });

                                                                            },
                                                                            Err(e) => {
                                                                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                                                let error_content = &e.to_string();
                                                                                let error_content_ = error_content.as_bytes().to_vec();
                                                                                let mut error_instance = HoopoeErrorResponse::new(
                                                                                    *constants::STORAGE_IO_ERROR_CODE, // error code
                                                                                    error_content_, // error content
                                                                                    ErrorKind::Storage(crate::error::StorageError::RedisPool(e)), // error kind
                                                                                    "NotifBrokerActor.redis_pool", // method
                                                                                    Some(&cloned_zerlog_producer_actor)
                                                                            ).await;
                                                                                return; // terminate the caller
                                                                            }
                                                                        }
                                                                    }

                                                                    // =============================================================================
                                                                    // ------------- if the store in db flag was activated we then store in database
                                                                    // =============================================================================
                                                                    if store_in_db{
                                                                        // -ˋˏ✄┈┈┈┈ store notif in db by sending message to the notif mutator actor worker
                                                                        // sending StoreNotifEvent message to the notif event mutator actor
                                                                        // spawning the async task of storing data in db in the background
                                                                        // worker of lightweight thread of execution using tokio threadpool
                                                                        tokio::spawn(
                                                                            {
                                                                                let cloned_message = notif_event.clone();
                                                                                let cloned_mutator_actor = notif_mutator_actor.clone();
                                                                                let zerlog_producer_actor = cloned_zerlog_producer_actor.clone();
                                                                                async move{
                                                                                    match cloned_mutator_actor
                                                                                        .send(StoreNotifEvent{
                                                                                            message: cloned_message,
                                                                                            local_spawn: true
                                                                                        })
                                                                                        .await
                                                                                        {
                                                                                            Ok(_) => { () },
                                                                                            Err(e) => {
                                                                                                let source = &e.to_string(); // we know every goddamn type implements Error trait, we've used it here which allows use to call the source method on the object
                                                                                                let err_instance = crate::error::HoopoeErrorResponse::new(
                                                                                                    *MAILBOX_CHANNEL_ERROR_CODE, // error hex (u16) code
                                                                                                    source.as_bytes().to_vec(), // text of error source in form of utf8 bytes
                                                                                                    crate::error::ErrorKind::Actor(crate::error::ActixMailBoxError::Mailbox(e)), // the actual source of the error caused at runtime
                                                                                                    &String::from("NotifBrokerActor.notif_mutator_actor.send"), // current method name
                                                                                                    Some(&zerlog_producer_actor)
                                                                                                ).await;
                                                                                                return;
                                                                                            }
                                                                                        }
                                                                                }
                                                                            }
                                                                        );
                                                                    }

                                                                },
                                                                Err(e) => {
                                                                    log::error!("[!] can't deserialized into NotifData struct");
                                                                    use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                                    let error_content = &e.to_string();
                                                                    let error_content_ = error_content.as_bytes().to_vec();
                                                                    let mut error_instance = HoopoeErrorResponse::new(
                                                                        *constants::CODEC_ERROR_CODE, // error code
                                                                        error_content_, // error content
                                                                        ErrorKind::Codec(crate::error::CodecError::Serde(e)), // error kind
                                                                        "NotifBrokerActor.decode_serde", // method
                                                                        Some(&cloned_zerlog_producer_actor)
                                                                    ).await;
                                                                    return; // terminate the caller
                                                                }
                                                            }
                                                        });

                                                        // acking here means we have processed payload successfully
                                                        match delv.ack(BasicAckOptions::default()).await{
                                                            Ok(ok) => { /* acked */ },
                                                            Err(e) => {
                                                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                                let error_content = &e.to_string();
                                                                let error_content = error_content.as_bytes().to_vec();
                                                                let mut error_instance = HoopoeErrorResponse::new(
                                                                    *constants::STORAGE_IO_ERROR_CODE, // error code
                                                                    error_content, // error content
                                                                    ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                                    "NotifBrokerActor.consume_ack", // method
                                                                    Some(&zerlog_producer_actor)
                                                                ).await;
                                                                return; // terminate the caller
                                                            }
                                                        }
                            
                                                    },
                                                    Err(e) => {
                                                        use crate::error::{ErrorKind, HoopoeErrorResponse};
                                                        let error_content = &e.to_string();
                                                        let error_content = error_content.as_bytes().to_vec();
                                                        let mut error_instance = HoopoeErrorResponse::new(
                                                            *constants::STORAGE_IO_ERROR_CODE, // error code
                                                            error_content, // error content
                                                            ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                            "NotifBrokerActor.consume_getting_delivery", // method
                                                            Some(&zerlog_producer_actor)
                                                        ).await;
                                                        return; // terminate the caller 
                                                    }
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            use crate::error::{ErrorKind, HoopoeErrorResponse};
                                            let error_content = &e.to_string();
                                            let error_content = error_content.as_bytes().to_vec();
                                            let mut error_instance = HoopoeErrorResponse::new(
                                                *constants::STORAGE_IO_ERROR_CODE, // error code
                                                error_content, // error content
                                                ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                "NotifBrokerActor.consume_basic_consume", // method
                                                Some(&zerlog_producer_actor)
                                            ).await;
                                            return; // terminate the caller 
                                        }
                                    }

                                });

                            },
                            Err(e) => {
                                the
                            },
                            Err(e) => {
                                use crate::error::{ErrorKind, HoopoeErrorResponse};
                                let error_content = &e.to_string();
                                let error_content = error_content.as_bytes().to_vec();
                                let mut error_instance = HoopoeErrorResponse::new(
                                    *constants::STORAGE_IO_ERROR_CODE, // error code
                                    error_content, // error content
                                    ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                    "NotifBrokerActor.consume_create_channel", // method
                                    Some(&zerlog_producer_actor)
                                ).await;
                                return; // terminate the caller   
                            }
                        }

                    },
                    Err(e) => {

                    }
                    Err(e) => {
                        use crate::error::{ErrorKind, HoopoeErrorResponse};
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();
                        let mut error_instance = HoopoeErrorResponse::new(
                            *constants::STORAGE_IO_ERROR_CODE, // error code
                            error_content, // error content
                            ErrorKind::Storage(crate::error::StorageError::RmqPool(e)), // error kind
                            "NotifBrokerActor.consume_pool", // method
                            Some(&zerlog_producer_actor)
                        ).await;
                        return; // terminate the caller
                    }
                };
            */
            
    }

    //==================================================================================
    /* -------------------------------------------------------------
        since futures are object safe traits hence they have all traits 
        features we can pass them to the functions in an static or dynamic 
        dispatch way using Arc or Box or impl Future or even as the return 
        type of a closure trait method, so a future task would be one of 
        the following types: 

            1) Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>
            2) Pin<Arc<dyn Future<Output = ()> + Send + Sync + 'static>
            3) Arc<dyn Fn() -> Pin<Arc<R>> + Send + Sync + 'static> where R: Future<Output = ()> + Send + Sync + 'static
            4) Box<dyn Fn() -> Pin<Box<R>> + Send + Sync + 'static> where R: Future<Output = ()> + Send + Sync + 'static
            5) Arc<Mutex<dyn Fn() -> Pin<Arc<R>> + Send + Sync + 'static>> where R: Future<Output = ()> + Send + Sync + 'static
            6) job: F where F: Future<Output = ()> + Send + Sync + 'static
            7) task: impl Future<Output = ()> + Send + Sync + 'static

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

    pub async fn na_getTask<F, R>(
        task1: F,
        task: impl std::future::Future<Output = ()> + Send + Sync + 'static
    ) where F: Fn() -> R + Send + Sync + 'static, R: std::future::Future<Output=()> + Send + Sync + 'static{
        
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
                tokio::spawn(async move{
                    job().await
                });
            }
        });

    }

}



impl<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S> Runner<J, S>{
    pub async fn execute(&mut self){
        let job = &self.job;
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

        Self{
            kademlia: kad,
            gossipsub,
        }

    }

}

impl Actor for NeuronActor{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {

        let pid = ctx.address(); // actor or process address or unique id

        // start swarm event loop once the actor starts
        let mut this = self.clone();
        tokio::spawn(async move{
            let mut getSwarm = this.synProt.swarm.clone();
            let mut swarm = getSwarm.lock().await;
            

        });

        ctx.run_interval(std::time::Duration::from_secs(10), |actor, ctx|{

            let mut this = actor.clone(); 
            tokio::spawn(async move{
                this.executekMeIntervally().await;
            });

        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        log::error!("neuron actor has stopped");        
    }
}