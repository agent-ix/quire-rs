//! FR-050-AC-60 — a trace target mints from table rows, list items or
//! headings, selected by `from:` (TC-2080).

use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::coverage::{compute, CoverageReport};
use quire_rs::symbols::{extract_tree, trace};
use quire_rs::{validate_bundle_at, BundlePosture, Registry, Spec};
use serde_json::Value;

fn cases() -> Vec<Value> {
    let cases: Value = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/trace-target-sources.json"
    ))
    .expect("trace-target-sources fixture");
    cases.as_array().expect("cases").clone()
}

fn case(name: &str) -> Value {
    cases()
        .into_iter()
        .find(|case| case["name"] == name)
        .unwrap_or_else(|| panic!("fixture case {name}"))
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

/// Materialise a case with `module` as its manifest and return the coverage
/// report, the registry and the scope it was computed over.
fn run(case: &Value, module: &str, root: &Path) -> (CoverageReport, Registry) {
    write(&root.join("module"), "manifest.yaml", module);
    let registry = Registry::load_module(&root.join("module")).expect("module loads");
    let scope = root.join("spec");
    for (path, body) in case["input"]["documents"].as_object().expect("documents") {
        write(&scope, path, body.as_str().expect("document text"));
    }
    let source = root.join("src");
    for (path, body) in case["input"]["sources"].as_object().expect("sources") {
        write(&source, path, body.as_str().expect("source text"));
    }
    let model = registry.traceability().expect("model declared");
    let graph = trace::bind(&extract_tree(&source), model);
    let report = compute(&Spec::from_path(&scope), &registry, &graph, &scope).expect("coverage");
    (report, registry)
}

#[trace("FR-050-AC-60")]
#[test]
fn list_items_and_headings_mint_from_capture_group_one_within_their_section() {
    let case = case("list-items-and-headings-mint-within-their-section");
    let root = tempfile::tempdir().expect("scope");
    let module = case["input"]["module"].as_str().expect("module");
    let (report, registry) = run(&case, module, root.path());

    let minted: Vec<Value> = report
        .minted_targets
        .iter()
        .map(|record| {
            serde_json::json!([
                record.target,
                record.id,
                record.document,
                record.line,
                record.backed
            ])
        })
        .collect();
    assert_eq!(
        Value::Array(minted),
        case["expect"]["minted"],
        "ids come from capture group 1, only under the declared section and archetype, \
         never from a fence"
    );

    let untracked: Vec<&str> = report
        .untracked_symbols
        .iter()
        .map(|symbol| symbol.trace_id.as_str())
        .collect();
    assert_eq!(
        serde_json::json!(untracked),
        case["expect"]["untracked_tags"]
    );

    let validation =
        validate_bundle_at(&root.path().join("spec"), &registry, BundlePosture::Strict);
    let dangling: Vec<&str> = validation
        .errors
        .iter()
        .chain(&validation.warnings)
        .filter(|finding| finding.reason == "dangling-trace-reference")
        .map(|finding| finding.message.as_str())
        .collect();
    let expected = case["expect"]["dangling_references"]
        .as_array()
        .expect("dangling references");
    assert_eq!(dangling.len(), expected.len(), "{dangling:?}");
    for (message, id) in dangling.iter().zip(expected) {
        let id = id.as_str().expect("id");
        assert!(
            message.contains(&format!("trace reference '{id}'")),
            "references resolve against list-item and heading ids as they do against rows: \
             {message}"
        );
    }
}

#[trace("FR-050-AC-60")]
#[test]
fn omitted_from_is_table_row_and_reports_identically() {
    let case = case("list-items-and-headings-mint-within-their-section");
    let module = case["input"]["module"].as_str().expect("module");
    let explicit = module.replacen(
        "    section: Acceptance Criteria\n    id_column: ID\n",
        "    section: Acceptance Criteria\n    from: table_row\n    id_column: ID\n",
        1,
    );
    assert_ne!(
        explicit, module,
        "the criterion target gains an explicit `from`"
    );

    let omitted_root = tempfile::tempdir().expect("scope");
    let explicit_root = tempfile::tempdir().expect("scope");
    let (omitted, _) = run(&case, module, omitted_root.path());
    let (stated, registry) = run(&case, &explicit, explicit_root.path());
    assert_eq!(
        serde_json::to_string(&omitted).expect("serialize"),
        serde_json::to_string(&stated).expect("serialize"),
    );
    assert_eq!(
        registry
            .traceability()
            .and_then(|model| model.target("criterion"))
            .and_then(|target| target.id_column()),
        Some("ID")
    );
}

#[trace("FR-050-AC-60")]
#[test]
fn malformed_non_table_sources_fail_module_load() {
    let broken: Vec<Value> = cases()
        .into_iter()
        .filter(|case| case["expect"]["module_error"].is_string())
        .collect();
    assert_eq!(broken.len(), 6);
    for case in broken {
        let name = case["name"].as_str().expect("name");
        let root = tempfile::tempdir().expect("scope");
        write(
            root.path(),
            "manifest.yaml",
            case["input"]["module"].as_str().expect("module"),
        );
        let error = Registry::load_module(root.path())
            .expect_err("a malformed trace target fails the load")
            .to_string();
        let expected = case["expect"]["module_error"].as_str().expect("error");
        assert!(error.contains(expected), "{name}: {error}");
    }
}
