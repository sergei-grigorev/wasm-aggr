use std::cell::RefCell;
use std::env;
use std::fs::metadata;
use std::rc::Rc;

use simplelog::SimpleLogger;
use size::Size;
use wasm32::memory::WasmMemory;
use wasmtime::{AsContextMut, Caller, Engine, Linker, Module, Store};

use crate::wasm32::errors::WasmError;
use crate::wasm32::helpers::find_function;

mod wasm32;

fn main() {
    // init simple configuration for the logger
    SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default()).unwrap();

    let params: Vec<_> = env::args().collect();
    let file_name = &params[1];
    let metadata = metadata(file_name);
    match metadata {
        Ok(meta) => {
            let file_size = Size::from_bytes(meta.len());
            log::info!("[{}] size: {}", file_name, file_size)
        }
        Err(e) => {
            log::error!("WASM file metadata receiver failed: {}", e);
            panic!("File does not exists or cannot be open");
        }
    }

    match run_wasm(&file_name) {
        Ok(res) => log::info!("Function finished successully. Result: {res}"),
        Err(err) => log::error!("Function has failed: {err}"),
    }
}

fn run_wasm(file_name: &str) -> Result<u64, WasmError> {
    // run wasm module
    let engine = Engine::default();

    // server functions
    let mut linker = Linker::new(&engine);
    linker
        .func_wrap(
            "host",
            "log_info",
            |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
                let caller = &mut caller;
                match wasm32::translate_str(caller, ptr, len) {
                    Ok(msg) => log::info!("WASM log_info: {}", msg),
                    Err(e) => log::error!("WASM log_info: {}", e),
                };
            },
        )
        .map_err(|e| WasmError::CannotMakeFunction("host.log_info".to_owned(), e))?;

    // load the WASM file
    let module = Module::from_file(&engine, file_name).map_err(WasmError::ModuleCorrupted)?;

    // print all exported functions
    let mut exports = module.exports();
    while let Some(func) = exports.next() {
        log::debug!("Exported function: {}", func.name());
    }

    // Context store
    let store = Store::new(&engine, ());
    let store = Rc::new(RefCell::new(store));

    // create an instance of WASM runtime
    let instance = linker
        .instantiate(store.borrow_mut().as_context_mut(), &module)
        .map_err(WasmError::InstantiateFailed)?;

    // function to run aggregation
    let func_add = find_function::<(u32, u32), u64>(store.clone(), &instance, "sum_func")?;

    // array to be aggregated
    let array: Vec<u64> = vec![10, 20, 30];
    let array_len = array.len();

    // allocate buffer
    let mut buffer = WasmMemory::allocate(array_len, store.clone(), &instance)?;
    // copy an array to the WASM
    buffer.copy_array(&array)?;

    log::trace!("Begin aggregation");
    let result = func_add
        .call(
            store.borrow_mut().as_context_mut(),
            (buffer.as_ptr(), array_len as u32),
        )
        .map_err(WasmError::FunctionCallFailed)?;
    log::trace!("End aggregation");

    // deallocate buffer (automatically)

    Ok(result)
}
