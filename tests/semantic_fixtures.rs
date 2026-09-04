//! Plan-003 Task-017: vendored golden fixtures and the semantic case suite.
//!
//! The quoin mapping/corpus fixtures under `tests/fixtures/semantic/quoin/`
//! are read-only copies pinned by `PROVENANCE.json`; `cases.json` is the
//! declarative suite FR-072-AC-1 runs and, like `corpus_cases`, every case
//! must name the filing it came from.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[derive(Deserialize)]
struct Provenance {
    files: BTreeMap<String, ProvenanceFile>,
}

#[derive(Deserialize)]
struct ProvenanceFile {
    repository: String,
    revision: String,
    path: String,
    sha256: String,
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

#[trace("TC-1610", "FR-070-AC-1")]
// the golden inputs are the quoin fixtures, byte-for-byte, at the pinned
// revision; the extraction assertions themselves live in
// tests/semantic_properties.rs and tests/semantic_clauses.rs (Tasks 018/019).
#[test]
fn quoin_fixtures_match_provenance() {
    let dir = root().join("tests/fixtures/semantic/quoin");
    let provenance: Provenance =
        serde_json::from_slice(&fs::read(dir.join("PROVENANCE.json")).unwrap()).unwrap();
    let mut on_disk = Vec::new();
    walk(&dir, &mut on_disk);
    let mut on_disk: Vec<String> = on_disk
        .iter()
        .map(|p| p.strip_prefix(&dir).unwrap().to_string_lossy().to_string())
        .filter(|rel| rel != "PROVENANCE.json")
        .collect();
    on_disk.sort();
    let recorded: Vec<String> = provenance.files.keys().cloned().collect();
    assert_eq!(
        on_disk, recorded,
        "fixture files and PROVENANCE.json disagree"
    );
    for (rel, record) in &provenance.files {
        assert_eq!(record.repository, "agent-ix/quoin");
        assert_eq!(record.revision.len(), 40, "{rel}: full commit id required");
        assert!(
            record.path.starts_with("tests/fixtures/semantic-module/"),
            "{rel}"
        );
        let digest = format!(
            "sha256:{:x}",
            Sha256::digest(fs::read(dir.join(rel)).unwrap())
        );
        assert_eq!(
            digest, record.sha256,
            "{rel}: edited locally; re-vendor instead"
        );
    }
    // The mapping README is the upstream hand-off contract; keep it present.
    assert!(provenance.files.contains_key("mapping/README.md"));
    assert!(provenance
        .files
        .contains_key("corpus/config-service/FR-006-config-version-entity.md"));
}

#[derive(Deserialize)]
struct CaseFile {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    issue_ref: String,
    #[serde(default)]
    tags: Vec<String>,
    #[allow(dead_code)]
    input: Value,
    #[allow(dead_code)]
    expect: Value,
}

#[trace("TC-1630", "FR-072-AC-1")]
// attribution and unique names, as corpus_cases enforces for its own suite;
// the per-case expectations run in tests/semantic_surface.rs (Task-020).
#[test]
fn semantic_cases_are_attributed_and_uniquely_named() {
    let file: CaseFile = serde_json::from_slice(
        &fs::read(root().join("tests/fixtures/semantic/cases.json")).unwrap(),
    )
    .unwrap();
    let mut names: Vec<&str> = file.cases.iter().map(|c| c.name.as_str()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "duplicate case name: {names:?}");
    for case in &file.cases {
        assert!(
            case.issue_ref.contains('#'),
            "{}: issue_ref must name a filing, got {:?}",
            case.name,
            case.issue_ref
        );
        assert!(
            case.tags.iter().any(|t| t.starts_with("TC-")),
            "{}: tag the matrix row it evidences",
            case.name
        );
    }
}

#[test]
fn bundle_index_fixture_is_well_formed() {
    let bundle: Value = serde_json::from_slice(
        &fs::read(root().join("tests/fixtures/semantic/config-version.bundle.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(bundle["package"], "agent-ix/config-service");
    let objects = bundle["objects"].as_array().unwrap();
    assert!(objects.iter().any(|o| o["id"] == "FR-006"
        && o["names"]
            .as_array()
            .unwrap()
            .contains(&Value::String("ConfigVersion".into()))));
}
