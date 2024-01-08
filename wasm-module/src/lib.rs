#![no_std]
cargo_component_bindings::generate!();
use crate::bindings::exports::docs::aggr::aggregation::Guest;

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use arrow::array::Int32Array;
use arrow::compute::sum;
use arrow::ipc::reader::StreamReader;
use arrow::record_batch::RecordBatch;
use thiserror_no_std::Error;

#[derive(Error, Debug)]
enum AggregateError {
    #[error("Cannot decode RecordBatch")]
    DecodingError,
    #[error("Data has a wrong format")]
    CastError,
}

struct Component;

impl Guest for Component {
    fn sum_func(array: Vec<u8>) -> Result<u32, String> {
        match sum_func_internal(&array) {
            Ok(res) => Ok(res),
            Err(err) => Err(format!("Aggregation function failed with an error: {err}")),
        }
    }
}

fn sum_func_internal(array: &[u8]) -> Result<u32, AggregateError> {
    let batch: RecordBatch = {
        let mut stream_reader =
            StreamReader::try_new(array, None).map_err(|_| AggregateError::DecodingError)?;
        if let Some(elem) = stream_reader.next() {
            elem.map_err(|_| AggregateError::DecodingError)?
        } else {
            return Err(AggregateError::DecodingError);
        }
    };

    // host_log::log(&format!(
    //     "function sum has been called with the table rows [{}]",
    //     batch.num_rows()
    // ));

    if let Some(column1) = batch.column(1).as_any().downcast_ref::<Int32Array>() {
        Ok(sum(column1).unwrap_or_default() as u32)
    } else {
        Err(AggregateError::CastError)
    }
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

        let result = Component::sum_func(buffer).unwrap();
        assert_eq!(result, 60);
    }
}
