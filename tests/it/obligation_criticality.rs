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

fn bundle(docs: &[(&str, String)]) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let scope = dir.path().join("spec");
    fs::create_dir_all(&scope).unwrap();
    for (file, text) in docs {
        fs::write(scope.join(file), text).unwrap();
    }
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
    let (_dir, scope) = bundle(&[
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
    ]);
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
    let (_dir, scope) = bundle(&[("FR-001.md", text.clone())]);
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

    // The single-document derivation agrees with the coverage path.
    let parsed = quire_rs::parse_document(&text);
    let by_id = for_document(&model, "FR", &parsed, Some(Path::new("FR-001.md")));
    assert_eq!(by_id["FR-001-AC-1"].criticality.as_deref(), Some("P1"));
    assert_eq!(by_id["FR-001-AC-2"].criticality.as_deref(), Some("high"));

    // And so does the property classification record itself.
    let registry = Registry::load_module(&fixture_module("obligations-rated")).unwrap();
    let records = quire_rs::classify_document_criteria(
        &registry,
        registry.archetype("FR").unwrap(),
        &text,
        Some(Path::new("FR-001.md")),
    );
    let on_record = |id: &str| {
        records
            .iter()
            .find(|r| r.row_id.as_deref() == Some(id))
            .and_then(|r| r.obligation.as_ref())
            .unwrap_or_else(|| panic!("{id} has no obligation on its record"))
            .criticality
            .clone()
    };
    assert_eq!(on_record("FR-001-AC-1").as_deref(), Some("P1"));
    assert_eq!(on_record("FR-001-AC-2").as_deref(), Some("high"));
}

#[trace("FR-053-AC-26")]
#[test]
fn tc_rating_is_not_inferred_across_any_relationship_type() {
    let rows = "| FR-001-AC-1 | The system shall do it. | Test | |\n";
    let verbs = ["traces_to", "satisfies", "implements", "refines"];
    let mut docs = vec![(
        "FR-001.md".to_string(),
        doc("FR-001", "rating: high\n", rows),
    )];
    for (n, verb) in verbs.iter().enumerate() {
        let id = format!("FR-{:03}", n + 2);
        let related = format!(
            "rating:\nrelationships:\n  - target: \"ix://x/y/FR-001\"\n    type: \"{verb}\"\n"
        );
        // A null `rating:` key and the relationship: nothing to inherit from.
        docs.push((
            format!("{id}.md"),
            doc(
                &id,
                &related,
                &format!("| {id}-AC-1 | The system shall relate. | Test | |\n"),
            ),
        ));
    }
    let docs: Vec<(&str, String)> = docs.iter().map(|(f, t)| (f.as_str(), t.clone())).collect();
    let (_dir, scope) = bundle(&docs);
    let (rated, _) = derive(&Spec::from_path(&scope), &scope, &rated_model());

    assert_eq!(
        criticality_of(&rated, "FR-001-AC-1").as_deref(),
        Some("high")
    );
    for n in 2..=5 {
        assert_eq!(
            criticality_of(&rated, &format!("FR-{n:03}-AC-1")),
            None,
            "FR-{n:03}"
        );
    }
}

#[trace("FR-053-AC-27")]
#[test]
fn tc_undeclared_source_records_equal_those_without_the_rating() {
    let rows = "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n\
                | FR-001-AC-2 | The system shall also. | Analysis | |\n";
    let (_rated_dir, rated_scope) = bundle(&[("FR-001.md", doc("FR-001", "rating: high\n", rows))]);
    let (_plain_dir, plain_scope) = bundle(&[("FR-001.md", doc("FR-001", "", rows))]);
    let model = model_of("obligations");

    let (with_rating, _) = derive(&Spec::from_path(&rated_scope), &rated_scope, &model);
    let (without, _) = derive(&Spec::from_path(&plain_scope), &plain_scope, &model);
    assert_eq!(with_rating.len(), 2);
    assert_eq!(with_rating, without);
}

