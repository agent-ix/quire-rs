#![no_main]
//! NFR-011 fuzz target — manifest YAML parse must not panic.

use libfuzzer_sys::fuzz_target;
use quire_rs::loader::manifest::parse_manifest;

fuzz_target!(|data: &[u8]| {
    let _ = parse_manifest(data);
});
