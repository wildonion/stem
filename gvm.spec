

▶ ownership, borrowing, interface and Rust lang concepts:
        → pass &'valid pointer instead of clone like passing slice form of dynamic and heap types like String and Vec
        → dep injection by boxing an object safe trait but for future trait their box must get pinned into the ram at an stable address 
        → bound and cast generic to traits and lifetime, break self-ref cycle with: Box, Arc, Arc<Mutex, &'valid, Box::pin 
        → fix address in the ram using pin & fut with Box::pin(async move{}), task: Arc<Fn() -> R> where R: Future<Output()>
        → Box<dyn Error> with ? and custom error handler all impls: Error, From and Display traits, use dyn to dispatch at runtime
        → task, actor(threadpool eventloop), mint, wallet and txpool service with NotifBrokerActor 
        → consumer in both kafka, rmq and redis pubsub is an streamer must be iterated over using while let some
        → kafka consumer per each app with unique id and rmq cosumer per each app with unique queue setup    
        → alwasy use &mut to mutate data, same addr new val by *, new addr same val by 2 &, new addr new binding 
        → future task or job async move{}: Box</Arc<dyn Fn() -> R> where R: Future<Output=()> and Box::pin(async move{})
        → await, !await, mutex, arc, spawn, select, channels, condvar, fut task, Box::pin(async move{}), os vs light threads
        → actor: jobq chan mailbox, cronScheduler (tokio time spawn + loop{}, ctx, redis pubsub exp key) 
        → actor: notif streaming with p2p:tcp,udp,ws,quic,noise,gossipsub redis, bidi grpc+capnpc, ws, kafka, rmq, pg streamer 
        → actor: mutators/accessors controllers and components
        → actor: NotifBrokerActor supports local and tcp based eventloop and jobq based chan using mpsc, rmq, redis, kafka, grpc
        → arc mutex receiver eventloop joq or task queue: mpsc channel, tcp based channels like rmq, bidi capnpc&grpc, redis, kafka, ws
        → receive long running notifications FROM brokers with jobId, shortPolling, websocket streaming
        → serverless smart contract worker: pub async fn handler(req: Req, streamer: Streamer, broker: Broker, depot: Depot, ctx: Context){} 
        → threadpools (Vec<JoinHandle>), condvar, arc and mutex for mutating the buffer of events
        → traits for poly, dyn stat dispatch generic bounding and dep injection used to create custom error handler
        → talking to actor means talking to light thread by using chan cuz each actor is a light thread
        → each actor can execute tasks by using cron scheduling in the background thread 
        → streaming is about listening to await on incoming task or packet as they're coming from the source and gether them all together while let Some(data) = rx.recv().await{} || for await data in streamer{}
        → an eventloop is constantly listening using the jobq like mpsc to receive async tasks and execute them in a separate light thread 
        → each stage has its own jobs or tasks which is an actor contains light thread will be used to execute each task in the background
        → crypter: high entropy hash of seed to generate rng to create ed25519 keypair
        → crypter: encrypted channel using circom, noir in zkp verifier contract
        → crypter: struct instance, base58, base64, hex of hashed, keypair and aes256 encrypted data <---> u8 bytes
        → Rust compiler will call the derfe if the pointer is being dereferenced or drop method if the type is begin dropped out of the ram
        → moving type into new scope allocates new ownership and size on the ram and drop the old one hence there is no access to the old one 
        → deref, derefmut, &mut, *, Box::pin, rc, arc, & to break the cycle of self ref types using &'valid/Arc/Mutex/Box
        → accessing heap data fields on &self takes the ownership of self and move out of it, can't do this if it's behind pointer 
        → return pointer from method with valid or the self lifetime
        → can't move out of data when the data is behind pointer to avoid having dangling pointer
        → can't return pointer from method cause all types get dropped once the function gets executed
        → pointer must have valid and longer lifetime than its scope in order to move it around and we must make sure the underlying type is not moved or dropped to return it from method
        → trait Box<dyn Arc<dyn and impl Trait use cases:
                → T: Arc<dyn Fn() -> R + Send + Sync> where R: Future<Output=()> + Send + Sync | Box::pin(async move{})
                → custom error handler (Error, Display, Debug, From) using Box<dyn Error> to explain themselves through display and debug
                → Pin<Box<Trait, Box<dyn Trait, Arc<dyn Trait, Arc<Mutex<dyn: closures, futures for poly dep inj stat and dyn dispatch, generic bounding
                → dynamic dispatch with &dyn or Box<dyn Arc<dyn
                → trait must be object safe for dynamic dispatching inside the Arc and Box
                → use Box<dyn AnyType> for dynamic typing and dispatch
                → use impl AnyType for static typing and dispatch 
                → use trait for polymorphism like wallet payment portal
                → pass Box<dyn AnyType in struct for dep injection and dynamic dispatch
                → use Box::pin() to pin the future trait objects into the ram
                → use onion, macrocosm and features to create plugins
                → dependency injection using Box<dyn AnyType>
                → passing trait to methods or struct using dyn as dynamic dispatch and impl Trait as static dispatch
                → Traits for static dynamic dispatch, polymorphism, dynamic typing with Any and dependency injection, future traits with box pin
                → dep injection with Box::pin(async move{}), R: Future<Output=()> + Send + Sync + 'static, F: Fn() → R + Send + Sync + 'static
                → closures and futures are traits and ?Sized needs to behind Box and use dyn for dynamic dispatch and dep injection
                → Box<dyn Trait>/Arc<dyn Trait> : dynamic dispatch (object safe trait, pin for self-ref) 
                → impl Triat     : static dispatch 
                → where T: Trait
                → where Trait<Gat: Send>
                → Box::pin self-ref like future objects
                → bounding gat and generics to traits and lifetimes
                → dynamic (object safe trait) and static dispatch 
                → polymorphism in its methods 
                → custom error handler 
                → Box::pin(async move{async_recursive().await})
                → dependency injection and extend structure behaviour
