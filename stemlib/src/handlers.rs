


/* ------------------------------- message handlers: 
    this ensures the actor isolation stays safe and secure cause there is no direct 
    state mutating using some kina locking mechanism like mutex, it's handled only by 
    sending message to the actor and the actor receives it from its mailbox and 
    runs it accordingly. 
*/

use salvo::Router;

use crate::*;
use crate::messages::*;
use crate::dto::*;
use crate::impls::*;



impl ActixMessageHandler<Broadcast> for Neuron{
    
    type Result = ();
    fn handle(&mut self, msg: Broadcast, ctx: &mut Self::Context) -> Self::Result {

        // unpacking the notif data
        let Broadcast { 
                rmqConfig,
                p2pConfig,
                local_spawn,
                notif_data,
                encryptionConfig,

            } = msg;
        
        let mut stringData = serde_json::to_string(&notif_data).unwrap();
        let mut scc = SecureCellConfig::default();
        let mut ruk = String::from(""); 

        let finalData = if encryptionConfig.is_some(){
            
            let CryptoConfig{ secret, passphrase, unique_key } = encryptionConfig.clone().unwrap();
            let mut secure_cell_config = &mut wallexerr::misc::SecureCellConfig{
                secret_key: hex::encode(secret),
                passphrase: hex::encode(passphrase),
                data: vec![],
            };
            
            scc = secure_cell_config.clone();
            ruk = unique_key;

            // after calling encrypt method stringData has changed and contains the hex encrypted data
            stringData.encrypt(secure_cell_config);
            
            stringData // contains aes256 encrypted data in hex format

        } else{
            stringData
        };

        let mut this = self.clone();

        // spawn the future in the background into the given actor context thread
        // by doing this we're executing the future inside the actor thread since
        // every actor has its own thread of execution.
        if local_spawn{
            async move{
                if let Some(rmqcfg) = rmqConfig{
                    let RmqPublishConfig { exchange_name, exchange_type, routing_key } = rmqcfg;
                    this.rmqPublish(&finalData, &exchange_name, &routing_key, &exchange_type, scc, &ruk).await;
                } else if let Some(p2pcfg) = p2pConfig{
                    this.p2pPublish(p2pcfg).await;
                } else{
                    return;
                }
            }
            .into_actor(self) // convert the future into an actor future of type NotifBrokerActor
            .spawn(ctx); // spawn the future object into this actor context thread
        } else{ // spawn the future in the background into the tokio lightweight thread
            tokio::spawn(async move{
                if let Some(rmqcfg) = rmqConfig{
                    let RmqPublishConfig { exchange_name, exchange_type, routing_key } = rmqcfg;
                    this.rmqPublish(&finalData, &exchange_name, &routing_key, &exchange_type, scc, &ruk).await;
                } else if let Some(p2pcfg) = p2pConfig{
                    this.p2pPublish(p2pcfg).await;
                } else{
                    return;
                }
            });
        }
        
        return;
        
    }

}

impl ActixMessageHandler<Subscribe> for Neuron{
    
