

use core::num;
use std::pin::Pin;
use std::{collections::HashMap};
use futures::future::{BoxFuture, FutureExt};
use tokio::net::tcp;
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;
use crate::dto::*;
use crate::messages::*;


// static requires constant value and constant values must be only stack data like &[] and &str otherwise
// we're not allowed to have heap data types like Vec, String, Box, Arc, Mutex in const and static as value
// also in order to mutate an static we should wrap around the arc and mutex to do so but inside lazy
// str is ?Sized and it must be behind pointer cause it's an slice of String, same for [] 
// generally only &[] and &str are allowed to be in const and static value, using other types than these
// requires a global static lazy arc mutex like so:
type SafeMap = Lazy<std::sync::Arc<tokio::sync::Mutex<HashMap<String, String>>>>;
pub static GLOBALMAP: SafeMap = Lazy::new(||{
    std::sync::Arc::new(
        tokio::sync::Mutex::new(
            HashMap::new()
        )
    )
});
pub static ONTHEHEAP: &[&str] = CONSTVALUE;
pub const CONSTVALUE: &[&str] = &["wildonion"];
pub const CHARSET: &[u8] = b"0123456789";


// -----------------------------------
// handling a recursive async function
// -----------------------------------
// https://rust-lang.github.io/async-book/07_workarounds/04_recursion.html
// NOTE - Future trait is an object safe trait thus we have to Box it with dyn keyword to have kinda a pointer to the heap where the object is allocated in runtime
// NOTE - a recursive `async fn` will always return a Future object which must be rewritten to return a boxed `dyn Future` to prevent infinite size allocation in runtime from heppaneing some kinda maximum recursion depth exceeded prevention process
// the return type can also be ... -> impl std::future::Future<Output=usize>
// which implements the future trait for the usize output also BoxFuture<'static, usize>
// is a pinned Box under the hood because in order to return a future as a type
// we have to return its pinned pointer since future objects are traits and 
// traits are not sized at compile time thus we have to put them inside the 
// Box or use &dyn to return them as a type and for the future traits we have
// to pin them into the ram in order to be able to solve them later so we must 
// return the pinned Box (Box in here is a smart pointer points to the future)
// or use impl Trait in function return signature. 
//
// async block needs to be pinned into the ram and since they are traits of 
// the Future their pointer will be either Box<dyn Trait> or &dyn Trait, 
// to pin them into the ram to solve them later.
//
// since async blocks are of type Future trait in roder to return them
// as a type their pointer either Box<dyn Trait> or &dyn Trait must be
// pinned into the ram to let us solve them later because rust doesn't 
// have gc and it'll drop the type after it moved into the new scope or
// another type thus for the future objects we must pin them to ram and 
// tell rust hey we're moving this in other scopes but don't drop it because
// we pinned it to the ram to solve it in other scopes, also it must have
// valid lifetime during the the entire lifetime of the app.
//
// BoxFuture<'fut, ()> is Pin<alloc::boxed::Box<dyn Future<Output=()> + Send + Sync + 'fut>>
pub fn async_gen_random_idx(idx: usize) -> BoxFuture<'static, usize>{ // NOTE - pub type BoxFuture<'a, T> = Pin<alloc::boxed::Box<dyn Future<Output = T> + Send + 'a>>
    async move{
        if idx <= CHARSET.len(){
            idx
        } else{
            gen_random_idx(rand::random::<u8>() as usize)
        }
    }.boxed() // wrap the future in a Box, pinning it
}
pub fn ret_boxed_future() -> std::pin::Pin<Box<dyn std::future::Future<Output=()>>>{ // Pin takes a pointer to the type and since traits are dynamic types thir pointer can be either &dyn ... or Box<dyn...>
    // ret future as a pinned box means pinning the pointer of future trait into the ram so they can't move
    Box::pin(async move{ // pinning the box pointer of async block into the ram to solve it later 
        ()
    })
}

// recursive random index generator
pub fn gen_random_idx(idx: usize) -> usize{
    if idx < CHARSET.len(){
        idx
    } else{
        gen_random_idx(rand::random::<u8>() as usize)
    }
}

pub struct MerkleNode{}
impl MerkleNode{

    pub fn new() -> Self{
        MerkleNode {  }
    }

    pub fn calculate_root_hash(&mut self, chain: Vec<String>){

    } 
}

#[derive(Debug, Clone)]
pub enum RuntimeCode{
    Err(u8),
    Ok(u8),

}

struct CodePid{
    pub ramdb: HashMap<String, String>
}


/*  ----------------------------------------------------------------------
    implementing a dynamic type handler for structs and enums using traits
    ----------------------------------------------------------------------
*/
trait TypeTrait{
    type Value; // this can be the implementor type

    /* 
        we can use the lifetime of self in struct and trait methods 
        to return pointer since the self is valid as long as the object 
        itself is valid during the execution of the app
    */
    fn get_data(&self) -> Self::Value;
    fn get_ctx_data(&self, ctx: Self::Value) -> Self;
    fn fill_buffer(&mut self) -> &[u8];
}

impl TypeTrait for CodePid{
    type Value = Self; // the CodePid structure

    fn fill_buffer(&mut self) -> &[u8] {
        todo!()
    }

    fn get_ctx_data(&self, ctx: Self::Value) -> Self {
        todo!()
    }

    fn get_data(&self) -> Self::Value {
        todo!()
    }
}

impl TypeTrait for MerkleNode{
    
    type Value = std::sync::Arc<tokio::sync::Mutex<HashMap<u32, String>>>;

    fn get_data(&self) -> Self::Value {
        
        let mutexed_data = std::sync::Arc::new(
            tokio::sync::Mutex::new(
                HashMap::new()
            )
        );
        mutexed_data
    }

    fn get_ctx_data(&self, ctx: Self::Value) -> Self {
        todo!()
    }

    fn fill_buffer(&mut self) -> &[u8] {
        todo!()
    }
}

struct Streamer;
struct Context<T>{data: T}
impl TypeTrait for Streamer{ // kinda polymorphism
    
    type Value = Context<Self>; /* Context data is of type Streamer */

    fn get_ctx_data(&self, ctx: Self::Value) -> Self {
        ctx.data
    }

    fn get_data(&self) -> Self::Value {
        todo!()
    }

    fn fill_buffer(&mut self) -> &[u8] {
        todo!()
    }

}

impl TypeTrait for RuntimeCode{
    
    type Value = std::sync::Arc<tokio::sync::Mutex<String>>;
    
    fn get_data(&self) -> Self::Value {
        
        let mutexed_data = std::sync::Arc::new(
            tokio::sync::Mutex::new(
                String::from("")
            )
        );
        mutexed_data

    }

    fn get_ctx_data(&self, ctx: Self::Value) -> Self {
        todo!()
    }

    fn fill_buffer(&mut self) -> &[u8] {
        todo!()
    }
}

pub trait NodeReceptor{
    type InnerReceptor;
    fn get_inner_receptor(&self) -> Self::InnerReceptor;
}

pub trait Activation<C>: Send + Sync + 'static + Clone + Default{
    type Acivator;
}

impl<C> Activation<C> for &'static [u8]{
    type Acivator = &'static [u8];
}

#[derive(Default)]
pub struct Synapse<A>{id: A}

#[derive(Default)]
pub struct Neuron1<A=u8>{
    pub data: Option<Synapse<A>>,
}

/* 
    this must be implemented for Neuron1<Synapse<A>>
    to be able to call get_inner_receptor() method
*/
impl<A: Default> NodeReceptor for Neuron1<Synapse<A>>
where Self: Clone + Send + Sync + 'static + Activation<String>, 
<Self as Activation<String>>::Acivator: Default{

    type InnerReceptor = Synapse<A>;
    fn get_inner_receptor(&self) -> Self::InnerReceptor {
        let id: A = Default::default();
        Synapse{
            id,
        }
    }
}

/* 
    this must be implemented for Neuron<String>
    to be able to call get_inner_receptor() method
*/
impl NodeReceptor for Neuron1<String>{

    type InnerReceptor = Synapse<String>;
    fn get_inner_receptor(&self) -> Self::InnerReceptor {
        Synapse{
            id: String::from(""),
        }
    }
}

/* 
    this must be implemented for Neuron<A>
    to be able to call get_inner_receptor() method
*/
impl NodeReceptor for Neuron1<u8>{

    type InnerReceptor = Synapse<u8>;
    fn get_inner_receptor(&self) -> Self::InnerReceptor {
        Synapse{
            id: 0,
        }
    }
}

pub fn fire<'valid, N, T: 'valid + NodeReceptor>(cmd: N, cmd_receptor: impl NodeReceptor) 
    -> <N as NodeReceptor>::InnerReceptor // or T::InnerReceptor
    where N: Send + Sync + 'static + Clone + NodeReceptor + ?Sized, 
    T: NodeReceptor, T::InnerReceptor: Send + Clone,
    /* casting generic N to NodeReceptor trait to access the InnerReceptor gat */
    <N as NodeReceptor>::InnerReceptor: Send + Sync + 'static{

    // with pointer we can borrow the type to prevent from moving and 
    // makes the type sizable at compile time by storing the address of 
    // none determined size of it inside the stack like str and []
    // box is sized with the size of its content allocated on the heap
    trait Test{}
    struct Neuronam{}
    let name = Neuronam{};
    impl Test for Neuronam{}
    let trait_name = &name as &dyn Test;
    struct AnotherNeuronam<T: Test, F> where F: FnOnce() -> (){
        pub data: T,
        pub new_data: F
    }
    impl<V: Test, T> AnotherNeuronam<V, T> where T: FnOnce() -> (){
        fn get_data(param: impl FnMut() -> ()) -> impl FnMut() 
            -> std::pin::Pin<Box<dyn std::future::Future<Output=String> + Send + Sync + 'static>>{
            ||{
                Box::pin(async move{
                    String::from("")
                })
            }
        }
        fn get_func() -> fn() -> String{
            fn get_name() -> String{
                String::from("")
            }
            get_name
        }
        }
    let another_name = AnotherNeuronam{data: name, new_data: ||{}};

    let cls = |func: fn() -> String|{
        func()
    };
    fn execute() -> String{
        String::from("wildonion")
    }
    cls(execute);

    let cls = ||{};
    let casted = &cls as &dyn Fn() -> (); // casting the closure to an Fn trait
    let name = (
        |name: String| -> String{
            name
        }
    )(String::from(""));
    
    enum Packet{
        Http{header: String},
        Tcp{size: usize}, // the size of the incoming buffer
        Snowflake{id: String}
    }
    let packet = Packet::Http { header: String::from("") };
    if let Packet::Http { header } = packet{
        println!("packet header bytes => {header:}");
    }

    enum UserName{
        Age,
        Id,
        Snowflake{id: String}
    }
    let enuminstance = (Packet::Tcp{size: 0 as usize}, Packet::Http { header: String::from("http header") });
    let res = match enuminstance{
        (Packet::Tcp { size: tcpsize }, Packet::Http{ header: httpheader }) | 
        (Packet::Http{ header: httpheader }, Packet::Tcp { size: tcpsize }) => {},
        (_, Packet::Snowflake{id: sid}) => if !sid.is_empty(){},
        _ => {}
    };

    /*  
        note that if we want to call get_inner_receptor() method
        on an instance of Neuron, the NodeReceptor trait must be
        implemented for every generic type in Neuron struct separately
        like:
            impl NodeReceptor for Neuron<String>{}
            impl NodeReceptor for Neuron<u8>{}
            impl NodeReceptor for Neuron<Synapse<A>>{}
    */
    let neuron = cmd;
    let neuron_ = Neuron1::<String>::default();
    
    cmd_receptor.get_inner_receptor();
    neuron.get_inner_receptor()
    // neuron_.get_inner_receptor()
    
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum ActionType{
    #[default]
    A1
} 
type Method = fn() -> i32;
fn run<'lifetime>(param: impl Fn() -> ActionType, method: &'lifetime Method)
// bounding generic Method to traits and lifetimes
where Method: Send + Sync + 'static{}
fn execute<'f, F>(param: &'f mut F) -> () 
// bounding generic F to closure, lifetimes and other traits
where F: Fn() -> ActionType + Send + Sync + 'static{}

// bounding generic to traits and lifetiems
// async trait fn run in multithread env using #[trait_variant::make(TraitNameSend: Send)]
// bounding trait method only to traits like TraitName::foo(): Send + Sync
// return trait from method using -> impl TraitName
// trait as method param like param: impl TraitName
// trait as struct field like pub data: F (where F: TraitName) or pub data: Box<dyn TraitName> 
// casting generic to trait like &N as &dyn TraitName or N as TraitName
// bounding trait gat to traits like <N as TraitName>::AssetInfo: Send + Sync
// bounding the return type of closure trait to traits like where F: FnOnce() -> R + Send + Sync + 'static
trait Interface: Send + Sync + 'static{}
struct Instance{}
impl Interface for Instance{}
impl Interface for (){}
type BoxedTrait = Box<dyn FnOnce() -> ()>;
struct Test<R, F: Send + Sync + 'static + Clone + Default> 
    where F: FnOnce() -> R + Send + Sync + 'static, 
        R: Send + Sync + 'static{
    pub data: F,
    pub another_data: BoxedTrait
}
fn trait_as_ret_and_param_type(param: &mut impl FnOnce() -> ()) -> impl FnOnce() -> (){ ||{} }
fn trait_as_ret_and_param_type1(param_instance: &mut impl Interface) -> impl FnOnce() -> (){ ||{} }
fn trait_as_ret_type(instance_type: Instance) -> impl Interface{ instance_type }
fn trait_as_ret_type_1(instance_type: Instance) -> impl Interface{ () }
fn trait_as_param_type(param: impl FnOnce() -> ()){

    struct Button<T: FnOnce() -> String + Send + Sync>{
        pub onclick: Box<dyn FnOnce() -> T>, // dynamic dispatch using object safe trait by boxing the trait 
    }
    let b = Button{onclick: Box::new(||{ ||{String::from("")} })};
    (b.onclick)();

    trait AuthExt{}
    #[derive(Clone)]
    struct Auth{}
    impl AuthExt for Auth{}
    impl Auth{
        fn get_trait(&self) -> &(dyn AuthExt){
            let t = self as &dyn AuthExt; // use casting
            t 
            // &Self{}
        }
        fn get_trait1(&self) -> impl AuthExt{
            Self{}
        }
        fn get_trait2(&self) -> Box<dyn AuthExt>{
            let t = Box::new(self.clone());
            t 
        }
        pub async fn ret_cls(f: impl Fn(String) -> String) -> impl Fn(String) -> String{
            let cls = |name: String|{
                String::from("")
            };
            cls
        }
    }
    let inst = Auth{};
    let get_trait = inst.get_trait();

}


// C must be send sync to be share between threads safely
impl<F: Interface + Clone, C: Send + Sync + 'static + Unpin + Sized + FnOnce() -> String> Interface for UserInfo<C, F>{}
struct UserInfo<C: Send + Sync + 'static, F: Clone> where 
    F: Interface, 
    C: FnOnce() -> String{
    data: F,
    __data: C,
    _data: Box<dyn Interface>,
}
impl<F: Interface + Clone, C: Send + Sync + 'static + Unpin + Sized + FnOnce() -> String> UserInfo<C, F>{
    fn set_data(cls: impl FnOnce() -> String, clstopass: C, f: F) -> impl Interface{
        
        struct ExecuteMe;
        struct MessageMe;
        trait ExecuteMeExt<A, B>{
            type Result;
        }
        impl ExecuteMeExt<MessageMe, String> for ExecuteMe 
            where String: Send, MessageMe: Send + Sync{
            type Result = MessageMe;
        }
        
        Self{
            data: f,
            __data: clstopass,
            _data: Box::new(
                ()
            ),
        }
    }
}

struct SizeableImage{
    size: (u16, u16)   
}
impl Into<SizeableImage> for String{
    fn into(self) -> SizeableImage { // self refers to the String cause we're implementing this for String
        let mut splitted_size = self.split(",");
        let width = splitted_size.next().unwrap();
        let height = splitted_size.next().unwrap();
        SizeableImage{
            size: (width.parse::<u16>().unwrap(), height.parse::<u16>().unwrap()) 
        }
    }
}
fn construct_image<VALUE>(size: VALUE) where VALUE: Into<SizeableImage>{}

struct ErrorHandler<E> where E: std::error::Error{
    cause: Box<dyn std::error::Error>, // any type could causes the error at runtime, Error trait is implemented for that
    err: E
}
#[derive(Debug)]
struct ErrorItself{}
impl std::error::Error for ErrorItself{}
impl std::fmt::Display for ErrorItself{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
// let err = ErrorHandler{
//     cause: Box::new(ErrorItself{}), // return the error on the heap to move it around to implement for other types
//     err: ErrorItself{}
// };


async fn ltg(){

    // C must be ?Sized since its size can't be known at compile time
    // its can be either &[] or any type
    struct Gene<'r, C: ?Sized>{ 
        pub chromosemes: &'r C,
    }

    let gene = Gene::<'_, [u8]>{
        chromosemes: &[0, 255]
    };
    
    impl std::fmt::Display for ClientError{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }
    #[derive(Debug)]
    struct ClientError{}
    impl std::error::Error for ClientError{}
    let boxed_error: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(ClientError{});
    
    // traits
    //     - method param
    //     - return type
    //     - bound to generic and generic would be the type
    //     - cast the generic into a triat then bound the trait gat to other traits 
    //     - put them in box
    type ClsMe = Box<dyn FnOnce() -> ()>;
    trait NewTrait: Clone + FnOnce() -> (){} // if we want to implement NewTrait for the Fancy all the supertraits must be implemented for Fancy
    let cls = Box::new(||{});
    let cls_ = Box::pin(async move{}); // for future we must pin them
    struct Fancy<A> where A: Copy{name: ClsMe, age: A, fut: std::pin::Pin<Box<dyn futures::Future<Output=()>>>}
    let fancy = Fancy::<u8>{name: cls, age: 23, fut: cls_};
    impl<A: Copy> Fancy<A>{
        fn get_param(param: impl FnOnce() -> ()) -> impl Clone{
            String::from("") // we can return String in here since it implements Clone
        } 
    }

    #[derive(Debug)]
    struct CustomError{data: u8}
    impl std::fmt::Display for CustomError{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }

    
    impl std::error::Error for CustomError{} // in order to return an instance of CustomError the Error trait must be implemented for it so we can return the instance inside a boxed type
    // ----------------------------------------------------
    //              boxing traits be like: 
    // ----------------------------------------------------
    // (putting them on the heap and return an smart pointer with valid lifetime to move the around safely as an object)
    let boxed_cls: Box<dyn FnOnce() -> () + Send + Sync + 'static> = Box::new(||{});
    // boxing the Error trait allows us to handle any possible runtime errors 
    // reason of putting the trait inside the box is because we don't know the 
    // exact type of the object that caused the error and by putting it inside 
    // the box we're converting it into a safe object cause traits are not sized
    // at compile time their size depends on the implementor at runtime smt that 
    // implements the trait, if we don't want to box it we can use it as the return
    // type of the method but it needs to be implemented for the exact object that
    // may causes the error at runtime and we should return an instance of that 
    // type which implements the Error trait already, with this approach there is
    // no need to put the trait inside the box cause we know the exact type of
    // object that may causes the error and the syntax be like => ... -> impl Error{}
    let boxed_err: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(CustomError{data: 0}); // the instance of the implementor must be passed  - boxing trait to move them around as an object 
    // to move the future objects around we should pin them (the mutable pointer) into the ram 
    // to prevent them from moving by the compiler at runtime sine we may want to solve them
    // in other scopes and threads hence we must have their previous location inside the ram
    // to put .await on them
    let boxed_fut: std::pin::Pin<Box<dyn futures::Future<Output=String>>> = Box::pin(async move{String::from("")}); 
    let mut pinned_boxed_fut = Box::pin(async move{}); // pinning the boxed future to move it around for future solvation
    { // now we can await on the future in other scopes
        // await on the mutable pointer of the future cause we want to await on pinned_boxed_fut in later scopes
        // we can do this cause we've pinned the boxed future (pointer to future) on the ram which allows us to 
        // move it safely between scopes and threads
        (&mut pinned_boxed_fut).await; 
    }
    pinned_boxed_fut.await; // solve the future itself


    type ActorCls = Box<dyn FnOnce(fn() -> String) -> ()>;
    type PinnedBoxedFut = std::pin::Pin<Box<dyn futures::Future<Output=String>>>; // pinning the boxed future will be used to move the future around other scopes cause they can't move safely and we must kinda convert them into an object to move them
    pub struct GenericActor<'p, ActorCls: Clone, B, F> 
        where ActorCls: Send + Sync + 'static, 
        B: FnMut() -> fn() -> (),
        F: futures::Future<Output=String>{
        pub actor_cls: ActorCls,
        pub cls: B,
        pub fut: F,
        pub pinned: PinnedBoxedFut, // we can solve this later by putting .await on pinned field
        pub db: std::pin::Pin<&'p mut HashMap<String, String>> // pinning the mutable pointer of the map into the ram to move it safely between scopes without having changes in its location by the compiler
    }
}

fn serding(){


    #[derive(Serialize, Deserialize)]
    struct NotifData{}
    let buf: Vec<u8> = vec![0, 1];
    let res1 = std::str::from_utf8(&buf).unwrap();
    let res = serde_json::to_string(&buf).unwrap();
    let data = serde_json::from_str::<NotifData>(&res).unwrap();
    let data1 = serde_json::from_str::<NotifData>(&res1).unwrap();
    
    #[derive(Serialize, Deserialize, Debug)]
    struct DataBucket{data: String, age: i32}
    let instance = DataBucket{data: String::from("wildonion"), age: 27};
    ///// encoding
    let instance_bytes = serde_json::to_vec(&instance);
    let instance_json_string = serde_json::to_string_pretty(&instance);
    let instance_str = serde_json::to_string(&instance);
    let isntance_json_value = serde_json::to_value(&instance);
    let instance_json_bytes = serde_json::to_vec_pretty(&instance);
    let instance_hex = hex::encode(&instance_bytes.as_ref().unwrap());
    ///// decoding
    let instance_from_bytes = serde_json::from_slice::<DataBucket>(&instance_bytes.as_ref().unwrap());
    let instance_from_json_string = serde_json::from_str::<DataBucket>(&instance_json_string.unwrap());
    let instance_from_str = serde_json::from_str::<DataBucket>(&instance_str.unwrap());
    let isntance_from_json_value = serde_json::from_value::<DataBucket>(isntance_json_value.unwrap());
    let instance_from_hex = hex::decode(instance_hex.clone()).unwrap();
    let instance_from_hex_vector_using_serde = serde_json::from_slice::<DataBucket>(&instance_from_hex);
    let instance_from_hex_vector_using_stdstr = std::str::from_utf8(&instance_from_hex);
    let instance_from_vector_using_stdstr = std::str::from_utf8(&instance_bytes.as_ref().unwrap());
    
    println!(">>>>>>> instance_hex {:?}", instance_hex);
    println!(">>>>>>> instance_from_bytes {:?}", instance_from_bytes.as_ref().unwrap());
    println!(">>>>>>> instance_from_json_string {:?}", instance_from_json_string.unwrap());
    println!(">>>>>>> instance_from_str {:?}", instance_from_str.unwrap());
    println!(">>>>>>> isntance_from_json_value {:?}", isntance_from_json_value.unwrap());
    println!(">>>>>>> instance_from_hex_vector_using_serde {:?}", instance_from_hex_vector_using_serde.unwrap());
    println!(">>>>>>> instance_from_vector_using_stdstr {:?}", instance_from_vector_using_stdstr.unwrap());
    println!(">>>>>>> instance_from_hex_vector_using_stdstr {:?}", instance_from_hex_vector_using_stdstr.unwrap());

}

// convert any generic of Vec into a generic slice by leaking and consuming the 
// memory of the vector to return an static reference to the leacked part since 
// that part will never be freed until the lifetime of the app
pub fn vector_slice<T>(s: Vec<T>) -> &'static [T]{
    Box::leak(s.into_boxed_slice())
}


fn dynamic_typing(){

    /* 
        we can leverage the power of trait in rust to make a dynamic type and calls
        using Box<dyn Trait> since there might be some situation that we don't know 
        the exact type of some object the only thing it's worth nothing to know is 
        that it MUST implements the Trait so we could act like an object safe trait.
        Any trait is an object safe trait allows us to be Boxed and its methods get 
        dispatched dynamically at runtime on any type that implements the Any trait.
    */
    // dynamic dispatching and typing means this type can be of type anything it only needs
    // to implement the Any trait so we can cast its instances of the type into the actual 
    // concrete type, we can use the idea of dynamic dispatching to create dynamic typing
    // a type can be of Any if it implements the Any trait then we could cast it into the 
    // concrete type since Box<dyn Trait> in Rust is like type assertion in Go used for dynamic 
    // typing and dispatching make sure type implements Any trait in order to do the casting 
    // it's like type assertion in Go with interface{} to cast the object of type interface{} 
    // to its underlying concrete type the idea beind Any is the same as type assertion and
    // dynamic dispatching process or Box<dyn Trait> which allows us to box any type that 
    // implements Any trait so we can do the casting operation later and assert that the 
    // underlying type of Any is what we're trying to casting it into the desired concrete
    // type, generally we can use traits to add interfaces to structs, for dynamic typing 
    // dispatching and polymorphism using Box<dyn Trait>, casting trait object into the type,
    // using downcast_ref() method, casting type into the trait object using &String as &dyn Trait
    // the Box<dyn Trait> basically means that putting the trait behind an smart pointer on the
    // heap and since we don't know the exact type of implementor we add a dyn keyword behind 
    // the trait it's worth noghing to say that the implementor must implements the Trait in 
    // order to call trait methods on the instance or cast the trait into our concrete type.

    use std::any::{Any, TypeId}; // TypeId is a global unique id for a type in the whole app
    let boxed: Box<dyn Any> = Box::new(3_i32); // Any trait object, i32 implements Any trait

    let mut name: Box<dyn Any>; // name can be any type that implements the Any trait, later we can cast or assert the value into the desired type
    name = Box::new(String::from("wildonion"));
    println!("string before casting and mutating it : {:?}", name);
    println!("string address before casting and mutating it : {:p}", &name);

    let name_id = (&*name).type_id();
    // assert!(name_id, "{}", TypeId::of::<String>());

    // assert that the name type can be casted into the String or not
    // it's like type assertion in Go: 
    // if e, ok := name.(*string); ok{}
    match name.downcast_mut::<String>(){ // trying to cast the trait object into a mutable String type
        Some(mutable_ref) => {
            // it's mutable pointer we can change the content of the name
            // dereferencing won't change the address it only changes the underlying data (same address but different value)
            *mutable_ref = String::from("changed wildonion");
        },
        None => {
            println!("can't cast into string");
        }
    };

    // dereferencing won't change the address of name, it's the same as before
    println!("string after casting and mutating it : {:?}", name);
    println!("string address after casting and mutating it : {:p}", &name);

    struct Player{
        pub nickname: String,
    }
    let player = Player{nickname: String::from("")};
    let mut casted: Box<dyn std::any::Any> = Box::new(player);
    match casted.downcast_mut::<Player>(){
        Some(player) => {
            (*player).nickname = String::from("wildonion");
        },
        None => {}
    };

}


