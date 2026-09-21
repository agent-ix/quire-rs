//! Plan-003 Task-017: first-party golden fixtures and the semantic case suite.
//!
//! The mapping/corpus fixtures under `tests/fixtures/semantic/` are
//! first-party quire-rs test data (CR-182), not a copy of another
//! repository's fixtures; `cases.json` is the declarative suite FR-072-AC-1
//! runs and, like `corpus_cases`, every case must name the filing it came
//! from.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

// Attribution and unique names, as corpus_cases enforces for its own suite;
// TC-1630's per-case expectations run in tests/semantic_surface.rs.
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