    type Result = ();
    fn handle(&mut self, msg: Subscribe, ctx: &mut Self::Context) -> Self::Result {

        // unpacking the consume data
        let Subscribe { 
                rmqConfig,
                p2pConfig,
                local_spawn,
                decryptionConfig,
                callback

            } = msg; // the unpacking pattern is always matched so if let ... is useless
        
        let mut this = self.clone();
        
        // spawn the future in the background into the given actor context thread
        // by doing this we're executing the future inside the actor thread since
        // every actor has its own thread of execution.
        if local_spawn{
            async move{
                if let Some(rmqcfg) = rmqConfig{
                    let RmqConsumeConfig{ queue, exchange_name, routing_key, tag } = rmqcfg;
                    this.rmqConsume(
                        &tag, 
                        &queue, 
                        callback,
                        &routing_key, 
                        &exchange_name,
                        decryptionConfig
                    ).await;
                } else if let Some(p2pcfg) = p2pConfig{
                    this.p2pConsume(p2pcfg).await;
                } else{
                    return;
                }
            }
            .into_actor(self) // convert the future into an actor future of type NotifBrokerActor
            .spawn(ctx); // spawn the future object into this actor context thread
        } else{ // spawn the future in the background into the tokio lightweight thread
            tokio::spawn(async move{
                if let Some(rmqcfg) = rmqConfig{
                    let RmqConsumeConfig{ queue, exchange_name, routing_key, tag } = rmqcfg;
                    this.rmqConsume(
                        &tag, 
                        &queue, 
                        callback,
                        &routing_key, 
                        &exchange_name,
                        decryptionConfig
                    ).await;
                } else if let Some(p2pcfg) = p2pConfig{
                    this.p2pConsume(p2pcfg).await;
                } else{
                    return;
                }
            });
        }
        return; // terminate the caller

    }

}

impl ActixMessageHandler<UpdateState> for Neuron{
    type Result = ();
    fn handle(&mut self, msg: UpdateState, ctx: &mut Self::Context) -> Self::Result {
        self.state = msg.new_state;
    }
}

impl ActixMessageHandler<SendRequest> for Neuron{
    type Result = ();
    fn handle(&mut self, msg: SendRequest, ctx: &mut Self::Context) -> Self::Result {
        let SendRequest{ rmqConfig, p2pConfig, encryptionConfig } = msg.clone();

        let mut this = self.clone();
        tokio::spawn(async move{
            if let Some(rmqcfg) = rmqConfig{
                this.sendRpcRequest(rmqcfg, encryptionConfig).await;
            } else if let Some(p2pcfg) = p2pConfig{
                this.sendP2pRequest(p2pcfg, encryptionConfig).await;
            } else{
                return;
            }
        });
    }
}

impl ActixMessageHandler<ReceiveResposne> for Neuron{
    type Result = ResponseData;
    fn handle(&mut self, msg: ReceiveResposne, ctx: &mut Self::Context) -> Self::Result {
        let ReceiveResposne { rmqConfig, p2pConfig, decryptionConfig } = msg.clone();

        let mut this = self.clone();
        // we can't do async io tasks inside the handle methods hence 
        // the only way in this case to receive the response is inside 
        // an async block which enforces us to put it inside the Box::pin
        // and return the pinned box which enables the caller await on it.
        ResponseData(
            Box::pin(async move{
            
                use tokio::sync::mpsc::channel;
                let (tx, mut rx) = channel(1024);
    
                tokio::spawn(async move{
                    if let Some(rmqcfg) = rmqConfig{
                        let res = this.receiveRpcResponse(rmqcfg, decryptionConfig).await;
                        tx.send(res).await;
                    } else if let Some(p2pcfg) = p2pConfig{
                        let res = this.receiveP2pResponse(p2pcfg, decryptionConfig).await;
                        tx.send(res).await;
                    } else{
                        return;
                    }
                });
    
                // we need to return the response to the caller, so we fixed this
                // by using channels 
                while let Some(res) = rx.recv().await{
                    return Some(res);
                }
    
                return None;
    
            })
        )
     
    }
}

