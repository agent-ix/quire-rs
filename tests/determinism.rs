//! Cross-thread + cross-run determinism (NFR-006).
//!
//! Identical input → byte-identical output across threads and across
//! repeated invocations. Covers the parser, the renderer, and the
//! merge+validate path.

use std::path::Path;
use std::sync::Arc;
use std::thread;

use quire_rs::{apply_patch, parse_document, render_by_name, Registry};
use serde_json::json;

fn modules_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("render_parity")
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

#[test]
fn render_is_deterministic_across_threads() {
    let r = Arc::new(Registry::load_from(&[modules_root().as_path()]).expect("load"));
    let data = Arc::new(json!({"id": "DEMO-001", "title": "deterministic", "body": "ok"}));
    let baseline = render_by_name(&r, "demo-item", &data).expect("baseline");
    let handles: Vec<_> = (0..32)
        .map(|_| {
            let r = Arc::clone(&r);
            let data = Arc::clone(&data);
            let baseline = baseline.clone();
            thread::spawn(move || {
                let got = render_by_name(&r, "demo-item", &data).unwrap();
                assert_eq!(got, baseline);
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
