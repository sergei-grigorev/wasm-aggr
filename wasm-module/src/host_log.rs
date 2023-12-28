mod wasm32;
use wasm32::log_info;

/// Call host logging.
pub fn log(message: &str) {
    let ptr = message.as_ptr() as i32;
    let len = message.len().try_into().unwrap();

    unsafe { log_info(ptr, len) };
}