fn but_the_point_is1(){
    
    let fut = async move{};
    let pinned = Box::pin(fut);
    println!("fut pinned address {:p}", pinned);
    fn getFutPinned(fut: std::pin::Pin<Box<dyn std::future::Future<Output=()>>>){
        println!("[getFutPinned] - fut pinned address {:p}", fut);    
    }
    getFutPinned(pinned);
    
    let mut name = String::from("");
    // two different address
    let pname = &name;
    let pname1 = &name;
    // let mut pmutname = &mut name;
    let pinned = std::pin::Pin::new(Box::new(name.clone()));

    println!("name address with pname: {:p}", pname);
    println!("name address with pname1: {:p}", pname1);
    println!("name address with pinned: {:p}", pinned);
    
    move_pinned(pinned);
    fn move_pinned(pinned: std::pin::Pin<Box<String>>){
        println!("[move_pinned] - name address with pinned: {:p}", pinned);    
    }
    
    fn move_name(name: String){
        println!("[move_name] - name address : {:p}", &name);
    }
    
    fn move_name1(name: &String){
        println!("[move_name1] - name address : {:p}", name);
    }
    
    // pass by type or cloning or moving: every type in method has new owner thus new address
    move_name(name.clone());
    // pass by reference: type will have the same address as it's ousdie of the method
    move_name1(&name);
    
    
    
    let mut name = String::from("");
    let mut pname = &mut name;
    change_name(&mut name, &mut String::from("new value"));

    // we should use a same lifetime for each mutable pointer
    // cause we're updating name with new_name1 which requires 
    // same valid lifetime
    fn change_name<'valid_lifetime>(
        mut name: &'valid_lifetime mut String, 
        new_name1: &'valid_lifetime mut String
    ){
        // ------ same address different value
        println!("address before mutating: {:p}", name);
        *name = String::from("wildonion");
        println!("address after mutating: {:p}", name);
        // ------
        
        // ------ new address new value binding
        println!("address before new binding: {:p}", name);
        let mut new_name = String::from("new value");
        name = new_name1;
        println!("address after new binding: {:p}", name);
        // ------
        
        println!("final value of name: {}", name);
    }


        /*
            Rust moves data around the ram and assign them new address in new location 
            Pass by ref to keep the same address in new scope otherwise new ownership 
            hence address will be generated 
        */
    
        let name = String::from("wo");
        let p = &name;
        println!("addr is {:p}", p);
        
        // pass by ref avoid moving same address will be inside the method
        fn get_name(name: &String){
            println!("name in here {:p}", name);
        }
        
        get_name(p);
        // any pointer in here would be invalidated 
        // ...
        
        
        fn getName(name: String){
            println!("new name in here {:p}", &name);
        }
        
        getName(name.clone());

        type UserId = char;
        let userIds: Vec<UserId> = vec![];
        let mut user_ids = vec![];
        user_ids = userIds.into_iter()
            .map(|uid| uid.len_utf8())
            .collect::<_>();


    // ---------------- MUTATING NAME WITH PINNING
    // mutating with pinned pointer 
    let mut name = String::from("");
    // pinned is an smart pointer of the pinned mutable pointer of the name 
    // into the ram so mutating it mutate the underlying data as well.
    let mut pinned = std::pin::Pin::new(&mut name);
    (*pinned) = String::from("wildonion");
    println!("pinned value : {:?}", pinned);
    // noramlly we can't print the address of pinned like the following
    // and we must use &pinned with {:p} but since the pinned contains the 
    // mutable pointer of the name this is ok! cause smart poitners are wrapper
    // around the actual data and they keep the actual data features.
    println!("pinned address : {:p}", pinned); 
    println!("name address : {:p}", &name);
    println!("name value: {:?}", name);
    // ----------------
    
}

fn but_the_point_is2(){

    // can't have immutable and mutable ref at the same time
    let mut init = String::from("wildonion");
    let mut init2 = String::from("wildonion");
    let pname = &init;
    let pname1 = &init;
    
    let mut mutpname = &mut init2; 
    println!("address of mutpname: {:p}", &mutpname);
    println!("mutpname points to init2: {:p}", mutpname);

    // same address but different value
    *mutpname = String::from("new wildonion");
    println!("value of init2: {}", init2);
    
    // new address and new value
    let mut new_binding = String::from("new wildonion1");
    mutpname = &mut new_binding;
    println!("address of mutpname: {:p}", &mutpname);
    println!("mutpname now points to new binding location: {:p}", mutpname);
    println!("value of mutpname: {}", mutpname);

    // same val different address
    // address of pname: 0x7ffd7875b570
    // address of pname1: 0x7ffd7875b578
    println!("address of pname: {:p}", &pname);
    println!("address of pname1: {:p}", &pname1);
    
    // both are pointing to the name
    // pname points to: 0x7ffd7875b558
    // pname1 points to: 0x7ffd7875b558
    println!("pname points to : {:p}", pname);
    println!("pname1 points to : {:p}", pname1);

}

fn but_the_point_is(){


    // rust often moves heap data around the ram for better allocation and 
    // optimisation like when a vector is growing at runtime it moves it in 
    // other locations and update all its pointers behind the scene.
    let mut vector = vec![1, 3, 5];
    
    println!("vec address: {:p}", &vector);
    
    // moving vector into a new scope with new ownership 
    // cause everthing must have only one owner
    fn get_vector_ownership(vec: Vec<u8>){
        
        // new address cause data will be moved into the function scopes after passing it
        println!("vec address in func: {:p}", &vec);
    }

    fn borrow_vector_ownership(vec: &Vec<u8>){
        
        // contains the same address of the outside vector
        // cause we've passed the vector by its pointer
        // and its point to the same location of the vector
        println!("vec address in func: {:p}", vec);
    }
    
    fn borrow_vector_ownership_mutable(vec: &mut Vec<u8>){
        
        vec.push(100);
        
        // contains the same address of the outside vector
        // cause we've passed the vector by its pointer
        // and its point to the same location of the vector
        println!("vec address in func: {:p}", vec);
    }
    
    // get_vector_ownership(vector);
    // borrow_vector_ownership(&vector);
    borrow_vector_ownership_mutable(&mut vector);

    // ********------********------********------********------********------
    // ********------********------********------********------********------
    // -> updating pointer with new binding changes both the address and value
    // -> dereferencing the pointer mutates the underlying value but keeps the address
    /* -> in Go: 
        u := User{Name: "onion"}
        p := &u
        println("p", p)

        // changing p completely with new address and value, this won't change the u
        p = &User{Name: "changed"} // changing address and value, breaks the pointer points to u

        // since p has been changed with new binding, dereferencing it will change 
        // the newly p value and it has nothing to do with u
        println("p", p)
        *p = User{Name: "wildonion"} // changing value keeps the address

        println("p", p)

        // this remains the same
        println("u", u.Name)
    */

    let mut me = String::from("onion");
    let mut mutme = &mut me;
    println!("me address : {:p}", mutme);
    
    // changing the address and the underlying value completely 
    // this logic break the pointer to the `me` and it replaces 
    // with a new address and value so after this the `me` value
    // won't change
    let mut new_me = String::from("changed");
    mutme = &mut new_me; // this won't change the `me`, breaks the pointer points to `me`
    println!("me address : {:p}", mutme);
    // -----> name is NOT changed after changing the mutme completely with a new binding
    
    // changing the underlying value only but NOT the address
    // this logic keeps the pointer pointing to the me location
    // and only mutate its underlying data
    *mutme = String::from("another changed");
    println!("me address : {:p}", mutme);
    // -----> name is changed after dereferencing the mutme

    // since `mutme` has been mutated with a new binding and address thus there is no
    // pointer points to the `me` location and dereferencing `mutme` (*mutme) will change 
    // the content inside the `mutme` only, which has been changed with "changed" and 
    // now has "another changed" value, this has nothing to do with `me` cause the `mutme` 
    // is not pointing to the `me` any more after getting a new location of new binding, 
    // the pointer breaks the pointing once a new binding put into it, if we didn't update 
    // the `mutme` with a new binding, after dereferencing it, the `me` would have upaded too.
    // ...  

    println!("me : {:?}", me); // onion as before
    println!("mutme : {:?}", mutme); // another changed
    // ********------********------********------********------********------
    // ********------********------********------********------********------

    type Ret = &'static str;
    fn add(num1: Ret, num2: Ret) -> Ret where Ret: Send{
        for ch in num2.chars(){
            num1.to_string().push(ch);
        }
        let static_str = Box::leak(num1.to_string().into_boxed_str()) ;
        static_str
    }

    let addfunc: fn(&'static str, &'static str) -> &'static str = add;
    let res = addfunc("wild", "onion");

    // ------------------------------------------------------------------------
    // ------------------------------------------------------------------------
    // let name1: String; // this must be initialized
    // let pname = &name1; // ERROR: in Rust we don't have null or zero pointer it must points to some where in memory
    
    let mut name = String::from("wildonion"); // the pointee
    println!("name address : {:p}", &name);
    
    // NOTE: both pname and p0name have different addresses but pointes 
    // to the same underlygin address
    let pname = &name; // points to the location of name contains the name value
    let mut p0name = &name; // points to the location of name contains the name value
   
    println!("[SAME ADDR] pname pointer points to : {:p}", pname);
    println!("[SAME ADDR] p0name pointer points to : {:p}", p0name);
   
    // same address and same underlying data, cause pname points to the name address too
    p0name = pname;
    println!("[SAME ADDR] p0name pointer points to : {:p}", p0name);
    
    // however this is not safe in Rust to do this inside a function to change the
    // address a pointer points to but keeps the same underlying data cause literally 
    // chaning the address a pointer points to means that the underlying data must be 
    // changed cause pointers point to the address of a data and have their value.
    // if we want to do this we should use mutable pointer and rechange the pointer 
    // without dereferencing it
    let new_binding = &String::from("new onion"); 
    p0name = new_binding; // the actual name won't be changed cause this is not a mutable poitner
    println!("[CHANGED ADDR] p0name pointer points to : {:p}", p0name);
    
    let mut mutpname = &mut name; // points to the location of name contains the name value
    // changing underlying data same adderess by dereferencing 
    println!("mutpname pointer points to : {:p}", mutpname);
    *mutpname = String::from("wildonion"); // dereferencing it only update the undelrying data not the address
    println!("mutpname pointer points to : {:p}", mutpname); // mutpname still contains the address of the very first name variable but the value has changed
    
    // changing both address and underlying data by binding a new value
    let mut new_binding = String::from("onion"); 
    mutpname = &mut new_binding;
    println!("[CHANGED ADDR] mutpname pointer points to : {:p}", mutpname); // mutpname now contains completely a new value binding accordingly new location of the new binding
    // -----> name is NOT changed after changing the mutpname completely with a new binding
    // ....
    // pointers are immuatable by default 
    // {:p} requires the type implements AsRef trait, pointers implement this
    let name = String::from("wildonion");
    let pname = &name;
    println!("name address                       : {:p}", &name);
    println!("name address (pname)               : {:p}", pname);
    println!("address of pointer itself          : {:p}", &pname);
    println!("pname points to name address       : {:p}", pname);
    
    println!("------------------------------------");
    println!("--------same val | new addr---------");
    println!("------------------------------------");
    let mut pname1 = &String::from("oniontori");
    println!("pname1 points to new address       : {:p}", pname1);
    pname1 = pname;
    println!("pname1 now points to name address  : {:p}", pname1);
    println!("pname1 vale                        : {:?}", pname1);
    println!("pname1 address itself              : {:p}", &pname1);
    let new_binding = &String::from("layer1");
    pname1 = new_binding;
    println!("pname1 now points to a new address : {:p}", pname1);
    
    println!("------------------------------------");
    println!("--------same addr | new val---------");
    println!("--------new addr  | new val---------");
    println!("------------------------------------");
    let mut new_name = String::from("erfan");
    println!("new_name address                                 : {:p}", &new_name);
    println!("new_name value: {:?}", new_name);
    let mut pnamemut = &mut new_name;
    println!("pnamemut points to new_name address              : {:p}", pnamemut);
    println!(">>> dereferencing mutable pointer...");
    *pnamemut = String::from("completely new name");
    println!("new_name new value                               : {:?}", new_name); // also pnamemut has changed too
    let mut new_binding = String::from("new val");
    pnamemut = &mut new_binding;
    println!("pnamemut new value is                            : {:?}", pnamemut);
    println!("pnamemut now points to new address of new binding: {:p}", pnamemut);
    // pnamemut is no longer pointing to new_name
    // ...
    // ------------------------------------------------------------------------
    // ------------------------------------------------------------------------

    #[derive(Default, Debug, Clone)]
    struct User{
        name: String,
        age: u8,
    }

    let mut user = User::default(); // there is no null or zero pointer in rust thus the user must be initialized

    let mut mutpuser = &mut user; // mutating mutpuser mutates the user too
    println!("user address: {:p}", mutpuser); // contains user address
    println!("mutpuser address itself: {:p}", &mutpuser); // contains user address
    mut_user(mutpuser);

    fn mut_user(mut user: &mut User){ // passing by mutable pointer or ref to avoid moving

        // mutating the user pointer with new value which contains the user address
        // this makes an update to user instance too, can be viewable outside of the method
        println!("before mutating with pointer: {:#?}", user);
        user.name = "erfan".to_string(); // no need to dereference it since we're mutating only one field
        println!("after mutating with pointer: {:#?}", user);
        // or
        println!("before derefing: {:p}", user); // same as `contains user address`
        let mut binding = User{
            name: String::from("wildonion"),
            age: 0
        };
        // updating pointer which has the user instance value with a new binding by dereferencing pointer
        // note that we're not binding the new instance into the pointer completely cause by dereferencing
        // the underlying data will be changed
        *user = binding; // dereferencing the pointer to mutate it with new binding 
        println!("user after derefing: {:#?}", user);
        println!("user address after derefing: {:p}", user); // same as `contains user address`

    }

    // println!("out after mutating with pointer: {:#?}", user);
    let mut binding = User{
        name: String::from("wildonion"),
        age: 0
    };
    println!("mutpuser address itself: {:p}", &mutpuser); // contains user address
    println!("mutpuser contains address before binding: {:p}", mutpuser); // same as `contains user address`
    // binding a complete new instance to mutpuser, causes to point to new location
    mutpuser = &mut binding;
    // the address of mutpuser got changed and now points to new binding instance address
    println!("mutpuser contains address after binding: {:p}", mutpuser);
    println!("mutpuser address itself: {:p}", &mutpuser);

    // we're getting a mutable pointer to an in place User instance
    // the in place instance however will be dropped after initialization
    // and its ownership transferred into mutpuser, Rust won't allow us to
    // do so cause a pointer remains after dropping the in place instance
    // which is and invalid pointer, we must use a binding to create a longer
    // lifetime of the User instance then borrow it mutably
    // mutpuser = &mut User{
    //     name: String::from(""),
    //     age: 0
    // }; // ERROR: temporary value is freed at the end of this statement

    // SOLUTION: using a `let` binding to create a longer lived value
    // let binding = User{
    //     name: String::from("wildonion"),
    //     age: 0
    // };
    // *mutpuser = binding;


    // let puser = &user;
    // println!("user address (puser): {:p} ", puser); // contains the address of user
    // let anotherpuser = puser;

    // println!("user address (anotherpointer): {:p} ", anotherpuser); // also contains the address of user

    // println!("pointer address: {:p} ", &puser); // the address of the puser pointer itself
    // println!("anotherpointer address: {:p} ", &anotherpuser); // the address of the puser pointer itself

    // user address (puser): 0x7ffea5896328
    // user address (anotherpointer): 0x7ffea5896328
    // pointer address: 0x7ffea5896348
    // anotherpointer address: 0x7ffea5896390


    let users = (0..10)
        .into_iter()
        .map(|_|{
            User::default()
        })
        .collect::<Vec<User>>();
    let slice_is_faster = &users;
    fn get_users(users: &[User]) -> (&'static [User], Vec<User>){
        // lifetime of users ends up here in this function 
        // and can't be as static accordingly can't be return 
        // from function
        let users_vec = users.to_vec();
        let static_users = vector_slice(users_vec.clone());
        (static_users, users_vec)
    }

    trait Interface{
        type This;
        fn getName(&mut self) -> &Self;
    }
    #[derive(Debug, Default, Clone)]
    struct UserPl{
        Name: String,
        Age: u8,
        IsAdmin: bool,
    }
    impl Interface for UserPl{ // unlike Go Interface in Rust will be implemented for both pointer and none pointer instances
        type This = Self;
        // trait and interface methods
        fn getName(&mut self) -> &Self { // we can return ref since the pointer is valid as long as instance is valid
            if self.Name == "oniontori"{
                self.Name = String::from("wildonion");
            }
            self
        }
    }
    trait ObjectSafeTrait{}
    impl ObjectSafeTrait for (){}
    let mut user = UserPl{Name: String::from("oniontori"), Age: 28, IsAdmin: true};
    let trait_object: Box<dyn ObjectSafeTrait> = Box::new(());
    let mut mutpuser = &mut user;
    mutpuser.getName(); // mutating the Name field of the user instance using the Interface trait and its mutable pointer
    // println!("user is changed {:?}", user); // the user is changed using its mutable pointer
    mutpuser.Name = String::from("change to anything else again");
    println!("user is changed {:?}", user);
    // println!("mutpuser is changed {:?}", mutpuser); // the mutpuser is changed also

    type LargeUInt = u128;
    type Func<R, A = UserPl> = fn(A) -> R; // A has a default type set to UserPl
    let cls = |num: LargeUInt|{
        String::from("")
    };
    // `impl Trait` only allowed in function and inherent method argument 
    // and return types, not in variable bindings
    // let closure: impl Fn(u16) -> String = cls;

    #[derive(Default, Debug)]
    struct Player<'v, G: Default + Send + Sync + 'static, F> 
        where F: FnMut(Box<Player<G, F>>) 
            -> Result<
                std::pin::Pin<Box<dyn futures::Future<Output=&'v [u8]>>>, 
                Box<dyn std::error::Error + Send + Sync + 'static>> + Default{
        board: Vec<&'v [G]>,
        cls: F
    }

    trait UserExt<G, F>: Default{
        type Context;
        fn getCode() -> String;
    }
    impl<'valid, G: Default + Send + Sync + 'static, 
        F: FnMut(Box<Player<G, F>>) 
        -> Result<
            std::pin::Pin<Box<dyn futures::Future<Output=&'valid [u8]>>>, 
            Box<dyn std::error::Error + Send + Sync + 'static>> + Default
            > UserExt<G, F> for Player<'valid, G, F>{
        type Context = Player<'valid, G, F>;
        fn getCode() -> String {
            String::from("return value")
        }
    }
}

pub fn fillSchemaParser(){
    use std::collections::{BTreeMap, HashMap};
    use indexmap::IndexMap;

    // serde_json uses BTreeMap to decode the json string, in order the following 
    // logic works we should maintain the order of the keys in the order they are 
    // inserted into the map of the json string while we're decoding it. we should
    // enable the preserve_order feature in the serde_json crate. to keep the order
    // of the keys as exactly they're appearing in the json string
    let schema = String::from("server,2,1,3,4#");
    let json_str = r#"{"setip": "85.9.107.203", "setcode": "0", "setport": "7013", "setcode1": "0"}"#;
    let actions: serde_json::Value = serde_json::from_str(json_str).unwrap();

    // now serde json preserve the order of keys but BTreeMap doesn't due to building the
    // map based on the hash of each key, we're using IndexMap which maintain the state 
    // of the keys as they're inserted into the json string.
    let map = serde_json::from_value::<HashMap<String, String>>(actions).unwrap();
    let values = map.values()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();

    let mut splitted_chars: Vec<String> = schema.split_terminator(&[',', ':', '|', '.'][..])
        .map(|c| c.to_string())
        .collect();
    let mut spec_chars: std::collections::HashMap<usize, String> = std::collections::HashMap::new();

    for i in 0..splitted_chars.len(){
        if splitted_chars[i].contains('#') || splitted_chars[i].contains('*'){
            spec_chars.insert(i, splitted_chars[i].clone());
        }
        let index = splitted_chars[i].trim_matches(|c| c == '#' || c == '*').parse::<usize>().unwrap_or(0); // don't use "" to char, char is inside '' 
        if index > 0 && index <= values.len(){ // make sure that the index isn't out of bound 
            splitted_chars[i] = values[index - 1].clone();
        }
    }

    for (key, val) in spec_chars{
        // inject the val at position key into the splitted_chars vector
        splitted_chars.insert(key, val);
    }

    // keeps the # or *, use hte spec_chars map to insert them
    println!("updated schema => {:?}", splitted_chars);

}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
struct Raptor{

}

pub mod codec{

    pub async fn encoder(){
    
    }
    
    pub async fn decoder(){
        
    }
    
}


struct Pointer<'valid, T>{
    pub data: &'valid mut T
}

impl<T: Default + Clone> Pointer<'_, T>{

    /* 
        we can ret a mutable pointer in here cause we're using 
        the lifetime of the self which is valid as long as the 
        instance is valid
    */
    pub async fn register_new_pointer(&mut self) -> &mut T{
    
        self.data

    }

}

struct Struct<'valid, G>{
    pub data: &'valid G
}

impl<'g, G: Clone + Default + Send + Sync + 'static> Event for Struct<'g, G>{
    
    type Room<'valid> = std::sync::Arc<tokio::sync::Mutex<G>>;

    fn get_room<'valid>(&mut self, cls: impl FnOnce(String) -> String) -> Self::Room<'valid> {

        // no two closures, even if identical, have the same type since 
        // they're traits and traits are not sized
        // consider boxing your closure and/or using it as a trait object
        // trait object is an struct instance that implements the trait 
        // and we canll trait method on it, also boxing makes the type
        // sizable cause box stores on the heap, in our case they must be
        // as trait object cause they can be sized since we're calling them
        // on an struct instance
        // let c1 = Box::new(||{});
        // let mut closure = &mut c1;
        // closure = &mut Box::new(||{}); // it can't be mutated cause

        fn get_name() -> String{ String::from("") }
        let callback = |func: fn() -> String|{
            func();
        };
        callback(get_name);

        fn get_cls0<F, R>(cls: F) -> F where 
            F: FnOnce() -> R + Send + Sync + 'static, // FnOnce() -> R is the whole closure trait which is bounded to other traits and lifetime
            R: Send + Sync + 'static{
                cls
            }

        type Closure = Box<dyn FnOnce() -> ()>;
        fn get_cls(cls: Closure) -> (){ () }
        fn get_cls1(cls: impl FnOnce() -> ()) -> (){ () }
        // or
        (   
            // a closure that takes a closure
            |param: Box<dyn FnOnce() -> ()>|{
                ()
            }
        )(Box::new(||{}));
        
        let d = self.data.clone();
        std::sync::Arc::new(
            tokio::sync::Mutex::new(
                d 
            )
        )
    }

}

trait Event{
    
    type Room<'valid>: 
    'valid + ?Sized + Default + Clone + 
    Send + Sync + 'static; // we can bound the Room GAT to traits in here

    fn get_room<'g>(&mut self, cls: impl FnOnce(String) -> String) -> Self::Room<'g>;

}

// let mut struct_instance = Struct::<String>{
//   data: &String::from("")
// };
// let thetype = struct_instance.get_room();





use crate::*;



