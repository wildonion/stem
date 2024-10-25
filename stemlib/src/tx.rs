

use std::sync::atomic::AtomicU8;
use neuron::ActionType;
use neuron::{Broadcast, RmqPublishConfig};
use wallexerr::misc::Wallet;
use crate::*;
use actix::prelude::*;
use actix::Handler as ActixMessageHandler;



/* -----------------
    https://boats.gitlab.io/blog/post/async-methods-i/
    GATs can now have generic params which allows us to have async 
    method in traits cause the future returned by an async function 
    captures all lifetimes inputed into the function, in order 
    to express that lifetime relationship, the Future type needs 
    to be generic over a lifetime, so that it can capture the 
    lifetime from the self argument on the method.
    generic and lifetime wasn't supported in GAT till Rust 1.79
    by the result we can have async methods in traits without 
    using third party crates.
    the fact that async method wasn't supported in traits was
    due to the unspported feature of generic and lifetime in GAT
    which wouldn't allow to return a future object from the trait
    method cause future obejcts capture lifetimes forces us to pass
    the GAT with lifetime as the return type of async trait method
    hence using the GAT as the return type of async trait method 
    wasn't supported therefore having future objects in trait method 
    return type was invalid.
    it's notable that traits with async methods can't be object safe 
    and Boxed with Box<dyn we can't use the builtin async method 
    instead we should either use the async_trait crate or remove 
    the async keywords.
*/
pub trait TransactionExt{
    type Tx;
    async fn commit(&self) -> Self;
    async fn get_status(&self) -> Self;
    async fn started(&self);
    async fn aborted(&self);
}

