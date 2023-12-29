#![no_std]
extern crate alloc;
use alloc::slice::from_raw_parts;

use alloc::format;
use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

mod host_log;

#[wasm_bindgen]
pub fn sum_func(arr_ptr: *const u64, len: usize) -> u64 {
    host_log::log(&format!(
        "function sum has been called with the array length [{}]",
        len
    ));

    let arr = unsafe { from_raw_parts(arr_ptr, len) };
    arr.into_iter().sum()
}

#[wasm_bindgen]
pub fn wasm_alloc_buffer(size: usize) -> *mut u64 {
    let mut vec = Vec::with_capacity(size);
    let ptr = vec.as_mut_ptr();
    core::mem::forget(vec);
    ptr
}

#[wasm_bindgen]
pub fn wasm_free_buffer(ptr: *mut u64, len: usize) {
    let vec = unsafe { Vec::from_raw_parts(ptr, len, len) };
    core::mem::drop(vec);
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn it_works() {
        let array = vec![10, 15, 20];
        let arr_length = array.len();
        let arr_ptr = array.as_ptr();
        let result = sum_func(arr_ptr, arr_length);
        assert_eq!(result, 45);
    }
}
