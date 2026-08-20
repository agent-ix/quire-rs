//! Cross-thread + cross-run determinism (NFR-006).
//!
//! Identical input → byte-identical output across threads and across
//! repeated invocations. Covers the parser, `validate_document`,
//! `extract`, and the merge+validate path.

use std::path::Path;
use std::sync::Arc;
use std::thread;

use quire_rs::{apply_patch, extract, parse_document, validate_document, Registry};
use serde_json::json;

fn modules_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("modules")
}

#[test]
fn parse_is_deterministic_across_threads() {
    let input = Arc::new(
        "---\nid: x\n---\n## A\nfirst body\n### A1\nchild\n## B\n second body\n".to_string(),
    );
    let baseline = parse_document(&input);
    let handles: Vec<_> = (0..32)
        .map(|_| {
            let input = Arc::clone(&input);
            let baseline = baseline.clone();
            thread::spawn(move || {
                let got = parse_document(&input);
                assert_eq!(got, baseline);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

const FR_DOC: &str = "---\nid: FR-001\ntitle: Determinism\ntype: FR\n---\n\n\
## Description\n\nThe engine is deterministic.\n\n\
## Specification\n\nGiven identical input it produces identical output.\n\n\
## Acceptance Criteria\n\nAll runs agree.\n\n\
## Dependencies\n\nNone.\n";

// TC-578, NFR-006-AC-4: validate_document + extract on the same input
// 100× across threads yield equal ValidationResult (ordered diagnostics)
// and ExtractionResult (records + edges + diagnostics) every time.
#[test]
fn validate_document_and_extract_are_deterministic_across_threads() {
    let r = Arc::new(Registry::load_from(&[modules_root().as_path()]).expect("load"));
    let arch_name = "FR";
    let doc = Arc::new(parse_document(FR_DOC));

    let v_baseline = {
        let arch = r.archetype(arch_name).expect("FR archetype");
        validate_document(arch, FR_DOC)
    };
    let e_baseline = {
        let arch = r.archetype(arch_name).expect("FR archetype");
        let dsl = arch.body_extraction().expect("FR has body_extraction");
        extract(&doc, dsl).expect("extract")
    };

    let handles: Vec<_> = (0..32)
        .map(|_| {
            let r = Arc::clone(&r);
            let doc = Arc::clone(&doc);
            let v_baseline = v_baseline.clone();
            let e_baseline = e_baseline.clone();
            thread::spawn(move || {
                for _ in 0..4 {
                    let arch = r.archetype(arch_name).expect("FR archetype");
                    let got_v = validate_document(arch, FR_DOC);
                    assert_eq!(got_v, v_baseline);
                    let dsl = arch.body_extraction().expect("dsl");
                    let got_e = extract(&doc, dsl).expect("extract");
                    assert_eq!(got_e, e_baseline);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn apply_patch_is_deterministic_for_object_merge() {
    let r = Registry::load_from(&[modules_root().as_path()]).expect("load");
    let arch = r.archetype("demo-item").expect("arch");
    let current = json!({"id": "DEMO-001", "title": "original"});
    let patch = json!({"title": "patched", "tags": ["a", "b"]});
    let a = apply_patch(arch, &current, &patch).unwrap();
    // 100 repeats should produce the same bytes — no observable
    // HashMap iteration in the merge or validation path.
    for _ in 0..100 {
        let b = apply_patch(arch, &current, &patch).unwrap();
        assert_eq!(a, b);
    }
}
