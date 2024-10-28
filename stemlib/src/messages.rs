


/* ------------------------------- messages: 
    messages that will be used to communicate 
    with neuron actors
*/

use crate::*;
use crate::schemas::*;
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartP2pSwarEventLoop{
    pub synprot: SynapseProtocol,
}

#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct StartGateWay{
    pub host: String,
    pub port: u16,
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
    pub encryptionConfig: Option<CryptoConfig>,
}

#[derive(MessageResponse)]
pub struct ResponseData(pub Pin<Box<dyn Future<Output = Option<String>> + Send + Sync + 'static>>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe{ // we'll create a channel then start consuming by binding a queue to the exchange
    pub p2pConfig: Option<P2pConsumeConfig>,
    pub rmqConfig: Option<RmqConsumeConfig>,
    pub local_spawn: bool, // either spawn in actor context or tokio threadpool
    pub decryptionConfig: Option<CryptoConfig>
}

