use wasmtime::{AsContextMut, WasmParams, WasmResults};

use super::errors::WasmError;

/// Find WASM Function.
// It returns a typed function to call or an error.
pub fn find_function<I, O>(
    store: &mut impl AsContextMut,
    instance: &wasmtime::Instance,
    name: &str,
) -> Result<wasmtime::TypedFunc<I, O>, WasmError>
where
    I: WasmParams,
    O: WasmResults,
{
    let func = instance
        .get_func(store.as_context_mut(), name)
        .ok_or_else(|| WasmError::FunctionNotFound(name.to_owned()))?;

    let func = func
        .typed::<I, O>(store.as_context_mut())
        .map_err(|e| WasmError::FunctionIncompatible(name.to_owned(), e))?;
    Ok(func)
}
