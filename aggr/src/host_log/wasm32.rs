#[link(wasm_import_module = "host")]
extern "C" {
    fn log_info(ptr: *const u8, len: usize);
}

pub fn log(message: &str) {
    let ptr = message.as_ptr();
    let len = message.len();

    unsafe { log_info(ptr, len) };
}
