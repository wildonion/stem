________________
>>> concepts <<<
----------------
https://github.com/wildonion/gvm/wiki/Ownership-and-Borrowing-Rules
actix actor(acter.rs) worker pubsub concepts to build p2p app like ai based nft game market and 
graph(adjmat,gvm) (gvm) allocation engine with oauth2, noir, sui, mdp, mlx, dag, thiserror, 
tauri, yew, wasm(async,multithreaded like spacetimedb) and merkle tree concepts by building graphs using 
    - ctxruninterval to sub constantly in sub actor and loop{} to run the app constantly 
    - move trait as object safely by putting them inside the Box to send them on the heap
    - share ownership and break cycle of self-ref instead of moving using &mut, rc, refcell arc, mutex and mpsc 
        between scopes and threads cause rust move heap data by default and updates all the pointers after moving 
        but can't use them so it's better not to pass them by reference instead of cloning or moving 
    - !Unping data must be pinned at a fixed position in the ram so we can move them safely, since they got fixed in the ram thus their pointer won't be updated after moving and remain valid and the old one
    - don't move if it's behind a pointer and to move pointer the borrow must last and live long enough once it’s moved, 
    - can’t move pointer between different scopes unless the pointer lifetime is greater than the new scope lifetime after moving that's because a reference of a pinned value remains valid
    - pin the boxed future to move the future safely like awaiting on its mutable pointer cause awaits consumes it
    - use Box<dyn std::error::Error> to handle all possibel runtime errors for every type
    - use Result<(), impl Error> to return the exact type of error at runtime
    - can’t start actor inside tokio spawn or the context of tokio::main it must be inside actix_web::main context
    - can't start tokio tcp inside the main function body of the actix_web::main context
    - tokio::spawn() and mpsc for nodejs like async execution
    - wallexerr::ed25519_with_aes_signing for checksum, file digital signature, secure communication and zk logics
    - local based pubsub using mpsc, actixbroker and tcp based pubsub using libp2p and redis and grpc
    - tokio::tcp,channels::mpsc,spawn,time,select,arcmutex,rwlock,asynciotraits,while let Some ::: #[tokio::main]
    - redis::queue,stream,set,get,pubsub 
    - actix http request, stream and ws stream handler also tokio::* ::: #[actix_web::main]
    - ipfs mdp graph::libp2p::kademliadht,gossipsub,noise protocol,quic,tokio::tcp,p2pwebsocketwebrtc
    - serde_json::from/to, std::str::from_utf8, stream:Payload, payload:Multipart for while let some byte streaming 
    - graph with rc refcell in single thread context | graph with arc mutex in multithread context
    - gloabl static lazy arc muted box pin futdata => global allocator in multithread context
    - threadlocal with rc refcell => global allocator in single thread context
    - enum unique storage key, actor id, r3bl_rust_utils, jemalloc and bumpalo arena as global allocator
    - gvmllevm:str<->hex<->bytes<->base58/64,&mut,casting,ltg,boxpin,boxleacking,codec,simd,wallexerr
    - sharing (AppState: Send + Sync + 'static) between threads using mpsc
    - #[actix_web::main] context
        - http  ---> apis, web::Json, web::Path, Payload, Multipart
        - ws    ---> while let some streaming over Payload bytes
        - actor ---> redis and local borker pubsub actors
        - async ---> tokio::spawn(), Box::pin
    - #[tokio::main] context
        - tcp listeners with while let some streaming
        - spawn(acter.rs),channels::mpsc,select,time,arcmutex