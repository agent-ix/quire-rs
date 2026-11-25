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
fn malformed_non_table_sources_are_dropped() {
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
        let registry = Registry::load_module(root.path())
            .expect("a malformed trace target drops itself, never the module");
        let error = registry
            .failures()
            .iter()
            .map(|failure| failure.reason.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        let expected = case["expect"]["module_error"].as_str().expect("error");
        assert!(error.contains(expected), "{name}: {error}");
    }
}

#[trace("FR-050-AC-70")]
#[test]
fn frontmatter_ids_resolve_typed_documents_without_a_section() {
    let root = tempfile::tempdir().expect("scope");
    let module = "name: frontmatter-targets\nversion: 0.1.0\nartifact_types:\n- name: FR\n- name: TC\ntraceability:\n  trace_targets:\n  - name: typed-case\n    archetype: TC\n    from: frontmatter_id\n    exclude: [fixtures/**]\n  - name: table-case\n    archetype: TC\n    section: Test Procedure\n    id_column: ID\n    required: false\n  document_references:\n  - name: verification\n    archetype: FR\n    section: Acceptance Criteria\n    column: Verification\n    row_id_column: ID\n    pattern: '(TC-[0-9]+)'\n    targets: [typed-case, table-case]\n";
    let case = serde_json::json!({
        "input": {
            "documents": {
                "FR-1.md": "---\nid: FR-1\ntype: FR\n---\n\n## Acceptance Criteria\n\n| ID | Criterion | Verification |\n|---|---|---|\n| FR-1-AC-1 | Covered | TC-7 |\n| FR-1-AC-2 | Missing | TC-99 |\n",
                "TC-7.md": "---\nid: TC-7\ntype: TC\n---\n\n## Test Procedure\n\nRuns the behavior.\n",
                "fixtures/TC-8.md": "---\nid: TC-8\ntype: TC\n---\n\n## Test Procedure\n\nFixture only.\n",
            },
            "sources": {}
        }
    });
    let (report, registry) = run(&case, module, root.path());
    let minted: Vec<_> = report
        .minted_targets
        .iter()
        .map(|record| {
            (
                record.target.as_str(),
                record.id.as_str(),
                record.document.as_str(),
            )
        })
        .collect();
    assert_eq!(minted, [("typed-case", "TC-7", "TC-7.md")]);

    let validation =
        validate_bundle_at(&root.path().join("spec"), &registry, BundlePosture::Strict);
    let dangling: Vec<_> = validation
        .errors
        .iter()
        .chain(&validation.warnings)
        .filter(|finding| finding.reason == "dangling-trace-reference")
        .map(|finding| finding.message.as_str())
        .collect();
    assert_eq!(dangling.len(), 1, "{dangling:?}");
    assert!(dangling[0].contains("TC-99"), "{dangling:?}");
}

#[trace("FR-050-AC-70")]
#[test]
fn frontmatter_id_rejects_section_and_pattern_fields() {
    let root = tempfile::tempdir().expect("module");
    write(
        root.path(),
        "manifest.yaml",
        "name: malformed-frontmatter-target\nversion: 0.1.0\nartifact_types:\n- name: TC\ntraceability:\n  trace_targets:\n  - name: cases\n    archetype: TC\n    from: frontmatter_id\n    section: Tests\n    pattern: '(TC-[0-9]+)'\n",
    );
    let registry = Registry::load_module(root.path()).expect("malformed declaration is dropped");
    assert!(registry.failures().iter().any(|failure| {
        failure.reason.contains("frontmatter_id")
            && failure
                .reason
                .contains("no `section`, `id_column`, or `pattern`")
    }));
    assert!(
        registry.traceability().is_none(),
        "the only invalid declaration is removed with its empty model"
    );
}

#[trace("FR-050-AC-70")]
#[test]
fn frontmatter_targets_report_missing_required_archetype_only() {
    let root = tempfile::tempdir().expect("scope");
    let module = "name: absent-frontmatter-targets\nversion: 0.1.0\nartifact_types:\n- name: TC\ntraceability:\n  trace_targets:\n  - name: required-case\n    archetype: TC\n    from: frontmatter_id\n  - name: optional-case\n    archetype: TC\n    from: frontmatter_id\n    required: false\n";
    let case = serde_json::json!({
        "input": {
            "documents": {},
            "sources": {}
        }
    });
    let (report, _) = run(&case, module, root.path());

    let archetype_diagnostics: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.reason == "archetype-matches-nothing")
        .map(|diagnostic| diagnostic.declaration.as_str())
        .collect();
    assert_eq!(archetype_diagnostics, ["required-case"]);
}
