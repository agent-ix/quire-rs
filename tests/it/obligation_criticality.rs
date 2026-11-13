//! FR-053-AC-24..26 — a requirement's rating reaches the obligations derived
//! from it, declared by `criticality_field` module data (PLAT-1597).

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::coverage::compute;
use quire_rs::obligation::{derive, for_document};
use quire_rs::symbols::{extract_tree, trace};
use quire_rs::traceability::TraceabilityModel;
use quire_rs::{Registry, Spec};

fn fixture_module(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability")
        .join(name)
}

fn model_of(module: &str) -> TraceabilityModel {
    Registry::load_module(&fixture_module(module))
        .expect("load module")
        .traceability()
        .cloned()
        .expect("model declared")
}

/// A model whose one source reads `rating` from the document.
fn rated_model() -> TraceabilityModel {
    let mut model = model_of("obligations");
    model.obligations[0].criticality_field = Some("rating".to_string());
    model
}

fn doc(id: &str, frontmatter_extra: &str, rows: &str) -> String {
    format!(
        "---\nid: {id}\ntype: FR\ntitle: A requirement\n{frontmatter_extra}---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification | Priority |\n\
         |----|----------|--------------|----------|\n{rows}"
    )
}

fn bundle(name: &str, docs: &[(&str, String)]) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let scope = dir.path().join("spec");
    fs::create_dir_all(&scope).unwrap();
    for (file, text) in docs {
        fs::write(scope.join(file), text).unwrap();
    }
    let _ = name;
    (dir, scope)
}

fn criticality_of(obligations: &[quire_rs::Obligation], id: &str) -> Option<String> {
    obligations
        .iter()
        .find(|o| o.id == id)
        .unwrap_or_else(|| panic!("{id} missing from {obligations:#?}"))
        .criticality
        .clone()
}

#[trace("FR-053-AC-24")]
#[test]
fn tc_rating_reaches_every_criterion_and_absence_stays_absent() {
    let rows = "| FR-001-AC-1 | The system shall do it. | Test | |\n\
                | FR-001-AC-2 | The system shall also. | Analysis | |\n";
    let (_dir, scope) = bundle(
        "rated",
        &[
            ("FR-001.md", doc("FR-001", "rating: high\n", rows)),
            (
                "FR-002.md",
                doc("FR-002", "", &rows.replace("FR-001", "FR-002")),
            ),
            (
                "FR-003.md",
                doc("FR-003", "rating:\n", &rows.replace("FR-001", "FR-003")),
            ),
            (
                "FR-004.md",
                doc(
                    "FR-004",
                    "rating: \"  \"\n",
                    &rows.replace("FR-001", "FR-004"),
                ),
            ),
            (
                "FR-005.md",
                doc("FR-005", "rating: 3\n", &rows.replace("FR-001", "FR-005")),
            ),
        ],
    );
    let spec = Spec::from_path(&scope);
    let (obligations, _) = derive(&spec, &scope, &rated_model());

    assert_eq!(
        criticality_of(&obligations, "FR-001-AC-1").as_deref(),
        Some("high")
    );
    assert_eq!(
        criticality_of(&obligations, "FR-001-AC-2").as_deref(),
        Some("high")
    );
    for id in ["FR-002-AC-1", "FR-003-AC-1", "FR-004-AC-1", "FR-005-AC-1"] {
        assert_eq!(criticality_of(&obligations, id), None, "{id}");
    }
}

#[trace("FR-053-AC-25")]
#[test]
fn tc_column_cell_wins_over_the_document_rating() {
    let rows = "| FR-001-AC-1 | The system shall do it. | Test | P1 |\n\
                | FR-001-AC-2 | The system shall also. | Analysis | |\n";
    let text = doc("FR-001", "rating: high\n", rows);
    let (_dir, scope) = bundle("both", &[("FR-001.md", text.clone())]);
    let model = rated_model();

    let spec = Spec::from_path(&scope);
    let (obligations, _) = derive(&spec, &scope, &model);
    assert_eq!(
        criticality_of(&obligations, "FR-001-AC-1").as_deref(),
        Some("P1")
    );
    assert_eq!(
        criticality_of(&obligations, "FR-001-AC-2").as_deref(),
        Some("high")
    );

    // The classification path agrees with the coverage path.
    let parsed = quire_rs::parse_document(&text);
    let by_id = for_document(&model, "FR", &parsed, Some(Path::new("FR-001.md")));
    assert_eq!(by_id["FR-001-AC-1"].criticality.as_deref(), Some("P1"));
    assert_eq!(by_id["FR-001-AC-2"].criticality.as_deref(), Some("high"));
}

#[trace("FR-053-AC-26")]
#[test]
fn tc_rating_is_not_inferred_and_undeclared_sources_are_unchanged() {
    let rows = "| FR-001-AC-1 | The system shall do it. | Test | |\n";
    let related =
        "---\nid: FR-002\ntype: FR\ntitle: Related\nrelationships:\n  - target: \"ix://x/y/FR-001\"\n    type: \"traces_to\"\n---\n\n\
         ## Acceptance Criteria\n\n| ID | Criteria | Verification | Priority |\n\
         |----|----------|--------------|----------|\n| FR-002-AC-1 | The system shall relate. | Test | |\n"
    .to_string();
    let (_dir, scope) = bundle(
        "infer",
        &[
            ("FR-001.md", doc("FR-001", "rating: high\n", rows)),
            ("FR-002.md", related),
        ],
    );
    let spec = Spec::from_path(&scope);

    let (rated, _) = derive(&spec, &scope, &rated_model());
    assert_eq!(
        criticality_of(&rated, "FR-001-AC-1").as_deref(),
        Some("high")
    );
    assert_eq!(criticality_of(&rated, "FR-002-AC-1"), None);

    // No declaration: the document's field is not read.
    let (plain, _) = derive(&spec, &scope, &model_of("obligations"));
    assert_eq!(criticality_of(&plain, "FR-001-AC-1"), None);
}

#[trace("FR-053-AC-26")]
#[test]
fn tc_coverage_json_carries_the_rating_and_validates() {
    let rows = "| FR-001-AC-1 | The system shall do it. | Test | |\n";
    let (dir, scope) = bundle(
        "json",
        &[
            ("FR-001.md", doc("FR-001", "rating: high\n", rows)),
            (
                "FR-002.md",
                doc("FR-002", "", &rows.replace("FR-001", "FR-002")),
            ),
        ],
    );
    let source = dir.path().join("src");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("lib.rs"), "//! empty\n").unwrap();

    let registry = Registry::load_module(&fixture_module("obligations-rated")).unwrap();
    let model = registry.traceability().cloned().unwrap();
    let graph = trace::bind(&extract_tree(&source), &model);
    let report = compute(&Spec::from_path(&scope), &registry, &graph, &scope).unwrap();
    let payload = serde_json::to_value(&report).unwrap();

    let entries = payload["obligations"].as_array().unwrap();
    let by_id = |id: &str| entries.iter().find(|o| o["id"] == id).unwrap();
    assert_eq!(by_id("FR-001-AC-1")["criticality"], "high");
    assert!(by_id("FR-002-AC-1").get("criticality").is_none());

    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../schemas/output/coverage-v1.schema.json")).unwrap();
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema)
        .unwrap();
    let errors: Vec<_> = validator
        .iter_errors(&payload)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}
