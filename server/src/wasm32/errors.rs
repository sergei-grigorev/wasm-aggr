use thiserror::Error;

#[derive(Error, Debug)]
pub enum WasmError {
    #[error("Function call has failed: {0}")]
    FunctionCallFailed(String),
    #[error("Arrow serialization failed: {0}")]
    ArrowError(arrow::error::ArrowError),
}
