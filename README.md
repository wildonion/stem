


## Project STEM 🤖

> Actor based model of the brain 🧠 based on DQL and MDP frameworks for AI and neuron computations which can be used and injected as a dynamic library or shared object (`.so`) file in other codes and as a `.wasm` file into `js` projects.

**STEM** is an injectable brain model with every neuron leveraging the power of actor worker as a multithreaded and parallel based smart entity, the communication between each neuron however can be taken place either remotely or locally by calling each other's method directly or sending async message known as electric nerve impulses through synapses to get each other's state eventually make an strong synaptic connection through `actix telepathy`, `redis pubub`, `mpsc` and `actix broker`, same cluster node actors can stablish a communication inside the network through `p2p gossipsub` or `RPC` protocols.

## Setup 
> first clone the repo then install the followings (refer to https://docs.cossacklabs.com/themis/installation/installation-from-packages/ if you don't want to build themis from source):

```bash
wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo apt update -y && sudo apt upgrade && sudo apt install -y libpq-dev pkg-config build-essential libudev-dev libssl-dev librust-openssl-dev
git clone https://github.com/cossacklabs/themis.git
cd themis
make
sudo make install
# install themis on MacOS M1
brew install libthemis
```
## 🧱 WIPs

- update crates, new rust edition, 
- schemas.rs, mathista, informationr.rs
- compile to .so, .wasm