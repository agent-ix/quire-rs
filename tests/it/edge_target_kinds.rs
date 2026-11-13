//! PLAT-1689 / FR-040-AC-12..14: a wrong-kind target of a declared edge is
//! enforced by its kind — the target document's own archetype as well as its
//! `object:` type — is an error under `Strict`, and names the file path.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::{validate_bundle_at, BundlePosture, BundleReport, Registry};

fn tmpdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "quire_edge_kinds_{tag}_{}",
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

/// `adr` may `realizes` only `FR` or `NFR` documents; `FR` and `NFR` are
/// archetypes, not object types.
fn registry() -> Registry {
    let manifest = r#"
name: edge-kinds
artifact_types:
- name: adr
  frontmatter_schema_ref: schemas/c.schema.json
  allowed_links:
    realizes: [FR, NFR]
    references: [adr]
    classifies: [domain-object]
- name: FR
  frontmatter_schema_ref: schemas/c.schema.json
  roles: [domain-object]
- name: NFR
  frontmatter_schema_ref: schemas/c.schema.json
edge_types:
  realizes: { description: realizes, category: realization }
  classifies: { description: classifies, category: traceability }
roles:
  domain-object: { description: a business-model type }
"#;
    let mut schemas = std::collections::BTreeMap::new();
    schemas.insert(
        "schemas/c.schema.json".to_string(),
        r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#.to_string(),
    );
    Registry::from_inline_parts(manifest.as_bytes(), &schemas).expect("inline registry")
}

fn doc(id: &str, kind: &str, realizes: Option<&str>) -> String {
    let rel = realizes
        .map(|t| format!("relationships:\n- target: {t}\n  type: realizes\n"))
        .unwrap_or_default();
    format!("---\nid: {id}\ntype: {kind}\n{rel}---\n# {id}\nbody\n")
}

fn validate(files: &[(&str, String)], posture: BundlePosture) -> BundleReport {
    let root = tmpdir("c");
    for (name, body) in files {
        write(&root, name, body);
    }
    let report = validate_bundle_at(&root, &registry(), posture);
    fs::remove_dir_all(root).expect("remove temporary bundle");
    report
}

fn target_findings(r: &BundleReport) -> usize {
    r.errors
        .iter()
        .chain(&r.warnings)
        .filter(|f| f.reason == "disallowed-edge-target")
        .count()
}

/// Right kind passes: an ADR realizing an FR and an NFR (archetype-name
/// allow-list entries) draws nothing, in either posture.
#[trace("FR-040-AC-12")]
#[test]
fn right_kind_by_archetype_name_passes() {
    for posture in [BundlePosture::Strict, BundlePosture::Okf] {
        let r = validate(
            &[
                ("ADR-1.md", doc("ADR-1", "adr", Some("FR-1"))),
                ("ADR-2.md", doc("ADR-2", "adr", Some("NFR-1"))),
                ("FR-1.md", doc("FR-1", "FR", None)),
                ("NFR-1.md", doc("NFR-1", "NFR", None)),
            ],
            posture,
        );
        assert_eq!(target_findings(&r), 0, "{posture:?}: {r:?}");
        assert!(r.is_valid(), "{posture:?}: {:?}", r.errors);
    }
}

/// Roles declared on an artifact archetype participate in target matching.
#[trace("FR-040-AC-7", "FR-040-AC-12")]
#[test]
fn target_archetype_roles_satisfy_the_allow_list() {
    let r = validate(
        &[
            (
                "ADR-1.md",
                "---\nid: ADR-1\ntype: adr\nrelationships:\n- target: FR-1\n  type: classifies\n---\n# ADR-1\n".to_string(),
            ),
            ("FR-1.md", doc("FR-1", "FR", None)),
        ],
        BundlePosture::Strict,
    );
    assert_eq!(target_findings(&r), 0, "archetype role: {r:?}");
}

/// Wrong kind fails: ADR realizing another ADR (no `object:` anywhere) is an
/// error under Strict, a warning under Okf, and the finding names the file.
#[trace("FR-040-AC-12")]
#[trace("FR-040-AC-13")]
#[test]
fn wrong_kind_is_error_under_strict_with_file_path() {
    let files = [
        ("a/ADR-1.md", doc("ADR-1", "adr", Some("ADR-2"))),
        ("b/ADR-2.md", doc("ADR-2", "adr", None)),
    ];
    let strict = validate(&files, BundlePosture::Strict);
    assert!(!strict.is_valid());
    let f: Vec<_> = strict
        .errors
        .iter()
        .filter(|f| f.reason == "disallowed-edge-target")
        .collect();
    assert_eq!(f.len(), 1, "{strict:?}");
    assert!(f[0].path.ends_with("a/ADR-1.md"), "path: {:?}", f[0].path);
    assert!(
        f[0].message.contains("a/ADR-1.md") && f[0].message.contains("b/ADR-2.md"),
        "message carries both file paths: {}",
        f[0].message
    );

    let okf = validate(&files, BundlePosture::Okf);
    assert!(okf.is_valid(), "Okf stays a warning: {:?}", okf.errors);
    assert_eq!(
        okf.warnings
            .iter()
            .filter(|f| f.reason == "disallowed-edge-target")
            .count(),
        1
    );
}