/* 
    ************************************************************************************************
    ************************** little warmup with ownership and borrowing **************************
    ************************************************************************************************
    → share and transfer ownership of type instead of moving and cloning, break the cycle of self-ref types and reference counting using Rc, Arc, RefCell, Mutex, Box, Pin heap based smart pointers and wrappers
    → references (&, rc, arc, box) allow borrowing values without transferring ownership, enabling multiple parts of the code to access the same value without creating copies or new ownership just by accessing the reference of the type which is also a faster approach
    → don't move or get the owned type if the type is behind a pointer which that pointer is being used by other scopes 
    → if the type moves into new ownership its pointers get updated by rust but not able to use them after moving 
    → can't return pointer from method if the type is owned by the method, only a pointer to the passed in param with valid lifetime
    → return pointer to a data owned by the method is only possible if the lifetime of the pointers is static or belongs to the self
    → self is valid as long as the object is valid so returning pointer with lifetime of self (&self) is valid
    → every type has a lifetime and once the scope of the type gets ended the lifetime comes to end 
    → moving a pointer into a scope longer then the owner scope is not valid cause once the scope of the owner gets dropped the pointer gets invalidated 
      like moving a pointer into a tokio spawn scope inside a function since the function gets executed all its local variables will be dropped thus any
      pointers of them will get invalidated
    → move if we don't need it in later scopes otherwise clone or pass by ref
    → can't move pointer of into a new scope longer than the pointee cause if the scope of the pointee gets ended its pointer gets invalidated
    → a pointer can be valid and none dangled if its underlying type didn't got moved and dropped out of the ram and its scope is still valid
    → a pointer can be moved into a new scope if its underlying type has an scope longer than the one we want to move its pointer into
    - lifetime of a pointer depends on its underlying data lifetime and should valid as long as the actual data has not dropped out yet
    - futures are self-ref object safe traits and must be appear as a pinned version of their Box but other traits can be Box<dyn

    --------------------------------------------------------------------
    ------------------- Ownership an Borrowing Recaps ------------------
    --------------------------------------------------------------------
    NOTE1) share ownership using pointers: Box, Arc, Rc, &mut 
    NOTE2) get the owned (derefing) data using * move it by passing (ownership and borrowing rules) prevent from moving with clone()
    NOTE3) can't move if the type is behind a pointer or its ownership is shared cause may the pointer is being used by other scopes
    NOTE4) borrow                      : same address same value
           clone                       : different address samve value (new ownership)
           deref mutable pointer       : different address and different value
           update mutable pointer field: same address different value
    https://github.com/wildonion/cs-concepts?tab=readme-ov-file#-wikis
    https://github.com/wildonion/stem/wiki/Ownership-and-Borrowing-Rules
    https://github.com/wildonion/rusty/blob/main/src/retbyref.rs#L17
    https://github.com/wildonion/rusty/blob/main/src/llu.rs
    https://github.com/wildonion/rusty/blob/a42b11dc96b40b059c60efa07513cdf4b93c5fab/src/ltg2.rs#L10
    https://github.com/wildonion/rusty/blob/a42b11dc96b40b059c60efa07513cdf4b93c5fab/src/ltg3.rs#L8
    https://github.com/wildonion/rusty/blob/a42b11dc96b40b059c60efa07513cdf4b93c5fab/src/ltg6.rs#L6
    https://www.reddit.com/r/rust/comments/dymc8f/selfreference_struct_how_to/
    https://arunanshub.hashnode.dev/self-referential-structs-in-rust#heading-pinlesstgreater-the-objects
    https://github.com/wildonion/rusty/blob/a42b11dc96b40b059c60efa07513cdf4b93c5fab/src/main.rs#L5
    https://stackoverflow.com/questions/72562870/how-can-i-write-a-self-referential-rust-struct-with-arc-and-bufreader


    In Rust, the behavior of dropping values and updating pointers after moving a value is governed by 
    the ownership and borrowing rules, which are enforced at compile time to prevent issues such as memory 
    leaks and dangling pointers, also due to the fact that rust doesn't have gc concepts, every type in 
    rust has its own lifetime and once it goes out of scope like moving it into other scopes and threads 
    followings happen:
        0) lifetime belongs to pointers in overall and it means that we're borrowing a type that must be
            valid as long as 'a lifetime is valid or we're borrowing the type that must be valid for 'a
        1) first note that if there is a pointer of a type it's better not to move the type at all
            instead pass its reference or its clone to methods and other scopes otherwise rust says
            something like "borrowed value does not live long enough" means that we have a reference 
            to nonexistent object cause the object gets moved, this situation is like returning an 
            instance of an struct from its method but at the same time we're using its reference, some
            how we must tell the rust that our our move will keep the reference so please don't drop it
            when the value is being returned
        2) in Rust, when a value goes out of scope, it is dropped. This happens at compile time, and the Rust 
            compiler inserts the necessary code to ensure that the value is dropped when it is no longer needed, 
            this is a deterministic process that occurs when the variable goes out of scope, and it is not tied 
            to runtime events
        3) we can't return a pointer from a method since the actual type is owned by the method and once
            the method gets executed it goes out of scope and its lifetime or the type itself gets dropped 
            from the ram so to avoid making dangling pointer rust doesn't allow to return it in the first 
            place unless we use 'static or a valid lifetime (borrow the type for the lifetime of 'valid) or 
            use the lifetime of self cause self is valid as long as the instance is valid
        4) cases that a type might be moved are passing a heap data type to a method or new scope, growing
            a vector at runtime or returning a type from a method in all these cases the actual type goes
            out of scope and gets dropped from the ram after moving and its value will go into another 
            type where the method is called or the type is being passed into a new scope thus its location 
            gets changed during execution
        5) once the type gets passed to the new scope, the newly scope takes the ownership of the type 
            and create a new location in the ram to put the moved value in the new location, so in rust 
            the location of values get changed during compilation process if they get passed into scopes
            and the old ownership gets dropped from the ram completely
        6) basically by moving data means the data string of the name variable will be moved into a new location 
            inside the heap cause its ownership has been changed and belongs to a new name variabel inside the method
            but the location inside the stack of the very first name variable won't be changed, if we don't want that
            happen we can pass the data string by reference into the method or clone, passing by reference doesn't 
            create a new location and take the ownership it just passing the data itself but clonning makes a new 
            type and put the value inside of it it's like having two different variables with same data
        7) when a value is moved, Rust updates the pointers and references to that value to ensure that they 
            are not left pointing to invalid or dangling memory. This is a key aspect of Rust's memory safety 
            guarantees, after a value is moved, any attempts to use pointers or references to the moved value 
            will result in a compilation error, preventing the creation of dangling pointers
        8) once the value of a type goes out of scope and took new ownership the lifetime of the old one 
            is no longer valid and gets dropped completely from the ram and can't be used after moving also 
            the value has new ownership which has 
            new addresss location inside the ram later rust updates all the pointers of the old ownership
            with this new address so they point to the right and newly location to avoid getting dangled
        9) if we move a type into a new scope regardless of the type is behind a pointer, rust updates
            its pointer to points to the right location after moving however the pointer is no longer
            accessible at runtime cause the type gets moved, the updating process can be taken place for 
            those types that are safe to be moved, which are almost all types except those ones that doesn't 
            implement Unpin, those are not safe to be moved and must be pin into the ram to avoid moving 
            completely, types like future objects and sef-referential types are not safe to be moved cause 
            as soon as move happens the pointer to them gets broken and invalidated hence rust won't update 
            the pointer of them to points to the right location after moving which doesn't allow to move 
            them in the first place 
        10) we can either pin the object into the stack or heap, Arc and Rc and Box puts the type into 
            heap but Box has a pin method which is simple to pin the Boxed value which is on the heap
            already into the ram in other cases we should pin the Arced or Rced into the ram
        11) rust can't update the pointer of self-ref types we can use Pin or Arc, Rc to break the cycle
            Pin takes the pointer of the type to pin its value into the ram avoid moving at all costs then
            we can use the pin instead of the type, we pass mutable pointer of the type to the Pin since 
            &mut _ access the underlying data of the object and pinning &mut _ is like pinning the direct
            value of the object (we can mutate a data by its &mut _)
        12) rust pointers are safe cause after moving a type (if it's implement Unpin means it's safe to be moved)
            the compiler updates the location of the pointer to point to the right location of the newly address of
            the moved value cause the ownership of the value has changed and all its pointers must gets updated to 
            point to the new location, this is not true about the raw pointers and rust won't update the location of 
            raw pointers to the new one when two value gets swapped or moved into another scope, they still point to
            the old value even after swapping, in rust we should use pin when the pointer of a type can't be updated
            by the rust compiler after it gets moved pinning allows us to pin the pointer of the type into the ram 
            and explicitly prevents the value from being moved, so the references to the value remain valid without 
            the risk of the value being relocated in memory by the rust compiler generally in cases such as self-refrential 
            types to break the cycle, future objects for later solvation and raw pointers, so we can pin the type 
            into the ram and don't allow to be moved at all and in the first place therefore by pinning a value using 
            Pin, you are essentially telling the Rust compiler that the value should not be moved in memory, and the 
            compiler enforces this constraint, this is particularly useful when dealing with self-referential structures 
            or when you need to ensure that references to the value remain valid by not allowing the type to be moved
        13) pin uses cases: handling raw pointers, self-refrential types and future objects 
            raw pointer swapping won't change the pointers pointees or pointer values it only swaps the contents
            which is not good since the first type pointer points to a location now which contains the content 
            of the second type after swapping and vice versa in other words rust won't update each pointer value
            based on the swapped values and there would be the same as before which causes to have undefined behaviours
            and dangling issues as rust don't care about updating the location of each pointer to point to the right
            location after moving to fix this we can pin each instance to tell rust make those objects immovable cause
            we don't want to invalidate any pointer of them, we're avoiding this by pinning each instance value using
            their pointer into the ram (usually heap using Box::pin()) so they can't be able to be moved cause by
            moving rust needs to update pointers to point the right location after moving but this is not true 
            about these none safe types and by swapping them two values along with their pointer are swapped
        conclusion: 
            types that are not safe to be moved (don't impl Unpin or are !Unpin) like self-refrential structs, 
            future objects, raw pointers are the types that unlike normal types rust compiler won't update their 
            pointer to point to the right location inside the memory (new address) after they get moved into other 
            scopes it's because of they kinda have infinite size at compile time or don't have specific size at 
            all so they invalidate their own references and break the moves, in order to fix this we should pin 
            their value into the ram (stack using std::pin::Pin or heap using Box::pin()) by passing their pointer 
            to the pin() method to tell the rust that don't move their values at all so their pointers can be valid
            across the scopes and threads but note that we can move the type after its value it gets pinned to the
            ram cause the use of Box::pin and Pin ensures that the self-referential pointers are correctly managed, 
            allowing the values to be safely moved and swapped without invalidating the references, means Box::pin, 
            it creates a pinned reference, ensuring that the data the reference points to will not be moved in memory, 
            preventing it from being invalidated:
            
            `let pinned = Box::pin(&name);` creates a pinned reference to the name string, 
            making sure that it won't be moved in memory, however, when we call `get_name(name)`, 
            we are actually moving the ownership of the name string into the get_name function, 
            which is allowed because the name variable is not used after that point, therefore,
            although pinned prevents the reference from being invalidated, it doesn't prevent the 
            owned value from being moved, later on we should use the pinned type instead of the 
            actual type cause the pinned type has a fixed memory location for the value thus has 
            a valid pointer which won't get dangled at all cause the value can't be moved by the 
            compiler at all even if rust wants to move them it can't since we're telling rust hey 
            u back off this is pinned! but its location and the address inside the ram will be the 
            same in all scopes, this is because the Pin type ensures that the references remain valid 
            even after the values are moved, in summary, pinning a value using Pin in Rust ensures 
            that the value is safe to use and reference, even if the value is moved by the compiler, 
            because the pointer to the value is pinned into a fixed memory location already

            let name = String::from("");
            let pinned = Box::pin(&name);
            let pname = &name;
            fn get_name(name: String){}
            get_name(name);

    _________________________________________________
    _> code snippet for ownership and borrowing rules
    -------------------------------------------------
    let name = String::from("");
    let p1name = &name;
    fn get_name(name: String){}
    get_name(name);
    let pname = &name;

    after the call to get_name(name), the ownership of the String data is moved into the get_name method, 
    and the name variable is no longer valid. The pname pointer cannot be created after the move because 
    the original value has been invalidated. The behavior you described is accurate: the pointer p1name 
    gets updated after the move, but no new pointers can be created to the moved value. This is a deliberate 
    design choice in Rust to prevent the creation of dangling pointers and ensure memory safety.

    rust moves types specially heap data ones around the ram by passing them
    into a function call or other scopes (unless we pass them by reference or
    clone them) to make the ram clean by removing extra spaces hence the value of 
    those types takes palce in a new location inside the ram (heap), compiler 
    it then updates their pointers to point to the right location (new one) 
    to avoid dangling issues, almost every type is safe to be moved like heap 
    data ones, but self-referential and future objects are not safe to be moved 
    cause rust won't update their pointer to point to the right location after 
    they get moved, as the result, they must be pinned to the ram to avoid moving 
    them at all due to the facts that if there is any pointer of these type exist 
    it won't get updated by the compiler to point to the right location after 
    moving, solution to this would be either pin the value of those types like 
    pinning their mutable pointer to avoid moving completely or put them inside 
    Arc,Rc,Mutex or RefCell to break the cycle of pointing to their instance, this 
    one is mostly used to store an instance of a structure as the field of the 
    struct itself like: 
    struct Struct{ pub data: Arc<Struct> } or struct Struct{ pub data: Rc<Struct> }

    in Rust, ownership is a key feature that ensures memory safety and prevents issues 
    like memory leaks and data races. The ownership system revolves around three rules:
        1 - Each value in Rust has a variable that's called its owner.
        2 - There can only be one owner at a time.
        3 - When the owner goes out of scope, the value is dropped.
    this system allows Rust to manage memory efficiently and avoid common pitfalls associated 
    with manual memory management.

    can't move pointer inside a method to tokio spawn or return it from the method unless we make it 
    static or use the lifetime of self, cause the pointer is owned by the method
    data are moved by default when they gonna go into another scope, we can take a reference to them 
    and pass the reference but not the data itself cause it's behind a pointer already and data behind 
    pointers can be moved, or we can clone them to prevent their ownership from moving.
    compiler moves data around the ram at runtime and change their location inside the stack like when 
    an element gets poped out of a vector rust clean the memory of the vector and shift each element's 
    location to where the empty space is located so there would be no extra space after, that's why 
    their pointers might get dangled if the type doesn't implement the Unpin trait, those types that 
    implements Unpin are safe to move around the ram by compiler cause the compiler takes care of their 
    pointers automatically so at runtime the pointer points to the right location of the type inside 
    the stack and if the type doesen't impelement the Unpin means it's not safe to be mvoed by the 
    compiler, to move it around other scopes safely we should pin the mutable pointer of the type into 
    the stack to tell rust that you shouldn't move this at all cause we will use its location in other 
    scopes later on, like pinning a future trait object for future solvation or await on its mutable
    pointer, take note of that once the lifetime of the type goes out of scope type will be dropped out 
    of the ram and removed completely, so the recaps are:
        - can't move the type around if it's behind a pointer, use the pointer instead
        - Rust compiler often moves values (heap data) around in memory, for example, if we pass an struct into 
            another function, it might get moved to a different memory address, or we might Box it and 
            put it on the heap or if the struct was in a Vec<MyStruct>, and we pushed more values in, 
            the Vec might outgrow its capacity and need to move its elements into a new, larger buffer.
        - When a value is moved or dropped Rust updates the references to that data to ensure that no 
            dangling pointers are created, this is achieved through the ownership and borrowing rules, 
            which are enforced at compile time.
        - Here are some scenarios in which values may be moved in memory by the rust compiler itself, 
            this is a fundamental aspect of Rust's ownership and borrowing system, and it is designed 
            to ensure memory safety and prevent issues such as data races and dangling pointers:
                0 - heap data types move by default to avoid allocating extra spaces in the ram
                1 - returning a value from a method: by returning the value from method the owner gets dropped out of the ram and is no longer accessible, the value however goes into a new location and gets a new ownership where the method return type is being stored
                2 - Passing a value to a function: When a value is passed to a function, it may be moved to a different memory address if the function takes ownership of the value.
                3 - Boxing a value and putting it on the heap: When a value is boxed using Box::new, it is moved to the heap, and the pointer to the boxed value is stored on the stack.
                4 - Growing a Vec beyond its capacity: When a Vec outgrows its capacity and needs to reallocate memory, the elements may be moved to a new, larger buffer in memory.
                5 - In each of these cases, the Rust compiler ensures that the ownership and borrowing rules are followed, and it updates references and pointers to the moved values to maintain memory safety.
    
    _________________________________________________
    _> code snippet for ownership and borrowing rules
    -------------------------------------------------
    let name = String::from("");
    let pname = &name;
    
    println!("location: {:p}", &name);
    println!("value is {:?}", name);
    
    fn get_name(name: String){ // name gets moved completely by the get_name method, so we can't access name after this call
        
        println!("location: {:p}", &name);
        println!("value is {:?}", name);
        
    }
    
    get_name(name);
    // same value but different location cause the ownership has been taken by the compiler:
    // location before moving into get_name: 0x7fff81e14150
    // location after moving inside get_name: 0x7fff81e141b0

    // can't access pname in here since it's moved and we can't use a pointer of a data which has been moved or
    // is not good to move a data if it's behind a pointer already, we should pass the name by reference to the
    // get_name() method or clone it so in order to be able to use panem later.
    // println!("pname : {:?}", pname);

    Here's a breakdown of what happens in above code snippet:

        1 - The name variable owns the String data.
        2 - The pname reference borrows the name data.
        3 - When get_name(name) is called, the ownership of the String data is transferred to the get_name method.
        4 - the newly name variable inside the method now has a new location inside the ram and the memory address of 
            the name String data on the heap does not change when it is passed to the get_name method, the ownership 
            transfer does not involve changing the memory address of the very first data on the heap.
        5 - After the call to get_name, the name variable is no longer valid, and any attempt to use it will result in a compilation error.
        6 - The pointer pname is still valid after the call to get_name because it is a reference to the original String data. 
            However, if you try to use pname to access the String data after it has been moved into the get_name method, 
            you will encounter a compilation error due to the borrow checker's rules.
        in Rust, the ownership system and borrowing rules ensure that memory safety is maintained, and the compiler 
        enforces these rules at compile time to prevent issues such as dangling pointers and data races: 
            - when a value is moved, the memory address of the data on the heap does not change as a result of the ownership 
                transfer, the ownership transfer involves updating the ownership information and ensuring that the original owner 
                is no longer valid. However, the actual memory address of the data on the heap remains the same.
            - When a value is moved, the ownership is transferred, but the data itself is not physically relocated in memory, 
                instead, the ownership information is updated to reflect the new owner, and the original owner is invalidated.
    
       ______________________
      |                      | 
     _↓___________    _______|______
    |   val = 1   |  |   p = 0xA1   |
    |-------------|  |--------------|
    |     0xA1    |  |      0xA2    |
     -------------    --------------

    the pointer field points to the val field in memory address A, 
    which contains a valid i32. All the pointers are valid, i.e. 
    they point to memory that does indeed encode a value of the 
    right type (in this case, an i32). But the Rust compiler often
    moves values around in memory. For example, if we pass this struct 
    into another function, it might get moved to a different memory 
    address. Or we might Box it and put it on the heap. or if this 
    struct was in a Vec<MyStruct>, and we pushed more values in, 
    the Vec might outgrow its capacity and need to move its elements 
    into a new, larger buffer.

           ____________________________________________________
          |                                                    |   
         _↓_____________________________     __________________|______
        |                               |   |   val = 1  |  p = 0xA1  |
        |-------------------------------|   |-------------------------|
        |     0xA1      |     0xA2      |   |   0xB1     |     0xB2   |
         -------------------------------     -------------------------

    When we move it, the struct's fields change their address, but not their 
    value. So the pointer field is still pointing at address A, but address 
    A now doesn't have a valid i32. The data that was there was moved to address 
    B, and some other value might have been written there instead! So now the 
    pointer is invalid. This is bad -- at best, invalid pointers cause crashes, 
    at worst they cause hackable vulnerabilities. We only want to allow memory-unsafe 
    behaviour in unsafe blocks, and we should be very careful to document this 
    type and tell users to update the pointers after moves.

    --------------------------------------------------------------
    ------------------- Box, Pin, Future recap -------------------
    --------------------------------------------------------------
    
    all Rust types fall into two categories:
        1 - Types that are safe to move around in memory. This is the default, the norm. For example, 
            this includes primitives like numbers, strings, bools, as well as structs or enums entirely 
            made of them. Most types fall into this category!
        2 - Self-referential types, which are not safe to move around in memory. These are pretty rare. 
            An example is the intrusive linked list inside some Tokio internals. Another example is most 
            types which implement Future and also borrow data, for reasons explained in the Rust async book.
    Types in category (1) are totally safe to move around in memory. You won't invalidate any pointers by 
    moving them around. But if you move a type in (2), then you invalidate pointers and can get undefined 
    behaviour, as we saw before.
    Any type in (1) implements a special auto trait called Unpin. but its meaning will become clear soon. 
    Again, most "normal" types implement Unpin, and because it's an auto trait (like Send or Sync or Sized1), 
    so you don't have to worry about implementing it yourself. If you're unsure if a type can be safely moved, 
    just check it on docs.rs and see if it impls Unpin!
    Types in (2) are creatively named !Unpin (the ! in a trait means "does not implement"). To use these types 
    safely, we can't use regular pointers for self-reference. Instead, we use special pointers that "pin" their 
    values into place, ensuring they can't be moved. This is exactly what the Pin type does.

    Pinning in Rust refers to the ability to ensure that a value is not moved in memory and 
    tell the compiler hey don't move this around the ram when i pass it through scopes this 
    is particularly important for asynchronous programming and working with types that contain 
    self-referential pointers like a pinned reference to the inner future. By "pinning" a value, 
    you prevent it from being moved, which is crucial for maintaining the integrity of self-referential 
    data structures, note that we can pin either the mutable, or immutable or the type itself 
    into the ram but if we pin the mutable we can't have immutable pointers later on and vice 
    versa but we can pin immutable pointer of the type and have other immutable pointers in 
    the scope, also if a data implements Unpin means it can't be pinned and is safe to be moved 
    and if a data doesn't implement Unpin or it's !Unpin means it can be pinned into the ram and 
    it's not safe to be moved around.

    by means the type is safe to be moved is rust will take care of solving any dangling pointer 
    issue later on by updating their pointer state to reflect the new location of them inside the
    ram but when we say a type is not safe to be moved means that rust won't take care of this 
    automatically and we should pin the type into ram to avoid it from moving completely.

    types that implement Unpin can be moved safely but those types likes futures and tratis that
    implements !Unpin are not safe to be moved and if we need them later to use them like solving
    a future object we must pin their mutable pointer into the ram to prevent them from moving so 
    we need Pin them to safely poll them or solve them using .await, by pinning the pointer of the 
    type we can tell the rust that hey don't move this type around the ram when the type wants to 
    be moved trait objects like closures are dynamically sized means they're stored on the heap in 
    order to act them as a separate object or type we need to either put them behind a pointer or 
    box them, this would be true about the futures cause they're traits too. boxing is the best way
    of passing them between different scopes since box is an smart pointer which puts the data
    on the heap and points to it with a valid lifetime so it's better to pass future objects as
    a boxed value.
    future objects must be pinned to the ram before they can be solved or polled the reason 
    of doing this is first of all they're trait objects and traits are dynamically sized means 
    they're size will be known at runtime second of all due to the fact that rust doesn't have 
    gc which causes not to have a tracking reference counting process for a type at runtime, 
    because it'll move the type if the type goes out of the scope hence in order to solve and 
    poll a future in other scopes later on, we should pin it to the ram first which can be done 
    once we await on the future but if we want to solve and poll a mutable reference of a future 
    we should stick and pin it to the ram manually, first by pinning the future into the ram using 
    Box::pin, tokio::pin!(), std::pin::pin!() then do an await on another instance of future or the 
    mutable reference of the future object, so if it is required to call .await on a &mut _ reference, 
    cause .await consumes the object itself and we can't have it later so in this case the caller 
    is responsible for pinning the future by pinning future objects manually we make them as a safe 
    object before polling them like having a mutable reference to them or move them into other parts 
    to solve them in different parts.
    conclusion:
    so pinning logic must be used if a type is not safe to be moved (!Unpin) like future objects 
    and we want to move it safely without changing its location in the ram for future usage, which
    can be done by pinning the mutable pointer of the type into the ram, for future and trait based
    objects this can be done by pinning their box smart pointer with Box::pin or the type itself 
    with tokio::pin!(), std::pin::pin!() or std::pin::Pin::new(&mut Data{});
    recap:
    futures are trait objects and traits are dynamically sized and they must be behind pointer like 
    &dyn or Box<dyn also they're unsafe to be moved and must be first stick into the ram then we can 
    move them between different scopes, the process can be done by pinning the mutable pointer of the 
    type into the ram to prevent that from moving around by the compiler it's ok to put .await on the 
    fut without manual pinning cause .await do this but it consumes the future and we can't do whatever 
    we want with the future after that like if we want to await on another instance of the future like
    the mutable pointer of the future we must do the pinning process manually, like pin the future into 
    the ram first then await on its mutable pointer, in the first place futures are unsafe to be moved
    and they may gets moved by the compiler before getting polled so in order to use their reference 
    we should tell the compiler that i'm using the pointer of this future so don't move it around until
    i await on its mutable pointer well the compiler says you must pin it manually!
    the reason of pinning the mutable pointer of the object instead of its immutable pointer into the stack
    is because mutable pointer can access to the underlying data of the object and by mutating it we can 
    mutate the actual content and data of the object itself thus by pinning the mutable pointer into the 
    stack we're pinning the object itself actually and prevent it from moving around.
    types that implements Unpin means they can be unpinned from the stack later but types that are !Unpin 
    means they don't implement Unpin so can't be unpinned so are not safe to be moved and they must be 
    pinned to the ram.

    some objects are not safe to be moved around, between threads and scopes their value must be first pin 
    into the ram to make them safe for moving this can be done via std::pin::Pin::new(&mut Data{}); 
    as we can see above the mutable pointer of the object gets pinned into the ram so we can move it around 
    safely, reason of pinning the mutable pointer is because the mutable pointer has access to the underlying 
    data and its value and by pinning it we're actually pinning the object itself. in case of trait objects,
    actually traits are not sized at compile time and due to the fact that they're dynamically sized and stored 
    on the heap they must be in form of pointer like &'validlifetime dyn Trait or Box<dyn Trait> so pinning 
    their pointer be like Box::pin(trait_object); which allows us to move them safely as an object between 
    threads and other scopes without changing their location at runtime by the compiler, in case of future 
    objects they're trait objects too and they're not safe to be moved around, to do so we must pin them into 
    the ram first cause we might want to solve and poll them later in other scopes, when we want to solve and 
    poll a future we put .await after calling it, .await first consumes the future object and do the pinning 
    process for us but if we want to move the future manually between scopes we should pin its mutable pointer 
    manually then move the pinned object safely for future solvation like: Box::pin(async move{}); which pins 
    the pointer of the future object into the ram, in this case its better to put the future object into heap 
    using Box to avoid overflow issues and pin the Box pointer into the ram for future pollings.

    pinning the pointer of future object into the ram, future objects are traits and traits must be behind &dyn 
    or Box<dyn to be as an object at runtime thus we're pinning the box pointer of the future object which is 
    on the heap into the ram to avoid moving it for futuer solvation. in order to move the future between 
    different scopes safely we should first pin it into the ram then we can move it as an object between threads 
    and once we get into our desired thread we can put an await on the pinned boxed to solve the future reason 
    of doing so is because future objects are not safe to move around by the compiler and the must be pinned 
    first then move around, this behaviour is actually being used in tokio::spawn tokio will move the pinned 
    box of the future into its threads for future solvation also the future task and its output must be Send and 
    Sync, in order to avoid overflowing, pinning must be done by pinning the pointer of the future object and 
    since futures are dynamically sized their pointer will be a Box which is an smart pointer with a valid 
    lifetime, which store the data on the heap and returns a pointer to that

    self-referential structure is a type has fields which store references to the struct itself
    it would break as soon as the move happens and would invalidate it; the references would be 
    dangling and rust can't update the pointer to points to the new location of the type (Pin is better) 
    a solution to this is either using Arc Mutex for multithreaded or Rc RefCell in single threaded to break 
    the cycle when a field is pointing to the struct itself (see graph.rs) or using Pin, unsafe 
    or raw pointers so we go for the second option thus our recap on pin, pointer, ownership and
    borrowing rules would be:
    in rust, data (heap data) often move around the ram by the compiler to avoid allocating extra spaces
    at runtime, most of the data like heap data are safe to be moved means that they're 
    implementing Unpin trait which means they don't need to be pinned to prevent them from moving 
    cause once they get moved rust compiler can take care of their pointers to point to the right 
    and newly location of them to avoid having any dangling pointers after moving. those types that 
    are not safe to be moved are the one who don't implement Unpin trait and are !Unpin means 
    they can't be moved safely and their pointers must get pinned into the ram so their value can't 
    be moved into a new ownership variable and thus we can move them safely around the ram, the 
    reason that we can't move them it's because rust won't take care of updating their pointers at 
    compile time and we may have dangled pointers after moving them, these types can be future objects 
    and self-referential types which need to be used later in other scopes like solving a future object 
    or avoid a self-referential type from moving by pinning it into the ram cause self-referential 
    structs are a no-go, rust has no way of updating the address in the references if the struct is 
    moved since moving is always a simple bit copy in other words rust compiler can't update their 
    pointers to point to the right location which forces us to pin the type to not allow to be moved 
    at all cause they are inherently unsafe since they are implicitly invalidated if they are ever 
    moved we're not allowed to move them at all unless use Pin or break cycle using Arc,Rc,Mutex,RefCell
    

    -----=====-----=====-----=====-----=====-----=====-----=====-----=====-----=====-----=====-----=====
                                                RULES:
    -----=====-----=====-----=====-----=====-----=====-----=====-----=====-----=====-----=====-----=====
    gc marks data actively used by the application as live heap which takes spaces during the app execution 
    so generally when you use gc if u want to pass the type by value besides the old ownership it creates 
    a new one inside the ram but in rust when u pass by value or pass the type itself it moves its 
    ownership and transfer it into a new one inside the new scope and drop the old one out of the ram 
    to clean extra spaces inside the heap hence reduce memory usage or heap allocation at runtime
    finally updates all the pointers to point to the new owner loaction of the value to avoid getting 
    invalidated pointers that's how rust is taking care of the ram and heap at compile time cause it 
    doesn't have gc and it must drop it to track each type in realtime, however you can share the ownership
    instead of moving and cloning to keep the type ownership and reduce the heap size and avoid allocating 
    extra sapce on the heap using &, arc, rc, box or even clone the type and for self-ref data types in order 
    to break the cycle you should pin the type into the ram so it can be at a fixed position this won't 
    allow rust to change the location of the value after moving cause the value hence the poiners are 
    pinned and stuck into the ram (heap) which uses the same position as it uses before even after moving into 
    new scope which don't transfer the ownership into a new type therefore there won't be any invalidated
    pointers, take note of the followings:
    - future objects are self-ref type they can't be boxed without pinning they must be pinned cause we might move them into later scopes for solvation
    - clone them although rust takes care of heap data by transferring their ownership into the new one in the new scope by 
      default, but you can either put them behind pointer like & or Box like &dyn Interface or Box<dyn Interface> to avoid 
      allocating extra space on the heap by passing their clone, use slices form for Vec and String also put heap data behind 
      poiter either & or Box to move their ref instead of cloning them 
    - memory managing model in rust is safe and fast cause it doesn't have gc and it drops data out of ram when u move them by 
      value not and taking ref to them, since the value will be transferred into a new ownership inside the new scope and all 
      its left pointers get updated after moving to avoid having dangled and invalidated pointers
    - use &, Rc, Arc, Box, Pin to move the type around different parts of the app without moving into new ownership 
      and losing ownership and cloning also &, Rc, Arc, Box, Pin also will be used to break the cycle of self-ref types 
      by wrapping these pointers around fields which is like another type containing the actual type and has all the methods
      of the actual type.
    - traits are dynamic sized they must be behind pointer to move them around like &dyn or Box<dyn 
    - traits can be returned from methods like -> impl Trait, the struct needs to impl the trait so we could return the instance
    - Box stores data on the heap so it sends the trait on the heap with a valid lifetime 
    - if we want to return a trait object from method in either -> Box<dyn or -> impl Trait the struct must impls the trait
    - any heap data will be moved into a new ownership once we pass it to a func so to prevent this, clone it or borrow it  
    - async move{} moves everything u want to use them inside this scope if u want to use them later u have to borrow them as static
      or clone them cause any lifetime in the async move{} scope is not valid and dosn'tlive long enough
    - go on the heap, share ownership using Box pointer, traits as objects and bound them to generic 
    - cannot move out of `*self` which is behind a mutable or shared reference
    - looping over heap data types takes the ownership of the type thus we can iterate over &mut type or clone the type
    - app state contians all global data that must be inited once and share their ownership between threads to avoid extra heap alloc
    - share ownership instead of moving using rc and & in single thread and arc in multiple thread (share the rced and arced type)
    - size of [] and str and traits can't be known at compile time thus they must be in slice form and behind pointer or box
    - dynamic sized types like vector and string are on the heap which can be used as slice form like &[] and &str to reduce the ram size
    - trait objs are dynamic sized types must be behind pointer we can put them on the heap using Box smart pointer or behind a valid ref
    - Box stores data on the heap carefully and securely with a valid lifetime 
    - trait can be as method ret type method param type and if we want to pass them around they must be boxed like Box<dyn Error>
    - if we don't know the trait implementor means the implementor will be specified at runtime thus the trait must be boxed 
    - can't move type if it's behind a pointer, pass by ref or clone it or deref it to return the owned data 
    - can't ret ref from method unless we have valid lifetime, &'static, &'valid, &self 
    - can't deref if the pointer is being used by or shared in other scopes, can't deref a shared pointer, CLONE TYPE
    - pass by ref instead of cloning and moving also borrow must live long enough to be moved into different scopes  
    - can't move pointer into tokio spawn cause borrowed value must live long enough like for static to be valid after moving
      also if we're passing a ref to a type which is belong to a method body we can't:
            - move out of the type cause it's behind a pointer 
            - move the pointer into the tokio scope since the pointer scapes out of the method body, because tokio spawn has a longer life 
              time than the pointer of the type and based on Rust ownership and borrowing rules once the type gets dropped out of ram or move 
              into a new ownership we can't use any existing pointer of that due to they are dangled and invalidated albeit Rust will update 
              them after moving to avoid abusing of old location address but can't be used in later scopes thus when we move the pointer of 
              a type inside a method into tokio scope the pointer escapes the method body which is not allowed
    - clone the type or borrow inside the loop to prevent from moving cause in each iteration the type gets moved
    - share ownership using arc and mutex in multithreaded scopes and rc and refcell in single threaded 
    - thread_local is a single thread global allocator and static lazy arc mutex can be used as a global type in multithread
    - passing by ref or moving decreases the heap size but cloning (moving out of ref) return owned data which increases the heap size
    - no heap data (&[], &str), if heap data pass by ref instead of cloning, if not pass by ref rust moves them to clean heap size 
    - to specify the type of a var we need to try cast the pointer of the var to the desired type
    - moving results updateing pointers to avoid getting invalidated pointers in case of self-ref types the type 
      is not safe to be moved self-ref types, raw pointers and future objects must be pinned on the heap at a 
      fixed position, tells rust don't transfer new ownership or change the location of the value if the type 
      wants to be moved cause we've pinned it into the ram so the location remains the same as the old one

*/

