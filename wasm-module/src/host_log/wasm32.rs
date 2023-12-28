#[cfg(not(test))]
#[link(wasm_import_module = "host")]
extern "C" {
    pub fn log_info(ptr: i32, len: i32);
}

#[cfg(test)]
pub unsafe fn log_info(_: i32, _: i32) {}
