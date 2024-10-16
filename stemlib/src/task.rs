


use crate::*;
use std::{ops::DerefMut, sync::atomic::{AtomicU64, AtomicUsize}};
use uuid::Uuid;


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

pub fn create_arena_node(){
    let mut arena_node = Arena::<String>::new();
    let arena_node_data = arena_node.set_data(Some(
        Box::new(NodeData::<String>{
            value: String::from("root"),
            parent: None,
            children: None
        })
    ));
}

pub fn box_arena_node<T: Sized + Clone>(node: NodeData<T>) -> Arena<T>{
    let mut arena_node = Arena::<T>::new();
    let arena_node_data = arena_node.set_data(Some(
        Box::new(node)
    ));
    arena_node_data
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


/* ----=====----=====----=====----=====----=====----=====----=====----=====----=====----=====----=====---- 
    async tasks or jobs that will be executed in the background inside a lightweight thread of 
    execution using tokio::spawn() task scheduler and jobq based channels; actor workers will run 
    these tasks in their own execution context like what i've simulated here.
    typically any instance of the Task which is kina actor must contains:
        - the future job itself
        - the sender to send the result of executed task to the channel for using outside of the thread
        - a thread safe (Mutex) worker as the background worker thread to execute the future job in it
        - a locker to lock the task when it's executing the task 
    the task can be awaited to complete its future or aborted during the dropping process of the
    task instance, tokio::spawn() is the backbone of each background worker, it gets a future and 
    move it into a lightweight thread of execution. 
    lock the instance using the lock field to check that if we're doing some heavy process or not
    then in switching the task or doing other heavy process check the lock that if the instance 
    is already locked or not also we should lock the worker if we want to execute something in the 
    background worker of the instance thread to tell obj caller that the worker is busy rn. 

    more details:
    locker, threadpool, worker, future io task, eventloop, 
    sender, signal condvar, job tree, dep inj future job:
        - cron scheduler method
        - execute in worker method 
        - receive from eventloop and exec in threadpool method
        - instance locker method
    
    future object
    job tree to push the job into the current tree
    sender to broadcast or publish some data to a channel
    an eventloop to receive a data from the channel or the queue to execute it in the background worker thread
    background worker to run the job
    locker to lock the task instance when a task is being executed 
    worker thread of type joinHandle to execute task or job of type async io or cpu tasks 
    threadpool to execute each task when receives them from mpsc recevier eventloop 
    atomic syncing with channels and mutex 
*/
// #[derive(Debug)] // don't implement this cause Pin doesn't implement Debug
pub struct Task<J: std::future::Future<Output = ()>, S> where // J is a Future object and must be executed with Box::pin(job);
    J: std::future::Future + Send + Sync + 'static + Clone,
    J::Output: Send + Sync + 'static
{
    pub status: TaskStatus,
    pub id: String,
    pub name: String, // like send_mail task 
    /* 
        Pin is a wrapper around some kind of pointer Ptr which makes 
        that pointer "pin" its pointee value in place, thus preventing 
        the value referenced by that pointer from being moved or otherwise 
        invalidated at that place in memory unless it implements Unpin
        which means tha type type doesn't require to be pinned into 
        the ram, self ref types must implement !Unpin or must be pinned
    */
    pub io: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>>, // a future as separate type to move between scopes
    pub job: J, 
    pub metadata: Option<serde_json::Value>,
    pub job_tree: Vec<Task<J, S>>,
    pub sender: tokio::sync::mpsc::Sender<S>, // use this to send the result of the task into the channel to share between other lightweight thread workers
    pub eventloop_queue: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<S>>>, // use this as eventloop to execute tasks as they're coming from the channel in the background worker thread
    pub pool: Vec<tokio::task::JoinHandle<()>>,
    pub worker: std::sync::Mutex<tokio::task::JoinHandle<()>>, // execute the task inside the background worker, this is a thread which is safe to be mutated in other threads 
    pub lock: std::sync::Mutex<()>, // the task itself is locked and can't be used by other threads
    pub state: std::sync::Arc<tokio::sync::Mutex<Vec<u8>>> // the state of the worker must be safe to be shared between threads
}

// thread safe eventloop and queue: arc mutex vec T vs arc mutex receiver T
pub struct QueueAndEventLoop<T: Clone + Send + Sync + 'static>{
    pub eventloop: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<T>>>,
    pub queue: std::sync::Arc<tokio::sync::Mutex<Vec<T>>>,
}


