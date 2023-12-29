use std::env;
use std::fs::metadata;

use simplelog::SimpleLogger;
use size::Size;
use wasmtime::{AsContextMut, Caller, Engine, Linker, Module, Store, WasmParams, WasmResults};

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

    // print all exported functions
    let mut exports = module.exports();
    while let Some(func) = exports.next() {
        log::info!("Exported function: {}", func.name());
    }

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

    let func_add = find_function::<(u32, u32), u64>(&mut store, &instance, "sum_func").unwrap();

    let func_alloc = find_function::<u32, u32>(&mut store, &instance, "wasm_alloc_buffer").unwrap();

    let func_free =
        find_function::<(u32, u32), ()>(&mut store, &instance, "wasm_free_buffer").unwrap();

    // array to be aggregated
    let array: Vec<u64> = vec![10, 20, 30];
    let array_len = array.len() as u32;

    // allocate buffer
    let buffer = func_alloc
        .call(&mut store, array_len)
        .expect("Function malloc has failed") as u32;

    // copy an array to the WASM
    if let Some(mem) = instance.get_memory(&mut store, "memory") {
        // by the specification WASM has only little endian byte-ordering
        let bytes_buffer: Vec<_> = array.into_iter().flat_map(|d| d.to_le_bytes()).collect();
        mem.write(&mut store, buffer as usize, &bytes_buffer)
            .expect("Copy failed");
    } else {
        panic!("Unexpected memory issue");
    }

    let result = func_add
        .call(&mut store, (buffer, array_len))
        .expect("Function call [add] failed");

    let _ = func_free
        .call(&mut store, (buffer, array_len))
        .expect("Function call [wasm_free_buffer] has failed");

    log::info!("Result: {}", result);
}

fn find_function<I, O>(
    store: &mut impl AsContextMut,
    instance: &wasmtime::Instance,
    name: &str,
) -> Result<wasmtime::TypedFunc<I, O>, String>
where
    I: WasmParams,
    O: WasmResults,
{
    let func = instance
        .get_func(store.as_context_mut(), name)
        .ok_or_else(|| format!("Funtion [{name}] is not found"))?;

    let func = func
        .typed::<I, O>(store.as_context_mut())
        .map_err(|_| format!("Function [{name}] has another interface"))?;
    Ok(func)
}
