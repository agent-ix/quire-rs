#![no_main]
//! NFR-011 fuzz target — `extract_frontmatter` must not panic.

use libfuzzer_sys::fuzz_target;
use quire_rs::extract_frontmatter;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = extract_frontmatter(s);
    }
});