impl<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S: Sync + Send + 'static> 
    Task<J, S> {

    pub async fn new(job: J, num_threads: usize,
        sender: tokio::sync::mpsc::Sender<S>, 
        eventloop_queue: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<S>>>) -> Self{

        // sender and receiver

        let task = Self{
            status: TaskStatus::Initializing,
            id: Uuid::new_v4().to_string(),
            name: String::from("KJHS923"),
            job: job.clone(),
            metadata: None,
            io: Box::pin(async move{  }), // pinning the future into the ram with the output of type O
            sender,
            pool: {
                (0..num_threads)
                    .map(|_| tokio::spawn(job.clone()))
                    .collect::<Vec<tokio::task::JoinHandle<()>>>()
            },
            eventloop_queue,
            job_tree: vec![],
            worker: { // this is the worker that can execute the task inside of itself, it's basically a lightweight thread
                std::sync::Mutex::new( // lock the worker
                    tokio::spawn(job)
                )
            },
            state: std::sync::Arc::new(tokio::sync::Mutex::new(vec![])),
            lock: Default::default(),
        };

        task 

    }

    pub async fn send(&self, d: S){
        let sender = self.sender.clone();
        sender.send(d).await;
    }

    pub fn is_busy(&mut self) -> bool{
        self.lock.try_lock().is_err() // is_err() can be either true or false, trying to acquire the lock
    }

    pub async fn spawn(&self){

        let job = self.job.clone();
        tokio::spawn(job);
    }

    // method to execute the job in the task worker
    pub async fn execute(&mut self){

        // wailt until the lock gets freed cause we're pushing tasks into the tree 
        // if we slide down into the while loop means the method returns true which
        // means the lock couldn't get acquired
        while self.is_busy(){ 
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }

        let t = self.job.clone(); // clone to prevent from moving
        let mut get_worker = self.worker.try_lock().unwrap(); // lock the worker
        (*get_worker) = tokio::spawn(t);
    }

    pub async fn switch_task(&mut self, task: J){
        
        // wailt until the lock gets freed cause we're pushing tasks into the tree 
        // if we slide down into the while loop means the method returns true which
        // means the lock couldn't get acquired
        while self.is_busy(){ 
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }

        let mut get_worker = self.worker.lock().unwrap();
        (*get_worker) = tokio::spawn(task);

    }

    pub fn push(mut self, tasks: Vec<Task<J, S>>) -> Task<J, S>{
        
        // lock the instance to push tasks into the tree
        self.lock.lock().unwrap();
        self.job_tree.extend(tasks);
        self
    }

    // task lifecycles
    pub fn halt(&mut self){
        self.status = TaskStatus::Hanlted;
    }

}

// once the task gets dropped drop any incomplete futures inside the worker 
impl<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S> Drop for Task<J, S>{
    fn drop(&mut self) { // use std::sync::Mutex instead of tokio cause drop() method is not async 
        if let Ok(job) = self.worker.lock(){
            job.abort(); // abort the current future inside the joinhandle
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum TaskStatus{
    #[default]
    Initializing,
    Executed,
    Hanlted
}

/* ----------------------------------------------------- */
//          a thread safe task tree executor
/* ----------------------------------------------------- 
|   use smart pointers to break the cycle of self ref 
|   types, in here we're creating a node for the entire 
|   task tree which contains a reference to the itself
|   it gets executed in BFS order.
|
*/

pub struct TaskTree<
    J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone, S>{
    // wrap it around mutex to share the task between threads cause we
    // want to execute the task in a light thread without blocking so 
    // we need to move the reference of the task into the thread which 
    // can be done via mutex since it's an smart pointer for sharing data
    // safely between threads
    pub task: tokio::sync::Mutex<Task<J, S>>, 
    pub weight: std::sync::atomic::AtomicU8,
    pub parent: std::sync::Arc<TaskTree<J, S>>, // the parent itself
    pub children: std::sync::Mutex<Vec<std::sync::Arc<TaskTree<J, S>>>> // vector of children
}

impl<J: std::future::Future<Output = ()> + Send + Sync + 'static + Clone + std::fmt::Debug, 
    S: std::fmt::Debug + Send + Sync + 'static> 
    TaskTree<J, S>{
    
    // execute all tasks in bfs order in none binary tree
    pub fn execute_all_tasks(&mut self, root: std::sync::Arc<TaskTree<J, S>>){
        let mut queue = vec![root]; 
        while !queue.is_empty(){
            let get_node = queue.pop(); // pop the child out
            if get_node.is_some(){
                let node = get_node.unwrap();
                let cloned_node = node.clone();
                
                // executing the task in the background light thread in a none 
                // blocking io manner, we tried to acquire the lock of the value 
                // in a separate thread to avoid blocking the current thread for doing so
                tokio::spawn(async move{
                    let mut task = cloned_node.task.lock().await;
                    println!("[*] executing the task with id: {:?}", task.id);
                    // this method contains a locking process on the task itself so it's better
                    // to execute it in a separate light io thread
                    task.execute().await;
                });

                let get_children = node.children.try_lock().unwrap();
                let children = get_children.to_vec();
                for child in children{
                    queue.push(child);
                }
            }
        }
    }

    pub fn push_task(&mut self, child: std::sync::Arc<TaskTree<J, S>>){
        let mut get_children = self.children.try_lock().unwrap();
        (*get_children).push(child);
    }

    pub fn pop_task(&mut self) -> Option<std::sync::Arc<TaskTree<J, S>>>{
        let mut get_children = self.children.try_lock().unwrap();
        let poped_task = (*get_children).pop();
        poped_task
    }

}