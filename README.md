# Experiments with WASM modules

## Build

### Build WASM Module

- go to the directory `aggr`
- run command `wasm-pack build --no-typescript --target dyno`

### Build and run server app

- go to the directory `server`
- run command `cargo run -p server -- aggr/pkg/aggr_bg.wasm`
