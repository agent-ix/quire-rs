#![no_main]
//! Fuzz source-level trace trailer attachment in the Rust symbol adapter.
//!
//! The extractor parses source without compiling or executing it. Repeating
//! extraction is the healthy determinism oracle; arbitrary trailers and
//! malformed attributes must not panic or produce order-dependent symbols.

use libfuzzer_sys::fuzz_target;
use quire_rs::symbols::extract_file;
use quire_rs::traceability::SourceLanguage;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    let first = extract_file("fuzzed.rs", SourceLanguage::Rust, &source);
    let second = extract_file("fuzzed.rs", SourceLanguage::Rust, &source);
    assert_eq!(first, second, "trace trailer extraction is not deterministic");
});
