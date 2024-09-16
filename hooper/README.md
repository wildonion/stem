


### Hooper VoD and audio live streamer upon TCP, gRPC, Capnp RPC and P2P (WebSocket, WebRTC (stun,turn), Gossipsub, QUIC) protocols and GStreamer or FFmpeg codecs

```bash
# ------------------------------
# ------ hooper servers --------
# ------------------------------
# launch as grpc with freshing db
cargo run --bin hooper -- --server grpc --fresh # default is grpc and fresh migrations
# launch as tcp with freshing db
cargo run --bin hooper -- --server tcp --fresh
# launch as p2p with freshing db
cargo run --bin hooper -- --server p2p --fresh
# or see help
cargo run --bin hooper -- --help
```