use std::env;
use std::fs::metadata;
use std::str;

use simplelog::SimpleLogger;
use wasmtime::{Caller, Engine, Extern, Linker, Module, Store};

fn main() {
    // init simple configuration for the logger
    SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default()).unwrap();

    let params: Vec<_> = env::args().collect();
    let file_name = &params[1];
    let metadata = metadata(file_name);
    match metadata {
        Ok(meta) => log::info!(
            "File [{}] has [{}] mb",
            file_name,
            (meta.len() / 1024 / 1024)
        ),
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
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };
                let data = mem
                    .data(&caller)
                    .get(ptr as u32 as usize..)
                    .and_then(|arr| arr.get(..len as u32 as usize));
                let string = match data {
                    Some(data) => match str::from_utf8(data) {
                        Ok(s) => s,
                        Err(_) => panic!("invalid utf-8"),
                    },
                    None => panic!("pointer/length out of bounds"),
                };
                log::info!("WASM: {}", string);
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
