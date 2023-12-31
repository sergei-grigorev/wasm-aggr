use std::cell::RefCell;
use std::env;
use std::fs::metadata;
use std::rc::Rc;
use std::sync::Arc;

use arrow::array::Int32Array;
use arrow::datatypes::{Field, SchemaBuilder};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
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

fn run_wasm(file_name: &str) -> Result<u32, WasmError> {
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
    let func_add = find_function::<(u32, u32), u32>(store.clone(), &instance, "sum_func")?;

    // array to be aggregated
    let column1 = Int32Array::from(vec![10, 20, 30]);
    let column2 = Int32Array::from(vec![30, 20, 10]);

    let mut schema = SchemaBuilder::with_capacity(2);
    schema.push(Field::new(
        "column1",
        arrow::datatypes::DataType::Int32,
        false,
    ));
    schema.push(Field::new(
        "column2",
        arrow::datatypes::DataType::Int32,
        false,
    ));
    let schema = schema.finish();

    let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(column1), Arc::new(column2)])
        .map_err(WasmError::ArrowError)?;

    // serialize to byte buffer
    let mut serialized: Vec<u8> = Vec::with_capacity(batch.get_array_memory_size() * 2);
    {
        let mut stream_writer = StreamWriter::try_new(&mut serialized, &batch.schema()).unwrap();
        stream_writer.write(&batch).unwrap();
    }

    // allocate buffer
    let mut buffer = WasmMemory::allocate(serialized.len(), store.clone(), &instance)?;
    // copy an array to the WASM
    let size = buffer.copy(&serialized)?;

    log::trace!("Begin aggregation");
    let result = func_add
        .call(
            store.borrow_mut().as_context_mut(),
            (buffer.as_ptr(), size as u32),
        )
        .map_err(WasmError::FunctionCallFailed)?;
    log::trace!("End aggregation");

    // deallocate buffer (automatically)

    Ok(result)
}
