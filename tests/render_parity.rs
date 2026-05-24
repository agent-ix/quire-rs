//! Render-parity integration suite (FR-012, Tasks 011/012/013).
//!
//! Reads `tests/render_parity/corpus.yaml`, loads every listed
//! module into a `Registry`, then walks the `cases:` entries
//! comparing `quire_rs::render_by_name` output to the
//! hand-authored / Python-rendered `expected.md` byte-for-byte.
//!
//! Failures print a unified-line diff so the divergence is obvious.
//!
//! The Gate G2 baseline ships under `modules/demo/` (a self-contained
//! hand-authored archetype). Full 17-archetype sweep (Task 013)
//! depends on the spec-artifacts-* corpora living elsewhere; see
//! `tests/render_parity/README.md`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quire_rs::{render_by_name, Registry};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Corpus {
    modules: Vec<ModuleEntry>,
    cases: Vec<CaseEntry>,
}

#[derive(Debug, Deserialize)]
struct ModuleEntry {
    path: PathBuf,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CaseEntry {
    archetype: String,
    input: PathBuf,
    expected: PathBuf,
}

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("render_parity")
}

fn load_corpus() -> Corpus {
    let path = corpus_root().join("corpus.yaml");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_yaml::from_slice(&bytes).expect("corpus.yaml parses")
}

fn build_registry(modules: &[ModuleEntry]) -> Registry {
    // The loader walks one level under each search path looking for
    // module directories. The corpus lists modules directly, so we
    // hand the *parent* of each module path to load_from.
    let mut parents: Vec<PathBuf> = Vec::new();
    for m in modules {
        let abs = corpus_root().join(&m.path);
        let parent = abs.parent().expect("module path has parent").to_path_buf();
        if !parents.contains(&parent) {
            parents.push(parent);
        }
    }
    let parent_refs: Vec<&Path> = parents.iter().map(|p| p.as_path()).collect();
    let r = Registry::load_from(&parent_refs).expect("registry load");
    // Verify each declared module did load.
    for m in modules {
        assert!(
            r.module_names().any(|n| n == m.name),
            "module {:?} did not load (failures: {:?}, diagnostics: {:?})",
            m.name,
            r.failures(),
            r.diagnostics(),
        );
    }
    r
}

fn case_input(case: &CaseEntry) -> serde_json::Value {
    let path = corpus_root().join(&case.input);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn case_expected(case: &CaseEntry) -> String {
    let path = corpus_root().join(&case.expected);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn diff_summary(actual: &str, expected: &str) -> String {
    let a: Vec<&str> = actual.lines().collect();
    let e: Vec<&str> = expected.lines().collect();
    let max = a.len().max(e.len());
    let mut out = String::new();
    for i in 0..max {
        let av = a.get(i).copied().unwrap_or("<EOF>");
        let ev = e.get(i).copied().unwrap_or("<EOF>");
        if av != ev {
            out.push_str(&format!(
                "line {i:>3}:\n  -expected: {ev}\n  +actual  : {av}\n"
            ));
        }
    }
    out
}

#[test]
fn corpus_yaml_loads() {
    let c = load_corpus();
    assert!(!c.modules.is_empty(), "corpus.yaml has at least one module");
    assert!(!c.cases.is_empty(), "corpus.yaml has at least one case");
}

#[test]
fn modules_in_corpus_load() {
    let c = load_corpus();
    let r = build_registry(&c.modules);
    assert!(r.failures().is_empty(), "load failures: {:?}", r.failures());
}

#[test]
fn every_case_matches_expected_byte_for_byte() {
    let c = load_corpus();
    let r = build_registry(&c.modules);
    let mut failed: BTreeMap<String, String> = BTreeMap::new();
    for case in &c.cases {
        let input = case_input(case);
        let expected = case_expected(case);
        let actual = match render_by_name(&r, &case.archetype, &input) {
            Ok(s) => s,
            Err(e) => {
                failed.insert(
                    format!("{} :: {}", case.archetype, case.input.display()),
                    format!("RENDER ERROR: {e}"),
                );
                continue;
            }
        };
        if actual != expected {
            failed.insert(
                format!("{} :: {}", case.archetype, case.input.display()),
                diff_summary(&actual, &expected),
            );
        }
    }
    if !failed.is_empty() {
        let mut msg = String::from("render-parity failures:\n");
        for (k, v) in &failed {
            msg.push_str(&format!("== {k} ==\n{v}\n"));
        }
        panic!("{msg}");
    }
}
