use std::env;
use std::fs::metadata;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::Int32Array;
use arrow::datatypes::{Field, SchemaBuilder};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use simplelog::SimpleLogger;
use size::Size;

use crate::wasm32::aggregate;
use crate::wasm32::errors::WasmError;

mod wasm32;

wasmtime::component::bindgen!({
    path: "wit/aggr.wit",
    world: "aggr",
    async: true
});

#[async_std::main]
async fn main() -> anyhow::Result<()> {
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

    let res = run_wasm(&file_name).await?;
    log::info!("Function finished successully. Result: {res}");
    Ok(())
}

async fn run_wasm(file_name: &str) -> Result<u32, anyhow::Error> {
    // array to be aggregated
    let column1 = Int32Array::from(vec![100, 200, 300]);
    let column2 = Int32Array::from(vec![400, 500, 600]);

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

    // the biggest sum of all rows
    let result = aggregate(PathBuf::from(file_name), &serialized).await?;
    assert_eq!(result, 900);

    Ok(result)
}
