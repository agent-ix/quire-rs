//! FR-083 clause-set manifest declaration and publishability
//! (TC-2313..TC-2317).

use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::loader::{load_single_module, LoadOutcome};
use quire_rs::{check_publishable, ClauseSet};

const SENTINEL: &str = "SENTINEL-sprocket-text-7731";

/// Write a synthetic clause-set file. `text` is the `rights.text` value,
/// `structure` the structure value, `clearance` an optional `clearance_ref`
/// and `with_text` whether the clause carries text.
fn clause_set_yaml(
    id: &str,
    version: &str,
    structure: &str,
    text: &str,
    clearance: Option<&str>,
    with_text: bool,
) -> String {
    let clearance = clearance
        .map(|c| format!("  clearance_ref: \"{c}\"\n"))
        .unwrap_or_default();
    let clause_text = if with_text {
        format!("    text: \"{SENTINEL}\"\n")
    } else {
        String::new()
    };
    format!(
        "schemaVersion: clause-set-v1\n\
         authority: example.invalid\n\
         id: {id}\n\
         title: Synthetic set {id}\n\
         version: \"{version}\"\n\
         rights:\n  structure: {structure}\n  text: {text}\n{clearance}\
         clauses:\n  - id: C-1\n    force: mandatory\n{clause_text}"
    )
}

fn module(dir: &Path, manifest_tail: &str, files: &[(&str, String)]) -> LoadOutcome {
    fs::write(
        dir.join("manifest.yaml"),
        format!("name: publishable-fixture\nversion: 0.1.0\n{manifest_tail}"),
    )
    .unwrap();
    for (name, body) in files {
        fs::write(dir.join(name), body).unwrap();
    }
    load_single_module(dir)
}

fn manifest_for(files: &[(&str, String)]) -> String {
    let mut out = String::from("clause_sets:\n");
    for (name, _) in files {
        out.push_str(&format!("  - {name}\n"));
    }
    out
}

fn load(dir: &Path, files: &[(&str, String)]) -> LoadOutcome {
    module(dir, &manifest_for(files), files)
}

#[test]
#[trace("TC-2313", "FR-083-AC-1", "FR-083-AC-2")]
fn malformed_clause_sets_and_missing_rights_fail_load() {
    let dir = tempfile::tempdir().unwrap();
    let outcome = module(dir.path(), "clause_sets: not-a-list\n", &[]);
    let message = format!("{:?}{:?}", outcome.manifest_errors, outcome.failures);
    assert!(message.contains("clause_sets"), "{message}");

    let dir = tempfile::tempdir().unwrap();
    let outcome = module(dir.path(), "clause_sets: [1, 2]\n", &[]);
    let message = format!("{:?}{:?}", outcome.manifest_errors, outcome.failures);
    assert!(message.contains("clause_sets"), "{message}");

    let dir = tempfile::tempdir().unwrap();
    let no_rights = "schemaVersion: clause-set-v1\nauthority: example.invalid\nid: bare\n\
                     title: Bare\nversion: \"1\"\nclauses: []\n"
        .to_string();
    let outcome = load(dir.path(), &[("bare.yaml", no_rights)]);
    assert_eq!(outcome.failures.len(), 1, "{:?}", outcome.failures);
    assert!(outcome.failures[0].path.ends_with("bare.yaml"));
    assert!(outcome.failures[0].reason.contains("rights"));

    let dir = tempfile::tempdir().unwrap();
    let blank = clause_set_yaml(
        "blank",
        "1",
        "original",
        "explicitly_cleared",
        Some(" "),
        true,
    );
    let outcome = load(dir.path(), &[("blank.yaml", blank)]);
    assert_eq!(outcome.failures.len(), 1);
    assert!(outcome.failures[0].reason.contains("clearance_ref"));

    let dir = tempfile::tempdir().unwrap();
    let unknown = clause_set_yaml("odd", "1", "original", "bogus", None, false);
    let outcome = load(dir.path(), &[("odd.yaml", unknown)]);
    assert_eq!(outcome.failures.len(), 1);
    assert!(outcome.failures[0].path.ends_with("odd.yaml"));
}

#[test]
#[trace("TC-2314", "FR-083-AC-3", "FR-083-AC-5")]
fn publishable_sets_and_structure_rights_are_not_restricted() {
    let dir = tempfile::tempdir().unwrap();
    let files = [
        (
            "a.yaml",
            clause_set_yaml("a", "1", "original", "none", None, false),
        ),
        (
            "b.yaml",
            clause_set_yaml("b", "1", "citation_only", "original", None, true),
        ),
        (
            "c.yaml",
            clause_set_yaml(
                "c",
                "1",
                "explicitly_cleared",
                "explicitly_cleared",
                Some("REV-1"),
                true,
            ),
        ),
    ];
    let outcome = load(dir.path(), &files);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    assert_eq!(outcome.modules[0].clause_sets.len(), 3);
    assert!(check_publishable(&outcome.modules[0]).is_empty());
}

#[test]
#[trace("TC-2315", "FR-083-AC-4", "FR-083-AC-9")]
fn licensed_sets_load_without_clearance_and_are_reported() {
    let dir = tempfile::tempdir().unwrap();
    let files = [
        (
            "t.yaml",
            clause_set_yaml("with-text", "2", "original", "licensed", None, true),
        ),
        (
            "n.yaml",
            clause_set_yaml("no-text", "1", "original", "licensed", None, false),
        ),
    ];
    let outcome = load(dir.path(), &files);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let module = &outcome.modules[0];
    for set in &module.clause_sets {
        let set: &ClauseSet = set;
        set.validate().unwrap();
    }
    let restricted = check_publishable(module);
    assert_eq!(restricted.len(), 2);
    assert_eq!(restricted[0].id, "no-text");
    assert_eq!(restricted[0].text_clauses, 0);
    assert_eq!(restricted[1].id, "with-text");
    assert_eq!(restricted[1].authority, "example.invalid");
    assert_eq!(restricted[1].version, "2");
    assert_eq!(restricted[1].text_clauses, 1);
}

#[test]
#[trace("TC-2316", "FR-083-AC-6", "FR-083-AC-7")]
fn mixed_module_lists_licensed_sets_in_stable_order_without_text() {
    let dir = tempfile::tempdir().unwrap();
    let files = [
        (
            "z.yaml",
            clause_set_yaml("zeta", "1", "original", "licensed", None, true),
        ),
        (
            "p.yaml",
            clause_set_yaml("public", "1", "original", "original", None, true),
        ),
        (
            "a.yaml",
            clause_set_yaml("alpha", "1", "original", "licensed", None, true),
        ),
    ];
    let outcome = load(dir.path(), &files);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let first = check_publishable(&outcome.modules[0]);
    let second = check_publishable(&outcome.modules[0]);
    assert_eq!(first, second);
    let ids: Vec<&str> = first.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, ["alpha", "zeta"]);
    assert!(!format!("{first:?}").contains(SENTINEL));
}

#[test]
#[trace("TC-2317", "FR-083-AC-8")]
fn result_is_unchanged_after_the_module_directory_is_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let files = [(
        "l.yaml",
        clause_set_yaml("held", "1", "original", "licensed", None, true),
    )];
    let outcome = load(dir.path(), &files);
    let before = check_publishable(&outcome.modules[0]);
    assert_eq!(before.len(), 1);
    drop(dir);
    assert_eq!(check_publishable(&outcome.modules[0]), before);

    let dir = tempfile::tempdir().unwrap();
    let outcome = module(dir.path(), "", &[]);
    assert!(check_publishable(&outcome.modules[0]).is_empty());
}
