#![no_main]
//! NFR-011 fuzz target (v0.3) — intra-spec reference resolution must
//! not panic on adversarial `relationships` arrays or `ix://` links.
//!
//! Embeds the fuzz bytes into both a frontmatter `relationships` target
//! and an `ix://` body link, then builds a `Spec` (which harvests +
//! resolves). Stresses the lexical `ix://` scan, target-id extraction,
//! dedup, and resolved/dangling classification.

use libfuzzer_sys::fuzz_target;
use quire_rs::corpus::walk::{LoadedDocument, RepoLoad};
use quire_rs::{parse_document, Spec};

fuzz_target!(|data: &[u8]| {
    let Ok(raw) = std::str::from_utf8(data) else {
        return;
    };
    // Keep the YAML scalar single-line so the frontmatter is parseable;
    // the body link gets the raw form to stress the regex scanner.
    let one_line: String = raw.chars().filter(|c| *c != '\n' && *c != '\r').collect();

    let md = format!(
        "---\nid: SRC\nartifact_type: FR\nrelationships:\n  - target: \"{one_line}\"\n    type: implements\n---\n\
         body with a link [x](ix://{raw}) and bare ix://{one_line}\n"
    );

    let src = LoadedDocument {
        path: std::path::PathBuf::from("fuzz/SRC.md"),
        id: "SRC".to_string(),
        uuid: None,
        doc: parse_document(&md),
    };
    let str_target = LoadedDocument {
        path: std::path::PathBuf::from("fuzz/StR-001.md"),
        id: "StR-001".to_string(),
        uuid: None,
        doc: parse_document("---\nid: StR-001\nartifact_type: StR\n---\n# need\n"),
    };

    let spec = Spec::from_repo(RepoLoad {
        documents: vec![src, str_target],
        diagnostics: Vec::new(),
    });

    let _ = spec.outgoing("SRC");
    let _ = spec.referencing("StR-001");
    let _ = spec.dangling();
    let _ = spec.edges();
});
