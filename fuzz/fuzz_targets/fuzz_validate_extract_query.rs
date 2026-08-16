#![no_main]
//! NFR-019 fuzz target — the retained query/validate/extract surfaces
//! must never panic on arbitrary input (TC-579).
//!
//! Feeds arbitrary byte slices (as lossy `&str`) into:
//!   - `parse_document`
//!   - `validate_document` (against a warm iso `FR` archetype)
//!   - `extract` (against the same archetype's `body_extraction` DSL)
//!   - the whole-spec query API (`search`, `sections`, `parse_tables`,
//!     `parse_bullet_list`, `extract_diagrams`)
//!
//! Any discovered crash is committed as a regression reproducer
//! (parity with NFR-011-AC-4).

// TC-579, NFR-019-AC-1: this target is the enforcement identity of the row.
//
// The tag is a `//` line rather than the `//!` header above it: the declared
// legacy forms match `//` and `///`, and `//!` matches neither, so a tag
// written there binds nothing. It binds from here — several lines above the
// invocation — because a fuzz target's span is its whole file (CR-061).

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use quire_rs::{
    extract, extract_diagrams, parse_bullet_list, parse_document, parse_tables, search, sections,
    validate_document, Registry,
};

fn registry() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| {
        let module = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("tests")
            .join("fixtures")
            .join("modules")
            .join("iso");
        Registry::load_module(&module).expect("load iso module")
    })
}

fuzz_target!(|data: &[u8]| {
    // Lossy conversion so non-UTF-8 byte sequences are exercised too.
    let s = String::from_utf8_lossy(data);
    let text: &str = &s;

    // parse_document — never panics.
    let doc = parse_document(text);

    // validate_document against a warm archetype.
    if let Some(arch) = registry().archetype("FR") {
        let _ = validate_document(arch, text);
        if let Some(dsl) = arch.body_extraction() {
            let _ = extract(&doc, dsl);
        }
    }

    // Whole-spec query API surfaces.
    let _ = search(&doc, "x");
    let _ = sections(&doc, None);
    let _ = parse_tables(text);
    let _ = parse_bullet_list(text, None);
    let _ = extract_diagrams(&doc, None);
});