#[trace("FR-053-AC-28")]
#[test]
fn tc_coverage_json_carries_the_rating_and_validates() {
    let rows = "| FR-001-AC-1 | The system shall do it. | Test | |\n";
    let (dir, scope) = bundle(&[
        ("FR-001.md", doc("FR-001", "rating: high\n", rows)),
        (
            "FR-002.md",
            doc("FR-002", "", &rows.replace("FR-001", "FR-002")),
        ),
    ]);
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

#[trace("FR-053-AC-24")]
#[test]
fn tc_selected_inventory_carries_the_rating() {
    let rows = "| FR-001-AC-1 | The system shall do it. | Test | |\n";
    let (_dir, scope) = bundle(&[
        ("FR-001.md", doc("FR-001", "rating: high\n", rows)),
        (
            "FR-002.md",
            doc("FR-002", "", &rows.replace("FR-001", "FR-002")),
        ),
    ]);
    let inventory = quire_rs::obligation::inventory(
        &scope,
        &rated_model(),
        &["acceptance-criterion"],
        Default::default(),
    );
    let rating_of = |id: &str| {
        inventory
            .entries()
            .find(|e| e.obligation().id == id)
            .unwrap_or_else(|| panic!("{id} not inventoried"))
            .obligation()
            .criticality
            .clone()
    };
    assert_eq!(rating_of("FR-001-AC-1").as_deref(), Some("high"));
    assert_eq!(rating_of("FR-002-AC-1"), None);
}

/// A model whose one combinatorial source reads `rating` from the document.
fn combinatorial_model() -> TraceabilityModel {
    use quire_rs::traceability::{CombinatorialColumns, ObligationSource};
    let model = TraceabilityModel {
        obligations: vec![ObligationSource {
            name: "configuration-space".to_string(),
            target: None,
            archetype: Some("FR".to_string()),
            section: Some("Configuration Dimensions".into()),
            id_format: Some("{document}-COMB".to_string()),
            exclude: vec![],
            statement_column: "Dimension".to_string(),
            method_column: None,
            criticality_column: None,
            criticality_field: Some("rating".to_string()),
            parameters: Default::default(),
            combinatorial: Some(CombinatorialColumns {
                dimension_column: "Dimension".to_string(),
                values_column: "Values".to_string(),
                excludes_column: None,
                strength: 2,
            }),
        }],
        ..TraceabilityModel::default()
    };
    model.validate().expect("a well-formed source loads");
    model
}

fn space_doc(id: &str, extra: &str) -> String {
    format!(
        "---\nid: {id}\ntype: FR\ntitle: A requirement\n{extra}---\n\n\
         ## Configuration Dimensions\n\n| Dimension | Values |\n|---|---|\n\
         | features | default, python |\n| target | linux, wasm32 |\n"
    )
}

#[trace("FR-053-AC-29")]
#[test]
fn tc_combinatorial_obligation_carries_the_rating_on_every_surface() {
    let (_dir, scope) = bundle(&[
        ("FR-001.md", space_doc("FR-001", "rating: high\n")),
        ("FR-002.md", space_doc("FR-002", "")),
    ]);
    let model = combinatorial_model();

    let (derived, _) = derive(&Spec::from_path(&scope), &scope, &model);
    assert_eq!(
        criticality_of(&derived, "FR-001-COMB").as_deref(),
        Some("high")
    );
    assert_eq!(criticality_of(&derived, "FR-002-COMB"), None);

    let single = for_document(
        &model,
        "FR",
        &quire_rs::parse_document(&space_doc("FR-001", "rating: high\n")),
        None,
    );
    assert_eq!(single["FR-001-COMB"].criticality.as_deref(), Some("high"));
    let single = for_document(
        &model,
        "FR",
        &quire_rs::parse_document(&space_doc("FR-002", "")),
        None,
    );
    assert_eq!(single["FR-002-COMB"].criticality, None);

    let inventory = quire_rs::obligation::inventory(
        &scope,
        &model,
        &["configuration-space"],
        Default::default(),
    );
    let rating_of = |id: &str| {
        inventory
            .entries()
            .find(|e| e.obligation().id == id)
            .unwrap_or_else(|| panic!("{id} not inventoried"))
            .obligation()
            .criticality
            .clone()
    };
    assert_eq!(rating_of("FR-001-COMB").as_deref(), Some("high"));
    assert_eq!(rating_of("FR-002-COMB"), None);
}

#[trace("FR-053-AC-30")]
#[test]
fn tc_blank_criticality_field_fails_module_load() {
    for blank in ["", "   "] {
        let mut model = rated_model();
        model.obligations[0].criticality_field = Some(blank.to_string());
        let err = model.validate().expect_err("a blank field is a load error");
        assert!(
            err.contains("acceptance-criterion") && err.contains("criticality_field"),
            "{err}"
        );
    }
    rated_model().validate().expect("a named field loads");
}

#[trace("FR-053-AC-31")]
#[test]
fn tc_engine_source_holds_no_criticality_key_literal() {
    for (file, text) in [
        ("src/obligation.rs", include_str!("../../src/obligation.rs")),
        (
            "src/obligation/inventory.rs",
            include_str!("../../src/obligation/inventory.rs"),
        ),
        (
            "src/traceability.rs",
            include_str!("../../src/traceability.rs"),
        ),
    ] {
        let production = text.split("#[cfg(test)]").next().unwrap();
        assert!(
            !production.contains("\"criticality\""),
            "{file} names the frontmatter key"
        );
    }
}
