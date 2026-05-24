#![no_main]
//! NFR-011 fuzz target — `parse_document` must not panic on any input.

use libfuzzer_sys::fuzz_target;
use quire_rs::parse_document;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_document(s);
    }
});
