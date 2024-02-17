


mod build;
use std::{sync::{Arc, Weak, RwLock}, cell::RefCell};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/*

    -----------------------------
    interact with gem repo mmr.rs
    -----------------------------
    build dl models based on phases dataset using keras and https://github.com/wildonion/stem/
    behavioural graph virtual machine built on top of each event's `phases` field inside the game 
    for each player to match them for new game and rank them based on their in-game statuses, the 
    match making rating or ranking (**MMR**) engine, on the other hand is is a weighted tree based 
    suggestion engine that suggests players, events and other games and players based on their ranks 
    earned using **GVM** during the game.
    

    never return poiner from method just return vec or string but pass them in slice form to not to lose 
    their ownerhsip cause can't return ref to a data owned by the method we have to either ret the type in 
    its slice form with valid lifetime or return it as self.field cause self has longer lifetime also the 
    memory allocation process in rust depends on the types' lifetimes means that every type in rust has a 
    valid lifetime and as soon as the type gets moved into other scopes or threads or methods its lifetime 
    will be dropped from the ram and will be owned by that scope in other words we can't move out of a type
    or deref it if it's behind a shared ref or pointer also we can't have a type in two scopes at the same time
    without passing a reference or a clone of that into the second scopes, we should either clone the type or 
    borrow it and pass one of these to the second scope cause rust doesn't have gc to track the references 
    came to the type and use that to destroy the type when it reaches the zero instead it's using lifetime 
    conceptes which in single thread contexts we can use Rc to count the references of a type shared between 
    scopes but in multithread contexts we must use Arc and Mutex or RwLock to share the ownership of the type 
    between threads without having race conditions and deadlocks although rust forces us to use every type only 
    once during the whole lifetime of the app which this manner prevents us from allocating extra space on the 
    ram and deadlocks situation since the concept of ownership and borrowing is about sharing a type with 
    reference or cloning to not to lose its ownership in future scopes, if we allocate something in a scope we 
    can't move it out of that cause its being used inside that scopes, we have to borrow it or clone it or convert 
    it to its owned type if it's a sliced form or pointer of a type, slices are just a representation of dynamic 
    types with no dynamic allocation feature.

    this is lle parallel based vm and game engine graph like DOM, an state manager 
    like yew and redux with tree walking using (shared ref and mutably) using rc and 
    arc weak and strong ref counting, shared ownership and interior mutability, based on 
    actor and graph concepts so we have followers weighted tree to understand the 
    relationship between peers to suggests events in a graph virtual machine by using 

    lazy static global, mutexed, rwlocked, mpsc, rusty ltg pointers, slices, codec hash hex and file bytes,
    ret &'validlifetime ref and trait as param like -> impl Trait as type from method and use them in method 
    param like param: impl Trait from method also can't move type if it's behind pointer send sync static, 
    shared ownership using Mutex and RwLock and RefCell, GlobalAlloc arena referene counting using Rc Arc, Box 
    leaking, Pin, &mut type, cap, length, macros, Cow, Borrowed, ToOwned, Deref, &mut (ast, token stream), 
    std::mem, generic, lifetimes, closures, traits, pointers and bytecode, .so and .elf bpf, wasm, bytes and 
    hex serding and codec ops using borsh and serde, async trait and associative bounding Trait::method(): Send and 
    ?async and ?const, gen block, r3bl_rs_utils crate, read/write io traits, Box<dyn Trait>, as_ref(), unwrap(), clone() 
    and Box stores data on the heap and contains an smart pointer with a valid lifetime to the underlying type, 
    also the size of the boxed type is the size of the type itself, the value of the box can be caught 
    by dereferencing the box
    

    every type has its own lifetime and if it goes out of scope it'll be dropped from the ram and we can 
    either borrow it or clone it since rust dones't have gc instead it has rc, refcell for single thread 
    and arc and mutex for multithread reference counting and borrowing based on these concetps share ownership 
    between threads using Arc by borrowing the ownership using pointers like & clone share ownership between 
    scopes using Rc by  borrwoing the ownership using pointers like & and clone Rc is not safe to be used between 
    threads but Arc can be used to share the type between multiple threads safely without having race conditions, 
    also if we want to mutate an immutable type at runtime we must use RefCell which is a single threaded smart 
    pointer and for mutating type between multiple threads we must use Mutex or RwLock to avoid deadlocks situations.
    
    Single Thread    Multithread             Usage
    Rc               --> Arc                 make the type shareable between scopes and threads
    RefCell          --> RwLock || Mutex     make the type mutable safe at runtime in scopes and threads

    everytime we try to move type betweeen scopes and threads without losing ownership we're taking a reference to that thread so:
        share and mutate the actual type using its &mut pointer in a single thread scope
        share and mutate the actual type using its Arc<RwLock<Type>> pointer in a multithread scope   
    
    all ltgs in rust ::::: https://github.com/wildonion/rusty/blob/main/src/retbyref.rs#L17
    zero copy        ::::: https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1621C6-L1621C6
    data collision   ::::: https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/utils.rs#L640 
    near rules       ::::: https://github.com/wildonion/smarties/blob/main/contracts/near/NEAR.rules
    solana rules     ::::: https://github.com/wildonion/solmarties/blob/main/SOLANA.rules
    https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1305
    https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1213
    https://developerlife.com/2022/02/24/rust-non-binary-tree/#naive-approach-using-weak-and-strong-references
    https://developerlife.com/2022/03/12/rust-redux/
    https://bevyengine.org/learn/book/introduction/  
    https://godotengine.org/
    https://nannou.cc/
    https://crates.io/crates/rg3d
    https://amethyst.rs/
    https://fyrox-book.github.io/introduction.html
    https://www.youtube.com/watch?v=yq-msJOQ4nU
    https://github.com/wildonion/cs-concepts
    https://github.com/wildonion/rusty => all ltg codes
    https://doc.rust-lang.org/nomicon/index.html
    https://stackoverflow.com/questions/26271151/precise-memory-layout-control-in-rust
    https://docs.rust-embedded.org/book/
    https://crates.io/crates/hotham
    https://developers.google.com/protocol-buffers/docs/encoding
    https://capnproto.org/encoding.html
    https://ethereum.org/nl/developers/docs/evm/
    https://blog.subnetzero.io/post/building-language-vm-part-01/
    https://rust-hosted-langs.github.io/book/
    https://benkonz.github.io/building-a-brainfuck-compiler-with-rust-and-llvm/
    https://opensource.com/article/19/3/rust-virtual-machine
    https://medium.com/iridium-vm/so-you-want-to-build-a-language-vm-in-rust-part-09-15d90084002
    https://medium.com/clevyio/using-rust-and-nom-to-create-an-open-source-programming-language-for-chatbots-12fe67582af5
    https://cheats.rs/#behind-the-scenes
    https://github.com/ethereum/evmone => compiled smart contract bytecode executes as a number of EVM opcodes
    https://blog.logrocket.com/guide-using-arenas-rust/
    https://zhauniarovich.com/post/2020/2020-12-closures-in-rust/
    https://blog.cloudflare.com/pin-and-unpin-in-rust/
    https://fasterthanli.me/articles/pin-and-suffering
    https://stackoverflow.com/questions/2490912/what-are-pinned-objects
    https://medium.com/tips-for-rust-developers/pin-276bed513fd1
    https://users.rust-lang.org/t/expected-trait-object-dyn-fnonce-found-closure/56801/2
    codec, virtual machine like move and evm with allocation concepts 
        - macro dsl and 
        - thread_local, actor id or address, std::alloc, jemalloc, bumpalo and r3bl_rs_utils arena as a global allocator
        - zero copy 
        - null pointer optimiser
        - unique storage key

*/


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


