



/*


    https://neurosciencenews.com/ -> get brain schemas from here
    https://en.wikipedia.org/wiki/Unconscious_mind
    https://www.sas.upenn.edu/~cavitch/pdf-library/Solms_Unconscious.pdf
    https://writings.stephenwolfram.com/2024/08/whats-really-going-on-in-machine-learning-some-minimal-models/
    https://drive.google.com/file/d/1Es7Ew8fqkRKGFYfcmFZ8gGJWYOzncA6v/view?usp=sharing
    https://drive.google.com/file/d/1K2uO8D_qLhXWcAHDpICmbfFx3hfQ4Sah/view?usp=share_link -> Models of the Mind
    https://drive.google.com/file/d/17aF74xeydgx_BulPknWv6NiU4wgTBwfQ/view?usp=share_link -> Descartes' Error
    https://www.biorxiv.org/content/10.1101/703801v1.full.pdf -> neuralink
    https://github.com/wildonion/gvm/
    https://github.com/wildonion/cs-concepts
    TODO - a neuron sends message to its closest neuron using slot, epoch and graph concepts
    TODO - all data for communication between neuron actors must be serialized with capn'p proto to call each other methods directly inside the brain
    TODO - implement all cognitive neuroscience concepts and schemas and code the whole brain instead of having neural network like a brain engine using various macro syntax like brain!{}
    TODO - every neuron can be an actor (or the column of our input matrix) to construct the GNN in an async and multithreading manner in such a way that every actor which is a neuron can communicate with each other to get the data of the next or the last neuron asyncly also they can build and form the brain  
    TODO - implement entropy measures for the training process of the brain and how much events are random 
    NOTE - cortex is involved in higher processes in the human brain, including memory, thinking, learning, reasoning, problem-solving, emotions, consciousness and functions related to your senses
    NOTE - a brain structure can have multiple interfaces like being in void and illusion abstract situations which can be implemented using traits 
    
    every neuron is an actor working in an mdp env, which signs the data that wants to fire using its private key, other
    neurons know each other through the public key, the cryptography logics can be done using wallexerr, local neurons can 
    communicate through actor message passing and sending pubsub algo and outsider neurons can communicate through libp2p 
    gossipsub, redis or gRPC pubsub patterns also every brain can be a canister or ipc contract whic is an actor model 
    runs brain methods.

    neuron wallet :
        - ed25519 keypair
        - sign aes256 bits encrypted data with ed25519 private key 
        - id with ed25519 pubkey
        - fire the signature, pubkey, encrypted data to other neurons 


*/

use crate::*;





 







// ---------------
//   INTERFACES
// ---------------
pub trait Void{
    type Illusion<Neuron>; //-- we can have GAT with generic arg; the generic type of the Illusion type is Neuron, we can use this later to transfer an illusion between neurons 
    type Pain<Neuron>; //-- we can have GAT with generic arg; the generic type of the Pain type is Neuron, we can use this later to transfer the pain between neurons
}

pub trait Illusion{
    //// visual cortex of the brain is the area of the cerebral cortex 
    //// that processes visual information. It is located in the occipital lobe.
    fn VisualCortex(&self) -> () {
    
    }
}

pub trait Synapse{
    //-- we also have a lifetime 'f for the future event notifs means that all notifs must be valid as long as 'f
    type FutureEventNotif<'f, Neuron>; //-- we can have GAT with generic arg; the generic type of the FutureEventNotif type is Neuron, we can use this later to transfer the future events notif between the selected neurons (some special neurons are responsible for receiving the future event notifs)

    fn communicate(&self, n: Option<&Neuron>) -> Self; //-- this is not object safe trait cause it's returning an associated type which is Self
}

///////
/// an abstract trait which rebuild the whole brain network, neuron connections, destroy consciousness and renew the self
///////
pub trait Reconnect{ //-- the following method must be invoked on taking mushrooms for a long period of time
    fn rebuild(&self) -> Self  //-- we can bind traits and lifetimes to return type using where
        where Self: Sized{ //-- it'll return the type that this trait will be implemented for - since it could be no type to implement this for thus we have to boung the Self to Sized trait since the compiler can't detect the size of the Self (there might be no type yet!) 

            todo!()

    }
}