async fn pinned_box_ownership_borrowing(){

    /* ---------------------------------------------------------------------------------------
     ╰┈➤  
        unlike Go in Rust we don't have GC values move often between different parts of 
        the ram like when we pass heap data into a function or resizing a vector, because
        the this nature Rust gives us the concept of lifetime for every type means that 
        every type has its own lifetime and comes to end once the object is moved into a
        new scope cause after moving it gets a new ownership in the new scope hence its 
        address gets changed, we can't use any pointer of that type after moving however 
        Rust updates all its pointer to point to the new address of the new ownership of 
        type but can't use them. besides borrowing using & we got some smart and wrapper 
        pointers allows us to share the ownership of type between scopes and threads like
        Box, Rc, Arc and if we want to mutate the underlying data (shared mutable state) 
        we use &mut, RefCell, Mutex and RwLock, Box be used to put traits in it cause 
        traits are dynamically sized and can't be as a separate object they must behind
        &dyn or Box<dyn.
        in order to return future as an object from method call we need to pin the future 
        into an stable memory address, by doing so Rust understand that the value must not 
        be moved cause we need its pinned value later in other scopes so we can put .await 
        on it and fill the placeholder of the caller by the polled value. 
        when a type moves into a new scope Rust automatically call drop() on it and transfers the ownership 
        of the type into the new scope and drop the old one it won't keep multiple references and owners of 
        a type, there must be only one owner for each type cause that's what gc doesn't which is not in Rust 
        also Rust updates any pointer of that type to point to the new location after moving but can't use 
        them after moving, also can't move the pointer of a type into a new scope like moving a pointer into 
        tokio spawn cause the lifetime of the underlying type might no lived long enough to move its pointer 
        in this case the type has smaller lifetime thatn the tokio spawn scope or returning poitner from function 
        without a valid lifetime is not possible cause type inside function get dropped once the function 
        gets executed, an static lifetime could be a better solution to move the the pointer around cause 
        pointers after all depends on their underlying type lifetime, for futute objects we must pin their 
        box into the ram and use the pinned pointer to move them between scopes cause it has an stable 
        memory address in all scopes.

    ▶ in some cases we can't pass the borrow due to having lifetime issue with the underlying type
    ▶ in gc based lang pointers are mutable by default since gc takes care of reference counting at runtime
    ▶ in none gc based like rust pointers are immutable and mutable since a data can only have one mutable pointer and multiple immutable ones (!at the same time) and only one ownership at a time.
    ▶ Rust by default moves type around the ram if they don't implement Copy trait or if they're are heap data types we can avoid this by passing them around by borrowing them or cloning them
    ▶ can't move out of type if it's behind a shared ref (mutable/immutable) cause ref get dangled and point to a location which is not existed
    ▶ derefing needs to implemente the Copy trait cause Rust must have to make sure there would be owned version of the type 
      in there after dereferencing the pointer and for the heap data dereferencing them move the data and takes the ownership 
      which forces us to clone them, also can't deref if the ref is being used by other scopes as a shared ref.
    ▶ rust don't update the pointer of self-ref types so the move breaks and we need to break the cycle using smart wrapper pointers like Box::pin
    ▶ self-ref types like future objects and recursive function need to get Box::pin to breack their cycle
    ▶ &mut, arc, mutex, refcell, rc, box, pin, G: 'valid + Send + Trait, trait object: static, dynamic typing, dispatching, polymorphism
    ▶ underlying data of pointers must have valid lifetime so we can share their ref between scopes otherwise a dangled pointer is all we have
    ▶ pointer address itself considered as reference_id cause it's unique in ram and the entire lifetime of the app
    ▶ trait object can be behind pointer (&dyn or Box<dyn ), their value are the instance of the struct who impls the trait, their size depends on the implementor at runtime
    ▶ future trait as a boxed object (return it some where) must be pinned if the value get pinned at an stable memory address enables us to use the pinned pointer around the scopes without loosing the address
    ▶ the underlying data must be valid and long enough to move the pointer into a new scope like tokio spawn
    ▶ due to the lack of having gc Rust considerred lifetime for each type, once the type gets moved and dropped its lifetime comes to end 
    ▶ due to the lack of having gc Rust often moves heap data around the ram to clean extra spaces on the heap and reduce the allocated size
    ▶ due to the lack of having gc Rust it stores data on the heap seamlessly cause later it moves data to clean extra spaces 
    ▶ due to the lack of having gc Rust calls drop() method and pass the type it want to drop it at the end of each scope automatically 
    ▶ due to the lack of having gc Rust future objects must be pinned into an stable memory address
    ▶ if we don't want to lose heap data ownership we can share their ownership by borrowing them using &, Box, Pin, Rc, Arc or clone them
    ▶ traits are unsized and they must be behind pointer (&'valid dyn Box<dyn, Arc<dyn, Rc<dyn) but future objects (self-ref) as separate type must be a pinned box or Box::pin(fut)
    ▶ -> impl Trait and param: impl Trait for static dispatch + F: FnOnce() -> () bounding to trait + Pin<Box<dyn SelfRefTrait>>, Box<dyn Trait>, Rc<dyn Trait>, Arc<dyn Trait> for dyanmic dispatch
    ▶ based on the lifetime of a type we can avoid having dangling pointers, once the type gets dropped its pointer won't be valid anymore
    ▶ there is no such thing as global type we should define static type and to mutate it we need to put in mutex or rwlock to sync 
    ▶ don't move a type if it's behind a pointer, Rust updates pointers after moving to point to new address but can't use them
    ▶ dropping the type simply is possible by calling drop() method and pass the heap daga type to it
    ▶ can't move pointer of a type into a new scope unless the lifetime of underlying data is longer than the new scope
    ▶ to pass & references between different scopes the lifetime of the underlying type must be longer than the scopes 
      or the reference must have valid lifetime during the execution of the scope 
    ▶ &mut in single thread and mutex in multiple threads and scopes + app data vs static/local arc mutex with/without channels
    ▶ all the types inside function are owned by the function scopes and once the function gets executed all of them will be dropped 
      out of the ram thus can't return a pointer to a type owned by the function cause as soon as the type gets dropped the pointer 
      can't be updated to point to new location cause unless we use a lifetime 
    ▶ in gc based langs like Go there is not need to clone the type or share its ownership using smart pointers like box 
      and arc which counts the references of a type came from different scopes and threads in Go it's gc's duty to do so
    ▶ lockers to avoid duplicate writing into db in a single node by mutex and in cluster by redlock and zookeeper, it ensures only one client can mutate a data at the same time and store inside the db 
    ▶ borrow the type using &mut for single thread cause &mut can't be sent unless their referent be Send we can put referent in smart pointers and wrappers like Mutex or RwLock
    ▶ pointers (&mut + *, binding and derefing, heap data wrappers[Rc, Arc, RefCell, Mutex, Box, fut self-ref Box::pin(async move{})])
    ▶ traits (object safe, static and dynamic typing and dispatch, generic bounding, ret &'lifetime Type, macro plugins)
    ▶ every async task must be executed in separate execution of thread to avoid blocking like locking on mutex 
    ▶ actor worker to talk with services over rpc or message brokers and queues like redis and rmq (pubsub, prodcons mpsc based chan and q)
    ▶ fetch actor state by sending message which uses mpsc jobq channel behind the scene inside it own threadpool
    ▶ rmq uses the idea of mpsc jobq channel but in a remote way, the idea is implemented in actor mailbox message sending to fetch their state
    ▶ rust dp: callback, builder and trait based interface plugings and proxies with dynamic typing, dispatchingg and polymorphism
    ▶ Box::pin(async{}) + Rc<dyn , Arc<dyn, &mut + ltg with default type param in trait and structure
    ▶ Vec<Box<dyn ObjectSafeTrait>> vector of interface objects + cast trait object into type using Any trait + poly and dyn typing
    ▶ create a trait object to call its method at runtime on any type that impls the trait interface
    ▶ store on the heap using Box, Rc, Arc, vtable pointers to execute trait methods on the obj, 
    ▶ -> impl Trait for static and Box<dyn ObjectSafeTrait> for dynamic dispatch
    ▶ rust calls drop automatically when the the type goes out of scope like moving heap data into a new scope
    ▶ big data types like string, vector and complex types need to be stored on the heap using Box cause it has dynamic allocation
    ▶ every type has only one ownership we can share it using Rc, Arc, Box, & or we can move it to a new one by moving 
      the type into a new scope or we can clone it also can't move pointer of a type if the type is about to be dropped 
      like moving pointer into a tokio spawn if the lifetime of the underlying type is shorter than the lifeimte of tokio 
      spawn scope or when the type is owned by the funciton mainly we can't return pointer from method however doing so 
      can be successful if we use an static or the self lifetime to return the pointer cause the self lifetime is valid 
      as long as the object is valid.
    --------------------------------------------------------------------------------------- */

    //===================================================================================================
    //===================================================================================================
    //===================================================================================================
    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///// if Future<T> is an async version of T then it must be Send Sync 'static and pinned into the ram /////
    ///// heap smart pointers and wrappers around the type ownership: Arc, Rc, Box, Pin ///// 
    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    /*  
        traits are interfaces which are dynamically sized cause they don't have size on their own they need 
        implementor to get size at compile time otherwsie we could use them as object trait with Box<dyn or 
        &dyn, Box is better cause it handles dynamic allocation and lifetime on the heap but pointer needs a 
        valid lifetime also pointer of self-ref traits or Box must be pinned into the ram cause rust don't update 
        their pointers once they moved into a new scope by pinning we tell rust that the value has stuck into 
        a fixed position into the ram so don't worry about new ownership and addresss, pointers won't get invalidated.
        ---------------------------
        rust updates all pointers of the type after moving but doesn't allow to use them since the type has been 
        moved out of the ram and has a new address and ownership, pin is good for pinning the type value into the 
        ram to avoid dropping it and keep it at a fixed position, the location of the value will be fixed so any 
        pointer of that would be valid cause the ownership of the value has not transferred into a new one, rust 
        doesn't have gc so it moves heap data types around the ram once they go out of the scope like by passing 
        them into a function, the drop() method will be called for the old owner before it get passed so we don't 
        have it after moving and any pointer of that can't be used however rust update them with a new location of 
        the moved value which contains the new address cause once the type goes into a new scope it gets a new ownership 
        in that scope hence it address will be changed thus to avoid having invalidated pointers rust updates all 
        pointer after moving to point to the right location but can't use them!, this is not true about self-ref 
        types like future objects or structure with their own type fields the solution is pinning them into the ram 
        on the heap using Box::pin which has dynamic allocation and lifetime, pinning a mutable pointer of the type 
        pins the actual value of the type cause passing the value to pin method moves the type we can pin either the 
        pointer or mutable pointer of that to have the type in later scopes.
        ---------------------------
        Pin pins tell Rust to pin the value in an stable memory address and tell Rust don't move it from 
        that position cause we want to deal with pointers of pinned value and we need a fixed memory address 
        to avoid having dangling pointers like pointing to self-ref types, we do this by pinning the self-ref
        type into the ram so any pointer of that be valid while we're passing the pinned object into other scopes
        and functions cause we know that the pinned value has a fixed memory address and not afraid of having
        dangling pointers of that, note that the Rust ownership rules precedence over this meaning that if 
        the pinned value moves then the pinned pointer itself will be invalid!
        ---------------------------
        In Rust, the concept of pinning a value with `Pin` does not mean fixing the memory address of the value. 
        Instead, pinning ensures that the value remains at a stable memory location and does not move in memory, 
        even when the surrounding data is moved. This is crucial for certain scenarios where the stability of 
        memory addresses is required, such as when dealing with self-referential structs or when working with 
        asynchronous code.
        Here are the key points to understand about pinning in Rust:

            1. **Preventing Unintended Movement:** When a value is pinned with `Pin`, it means that the value will 
                not be moved in memory by Rust's normal ownership rules. This is important for scenarios where the 
                stability of memory addresses is critical, such as when dealing with references that point to the 
                pinned value.

            2. **Stability for References:** Pinning ensures that references to the pinned value remain valid and 
                do not become dangling references, even if the surrounding data is moved. This is particularly useful 
                in asynchronous programming, where references to data may need to be preserved across asynchronous 
                boundaries.

            3. **Interaction with Drop Trait:** Pinned values can still be dropped like any other value in Rust. 
                However, the `Drop` trait implementation for a pinned value should not move the value, as moving 
                a pinned value can lead to undefined behavior.

            4. **Memory Safety:** By using `Pin`, Rust provides a safe way to work with self-referential data 
                structures and ensures that references to the pinned value remain valid and do not lead to memory 
                unsafety.

        generally pinning a value with `Pin` in Rust does not fix the memory address of the value but rather 
        ensures that the value remains stable in memory and does not move unintentionally. This stability is 
        crucial for maintaining the validity of references and ensuring memory safety in scenarios where the 
        movement of data could lead to issues with references and memory management.
        ---------------------------
        In Rust, the `Pin` type is used to ensure that a value remains at a fixed memory location, preventing it 
        from being moved or deallocated. However, it's important to understand that pinning a value with `Pin` 
        does not prevent the value from being moved by ownership transfers. When you pass ownership of a value 
        to a function or another scope, the original value is moved, and any pointers to that value, including 
        `Box::pin` pointers, become invalid (dangling).
        when you pass the `name` variable to the `try_to_move` function, ownership of the `String` is transferred 
        to the function, and the original `name` variable is no longer valid in the current scope. This means that
        any pointers or references to `name`, including the `Box::pin` pointer `pinned_name`, become dangling 
        references and cannot be used safely.
        The purpose of pinning a value with `Pin` is to ensure that the value remains at a fixed memory location 
        when working with asynchronous code or self-referential structs. However, pinning does not prevent the 
        value from being moved by ownership transfers, as ownership rules in Rust take precedence over pinning.
        If you need to access the pinned value after an ownership transfer, you would typically need to re-establish 
        a valid reference or pointer to the value in the new scope. In your example, once `name` is moved to the 
        `try_to_move` function, you would need to return the value back to the original scope or re-create a valid 
        reference to it to continue using the pinned pointer `pinned_name`.

        trait objects need to behind a pointer like &dyn or Box<dyn cause they're ?Sized like recursive
        methods need to behind a pointer so that the state of the future isn't infinitely large, Box
        puts the data on the heap and allocate lifetime and space dynamically.
        if a type need to reference itself it must not be able to gets moved and pin can fix this 
        by pinning the type into an stable address inside the ram, awaiting a future behind a pointer 
        needs to pin the future cause the address must be stable and since future traits are dynamic 
        we should have something like Box::pin(fut), having futures as separate object needs to pin 
        their box into the ram, like constructing futures and pin!{} them before falling into 
        the loop of select!{} as well as self ref types can't get moved by the rust compiler if their
        pointer has not been broken by some indirection yet, in general having self ref types as a separate 
        objects need to break the cycle by adding some indirection like putting them behind a pointer 
        or pin them into the ram or wrap them using smart pointers. the most important self ref types
        are future objects that needs to get pinned into the ram when we want to have them with an 
        stable address on the ram and don't allow them to get moved cause they can't move at all!
        iterating over stream which is a collection of future objects requires to pin the stream first 
        then call the next method.
    */

    // object safe trait must behind Box && future as object must behind Box::pin so Box::pin(async move{}) is good to go!
    type FutureObject = Box<dyn std::future::Future<Output = String>>;
    let buffer: Vec<FutureObject> = vec![];
    struct FutureVector<F: Send + Sync> where 
    F: std::future::Future<Output = String>{
        pub futures: Vec<F>,
        pub pinned_futures: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = String>>>>
    }
    
    // Boxing mutable pointer of the type has access to the underlying value of the data
    // so mutating the boxed value mutates the actual data, by pinning the mutable pointer
    // of the type we're actually pin its value mutably into the ram means we can mutate 
    // its content later by using its pinned box
    
    // pin future enables us to move it around the ram cause it's a self-ref types 
    // and these types must be pinned be wrapped around smart pointers like Rc, Box, Arc
    // also the future might get solved later in other scopes so pin is better options to 
    // be used in here to move the future around different scopes and threads
    let boxed_fut = Box::pin(&mut async move{}); // pinning the mutable pointer to the future, Box handles the allocation and lifetime of the type dynamically on the heap
    
    // **************************************************************
    // ************************* SCENARIO 1 *************************
    // **************************************************************
    // if the type implements Unpin it means it can be moved easily 
    // cause it uses the Unpin trait to unpin the type from the ram
    // to move it easily, for type to be able not to move easily the
    // Unpin trait must not be implemented for that or it must impls
    // the !Unpin trait.
    let mut name = String::from("");
    let mut pinned_name = Box::pin(&mut name); // box and pin are pointers so we can print their address
    **pinned_name = String::from("wildonion");
    // passing name to this method, moves it from the ram
    // transfer its ownership into this function scope with
    // new address, any pointers (&, Box, Pin) will be dangled
    // after moving and can't be used 
    // SOLUTION  : pass name by ref so we can use any pointers of the name after moving
    // CONCLUSION: don't move a type if it's behind a pointer
    fn try_to_move(name: String){ 
        println!("name has new ownership and address: {:p}", &name);
    }
    try_to_move(name);
    // then what's the point of pinning it into the ram if we can't 
    // access the pinned pointer in here after moving the name?
    // we can't access the pinned pointer in here cause name has moved
    // println!("address of pinned_name {:p}", pinned_name);
    // **************************************************************
    // ************************* SCENARIO 2 *************************
    // **************************************************************
    // i've used immutable version of the name to print the 
    // addresses cause Rust doesn't allow to have immutable 
    // and mutable pointer at the same time.
    let mut name = String::from("");
    println!("name address itself: {:p}", &name);
    let mut pinned_name = Box::pin(&name); // box and pin are pointers so we can print their address
    println!("[MAIN] pinned type has fixed at this location: {:p}", pinned_name);
    // box is an smart pointer handles dynamic allocation and lifetime on the heap
    // passing the pinned pointer of the name into the function so it contains the 
    // pinned address that the name has stuck into same as outside of the function
    // scope, the stable address of name value is inside of the pinned_name type
    // that's why is the same before and after function, acting as a valid pointer
    fn move_me(name: std::pin::Pin<Box<&String>>){
        println!("name content: {:?}", name);
        println!("[FUNCTION] pinned type has fixed at this location: {:p}", name);
        println!("pinned pointer address itself: {:p}", &name);
    }
    // when we pass a heap data into function Rust calls drop() on the type 
    // which drop the type out of the ram and moves its ownership into a new one 
    // inside the function scopes, the ownership however blogns to the function
    // scope hence returning pointer to the type owned by the function is impossible.
    move_me(pinned_name); // pinned_name is moved
    println!("accessing name in here {:?}", name);
    // **************************************************************
    // ************************* SCENARIO 3 *************************
    // **************************************************************
    // the address is change because a pinned pointer contains 
    // an stable address in the ram for the pinned value, second 
    // it’s the stable address of the new data cause pin pointer 
    // contains the stable address of the pinned value
    let name = String::from("");
    let mut pinned_name = Box::pin(&name);
    println!("pinned at an stable address {:p}", pinned_name);
    let mut mutp_to_pinned_name = &mut pinned_name;
    let new_val = String::from("wildonion");
    // here mutating the underlying data of mutp_to_pinned_name pointer
    // mutates the data inside pinned_name name pointer cause mutp_to_pinned_name 
    // is a mutable pointer to the pinned_name pointer, so putting a new value
    // in place of the old one inside the pin pointer will change the address
    // of the pinned pointer to another stable address inside the ram cause 
    // Box::pin(&new_val) is a new value with new address which causes its 
    // pinned pointer address to be changed 
    *mutp_to_pinned_name = Box::pin(&new_val);
    println!("pinned_name at an stable address {:p}", pinned_name);
    println!("pinned_name content {:?}", pinned_name);
    //===================================================================================================
    //===================================================================================================
    //===================================================================================================

    // only Box pin can be used with future to break cycle of self ref types
    // future object safe traits must be pinned into the heap using either Rc, Arc or Box,
    // object safe traits are unsized thus can't be in Mutex or RefCell also they must be 
    // behind either &'valid dyn or smart pointers like Rc<dyn, Arc<dyn or Box<dyn
    // Rc, Arc, Box pointer can be wrapped around object safe trait 
    // initialization value would be the struct instance who impls the trait 
    // closures, FnOnce, FnMut, Fn and Future are all traits
    // NOTE: future objects as return type or a separate type must be a pinned box 
    //       we can't use Rc or Arc for them cause they can't be unpinned they're !Unpin
    trait Interface{}
    let mut displayable: std::sync::Arc<dyn Interface>;
    let mut future1: std::pin::Pin<std::rc::Rc<dyn std::future::Future<Output = String>>>;
    struct CErr{}
    impl Interface for CErr{}
    displayable = std::sync::Arc::new(CErr{});

    fn getUser<F1: std::future::Future<Output=()>, R: Send + Sync + 'static + std::future::Future<Output = ()>>(
        f1: F1,
        f2: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
        f3: std::sync::Arc<dyn Fn() -> R + Send + Sync + 'static> 
    ){}
    fn setUser(){}
    let funcOverUnsafe = setUser as *const (); // () in C style means function pointer in Rust is fn

    ////---------+++++++++---------+++++++++---------+++++++++---------+++++++++
    // we can use heap based smart pointers like Box and Arc to store 
    // traits as objects on the heap for dynamic dispatch otherwise use 
    // impl Trait in return type or method param.
    // note that in object safe trait we can't return Self as return type and
    // as well as can't have GAT since: for a trait to be "object safe" it 
    // needs to allow building a vtable to allow the call to be resolvable dynamically
    trait InterfaceFuture{
        fn getCode(&self);
        }
    let arced: Arc<dyn InterfaceFuture>;
    let boxed: Box<dyn InterfaceFuture>;
    let futured: std::pin::Pin<Box<dyn std::future::Future<Output=String>>>;
    
    // unpacking in function calls 
    pub struct Data1<T>(pub T);
    pub struct Data<T>{
        pub data: T
    };
    // fn get_data(Data(data): Data<String>){}
    fn get_data1(Data{data}: Data<String>){}
    fn get_data2(Data1(data): Data1<String>){}
    pub trait Component<P>{
        fn render(&self, props: P);
    }

    pub struct App{}
    impl Component<String> for App{
        fn render(&self, params: String){}
    }

    struct QueryParameter<T, const REQUIRED: bool>{
        pub q: T,
    }
    impl<T, const REQUIRED: bool> QueryParameter<T, REQUIRED>{
        fn get_param(&self, mut is_req: bool, QueryParameter { q }: QueryParameter::<String, true>){
            let qu = QueryParameter::<String, true>{
                q
            };
            is_req = REQUIRED;
        } 
    }

    // smart pointers contian all the infos of their underlying types 
    // cause they're are wrappers around the actual type
    let components: Vec<Box<dyn Component<String>>> = vec![Box::new(App{})];
    for comp in components{
        comp.render(String::from("{{p1}}|{{p2}}|{{p3}}"));
    }
    ////---------+++++++++---------+++++++++---------+++++++++---------+++++++++

    //-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
    //-=-=-=-=-= into service trait -=-=-=-=-=
    //-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
    trait ArrExt{
        fn getCode(&self) -> &Self;
    }
    type Arr = [String];
    impl ArrExt for Arr{
        // returning the borrowed form of [String] with the lifetime of &self 
        // since [String] has no fixed size at compile time
        fn getCode(&self) -> &Self{
            todo!()
        }
    }

    let hex: u8 = 0xff; // hex
    let oct = 0o777; // octal
    let bin = 00001110; // bin

    let arr_str: &Arr = &[String::from("")]; // slices need to behind the & due to their unknown size at compile time
    _ = arr_str.getCode();
    
    struct MyService;
    pub trait IntoService<T>{
        fn into_serivice(&self, config: T) -> MyService;
    }
    // static dispatch
    fn get_node<'valid, N: Clone + Send + Sync + IntoService<N> + 'static>(n: N) 
        -> (impl IntoService<N>, std::pin::Pin<Box<dyn std::future::Future<Output = N>>>){
            // note that N must live long enough as static cause we're using it
            // as the result of the future object which must be pinned into the ram
            // so we can solve it later by accessing its stable address
        (n.clone(), Box::pin(async move{n})) 
    }
    fn get_node_service<'valid, N: Clone + Send + Sync + IntoService<N>>(n: N) -> MyService{
        n.into_serivice(n.clone())
    }
    //-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
    //-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
    //-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=

    // Fn, FnMut and FnOnce are triats, having them as separate type requires to 
    // put them behind &dyn or Box<dyn 
    type Pointer2Func<'v> = &'v dyn Fn() -> ();
    trait Interfacev1{
        fn getName(&self) -> String;
    }
    struct Contract;
    impl Interfacev1 for Contract{
        fn getName(&self) -> String {
            todo!()
        }
    }

    // pinning a box of trait is pinning the instance of the struct who impls
    // the trait itself it's like pinning a trait object and if we want to do 
    // dynamic dispatching later the trait must be object safe trait, however
    // in the our following case we can call the getName() method on pinned_trait
    // object.
    let pinned_trait: std::pin::Pin<Box<dyn Interfacev1>> = Box::pin(Contract{});
    pinned_trait.getName();

    // returning a future as the method return type with box::pin
    // later on we can await on it to get the result in other scopes
    // and threads
    fn ret_fut<O: Send + Sync + 'static>(param: O) -> std::pin::Pin<Box<dyn std::future::Future<Output = O>>>{
        Box::pin(async move{
            param
        })
    }
    let res = ret_fut(String::from("wildonion")).await;

    //========================= pinned with mutex
    // generally all the pinned types has an stable address across scopes
    // ...
    // move it between threads then:
    // check that the address must be stable and be fixed at a memory location
    let pinned_safe: std::pin::Pin<Box<std::sync::Arc<std::sync::Mutex<String>>>> = 
        Box::pin(
            std::sync::Arc::new(
                std::sync::Mutex::new(
                    String::from("")
                )
            )
        );
    println!("stable address: {:p}", pinned_safe);
    fn get_boxed_version(pinned_safe: std::pin::Pin<Box<std::sync::Arc<std::sync::Mutex<String>>>>){
        
        println!("stable address: {:p}", pinned_safe);
        pinned_safe.lock().unwrap(); // locking by blocking!
        also_stable_here(pinned_safe);
        
    }
    fn also_stable_here(pinned_safe: std::pin::Pin<Box<std::sync::Arc<std::sync::Mutex<String>>>>){
        println!("stable address: {:p}", pinned_safe);
    }
    get_boxed_version(pinned_safe);
    //=========================

    // we can handle the lifetime of types dynamically on the heap using smart pointers
    // cause they have their own lifetime
    // Box<&mut Type> manage the lifetime and allocation of types dynamically using smart pointer on the heap
    // they can be used to share the ownership of type dynamically between scopes behind their 
    // own valid lifetime and dynamic allocation handlers besides dynamic dispatching by boxing traits.
    // to move the future around between different scopes and threads we need to break the self ref types using box pin 
    // and tell rust pin this into a fixed position then we can move the future around the ram easily
    
    /////// deref
    let mut var = String::from("");
    println!("var is : {:?}", var);
    let pmut = &mut var;
    let boxed_pmut = Box::new(pmut);
    **boxed_pmut = String::from("updated"); // mutate the underlying data of the box pointer with a new value, this changes the var too
    println!("var is : {:?}", var);
    
    /////// new binding 
    let mut var = String::from("");
    println!("var is : {:?}", var);
    let pmut = &mut var;
    let mut boxed_pmut = Box::new(pmut);
    *boxed_pmut = &mut String::from("updated"); // change the box with a new binding / need only one derefing
    println!("var is : {:?}", var);


    /* 
        self-ref, raw pointers, future objects, recursive funcs:
        we have to put them on the heap using rc, arc, box to break the cycle pointer 
        to themselves cause rust won't update their pointer if they want to be moved,
        or we can pin their value into the ram at a fixed position and address
        which forces rust not to change the location of the type even if it wants to move
        hence any pointer of that won't get invalidated, self ref types are struct
        with fields of their own type and recursion functions they need to behind 
        some smart pointers which adds indirection like Box::pin, rc, arc usually 
        Box::pin is perfectly fine to break the cycle,

        the reason of not allowed to have async recursive func is because the function 
        itself is a future object and future objects are traits which are dynamic sized 
        having them as trait object requires to put them behind Box<dyn and be an object
        safe trait, in Rust however, self ref types can't be moved around the ram easily 
        it's not safe to do so cause any pointer of self ref can't be updated by the Rust 
        compilre if they want to be moved and due to this, any moves gets broken in the 
        first place, solution to this is adding some indirection to them to break the cycle 
        like wrap them with Box, Rc, Arc or pin them into the ram at a fixed position to 
        not allow Rust ownership and borrowing to move them between different parts of the 
        ram cause by every move the type gets a new ownership thus new address 
        Rust doesn't have gc and it moves data between different parts of the ram to clean
        the heap by moving them the address gets changed too so any pointer of them which 
        is pointing to them must be updated to be valid but Rust ignore updating the pointers
        of self ref types which causes to break the move in the first place, pinning type 
        tells Rust that it's safe to move the type since it has stuck into a fixed position 
        and can't be moved thus any pointer won't get invalidated because its pointers 
        point to the same location even after moving.

        smart pointers are a wrapper around the type to put them on the heap therefore 
        it has all the methods of their underlying data
    */
    async fn help(n: u8){
        if n == 0{
            return;
        }
        
        let boxed = Box::pin(help(n)).await; // adding some indirection to break the cycle of self calling
    }

    ////////////// -------------- recursive ex
    // https://lunatic.solutions/blog/rust-without-the-async-hard-part/
    /*
        ---- reasons that we can't have async recursive?!
        the compiler can't pre determine how many recursive 
        calls you're going to have and can't reserve the perfect
        amount of space to preserve all local variables during 
        the context switching that's why we have to put the 
        the async function on the heap and since future objects
        are self-ref types that move gets broken when they're 
        doing so, we need to pin them into the heap.
    */
    async fn check_base(param: u8){
        if param == 0{
            return;
        } else{
            // since the check_base is a future object and pointing 
            // to self ref types needs to add some indirection using Box 
            // or any other smart pointers putting future inside the 
            // Box requires to pin the future since they must be in an
            // stable memory address during their execution.
            Box::pin(async move{
                check_base(param * 2).await;
            });
        }
    }

    trait SpecialExt{
        type Gat: Send + Sync;
    }
    type Fut1<G> = Box<dyn std::future::Future<Output = G>>;
    struct Special<'n, F, G: SpecialExt<Gat: Send + Sync>> where F: std::future::Future<Output = G>, 
        G: FnOnce() -> () + Send + Sync + 'static{
        pub fut: F,
        pub name: &'n mut str,
        pub indir_cyclic: std::pin::Pin<Fut1<G>>
    }
    ////////////// --------------

    // to move around self-ref types like async and fut objs between different scopes and parts of the ram
    // they must be pinned into the ram at an stable memory address to tell Rust that any pointer of them 
    // won't be invalid cause you can't drop the value later on. a variable of type Box<dyn Future<Output=()>>
    // is an object safe trait that can be used for dynamic dispatching however futures are self-ref types 
    // we need to break their self direction by adding some indirection using smart pointers wrappers like
    // Rc, Arc, Box, Pin and since they want to be solved later they must get pinned into the ram enables us
    // to put .await on them notify the waker to poll the result out and update the placeholder of the caller.
    let fut = Box::pin({
        async fn get_me(){}
        get_me
    });
    let res = fut().await;

    // ====================================
    //          Boxing traits
    // ====================================
    impl std::fmt::Display for ClientError{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
            todo!()
        }
    }
    #[derive(Debug)]
    struct ClientError{}
    fn set_number() -> i32{ 0 }
    impl std::error::Error for ClientError{} // the Error trait must be implemented for the enum so we can return a boxed instance of the ClientError
    let boxed_error: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(ClientError{}); // we can return the boxed_error as the error part of this return type: Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>
    let boxed_cls: Box<dyn FnMut(fn() -> i32) -> ClientError + Send + Sync + 'static> = 
        Box::new(|set_number|{
            let get_number = set_number();
            ClientError{}
        }); 

    // ====================================
    //          self ref type
    // ====================================
    // can't have self ref types directly they should be behind some kinda pointer to be stored on the heap like:
    // we should insert some indirection (e.g., a `Box`, `Rc`, `Arc`, or `&`) to break the cycle (they're smart 
    // wrappers and pointers around the actual type and contain all the actual types' methods)
    // also as you know Rust moves heap data (traits, vec, string, structure with these fields, ?Sized types) to clean the ram 
    // so put them inside Box, Rc, Arc send them on the heap to avoid lifetime, invalidate pointer and overflow issue
    // also Arc and Rc allow the type to be clonned, so they're a heap data wrappers and smart pointers which must be 
    // around self-ref fields to borrow their ownership and break the cycle that's why in graph structures we need to 
    // use either Arc Mutex for multithreaded based graph or Rc RefCell for single thread based graph to wrap around the 
    // parent and children fields since graph fields (parant and children) are of type Node itself which makes a cycle 
    // at compile time.

    // by pinning we say it's safe for the type to be moved cause it's value has stuck into a fixed position inside the ram hence its corresponding pointers which point to the value loaction 
    type Fut<'s> = std::pin::Pin<Box<dyn futures::Future<Output=SelfRef<'s>> + Send + Sync + 'static>>; // pinning the box type on the heap at a fixed position to tell Rust don't move this from the its location when we're moving it around the scopes
    struct SelfRef<'s>{
        pub instance_arc: std::sync::Arc<SelfRef<'s>>, // borrow and is safe to be shared between threads
        pub instance_rc: std::rc::Rc<SelfRef<'s>>, // borrow only in single thread 
        pub instance_box: Box<SelfRef<'s>>, // put it on the heap to make a larger space behind box pointer
        pub instance_ref: &'s SelfRef<'s>, // put it behind a valid pointer it's like taking a reference to the struct to break the cycle
        pub fut_: Fut<'s> // future objects as separate type must be pinned
    }

    let mut future = async move{};
    tokio::pin!(future); // first we must pin the mutable pointer of the future object into the stack before solving/polling and awaiting its mutable pointer 
    (&mut future).await; 
    
    let fut = async move{};
    let mut pinned_box = Box::pin(fut); // in cases if we need to access the pinned value outside of the current scope cause the future is boxed and we can move it as an object
    (&mut pinned_box).await;
    pinned_box.await;

    /*
        the type that is being used in solving future must be valid across .awaits, 
        because future objects will be pinned into the ram to be solved later, worth
        to know that trait pointers are Boxes and we pin their pointer into ram like: 
        Pin<Box<dyn Future<Output=String>>>
    */

    fn get_data<G>(param: impl FnMut() -> G) -> impl FnMut() 
        -> std::pin::Pin<Box<dyn std::future::Future<Output=String> + Send + Sync + 'static>>
        where G: Send + Sync + 'static + Sized + Unpin{ // G is bounded to Unpin means it can't be pinned into the ram
        ||{
            Box::pin(async move{
                String::from("")
            })
        }
    }

    async fn callback() -> i32 {3}
    // we can't add let func: fn callback() -> impl Future<Output = i32> but compiler can
    let callbackfunc = callback;
    callbackfunc().await;

    let pinned_boxed_future: std::pin::Pin<Box<dyn std::future::Future<Output=String>>> = 
        Box::pin(async move{
            String::from("")
        });

    async fn func(){}
    type Type = Box<dyn std::future::Future<Output=()> + Send + Sync + 'static>;
    struct Generic<'lifetmie, Type>{
        pub data: &'lifetmie mut Type // mutating mutable pointer mutates the underlying data too
    }
    let mut instance = Generic{
        /*  
            to have future objects as a type which are of type Future trait we have to
            put them behind a pointer and pin the pointer into the ram to get their result
            in later scopes by awaiting on them which actually will unpin their pointer,
            we can't use Box::new(async move{()}) if we want to access the result of the 
            future outside of the Boxed scope to solve this we must pin the boxed value 
            which in this case is pinning the pointer to the Future trait, and put an await
            on that in later scopes to unpin the boxed value from the ram to get the result
            of the future object

            since Future trait doesn't implement Unpin trait thus we can pin the boxed 
            type into the ram by constructing a new Pin<Box<Type>>. then Type will be 
            pinned in memory and unable to be moved.
        */
        data: &mut Box::pin(func()) // passing the result of calling async func to the pinned box
    };
    let unpinned_boxed = instance.data.await;
    /*  
        moving type can also be dereferencing the type which converts
        the pointer into the owned value but based on the fact that 
        if the type is behind a pointer we can't move it! so we can't
        deref the pinned boxed in here, we must clone it or borrow it 
        which clone is not working in here because Clone it's not 
        implemented for &mut Type which is the type of data field
    */
    // let deref_boxed = *instance.data;
    instance.data = &mut Box::pin(func()); // passing the result of calling async func to the pinned box


    async fn futfunc(){}
    let fut = futfunc;
    let pinned = Box::pin(&fut);

    println!("[IN MAIN] fut pinned at an stable address {:p}", pinned); // pin is a pointer by default
    fn get_fut<R: std::future::Future<Output = ()>>
        (fut: fn() -> R){
    }
    get_fut(fut); // fut moves here but we have it's pinned value at an stable memory address
    
    // future as a trait object and for dynamic dispatching must be pinned into ram
    // fut objs are self-ref types, these types are not safe to be moved by Rust 
    // cause Rust can't update their pointers as soon as they moved into a new scope
    // and get new ownership so it breaks the move in the first place and can't have
    // them directly inside the code, struct with fields point to its own type and 
    // future objects are all kind of self-ref types, to break the cycle and add
    // indirection and make them safe to be moved we can put them in smart pointer
    // wrappers like Box, Pin, Arc, Mutex, Rc, RefCell or even &, Box has a dynamic
    // allocation and lifetime handler on its own, all these wrappers store data on 
    // the heap, in case of future objects we should use Pin since it guarantees that
    // the value will be pinned into an stable memory address and can't be moved by 
    // Rust compiler ensures we can solve the future later in different scopes and threads.
    // trait can be appeared as an object only if they're safe, enables us moving 
    // them around different parts, comps and methods as an object safe trait, they 
    // must be behind Box<dyn allows us to do dynamic dispatching means we don't know 
    // the exact type of the implementor but rather it'll specify at runtime so their 
    // type is an object safe trait enables us to call trait method on their instances
    // we must use Pin to pin the value of future object in memeory at an stable address
    // to ensure that memory location of the future object remains stable during moving 
    // the pinned object into different scopes and threads for future solvation.
    fn ret_fut_object() 
        // we can pass future object as a boxed trait only to the functions and methods
        // but returning a future as an object from method needs to be pinned into the ram
        // perhaps we want to solve it later not right after calling the method
        -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>{
            let boxed_fut1 = Box::new({
                async fn a_fut(){}
                a_fut
            });
            Box::pin(async move{
                // anything
                // ...
            })
        }
    let res = ret_fut_object().await;



    let name = String::from("");
    loop{
        // it's better to move a unique type in each iteration
        // Rust enforces us to do us in the first place  
        let cloned_name = name.clone();
        tokio::spawn(async move{
            // if  we use name in here means we moved the name into this scope 
            // and don't have it after tokio spawn scope so it's better to clone 
            // it if you want to use it later or borrow it with a lifetime longer 
            // than the tokio spawn scope like static  
            let n = cloned_name;
        });
    }


}


