default:
    just -l

build:
  cd wasm-module; cargo component build --release

run:
  cd server; cargo run --release -- ../wasm-module/target/wasm32-wasi/release/aggr.wasm