///////
/// an abstract trait which can echo the feeling of pain through the neurons to the whole brain
///////
trait Pain<T>{
    type Context;
    fn set_pain(&mut self, kind: T);
}
struct Brain;
struct RigthHandPain;
impl Pain<RigthHandPain> for Brain{
    type Context = Brain;
    fn set_pain(&mut self, kind: RigthHandPain){}
}


///////
/// an abstract trait which can buffer (store them) the suspended, unsolved and unaddressed data inside neurons
///////
pub trait Suspend{} //-- a buffer contains unaddressed issues, feelings, pains and etc..














// ---------------
//   STRUCTURES
// ---------------


//// decision making process will be done through the followings:
////      • select an event from the event pool at time T
////      • occure that event inside the brain using unconsciousness structures
////      • consciousness (the cerebral cortex) must 
////           - interpret that event
////           - choose a response from the generated response pool
pub struct EventPool; //// this is the pool of events that can be occured at a specific time T

pub struct CerebralCortex; //// this is the where the consciousness is located

pub struct ERTAS; //// this is the extended reticulothalamic activating system responsibles for awareness and awakeness and is located in the upper brain stem well below the cortex

//// BasalGanglia and Cerebellum structures receive input from and send output to the cerebral cortex which is the location of consciousness
//// they are responsible for cognitions, performing functions, feelings and memories (memories can't be retrieved consciously) thus, 
//// the basal ganglia and cerebellum form multisynaptic loops with the cerebral cortex.
pub struct BasalGanglia; //// the unconsciousness engine

pub struct Cerebellum; //// the unconsciousness engine

pub struct BrainContext<Neuron>(pub Vec<Neuron>, pub i64);


//// every neuron is an actor that can
//// communicate with each other
//// through the RPC channel synapses.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Neuron{
    pub id: Uuid,
    pub name: String,
    pub timestamp: i64,
    pub data: Vec<f32>, //// data is a 32 bits float 
}


pub struct Model{
    pub layers: Vec<Layer>
}

pub struct Layer{
    pub neurons: Vec<Neuron>, // columns
    pub data: Vec<f64>, // rows
    pub metadata: String,
}

impl<Neuron> Default for BrainContext<Neuron>{
    fn default() -> Self{
        BrainContext(vec![], chrono::Local::now().naive_local().timestamp())
    }
}


trait Mdp{}
struct Qlearning;
impl Mdp for Qlearning{}
// let qmodel: &'static dyn Mdp = &Qlearning;

impl Synapse for Neuron{

    //-- we also have a lifetime 'f for the future event notifs means that all notifs must be valid as long as 'f
    type FutureEventNotif<'f, Neuron> = BrainContext<Neuron>; //-- the type of FutureEventNotif with Neuron generic type is BrainContext; we've passed a generic of type Neuron since we want to use the BrainContext structure, and the generic type of that struct is also Neuron; BrainContext structure contains a list of selected neurons

    fn communicate(&self, next_neuron: Option<&Self>) -> Self{
        let next_neuron = next_neuron.unwrap();
        let new_neuron_data: Vec<f32> = self.data.iter().zip(next_neuron.data.iter()).map(|(x, y)| x * y).collect();
        Neuron{
            id: Uuid::new_v4(),
            name: "Genesis-AJG7$%-12".to_string(),
            timestamp: chrono::Local::now().naive_local().timestamp(),
            data: new_neuron_data
        }
    }
}

impl Default for Neuron{
    fn default() -> Self{
        Neuron{
            id: Uuid::new_v4(),
            name: "Genesis-AJG7$%".to_string(),
            timestamp: chrono::Local::now().naive_local().timestamp(),
            data: vec![0.0, 0.0]
        }
    }
}
