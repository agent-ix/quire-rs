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
fn registry(extra_manifest: &str) -> Registry {
    let manifest = format!(
        r#"
name: edge-kinds
artifact_types:
- name: adr
  frontmatter_schema_ref: schemas/c.schema.json
  allowed_links:
    realizes: [FR, NFR]
- name: FR
  frontmatter_schema_ref: schemas/c.schema.json
- name: NFR
  frontmatter_schema_ref: schemas/c.schema.json
{extra_manifest}
edge_types:
  realizes: {{ description: realizes, category: realization }}
"#
    );
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
    validate_bundle_at(&root, &registry(""), posture)
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
    for (level, strict_errors) in [
        (GrammarSeverityLevel::Warning, 0),
        (GrammarSeverityLevel::Off, 0),
    ] {
        let mut map = GrammarSeverityMap::new();
        map.insert(key.clone(), level);
        let reg = registry("").with_grammar_severity(map);
        let r = validate_bundle_at(&root, &reg, BundlePosture::Strict);
        assert_eq!(r.errors.len(), strict_errors, "{level:?}: {:?}", r.errors);
        let expect_warn = usize::from(level == GrammarSeverityLevel::Warning);
        assert_eq!(target_findings(&r), expect_warn, "{level:?}");
    }
    let mut map = GrammarSeverityMap::new();
    map.insert(key, GrammarSeverityLevel::Error);
    let reg = registry("").with_grammar_severity(map);
    let r = validate_bundle_at(&root, &reg, BundlePosture::Okf);
    assert!(!r.is_valid(), "error promotes under Okf");
}

/// Dangling target of a declared edge still fails under Strict.
#[trace("FR-040-AC-12")]
#[test]
fn dangling_target_fails_strict() {
    let r = validate(
        &[("ADR-1.md", doc("ADR-1", "adr", Some("FR-404")))],
        BundlePosture::Strict,
    );
    assert!(!r.is_valid());
    assert!(r.errors.iter().any(|f| f.reason == "dangling-reference"));
}