impl ActixMessageHandler<ShutDown> for Neuron{
    type Result = ();
    fn handle(&mut self, msg: ShutDown, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

impl ActixMessageHandler<InjectPayload> for Neuron{
    type Result = ();
    fn handle(&mut self, msg: InjectPayload, ctx: &mut Self::Context) -> Self::Result {
        
        let InjectPayload{ payload, method} = msg.clone();
        match method{
            TransmissionMethod::Local => {

            },
            TransmissionMethod::Remote(methodName) => {
                
            },
            _ => {}
        }
    }
}

impl ActixMessageHandler<BanCry> for Neuron{
    type Result = ();
    fn handle(&mut self, msg: BanCry, ctx: &mut Self::Context) -> Self::Result{

        let BanCry { cmd, tx } = msg;
        match cmd.as_str(){
            "executeTransaction" => {
                let this = self.clone();
                let task = async move{
                    let getNeuronTransactions = &this.transactions;
                    if getNeuronTransactions.is_some(){
                        let neuronTransactions = getNeuronTransactions.as_ref().unwrap();
                        let mut lockedNeuronTransactions = neuronTransactions.lock().await;
                        let tx = Transaction::new(
                            tx,
                            Uuid::new_v4().to_string().as_str(), 
                            100.0, 
                            "0x01", 
                            "0x00", 
                            2.5, 
                            String::from("some data").as_bytes()
                        ).await;
                        (*lockedNeuronTransactions).push(tx);
                    } else{
                        log::error!("[!] actor has no transactions");
                    }
                };
                spawn(task); // spawn the task of pushing tx into the neuron transactions in the background thread
            },
            _ => {
                log::error!("[!] invalid command for bancry!");
            }
        }
        
    }
}

// this handler used to send a message to a local actor 
impl ActixMessageHandler<TalkTo> for Neuron{
    type Result = ();
    fn handle(&mut self, msg: TalkTo, ctx: &mut Self::Context) -> Self::Result {
        let TalkTo { neuron, message } = msg.clone();
        async move{
            neuron.send(HeyThere { message }).await;
        }.into_actor(self)
        .spawn(ctx);
    }
}

// this handler is used to receive a message sent by other local actors
impl ActixMessageHandler<HeyThere> for Neuron{
    type Result = ();
    fn handle(&mut self, msg: HeyThere, ctx: &mut Self::Context) -> Self::Result {
        let HeyThere { message } = msg.clone();
        log::info!("message received from: {message:}");
    }
}

impl ActixMessageHandler<ExecutePriodically> for Neuron{
    type Result = ();
    fn handle(&mut self, msg: ExecutePriodically, ctx: &mut Self::Context) -> Self::Result {

        let ExecutePriodically{period, job} = msg;
        let clonedJob = job.clone(); // return type of closure is async io task

        // a callback is called after ticking the time during the interval process
        // we'll execute the job in each period of time. 
        ctx.run_interval(std::time::Duration::from_secs(period), move |actor, ctx|{
            
            // execute the passed in task inside the tokio light thread in the background
            tokio::spawn(clonedJob());

        });
    }
}

impl ActixMessageHandler<Execute> for Neuron{
    type Result = ();
    fn handle(&mut self, msg: Execute, ctx: &mut Self::Context) -> Self::Result {
        let Execute(job, local_spawn) = msg.clone();

        if local_spawn{
            // spawn the async task inside the actor thread itself
            job().into_actor(self).spawn(ctx);
        } else{
            // execute the task in the background light thread
            tokio::spawn(job()); // tokio takes the job() and await on it inside a light thread
        }
    }
}

impl ActixMessageHandler<TalkToContainer> for Container<Router>{ // use this to send the wake up message to a container
    type Result = ();
    fn handle(&mut self, msg: TalkToContainer, ctx: &mut Self::Context) -> Self::Result {
        let TalkToContainer { msg, container } = msg.clone();
        go!{
            {
                container.send(WakeUp { msg }).await;
            }
        }
    }
}

impl ActixMessageHandler<WakeUp> for Container<Router>{ // use this to wake up a container
    type Result = ();
    fn handle(&mut self, msg: WakeUp, ctx: &mut Self::Context) -> Self::Result {
        let WakeUp { msg } = msg.clone();
        match msg{
            MsgType::Serve => {
                let host = self.host.clone();
                let port = self.port.clone();
            },
            MsgType::Stop => {
                ctx.stop();
            },
            _ => {
                log::error!("not supported command for the container");
            }
        }

    }
}