▶ tryout:
        - distributed p2p over all tlps based onion protocol with onion codec and vm like Ton and evm like a p2p 
          file sharing app to notify nodes through gossipsub protocol on top of actors with remote rpc and local 
          mpsc mailbox for message sending logic between other actor contracts through mpsc and rpc channels via 
          atomic actor addr or local ip address, pubkey address in a p2p network and ingress 
        - distributed serverless actor smart contract functions with global storage allocation: 
                1) use abi/idl to create tx object to call actor contract method through rpc (client-2-node) and its ed25519 pubkey address
                2) actor contract receives the message from rpc channel and tries to find the tx bytecode within its compiled bytecode component 
                3) if it matches it then executes the tx bytecode on vm inside validator node
                concepts: contract, encryption, tx hash, tx sig, keypairs signing, distributed p2p infra like ipfs and chain
                every actor contract will be used to mutate the state of a service on the cloud or in p2p based app
                communicate with actor contract (node-2-node) by sending them message through rpc or internal talking within the blockchain itself using the vm   
                actor contract methods can be inovked by using idl or abi to create a tx object to send message to actor contract
                everything on chain has its own ed25519 pubkey address even actor contracts
                actor contract1 <----transaction obj msg through rpc and 256bits ed25519 pubkey----> actor contract2
                struct Contract{ byteCode: ByteCode, state: InitState, owner: Owner}, impl Actor for Contract{}
                every actor contract executes its bytecode on the vm stack machine
                every actor must have 256bits or 32 byte ed25519 pubkey address to find them in the network to send message to them through that 
                every contract is an actor contains instruction bytecode to mutate the state of blockchain in form of tx object which will be executed on the vm then append to a block
                every talk request or message between actor contracts is a transaction object contains tokens that would be sent through capnprpc or mpsc mailbox
                every method that gets called from the contract actor by sending message to that must be in form of a valid tx object
                every contract actor contains its compiled code or bytecode used to execute on validator nodes and its init state components
                pubsub gossipsub subscription process to receive updates from network
                actor address, thread_local, malloc, drop, deref, derefmut, heap box, data collision
                azure, aws and cloudflare vm to execute smart contract methods in v8 wasm and linux bin 
                use abi/idl bytecodes (shellcode) to interact with contract methods from other langs by sending rpc request to chain
                the eventloop jobq based channel in tcp, rpc, ws for rmq, redis, kafka 
        - p2p and graph based serverless smart contract using actor worker concepts like ipfs p2p based market broker and orderbook 
                - emitter, broadcaster, publisher, sender [kafka/rmq/redis/rpc/ws] / receiver, subscriber, ws channels [while let some]
                        https://www.youtube.com/watch?v=rht1vO2MBIg
                        https://medium.com/@harshiljani2002/building-stock-market-engine-from-scratch-in-rust-ii-0c7b5d8a60b6
                        https://github.com/MikeECunningham/rust-trader-public/tree/main 
                        https://github.com/salvo-rs/salvo/tree/main/examples
        - Serverless smart contract and function handlers on cloudflare, aws, azure to compile to wasm to run on v8 and binary to run in linux sandboxes  
        - Every function handler is an smart contract which is an actor that gets deployed on distributed p2p based cloud with graph algos enables us to call its methods through (JSON/G/Capnp)RPC calls
        - dsl macrocosm for shop framework (contract!{}, product!{}, atomic_purchase!{}, contract!{}) and component!{} actor macro over onion arch which contains the notif streamer actor setup
        - send message to update the component actor like set method to update the component state in react
        - gvm (redux and template engin) with macro and dsl and global allocaitons like thread_local, malloc, drop, heap box
        - gmv static lazy arc mutex rwlock lazylock: Send (thread safe), static lazycell and thread_local!{}: !Send (!thread safe)
        - gvm (stack and graph based vm run each graph bytecodes instructions on a virtual register and stack)
        - cloudflare, firebase, azure and aws serverless handlers USING ACTORS AND K8S compile 2 wasm to run on v8 and linux sandbox
        - cloudflare actor worker wasm (wasix, wasmedge, wasi, wasmer) on v8 and aws lambda binary on linux sandboxes: streamer, queue, http, bot events
        - actor service controller, cron scheduler, bot, http/ws api, while let some stream, rmq queue: scale node instance using k8s on firebase, aws, clf, azure
        - p2p and distributed based family tree using merkle tree and high entropy seed to create crypter ed25519 keypairs
        - redis cluster, dht vs redlock, zookeeper, k8s, sharding, raft, ipfs and libp2p (graph algos like dag, mdp, adjmat, merkle tree, gvm allocation engine, wrtc, stun, turn, ws, tcp, udp, quic, kademliadht (find peers which are behind nat over wan), gossipsub, noise protocol)
        - load balancer api gateway with salvo and caddy (resource accessing oauth2: verify requests with ed25519 timehash sig with high entropy seed phrase using wallexerr) => api.app.com/router-service1 | api.app.com/router-service2
        - fire and publish an event like a user update his profile to rmq exchange so his friends actor can bind their own queue to receive the updated data in a fanout way
        - crypter crate (convert keypair, encrypted, sha256 bytes into hex or base58 or base64 chars, zkp, keypairs with high entropy seed for its rng) for zkp secure session, secure streaming channels, ed25519 wallet for ssl and ssh
        - serverless distributed and p2p coding using cloudflare actor workers 
          and aws lambda functions (execute functions in lightweight thread of execution on the cloud):  
                https://github.com/cloudflare/workers-rs
                https://www.cargo-lambda.info/
                https://github.com/capnproto/capnproto-rust/tree/master/capnp-rpc
                https://crates.io/crates/worker
                ▶ in serverless env like smart contract actors, aws and cloudflare functions there is no std or async tools 
                and the code gets compiled to wasm or some executable binary to run them on v8 or linuex sandboxes.
                ▶ each serverless app is an actor that runs functions in its own lightweight thread of execution then
                the whole app gets compiled to wasm or a binary which can be executed on the whole datacenter servers 
                inside the v8 engine or linux sandbxoes behind dockers, responses will be sent to the client from 
                a server which is the most nearest one to the end user.
                ▶ cloudflare actor workers get compiled to wasm which will be executed by the v8 engine on the cloudflare
                cloud hence we can't use std, async and threading crates in wasm unless there are their crates are already 
                written and have compiled to wasm on the other hand aws Lambda functions run on Linux sandboxes., these 
                sandboxes only include the bare minimum functionality for Rust binaries to work means those sandboxes use 
                amazon Linux 2 as the operating system, and by default, sandboxes only include the necessary libraries for 
                the OS to work. *-sys libraries are not guaranteed to work unless they are completely linked to your 
                binary, or you provide the native dependencies in some other way.
                ▶ a serverless function is only a single function that can be coded to handle different kind of events 
                like handling http apis, msg brokerring and cron scheduler or a bot, each serverless app is an actor 
                that gets compiled to wasm or binary and executed on the cloud and shared across the whole datacenters
                without worring about scaling or managing the load of the app, each function will be handled in a
                lightweight thread of execution of the actor itself, actors on the other hand talk with each other 
                or other node actors through sending message over capnp rpc channel.
                ▶ to each handler we can pass the event type since the function itself gets executed and compiled to binary or wasm 
                on the cloud which means there is a little server is running for the app contains this handler to accept and parse 
                events coming from the client calls, since each serverless app is an actor that can talks to the cloud or other nodes 
                through capnp rpc method object calling, the handlers inside of it can get executed in the actor lightweight tokio 
                thread of execution itself and send the result to different parts of the app through mpsc and remotely through 
                capnp rpc channels.
                ▶ rust worker ---> wasm runtime ---> deploy the wasm to all cloudflare servers around the world ---> execute on v8 engine runtime to load in browsers
                ▶ rust functions ---> lambda runtime ---> deploy the binary to all aws servers around the world ---> execute on linux sandboxes
                ▶ contract actor worker object 1 <----RPC/MPSC channels[serialized message(protobuf/capnp)]-----> contract actor worker object 2 
                ▶ actor worker lightweight thread of execution for executing smart contract, streaming apis, serverless 
                methods and functions in a distributed cluster through jobq based channels like capnp rpc and mpsc, they
                gets compiled to wasm to get executed on v8 engines so every actor based contract, serverless worker or 
                functions, component will be compiled to wasm to run them on cloud.
                ▶ an event is an action that can be triggered to execute an asynchronous task or job
                in a lightweight thread of execution of an actor worker so actions or events can be 
                http apis, msg brokerring and cron scheduling.
                ▶ we use rpc to call actor methods remotely in local however we send message through the mpsc 
                channel to the actor itself to call a method or execute some task inside the message handler.
                ▶ are actor worker objects which execute serverless functions like brokerring, http api and 
                cron scheduling on cloud and talk to other node actors through capnp rpc, each actor worker 
                gets compiled to wasm, each of them is a lightweight thread that executes tasks (async/sync) 
                or serverless api or functions inside of itself and send the result to other parts of the 
                app through mpsc joq channels and remotely via capnp rpc. deploying on cloudflare and executing
                the serverless function on the cloud can be done using capnp rpc.
                ▶ no std, tokio and threading libs in wasm workers, it's like lunatic which uses the concept of actor 
                worker and lightweight thread of execution to execute each task or api inside an actor thread which 
                is not expensive but there is no async support in there and it's something like go in golang and tokio 
                spawn in which the task gets executed in the background thread and send the result outside through 
                mpsc channels so wasm based servers and apis on cloud using cloudflare actor workers.
                every serverless function is an async task or job that would gets executed in a lightweight thread of 
                execution of an actor worker which communicate with other server nodes and instance of your deployed 
                serverless functions through capnp rpc, but in local env the talking is done through mpsc channels.
                actor with capnpc for coding distributed object protocol like cloudflare actor workers (async, threading, 
                channels, atomic syncing and locking) in serverless contexts:
                        distributd object protocol can be used to communicate with actor worker object in a remote way using rpc 
                        execute an async job/task in a none blokcing manner inside a lightweitht thread of execution 
                        in the background using tokio spawn then use jobq channels like mpsc to send result to outside
                        of the thread, each thread can be an actor object allows us to commnunicate with them over 
                        local (mpsc) and remote (rpc based like rmq) channels to call each other methods on a distributed
                        cluster. like sending message to smart contracts in a distributed and p2p based arch.
                        the message to be sent between actors using jobq based channels (mpsc and rpc) in remote or local 
                        can be in form of capnp or protobuf which is a codec and serialization protocol to encode/decode 
                        message data between text, binary and json formats.
                ▶ example:
                        compile each actor smart contract methods which communicate with other contract through capnp rpc 
                        to wasm to execute them on chain or a peer in the whole blockchain network, this approach can be 
                        used to run serverless functions, compile them to wasm and execute each code on the cloud across glob
                ▶ goal: 
                        execute a serverless api function in an actor lightweight thread of execution
                        async fn main(req: Request, env: Env, app_ctx: Context) -> Result<Response>{ Response::ok("hello world") }
                ▶ tools: 
                        compile server to wasm to run on cloud using wasi wasmer to run the server on wasm runtime 
                        cloudflare worker, actix actor, capnp rpc, tokio spawn select arc mutex jobq channels (mpsc, rmq broker)
                ▶ event types: 
                        event:‌ ActorWorkerCronScheduler, event: HttpRequest, event: MsgBroker, event: Streamer, event: Bot
                ▶ distributed coding using azure functions, cloudflare worker, aws lambda and smart contracts:
                        azure, cloudflare, aws serverless handlers and functions (capnpc) (streamer, actor, httpapi, broker, bot service) per each event
                        smart contract like tact then compile to binary or wasm to gets executed on chain 
                        serverless function and handlers to handle different types of events then deploy them on cloud to compile to wasm or binary 
                ▶ concepts:
                        deref &mut, mutex, atomic using * to mutate the content, while let some streaming, tokio spawn background and channels
                        object safe trait for dynamic with Box<dyn Trait> and impl Trait for static dispatching
                        break the cycle in self ref types using &'valid, Box, Arc, Mutex and Box::pin(fut) for future objects
                        actor: channel[mpsc,rpc,rmq], worker[tokio spawn], fut/task execution[tokio select], atomic syncing[Atomic,static,arc,mutex,rwlocl]
                finally: call the methods from ui or cli, the scaling and load balancing will be done automatically by the chain or the cloud
                ▶ docker<->docker with container_name:5432, host<->docker with localhost:5433
                ▶ ssl and ssh certs using ring rsa and wallexerr ed25519 ecc curve with aes256 hash of data for ssh, tcp, rpc with tokio-rustls to sign and encrypt the packets with pubkey to pass them through socket
                ▶ generate an ed25519 wallet and a secure cell config and share them between server and clients for making a secure connection
                ▶ from that moment on server and clients communicate with each other by sending encrypted and signed packet through the socket 
                ▶ only the signature and hash data will be sent through the socket so we can verify the signature and decrypt the data
                ▶ cpu task scheduling, weighted round robin dns, vector clock, event loop
                ▶ iptables and ssh tunneling
                ▶ noise-protocol and tokio-rustls to implement ssl protocols
                ▶ simd BTreeMap, HashMap lookup and divide and conquer based vectorization using rayon multithreading
                ▶ reverse proxy for NAT traversal implemented in Rust based macros
                ▶ implement DNS Server in Rust (DNS hijacking and spoofing using mitm tools)
                ▶ a dns server like docker to map the dns to the container ip in host
                ▶ google Search Crawler implemented in Rust (scalable and secure)
                ▶ scalable and Secure Firewall implemented in Rust 
                ▶ ngrok process like turn server: [https://docs.rs/ngrok/latest/ngrok/] || [https://ngrok.com/docs/using-ngrok-with/rust/]
                        ▶ first it'll open a port on local machine 
                        ▶ then it will create a session on that port with a random dns on its servers 
                        ▶ finally it forwards all the traffic to that session to the local port it created
                        ▶ ngrok and ssh vps will starts a server on a random part then forward all the packets 
                        ▶ coming from outside to the localhost it's like: 
                        ngrok is like a gateway, proxy, ingress or a listener allows packets go to it first then to the actual server
                        it acts like turn server allows packets to get redirected to destination if the destination is behind NAT
                        outside <---packet---> ngrok or ssh vps server act like proxy or listener <---packet---> localhost session
                ▶ cloudflare warp vpn
                        ▶ boringtun protocol which is based on wireguard protocol
                        ▶ uses noise protocol with ed25519 encryption
                        ▶ 1111 dns based protocol 
                        ▶ udp and quic for packet sending   
                        ▶ argo routing to send packets to cloudflare gateways
                        ▶ ed25519 digital signature pubkey with chacha20 and aes256 in noise protocol for making vpn
                ▶ V2RAY protocols and 
                        there are bunch of pcs over lan 
                        use vpn to connect to the local dns server
                        get a new local ip over lan behind nat 
                        connect to other lan systems
                ▶ use warp to build a dns over https/tls vpn: hide dns queries inside the https traffic like client requests the dns and the server respond him within the encrypted https packets:
                        ▶ send https request to query the dns addrs
                        ▶ update device dns endpoints with the new ones 
                        ▶ request any websites and apis through those dns
                ▶ VPS configuration according to the source usage of each node 
                        ▶ like dpi to detect anomal packets to coiniXerr server and automatic load balancer and vps config using transformers and drl
                        ▶ OS and a security management app(malware detection) using RL
                        ▶ our VPS must detect the amount of CPU and RAM that every servers needs to get, without running the app
                        ▶ our VPS must detect the number of instances of every servers needs to be run and the load balancing algorithm 
                        bpf based proxy, firewall, vpns, packet sniffer and load balancer

all ltgs in rust ::::: https://github.com/wildonion/rusty/blob/main/src/retbyref.rs#L17
zero copy        ::::: https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1621C6-L1621C6
data collision   ::::: https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/utils.rs#L640 
near rules       ::::: https://github.com/wildonion/smarties/blob/main/contracts/near/NEAR.rules
solana rules     ::::: https://github.com/wildonion/solmarties/blob/main/SOLANA.rules
https://doc.rust-lang.org/nightly/unstable-book/index.html
https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1305
https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1213
https://developerlife.com/2022/02/24/rust-non-binary-tree/#naive-approach-using-weak-and-strong-references
https://developerlife.com/2022/03/12/rust-redux/
https://bevyengine.org/learn/book/introduction/  
https://godotengine.org/
https://docs.ton.org/learn/tvm-instructions/tvm-overview
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
https://rust-unofficial.github.io/patterns/
https://without.boats/blog/
https://github.com/steadylearner/Rust-Full-Stack/
https://book.avr-rust.com/001-introduction.html
https://nnethercote.github.io/perf-book/title-page.html
https://rust-lang-nursery.github.io/rust-cookbook/intro.html
https://smallcultfollowing.com/babysteps/
https://lucumr.pocoo.org/
https://www.lpalmieri.com/
https://blog.yoshuawuyts.com/
https://www.i-programmer.info/programming/theory.html
https://www.i-programmer.info/babbages-bag/
https://without.boats/blog/
https://crates.io/crates/pyo3
https://getstream.io/
httsp://agora.io
https://rust-unofficial.github.io/patterns/patterns/index.html 
https://github.com/wildonion/gvm/wiki/Ownership-and-Borrowing-Rules
https://techblog.skeepers.io/video-streaming-at-scale-with-kubernetes-and-rabbitmq-6e23fd0e75fb
https://www.cloudflare.com/learning/video/what-is-mpeg-dash/
https://en.wikipedia.org/wiki/Head-of-line_blocking -> fix the head of line blocking issue
https://github.com/wildonion/cs-concepts
https://github.com/codepr/tasq
https://dev.to/zeroassumptions/build-a-job-queue-with-rust-using-aide-de-camp-part-1-4g5m
https://poor.dev/blog/what-job-queue/
https://cetra3.github.io/blog/implementing-a-jobq/
https://rodent.club/queue-manager/
https://cetra3.github.io/blog/implementing-a-jobq-with-tokio/
https://tokio.rs/tokio/tutorial/channels
https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html
https://www.fpcomplete.com/blog/http-status-codes-async-rust/
https://github.com/mahdi-shojaee/loole => create something like mpsc
https://rust-random.github.io/book/intro.html => all about high entropy crypto seedable rng
https://github.com/cossacklabs/themis/tree/master/docs/examples/rust => themis sample codes
https://refactoring.guru/design-patterns/catalog
https://sourcemaking.com/design_patterns
https://developers.cloudflare.com/1.1.1.1/encryption/dns-over-https/
https://internetcomputer.org/docs/current/developer-docs/backend/rust/
https://github.com/wildonion/gem/wiki/Realtime-Push-Notification-Strategy
https://drive.google.com/file/d/1-8M8BNMabNPzPZM43ekWqX_D456KaUvT/view => the programmer guides to theory
https://drive.google.com/file/d/14l2B6cdAECz_tIRtQtkf2iYxnc5pDv9S/view?usp=drive_link => consensus timing algo in distributed (clustering/sharding) patterns in backend like ipfs with libp2p + data compression in pubsub pattern based tlps
https://github.com/MoonKraken/youtube/tree/main/KonaaAuth
https://github.com/actix/examples/tree/master
https://github.com/actix/examples/tree/master/protobuf
https://github.com/actix/examples/blob/master/websockets/chat-tcp/src/codec.rs => start a tcp server in a separate tokio::spawn thread which runs the session actor with a custom codec instead of using the websocket codec
https://github.com/wildonion/cs-concepts
https://github.com/wildonion/cs-concepts#-blogs-and-books
https://github.com/wildonion/cs-concepts/blob/main/backend-roadmap.pdf
https://connectivity.libp2p.io/
https://blog.cloudflare.com/rust-nginx-module/
https://github.com/wildonion/cs-concepts?tab=readme-ov-file#-concepts => graph algos for searching actor nodes in distributed network and clusters on wan and lan using p2p kademlia and mdns
https://github.com/wildonion/uniXerr/blob/master/infra/valhalla/coiniXerr/src/tlps/p2p.pubsub.rs
https://github.com/libp2p/rust-libp2p/tree/master/examples
https://github.com/foniod/build-imageshttps://www.qualcomm.com/content/dam/qcomm-martech/dm-assets/documents/RaptorQ_Technical_Overview.pdf
https://docs.peer5.com/guides/production-ready-hls-vod/
https://blog.tempus-ex.com/hello-video-codec/
https://stackoverflow.com/a/56475851
https://www.quora.com/How-do-you-write-a-video-codec
https://coaxion.net/blog/2017/07/writing-gstreamer-applications-in-rust/
https://github.com/security-union/rust-zoom
https://999eagle.moe/posts/rust-video-player-part-1/
https://ffplayout.github.io/
https://bparli.medium.com/adventures-in-rust-and-load-balancers-73a0bc61a192
https://github.com/jsdw/weave
https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs
https://github.com/hyperium/hyper/blob/master/examples/gateway.rs
https://dzone.com/articles/rust-based-load-balancing-proxy-server-with-async
https://truelayer.com/blog/grpc-load-balancing-in-rust
https://medium.com/load-balancer-series/writing-a-http-load-balancer-in-python-using-tdd-theoretical-concepts-fb6dab3e879b
https://kemptechnologies.com/load-balancer/load-balancing-algorithms-techniques
https://github.com/bparli/convey
https://github.com/NicolasLM/nucleon
https://github.com/wildonion/smarties/blob/main/contracts/near/NEAR.rules
https://github.com/wildonion/solmarties/blob/main/SOLANA.rules
https://sui.io/ => vm like sui
https://github.com/mozilla/cbindgen -> generate c bindings and .so from rust code using unsafe coding
codec, virtual machine like move and evm with allocation concepts 
    - macro dsl
    - thread_local, actor id or address, std::alloc, jemalloc, GlobalAlloc, bumpalo and r3bl_rs_utils arena as a global allocator
    - zero copy 
    - null pointer optimiser
    - unique storage key

