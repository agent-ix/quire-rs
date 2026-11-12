//! FR-050-AC-52: a corpus input is scored only from a clean checkout.

use std::fs;
use std::path::Path;
use std::process::Command;

use ix_trace_rs::trace;
use quire_bench::collection::{collect, require_clean, resolve_default_module};
use quire_bench::Manifest;
use serde_json::json;

fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
        ])
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

/// A committed checkout at `root/name` holding `module/manifest.yaml`.
fn checkout(root: &Path, name: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.join("module")).unwrap();
    fs::write(path.join("module/manifest.yaml"), "name: fixture\n").unwrap();
    git(&path, &["init", "-q"]);
    git(&path, &["add", "."]);
    git(&path, &["commit", "-q", "-m", "fixture"]);
}

fn manifest(module_clean: bool, entry_module: Option<&str>) -> Manifest {
    let mut entry = json!({"name": "corpus", "path": "corpus", "require_clean": true});
    if let Some(module) = entry_module {
        entry["module"] = json!(module);
    }
    serde_json::from_value(json!({
        "metrics": {},
        "module_source": {
            "name": "modules",
            "path": "modules",
            "module": "module",
            "require_clean": module_clean
        },
        "corpora": [entry]
    }))
    .unwrap()
}

fn error<T>(result: Result<T, quire_bench::BenchError>) -> String {
    match result {
        Ok(_) => panic!("expected a refusal"),
        Err(error) => error.to_string(),
    }
}

#[trace("FR-050-AC-52")]
#[test]
fn modified_tracked_file_refuses_and_clean_checkout_is_accepted() {
    let root = tempfile::tempdir().unwrap();
    checkout(root.path(), "corpus");
    require_clean(root.path(), "corpus", "corpus", true).unwrap();
    fs::write(root.path().join("corpus/module/manifest.yaml"), "edited\n").unwrap();
    assert_eq!(
        error(require_clean(root.path(), "corpus", "corpus", true)),
        "corpus: corpus tree is dirty. Refusing to score an input with uncommitted edits."
    );
}

#[trace("FR-050-AC-52")]
#[test]
fn absent_path_and_non_checkout_refuse() {
    let root = tempfile::tempdir().unwrap();
    assert!(error(require_clean(root.path(), "corpus", "missing", true))
        .starts_with("corpus: corpus absent at "));
    fs::create_dir(root.path().join("plain")).unwrap();
    assert!(
        error(require_clean(root.path(), "corpus", "plain", true)).contains("corpus tree is dirty")
    );
}

#[trace("FR-050-AC-52")]
#[test]
fn nested_directory_follows_its_enclosing_checkout() {
    let root = tempfile::tempdir().unwrap();
    checkout(root.path(), "corpus");
    require_clean(root.path(), "nested", "corpus/module", true).unwrap();
    fs::write(root.path().join("corpus/untracked.md"), "new\n").unwrap();
    assert!(
        error(require_clean(root.path(), "nested", "corpus/module", true))
            .starts_with("nested: corpus tree is dirty")
    );
}

#[trace("FR-050-AC-52")]
#[test]
fn normal_run_skips_a_dirty_entry_and_strict_run_fails_naming_it() {
    let root = tempfile::tempdir().unwrap();
    checkout(root.path(), "modules");
    checkout(root.path(), "corpus");
    fs::write(root.path().join("corpus/untracked.md"), "new\n").unwrap();
    let manifest = manifest(true, None);
    let quire = root.path().join("no-quire");
    assert!(error(collect(root.path(), &manifest, &quire, None, false))
        .starts_with("no corpus entry could be scored"));
    assert_eq!(
        error(collect(root.path(), &manifest, &quire, None, true)),
        "corpus: corpus: corpus tree is dirty. Refusing to score an input with uncommitted edits."
    );
}

#[trace("FR-050-AC-52")]
#[test]
fn module_source_rules_apply_when_an_entry_reads_the_default_module() {
    let root = tempfile::tempdir().unwrap();
    checkout(root.path(), "modules");
    checkout(root.path(), "corpus");
    let quire = root.path().join("no-quire");
    resolve_default_module(root.path(), &manifest(true, None).module_source, None).unwrap();
    assert_eq!(
        error(collect(
            root.path(),
            &manifest(false, None),
            &quire,
            None,
            false
        )),
        "benchmark module_source must require a clean tree"
    );
    // Every entry names its own module: the default module is not read.
    assert!(!error(collect(
        root.path(),
        &manifest(false, Some("corpus/module")),
        &quire,
        None,
        false
    ))
    .contains("module_source"));
    fs::write(root.path().join("modules/untracked.md"), "new\n").unwrap();
    assert_eq!(
        error(collect(
            root.path(),
            &manifest(true, None),
            &quire,
            None,
            false
        )),
        "modules: corpus tree is dirty. Refusing to score an input with uncommitted edits."
    );
}
