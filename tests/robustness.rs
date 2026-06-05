//! Input robustness (NFR-019-AC-2 / TC-580).
//!
//! A proptest generates random strings (including empty, fence-only,
//! frontmatter-only, and deeply nested heading inputs) and asserts
//! `parse_document`, `validate_document`, and `extract` each return a
//! value or typed error without panicking across all generated cases.
//! Run with `PROPTEST_CASES=512` in CI.

use std::path::Path;
use std::sync::OnceLock;

use proptest::prelude::*;
use quire_rs::{extract, parse_document, validate_document, Registry};

fn registry() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| {
        let module = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("modules")
            .join("iso");
        Registry::load_module(&module).expect("load iso module")
    })
}

/// Exercise all three retained surfaces; the harness fails only if one
/// panics. No assertion on the result — the contract is "no panic".
fn exercise(text: &str) {
    let doc = parse_document(text);
    if let Some(arch) = registry().archetype("FR") {
        let _ = validate_document(arch, text);
        if let Some(dsl) = arch.body_extraction() {
            let _ = extract(&doc, dsl);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // Arbitrary unicode strings.
    #[test]
    fn no_panic_on_arbitrary_strings(s in ".*") {
        exercise(&s);
    }

    // Fence-heavy / heading-heavy inputs assembled from a small alphabet of
    // structural tokens (frontmatter delimiters, fences, headings, pipes).
    #[test]
    fn no_panic_on_structural_token_soup(
        tokens in proptest::collection::vec(
            prop_oneof![
                Just("---".to_string()),
                Just("```".to_string()),
                Just("~~~".to_string()),
                Just("# H".to_string()),
                Just("## H".to_string()),
                Just("###### deep".to_string()),
                Just("| a | b |".to_string()),
                Just("- item".to_string()),
                Just("{{ x }}".to_string()),
                Just("id: x".to_string()),
                Just(String::new()),
            ],
            0..64,
        )
    ) {
        let text = tokens.join("\n");
        exercise(&text);
    }
}

// Explicit edge cases (cheap, deterministic) alongside the proptest.
#[test]
fn no_panic_on_explicit_edge_cases() {
    let cases = [
        "",
        "---",
        "---\n",
        "---\nid: x\n---\n",
        "```",
        "~~~rust\nunclosed",
        "###### a\n####### b\n",
        "| | |\n|-|-|\n",
        "\u{feff}---\nid: x\n---\n# H\n",
        &"#".repeat(10_000),
    ];
    for c in cases {
        exercise(c);
    }
}