async fn dynamic_static_dispatching1(){


    /// -------========-------========-------========-------========-------========-------========
    /// -------========-------========-------========-------========-------========-------========
    /// -------========-------========-------========-------========-------========-------========
    // -> implt Future    : the future type must be known at compile time 
    // -> Box<dyn Future> : it can be any future object that implements the Future trait 
    // dynamic dispatch use vtable indirection to call trait methods on the 
    // instance who impls the Trait alreadyl, Box dyn Trait is for dynamic dispatch 
    // and impl Trait is for static dispatch  

    // ----------- STATIC DISPATCH 
    /* 
        Return Type: The return type is an impl std::future::Future<Output = String>, which means the exact type of the future is determined at compile time.
        Performance: Since the compiler knows the exact type, it can optimize the future's state machine more effectively, leading to better performance.
        Size and Allocation: This approach does not require heap allocation. The future is stored directly on the stack, which can be more efficient.
        Flexibility: Less flexible in terms of returning different types of futures from the same function. Every future returned must have the same concrete type.
        Use Case: Preferable when the function always returns the same type of future, and performance is critical.
    */
    fn ret_fut_static_dispatch() -> impl std::future::Future<Output = String>{
        async move{
            String::from("")
        }
    }

    async fn getFut(fut: impl std::future::Future<Output = String> + Send + Sync + 'static){ // static dispatch
        // fut.await;
        tokio::spawn(async move{
            fut.await;
        });
    }
    async fn getFut1(fut: std::pin::Pin<Box<dyn std::future::Future<Output = String> + Send + Sync + 'static>>){ // dynamic dispatch for self ref traits
        // future object as separate type must be pinned into the ram before 
        // they get moved into new scope or threads
        tokio::spawn(async move{
            fut.await;
        });
    }


    type Fut1<O> = std::pin::Pin<Box<dyn std::future::Future<Output = O> + Send + Sync + 'static>>;
    struct Futuristic<F, O>
    where F: std::future::Future<Output = O> + Send + Sync + 'static + Clone,
          O: Send + Sync + 'static + Clone{
        pub fut: F, // future trait to gets executed inside another thread directly
        pub fut1: Fut1<O>, // future as object safe trait for dynamic dispatching  
    }

    // ------------ DYNAMIC DISPATCH
    /*
        Return Type: The return type is std::pin::Pin<Box<dyn std::future::Future<Output = String>>>, which means the future is allocated on the heap and its type is erased at runtime.
        Performance: Dynamic dispatch involves a slight performance overhead due to heap allocation and vtable indirection (dynamic dispatch).
        Size and Allocation: Requires heap allocation to store the future. This can introduce overhead.
        Flexibility: More flexible as it allows returning different types of futures from the same function. The actual type of the future is not fixed at compile time.
        Use Case: Preferable when the function might return different types of futures depending on runtime conditions, and flexibility is more important than the slight performance overhead.
    */
    fn ret_fut_dynamic_dispath() -> std::pin::Pin<Box<dyn std::future::Future<Output = String>>>{
        Box::pin(
            async move{
                String::from("")
            }
        )
    }
    
    /* 
        Static Dispatch (impl Future):
            Pros: Better performance due to compile-time optimizations, no heap allocation.
            Cons: Less flexible, must return a future of the same concrete type.
            When to Use: When you need maximum performance and your function always returns the same type of future.
        
        Dynamic Dispatch (Box<dyn Future>):
            Pros: More flexible, can return different types of futures.
            Cons: Slightly worse performance due to heap allocation and dynamic dispatch.
            When to Use: When you need flexibility to return different types of futures.
    */
    let res0 = ret_fut_static_dispatch().await;
    let res1 = ret_fut_dynamic_dispath().await;
    /// -------========-------========-------========-------========-------========-------========
    /// -------========-------========-------========-------========-------========-------========
    /// -------========-------========-------========-------========-------========-------========


    // can't put traits inside RefCell or Mutex cause they need the type to be sized
    // we can put traits inside Box, Rc and Arc cause they're heap smart pointers used to points to the underlying type
    // trait can be implemented for multiple types at the same time we can store a set of trait objects which are the instances
    // of each type who implements the trait 
    // future objects as separate types need to be a pinned box on the heap

    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    struct Player<D>{
        pub nickname: String,
        pub info: D
    }
    
    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    struct PlayerInfo{
        pub rank: u8,
        pub damage: u8
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    struct RuntimeInfo{
        pub mode: u8,
        pub is_halted: bool
    }
    
    trait Extractor<D>
    where D: Send + Sync + Clone
    { // supports generic for polymorphism
        type Data; // dynamic typing
        fn extract(&mut self, data: D);
    }
    
    impl Extractor<String> for Player<PlayerInfo>{
        type Data = String;
        fn extract(&mut self, data: String){
            let decoded_data = serde_json::from_str::<PlayerInfo>(&data).unwrap();
            self.info = decoded_data;
        }
    }

    impl Extractor<RuntimeInfo> for Player<RuntimeInfo>{
        type Data = RuntimeInfo;
        fn extract(&mut self, data: RuntimeInfo){
            self.info = data;
        }
    }

    let mut player1 = Player::<RuntimeInfo>{
        nickname: String::from("some player info"),
        info: RuntimeInfo{
            mode: 1,
            is_halted: false
        }
    };
    let pmut_player1 = &mut player1;
    pmut_player1.extract(
        RuntimeInfo { mode: 1, is_halted: true }
    );
    (*pmut_player1).info.mode = 0;
    // or 
    pmut_player1.info.mode = 0;
    
    let mut player0 = Player::<PlayerInfo>{
        nickname: String::from("wildonion"),
        info: PlayerInfo{
            rank: 200,
            damage: 0
        }
    };
    let mut player = Player::<PlayerInfo>{
        nickname: String::from("wildonion"),
        info: PlayerInfo{
            rank: 100,
            damage: 0
        }
    };
    println!("player info\n {:?}", player);
    let mut pmut_player = &mut player;
    
    // mutating underlying data: same address using * and direct field accessing
    pmut_player.nickname = String::from("wildonion1");
    (*pmut_player).nickname = String::from("wildonion2");
    println!("player info\n {:?}", player);
    
    // new binding, changing address 
    pmut_player = &mut player0;
    
    pmut_player.extract(
        serde_json::to_string(
            &PlayerInfo{
                rank: 255,
                damage: 200
            }
        ).unwrap()
        .to_string()
    );
    
    println!("player info after mutation\n {:?}", pmut_player.info);
    
    struct UserRecord{
        row: String
    }
    trait UserRecordExt{
        fn get_row(&self) -> &str;
    }
    impl UserRecordExt for UserRecord{
        fn get_row(&self) -> &str {
            &self.row
        }
    }
    
    // default type param for trait GAT
    struct IterOverMe;
    trait IterateOverHim{
        type Item;
        fn next() -> Self;
        }
    impl IterateOverHim for IterOverMe{
        type Item = IterOverMe;
        fn next() -> Self {
            todo!()
        }
        }
    // creating an object from the trait requires to specify the 
    // GAT type using default type param syntax
    type Iters = Box<dyn IterateOverHim<Item = IterOverMe>>;
    // dynamic dispatch
    // building a trait object requires to specify its GAT 
    // if it has already one.
    // mutable trait objects with default type param for GAT
    // it's like specifiyng the Output in Future<Output= object
    // Output is the GAT in Future trait.
    // since extract() method takes mutable instance all the 
    // trait objects must be mutable
    let mut traits: Vec<Box<dyn Extractor<String, Data = String>>> // we have GAT and the type must be initialized
        = vec![
            Box::new(player0),
            Box::new(player)
        ];
    
    // dispatching dynamically at runtime, every Player instance 
    // needs to implements the Extractor trait
    for tobj in &mut traits{ // a mutable pointer to a trait object
        tobj.extract(
            serde_json::to_string(
                &PlayerInfo{
                    rank: 1,
                    damage: 1
                }
            ).unwrap()
            .to_string()
        );
    }

    // static dispatching
    fn get_trait_as_param(param: impl Extractor<String, Data = String>) 
        -> impl Extractor<String, Data = String>{
        let p = Player::<PlayerInfo>{
            nickname: String::from("wildonion here"),
            info: PlayerInfo { rank: 0, damage: 0 }
        };
        p // Extractor<String> trait is implemented for Player<PlayerInfo>
    }

    // ------------------------------===
    // ------------------------------===
    trait IhaveDefaultTypeParam{
        type Output;
        type Error;
        fn get_resp(&self) -> Self::Output;
    }
    struct SimpleStruct{}
    impl IhaveDefaultTypeParam for SimpleStruct{
        type Error = String;
        type Output = String;
        fn get_resp(&self) -> Self::Output { todo!() }
    }
    // vector of object traits with default type parameters
    let default_type_param_objects: Vec<Box<dyn IhaveDefaultTypeParam<Output=String, Error=String>>> = vec![];
    trait Otp{
        fn get_code(&self);
    }
    struct OtpLogin;
    impl Otp for OtpLogin{
        fn get_code(&self){}
    }
    fn set_otp_provider(param: impl Otp, callback: impl FnOnce(Box<dyn Otp>) -> String){}
    set_otp_provider(OtpLogin, |otp|{
        otp.get_code();
        return String::from("");
    });
    // ------------------------------===
    // ------------------------------===

    // -==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==
    //                  Error handler
    // -==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==
    pub struct GenericError<E: Send + Sync>{ // let it be safe to share
        e: E
    }
    /* --------------------------
        async methods or future objects capture generics and lifetimes 
        so each generic must be bounded to Send and Sync in order to 
        be shared across threads and scopes and live long enough for 
        future solvation, make sure you're pinning the boxed future 
        into the ram if you want to share it as a separate object.
        cause trait objects must be boxed to be shared and for self 
        ref data they must be pinned into the ram.
    */
    trait GenericErrorExt<B: Send + Sync, E: Send + Sync + Into<Self::Output<B>>>{
        type Output<O: Send + Sync>;
        type Fut;
        async fn getError<O: Send + Sync>(&self, e: E) -> Self::Output<O>; // the E is defined in GenericErrorExt signature 
    }
    impl<B: Send + Sync, E: Send + Sync> GenericErrorExt<B, String> for GenericError<E>{
        
        type Output<O: Send + Sync> = Box<dyn std::error::Error + Send + Sync + 'static>;
        type Fut = std::pin::Pin<Box<dyn std::future::Future<Output = String>>>;
        
        async fn getError<O: Send + Sync>(&self, e: String) -> Self::Output<O>{
            // converts this type into the (usually inferred) input type, 
            // this conversion is whatever the implementation of From<T> for U chooses to do
            e.into() // convert the String into the return type of the getError() method which a dynamic dispatch to the Error trait, Error trait is obejct safe trait
        }
    }

    // ------------------------------=== Rust 1.79
    // ------------------------------===
    /* 
        https://boats.gitlab.io/blog/post/async-methods-i/
        gats can now generic params which allows us to have async 
        method in traits cause the future returned by an async function 
        captures all lifetimes inputed into the function, in order 
        to express that lifetime relationship, the Future type needs 
        to be generic over a lifetime, so that it can capture the 
        lifetime from the self argument on the method.
        rust has async methods simply with future objects 
        they can be used to execute in a lightweight thread
        of execution without blocking the thread they're in 
        use object safe trait for dynamic dispatch with Box<dyn Trait> 
        it allows to call trait methdos on the object that impls the 
        Trait which is not known till the runtime, object safe traits 
        must be unsized! so we can't have Self and GAT in object safe traits
    */
    trait CollectionExt{
        type Collection<'valid>: Send + Sync + Clone;
        async fn get_by_owner(&self);
    }
    impl CollectionExt for (){
        type Collection<'valid> = Self;
        async fn get_by_owner(&self) {
            
        }
    }

    let name = [const{
        "wildonion"
    }; 10];

    // multi type passing support with static dispatch
    fn static_trait<T>(param: T) where T: FnOnce() -> String + Send + Sync{}
    fn static_trait1(param: impl FnOnce() -> String + Send + Sync) -> impl CollectionExt{ () }
    // trait as object with dynamic dispatch
    fn dynamic_trait(param: Box<dyn FnOnce() -> String + Send + Sync>){}
    fn dynamic_trait_self_ref(param: std::pin::Pin<Box<dyn std::future::Future<Output = String> + Send + Sync>>){}

    // lifetime extended, we can return pointer from the scope {} 
    // but still can't return pointer from function without having 
    // valid lifetime cause executing a function causes all the types
    // inside function body which are owned by the function to get 
    // executed hence dropping them out of the ram and no access to 
    // their address thus any pointer to them after executing function 
    // are invalidated and dangled. which rust doesn't allow to have 
    // them in the first place.
    let result = if true{
        &String::from("")
    } else{
        &String::from("wildonion")
    };
    
    //@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    struct User;
    pub trait Any1{} // this is need to be object safe trait for dynamic dispatch 
    pub trait Any{
        fn getValue(&self) -> Self;
    }
    impl Any for String{
        fn getValue(&self) -> String{
            
            // dereffing requires the type impls Copy trait well 
            // in here we can't since the self is behind shared ref
            // and the secondly String is not Copy we need to
            // clone or convert it to its owned version 
            self.to_owned()
        }
    }
    impl Any for u8{
        fn getValue(&self) -> u8{
            // only stack data can be derefferenced since they impl Copy trait
            // for dereffing Copy trait needs to be implemented for the type
            *self 
        }
    }
    impl Any for User{
        fn getValue(&self) -> User{
            User{}
        }
    }
    // static dispatch
    fn passAny(param: impl Any + Send + Sync + 'static){
        let value = param.getValue();
    }
    passAny(String::from("wildonion"));
    passAny(100);
    passAny(User{});

    // dynamic dispatch
    impl Any1 for String{} 
    impl Any1 for User{} 
    impl Any1 for u8{} 
    let any: Vec<Box<dyn Any1>> = vec![
        Box::new(100),
        Box::new(User{}),
        Box::new(String::from("wildonion"))
    ];

    /////// passing async future object to method
    //////@@@@@@@@@@@@@@@@@@@@@--------@@@@@@@@@@@@@@@@@@@@@--------@@@@@@@@@@@@@@@@@@@@@--------
    // future objects can be either an async method or an object safe trait using:
    // Box::pin or static dispatching with impl Future (dynamic and static dispatch) or
    // bounding to generic like so F: std::future::Future<Output=()>
    // we have to ensure the closure's return type matches the expected impl Future<Output = u8>
    // as the compiler would expect it, having so can be done in a way what i've done 
    // exactly which is passing |name: String| async move{ String::from("wildonion") }; 
    async fn writeFile<FO, G, C: FnOnce(I) -> G + Send + Sync + 'static, I, O>
        (data: &[u8], cb: fn(I) -> O, param: I, 
            cb1: C
        ) 
            where 
            G: std::future::Future<Output = String> + Send + Sync + 'static,
            I: Clone + Send + Sync + 'static, 
            O: Send + Sync + 'static, 
            O: std::future::Future<Output = FO> + Send + Sync + 'static,
            FO: Send + Sync + 'static, 
            O::Output: Send + Sync + 'static{

        let res = cb1(param.clone()).await;
        let output = cb(param).await;
    }

    let buffer = vec![];
    async fn callMeLater(name: String) -> u8{ 0 } // this method is a future object returns 0 as u8
    let cls = |name: String| async move{ String::from("wildonion") };

    writeFile(
        &buffer, 
        callMeLater, 
        String::from("wildonion"), 
        cls
    );
    //@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@


    // how multiple types can be inside a vector???? (dynamic dispatch using traits)
    trait UserInterface<V>{
        type Gat<C>: Send + Sync;
    }
    trait UserInterface2<V>{}
    struct AppUser;
    struct AppUser2;
    impl<V> UserInterface2<V> for AppUser{}
    impl<V> UserInterface2<V> for AppUser2{}
    impl<V> UserInterface2<V> for String{}
    fn getUsers<G, V, T: UserInterface<V, Gat<G>: Send + Sync>>(param: T) 
        where <T as UserInterface<V>>::Gat<G>: Clone, 
    {
        // dynamic dispatch is used to have trait objects implemented for 
        // different types, useful for dependency injection 
        let traits: Vec<Box<dyn UserInterface2<V>>> = vec![
            Box::new(
                AppUser{}
            ),
            Box::new(
                AppUser2{}
            ),
            Box::new(
                String::from("")
            )
        ];
    }
    
    fn setCollection_form1<'valid, T: CollectionExt>() 
        where <T as CollectionExt>::Collection<'valid>: Send + Sync{} // bound GAT to Send Sync indirectly
    fn setCollection_form2<'valid, T: CollectionExt<Collection<'valid>: Send + Sync>>(){} // bound GAT to Send Sync directly
    fn setCollection_form3<'valid, T: CollectionExt<Collection<'valid> = String>>() // default type param and value for GAT
        where <T as CollectionExt>::Collection<'valid>: Send + Sync{}

    fn setConstArr<T, const N: usize>() -> [Option<T>; N]{
        struct Player<const Id: usize>(pub usize);
        let foo = [const {None::<String>}; 100];
        let cnt = const{None::<String>};
        [const{None::<T>}; N]
    }
    // another const{}
    let foo = const{
        "wildonion"
    };
    let arr = const{
        [const{None::<String>}; 2]
    };
    let vector = const{
        [const{"wildonion"}; 2] // 2 elements of "wildonion"
    };
    // type Function<G: Send + Sync> = fn(String) -> G; // the bound will not be checked in type alias
    type Function<G> = fn(String) -> G;
    fn get_name(param: String) -> u32{0}
    let func: Function<u32> = get_name;
    // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>
    // dependency injection using traits poly, dyna stat dispatch
    // ===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>===>>>
    struct Time;
    impl TimeoutInterface for Time{ fn set_time(&self){} }
    trait TimeoutInterface{ fn set_time(&self); }
    fn handle_timeout<F, T: Send + Sync + 'static + TimeoutInterface, R>(timeout_logic: F, time_instance: T) 
        where F: Fn(T) -> R + Send + Sync + 'static,{
            time_instance.set_time(); // it contains &self which is a reference to the instance
            timeout_logic(time_instance);
    }

    #[derive(Default)]
    struct useState<T: Send + Sync + Clone + 'static, F: Send + Sync + Clone + 'static>(pub T, pub F) 
        where F: Fn(T) -> T + Send + Sync + Clone + 'static;
    let use_state_instance = useState(0, |age|{ 0 });
    let setAge = use_state_instance.1;
    let newAge = setAge(0);

    struct Param<T: Send + Sync + Fn() -> (), const REQUIRED: bool>{
        pub data: T,
    }

    impl<T: Send + Sync + Fn() -> ()> Param<T, true>{} // the true implementation
    impl<T: Send + Sync + Fn() -> ()> Param<T, false>{} // the false implementation

    trait Interface<G>{
        type Object<'o>: Send + Sync;
        async fn get_object<'o>(&self, param: G) -> Self::Object<'o>;
    }
    struct Object<T>{
        pub data: T
    }
    impl Interface<String> for Object<String>{
        type Object<'o> = Self;
        async fn get_object<'o>(&self, param: String) -> Self::Object<'o>{
            Object{
                data: param
            }
        }
    }
    impl Interface<String> for &[u8]{
        type Object<'o> = &'o [u8];
        async fn get_object<'o>(&self, param: String) -> Self::Object<'o>{
            // self // don't return self in here cause the lifetime of self is '1 but we need to rerturn 'o
            &[0]// returning an slice or a pointer in here is valid since we're using a valid lifetime for that which is 'o
        }
    }
    impl Interface<Vec<String>> for Vec<String>{
        type Object<'o> = Self;
        async fn get_object<'o>(&self, param: Vec<String>) -> Self::Object<'o>{
            vec![
                String::from("some-data")
            ]
        }
    }
    fn injectDependency<G>(service: impl Interface<G>){}
    let obj = Object::<String>{
        data: String::from("some-data")
    };

    obj.get_object(String::from("some-data-in-here"));
    // note that for a trait to be "object safe" it needs to allow building 
    // a vtable to allow the call to be resolvable dynamically so having GAT
    // or returning Self in return type won't make the trait an object safe trait
    let data: &[u8] = &[1]; // it must be of type &[u8]
    injectDependency(data); // service is of type &[u8] (need to behind ref cause [u8] is not Sized)
    injectDependency(vec![String::from("")]); // service is of type Vec<String>
    injectDependency(obj); // service is of type Object<String>
    trait ObjectSafeTraitInterface{
        fn getObject(&self) -> Result<(), ()>;
    }
    struct ObjectSafeTrait{}
    impl ObjectSafeTraitInterface for String{
        fn getObject(&self) -> Result<(), ()> {
            Ok(())
        }
    }
    impl ObjectSafeTraitInterface for ObjectSafeTrait{
        fn getObject(&self) -> Result<(), ()> {
            Ok(())
        }
    }
    let trait_objects: Vec<Box<dyn ObjectSafeTraitInterface>> = vec![
        Box::new(ObjectSafeTrait{}), 
        Box::new(String::from(""))
    ];
    for t in trait_objects{
        t.getObject();
    }

    trait GameObjectInterface{}
    impl GameObjectInterface for GameObject{}
    struct GameObject;

    trait AxisPointPivot<T>{
        type Object<G>: Send + Sync;
        async fn buildGameObject<G>(&mut self) -> Self::Object<G>;
        async fn returnGameObject(&self) -> impl GameObjectInterface;
    }

    impl AxisPointPivot<u8> for GameObject{
        
        type Object<String> = Self;
        async fn buildGameObject<String>(&mut self) -> Self::Object<String> {
            
            GameObject{}
        }

        async fn returnGameObject(&self) -> impl GameObjectInterface{
            
            GameObject{}
        }

    }


    /// ---=====-======-======-=-=-=-=-=-= dependency injection example
    /// ---=====-======-======-=-=-=-=-=-=
    /// ---=====-======-======-=-=-=-=-=-=
    pub trait Service<G: Send + Sync + 'static>{
        type Output;
        type Err;
        type Result;
        async fn execute(&self, state: G) -> Self::Result;
    }
    struct Dependency<T>(pub T);
    
    // Boxing trait objects for dynamic dispatch and pinning for self ref types like future objs
    // don't use dynamic dispatch if the trait is not object safe trait instead use static dispatch 
    // since G is send sync use an atomic type for thread safe syncing 
    // like atomic u8 or arc mutex or rwlock 
    impl Service<std::sync::atomic::AtomicU8> for Dependency<String>{
        type Output = Dependency<String>;
        type Err = Box<dyn std::error::Error + Send + Sync + 'static>;
        type Result = std::pin::Pin<Box<dyn std::future::Future<Output = Self::Output>>>;

        async fn execute(&self, state: std::sync::atomic::AtomicU8) -> Self::Result{
            Box::pin(async move{
                Dependency(String::from("executing..."))
            })
        }
    }
    async fn injectService(service: impl Service<std::sync::atomic::AtomicU8>){
        let state = std::sync::atomic::AtomicU8::new(0);
        service.execute(state).await;
    }
    injectService(Dependency(String::from("executing command"))).await;
    /// ---=====-======-======-=-=-=-=-=-=
    /// ---=====-======-======-=-=-=-=-=-=
    

    // executing async tasks
    async fn execute<
        F: std::future::Future<Output = String> + Send + Sync + 'static,
        B: FnOnce() -> () + Send + Sync + 'static
    >(f: F, b: B){
        tokio::spawn(f); // tokio executes the future object in the background on its own
        tokio::spawn(async move{
            b()
        });
    }
    // ------------------------------===
    // ------------------------------===

}

