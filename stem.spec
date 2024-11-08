
ُREAD: desktop books for neuroscience and information theory
READ: quantum computing, codeforces, graph and ai algos with nalgebra
READ: publish stemlib to crate cicd from main branch
TODOs:
        1 -> neuron message handlers, dsl/mod.rs macros, stemplugins and self.on() method
        2 -> p2p concepts and network behavior: stream, request response, kademlia, gossipsub, start swarm eventloop
                SYNAPSE protocol features1: file sharing, vpn like tor, ton and v2ray, firewall, gateway, 
                SYNAPSE protocol features2: loadbalancer, ingress listener like ngrok, proxy and dns server
                ▶ gaming, Ai pipelining, robotic blockchain dex and cex with Rust using 
                  cloudflare worker wrangler with neuron actor cli for the p2p based Dex and Cex:
                  they talk with each other through messge sending we can use 
                        Dex AMM liquidity pool, escrow and orders contracts.
                        create bridge between chains
                        Cex broker order book and match engine rmq contains all orders.
                        Atomic orderTx with neuron actor 
                        Ws, wrtc, tcp, udp, ed25519 noise for live orders with IPFS raft crypter graph concept
                ▶ onion protocol with tcp, quic, wrtc, ws, udp and p2p, os, codec like ffmpeg and Gstreamer
                ▶ cpu task scheduling, weighted round robin dns, vector clock
                ▶ iptables and ssh tunneling
                ▶ simd BTreeMap, HashMap lookup and divide and conquer based vectorization using rayon multithreading
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
        3 -> AppEvents enum with #[event] proc macro to broadcast events with neuron actor during execution 
        4 -> neuron crypter based operations:
                contract and wallet over zk
                noise (ed25519) and rustls for secure communication between neurons in a brain (playground/app.rs)
                #[inject(ram, network=p2p)] proc macro on top of an io task to distribute shellcode of the compressed, encoded and encrypted neuron object into the ram and through the network
https://www.youtube.com/watch?v=rht1vO2MBIg
https://medium.com/@harshiljani2002/building-stock-market-engine-from-scratch-in-rust-ii-0c7b5d8a60b6
https://github.com/MikeECunningham/rust-trader-public/tree/main 
https://github.com/salvo-rs/salvo/tree/main/examples
https://github.com/cloudflare/workers-rs
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
https://github.com/mozilla/cbindgen -> generate c bindings and .so from rust code using unsafe coding
https://github.com/mozilla/cbindgen -> generate c bindings and .so from rust code
https://github.com/wildonion/cs-concepts
https://github.com/alordash/newton-fractal
https://github.com/Patryk27/shorelark/ -> GA, NN and WASM
https://crates.io/crates/wasmtime
https://wasmer.io/
https://os.phil-opp.com/
all ltgs in rust ::::: https://github.com/wildonion/rusty/blob/main/src/retbyref.rs#L17
zero copy        ::::: https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1621C6-L1621C6
data collision   ::::: https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/utils.rs#L640 
near rules       ::::: https://github.com/wildonion/smarties/blob/main/contracts/near/NEAR.rules
solana rules     ::::: https://github.com/wildonion/solmarties/blob/main/SOLANA.rules
design patters:  
        https://www.hackingwithrust.net/2023/06/03/the-decorator-pattern-an-easy-way-to-add-functionality/
        https://www.hackingwithrust.net/2023/05/28/design-patterns-in-rust-flyweight-or-go-easy-on-your-memory/
        https://www.hackingwithrust.net/2023/05/01/a-simple-quaternion-library-or-a-lesson-in-operator-overloading/
        https://www.hackingwithrust.net/2023/09/23/design-patterns-in-rust-easy-container-traversing/
        https://www.hackingwithrust.net/2023/10/20/easy-patterns-in-rust-the-adapter-pattern/
        https://www.hackingwithrust.net/2023/10/23/a-composite-pattern-in-rust/
        https://www.hackingwithrust.net/2023/03/12/design-patterns-in-rust-proxy/
        https://www.hackingwithrust.net/2023/04/16/design-patterns-in-rust-mediator-or-uncoupling-objects-made-easy/
        https://www.hackingwithrust.net/2023/04/10/design-patterns-in-rust-memento-or-how-to-undo-your-actions/
        https://www.hackingwithrust.net/2023/04/08/design-patterns-in-rust-the-state-pattern/
        https://www.hackingwithrust.net/2023/04/01/builder-pattern-in-rust-a-generic-solution/
        https://www.hackingwithrust.net/2023/03/27/design-patterns-in-rust-observer/
        https://www.hackingwithrust.net/2023/03/25/design-patterns-in-rust-visitor/
        https://www.hackingwithrust.net/2023/03/29/design-patterns-in-rust-prototype-or-creating-your-own-clone-and-debug-implementations/
        https://www.hackingwithrust.net/2023/03/24/design-patterns-in-rust-builder-pattern/
        https://www.hackingwithrust.net/2023/06/03/the-decorator-pattern-an-easy-way-to-add-functionality/
        https://www.hackingwithrust.net/2023/05/28/design-patterns-in-rust-flyweight-or-go-easy-on-your-memory/
        https://www.hackingwithrust.net/2023/04/30/design-patterns-in-rust-chain-of-responsibility-there-is-more-than-one-way-to-do-it/
        https://www.hackingwithrust.net/2023/04/24/design-patterns-in-rust-singleton-a-unique-way-of-creating-objects-in-a-threadsafe-way/
        https://www.hackingwithrust.net/2023/04/17/design-patterns-in-rust-facade-hiding-a-complex-world/
        https://www.hackingwithrust.net/2023/04/23/design-patterns-in-rust-the-command-a-simple-implementation-of-a-versatile-pattern/
        https://www.hackingwithrust.net/2023/04/16/design-patterns-in-rust-interpreter-making-sense-of-the-world/
        https://www.hackingwithrust.net/2023/03/30/design-patterns-in-rust-strategy/
        https://www.hackingwithrust.net/2023/11/26/easy-mastery-a-deep-dive-into-the-active-object-pattern-in-rusts-seamless-concurrency-model/
        https://www.hackingwithrust.net/2023/11/12/serving-simplicity-mastering-the-servant-pattern-in-rust-for-easy-and-elegant-code-design/
        https://www.hackingwithrust.net/2023/10/29/unlocking-the-power-of-rust-exploring-the-extension-object-pattern-for-ultimate-flexibility/
        https://www.hackingwithrust.net/2023/10/28/easy-delegation-in-rust-the-delegation-pattern/
        https://www.hackingwithrust.net/2023/11/05/a-guide-to-flexible-easy-thread-safe-rust-unveiling-the-multiton-pattern-for-efficient-lazy-initialization/