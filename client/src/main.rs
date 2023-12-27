use std::env;
use std::fs::metadata;

use wasmtime::{Engine, Instance, Module, Store};

fn main() {
    let params: Vec<_> = env::args().collect();
    let file_name = &params[1];
    let metadata = metadata(file_name);
    match metadata {
        Ok(meta) => println!(
            "File [{}] has [{}] mb",
            file_name,
            (meta.len() / 1024 / 1024)
        ),
        Err(e) => {
            eprintln!("{}", e);
            panic!("File does not exists or cannot be open");
        }
    }

    // run wasm module
    let engine = Engine::default();

    let module = Module::from_file(&engine, file_name).expect("Problem openning file");
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("Problem generating an instance");

    let func_add = instance
        .get_func(&mut store, "add")
        .expect("Funtion [add] is not found");
    let func_add = func_add
        .typed::<(u64, u64), u64>(&store)
        .expect("Function [add] has another interface");
    let result = func_add
        .call(&mut store, (1, 3))
        .expect("Function call [add] failed");

    println!("Result: {}", result);
}