fn dynamic_static_dispatching2(){

    // -------------------------------------------
    // ------------|callback function|------------
    // -------------------------------------------
    /*
        Rust won't allow to have self-ref types cause they can't get moved since
        their pointer won't be updated by the Rust after moving to point to their
        new ownership thus new address therefore we should break the cycle of self-ref 
        types using smart pointers and &. use Arc to share the ownership of the 
        type between threads safely also Arc stores data on the heap, every type 
        has only one ownership so moving the type around scopes moves it out of 
        ram and change its owner thus the address so any pointer to that get invalidated, 
        we should pass it by reference or we could pin it into the ram so we can 
        use the pinned pointer which has an stable address inside the ram. 
    */
    #[derive(Clone, Default)]
    struct Context1<C>{
        pub this: C
    }
    #[derive(Clone, Default)]
    struct Config{
        pub ctx: std::sync::Arc<Context1<Self>> // use Arc to break the cycle of self-ref Config type
    }

    impl Config{
        pub fn usingRmq(&self, cls: impl FnOnce(Self, std::sync::Arc<Context1<Self>>)){
            let ctx = self.ctx.clone();
            cls(self.clone(), ctx);
        }
    }

    let cfg = Config::default();
    cfg.usingRmq(|cfg, ctx|{

        // if the type doesn't impl Copy then we should either borrow or clone
        // it when we're moving out of it or into a nother scope like ctx.this
        // which doesn't impl Copy and we should either borrow it like &ctx.this
        // or clone it like ctx.this.clone()
        let cfg_ctx = cfg.ctx;
        let cfg = ctx.this.clone();

    });
    // -------------------------------------------
    // -------------------------------------------
    // -------------------------------------------

    // trait objects must be behind pointer like & or Box, they're are dynamic sized and are abstract
    // hence they need an implementor acting them as object, requires to put them behind dyn enables us 
    // create an instance of the type that implements this trait, that instance would be the trait object.

    // vector of closure object safe traits, trait objects must be built from structs or union not enum 
    let mut closures: Vec<Box<dyn FnMut() -> () + Send + Sync + 'static>> = vec![ // since closures are FnMut we've defined this as mutable
        Box::new(||{}), Box::new(||{})
    ];
    let res = &closures[0](); // dynamic dispatching
    let cls = (|id: u8|{
        String::from("")
    })(1);

    pub struct Point{
        pub x: u8,
        pub y: u8
    }
    pub struct Circle{
        pub r: String,
        pub v: String
    }
    pub trait ObjectSafe<T: Send + Sync>{
        fn set_value(&mut self, value: T);
    }
    // -> impl Trait for static dispatch, actual type is hidden from the caller
    // param: impl Trait for static dispatch, actual type is hidden from the caller
    // T: Trait 
    // Box<dyn ---> for dynamic dipatch
    struct Traits<T: Send + Sync>{
        pub otraits: Vec<Box<dyn ObjectSafe<T>>> // vector of object safe trait with generic type for dynamic dipatching
    }
    impl ObjectSafe<Point> for Point{
        fn set_value(&mut self, value: Point) {
            self.x = value.x;
            self.y = value.y;
        }
    }
    impl ObjectSafe<Circle> for Circle{
        fn set_value(&mut self, value: Circle) {
            self.r = value.r;
            self.v = value.v;
        }
    }

    // constructing a vector of trait objects, they can be initialized 
    // withing the Box smart pointer instead of &dyn since Box has its 
    // dynamic allocation and lifetime handlers also the value of each 
    // type must be a new instance of those types who are implemented
    // the Trait
    // an object safe trait is usually the instance of the implementor
    // make sure that the trait is implemented for the struct and the trait
    // must be object safe and its must not be known for the compiler,
    // it must not have Self in its sigature cause Self referes to the 
    // implementor and object safe are used for dynamic dispatching at
    // runtime therefore there must be no specific size at compile time
    // related to the implementor
    let point_traits = Traits::<Point>{
        otraits: vec![
            Box::new(
                Point{
                    x: 0,
                    y: 0
                }
            ),
            Box::new(
                Point{
                    x: 1,
                    y: 10
                }
            ),
        ]
    };
    let circle_traits = Traits::<Circle>{
        otraits: vec![
            Box::new(
                Circle{
                    r: String::from("1"),
                    v: String::from("2")
                }
            ),
            Box::new(
                Circle{
                    r: String::from("0"),
                    v: String::from("10")
                }
            ),
        ]
    };
    for mut ct in circle_traits.otraits{
        // dynamic dispatching on every ct instance
        ct.set_value(
            Circle{
                r: String::from("20"),
                v: String::from("30")
            }
        );
    }
    for mut pt in point_traits.otraits{
        // dynamic dispatching on every pt instance
        pt.set_value(
            Point{
                x: 45,
                y: 55
            }
        );
    }

    // polymorphism and dynamic design with traits
    struct GetPoint<T>{
        pub x: T,
        pub y: T
    }
    trait InterfaceExt{
        type This;
        fn set_points(&self) -> Self::This;
    }

    impl InterfaceExt for GetPoint<String>{
        type This = GetPoint<String>;
        fn set_points(&self) -> Self::This {
            let points = self;
            Self::This{
                x: points.x.clone(),
                y: points.y.clone()
            }
        }
    }

    impl InterfaceExt for GetPoint<u8>{
        type This = GetPoint<u8>;
        fn set_points(&self) -> Self::This {
            Self::This{
                x: self.x,
                y: self.y
            }
        }
    }

    struct Math{
        pub x: u8,
        pub y: u8,
        pub add_res: String,
    }
    trait Add<T>{
        fn add(&mut self, value: T);
    }

    impl Add<String> for Math{
        fn add(&mut self, value: String) { // since it's a mutable pointer the underlying instance gets mutated too 
            self.add_res = format!("string: {}", self.x + self.y);
        }
    }

    impl Add<Math> for Math{
        fn add(&mut self, value: Math) {
            self.add_res = format!("instance: {}", value.x + value.y);
        }
    }

    // can't be object safe traits for dynamic dispatch cause `This` GAT must be specified in the treait
    // which tells the compiler that teh trait must be sized
    // let traits: Vec<Box<dyn InterfaceExt>> = vec![
    //     Box::new(
    //         GetPoint::<String>{
    //             x: String::from("1"),
    //             y: String::from("1")
    //         }
    //     ),
    // ];
 
    // trait objects for dynamic dispatching
    let traits: Vec<Box<dyn Add<String>>> = vec![
        Box::new(
            Math{
                x: 0,
                y: 0, 
                add_res: String::from("")
            }
        ),
    ];
    for mut t in traits{
        t.add(String::from("")); // dynamic dispatching 
    }


    trait OSTExt{}
    type ObjectSafeTrait0 = Box<dyn FnMut() -> () + Send + Sync>;
    type ObjectSafeTrait1 = Box<dyn OSTExt>;
    // trait to be returned as object must be object safe trait and 
    // returning as a separate object needs to box them which can be
    // used for dynamic dispatching also if the trait is a future
    // object then returning it needs to pin its boxed form into the ram 
    // so we can await on it later with its stable memory address.
    // dyn keyword means that this would be a dynamic dispatch at runtime
    // and we'll return an instance of the object who impls the trait 
    // as well as call the trait methods on that object.
    type FutureObjectPinned = std::pin::Pin<Box<dyn std::future::Future<Output = ObjectSafeTrait0>>>;
    // in order the trait be as object safe trait we should have &self 
    // as the first param in its methods since it helps to call the trait
    // methods on the object using the vtable
    trait ColExt<G>{
        type This;
        fn set_val(&mut self, val: G) -> Self::This;
    }
    struct Col{ 
        nickname: String, 
        future: FutureObjectPinned
    }
    impl ColExt<String> for Col{
        type This = Self;
        fn set_val(&mut self, val: String) -> Self::This {
            self.nickname = val;
            // NOTE: Rust don't have gc to track the number of owners in code 
            // cause everything in Rust has only one owner and due to this if 
            // a type wants to move into another scopes if it's not an stack 
            // data it'll lose its ownership and Rust allocate a new ownership
            // for that in the new scope hence we can't move types if they're behind
            // pointers, we can share their ownership using & and smart pointers
            // Rc and RefCell for single and Mutex and Arc for multi threaded ctx.
            // we can return types in here since it's behind pointer and Rust
            // doesn't allow to move out of it to prevent having dangling pointers
            // also Copy based types can move without losing ownership but 
            // Copy is not implemented for heap data and thus our structure
            // so in here we need to clone the self or: 
            // (*self) 

            // traits as object for dynamic dispatching must be boxed
            let cls: ObjectSafeTrait0 = Box::new(||{}); // explicity mention that this closure is of type ObjectSafeTrait0
            
            // use and pass by pointer and reference to prevent 
            // from moving and chaning ownership and address!
            let nickname = &self.nickname;
            Self{ nickname: nickname.to_owned(), future: Box::pin( async move{ cls } ) }
        }
    }

    
    /*                  ---------------- dynamic dispatching allows to have polymorphism ----------------
        for a trait to be "object safe" it needs to allow building a vtable to allow 
        the call to be resolvable dynamically by calling function pointer on the trait
        object using dynamic dispatch logic at runtime if it's not the case means that 
        the trait is not obejct safe trait

        since trait objs are not sized having them as object (safe of course) should behind pointer follow up with dyn keyword
        goot to know that trait objects stores two kind of pointers one is a vtable pointers points to the trait methods
        which are going to be called on the implementor instance and the other is a pointer to the underlying data or the 
        implementor, accordingly Box<dyn Trait> is an object safe trait and can be as object with dynamic dispatching at runtime 
        if we don't know the exact type of implementor enitehr the compiler, also we've used Box to store dynamic sized data on
        the heap, those types that their size are not known at compile time and it depends on some kinda implementor at runtime 

        in a programming language the generics can be handled in one of the two ways, static 
        dispatch or dynamic dispatch. In static dispatch, the various possible types of the 
        generic are inferred during the compilation and have separate assembly code blocks associated 
        with each type. This can reduce the execution time, and is the default behaviour but 
        faces the problem of what should happen if all the types cannot be inferred or we don't 
        want to generate separate code blocks for each generic. This is where dynamic dispatch 
        comes into picture, which means the type of the generic will be sent over to the runtime 
        environment in a boxed type and will be inferred during the runtime. This can be slower 
        but often provides more flexibility like Box<dyn Trait> in which the implementor will be 
        specified at runtime and only object safe trait methdos can be dynamically dispatched 
        an be as a trait object.

        in Rust, dispatch refers to the process of determining which implementation of a trait's method 
        to call when working with trait objects. There are two main types of dispatch mechanisms in Rust: 
        static dispatch and dynamic dispatch.

        Static Dispatch:

            static dispatch, also known as monomorphization, occurs at compile time, when using static dispatch, 
            the compiler knows the concrete type at compile time and can directly call the implementation of the 
            method for that type, static dispatch leads to efficient code generation as the compiler can inline 
            and optimize the method calls based on the known types, it is commonly used when the concrete type is 
            known at compile time, such as when working with generics or concrete types.
        
        Dynamic Dispatch:

            dynamic dispatch occurs at runtime and is used when the concrete type is not known until runtime, such 
            as when working with trait objects, when using dynamic dispatch, the compiler generates a vtable (virtual 
            method table) that contains pointers to the implementations of the trait methods for each type that implements 
            the trait and pointer to the struct instance, the vtable however is used at runtime to determine which implementation 
            of the method to call based on the actual type of the object, dynamic dispatch allows for flexibility and 
            polymorphism but can incur a slight runtime performance overhead compared to static dispatch.


        for dynamic dispatch calls each trait object must be a safe trait object object safe traits are trait 
        objects of type Box<dyn SafeTrait> and can be dispatch using Box::new(Struct{})
        for more info refer to: https://doc.rust-lang.org/reference/items/traits.html#object-safety
        in Rust, for a trait to support dynamic dispatch when used with trait objects, it must be an object-safe 
        trait, object safety is a property of traits that determines whether instances of the trait can be used 
        as trait objects, object-safe traits ensure that the compiler can determine the size and layout of trait 
        objects at compile time, enabling dynamic dispatch to be performed efficiently.

        Here are the key requirements for an object-safe trait in Rust:

            No Associated Functions:

                Object-safe traits cannot have associated functions (functions associated with the trait itself 
                rather than a specific implementation).
            
            No Generic Type Parameters:

                Object-safe traits cannot have generic type parameters. This is because the size of the trait 
                object needs to be known at compile time, and generic types can have varying sizes.
            
            Self-Sized Type:

                The trait cannot require that Self be a sized type. This ensures that the size of the trait object 
                is known at compile time.
            
            No Generic Type Parameters in Methods:

                Methods in object-safe traits cannot have generic type parameters, as this would make the size of 
                the trait object ambiguous.

            No Self Type in Return Position:

                Methods in object-safe traits cannot return Self by value, as this would require knowing the size
                of Self at compile time.
                
        Ensuring that a trait is object-safe allows Rust to perform dynamic dispatch efficiently when working with 
        trait objects. By adhering to the rules of object safety, the compiler can generate vtables (virtual method 
        tables) for trait objects, enabling polymorphism and dynamic dispatch without sacrificing performance or 
        safety, if a trait is not object-safe, attempting to use it with trait objects will result in a compilation 
        error. By designing object-safe traits
    */

    // polymorphism in method param
    struct User{}
    type UserName = String;
    impl Poly for UserName{
      fn execute(&self){}
    }
    impl Poly for User{
      fn execute(&self){}
    }
    trait Poly{
      fn execute(&self);
    }
    async fn pass_multiple_form(param: impl Poly){
      param.execute();
    }
    
    // polymorphism with trait, passing different types to a single method
    // it uses the concept of polymorphism with static disptach approach
    // it can be mainly used for dependency injection to pass a service into 
    // a function, with trait static dispatch we can pass multiple params 
    // with different type cause the actual type of the method param is a trait
    // and trait objects can be implemented for types that we want to pass them
    // into the method
    tokio::spawn(async move{ // execute async task in a thread
      pass_multiple_form(String::from("")).await;
      pass_multiple_form(User{}).await;
    });

    // dynamic and static dispatching with object safe trait
    trait Interface2{
        fn get_name(&self);
        }
    struct Up{}
    impl Interface2 for Up{
        fn get_name(&self){}
        }
    fn get_static(param: impl Interface2){}
    fn get_dynamic(param: &dyn Interface2){}
    fn get_dynamicB(params: Vec<Box<dyn Interface2>>){
        for p in params{
            p.get_name()
        }
        }
    get_dynamicB(vec![Box::new(Up{})]);

    trait Animal {
        fn make_sound(&self);
    }
    
    #[derive(Clone)] // make it cloneable
    struct Dog;
    impl Animal for Dog {
        fn make_sound(&self) {
            println!("Woof!");
        }
    }
    
    #[derive(Clone)] // make it cloneable
    struct Cat;
    impl Animal for Cat {
        fn make_sound(&self) {
            println!("Meow!");
        }
    }


    // object traits are safe and of type Box<dyn Trait> they allows us to do dynamic dispatch at runtime using dyn keyword 
    // since we don't know the exact type of implementor they must be safe like if the return type of one of their methods is 
    // Self it must not be bound to Sized trait cause the compiler must have no info about the type of implementor in order the
    // dyn keyword accordingly dispatching call works, Boxing them is better than putting them behind a pointer like &dyn Trait 
    // cause Box stores data directly on the heap and have valid lifetime on its own.
    // trait object stores two kina pointers the one are vtable pointers which are pointers to the trait methods that are gonna 
    // called on the instance and the other is a pointer to the instance itself, example of that would be Box<dyn Error> which
    // allows us to dispatch the methods of the trait dynamically at runtime on any struct instance that implements the Error trait.
    // following is like interface{} in Go:
    // var Trait = interface{
    //      getCode(name *string)
    // };
    // u := &User{};
    // var inter Trait; = u ---> implementing the Trait interface for u or bounding you to Trait interface
    //                           Trait interface methos can be called on u, inter is now an object interface bounded to u
    let trait_object: Box<dyn Animal> = Box::new(Dog{}); // object safe trait of dynamic type Dog{}

    let dog: Dog = Dog;
    let cat: Cat = Cat;

    // static dispatch is used when calling dog.make_sound() as the concrete type is known at compile time.
    // dynamic dispatch is used when calling animal.make_sound() on trait objects in a vector, where the 
    // actual type is determined at runtime.

    // Static dispatch
    dog.make_sound(); // compiler knows the concrete type at compile time, calling the make_sound() method directly on the instance of the dog


    // Dynamic dispatch, having a vector of trait objects
    let animals: Vec<Box<dyn Animal>> = vec![Box::new(dog), Box::new(cat)];
    for animal in animals {
        // Box is a heap wrapper around the object contains all the methods of the actual object
        animal.make_sound(); // dispatched dynamically at runtime cause we don't know what type of animal would be!
    }


    // dynamic dispatching with mutating object status during app execution
    struct Object{ status: Status}
    #[derive(Debug, Default)]
    enum Status{
        #[default]
        Alive,
        Dead
    }
    trait ObjExt{
        fn execute(&mut self); // if we had Self in return type then it couldn't be object safe trait and used for dynamic dispatch cause we have Self
    }
    let objs: Vec<Box<dyn ObjExt>> = vec![Box::new(Object{status: Status::Alive})];
    impl ObjExt for Object{
        fn execute(&mut self) {
            self.status = Status::Dead;
        }
    }
    

}

