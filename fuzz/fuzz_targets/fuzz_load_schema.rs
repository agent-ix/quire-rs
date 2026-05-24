#![no_main]
//! NFR-011 fuzz target — JSON Schema compile must not panic.

use libfuzzer_sys::fuzz_target;
use quire_rs::loader::compile::compile_schema;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    let s = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };
    let v: Value = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return,
    };
    let _ = compile_schema(&v);
});
