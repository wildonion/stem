


/* ------------------------------- messages: 
    messages that will be used to communicate 
    with neuron actors
*/

use crate::*;
use crate::dto::*;
use std::pin::Pin;
use std::future::Future;


#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct UpdateState{
    pub new_state: u8
}

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct ShutDown;

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct InjectPayload{
    pub payload: Vec<u8>, // the shellcode or bytecode
    pub method: TransmissionMethod,
}

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct BanCry{
    pub cmd: String,
    pub tx: Transaction,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct TalkTo{
    pub neuron: Recipient<HeyThere>, // we send a HeyThere message to the Neuron actor of type Recipient
    pub message: String,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct HeyThere{
    pub message: String
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Broadcast{
    pub local_spawn: bool, // either spawn in actor context or tokio threadpool
    pub notif_data: EventData,
    pub rmqConfig: Option<RmqPublishConfig>,
    pub p2pConfig: Option<P2pPublishConfig>,
    pub encryptionConfig: Option<CryptoConfig>,
}

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct SendRequest{
    pub rmqConfig: Option<RmqRequestConfig>,
    pub p2pConfig: Option<P2pRequestConfig>,
    pub encryptionConfig: Option<CryptoConfig>,
}

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "ResponseData")]
pub struct ReceiveResposne{
    pub rmqConfig: Option<RmqResponseConfig>,
    pub p2pConfig: Option<P2pResponseConfig>,
    pub decryptionConfig: Option<CryptoConfig>,
}

#[derive(MessageResponse)]
pub struct ResponseData(pub Pin<Box<dyn Future<Output = Option<String>> + Send + Sync + 'static>>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe{ // we'll create a channel then start consuming by binding a queue to the exchange
    pub p2pConfig: Option<P2pConsumeConfig>,
    pub rmqConfig: Option<RmqConsumeConfig>,
    pub local_spawn: bool, // either spawn in actor context or tokio threadpool
    pub callback: IoEvent,
    pub decryptionConfig: Option<CryptoConfig>
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ExecutePriodically{
    pub period: u64,
    pub job: Io,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Execute(pub Io, pub bool);

// always pin the boxed or arced type specially future traits 
// into the ram so Rust can dynamically allocate more space for 
// them on the heap without moving them into a new location cause 
// in that case we violate the ownership and pin rules.

// an io type is an arced closure which returns a pinned boxed 
// version of an async object or future trait
pub type Io = Arc<dyn Fn() -> Pin<Box<dyn std::future::Future<Output = ()> 
    + Send + Sync + 'static>> 
    + Send + Sync + 'static>;

// io event
pub type IoEvent = Arc<dyn Fn(Event) -> Pin<Box<dyn std::future::Future<Output = ()> 
    + Send + Sync + 'static>> 
    + Send + Sync + 'static>;