#![no_main]
//! NFR-011 fuzz target — DSL YAML parse + extract must not panic.

use libfuzzer_sys::fuzz_target;
use quire_rs::{extract, parse_document, ExtractionDsl};

fuzz_target!(|data: &[u8]| {
    let s = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };
    let dsl: ExtractionDsl = match serde_yaml::from_str(s) {
        Ok(d) => d,
        Err(_) => return,
    };
    let doc = parse_document("## A\nx\n## B\ny");
    let _ = extract(&doc, &dsl);
});
