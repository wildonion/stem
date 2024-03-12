


use std::collections::HashMap;
mod build;
use std::{sync::{Arc, Weak, RwLock}, cell::RefCell};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
mod misc;
use crate::misc::*;
mod graph;
use crate::graph::*;
mod bop;
use crate::bop::*;


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

    fn func() -> i32{
        0
     }
    let funcpointer = func as *const (); // () is function in c

    pub static DB: std::cell::RefCell<std::collections::HashMap<String, String>> = 
        std::cell::RefCell::new(HashMap::new());
}
// DB.with_borrow_mut(|db| {
//     db.insert("key".to_string(), "value".to_string())
// });

fn local_storage_ex(){

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


