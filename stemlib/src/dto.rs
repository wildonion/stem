


/* ------------------------------- schemas: 
    schemas and structures
*/

use std::pin::Pin;
use interfaces::ObjectStorage;
use interfaces::ServiceExt1;
use crate::*;
use crate::messages::*;
use crate::impls::*;
use crate::handlers::*;

pub type LapinPoolConnection = deadpool_lapin::Pool;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct State;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Env{
    pub states: Vec<State>
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct MdpFramework{
    pub env: Env,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct RmqPublishConfig{
    pub exchange_name: String,
    pub exchange_type: String,
    pub routing_key: String,
}

pub struct P2pPublishConfig{
    pub topic: String,
    pub peerId: String,
    pub message: String,
    pub synProt: SynapseProtocol
}

pub struct P2pConsumeConfig{
    pub topic: String,
    pub synProt: SynapseProtocol
}

#[derive(Clone)]
pub struct SynapseProtocol{
    pub swarm: std::sync::Arc<tokio::sync::Mutex<Swarm<NeuronBehaviour>>>
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct RmqRequestConfig{
    pub notif_data: EventData,
    pub requestQueue: String, // a queue to send messages for server: server <---requestQueue---> client
    pub correlationId: String, // used to identify messages in reply queue which contains responses from server, client consume from this queue
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct P2pRequestConfig{
    pub peerId: String,
    pub message: String
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct RmqResponseConfig{
    pub notif_data: EventData,
    pub requestQueue: String, // used to specify to which queue client sends the message for server
    pub encryptionConfig: Option<CryptoConfig>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct P2pResponseConfig;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum TransmissionMethod{
    #[default]
    Local,
    Remote(String)
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct CryptoConfig{
    pub secret: String,
    pub passphrase: String,
    pub unique_key: String
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct RmqConsumeConfig{
    /* -ˋˏ✄┈┈┈┈ 
        following queue gets bounded to the passed in exchange type with its 
        routing key, when producer wants to produce notif data it sends them 
        to the exchange with a known routing key, any queue that is bounded 
        to that exchange routing key will be filled up with messages coming 
        from the producer and they stay in there until a consumer read them
    */
    pub queue: String,
    pub exchange_name: String,
    /* -ˋˏ✄┈┈┈┈ 
        routing_key is pattern for the exchange to route the messages to the 
        bounded queue.
        multiple producers can send their messages to a single exchange but 
        each of with different routing keys.
        any queue that is bounded to the exchange routing key will receive 
        all the messages that follows the pattern inside the routing_key.
        a message can be sent from producer to an exchange in a topic way with 
        an sepecific routing key which tells the exchange this is the way of 
        receiving messages that a bounded queue does since we might have 
        sent messages to the same exchange with multiple different routing 
        keys per each message and for a queue that is bounded to the exchange 
        with the passed in routing key can only receives the messages that 
        follow the pattern in the selected routing key. so the routing key in 
        consumer is the patterns for this queue to tell exchange to what 
        messages this queue is interested in:

        1) producer produces messages and send them to the exchange with an specific routing key
        2) a consumer create its own queue and bind it to the exchange with the bind key that 
           is interested to receive the message from the exchange based on that key.
        3) it's notable that a queue can be bounded to multiple exchange at the same time 
           it allows to receive different messages based on each exchange routing key.
                                                                                                                 --------          ---------
                                                                                                                | queue1 | <----- |consumer1|
                                                                        ------> routing_key1 <---------------------------          ---------
                                                                       |                                            
        producer1 ----------                                       -----------------> routing_key0  .........        
                            |____ messages > routing_key1 ------> | exchange1|                                                
                             ____ messages > routing_key4 ------>  -----------------> routing_key2  .........                                   
                            |                                          |                                --------        -----------
       producer2 -----------                                           |                               | queue2 | <----| consumer2 |
                                                                        ------> routing_key4 <------------------        -----------
                                                                     ----------                             |
                                                                    | exchange2| -----bind(routing_key)-----
                                                                     ----------
    */
    pub routing_key: String, // patterns for this queue to tell exchange what messages this queue is interested in
    pub tag: String,
}

// the following defines our p2p protocol
#[derive(NetworkBehaviour)]
pub struct NeuronBehaviour {
    pub kademlia: kad::Behaviour<MemoryStore>, // peer discovery over wan
    pub gossipsub: gossipsub::Behaviour, // pubsub messaging
}

#[derive(Clone)]
pub enum ContractType{
    Verifier,
    Transmitter(Addr<Neuron>),
}

#[derive(Clone)]
pub struct Contract{
    pub cont_type: ContractType,
    pub opcode: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct YmlConfig{
    pub rmqHost: String,
    pub rmqPort: u16,
    pub rmqUsername: String,
    pub rmqPassword: String,
    pub redisHost: String,
    pub redisPort: u16,
    pub redisUsername: String,
    pub redisPassword: String,
}

#[derive(Clone)]
pub struct AppService{
}


#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct CompressionConfig{

}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ElectricNerveImpulse{

}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SynapticConnection{

}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum ActionType{ // all the action type that causes the notif to get fired
    #[default]
    EventCreated,
    EventExpired,
    EventLocked,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct EventData{
    pub id: String,
    pub receiver_info: String,
    pub action_data: serde_json::Value, // you can map this to any structure you know!
    pub actioner_info: String,
    pub action_type: ActionType,
    pub fired_at: i64,
    pub is_seen: bool,
}

#[derive(Clone)]
pub struct Worker{
    pub id: String,
    pub thread: std::sync::Arc<tokio::task::JoinHandle<()>>,
}

/* ------------------------------------------------------------------------
this is the internal executor, it's a job or task queue eventloop
it has a buffer of Event objets, thread safe eventloop receiver and 
a sender to send data to its channel, this is the backbone of actor
worker objects they talk locally through the following pattern sicne
they have isolated state there is no mutex at all mutating the state
would be done through message sending pattern.
threadpool executor eventloop: 
     sender, 
     Vec<JoinHanlde<()>>, 
     while let Some(task) = rx.lock().await.recv().await{ spawn(task); }
*/
#[derive(Clone)]
pub struct InternalExecutor<Event>{
    pub id: String, 
    pub buffer: Buffer<Event>,
    pub sender: tokio::sync::mpsc::Sender<Event>,
    // what does the eventloop do? it receives Event inside tokio::spawn and execute them in the background
    pub eventloop: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Event>>> 
}

// a job contains the io task 
pub struct Job<J: Clone, S>
where J: std::future::Future<Output = ()> + Send + Sync + 'static{
    pub id: String, 
    pub task: Task<J, S>,
    pub io: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>> + Send + Sync + 'static >
}

// Pin, Arc, Box are smart pointers Pin pin the pointee into the ram 
// at an stable position which won't allow the type to gets moved.
struct Task0<R, F: Fn() -> std::pin::Pin<Arc<R>> + Send + Sync + 'static> where 
R: std::future::Future<Output=()> + Send + Sync + 'static{
    pub job: Arc<F>
}

// an event contains the offset in the cluster, execution status and the data
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Event{
    pub data: EventData,
    pub status: EventStatus,
    pub offset: u64, // the position of the event inside the brain network
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum EventStatus{
    #[default]
    Committed,
    Executed,
    Halted
}

// a buffer contains a thread safe vector of Events
#[derive(Clone)]
pub struct Buffer<E>{ // eg: Buffer<Event>
    pub events: std::sync::Arc<tokio::sync::Mutex<Vec<E>>>,
    pub size: usize
}

#[derive(Clone, Debug)]
pub enum StreamError{
    Sender(String),
    Receiver(String)
}

#[derive(Clone, Debug)]
pub enum NeuronError{
    Job(String),
    Buffer(String),
}

/* ----------------------------
    use neuron actor worker and wrangler worker to execute async io processes workload 
    and functions inside an isolated environment like light thread which doesn't share memory 
    with other isolates, each worker gets compiled to wasm to be executed in a single v8 
    engine, each neuron actor worker execute tasks in it arbiter event loop in a thread 
    in the background.
*/
#[derive(Clone)]
pub struct Neuron{
    pub name: String,
    pub synProt: SynapseProtocol,                                                    /* -- synapse protocol -- */
    pub peerId: libp2p::PeerId,                                                      /* -- p2p peer id -- */
    pub rmqPool: Arc<LapinPoolConnection>,                                           /* -- rmq config -- */
    pub wallet: Option<wallexerr::misc::Wallet>,                                     /* -- a cryptography indentifier for each neuron -- */
    pub metadata: Option<serde_json::Value>,                                         /* -- json object contains the actual info of an object which is being carried by this neuron -- */
    pub internal_executor: InternalExecutor<Event>,                                  /* -- eventloop sender and thread safe receiver, potentially we can use the actor msg sending pattern as well -- */
    pub transactions: Option<std::sync::Arc<tokio::sync::Mutex<Vec<Transaction>>>>,  /* -- all neuron atomic transactions -- */
    pub internal_worker: Option<std::sync::Arc<tokio::sync::Mutex<Worker>>>,         /* -- an internal lighthread worker -- */
    pub internal_locker: Option<std::sync::Arc<tokio::sync::Mutex<()>>>,             /* -- internal locker -- */
    pub internal_none_async_threadpool: std::sync::Arc<Option<RunnerActor>>, /* -- internal none async threadpool -- */
    pub signal: std::sync::Arc<std::sync::Condvar>,                                  /* -- the condition variable signal for this neuron -- */
    pub dependency: std::sync::Arc<dyn ServiceExt>,                                  /* -- inject any type that impls the ServiceExt trait as a dependency -- */
    pub contract: Option<Contract>, // circom and noir for zk verifier contract (TODO: use crypter)
    pub state: u8
}

// custom error handler for the stem
#[derive(Debug)]
pub struct StemError{
    pub message: String, 
    pub code: u16,
    pub error: ErrorKind
}

#[derive(Debug)]
pub enum ErrorKind{
    Io(std::io::Error),
    StreamError
}

/* --------------------------------------------------
    reusable components are trait objects that can be implemented 
    for any type which enables us to access different types through 
    a single interface, can be used for testing, building sdks with 
    multiple engine like ObjectStorage trait for File handler that
    can supports uploading file to multiple backends, building actor 
    worker isoalted componenets that can talk with each other and other parts
    of the app through message sending, app contexts or services through
    dependency injection and dynamic dispatching.  

    accessing multiple types through a single interface
    we can use the interface trait to register a service 
    of any type whose impls the ServiceExt trait, build 
    reusable componenets, sdks and testing logics.
    by implementing Actor trait for the C3 each instance 
    of C3 can talk to each other and other components of 
    the app through message sending pattern.

    solid design pattern: n different type of services can be accessible 
    through a single interface like implementing Actor trait for a component
    with dep inj, dyn dist and poly which enables us for isolated talking
*/
pub struct C3{ // Context Container Component
    pub id: String,
    pub name: String,
    pub task: Arc<dyn Fn() -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>>,
    pub service: Box<dyn ServiceExt1> // the trait must be object safe trait for dep injection through dyn dispatch pattern
}

#[derive(Clone, Debug)]
pub struct MinIoDriver{
    pub content: Arc<Vec<u8>>,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct SeaFileDriver{
    pub content: Arc<Vec<u8>>,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct DigiSpaces{
    pub content: Arc<Vec<u8>>,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct LocalFileDriver{
    pub content: Arc<Vec<u8>>,
    pub path: String,
}