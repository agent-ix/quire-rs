//! Plan-003 Task-017: the golden mapping fixtures and the semantic case suite.
//!
//! The mapping/corpus fixtures TC-1610/TC-1852/etc. exercise live in the
//! `quire-fixtures` workspace crate (PLAT-901) — moved there, not vendored,
//! since quire-rs is the parser that defines the forms they exercise and
//! quoin (their original home) already depends on quire-rs as a crate.
//! `cases.json` is the declarative suite FR-072-AC-1 runs and, like
//! `corpus_cases`, every case must name the filing it came from.

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
// TC-1630's per-case expectations run in tests/it/semantic_surface.rs.
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
            names_a_filing(&case.issue_ref),
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

/// A GitHub `<org>/<repo>#<n>` reference or a tracker key such as `PLAT-35`.
fn names_a_filing(issue_ref: &str) -> bool {
    let tracker_key = issue_ref.split_once('-').is_some_and(|(team, number)| {
        !team.is_empty()
            && team.chars().all(|c| c.is_ascii_uppercase())
            && !number.is_empty()
            && number.chars().all(|c| c.is_ascii_digit())
    });
    issue_ref.contains('#') || tracker_key
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
