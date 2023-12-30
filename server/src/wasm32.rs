use wasmtime::{Caller, Extern};

use std::str;

use self::errors::WasmError;
pub mod errors;
pub mod helpers;
pub mod memory;

/**
 * Read string from the WASM memory and copy to the app memory.
 */
pub fn translate_str<'a>(
    caller: &'a mut Caller<'_, ()>,
    ptr: i32,
    len: i32,
) -> Result<&'a str, WasmError> {
    read_string(caller, ptr as *const u8, len as usize)
}

fn read_string<'a>(
    caller: &'a mut Caller<'_, ()>,
    ptr: *const u8,
    len: usize,
) -> Result<&'a str, WasmError> {
    if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
        let data = mem
            .data(caller)
            .get(ptr as u32 as usize..)
            .and_then(|arr| arr.get(..len as u32 as usize));
        if let Some(data) = data {
            if let Ok(str) = str::from_utf8(data) {
                Ok(str)
            } else {
                Err(WasmError::InvalidUtf8)
            }
        } else {
            Err(WasmError::InvalidPointers)
        }
    } else {
        Err(WasmError::FunctionNotFound("memory".to_owned()))
    }
}
