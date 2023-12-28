use std::env;
use std::fs::metadata;

use simplelog::SimpleLogger;
use size::Size;
use wasmtime::{Caller, Engine, Linker, Module, Store};

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

    // run wasm module
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());

    // load the WASM file
    let module = Module::from_file(&engine, file_name).expect("Problem openning file");

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
        .expect("Problem with registering exported function");

    // create an instance of WARM runtime
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Problem generating an instance");

    let func_add = instance
        .get_func(&mut store, "add")
        .expect("Funtion [add] is not found")
        .typed::<(u64, u64), u64>(&store)
        .expect("Function [add] has another interface");

    let result = func_add
        .call(&mut store, (1, 3))
        .expect("Function call [add] failed");

    log::info!("Result: {}", result);
}
