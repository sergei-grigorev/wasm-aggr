#![no_std]
extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use arrow::array::Int32Array;
use arrow::compute::sum;
use arrow::ipc::reader::StreamReader;
use arrow::record_batch::RecordBatch;
use core::ptr::NonNull;
use thiserror_no_std::Error;
use wasm_bindgen::prelude::*;

mod host_log;

#[derive(Error, Debug)]
enum AggregateError {
    #[error("Cannot decode RecordBatch")]
    DecodingError,
    #[error("Data has a wrong format")]
    CastError,
}

#[wasm_bindgen]
pub fn sum_func(ptr: *mut u8, len: usize) -> i32 {
    if ptr.is_null() {
        return 0;
    }

    let ptr = unsafe { NonNull::new_unchecked(ptr) };
    match sum_func_internal(ptr, len) {
        Ok(res) => res,
        Err(err) => {
            host_log::log(&format!("Aggregation function failed with an error: {err}"));
            0
        }
    }
}

fn sum_func_internal(ptr: NonNull<u8>, len: usize) -> Result<i32, AggregateError> {
    let batch: RecordBatch = {
        let data: Vec<u8> = unsafe { Vec::from_raw_parts(ptr.as_ptr(), len, len) };
        let mut stream_reader = StreamReader::try_new(data.as_slice(), None)
            .map_err(|_| AggregateError::DecodingError)?;
        if let Some(elem) = stream_reader.next() {
            elem.map_err(|_| AggregateError::DecodingError)?
        } else {
            return Err(AggregateError::DecodingError);
        }
    };

    host_log::log(&format!(
        "function sum has been called with the table rows [{}]",
        batch.num_rows()
    ));

    if let Some(column1) = batch.column(1).as_any().downcast_ref::<Int32Array>() {
        Ok(sum(column1).unwrap_or_default())
    } else {
        Err(AggregateError::CastError)
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
