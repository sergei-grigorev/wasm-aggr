use wasmtime::{AsContextMut, Instance};

use super::{errors::WasmError, helpers::find_function};

pub struct WasmMemory {
    ptr: usize,
    elements: usize,
}

impl WasmMemory {
    pub fn allocate(
        elements: usize,
        store: &mut impl AsContextMut,
        instance: &Instance,
    ) -> Result<WasmMemory, WasmError> {
        let alloc_func =
            find_function::<u32, u32>(&mut store.as_context_mut(), &instance, "wasm_alloc_buffer")?;

        let ptr = alloc_func
            .call(&mut store.as_context_mut(), elements as u32)
            .map_err(WasmError::FunctionCallFailed)? as usize;

        Ok(WasmMemory { ptr, elements })
    }

    pub fn as_ptr(&self) -> u32 {
        self.ptr as u32
    }

    pub fn copy_array(
        &mut self,
        arr: &[u64],
        store: &mut impl AsContextMut,
        instance: &Instance,
    ) -> Result<(), WasmError> {
        // copy an array to the WASM
        if let Some(mem) = instance.get_memory(&mut store.as_context_mut(), "memory") {
            // by the specification WASM has only little endian byte-ordering
            let bytes_buffer: Vec<_> = arr.into_iter().flat_map(|d| d.to_le_bytes()).collect();
            mem.write(store.as_context_mut(), self.ptr, &bytes_buffer)
                .map_err(WasmError::MemoryWriteError)?;
            Ok(())
        } else {
            Err(WasmError::MemoryBlockIsNotFound("memory".to_owned()))
        }
    }

    pub fn free(
        &mut self,
        store: &mut impl AsContextMut,
        instance: &Instance,
    ) -> Result<(), WasmError> {
        let free_func = find_function::<(u32, u32), ()>(
            &mut store.as_context_mut(),
            &instance,
            "wasm_free_buffer",
        )?;

        free_func
            .call(
                &mut store.as_context_mut(),
                (self.ptr as u32, self.elements as u32),
            )
            .map_err(WasmError::FunctionCallFailed)?;

        Ok(())
    }
}
