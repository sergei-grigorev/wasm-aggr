use thiserror::Error;
use wasmtime::{Error as WError, MemoryAccessError};

#[derive(Error, Debug)]
pub enum WasmError {
    #[error("Cannot find a function [{0}]")]
    FunctionNotFound(String),
    #[error("Function signature [{0}] is not compatible [{1}]")]
    FunctionIncompatible(String, WError),
    #[error("pointer/length out of bound")]
    InvalidPointers,
    #[error("Invalid utf-8")]
    InvalidUtf8,
    #[error("Out of WASM Memory")]
    OutOfMemory,
    #[error("Problem updating WASM memory: {0}")]
    MemoryWriteError(MemoryAccessError),
    #[error("Memory function [{0}] is not found")]
    MemoryBlockIsNotFound(String),
    #[error("Function call has failed: {0}")]
    FunctionCallFailed(WError),
    #[error("WASM module is corrupted and cannot be loaded: {0}")]
    ModuleCorrupted(WError),
    #[error("Cannot create exported function [{0}]: {1}")]
    CannotMakeFunction(String, WError),
    #[error("Module instantiation failed: {0}")]
    InstantiateFailed(WError),
    #[error("Apache Arrow error: {0}")]
    ArrowError(arrow::error::ArrowError),
}
