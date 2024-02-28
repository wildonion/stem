


use crate::*;


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
/** a multithreaded based graph, parent and child are both of type NodeData<T> */
///// Arc ----> Rc | Mutex -----> RefCell | Weak is Weak
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
    parent points to children in a strong way since if a parent want to be removed all its 
    children or strong references must be removed and reaches zero first then the parent can be 
    dropped and child points to parent in a weak way since by dropping a child the parent shouldn’t 
    be dropped which is the nature of weak reference since the type can be dropped even there are 
    multiple weak references are pointing to the type, but in strong case first all the pointers 
    and references came to the type must be dropped and reach zero count then the type itself can 
    be dropped after that easily.
    any instance of NodeData, if it's a child node instance then the pointer to the parent field must be weak
    if it's a parent node instance then the pointer to the child must be strong
*/
struct NodeData<T>{
    pub value: T,
    pub parent: Option<ThreadSafeMutableParent<T>>, // Mutex<Weak<NodeData<T>>> means any child node contains the parent node must be in form of a weak reference to parent node data but safe to be mutated
    pub children: Option<ThreadSafeMutableChildren<T>> // // Mutex<Vec<Arc<NodeData<T>>>> means any parent node contains its children must be a vector of strong references to its children and safe to be mutated cause if the parent get removed all its children must be removed
}

struct Arena<T: Sized + Clone>{
    data: Option<Box<NodeData<T>>> // the arena in this case is the Box
}
impl<T: Sized + Clone> Arena<T>{
    pub fn new() -> Self{
        Self{
            data: None
        }
    }
}
trait ArenaExt{
    type Data;
    fn set_data(&mut self, new_data: Self::Data) -> Self;
    fn get_data(self) -> Self::Data;

}
impl<T: Sized + Clone> ArenaExt for Arena<T>{
    type Data = Option<Box<NodeData<T>>>;
    fn set_data(&mut self, new_data: Self::Data) -> Self{
        Self { data: new_data }
    }
    fn get_data(self) -> Self::Data {
        // let data = self.data; // can't move out of self since it's behind a shared reference and is valid as long as the object is valid
        self.data
    }
}

fn create_arena_node(){
    let mut arena_node = Arena::<String>::new();
    let arena_node_data = arena_node.set_data(Some(
        Box::new(NodeData::<String>{
            value: String::from("root"),
            parent: None,
            children: None
        })
    ));
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