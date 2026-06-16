//! Bundle validation postures (OKF) + index-completeness.
//!
//! Strict = our archetype-conformant posture; Okf = permissive
//! foreign-bundle reading. `type` is required in BOTH (the
//! "untyped corpus doc is an error, not a warning" fix); unknown types,
//! broken `ix://` links, and index gaps degrade to warnings only under Okf.

use std::fs;
use std::path::{Path, PathBuf};

use quire_rs::{validate_bundle_at, BundlePosture, Registry};

fn bundle_registry() -> Registry {
    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules/bundle");
    Registry::load_module(&module).expect("load bundle test module")
}

fn tmpdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "quire_bundle_{tag}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&p).unwrap();
    p
}

fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

fn note(id: &str, body: &str) -> String {
    format!("---\nid: {id}\ntype: note\n---\n# {id}\n{body}\n")
}

/// Strict: a document with no `type` is a hard error (was a non-fatal
/// `UntypedArtifact` warning before this change).
#[test]
fn strict_untyped_doc_is_error() {
    let root = tmpdir("strict_untyped");
    write(
        &root,
        "NOTE-001.md",
        "---\nid: NOTE-001\n---\n# note\nbody\n",
    );
    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(!report.is_valid());
    assert!(report
        .errors
        .iter()
        .any(|f| f.reason == "frontmatter" && f.message.contains("type")));
}

/// Okf: `type` is still required — an untyped doc is an error even under
/// the permissive posture.
#[test]
fn okf_untyped_doc_is_still_error() {
    let root = tmpdir("okf_untyped");
    write(
        &root,
        "NOTE-001.md",
        "---\nid: NOTE-001\n---\n# note\nbody\n",
    );
    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);
    assert!(!report.is_valid());
    assert!(report.errors.iter().any(|f| f.reason == "frontmatter"));
}

/// Okf tolerates an unknown type and a dangling `ix://` reference as
/// warnings; Strict rejects the unknown type.
#[test]
fn okf_tolerates_unknown_type_and_broken_links() {
    let root = tmpdir("okf_tolerant");
    write(
        &root,
        "X-1.md",
        "---\nid: X-1\ntype: weird\n---\n# x\nsee [missing](ix://o/r/MISSING)\n",
    );

    let okf = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);
    assert!(okf.is_valid(), "okf errors: {:?}", okf.errors);
    assert!(okf.warnings.iter().any(|f| f.reason == "unknown-type"));
    assert!(okf
        .warnings
        .iter()
        .any(|f| f.reason == "dangling-reference"));

    let strict = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(!strict.is_valid());
    assert!(strict.errors.iter().any(|f| f.reason == "unknown-type"));
    assert!(strict
        .errors
        .iter()
        .any(|f| f.reason == "dangling-reference"));
}

/// A typed, archetype-conformant bundle whose index lists every sibling +
/// carries `okf_version` is valid under Strict.
#[test]
fn strict_conformant_bundle_with_complete_index_is_valid() {
    let root = tmpdir("strict_ok");
    write(&root, "NOTE-001.md", &note("NOTE-001", "body"));
    write(
        &root,
        "index.md",
        "---\ntype: index\ntitle: Root\nokf_version: \"0.1\"\n---\n# Root\n\n## Contents\n\n* [NOTE-001](./NOTE-001.md)\n",
    );
    write(
        &root,
        "log.md",
        "---\ntype: log\n---\n# Log\n\n## History\n\n* 2026-06-16 created\n",
    );

    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(report.is_valid(), "errors: {:?}", report.errors);
}

/// An index.md missing a sibling artifact is an index-completeness error
/// under Strict, a warning under Okf.
#[test]
fn index_incompleteness_is_error_strict_warning_okf() {
    let root = tmpdir("index_incomplete");
    write(&root, "NOTE-001.md", &note("NOTE-001", "body"));
    write(&root, "NOTE-002.md", &note("NOTE-002", "body"));
    // index lists only NOTE-001.
    write(
        &root,
        "index.md",
        "---\ntype: index\nokf_version: \"0.1\"\n---\n# Root\n\n## Contents\n\n* [NOTE-001](./NOTE-001.md)\n",
    );

    let strict = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(strict
        .errors
        .iter()
        .any(|f| f.reason == "index-incomplete" && f.message.contains("NOTE-002")));

    let okf = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);
    assert!(okf
        .warnings
        .iter()
        .any(|f| f.reason == "index-incomplete" && f.message.contains("NOTE-002")));
}

/// The bundle-root index.md must declare `okf_version`.
#[test]
fn root_index_missing_okf_version_is_flagged() {
    let root = tmpdir("no_okf_version");
    write(&root, "NOTE-001.md", &note("NOTE-001", "body"));
    write(
        &root,
        "index.md",
        "---\ntype: index\n---\n# Root\n\n## Contents\n\n* [NOTE-001](./NOTE-001.md)\n",
    );

    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(report
        .errors
        .iter()
        .any(|f| f.reason == "index-okf-version"));
}

/// A subdirectory index need not carry `okf_version` (only the root does).
#[test]
fn subdir_index_does_not_require_okf_version() {
    let root = tmpdir("subdir_index");
    write(&root, "NOTE-000.md", &note("NOTE-000", "body"));
    write(
        &root,
        "index.md",
        "---\ntype: index\nokf_version: \"0.1\"\n---\n# Root\n\n## Contents\n\n* [NOTE-000](./NOTE-000.md)\n* [sub](./sub/index.md)\n",
    );
    write(&root, "sub/NOTE-001.md", &note("NOTE-001", "body"));
    write(
        &root,
        "sub/index.md",
        "---\ntype: index\n---\n# Sub\n\n## Contents\n\n* [NOTE-001](./NOTE-001.md)\n",
    );

    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(report.is_valid(), "errors: {:?}", report.errors);
}

/// A mistyped optional `description` is caught by the base concept contract
/// under Strict.
#[test]
fn strict_rejects_mistyped_description() {
    let root = tmpdir("mistyped_desc");
    write(
        &root,
        "NOTE-001.md",
        "---\nid: NOTE-001\ntype: note\ndescription: 7\n---\n# note\nbody\n",
    );
    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(!report.is_valid());
    assert!(report
        .errors
        .iter()
        .any(|f| f.message.contains("description")));
}
