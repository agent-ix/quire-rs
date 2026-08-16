#![no_main]
//! NFR-011 fuzz target (v0.3) — the corpus construction path must not
//! panic on adversarial document content.
//!
//! `load_repo`'s filesystem walk is upstream-tested (`ignore` + rayon),
//! so we fuzz the *bytes-in* path we own: parse a fuzzed document, wrap
//! it as a `LoadedDocument`, build a `Spec` (indexing + resolution),
//! and run every query. Any panic is a bug.

use libfuzzer_sys::fuzz_target;
use quire_rs::corpus::walk::{LoadedDocument, RepoLoad};
use quire_rs::{parse_document, Spec};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };

    let fuzzed = LoadedDocument::from_parsed(
        std::path::PathBuf::from("fuzz/A.md"),
        "A".to_string(),
        None,
        parse_document(text),
    );
    // A fixed second document so resolution has a real target to hit.
    let target = LoadedDocument::from_parsed(
        std::path::PathBuf::from("fuzz/B.md"),
        "B".to_string(),
        None,
        parse_document("---\nid: B\ntype: StR\n---\n# target\n"),
    );

    let spec = Spec::from_repo(RepoLoad {
        documents: vec![fuzzed, target],
        diagnostics: Vec::new(),
    });

    // Exercise the read-only query surface — none may panic.
    let _ = spec.len();
    let _ = spec.by_id("A");
    let _ = spec.by_type("StR");
    let _ = spec.referencing("B");
    let _ = spec.outgoing("A");
    let _ = spec.orphans("FR", "implements", Some("StR"));
    let _ = spec.dangling();
    let _ = spec.diagnostics();
});
