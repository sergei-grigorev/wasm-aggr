# Experiments with WASM modules

## Build

### Build WASM Module

- go to the directory `aggr`
- run command `wasm-pack build --no-typescript --target web`

### Build and run client app

- go to the directory `client`
- run command `cargo run -p client -- aggr/pkg/aggr_bg.wasm`