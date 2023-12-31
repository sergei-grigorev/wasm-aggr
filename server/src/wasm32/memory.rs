use std::{cell::RefCell, rc::Rc};

use wasmtime::{AsContextMut, Instance};

use super::{errors::WasmError, helpers::find_function};

pub struct WasmMemory<'a, S: AsContextMut> {
    ptr: usize,
    elements: usize,
    store: Rc<RefCell<S>>,
    instance: &'a Instance,
}

impl<'a, S: AsContextMut> WasmMemory<'a, S> {
    pub fn allocate(
        elements: usize,
        store: Rc<RefCell<S>>,
        instance: &'a Instance,
    ) -> Result<WasmMemory<'a, S>, WasmError> {
        let alloc_func = find_function::<u32, u32>(store.clone(), &instance, "wasm_alloc_buffer")?;

        let mut mut_store = store.borrow_mut();
        let ptr = alloc_func
            .call(mut_store.as_context_mut(), elements as u32)
            .map_err(WasmError::FunctionCallFailed)? as usize;
        drop(mut_store);

        Ok(WasmMemory {
            ptr,
            elements,
            store,
            instance,
        })
    }

    pub fn as_ptr(&self) -> u32 {
        self.ptr as u32
    }

    pub fn copy_array(&mut self, arr: &[u64]) -> Result<(), WasmError> {
        let mut mut_store = self.store.borrow_mut();

        // copy an array to the WASM
        if let Some(mem) = self
            .instance
            .get_memory(mut_store.as_context_mut(), "memory")
        {
            // by the specification WASM has only little endian byte-ordering
            let bytes_buffer: Vec<_> = arr.into_iter().flat_map(|d| d.to_le_bytes()).collect();
            mem.write(mut_store.as_context_mut(), self.ptr, &bytes_buffer)
                .map_err(WasmError::MemoryWriteError)?;
            Ok(())
        } else {
            Err(WasmError::MemoryBlockIsNotFound("memory".to_owned()))
        }
    }

    fn free(&mut self) -> Result<(), WasmError> {
        let free_func = find_function::<(u32, u32), ()>(
            self.store.clone(),
            &self.instance,
            "wasm_free_buffer",
        )?;

        free_func
            .call(
                self.store.borrow_mut().as_context_mut(),
                (self.ptr as u32, self.elements as u32),
            )
            .map_err(WasmError::FunctionCallFailed)?;

        Ok(())
    }
}

impl<'a, C: AsContextMut> Drop for WasmMemory<'a, C> {
    fn drop(&mut self) {
        if let Err(e) = self.free() {
            log::error!("WASM memory free error: {e}")
        }
    }
}
