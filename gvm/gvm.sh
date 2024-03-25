#!/bin/bash
set -e
sudo npm i wasm-opt -g
curl https://get.wasmer.io -sSfL | sh
curl https://wasmtime.dev/install.sh -sSf | bash
rustup target add wasm32-wasi # compilation target for wasm32 wasi WebAssembly
# rustup target add wasm32-unknown-unknown # compilation target for browser-based WebAssembly
cargo build --bin gvm --target wasm32-wasi --release 
sudo cp ../../../target/wasm32-wasi/release/gvm.wasm ./gvm.wasm
wasm-opt -Oz gvm.wasm -o gvm.wasm # execute default optimization, passes, super-focusing on code
wasmtime gvm.wasm
wasmer run gvm.wasm