extern crate parity_wasm;

use std::env;

use parity_wasm::builder;
use parity_wasm::elements;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} output_file.wasm", args[0]);
        return;
    }

    let module = builder::module()
        .function()
            .signature().with_param(elements::ValueType::I32).build()
            .body().build()
            .build()
        .build();

    parity_wasm::serialize_to_file(&args[1], module).unwrap();
}