*/
    

/*      
           --------------------------------------------------
                 a thread safe global response objects
           --------------------------------------------------


        single thread and none gc concepts: ltg &mut pointer (rc,refcell,arc,mutex,lazy,threadlocal),box,pin,impl Trait,

        code order execution and synchronization in multithreaded based envs like
        actor worker like having static lazy arced mutex data without having deadlocks 
        and race conditions using std::sync tokio::sync objects like 
        semaphore,arc,mutex,rwlock,mpsc also data collision, memory corruption, deadlocks 
        and race conditions avoidance in async and multithreaded contexts are: 
            - share none global app state data between tokio::spawn() threads using mpsc 
            - enum and actor id as unique storage key
            - mutate a global storage using thread local in single-threaded contexts
            - mutate a gloabl storage using static lazy arc mutexed in multi-threaded contexts
            - can't move out of a reference or deref a type if its pointer is being used by and shared with other scopes
            - can't mutate data without acquiring the lock of the mutex in other threads
            - can't have both mutable and immutable pointers at the same time

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
    thread can pull from this buffer
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

/*

concepts:
    shared ownership, 
    interior mutability, 
    weak, and strong references

Shared ownership
    While the children are owned by the struct, it is necessary to provide access to these 
    children node to other code that use this tree data structure. Moving these references 
    out of the tree isn’t desirable. And cloning the entire node before moving it out of the 
    tree isn’t optimal either. This is where shared onwnership comes into play. In order to 
    do that, we wrap the underlying node in a Rc. This is a reference counted pointer. However, 
    that isn’t enough, since once we pass a (shared) reference to other code (that is using 
        this tree), we need to provide the ability to mutate what is inside the node itself, 
        which leads us to interior mutability.
Interior mutability
    Once a reference (that allows for shared ownership) of a node is passed to code 
    using the tree, it becomes necessary to allow modifications to the underlying node 
    itself. This requires us to use interior mutability by wrapping the node in a 
    RefCell. Which is then wrapped in the Rc that we use to share ownership. Combining 
    these two together gets us to where we need to be.


NodeData
 | | |
 | | +- value: T ---------------------------------------+
 | |                                                    |
 | |                                        Simple onwership of value
 | |
 | +-- parent: RwLock<WeakNodeNodeRef<T>> --------+
 |                                                |
 |                 This describes a non-ownership relationship.
 |                 When a node is dropped, its parent will not be dropped.
 |
 +---- children: RwLock<Vec<Child<T>>> ---+
                                          |
                This describes an ownership relationship.
                When a node is dropped its children will be dropped as well.


1 - When a node is dropped, its children will be dropped as well (since it owns them). 
    We represent this relationship w/ a strong reference.

2 - However, the parent should not be dropped (since it does not own them). 
    We represent this relationship w/ a weak reference.

*/

