

<p align = "center">
  <b>🚧 UNDER ACTIVE DEVELOPMENT 🚧</b> 
</p>

## Project STEM 🤖

**STEM** is an injectable brain model with every neuron leveraging the power of actor worker as a multithreaded and parallel based smart entity, the communication between each neuron however can be taken place either remotely or locally by calling each other's method directly or sending async message known as electric nerve impulses through synapses to get each other's state eventually make an strong synaptic connection through RPC and p2p gossipsub.

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

- update crates, new rust edition, [wasm32 updation](https://blog.rust-lang.org/2024/04/09/updates-to-rusts-wasi-targets.html)
- schemas.rs, mathista, informationr.rs
- compile to .so, .wasm
