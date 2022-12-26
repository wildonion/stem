#!/bin/bash
set -e
sudo npm i wasm-opt -g
curl https://wasmtime.dev/install.sh -sSf | bash
rustup target add wasm32-wasi
cargo build --bin walleXerr --target wasm32-wasi --release
sudo cp ../../../target/wasm32-wasi/release/walleXerr.wasm ./walleXerr.wasm
wasmtime walleXerr.wasm
wasm-opt -Oz walleXerr.wasm -o walleXerr.wasm # execute default optimization, passes, super-focusing on code