async fn pinned_box(){

    /* 
        types that implement Unpin can be moved safely but those types likes futures and tratis that
        implements !Unpin are not safe to be moved and if we need them later to use them like solving
        a future object we must pin their pointer into the ram to prevent them from moving so we need
        Pin them to safely poll them or solve them using .await, by pinning the pointer of the type
        we can tell the rust that hey don't move this type around the ram.
        trait objects like closures are dynamically sized means they're stored on the heap in order 
        to act them as a separate object or type we need to either put them behind a pointer or box 
        them, this would be true about the futures cause they're traits too.
        future objects must be pinned to the ram before they can be solved or polled the reason 
        of doing this is first of all they're trait objects and traits are dynamically sized means 
        they're size will be known at runtime second of alldue to the fact that rust doesn’t have 
        gc which allows us not to have a tracking reference counting process for a type at runtime 
        cause it’ll move the type if the type goes out of the scope hence in order to solve and 
        poll a future in other scopes later on, we should pin it to the ram first which can be done 
        once we await on the future but if we want to solve and poll a mutable reference of a future 
        we should stick and pin it to the ram manually, first by pinning the future into the ram using 
        Box::pin, tokio::pin!(), std::pin::pin!() then do an await on the mutable reference of the 
        future object, so if it is required to call .await on a &mut _ reference, the caller is 
        responsible for pinning the future by pinning future objects manually we make them as an object 
        before polling them like having a mutable reference to them or pass them into other parts to 
        solve them in different parts
    */
    let mut future = async move{};
    tokio::pin!(future); // first we must pin the future object before solving/polling its mutable pointer 
    (&mut future).await;
    
    // pinning the pointer of future object into the ram, future objects are traits
    // and traits must be behind &dyn or Box<dyn to be as an object at runtime thus
    // we're pinning the box pointer of the future object which is on the heap into 
    // the ram to avoid moving it for futuer solvation.
    let pinned_boxed = Box::pin(&mut future); // in cases if we need to access the pinned value outside of the current scope cause the future is boxed and we can move it as an object
    std::pin::pin!(&future); // first we must pin the future object before solving/polling its mutable pointer, also pin!() macro returns a pinned mutable pointer which we can solve it later
    (&mut future).await; // polling the mutable reference of the futuer object

    /*
        the type that is being used in solving future must be valid across .awaits, 
        because future objects will be pinned into the ram to be solved later, worth
        to know that trait pointers are Boxes and we pin their pointer into ram like: 
        Pin<Box<dyn Future<Output=String>>>
    */

    fn get_data<G>(param: impl FnMut() -> ()) -> impl FnMut() 
        -> std::pin::Pin<Box<dyn std::future::Future<Output=String> + Send + Sync + 'static>>
        where G: Send + Sync + 'static + Sized + Upin{
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
    

}