/// The unconfigured tier is overridable through the existing mechanism.
#[trace("FR-040-AC-14")]
#[test]
fn severity_map_overrides_target_tier() {
    use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
    let files = [
        ("ADR-1.md", doc("ADR-1", "adr", Some("ADR-2"))),
        ("ADR-2.md", doc("ADR-2", "adr", None)),
    ];
    let root = tmpdir("ov");
    for (n, b) in &files {
        write(&root, n, b);
    }
    let key = "edges:disallowed-edge-target".to_string();
    for (posture, level, expected_errors, expected_warnings) in [
        (BundlePosture::Strict, GrammarSeverityLevel::Error, 1, 0),
        (BundlePosture::Strict, GrammarSeverityLevel::Warning, 0, 1),
        (BundlePosture::Strict, GrammarSeverityLevel::Off, 0, 0),
        (BundlePosture::Okf, GrammarSeverityLevel::Error, 1, 0),
        (BundlePosture::Okf, GrammarSeverityLevel::Warning, 0, 1),
        (BundlePosture::Okf, GrammarSeverityLevel::Off, 0, 0),
    ] {
        let mut map = GrammarSeverityMap::new();
        map.insert(key.clone(), level);
        let reg = registry().with_grammar_severity(map);
        let r = validate_bundle_at(&root, &reg, posture);
        assert_eq!(
            r.errors
                .iter()
                .filter(|f| f.reason == "disallowed-edge-target")
                .count(),
            expected_errors,
            "{posture:?} + {level:?}: {:?}",
            r.errors
        );
        assert_eq!(
            r.warnings
                .iter()
                .filter(|f| f.reason == "disallowed-edge-target")
                .count(),
            expected_warnings,
            "{posture:?} + {level:?}: {:?}",
            r.warnings
        );
        assert_eq!(
            r.is_valid(),
            expected_errors == 0,
            "{posture:?} + {level:?}"
        );
    }
    fs::remove_dir_all(root).expect("remove temporary bundle");
}

/// Dangling target of a declared edge still fails under Strict.
#[trace("FR-026-AC-3")]
#[test]
fn dangling_target_fails_strict() {
    let r = validate(
        &[("ADR-1.md", doc("ADR-1", "adr", Some("FR-404")))],
        BundlePosture::Strict,
    );
    assert!(!r.is_valid());
    assert!(r.errors.iter().any(|f| f.reason == "dangling-reference"));
}

/// Only declared `relationships` entries receive Tier-2 target-kind checks;
/// frontmatter URIs, body URIs and relative navigation links stay references.
#[trace("FR-040-AC-12")]
#[test]
fn synthesized_reference_edges_are_not_target_kind_checked() {
    let source = "---\nid: ADR-1\ntype: adr\nrelationships:\n- target: ix://o/r/FR-1\n  type: realizes\n---\n# ADR-1\nSee ix://o/r/FR-1 and [the requirement](./FR-1.md).\n";
    let r = validate(
        &[
            ("ADR-1.md", source.to_string()),
            ("FR-1.md", doc("FR-1", "FR", None)),
        ],
        BundlePosture::Strict,
    );
    assert_eq!(target_findings(&r), 0, "synthesized references: {r:?}");
}

/// An unresolved `object:` contributes no object kind or roles, while the
/// target's registered document archetype remains checked.
#[trace("FR-040-AC-12")]
#[test]
fn unresolved_object_uses_registered_document_archetype() {
    let target = "---\nid: ADR-2\ntype: adr\nobject: missing-object\n---\n# ADR-2\n";
    let r = validate(
        &[
            ("ADR-1.md", doc("ADR-1", "adr", Some("ADR-2"))),
            ("ADR-2.md", target.to_string()),
        ],
        BundlePosture::Strict,
    );
    assert!(
        r.errors
            .iter()
            .any(|f| f.reason == "disallowed-edge-target"),
        "known document archetype still participates in target matching: {r:?}"
    );
}
