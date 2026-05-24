//! No-hidden-recompile guard (FR-013-AC-5 / TC-121).
//!
//! Confirms the render path doesn't re-read schema / template files
//! from disk after `Registry::load_from`.
//!
//! Method: load a registry from a *copy* of the bootstrap demo
//! module, capture a baseline render, then **rename the entire
//! source directory** out from under the registry. If the engine
//! were silently re-reading disk on each render, the renames would
//! either (a) break renders entirely, or (b) surface stale-data
//! divergence. Neither must happen: every subsequent render must
//! match the baseline byte-for-byte.

use std::path::{Path, PathBuf};

use quire_rs::{render_by_name, Registry};
use serde_json::json;

fn tmpdir() -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut p = std::env::temp_dir();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    p.push(format!(
        "quire-rs-no-recompile-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).expect("mkdir");
    p
}

/// Recursive directory copy. Used to seed an isolated copy of the
/// bootstrap demo module that we can safely rename.
fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap().flatten() {
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

#[test]
fn render_after_load_survives_post_load_source_rename() {
    let bootstrap = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("render_parity")
        .join("modules");
    let work = tmpdir();
    let work_modules = work.join("modules");
    copy_dir(&bootstrap, &work_modules);

    let r = Registry::load_from(&[work_modules.as_path()]).expect("load");
    let data = json!({"id": "DEMO-001", "title": "post-load rename", "body": "ok"});
    let baseline = render_by_name(&r, "demo-item", &data).expect("baseline render");

    // Now hide the source from disk — if any render path were
    // silently reading templates/ or schemas/, this would break.
    let renamed = work.join("modules-moved");
    std::fs::rename(&work_modules, &renamed).expect("rename");

    // 1000 renders against the registry must all match the baseline.
    for _ in 0..1000 {
        let got = render_by_name(&r, "demo-item", &data).expect("post-rename render");
        assert_eq!(
            got, baseline,
            "render diverged after source-dir rename — implies a silent disk re-read"
        );
    }

    // Sanity: a fresh load_from the renamed path still works (proves
    // the rename actually moved real bytes; the post-rename renders
    // above can't be explained by "rename was a no-op").
    let r2 = Registry::load_from(&[renamed.as_path()]).expect("reload from renamed");
    let got2 = render_by_name(&r2, "demo-item", &data).expect("reloaded render");
    assert_eq!(got2, baseline);
}
