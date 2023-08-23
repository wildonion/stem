



use std::sync::{Arc, Weak, RwLock};

/*

    ----------------------
    interact with gem repo
    ---------------------- 
    behavioural graph virtual machine (**GVM**) built on top of each event's `phases` 
    field inside the game for each player to suggests them the tips and tricks for a new 
    game and reward them based on their game scores using an AI based coin generation 
    model in which players get rewarded based on their scores and positions then update t
    he balance field of the user based on those attributes.
    

    this is lle parallel based vm and game engine graph like DOM, an state manager 
    like yew and redux with tree walking using (shared ref and mutably) using rc and 
    arc weak and strong ref counting, shared ownership and interior mutability, based on 
    actor and graph concepts so we have followers weighted tree to understand the 
    relationship between peers to suggests events in a graph virtual machine by using 
    send sync static, shared ownership using Mutex and RwLock and RefCell, 
    referene counting using Rc Arc, Box, Pin, &mut pointer, cap, length, traits, macros 
    (ast, token stream), std::mem, generic, lifetimes, closures, traits, pointers and 
    bytes and hex serding ops, async trait and associative bounding Trait::method(): Send 
    and ?async and ?const, &mut
    
    
    share ownership between threads using Arc by borrowing the ownership using pointers like & clone 
    share ownership between scopes using Rc by  borrwoing the ownership using pointers like & and clone
    Rc is not safe to be used between threads but Arc can be used to share the type between multiple 
    threads safely without having race conditions, also if we want to mutate an immutable type at runtime
    we must use RefCell which is a single threaded smart pointer and for mutating type between multiple 
    threads we must use Mutex or RwLock to avoid deadlocks situations.
    
    Single Thread    Multithread             Usage
    Rc               --> Arc                 make the type shareable between scopes and threads
    RefCell          --> RwLock || Mutex     make the type mutable safe at runtime in scopes and threads

    https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1305
    https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1213
    https://developerlife.com/2022/02/24/rust-non-binary-tree/#naive-approach-using-weak-and-strong-references
    https://developerlife.com/2022/03/12/rust-redux/
    https://bevyengine.org/learn/book/introduction/  
    https://godotengine.org/
    https://fyrox-book.github.io/introduction.html
    https://www.youtube.com/watch?v=yq-msJOQ4nU
    https://github.com/wildonion/cs-concepts
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

*/

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
 |                                            |
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


type ChildNodeToParentIsWeak<T> = Weak<NodeData<T>>;
type ParentNodeToChildIsStrongThreadSafe<T> = Arc<NodeData<T>>;
type ThreadSafeMutableParent<T> = RwLock<ChildNodeToParentIsWeak<T>>;
type ThreadSafeMutableChildren<T> = RwLock<Vec<ParentNodeToChildIsStrongThreadSafe<T>>>;

/* thread safe tree using Arc and RwLock to create DOM */
struct NodeData<T>{
    pub value: T,
    /* 
        parent is a weak ref since it's not owned by the struct 
        also RwLock is RefCell in single thread context
    */
    pub parent: ThreadSafeMutableParent<T>, 
    /* 
        children is a strong reference since it's owned by the 
        parent so we've put Arc which is Rc in single theread 
        context thus it's like RefCell<Rc<T>> in single 
        thread context 
    */
    pub children: ThreadSafeMutableChildren<T>
}
