use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    #[wasm_bindgen_test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
