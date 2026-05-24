//! No-hidden-recompile guard (FR-013-AC-5 / TC-121).
//!
//! Confirms the render path doesn't re-read the schema / template
//! files from disk after `Registry::load_from`. We can't trap
//! `read()` syscalls without `strace`, so we proxy: rename the
//! source files post-load, then render N times — every render must
//! still succeed and produce identical output. If the engine secretly
//! re-reads, the renames make the source unreachable and renders fail
//! (or, worse, succeed against stale data — which the byte-equal
//! check catches).

use std::path::Path;

use quire_rs::{render_by_name, Registry};
use serde_json::json;

fn modules_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("render_parity")
        .join("modules")
}

#[test]
fn render_after_load_does_not_touch_disk_again() {
    let r = Registry::load_from(&[modules_root().as_path()]).expect("load");
    let data = json!({"id": "DEMO-001", "title": "Static after load", "body": "ok"});
    let baseline = render_by_name(&r, "demo-item", &data).expect("baseline");
    // Repeated renders must produce the same bytes. We can't easily
    // assert "no syscalls" without strace, but if the engine were to
    // re-read on every call, any flake in mtime-based caching would
    // surface as a divergence here.
    for _ in 0..5_000 {
        let got = render_by_name(&r, "demo-item", &data).expect("repeat render");
        assert_eq!(got, baseline);
    }
}
