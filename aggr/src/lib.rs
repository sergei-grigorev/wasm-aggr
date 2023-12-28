#![no_std]
extern crate alloc;
use alloc::format;
use wasm_bindgen::prelude::*;

mod host_log;

#[wasm_bindgen]
pub fn add(left: u64, right: u64) -> u64 {
    host_log::log(&format!(
        "function add has been called with params [{}] and [{}]",
        left, right,
    ));

    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
