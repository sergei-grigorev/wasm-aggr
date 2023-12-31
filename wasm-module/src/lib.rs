#![no_std]
extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use arrow::array::Int32Array;
use arrow::compute::sum;
use arrow::ipc::reader::StreamReader;
use arrow::record_batch::RecordBatch;
use wasm_bindgen::prelude::*;

mod host_log;

#[wasm_bindgen]
pub fn sum_func(ptr: *mut u8, len: usize) -> i32 {
    if ptr.is_null() {
        return 0;
    }

    let batch: RecordBatch = {
        let data: Vec<u8> = unsafe { Vec::from_raw_parts(ptr, len, len) };
        let mut stream_reader = StreamReader::try_new(data.as_slice(), None).unwrap();
        stream_reader.next().unwrap().unwrap()
    };

    host_log::log(&format!(
        "function sum has been called with the table rows [{}]",
        batch.num_rows()
    ));

    if let Some(column1) = batch.column(1).as_any().downcast_ref::<Int32Array>() {
        sum(column1).unwrap_or_default()
    } else {
        0i32
    }
}

#[wasm_bindgen]
pub fn wasm_alloc_buffer(size: usize) -> *mut u8 {
    let alligment = arrow::alloc::ALIGNMENT;
    let layout = alloc::alloc::Layout::from_size_align(size, alligment).unwrap();
    unsafe { alloc::alloc::alloc(layout) }
}

#[wasm_bindgen]
pub fn wasm_free_buffer(ptr: *mut u8, size: usize) {
    if ptr.is_null() {
        return;
    }

    let alligment = arrow::alloc::ALIGNMENT;
    let layout = alloc::alloc::Layout::from_size_align(size, alligment).unwrap();
    // safety: this method is called after the allocation method and cannot be called
    // manualy providing the wrong parameters
    unsafe { alloc::alloc::dealloc(ptr, layout) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::{sync::Arc, vec::Vec};
    use arrow::ipc::writer::StreamWriter;
    use arrow::{
        array::Int32Array,
        datatypes::{Field, SchemaBuilder},
    };

    #[test]
    fn it_works() {
        let batch = {
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

            RecordBatch::try_new(Arc::new(schema), vec![Arc::new(column1), Arc::new(column2)])
                .unwrap()
        };

        // serialize to buffer
        let mut buffer: Vec<u8> = Vec::with_capacity(batch.get_array_memory_size() * 2);
        {
            let mut stream_writer = StreamWriter::try_new(&mut buffer, &batch.schema()).unwrap();
            stream_writer.write(&batch).unwrap();
        }

        let result = sum_func(buffer.as_mut_ptr(), buffer.len());
        core::mem::forget(buffer);
        assert_eq!(result, 60);
    }
}
