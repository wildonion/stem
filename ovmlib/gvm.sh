#!/bin/bash
set -e
sudo npm i wasm-opt -g
curl https://get.wasmer.io -sSfL | sh
curl https://wasmtime.dev/install.sh -sSf | bash
rustup target add wasm32-wasi # compilation target for wasm32 wasi WebAssembly
# rustup target add wasm32-unknown-unknown # compilation target for browser-based WebAssembly
cargo build --bin ovm --target wasm32-wasi --release 
sudo cp ../../../target/wasm32-wasi/release/ovm.wasm ./ovm.wasm
wasm-opt -Oz ovm.wasm -o ovm.wasm # execute default optimization, passes, super-focusing on code
wasmtime ovm.wasm
wasmer run ovm.wasm