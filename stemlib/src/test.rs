


use neuron::{AppService, RmqConfig};
use crate::*;
use crate::dsl::*;



#[tokio::test]
pub async fn upAndRun(){

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


    let mut neuron = NeuronActor{
        wallet: Some(wallexerr::misc::Wallet::new_ed25519()),
        internal_executor: InternalExecutor::new(
            Buffer{ events: std::sync::Arc::new(tokio::sync::Mutex::new(vec![
                Event{ data: EventData::default(), status: EventStatus::Committed, offset: 0 }
            ])), size: 100 }
        ), 
        metadata: None,
        internal_worker: None,
        transactions: None,
        internal_locker: None,
        internal_none_async_threadpool: Arc::new(None),
        signal: std::sync::Arc::new(std::sync::Condvar::new()),
        contract: None,
        rmqConfig: Some(RmqConfig{ 
            host: String::from("0.0.0.0"), 
            port: 5672, 
            username: String::from("rabbitmq"), 
            password: String::from("geDteDd0Ltg2135FJYQ6rjNYHYkGQa70") 
        }),
        dependency: std::sync::Arc::new(AppService{}),
        state: 0 // this can be mutated by sending the update state message to the actor
    };

    /* --------------------------
        execution thread process for solving future:
        await on async task suspend it to get the result but won't block thread 
        means the light thread can continue executing other tasks
        future objects are being done in the background awaiting on or polling  
        them tells runtime that we need the result if the future was ready he sends the 
        result to the caller otherwise it forces the thread to get another task 
        from the eventloop to execute it meanwhile the future is being solved, 
        this allows to execute tasks in a none blocking manner 
    */
    neuron.runInterval(|| async move{
        println!("i'm running every 10 seconds, with retries of 12 and timeout 0");
    }, 10, 12, 0).await;

    // streaming over neuron, the on() method belongs the StreamRmq interface
    // enables us to start streaming over a neuron object either locally or 
    // remotely, the method however takes a callback closure in which we have
    // access to the sent or received event.
    neuron
        .on("local", "receive", move |event, error| async move{

            if error.is_some(){
                println!("an error accoured during receiving: {:?}", error.unwrap());
                return;
            }

            println!("received task: {:?}", event);

            // do whatever you want to do with received task:
            // storing in db or cache on redis 
            // ...


        }).await
        .on("rmq", "send", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during sending: {:?}", error.unwrap());
                return;
            }
            
            println!("sent task: {:?}", event);

            // do whatever you want to do with sent task:
            // please shiaf the sent task!
            // ...

        }).await
        .on("rmq", "receive", move |event, error| async move{
            
            if error.is_some(){
                println!("an error accoured during receiving: {:?}", error.unwrap());
                return;
            }

            println!("received task: {:?}", event);

            // store the event in db or cache on redis
            // ...

        }).await;
    
    // starting the neuron actor 
    let nueronComponentActor = neuron.clone().start();
    
    // sending update state message
    nueronComponentActor.send(UpdateState{new_state: 1}).await;


}