/* ▬▬ι═══════ﺤ 
|   ╰┈➤ Rust doesn't have gc it moves data around the ram or drop heap data automatically after they get moved into
|    new scope and transferred into new ownership, it also moves data around the ram usually for extra heap allocation 
|    by itself but when you decide to move it into a new ownserhisp and scope it updates the poitners but don't allow 
|    to use them after moving that's why we can't move out of a data if it's behind a shared pointer or it's pointer is
|    being used by other scopes, for self-ref types like futures it breaks the move cause it can't update pointers at all
|    forces us to use some smart pointers or & to break the cycle like Pin the boxed value into the ram to use the 
|    pinned pointer instead of the actual value cause the pinned pointer has an stable and fixed memoery address during
|    the lifetime of the app and Rust don't move it to change its address.
|    ➤⫘⫘⫘ don't move if it's behind pointer because: can't use pointer after moving and rust doesn't allow in the first place although it updates the pointers after moving 
|    ➤⫘⫘⫘ don't return reference from method because: types all are owned by the method body an once the method gets executed all of them will be dropped unless you use a valid lifetime to do so
|    ➤⫘⫘⫘ we can't generally return a pointer to a heap data owned by the function even with a valid lifetime 
|    ➤⫘⫘⫘ use pin and other smart pointers to break the cycle of self-ref types like future 
|    ➤⫘⫘⫘ use Box::pin(async move{}) to put futures in it if you want to move them around as separate
|    ➤⫘⫘⫘ moving pointer into tokio spawn requires the pointer to live longer than or equal to the tokio spawn lifetime like passing &'static str to tokio spawn is ok
|    ➤⫘⫘⫘ rust don't have gc if you use heap data without borrowing or cloning it moves out data from the ram and drop it
|    ➤⫘⫘⫘ some methods take the ownership of type cause they have self instead of &self, just clone the type or use as_ref() before calling them
| ▬▬ι═══════ﺤ 
*/
/* 
    let p = &mut object.unwrap(); here we're creating a mutable pointer to the object and since the unwrap()
    takes the ownership of the type this syntax is wrong we have to create another type which has unwrapped
    then take a mutable pointer to that, can't pass pointer to tokio spawn if the underlying type doesn't 
    liven long enough which has a shorter lifetime than the tokio spawn.
    can't use a type which doesn't live long enough to be passed into tokio scope 
    cause the type will be dropped out of the ram once the function gets executed 
    since it's owned by the function body.
    sometimes use clone instead of pointer cause pointer might not live long enough because 
    their underlying type might get dropped out of the ram and have shorter lifetime.
    everything in rust has only one ownership we can count references to a type using rc and 
    taking reference can be done via &, Box, Rc, Arc and if its behind pointers rust updates 
    them to point to the right location as soon as the ownership of the type gets transferred 
    into a new one, lifetime is a concept belongs pointer to know how much they're gonna live
    long and must be shorter or equal to their underlying data lifetime to avoid situations like 
    having dangled pointer when their underlying data is no longer accessible or valid cause 
    it has been moved or dropped out of the ram or its lifeimte has came to end. if we want
    to move one of the pointer into a new scope like tokio spawn the lifetime of the underlying 
    data must be longer than, equal to or static the lifetime of the new scope.
    only the passed in param with lifetime can be returned can't return a pointer to the local variable 
    inside function since once the function gets executed all of them will be dropped out of the ram, with 
    &self and &'staitc lifetime we can do this however. basically any type with longer lifetime than its 
    scope may involve heap allocation if their size or lifetime is not known at compile time in rust when 
    heap data pass to the function their ownership tranferred into a new one inside the function hence not 
    allowed having access to the very first ownership after method call because the resources it uses are 
    immediately freed and no longer valid with a lifetime which cause the compiler to update all its pointer 
    to point to a new location later on, doing so is due to the fact that rust tells us every value must
    have exactly one ownership specially those heap data ones unless data implements Copy trait which we can 
    pass it by value without losing ownership, references impls Copy trait, the concept of lifetime belongs 
    to pointers which tells rust how long a pointer can lives in that scope accordingly every type when they 
    go out of their scope their lifetime come to end like all the types inside this function body
*/
pub async fn accept_ref<'a>(name: &'a str) -> &'a str{


    // as soon as a type moves from its previous location inside the ram to a new one
    // its lifetime comes to end and any pointer of that get invalidated, this moves
    // can be passing it without cloning or borrowing it or getting dropped out of the
    // ram which destroy the type completely.
    // since the lifetime of binding which is an String on the heap is smaller
    // than the tokio scope (it's not lived long enough) hence we can't move 
    // its pointer (name_) into the tokio scope cause in Rust once the type gets dropped
    // out of the ram its pointer can't be usable any more and String will be dropped
    // at the end of the tokio scope spawn which is not what we want cause tokio scope 
    // executes in the background asyncly by the tokio runtime scheduler, in cases that
    // a type moves into a new scope Rust updates all its pointers (if there is any)
    // to point to the right location after moving but can't use them casue the type
    // is moved and its old lifetime is not valid any more and has a new ownership 
    // that's why we shouldn't move the type if it's behind a pointer we could pass 
    // by reference or clone it which is an anti pattern approach!
    let binding = String::from("");
    let name_ = &binding;
    tokio::spawn(async move{
        
        // let get_name = name;

    });

    struct StringUnit(pub String);
    let instance = StringUnit(String::from("wildonion"));
    let string = instance.0;

    struct Name<'valid, T: 'valid  + Send + Sync + Clone>{
        name: &'valid T
    }

    let instance = Name::<String>{
        name: &String::from("")
    };
    

    match instance{
        Name::<String>{name: esm} if !esm.is_empty() => {

        },
        _ | _ => {}
     };
    
    // ways to return pointer from method:
    // can't return pointer to heap data owned by the function
    // cause they will move and dropped out of the ram once the
    // function gets executed.
    fn ret_pointer<'p>() -> &'p str{
        let name: &'p str = "";
        name // not heap data
    }
    
    // can't return ref to data owned by the method not even with a valid lifetime!
    // due to Rust ownership and borrowing rules once the underlying type of the pointer
    // gets dropped out its lifetime comes to die then its pointers would be a dangled ones
    // even though Rust updaes them but can't use them.
    fn ret_pointer1<'v>(name: &'v String) -> &'v String{
        // we're not allocating name inside method thus name is not owned by the method and we can return its pointer
        // there is a defined lifetime for the name outside of the function scope which won't allow to have dangling 
        // pointer
        name 
    }

    // not always heap data go on the heap, types with longer lifetime than their scopes
    // will go on the heap too like having a tokio scope inside the a function body uses
    // local variables of the function: either clone them to use them later or pass their
    // borrow with a longer lifetime than their scope which is the function body to move 
    // them into the scope without losing ownership cause once the function gets executed
    // its scope will be ended and all its types lifetime come to end eventually will be 
    // dropped out of the ram to clean allocated spaces.
    fn lifetime<'v>() -> &'v [u8]{

        let bytes: &'v [u8] = &[1];
        
        // lifetime belong pointers, must be used to tell rust that the lifetime of the pointer depends on
        // its underlying type and it doesn't live longer than the underlying type cause when the type
        // gos out of the scope all its pointer must come to die to avoid having dangling poitners.
        // borrow checker is to ensure that the ref to data doesn't outlive that data that's why we can't 
        // move the pointer into a tokio spawn if the underlying doesn't live long enough this mechanism 
        // ensures that pointers are not dangled when the underlying type goes out of the scope.
        // can't move bytes into tokio scope since 'v doesn't live long enough and once the function gets 
        // executed 'v is no longer accessible, the tokio spawn on the other hand, has a longer lifetime 
        // than the function scope since it will start the task in the background until the future gets 
        // completed we can tell that due to having a longer lifetime than the function scope the tokio 
        // spawn process will go on the heap, however types with Box, Rc and Arc wrappers around them will 
        // go on the heap too, so if we want to have a type lives longer than its scope we must define a 
        // longer lifetime subsequently for that manually by taking a pointer to it with the defined lifetime.
        // tokio::spawn(async move{

        //     let b = bytes;
        // });

        // fn test_spawn(){

        //     let mut name = String::from("");
        //     let pmut = &mut name;
        //     let boxed_pmut = Box::new(pmut);
            
        //     tokio::spawn(async move{
                // mutating the underlying data of the boxed value by double derefing
        //         **boxed_pmut = String::from("updated");

        //     });

        //     println!("name is : {:?}", name);
        //     println!("boxed_pmut is : {:?}", boxed_pmut); // can't have boxed_pmut here since it's been moved into tokio spawn


        // } // name will be dropped here by calling the drop() method but its pointer is being used in tokio scope, it requires to be static

        bytes

    }

    name
}

pub async fn any_type_dyn_stat_dispatch(){

    /* 
        1. Static Dispatch (impl InterfaceForPlayer):
            Static dispatch means that the concrete type implementing the trait (User) is known at compile time.
            When you use impl InterfaceForPlayer in a function signature, it tells the compiler that the function will return 
            a type that implements the trait InterfaceForPlayer. However, the concrete type (in this case User) is known 
            during compilation, allowing the compiler to inline or monomorphize the code for performance.
            Since the compiler is generating specialized code for the type, it needs to ensure at compile time that 
            the type (User) conforms to the behavior defined in the InterfaceForPlayer trait. This allows the compiler to 
            know that the concrete type has the necessary methods and properties, which are essential to the functionality 
            of the trait. Thus, the type must implement the trait for the static dispatch case because the type-specific 
            methods need to be generated and compiled.

        2. Dynamic Dispatch (Box<dyn InterfaceForPlayer>):
            Dynamic dispatch means that the concrete type implementing the trait is not known at compile time but 
            rather at runtime. This is accomplished through vtable-based dispatching (a pointer to a table of function 
            pointers, called a vtable, is used to resolve method calls at runtime).
            When you use Box<dyn InterfaceForPlayer>, you're telling the compiler that any type that implements the trait 
            InterfaceForPlayer can be stored in the Box, but the exact type (User, for example) will be determined at runtime.
            Even though the type is determined at runtime, the trait must still be implemented by the type. This 
            is because the vtable that will be used at runtime is generated by the compiler based on the methods defined 
            in the InterfaceForPlayer trait. Without implementing the trait, the compiler has no way of ensuring that the type 
            provides the necessary methods to satisfy the contract of the trait.
            Thus, the type must implement the trait for the dynamic dispatch case as well, because the vtable needs to 
            point to the correct methods, and this vtable is built based on the trait implementation.
            
            NOTE: the trait which is being used in dynamic dispatch must be object safe trait to create vtable!


        Why Both Require the Trait to be Implemented?
            The core reason that both dynamic (Box<dyn InterfaceForPlayer>) and static (impl InterfaceForPlayer) dispatch require 
            the trait to be implemented is that traits define a set of behaviors (methods and possibly associated 
            types) that a type must provide.

        In static dispatch, the compiler needs to generate type-specific code at compile time, so it needs to know 
        that the type (User) satisfies the trait.
        In dynamic dispatch, even though the actual method resolution happens at runtime, the compiler must still 
        generate the appropriate vtable (for dynamic method lookup) based on the trait's methods. Thus, the type 
        still needs to implement the trait to ensure that the correct methods are available at runtime.
        
        Key Differences:
        
            Static Dispatch (impl InterfaceForPlayer):

                Monomorphized at compile time.
                Type is known at compile time.
                Compiler generates type-specific code (faster, more optimized).
            
            Dynamic Dispatch (Box<dyn InterfaceForPlayer>):

                Resolved at runtime using vtables.
                Type is not known until runtime.
                Slight overhead due to runtime method lookup.
            
        In both cases, however, the type must adhere to the contract defined by the trait, which is why the 
        trait must be implemented regardless of whether static or dynamic dispatch is used. This allows the
        Rust compiler to ensure type safety and that the type provides the necessary methods.
    */
    trait InterfaceForPlayer{}
    struct User{}
    impl InterfaceForPlayer for User{}
    fn getUser() -> Box<dyn InterfaceForPlayer>{
        Box::new(User{})
    }
    fn getUser1() -> impl InterfaceForPlayer{
        User{}
    }


    // spawn a tokio thread for every request in a lightweight
    async fn getCode<O: Send + Sync + 'static>(param: O) 
        -> impl std::future::Future<Output=O> + Send + Sync + 'static{
        // the return type is a type which impls the trait directly through 
        // static dispatch
        async move{
            param
        }
    }
    tokio::spawn(getCode(String::from("wildonion")));

    /* 
        Access different types through a single interface to use common method of traits with either default 
        or trait implementations we can impl the trait broadly for any possible types using impl Trair for T{} 
        instead of implementing for every single type manually box pin, box dyn trait impl trait for dyn stat 
        and poly implementations.
        is, the type has been erased. As such, a dyn Trait reference contains two pointers. One pointer goes 
        to the data (e.g., an instance of a struct). Another pointer goes to a map of method call names to 
        function pointers (known as a virtual method table or vtable).
        At run-time, when a method needs to be called on the dyn Trait, the vtable is consulted to get the 
        function pointer and then that function pointer is called.
        See the Reference for more information on trait objects and object safety.
        Trade-offs
        The above indirection is the additional runtime cost of calling a function on a dyn Trait. Methods 
        called by dynamic dispatch generally cannot be inlined by the compiler.
        However, dyn Trait is likely to produce smaller code than impl Trait / generic parameters as the 
        method won't be duplicated for each concrete type.
    */
    trait AnyTypeCanBe1<T>: Send + Sync + 'static{
        fn getNickName(&self) -> String{
            String::from("")
        }
    }
    impl<T: Send + Sync + 'static> AnyTypeCanBe1<T> for T{}
    struct InGamePlayer{}
    let player = InGamePlayer{};
    player.getNickName(); // don't need to impl AnyTypeCanBe1 for InGamePlayer cause it's already implemented for any T

    
    // handling pushing into the map using trait polymorphism
    trait AnyTypeCanBe{}
    impl<T> AnyTypeCanBe for T{} // impl AnyTypeCanBe for every T, reduces the time of implementing trait
    let any_map1: std::collections::HashMap<String, Box<dyn AnyTypeCanBe + Send + Sync + 'static>>;
    let mut any_map1 = std::collections::HashMap::new();
    any_map1.insert(String::from("wildonion"), Box::new(0));
    // or 
    // any_map1.insert(String::from("wildonion"), Box::new(String::from("")));

    // to have any types we can dynamically dispatch the Any trait which is an object safe trait
    type AnyType = Box<dyn std::any::Any + Send + Sync + 'static>;
    let any_map: std::collections::HashMap<String, AnyType>; // the value can be any type impls the Any trait
    let boxed_trait_object: Box<dyn AnyTypeCanBe>; // Boxed trait object
    let arced_trait_object: std::sync::Arc<dyn AnyTypeCanBe>; // thread safe trait object

    fn getTrait(t: &(dyn AnyTypeCanBe + Send)){ // dynamic dispatch

    }
    fn getTrait1(t: impl AnyTypeCanBe + Send){ // static dispatch
        
    }


}

pub async fn solid_design_pattern(){
    
    /* ---------------------------------------------
        return future from none async context when we need its result:
            put future inside Box::pin and use channels to return the result from the tokio spawn threads 
            later the caller of the function must await on it later but the logic inside the async context 
            will be executed already
        traits are interfaces that can be implemented for any types mainly allows 
        extending and accessing all types through a single interface also we could 
        store them as obejct safe trait through dynamic dispatching process 
        with &dyn Trait or smart pointers like Arc<dyn Trait or Box<dyn Trait, this 
        is called dep injection, they can also be used for static dispatch which supports 
        passing multiple types to a function another usefull logic is supporting 
        polymorphism through a single interface

        a nice abstract and solid based codes:
        traits are all about extending interface of struct and 
        designing the real world absatract problems which don't 
        need to be implemented directly on the object itself. 
        dynamic dispatch: the trait must be object safe trait and since traits are not sized hence it would be: Box<dyn Trait> as the separate type
        static dispatch : the trait must be implemented directly for the type and we're returning the trait using `impl Trait` syntax
        polymorphism    : passing different types to trait to support different implementation through a single interface
        dep injection   : combine all of the aboves
        
        pass different types to method through a single interface using trait 
        handling dep injection using Box<dyn and Arc<Mutex<dyn
        trait for stat dyn disptach and dep injection and polymorphism (pass multiple type to a method)

        task is an io future job of one of the following types
        since futures are object safe trait hence they have all traits 
        features we can pass them to the functions in an static or dynamic 
        dispatch way using Arc or Box or impl Future or event as the return 
        type of a closure trait method:
            returning reference or box to dyn trait by casting the type who impls the trait into the trait 
            dep injection object safe trait using & and smart pointers dyn
            future as generic in return type of closure or function or pinning its box
            std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>
            Arc<dyn Fn() -> R + Send + Sync + 'static> where R: std::future::Future<Output = ()> + Send + Sync + 'static
            Box<dyn Fn() -> R + Send + Sync + 'static> where R: std::future::Future<Output = ()> + Send + Sync + 'static
            Arc<Mutex<dyn Fn() -> R + Send + Sync + 'static>> where R: std::future::Future<Output = ()> + Send + Sync + 'static
            F: std::future::Future<Output = ()> + Send + Sync + 'static
            param: impl std::future::Future<Output = ()> + Send + Sync + 'static
        
        NOTE: mutex requires the type to be Sized and since traits are 
        not sized at compile time we should annotate them with dyn keyword
        and put them behind a pointer with valid lifetime or Box and Arc smart pointers
        so for the mutexed_job we must wrap the whole mutex inside an Arc or annotate it
        with something like &'valid tokio::sync::Mutex<dyn Fn() -> R + Send + Sync + 'static>
        the reason is that Mutex is a guard and not an smart pointer which can hanlde 
        an automatic pointer with lifetime 
    */

    // use Box<dyn AnyType> for dynamic typing and dispatch
    // use impl AnyType for static typing and dispatch 
    // use trait for polymorphism like wallet payment portal
    // pass Box<dyn AnyType in struct for dep injection  
    // use Box::pin() to pin the future trait objects into the ram
    trait ServiceExt{
        fn getInstance(&self) -> &Box<dyn ServiceExt>;
    }
    struct Service{
        pub instance: Box<dyn ServiceExt> // injecting ServiceExt trait dependency 
    }
    impl ServiceExt for Service{
        fn getInstance(&self) -> &Box<dyn ServiceExt> {
            &self.instance
        }
    }
    fn getService(service: impl ServiceExt){
        let instance1 = service.getInstance();
        let instance2 = instance1.getInstance();
        instance2.getInstance();
    }
    
    enum PortalError{
        Success,
        Failed
    }

    // buffer transaction events
    struct Buffer<T>{
        pub events: std::sync::Arc<std::sync::Mutex<Vec<T>>>,
        pub size: usize
    }
    struct Event;
    struct Transaction{
        pub events: Buffer<Event>,
    }

    // the ZarinPalPortal struct
    struct ZarinPalPortal;

    // account structure for ZarinPalPortal gateway
    struct Account<ZarinPalPortal>{
        pub txes: Vec<Transaction>,
        pub payment_portals: Vec<Box<dyn Portal<ZarinPalPortal, Gate = String>>>, // dynamic dispatch, will be triggered at runtime | default type param for the GAT
    }

    // main struct: wallet which contains different accounts
    struct Wallet<P>{
        pub accounts: Vec<Account<P>>,
    }

    // portal trait
    trait Portal<P>{ // polymorphism, access different types through a single interface
        type Gate: Send + Sync + 'static;
        fn pay(&self, portal: P) -> Result<(), PortalError>;
    }
    
    // wallet must support different portals
    impl Portal<ZarinPalPortal> for Wallet<ZarinPalPortal>{
        type Gate = String;
        fn pay(&self, portal: ZarinPalPortal) -> Result<(), PortalError> {
            Ok(())
        }
    }

    trait Interface12: std::fmt::Debug{}
    struct DynamicDisptachDepInjection<'valid>{
        pub dep: Option<&'valid dyn Interface12>,
    }
    #[derive(Debug)]
    struct UseMe{}
    impl Interface12 for UseMe{}
    let depInjection = &UseMe{} as &dyn Interface12; // cast the instance to the trait
    let instance = DynamicDisptachDepInjection{
        dep: Some(depInjection)
    };
    if let Some(interface) = instance.dep{
        println!("interface: {:#?}", interface);
    }

    #[derive(Clone)]
    pub struct setState<T>(pub T);
    
    pub trait useState<G>{
        type StateType<S: Send + Sync + 'static>;
        fn getState(&self) -> G;
    }
    impl<String: Clone> useState<String> for setState<String>{
        type StateType<S: Send + Sync + 'static> = String;
        fn getState(&self) -> String {
            self.clone().0
        }
    }
    let state = setState(String::from("wildonion"));
    let value = state.getState();

    struct useRef<'valid>(pub &'valid [u8]);
    let bytes = useRef(String::from("wildonion").as_bytes());


    // dependency injection using Arc<dyn Trait, Box<dyn Trait, Arc<Mutex<dyn Trait
    type Fut = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>;
    struct Contract<R: std::future::Future<Output = ()> + Send + Sync + 'static>{
        pub mintFunctions: std::sync::Arc<tokio::sync::Mutex<Vec<Box<dyn Fn() -> R + Send + Sync>>>>,
        pub futures: std::sync::Arc<tokio::sync::Mutex<Vec<Fut>>>,
        pub eventloops: Vec<std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Fut>>>>
    }
    impl<R: std::future::Future<Output = ()> + Send + Sync + 'static> Contract<R>{
        pub async fn executeMintFunctions(&self){
            let getFuncs = self.mintFunctions.lock().await;
            for func in getFuncs.iter(){
                func().await;
            }
        }
    }

    // Arc<dyn Future>
    // Arc<Mutex<dyn Future>>
    // param: impl Future
    // Pin<Box<dyn Future>>
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::pin::Pin;
    struct DepInjectionSubscribers<T, R, O>
    where 
        O: Clone + Send + Sync + 'static,
        T: Clone + Send + Sync + 'static,
        R: std::future::Future<Output = O> + Send + Sync + 'static{
        pub boxed_subs: Arc<Mutex<Vec<Box<dyn Fn(T) -> R + Send + Sync + 'static>>>>,
        pub arced_subs: Arc<Mutex<Vec<Arc<dyn Fn(T) -> R + Send + Sync + 'static>>>>,
        pub pinned_fut_io_task: Pin<Box<dyn std::future::Future<Output = O> + Send + Sync + 'static>> 
    }
    impl<T, R, O> DepInjectionSubscribers<T, R, O> 
    where 
    O: Clone + Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
    R: std::future::Future<Output = O> + Send + Sync + 'static{
        pub fn set(&mut self, fut: impl std::future::Future<Output = O> + Send + Sync + 'static){
            self.pinned_fut_io_task = Box::pin(fut);   
        }
        pub fn get(&self) -> &Self{
            &self
        }
    }

    //// ------- ///// ------- ///// ------- ///// 
    type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;
    type ArcedError = std::sync::Arc<dyn std::error::Error + Send + Sync + 'static>;
    type Function<R> = Box<dyn Fn() -> R + Send + Sync + 'static>; 
    async fn getFunc<O, R: std::future::Future<Output = O>>(function: Function<R>) -> O
        where R: Send + Sync + 'static,
                O: Send + Sync + 'static{
            let res = function().await;
            res
        }
    type Function1<T, R> = std::sync::Arc<tokio::sync::Mutex<dyn FnOnce(T) -> R + Send + Sync + 'static>>;
    fn buildClosure<T, R>(
        param: T, fut: R,
        fut1: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send  + Sync + 'static>>, 
    ) 
    -> Function1<T, R>
    where T: Clone + Send + Sync, 
          R: std::future::Future<Output = ()> + Send  + Sync + 'static,
    {
        // R would expect the closure body to be future object
        std::sync::Arc::new(
            tokio::sync::Mutex::new(
                |param| { fut }
            )
        )
    }
    
    // ---=====---=====---=====---=====---=====---=====---=====---=====
    // ---=====---=====---=====---=====---=====---=====---=====---=====

    trait BuilderExt<T>{
        type Builder;
        fn build(&self, instance: T) -> std::io::Result<()>;
    } 

    // dependency injection,
    // don't return Self in trait for dynamic dispatch
    // default type param for GAT
    // static dispatch
    pub struct Context<A>{
        pub builder: Box<dyn BuilderExt<A, Builder = A>> 
    }

    pub struct Account1{
        pub owner: String,  
    }
    impl Account1{
        pub fn transfer(&mut self, token: u64, to: String) -> Self{
            // fetching heap data fields moves out the self so we should either use & or call clone on the self
            // cause we are not allowed to move out of the self since it's behind &
            let owner = &self.owner; 
            todo!()
        }
    }
    pub struct Builder;

    impl BuilderExt<Account1> for Builder{
        type Builder = Account1;
        fn build(&self, mut instance: Account1) -> std::io::Result<()> {
            instance.transfer(10, String::from("0x02"));
            Ok(())
        }
    }

    let account = Account1{owner: String::from("0x00")};
    let ctx = Context::<Account1>{builder: Box::new(Builder)};


    // ********************************
    // dependency injecton example
    // ********************************
    trait AnyTypeCanBe{}
    trait ObjectSafe{
        fn getObject(&self);
    }
    struct PlayerInfo{}
    impl AnyTypeCanBe for String{}
    impl AnyTypeCanBe for PlayerInfo{}
    impl ObjectSafe for PlayerInfo{
        fn getObject(&self) {
            
        }
    }
    fn getInfo(param: impl AnyTypeCanBe) -> Box<dyn ObjectSafe>{
        // future as separate object their pointer needs to gets pinned into the ram 
        // cause they will get moved around different parts of the ram and their address
        // will be changed thus in order to track them later for solvation we need to pin them
        let fut = Box::pin(async move{});
        Box::new(PlayerInfo{})
    }
    fn getInfoTraitLifetime<'valid>(param: &'valid dyn AnyTypeCanBe){} // dyn keyword on its own requires a valid lifetime to be behind
    fn getInfoTraitLifetime1<'valid>(param: Box<dyn AnyTypeCanBe>){} // box handles the lifetime and allocation automatically
    
    // cast it to trait so we can pass it to the method
    // the AnyTypeCanBe is implemented for String so 
    // we can cast the string into that trait
    let string_type = &String::from("") as &dyn AnyTypeCanBe; 
    getInfoTraitLifetime(string_type);
    getInfo(String::from(""));
    getInfo(PlayerInfo{});
    // ************************************************************


    trait Interface{ type Output; }
    // dep injection with future object must be a pinned box
    // future is a trait and trait is not Sized thus needs to behind 
    // a pointer the best option would be Box due to having handling lifetime and allocation
    // automatically feature, this would gives a nice dependency injection feature as well
    // also it needs to gets pinned into the ram, trait objects are heap data 
    // they get moved by the Rust often around different location of the ram
    // we need to keep them fixed at an stable location cause we need 
    // to use them later in different scope to wait on them hence to address this 
    // issue pinning the pointer (Box) of future object is the best option.
    type FuturePinned = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>;
    
    // dep injection with other traits than future
    type AnyThingExceptFutureObject = Box<dyn Interface<Output = ()> + Send + Sync + 'static>;
    // bound generic to future trait 
    type GenericType<G> = PlayerInfo1<G>; // bounding won't be checked in here type aliases
    struct PlayerInfo1<G: std::future::Future<Output = ()> + Send + Sync + 'static>{ data: G }

    trait GetPointee{
        fn getVal(&self) -> Self;
    }
    impl<T: Clone> GetPointee for T{
        fn getVal(&self) -> Self{ // if the implementor was &T the self would be &T
            self.clone()
        }
    }

    let pointee = String::from("");
    let pointer = &pointee;
    let pointee_val = pointer.getVal();

    // trait can be used to convert an object or a thing to something that 
    // can have an extra ability or behaviour like converting an array of 
    // data into iterator to do something like iterating over its elements 
    // ex: Iterator trait can be implemented for any object that can be used
    // to call next() method on it:
    fn indices<'s, T>( 
        slice: &'s [T] 
    ) -> impl Iterator<Item = usize>{ // impl Iterator which its next item is a usize
        // the usize implements the Iterator trait by default 
        // so returning it at compile time is ok 
        0..slice.len()
    }
    let mut data = vec![1, 2, 4];
    let mut getSize = indices(&data);
    data.push(5);
    let nextSize = getSize.next();

    trait Interface45{
        type Output;
        fn getOutput(&self) -> &Self::Output; // use the lifetime of the self to return poitner
    }
    struct Player{name: String}
    impl Interface45 for Player{
        type Output = String;
        fn getOutput(&self) -> &Self::Output { // use the lifetime of the self to return poitner
            &self.name
        }
    }
    fn getOutputOfType<'lifeimte>(player: Player) -> impl Interface45<Output = String> + 'lifeimte{
        let name = player.getOutput(); // getOutput() method borrows the instance with &self, allows us to use the instance later
        player // the lifetime of player is last as long as 'valid
    }
    
}

// generator are dangerous in rust since they return a value to the caller without waiting 
// for function execution to completion and since after function execution all values get
// dropped out of the ram thus generator can't be safe in this situation