pub trait PaymentProcess<M: Send + Sync + 'static>{
    type Status;
    type Output<O: Send + Sync + 'static>;
    type Wallet;
    
    async fn pay<O: Send + Sync + 'static>(&self, gateway: M) -> Self::Output<O>;
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
struct Execute{
    pub tx: Transaction,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
struct Send2Pool{
    pub tx: Transaction,
    pub tx_producer: Addr<NeuronActor>, // use this actor to send produce message to it
    pub spawn_local: bool
}

pub static WALLET: Lazy<std::sync::Arc<tokio::sync::Mutex<wallexerr::misc::Wallet>>> = Lazy::new(||{
    let wallet = Wallet::new_ed25519();
    std::sync::Arc::new(
        tokio::sync::Mutex::new( // since most of the wallet methods are mutable we need to define this to be mutex 
            wallet
        )
    ) // a safe and shareable wallet object between threads
});

/* --------------------- a fintech object solution to do any financial process
    atomic tx syncing execution to avoid deadlocks, race conditions and double spendings using tokio select spawn arc mutex channels
    produce tx actor objects to rmq then consume them in txpool service and update the balance in there
    use notif prodcons actor worker to produce and consume tx objects
    send each tx object into the exchange 
    tx pool service concumes every incoming tx and execute them in the background
    safe tx execution without double spending issue using tokio spawn select mutex and channels
    finally tx pool service publishes the result of executed each tx into the exchange 

    create tx object with unique id -> publish to rmq 
    receive tx in txpool -> commit tx -> execute tx -> update records -> record in treasury
    all or none to avoid double spending which is sending same amount for two destinations but charge only once 

    once a tx object is made publish it to the rmq exchange so consumer 
    can consume it for committing and executing all tx objects finally 
    produce the result to the TxResultExchange so main service can consume 
    it and update the platform based on the result

    1 => create ed25591 wallet keypairs with high entropy seed for its rng
    2 => then build tx object and use sha256 to make a hash of the stringified of tx object
    3 => sign the hash of the tx object with prvkey to create the tx signature
    4 => use pubkey and hash of tx object to verify the signature
    use hex::encode() to generate hex string from utf8 bytes
    use hex::decode() to generate utf8 bytes from hex string
    use base58 and base64 to generate base58 or base64 from utf8 bytes  

    every tx object must atomic: Arc<Mutex<Transaction>> or Arc<RwLock<Transaction>>

*/
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Transaction{
    amount: f64, 
    from: String, 
    to: String, 
    tax: f64,
    action: TxAction,
    memo: String, // this field is required when the exchange is using a single address to deposit tokens instead of a new one per each user 
    data: Vec<u8>, // every tx object can store data in form of bytecode or shellcode that can be executed on some VM
    tx_type: TxType,
    treasury_type: TreasuryType,
    status: TxStatus,
    hash: Option<String>, // sha256 ash of the transaction
    tx_sig: Option<String>, // the signature result of signing the tx hash with private key, this will use to verify the tx along with the pubkey of the signer
    signer: String, // the one who has signed the tx
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum TxAction{
    UpdateCode,
    #[default]
    CallMethod,
    UpdateLibs
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum TxType{
    Withdraw,
    #[default]
    Deposit,
    Buy,
    Airdrop,
    Claim,
    Sell
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum TreasuryType{
    #[default]
    Debit, 
    Credit
}

impl Transaction{
    pub async fn new(last_tx: Self, memo: &str, amount: f64, from: &str, to: &str, tax: f64, data: &[u8]) -> Self{ // a tx object might have some data inside of itself

        // create a new tx object
        let mut tx_data = Self{
            amount,
            from: from.to_string(),
            to: to.to_string(),
            tax,
            action: TxAction::CallMethod,
            memo: memo.to_string(),
            data: data.to_vec(), 
            status: TxStatus::Started,
            tx_type: TxType::Deposit,
            treasury_type: TreasuryType::Credit,
            hash: Some(String::from("")), // stringify the whole tx object then hash it
            tx_sig: Some(String::from("")), // sign the the stringified_tx_object with prvkey
            signer: String::from("") // the one who has signed with the prv key usually the server
        };

        tx_data
        
    }

    pub fn on_error<E, R>(e: E) where E: FnMut() -> Arc<dyn std::future::Future<Output = ()>> + Send + Sync + 'static, 
    { // error is of type a closure trait
        let arced_e = Arc::new(e);
    }

    pub fn on_success<S, R>(s: S) where S: FnMut() -> Arc<dyn std::future::Future<Output = ()>> + Send + Sync + 'static, 
    { // success is of type a closure trait
        let arced_s = Arc::new(s);
    }

    pub fn on_reject<J, R>(r: J) where J: FnMut() -> Arc<dyn std::future::Future<Output = ()>> + Send + Sync + 'static, 
    { // reject is of type a closure trait
        let arced_r = Arc::new(r);
    }

    pub fn queueTx(&mut self){
        self.status = TxStatus::Queued
    }

    pub fn pendingTx(&mut self){
        self.status = TxStatus::Pending
    }

    pub fn commitTx(&mut self){
        self.status = TxStatus::Committed
    }

    // implement ALL OR NONE logic : atomicity
    pub fn revertTx(&mut self){

    }
    
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub enum TxStatus{
    #[default]
    Started,
    Queued,
    Pending,
    Committed,
    Dropped,
    Mined,
    Rejected(FailedTx),
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct FailedTx{
    time: i64,
    cause: String
}

// drop the transaction object from the ram
// note that for Rc shared reference, the total reference count 
// of type (type pointers used by different scopes) must reaches 
// zero so the type can gets dropped out of the ram but not for weak. 
impl Drop for Transaction{
    fn drop(&mut self) {
        self.status = TxStatus::Dropped
    }
}

impl TransactionExt for Transaction{
    
    type Tx = Self;

    async fn commit(&self) -> Self {
        
        let mut tx_data = self.clone();
        let mut wallet = WALLET.clone();
        let tx_json = serde_json::json!({
            "amount": tx_data.amount,
            "from": tx_data.from,
            "to": tx_data.to,
            "tax": tx_data.tax,
            "data": tx_data.data,
            "tx_type": tx_data.tx_type,
            "treasury_type": tx_data.treasury_type,
            "signer": tx_data.signer,
        });
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(Option<String>, Option<String>)>(1024);
        
        // use a lightweight thread to lock on the wallet in order to avoid blocking issues
        // basically we should do our async stuffs inside a lightweight thread then use 
        // channels to send the result to outside of the thread scope.
        tokio::spawn(
            {
                let tx = tx.clone();
                async move{
                    let data = serde_json::to_string(&tx_json).unwrap();
                    let mut locked_wallet = wallet.lock().await;
                    let prvkey = locked_wallet.ed25519_secret_key.clone().unwrap();
                    let sig = locked_wallet.self_ed25519_sign(&data, &prvkey);
                    let tx_data_hash = locked_wallet.self_generate_sha256_from(&data);
                    let hex_tx_data_hash = Some(hex::encode(&tx_data_hash));
                    tx.send((sig, hex_tx_data_hash)).await;
                }
            }
        );

        // update tx_sig and hash field simultaneously as they're coming from the background thread
        while let Some((tx_signature, tx_hash)) = rx.recv().await{
            tx_data.tx_sig = tx_signature;
            tx_data.hash = tx_hash;
        }

        // update from and to balances
        // calculate user and sys treasury
        // 2.5 percent of each tx must goes to sys treasury
        // destination amount = current amount - 2.5 % of current amount
        // sys treasury = 2.5 % of current amount 
        // ...

        tx_data.commitTx(); // commit the tx
        tx_data

    }

    async fn get_status(&self) -> Self {
        todo!()
    }

    async fn started(&self){}

    async fn aborted(&self){}

}


struct StatelessTransactionPool{
    pub lock: tokio::sync::Mutex<()>, // the pool is locked and busy 
    pub worker: tokio::sync::Mutex<tokio::task::JoinHandle<()>>, // background worker thread to execute a transaction
    pub queue: std::sync::Arc<tokio::sync::Mutex<Vec<Transaction>>>, // a queue contains all queued transactions
    pub broadcaster: tokio::sync::mpsc::Sender<Transaction>
}

impl StatelessTransactionPool{
    
    pub fn new(broadcaster: tokio::sync::mpsc::Sender<Transaction>) -> Self{
        Self { 
            lock: tokio::sync::Mutex::new(()), 
            worker: tokio::sync::Mutex::new(tokio::spawn(async move{ () })), 
            broadcaster,
            queue: std::sync::Arc::new(tokio::sync::Mutex::new(vec![]))
        }
    }

    pub async fn push(&mut self, tx: Transaction){
        let mut get_queue = self.queue.lock().await;
        (*get_queue).push(tx.clone());
        let sender = self.broadcaster.clone();
        sender.send(tx).await;
    }

}
impl Actor for StatelessTransactionPool{
    
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

    }
}

// msg handler to send the tx object to the rmq, the consumer service
// which is the tx-pool service will consume each one and execute them
// asyncly in the background worker thread
impl ActixMessageHandler<Send2Pool> for StatelessTransactionPool{
    
    type Result = ();
    fn handle(&mut self, msg: Send2Pool, ctx: &mut Self::Context) -> Self::Result {
        
        let tx = msg.tx;
        let producer = msg.tx_producer;
        let spawn_local = msg.spawn_local;
        let cloned_tx = tx.clone();

        // send only queued transaction to the pool
        if tx.status != TxStatus::Queued{
            return;
        }

        let prod_event = 
            Broadcast{
                local_spawn: true,
                notif_data: EventData{ 
                    id: Uuid::new_v4().to_string(), 
                    receiver_info: String::from("txpool"), 
                    action_data: serde_json::to_value(&cloned_tx).unwrap(), 
                    actioner_info: String::from("wallet-service"), 
                    action_type: ActionType::EventCreated, 
                    fired_at: chrono::Local::now().timestamp(), 
                    is_seen: false 
                },
                p2pConfig: None,
                rmqConfig: Some(RmqPublishConfig{
                    exchange_name: format!("{}.notif:TxPool", APP_NAME),
                    exchange_type: String::from("fanout"),
                    routing_key: String::from(""),
                }),
                encryptionConfig: None, // don't encrypt the data 
            };

        // background worker thread using tokio
        let cloned_producer = producer.clone();
        tokio::spawn(async move{
            cloned_producer.send(prod_event).await;
        });


    }

}

// send Execute message to the actor to execute a tx 
impl ActixMessageHandler<Execute> for StatelessTransactionPool{
    type Result = ();

    fn handle(&mut self, msg: Execute, ctx: &mut Self::Context) -> Self::Result {
        
        let tx = msg.tx;
        
        let cloned_tx = tx.clone();
        tokio::spawn(async move{
            cloned_tx.clone().commit().await;
        });

    }

}

// ---------------------------------------------------------
//         SOLID BASED DESIGN PATTERN FOR PAYMENT
// ---------------------------------------------------------
// there should be always a rely solution on abstractions like 
// implementing trait for struct and extending its behaviour 
// instead of changing the actual code base and concrete impls. 
struct PayPal; // paypal gateway
struct ZarinPal; // zarinpal gateway

struct PaymentWallet{
    pub owner: String,
    pub id: uuid::Uuid,
    pub transactions: Vec<Transaction> // transactions that need to be executed
}

impl PaymentProcess<PayPal> for PaymentWallet{
    type Status = AtomicU8;

    // future traits as objects must be completelly a separate type
    // which can be achieved by pinning the boxed version of future obj 
    type Output<O: Send + Sync + 'static> = std::pin::Pin<Box<dyn std::future::Future<Output = O>>>;
    
    type Wallet = Self;

    async fn pay<O: Send + Sync + 'static>(&self, gateway: PayPal) -> Self::Output<O> {

        // either clone or borrow it to avoid from moving out of the self 
        // cause self is behind reference which is not allowed by Rust 
        // to move it around or take its ownership.
        let txes: &Vec<Transaction> = self.transactions.as_ref();
        
        // process all transactions with the paypal gateway
        // ...

        todo!()

    }
}

impl PaymentProcess<ZarinPal> for PaymentWallet{
    type Status = AtomicU8;

    // future traits as objects must be completelly a separate type
    // which can be achieved by pinning the boxed version of future obj 
    type Output<O: Send + Sync + 'static> = std::pin::Pin<Box<dyn std::future::Future<Output = O>>>;
    
    type Wallet = Self;

    async fn pay<O: Send + Sync + 'static>(&self, gateway: ZarinPal) -> Self::Output<O> {

        // either clone or borrow it to avoid from moving out of the self 
        // cause self is behind reference which is not allowed by Rust 
        // to move it around or take its ownership.
        let txes: &Vec<Transaction> = self.transactions.as_ref();
        
        // process all transactions with the paypal gateway
        // ...
        

        todo!()

    }
}