use std::path::PathBuf;

use anyhow::Context;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::preview2::{command, Table, WasiCtx, WasiCtxBuilder, WasiView};

use self::errors::WasmError;

pub mod errors;

wasmtime::component::bindgen!({
    path: "wit/aggr.wit",
    world: "aggr",
    async: true
});

pub async fn aggregate(path: PathBuf, array: &[u8]) -> wasmtime::Result<u32> {
    let mut config = Config::default();
    config.wasm_component_model(true);
    config.async_support(true);

    // run wasm module
    let engine = Engine::new(&config)?;

    // server functions
    let mut linker = Linker::new(&engine);

    // Add the command world (aka WASI CLI) to the linker
    command::add_to_linker(&mut linker).context("Failed to link command world")?;
    let wasi_view = ServerWasiView::new();
    let mut store = Store::new(&engine, wasi_view);

    let component = Component::from_file(&engine, path).context("Component file not found")?;

    let (instance, _) = Aggr::instantiate_async(&mut store, &component, &linker)
        .await
        .context("Failed to instantiate the example world")?;
    instance
        .docs_aggr_aggregation()
        .call_sum_func(&mut store, array)
        .await
        .context("Failed to call sum-func function")?
        .map_err(WasmError::FunctionCallFailed)
        .context("Computation failed")
}

struct ServerWasiView {
    table: Table,
    ctx: WasiCtx,
}

impl ServerWasiView {
    fn new() -> Self {
        let table = Table::new();
        let ctx = WasiCtxBuilder::new().inherit_stdio().build();

        Self { table, ctx }
    }
}

impl WasiView for ServerWasiView {
    fn table(&self) -> &Table {
        &self.table
    }

    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }

    fn ctx(&self) -> &WasiCtx {
        &self.ctx
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}