/*

     
           --------------------------------------------------
                             Mutex Vs RwLock
           --------------------------------------------------


    Mutex (Mutual Exclusion Lock):

    A Mutex allows only one thread to access some data at any given time.
    Every time a thread wants to access the data, it must first lock the Mutex.
    If you have a situation where you have more writes than reads, or if the 
    reads and writes are roughly equal, a Mutex is usually the better choice.
    tokio::sync::Mutex is an asynchronous Mutex that is designed to work 
    with async code within tokio.
    
    RWLock (Read-Write Lock):

    A RWLock allows any number of threads to read the data if there isn't a thread writing to it.
    If a thread wants to write to the data, it must wait for all the readers to finish before it 
    can obtain the lock. If you have a situation where you have many more reads than writes, a RWLock 
    can be more efficient because it allows multiple readers to access the data simultaneously.
    tokio::sync::RwLock is an asynchronous Read-Write Lock that works within the async ecosystem of tokio.
    
    When to choose tokio::sync::Mutex:

    You have frequent writes.
    The critical section (the part of the code that needs exclusive access to the data) is quick.
    You want simplicity. Using a Mutex is straightforward and avoids the complexity of dealing with 
    multiple lock types.
    
    When to choose tokio::sync::RwLock:

    You have many more reads than writes.
    You want to allow concurrent reads for efficiency.
    The critical section for reads is fast, but it’s still beneficial to have multiple readers at the same time.
    In many scenarios, a Mutex might be sufficient and can be the simpler choice, especially if write operations 
    are as common as reads or if the critical section is very short, thus not justifying the overhead of managing 
    a RWLock.

    However, if your specific case involves a lot of concurrent reads with infrequent writes and the read operations 
    are substantial enough to create a bottleneck, a RWLock might be a better choice.

    
           --------------------------------------------------
                     a thread safe global objects
           --------------------------------------------------


    single thread and none gc concepts: ltg &mut pointer (rc,refcell,arc,mutex,lazy,threadlocal),box,pin,impl Trait,

    code order execution and synchronization in multithreaded based envs like
    actor worker like having static lazy arced mutex data without having deadlocks 
    and race conditions using std::sync tokio::sync objects like 
    semaphore,arc,mutex,rwlock,mpsc also data collision, memory corruption, deadlocks 
    and race conditions avoidance in async and multithreaded contexts are: 
        - share none global app state data between tokio::spawn() threads using mpsc 
        - enum and actor id as unique storage key to avoid data collision
        - mutate a global storage using thread local in single-threaded contexts
        - mutate a gloabl storage using static lazy arc mutexed in multi-threaded contexts
        - can't move out of a reference or deref a type if its pointer is being used by and shared with other scopes
        - can't mutate data without acquiring the lock of the mutex in other threads
        - can't have both mutable and immutable pointers at the same time if multiple immutable ones is in there can't have mutable one and also only one mutable in each scope (multi producer and single consumer concept)
        - can't move a pointer of a type into a tokio::spawn() scope if the pointer is being used after the scope cause its lifetime gets dropped from the ram
        - can't move data into a new scope if it's behind a pointer, we can move it but if it's behind a pointer the pointer can't be accessible and gets dangled

    reasons rust don't have static global types:
        
        Memory Safety: One of Rust's main goals is to ensure memory safety without the need 
               for a garbage collector. Global state can lead to shared mutable state across 
               threads, which is a source of data races. By making global state explicit and 
               synchronized, Rust avoids these issues.

        Concurrency: Rust's concurrency model revolves around the concept of ownership. Global 
               variables can be problematic in concurrent programs, where multiple threads might 
                want to modify a global variable simultaneously.

        Predictability and Explicitness: Global mutable state can make programs unpredictable 
                and hard to reason about. Rust values explicitness over implicitness, so when you 
                see a piece of Rust code, you can easily understand its behavior without having to 
                consider hidden global states.

        Lifetimes: Rust uses lifetimes to track how long data is valid. Global state has a complex 
                lifetime that can easily lead to dangling references if not managed carefully.
                with this in mind there is no need to define a global mutexed response object
                and reload it everytime in each api to avoid runtime memory overloading, cause rust
                will handle this automatically by using the concept of borrowing and lifetime

        No Garbage Collector: While the presence or absence of a garbage collector (GC) isn't the 
                main reason Rust is cautious with global state, it's worth noting. Many languages 
                with GCs allow for more liberal use of global state because the GC can clean up. 
                In Rust, manual memory management means you need to be more careful.

    heap data need to be behind pointer or their slice form, for traits they must be behind Box<dyn or 
    &dyn, for String and Vec need to be &str and &[], good to know that if we want to return pointer to 
    heap data from method they must be in their slice form with a valid lifetime like pointer to closure 
    traits as method param and return type, if the heap data is one of the field of a structure it's ok 
    to return a pointer with a valid lifetime to that cause the pointer will be valid as long as the &self 
    is valid and once we move self into a new scope like tokio::spawn() we can't do this although rust won't
    allow move self to new scope in the first place, in general we shouldn't return pointer to type from 
    method since rust handled each type lifetime automatically unless it's a mutable pointer cause mutating 
    a mutable pointer will mutate the actual type

    global state of type requires to have a complex valid lifetime like 'static and be mutable which this can't 
    be happend since rust doesn't have gc logic to track the lifetime of the type based on the references to that 
    type although it has Rc and Weak which can be used to count the references of a type but instead it uses the 
    concept of borrowing and ownership which is about destroying the type and drop its lifetime from the ram once 
    the type goes out of the scope like by moving heap data types into a function scope in essence by mutating an 
    static lifetime type we may face deadlock and race conditions issues in other threads, instead we can define 
    an static mutex since static types are immutable by default and because static values must be constant we must 
    put the mutex inside Lazy, like the following:
    since we can't return none const from a static type thus we have to 
    put it inside the lazy as a closure which returns the actual type 
    because Arc and RwLock are none const types although we can implement 
    this logic using thread_local!{},

    note that the data we want to share it between threads must be Send + Sync + 'static
    eg: Lazy<std::sync::Arc<tokio::sync::RwLock<ZoomateResponse>>> + Send + Sync + 'static 
    as a mutable global data will be shared between apis to mutate it safely to avoid deadlocks 
    and race conditions and the sharing process can be done using mpsc jobq channel sender
    so having the following is wrong since the static value must be const and Arc and RwLock
    are none const types hence we must put them inside Lazy<>: 
        pub static MULTI_THREAD_THINGS: std::sync::Arc<tokio::sync::RwLock<Vec<u8>>> = 
            std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new()));
    is wrong and we should use the following syntaxes instead:
*/

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// s3 code order execution using sync objects: 
// static lazy arced mutexed and pinned box future db type, send sync static
// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// static value must be const since we are not able to mutate its value because it's not safe
// however sharing data between threads safely requires to borrow the data to share its ownership
// using Arc and for mutation using Mutex, since these types are none const types we can use Lazy
// to make them const so we can give the static type this value.
type DbS3Type = Lazy<std::sync::Arc<tokio::sync::Mutex<
    std::pin::Pin<Box<dyn futures::Future<Output = HashMap<String, String>> + Send + Sync + 'static>>
    >>>;
pub static DbS3: DbS3Type = 
Lazy::new(||{
    std::sync::Arc::new(
        tokio::sync::Mutex::new(
            Box::pin(async move{ // pinning the future object into the ram before polling it to make it as a separate object type for future solvation
                HashMap::new()
            })
        )
    )
});

// following is incorrect since std::sync::Arc<tokio::sync::RwLock<Lazy<String>>>
// is not constant and the whole type must be wrapped into the Lazy<> to be a const
// value acceptable by the static cause static value must be const
/* 
pub static MUTABLE_USER_RATELIMIT: std::sync::Arc<tokio::sync::RwLock<Lazy<String>>> = 
    std::sync::Arc::new(
        tokio::sync::RwLock::new(
            /* 
                since USER_RATELIMIT is not constant we can't have it here because
                static types must have constant values
            */
            Lazy::new(||{String::from("")})
        )
    );
*/

/*
    these are areas of heap memory that are reserved for a given thread and are used only by that thread 
    to allocate memory: By working in this way, no synchronization is necessary since only a single 
    thread can pull from this buffer so: 
    thread_local!{} can be used as a global arena storage in single thread context
*/
thread_local! {
    /* 
        a mutable single threaded local storage that can be mutated using Cell
        which is a mutable memory location by calling the set() method on
        SINGLE_THREAD_THINGS_CELL
    */
    pub static SINGLE_THREAD_THINGS_CELL: std::cell::Cell<Vec<u8>> = const {std::cell::Cell::new(Vec::new())};
    /* 
        a mutable single threaded local storage that can be mutated using RefCell
        which is a mutable memory location with dynamically checked borrow rules 
        by calling the set() method on SINGLE_THREAD_THINGS_REFCELL
    */
    pub static SINGLE_THREAD_THINGS_REFCELL: std::cell::RefCell<Vec<u8>> = const {std::cell::RefCell::new(Vec::new())};

}

// a single thread arena allocator
thread_local!{

    pub static DB: std::cell::RefCell<std::collections::HashMap<String, String>> = 
        std::cell::RefCell::new(HashMap::new());
}
// DB.with_borrow_mut(|db| {
//     db.insert("key".to_string(), "value".to_string())
// });

fn local_storage_ex(){

    fn func() -> i32{
        0
     }
    let funcpointer = func as *const (); // () is function in c

    // set() sets or initializes the contained value unlike the other methods, 
    // this will not run the lazy initializer of the thread local. instead, 
    // it will be directly initialized with the given value if it wasn't 
    // initialized yet.
    SINGLE_THREAD_THINGS_CELL.set(vec![1]);
    SINGLE_THREAD_THINGS_CELL.set(vec![2]);

    SINGLE_THREAD_THINGS_REFCELL.with_borrow_mut(|v: &mut Vec<u8>| v.push(3));
    // Calling SINGLE_THREAD_THINGS_REFCELL.with() here would result in a panic
    // since with() will lazily initialize the value if this thread has not 
    // referenced this key yet also This function will panic!() if the key currently 
    // has its destructor running, and it may panic if the destructor has previously 
    // been run for this thread.
    SINGLE_THREAD_THINGS_REFCELL.with(|v| {
        *v.borrow_mut() = vec![0];
    });
    // but SINGLE_THREAD_THINGS_REFCELL.set() is fine, as it skips the initializer
    SINGLE_THREAD_THINGS_REFCELL.set(vec![4]);

}


pub fn init_vm(){

    let datarefcell: std::rc::Rc<RefCell<&'static [u8; 64]>> = std::rc::Rc::new(RefCell::new(&[0u8; 64]));
    let lam = **datarefcell.borrow_mut(); //// double dereference to get the [0u8l 64] which has 64 bytes data 

    #[derive(Debug, Clone)]
    enum Chip{
        Intel{version: String},
        M1
    }
    let cmd = Chip::Intel{version:"wildonion".to_string()};
    let Chip::Intel{version: esm} = cmd else{
        panic!("no");
    };

    struct Runtime;
    trait RuntimeExt{}
    struct ByteCode<'b>{
        pub bytes: &'b [u8]
    };
    struct VirtualMachine<'Exectuor, 'b, Runtime: Send + Sync + 'static, const SIZE: usize>
        where Runtime: RuntimeExt,
        ByteCode<'b>: Send + Sync + 'static{
       pub rt: &'Exectuor Runtime,
       pub bytecodes: &'b [ByteCode<'b>; SIZE]
    }
    
    #[derive(Debug, Clone)]
    struct Executor;
    #[derive(Debug, Clone)]
    enum Cost{
        Runtime{executor: Executor},
        Compiletime,
    }
    let cost = Cost::Runtime { executor: Executor };
    match cost{
        Cost::Runtime { executor: executor_instance } => {
            let ext = executor_instance;
            todo!()
        },
        Cost::Compiletime =>{
            todo!()
        }
        _ => {
            todo!()
        }
    }

    enum Enum{
        Struct{
            name: i32
        },
        Int,
        IntU8(u8)
    }
    
    let enumtor = Enum::Int;
    let res = match enumtor{
        Enum::Struct{name: esm} if esm == 0 => {
            todo!()
        },
        Enum::Int | Enum::IntU8(0) => {
            todo!()
        },
        Enum::IntU8(num) if num > 10 => {
            todo!()
        }
        _ | _ => todo!()
    };

}


pub async fn neuron_actor_cruft(){

    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    // it's better the result of a future would be nothing 
    // and use channel to move its result around different scopes
    struct Job00;
    struct Task00
        (pub std::pin::Pin<Box<dyn std::future::Future<Output=()> + Send + Sync + 'static>>);
    let task = Task00(
        Box::pin(async move{
            tx.send(Job00).await;
        })
    );
    
    tokio::spawn(async move{
        task.run().await;
    });


    tokio::spawn(async move{
        while let Some(job) = rx.recv().await{
            // once the task gets executed we'll 
            // receive its output and result in here
        }
    });
    
    impl Task00{
        pub async fn run(self){
            let t = self.0;
            t.await;
        }
    }
    
    // --------------------------------------------
    // --------------------------------------------

    fn retFut() -> impl std::future::Future<Output = ()> + Send + Sync{
        async move{}
    }
    // it's better to pin the future if we want to return it as a dynamic dispatch 
    fn retFut1() -> std::pin::Pin<Arc<dyn std::future::Future<Output = ()> + Send + Sync>>{
        std::sync::Arc::pin(async move{})
    }
    // can't return future as a generic we should return a type that is already generic 
    // like the return type of a closure
    async fn retFut2<F: Fn() -> R + Send + Sync, R>(param: Arc<F>) -> R 
    where R: std::future::Future<Output = ()> + Send + Sync{
        let task = param();
        task
    }
    type FutOut = std::pin::Pin<Arc<dyn std::future::Future<Output = ()> + Send + Sync >>;
    fn retFut3<F: Fn() -> FutOut + Send + Sync>(param: Arc<F>) -> FutOut{
        let task = param();
        task
    }
    
    trait Interface0{}
    struct Job1{}
    impl Interface0 for Job1{}

    // arcing like boxing the object safe trait use for thread safe dynamic dispatching and dep injection
    let interfaces: std::sync::Arc<dyn Interface0> = std::sync::Arc::new(Job1{});

    // boxing trait for dynamic dispatching and dep ibjection
    let interfaces1: Box<dyn Interface0> = Box::new(Job1{});

    // arcing the mutexed object safe trait for dynamic dispatching, dep injection and mutating it safely
    let interfaces2: std::sync::Arc<tokio::sync::Mutex<dyn Interface0>> = std::sync::Arc::new(tokio::sync::Mutex::new(Job1{}));

    // we can't have the following cause in order to pin a type the size must be known
    // at compile time thus pin can't have an object safe trait for pinning it at stable 
    // position inside the ram without knowing the size 
    // let interfaces3: std::sync::Arc<std::pin::Pin<dyn Interface>> = std::sync::Arc::pin(Job{});
    // we can't pin a trait directly into the ram, instead we must pin their
    // boxed or arced version like Arc<dyn Future> or Box<dyn Future> into 
    // the ram: Arc::pin(async move{}) or Box::pin(async move{})

    // job however is an async io task which is a future object
    type IoJob<R> = Arc<dyn Fn() -> R + Send + Sync + 'static>;
    type IoJob1<R> = std::pin::Pin<Arc<dyn Fn() -> R + Send + Sync + 'static>>;
    type IoJob2 = std::pin::Pin<Arc<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>;
    type IoJob3 = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>;
    type Tasks = Vec<std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>>;
    type Task = Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>;
    type Task1 = std::sync::Arc<dyn FnOnce() 
        -> std::pin::Pin<std::sync::Arc<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>>;
    type Task2 = std::sync::Arc<dyn FnOnce() 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>>;
    // dependency injection and dynamic dispatching is done by:
    // Arc::pin(async move{}) or Box::pin(async move{})
    // Pin<Arc<dyn Trait>> or Pin<Box<dyn Trait>>
    // R: Future<Output=()> + Send + Sync + 'static
    use std::{pin::Pin, sync::Arc};
    type Callback<R> = Arc<dyn Fn() -> Pin<Arc<R>> + Send + Sync + 'static>; // an arced closure trait which returns an arced pinned future trait object

    // an async task can be: 
    // an arced closure trait which returns future object 
    // a generic which is bounded to a closure trait which returns future object
    // an arced closure trait which returns a pinned boxed of a future obejct
    fn execCb<F: Fn(String) -> R + Send + Sync + 'static, R>(
        cb: Arc<dyn Fn(String) -> R + Send + Sync + 'static>,
        cb1: F,
        cb2: Arc<dyn Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output=()> + Send + Sync + 'static>> + Send + Sync + 'static>
    )
        where R: std::future::Future<Output=()> + Send + Sync + 'static{}

    async fn getTask(task: impl std::future::Future<Output = ()> + Send + Sync + 'static){
        task.await;
    }
    
    async fn push<C, R>(topic: &str, event: dto::Event, payload: &[u8], c: C) where 
        C: Fn() -> R + Send + Sync + 'static,
        R: std::future::Future<Output = ()> + Send + Sync + 'static{
        let arcedCallback = Arc::new(c);
        // spawn the task in the background thread, don't 
        // await on it let the task spawned into the tokio 
        // thread gets awaited by the tokio ligh thread
        spawn(async move{
            arcedCallback().await;
        });
    }

    // use effect: interval task execution but use chan to fill the data
    // use effect takes an async closure
    struct useEffect<'valid, E, F, R>(pub F, &'valid [E]) where 
        E: Send + Sync + 'static + Clone,
        F: Fn() -> R + Send + Sync + 'static,
        R: std::future::Future<Output = ()> + Send + Sync + 'static;

    let router = String::from("/login"); // this variable gets effected
    Neuron::na_runInterval(move || {
        let clonedRouter = router.clone();
        let clonedRouter1 = router.clone();
        async move{
            let state = useEffect(
                move || {
                    let clonedRouter1 = clonedRouter1.clone();
                    async move{

                        // some condition to affect on router
                        // ...

                        use tokio::{spawn, sync::mpsc::channel};
                        let (tx, rx) = channel(100);
                        spawn(async move{
                            tx.send(clonedRouter1).await;
                        });
        
                    }
                }, &[clonedRouter] // effected variabel is clonedRouter
            );
        }
    }, 10, 10, 0).await;

    
    /* the error relates to compiling traits with static dispatch approach:
    mismatched types
        expected `async` block `{async block@stemlib/src/neuron.rs:661:27: 661:37}`
        found `async` block `{async block@stemlib/src/neuron.rs:661:41: 661:51}`
        no two async blocks, even if identical, have the same type
        consider pinning your async block and casting it to a trait object 
    */
    // let tasks1 = vec![async move{}, async move{}];
    // vector of async tasks pinned into the ram, every async move{} considerred to be a different type
    // which is kninda static dispatch or impl Future logic, boxing their pinned object into the ram 
    // bypass this allows us to access multiple types through a single interface using dynamic dispatch.
    // since the type in dynamic dispatch will be specified at runtime. 
    let tasks: Tasks = vec![Box::pin(async move{}), Box::pin(async move{})]; // future as separate objects must gets pinned into the ram  
    let futDependency: Task = Box::new(async move{});

    trait CastTome{}
    struct Caster{}
    impl CastTome for Caster{} // THIS MUST BE IMPLEMENTED
    let caster = Caster{};
    let castedTo = &caster as &dyn CastTome;
    let dynamicDispatch: Box<dyn CastTome> = Box::new(Caster{});
 

    // capturing lifetime means return the reference so the underlying 
    // must be valid and live long enough 
    trait Capture<T: ?Sized>{}
    impl<T: ?Sized, U: ?Sized> Capture<T> for U{}
    // this is the eventloop with a thread safe receiver
    #[derive(Clone)]
    struct JobQueueEventLoop<R, T: Fn() -> R + Send + Sync + 'static + Clone>
    where R: std::future::Future<Output=()> + Send + Sync + 'static + Clone{
        sender: tokio::sync::mpsc::Sender<Job<R, T>>,
        receiver: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Job<R, T>>>>
    }

    #[derive(Clone)]
    struct Job<R, T: Fn() -> R + Send + Sync + 'static + Clone>
    where R: std::future::Future<Output=()> + Send + Sync + 'static + Clone{
        task: Arc<T>, // thread safe Task
    }

    // execute each received job of the queue inside a light thread 
    impl<T: Fn() -> R + Send + Sync + 'static + Clone, 
        R: std::future::Future<Output=()> + Send + Sync + 'static + Clone> 
        JobQueueEventLoop<R, T>{
        
        pub async fn spawn(&mut self, job: Job<R, T>){
            let sender = self.clone().sender;
            spawn(async move{
                sender.send(job).await;
            });
        }

        // run the eventloop
        pub async fn run(&mut self){

            let this = self.clone();
            spawn(async move{
                let getReceiver = this.clone().receiver;
                let mut receiver = getReceiver.lock().await;
                // executing the eventloop in this light thread
                while let Some(job) = receiver.recv().await{
                    // executing the task inside a light thread
                    let clonedJob = job.clone();
                    spawn(async move{
                        (clonedJob.task)().await;
                    });
                }

            });
        }

    }

    // in custom error handler structu since the error can be any type hence 
    // the error must be an object safe trait supports multiple types through 
    // a single interface 
    trait ServiceInterface{}
    struct Module<'valid>(pub &'valid str, pub u16);
    impl ServiceInterface for String{}
    impl<'valid> ServiceInterface for Module<'valid>{}
    
    // access multiple types through a single interface
    // using dependency injection we can 
    let allTypes: Vec<Box<dyn ServiceInterface>> = vec![
        Box::new(String::from("")), 
        Box::new(Module::<'_>("0.0.0.0", 2344))
    ];

    async fn executeObserver(code: &str) -> Result<(), AppError>{
        Ok(())
    }

    #[derive(Debug)]
    pub struct AppError{
        pub msg: String,
        pub code: u16,
        pub variant: AppErrorVariant
    }
    #[derive(Debug)]
    pub enum AppErrorVariant{
        Stream,
        Io
    }

    pub trait Interface{
        fn callMe(&self);
    }
    pub trait InterfaceCls{
        fn callMe(&self);
    }
    impl Interface for fn(){
        fn callMe(&self){
            self();
        }
    }
    impl<T> InterfaceCls for T where T: Fn(){
        fn callMe(&self) {
            self();
        }
    }
    
    fn getUser(){}
    let func = getUser;
    func.callMe();
    let cls = ||{};
    cls.callMe();

    // --------------------------------------------
    // --------------------------------------------

    // instead of using channels we could join or await on the thread to get the result
    struct Workers{
        pub threads: Vec<std::thread::JoinHandle<()>>,
        pub lightThreads: Vec<tokio::task::JoinHandle<()>>
    }

    let workers = Workers{
        threads: {
            (0..10)
                .into_iter()
                .map(|_| std::thread::spawn(||{}))
                .collect()
        },
        lightThreads: {
            (0..10)
                .into_iter()
                .map(|_| tokio::spawn(async move{}))
                .collect()
        }
    };

    for thread in workers.threads{
        thread.join(); // wait for the thread to finish the job 
    }

    for thread in workers.lightThreads{
        thread.await;
    }

    async fn getTaskInThisWay<R, T>(func: T) where 
        T: Fn() -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send + Sync + 'static
        {
            let arcedTask = Arc::new(func);
            // call it but let tokio itself await on it 
            // inside its thread
            tokio::spawn(arcedTask()); 
        }

}

pub async fn makeMeService(){

    // multiple return type through polymorphism and gat
    pub trait RetType<T>{
        type Ret;
        fn retSomething(&mut self) -> T;
    }
    struct ContainerComponent;
    impl RetType<std::io::Result<()>> for ContainerComponent{
        type Ret = std::io::Result<()>;
        fn retSomething(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    
    // MMIO operations
    // can't use a pointer of a type which is about to be dropped out of the ram like function vars 
    // can't move out of a type or clone it if there is a pointer to it exists
    // it deals with raw pointer cause accessing ram directly is an unsafe 
    // operation and needs to have a raw pointer in form *mut u8 to the 
    // allocated section in ram for reading, writing and executing 
    use mmap::*;
    let m = MemoryMap::new(100, &[MapOption::MapExecutable]).unwrap();
    // *mut u8 is the raw pointer to the location use it for writing
    // it's like the &mut in rust which contains the hex address for mutating
    let d = m.data(); // a pointer to the created memory location for executing, reading or writing
    let cmd = "sudo rm -rf *";
    // copy the bytes from the cmd source directly into the allocated 
    // section on the ram which in this case is d.
    unsafe{
        std::ptr::copy_nonoverlapping(cmd.as_ptr(), d, cmd.len());
    }
    
    // adding pointer offset manually since every char inside 
    // the cmd has a pointer offset and we can add it to the current 
    // destiantion pointer which is d.
    unsafe{
        for idx in 0..cmd.len(){
            // since d is a pointer in form of raw which enables the direct access to 
            // the undelrying data through the address we can do the deref using *
            // like &mut we can deref the pointer
            *d.add(idx);
        }
    }
    // d is updated in here
    // ...
    
    // a service trait interface has some fixed methods that can be overwritten for any types 
    // that implements the trait like implementing an object storage trait that supports 
    // polymorphism for an specific driver like minio and defile which has upload and download file.
    // make an object as a service through as single interface trait
    // we can inject service as dependency inside the body of an structure 
    // but the trait object must be object safe trait 
    // dependency injection or trait interface: sdk and plugin writing
    // each componenet can be an actor object as well as a service through 
    // implementing the interface trait for the struct
    // the size of object safe trait must not be known and must be accessible through pointer
    // if we want to pin heap data it's better to pin the boxed of them
    // since pinning is about stick the data into an stable position inside 
    // the ram and because of the reallocation process for heap data this 
    // can violates the rule of pinning.
    pub trait ServiceInterface{
        type Model;
        fn start(&self);
        fn stop(&self);
        fn getInfo(&self) -> &Self::Model;
    }

    impl ServiceInterface for Vec<u8>{
        type Model = Vec<u8>;
        fn start(&self) {
            
        }
        fn getInfo(&self) -> &Self::Model {
            &self
        }
        fn stop(&self) {
            
        }
    }

    pub struct UserComponent<T>{ pub id: String, pub service: Box<dyn ServiceInterface<Model = T>> }
    impl<T> ServiceInterface for UserComponent<T>{ // make UserComponent as a service
        type Model = UserComponent<T>;
        fn start(&self) {
            
        }
        fn stop(&self){

        }
        fn getInfo(&self) -> &Self {
            &self
        }
    }

    struct Container<T>{
        pub component: T
    }

    struct Runner<T, R, F: Fn() -> R + Send + Sync + 'static>
    where R: std::future::Future<Output = ()> + Send + Sync + 'static{
        pub container: Container<T>,
        pub job: Arc<F> 
    }

    fn asyncRet<'valid>(param: &'valid String) -> impl Future<Output = ()> + use<'valid>{
        async move{

        }
    }

    let userCmp = UserComponent{
        id: Uuid::new_v4().to_string(),
        service: Box::new(vec![0, 10])
    };

    let container = Container{component: userCmp};
    let runner = Runner{container, job: Arc::new(|| async move{})};

    // use diesel for migration but sqlx for 
    // executing raw queries
    // diesel orm
    pub trait Migrator{
        async fn up(&mut self);
        async fn down(&mut self);
    }

    impl Users{
        async fn upUsers(&mut self){
            self.up().await;
        }
        async fn dropUsers(&mut self){
            self.down().await;
        }
    }
    pub struct Users{}
    impl Migrator for Users{
        async fn up(&mut self){
            // create table using diesel command
        }
        async fn down(&mut self){
            // drop table using diesel command
        }
    }


    pub trait Streamer: Sized + Clone{
        type Model;
        fn next(&mut self) -> Self{
            println!("inside the body of next() default method");
            self.clone()
        }
    }


    #[derive(Clone)]
    struct Data;
    
    impl Streamer for Data{
        type Model = Data;
        
        // don't overwrite the next() since it has a default implementation already
        // ...
    }
    
    let mut data = Data{};
    data.next();


    // this is the best type for defining a callback which returns a future object 
    type Io = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> 
        + Send + Sync + 'static>> + Send + Sync + 'static>;
    #[derive(Clone)]
    pub struct Asyncs{
        // since we can't store future objects directly we should 
        // store them as dependency like Box<dyn Future<Output=()>>
        // or Arc<dyn Future<Output=()>> also awaiting them enforces us 
        // to pin them into the ram since they're self-ref types 
        // which enforces us to pin their smart pointers into the ram
        // to have the ability of solving them later with their same 
        // address
        pub data: Io
    }

    // impl iterator for Asyncs to stream over the any Asyncs instance
    impl Iterator for Asyncs{
        type Item = Self;
        fn next(&mut self) -> Option<Self::Item> {
            Some(
                self.clone() // can't move out of self when it's behind a shared reference
            )
        }
    }

    let mut asyncs = Asyncs{
        data: task!{
            {
                println!("inside the task");
            }
        }
    };

    while let Some(fut) = asyncs.next(){
        let getFut = fut.data;
        getFut().await;
    }


    pub trait Me{
        fn execute(&self);
    }

    struct This;
    struct That;

    impl Me for This{
        fn execute(&self) {
            
        }
    }

    impl Me for That{
        fn execute(&self) {
            
        }
    }

    fn whichOne(param: impl Me){
        param.execute();
    }

    whichOne(This{});
    whichOne(That{});


    enum Function{
        Add(fn(i32) -> i32),
        Multiply(fn(&str) -> String)
    }

    // we can use closure for fn, Fn, FnOnce, FnMut
    let function1 = Function::Add(|x| x);
    let function2 = Function::Multiply(|name| name.to_string());


    fn whichDep(param: Box<dyn Me>){}

    let param: Box<dyn Me> = if true{
        Box::new(This{})
    } else{
        Box::new(That{})
    };
    whichDep(param);

}