fn serding(){
    
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


fn init_vm(){

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
        Vm,
    }
    let cost = Cost::Runtime { executor: Executor };
    match cost{
        Cost::Runtime { executor: executor_instance } => {
            let ext = executor_instance;
            todo!()
        },
        Cost::Vm =>{
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


#[derive(Clone)]
struct Gadget{
    me: std::rc::Weak<Gadget>, 
    you: std::rc::Rc<Gadget>
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
struct Generic<'info, Gadget>{
    pub gen: Gadget,
    pub coded_data: &'info [u8]
}

impl Gadget{

    fn new(ga: Gadget) -> std::rc::Rc<Self>{
        std::rc::Rc::new_cyclic(|g|{
            Gadget { me: g.to_owned(), you: std::rc::Rc::new(ga) }
        })
    }

    fn me(&self) -> std::rc::Rc<Self>{
        std::rc::Rc::new(self.clone()) /* upgrade weak pointer to rc */
    }
}


/** a multithreaded based graph, parent and child are both of type NodeData<T> */
///// Arc ----> Rc | Mutex -----> RefCell
type ChildNodeToParentIsWeak<T> = Weak<NodeData<T>>;
type ParentNodeToChildIsStrongThreadSafe<T> = Arc<NodeData<T>>; // equivalent to Rc<NodeData<T>> in single thread
type ThreadSafeMutableParent<T> = tokio::sync::Mutex<ChildNodeToParentIsWeak<T>>; // Mutex<Weak<NodeData<T>>> is equivalent to RefCell<Weak<NodeData<T>> in single thread
type ThreadSafeMutableChildren<T> = tokio::sync::Mutex<Vec<ParentNodeToChildIsStrongThreadSafe<T>>>; // Mutex<Vec<Arc<NodeData<T>>>> is equivalent to RefCell<Vec<Rc<NodeData<T>>>> in single thread
/* future are traits that must be behind pointers like Box<dyn> or &dyn */
// let pinned_box_pointer_to_future: PinnedBoxPointerToFuture = Box::pin(async{34});
type PinnedBoxPointerToFuture = std::pin::Pin<Box<dyn std::future::Future<Output=i32>>>;

/* 
    thread safe tree using Arc and tokio::sync::Mutex to create DOM and redux like system 
    note that a node data can be either a parent or a child node, if it's a parent node 
    then all its `children` must be in form Arc<NodeData<T>> or a strong reference to all 
    children and if it's a child node then its `parent` field must be in form Weak<NodeData<T>>
    or a weak reference to its parent
    parent to points to children in a strong way since if a parent want to be removed all its 
    children or strong references must be removed and reaches zero first then the parent can be 
    dropped and child points to parent in a weak way since by dropping a child the parent shouldn’t 
    be dropped which is the nature of weak reference since the type can be dropped even there are 
    multiple weak references are pointing to the type
*/
struct NodeData<T>{
    pub value: T,
    pub parent: ThreadSafeMutableParent<T>, // Mutex<Weak<NodeData<T>>> means any child node contains the parent node must be in form of a weak reference to parent node data but safe to be mutated
    pub children: ThreadSafeMutableChildren<T> // // Mutex<Vec<Arc<NodeData<T>>>> means any parent node contains its children must be a vector of strong references to its children and safe to be mutated cause if the parent get removed all its children must be removed
}

struct Arena<T: Sized + Clone>{
    data: Box<NodeData<T>>
}
trait ArenaExt{
    type Data;
    fn set_data(&mut self, new_data: Self::Data) -> Self;
    fn get_data(self) -> Self::Data;

}
impl<T: Sized + Clone> ArenaExt for Arena<T>{
    type Data = Box<NodeData<T>>;
    fn set_data(&mut self, new_data: Self::Data) -> Self{
        Self { data: new_data }
    }
    fn get_data(self) -> Self::Data {
        // let data = self.data; // can't move out of self since it's behind a shared reference and is valid as long as the object is valid
        self.data
    }
}
