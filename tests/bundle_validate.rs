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

/// Build an inline registry with a `concept` artifact type plus object
/// types carrying roles + allowed_links, for the Tier-2 edge-target test
/// (FR-040-AC-9).
fn edge_target_registry() -> Registry {
    let manifest = br#"
name: edge-target-test
artifact_types:
- name: concept
  frontmatter_schema_ref: schemas/concept.schema.json
object_types:
- name: api_endpoint
  allowed_links:
    exposes: [domain-object]
- name: entity
  roles: [domain-object]
- name: data_schema
edge_types:
  exposes: { description: surface, category: realization }
roles:
  domain-object: { description: a business-model type }
"#;
    let mut schemas = std::collections::BTreeMap::new();
    schemas.insert(
        "schemas/concept.schema.json".to_string(),
        r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#.to_string(),
    );
    Registry::from_inline_parts(manifest, &schemas).expect("inline registry")
}

/// TC-642 (FR-040-AC-9): a resolved edge whose target object type satisfies
/// no token in the verb's target list yields a `disallowed-edge-target`
/// warning; the same verb to a target carrying the required role passes —
/// across object-type boundaries. Warn-tier: the bundle stays valid.
#[test]
fn tc642_disallowed_edge_target_is_warning() {
    let root = tmpdir("edge_target");
    // Source api_endpoint `exposes` two targets: an entity (role
    // domain-object → OK) and a data_schema (no role → flagged).
    write(
        &root,
        "API-1.md",
        "---\nid: API-1\ntype: concept\nobject: api_endpoint\nrelationships:\n- target: ix://o/r/ENT-1\n  type: exposes\n- target: ix://o/r/DS-1\n  type: exposes\n---\n# API-1\nbody\n",
    );
    write(
        &root,
        "ENT-1.md",
        "---\nid: ENT-1\ntype: concept\nobject: entity\n---\n# ENT-1\nbody\n",
    );
    write(
        &root,
        "DS-1.md",
        "---\nid: DS-1\ntype: concept\nobject: data_schema\n---\n# DS-1\nbody\n",
    );

    let report = validate_bundle_at(&root, &edge_target_registry(), BundlePosture::Okf);
    let target_warnings: Vec<_> = report
        .warnings
        .iter()
        .filter(|f| f.reason == "disallowed-edge-target")
        .collect();
    assert_eq!(
        target_warnings.len(),
        1,
        "exactly one disallowed target, got {:?}",
        report.warnings
    );
    assert!(
        target_warnings[0].message.contains("DS-1")
            && target_warnings[0].message.contains("data_schema"),
        "names the offending target + object type: {}",
        target_warnings[0].message
    );
    // Warn-tier: disallowed edge targets never invalidate the bundle.
    assert!(report.is_valid(), "errors: {:?}", report.errors);
}
