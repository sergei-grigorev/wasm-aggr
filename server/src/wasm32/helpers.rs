use std::{cell::RefCell, rc::Rc};
use wasmtime::{AsContextMut, Instance, TypedFunc, WasmParams, WasmResults};

use super::errors::WasmError;

/// Find WASM Function.
/// It returns a typed function to call or an error.
pub fn find_function<I, O>(
    store: Rc<RefCell<impl AsContextMut>>,
    instance: &Instance,
    name: &str,
) -> Result<TypedFunc<I, O>, WasmError>
where
    I: WasmParams,
    O: WasmResults,
{
    let mut mut_store = store.borrow_mut();

    let func = instance
        .get_func(mut_store.as_context_mut(), name)
        .ok_or_else(|| WasmError::FunctionNotFound(name.to_owned()))?;

    let func = func
        .typed::<I, O>(mut_store.as_context_mut())
        .map_err(|e| WasmError::FunctionIncompatible(name.to_owned(), e))?;
    Ok(func)
}
