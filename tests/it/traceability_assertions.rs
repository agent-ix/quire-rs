//! RoleSpec-style parent groups, orphan warnings and table assertions
//! (SWM-247/248; FR-050-AC-71..74).

use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::coverage::{compute, CoverageReport};
use quire_rs::symbols::{extract_tree, trace};
use quire_rs::{validate_bundle_at, BundlePosture, Registry, Spec};
use serde_json::Value;

const MODULE: &str = r#"
name: role-spec-fixture
version: 0.1.0
artifact_types:
- name: RoleSpec
traceability:
  trace_targets:
  - name: role-rule
    archetype: RoleSpec
    section: Rules
    id_column: ID
    evidence: reference-only
  - name: role-evaluation
    archetype: RoleSpec
    section: Evaluations
    id_column: ID
    parent_target: role-rule
    parent_id_column: Rule
    all_children_value_warning:
      column: Method
      value: judgement
  - name: role-metric
    archetype: RoleSpec
    section: Metrics
    id_column: ID
    evidence: reference-only
    required_references: [role-objective-metrics]
  - name: role-objective
    archetype: RoleSpec
    section: Objectives
    id_column: ID
    evidence: reference-only
  document_references:
  - name: role-evaluation-rule
    archetype: RoleSpec
    section: Evaluations
    column: Rule
    row_id_column: ID
    pattern: '(RS-R[0-9]+)'
    targets: [role-rule]
  - name: role-objective-metrics
    archetype: RoleSpec
    section: Objectives
    column: Metrics
    row_id_column: ID
    pattern: '(RS-M[0-9]+)'
    targets: [role-metric]
  table_assertions:
  - kind: unique_ids
    name: unique-role-metric-ids
    target: role-metric
  - kind: all_pairs_unequal
    name: metric-query-differs-from-evaluation-query
    left_target: role-metric
    left_column: Query
    right_target: role-evaluation
    right_column: Query or rubric
"#;

const ROLE_SPEC: &str = r#"---
id: RS
type: RoleSpec
---

## Rules

| ID | Rule |
|---|---|
| RS-R01 | First rule |
| RS-R02 | Rule with no evaluations |
| RS-R03 | Deterministic rule |
| RS-R04 | Mixed rule |

## Evaluations

| ID | Rule | Method | Query or rubric | Trigger |
|---|---|---|---|---|
| RS-R01-E1 | RS-R01 | judgement | rubric@1#check-one | ledger.eval_one |
| RS-R01-E2 | RS-R01 | judgement | rubric@1#check-two | ledger.eval_two |
| RS-R03-E1 | RS-R03 | deterministic | ledger.eval_three | ledger.shared_trigger |
| RS-R04-E1 | RS-R04 | judgement | rubric@1#check-four | ledger.eval_four |
| RS-R04-E2 | RS-R04 | deterministic | ledger.eval_five | ledger.eval_five |

## Metrics

| ID | Query |
|---|---|
| RS-M01 | ledger.metric_one |
| RS-M02 | ledger.metric_unused |

## Objectives

| ID | Metrics |
|---|---|
| RS-O01 | RS-M01 |
"#;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture path has parent")).expect("parent created");
    fs::write(path, content).expect("fixture written");
}

fn setup(module: &str, role_spec: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("fixture root created");
    write(root.path(), "module/manifest.yaml", module);
    write(root.path(), "spec/role.md", role_spec);
    write(root.path(), "src/empty.rs", "fn no_trace_markers() {}\n");
    root
}

fn report(root: &Path) -> (CoverageReport, Registry) {
    let registry = Registry::load_module(&root.join("module")).expect("module loads");
    let model = registry.traceability().expect("traceability model loads");
    let spec_path = root.join("spec");
    let graph = trace::bind(&extract_tree(&root.join("src")), model);
    let report = compute(&Spec::from_path(&spec_path), &registry, &graph, &spec_path)
        .expect("coverage computes");
    (report, registry)
}

fn validate(root: &Path, registry: &Registry) -> quire_rs::BundleReport {
    validate_bundle_at(&root.join("spec"), registry, BundlePosture::Strict)
}

#[trace("FR-050-AC-71")]
#[trace("FR-050-AC-74")]
#[test]
fn matrix_keeps_parent_groups_with_empty_children_and_warns_only_for_all_judgement() {
    let root = setup(MODULE, ROLE_SPEC);
    let (report, registry) = report(root.path());
    let matrix: Value = serde_json::from_str(&report.to_json()).expect("matrix report JSON parses");
    let groups = matrix
        .get("parented_target_groups")
        .and_then(Value::as_array)
        .expect("matrix serializes parent groups");
    assert_eq!(
        groups.len(),
        4,
        "one group for every parent rule: {groups:?}"
    );
    let group = |id: &str| {
        groups
            .iter()
            .find(|group| group.get("parent_id").and_then(Value::as_str) == Some(id))
            .expect("parent group present")
    };
    assert_eq!(
        group("RS-R01")["child_ids"],
        serde_json::json!(["RS-R01-E1", "RS-R01-E2"])
    );
    assert_eq!(group("RS-R02")["child_ids"], serde_json::json!([]));
    assert_eq!(
        group("RS-R03")["child_ids"],
        serde_json::json!(["RS-R03-E1"])
    );

    let warnings: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.reason == "parent-children-all-match")
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "only the all-judgement group warns: {warnings:?}"
    );
    assert!(warnings[0].message.contains("RS-R01"));
    let unmeasured: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.reason == "parent-target-has-no-children")
        .collect();
    assert_eq!(
        unmeasured.len(),
        1,
        "only the empty parent warns: {unmeasured:?}"
    );
    assert!(unmeasured[0].message.contains("RS-R02"));

    let validation = validate(root.path(), &registry);
    assert!(
        validation.errors.is_empty(),
        "warnings do not fail validation: {validation:#?}"
    );
}

#[trace("FR-050-AC-72")]
#[test]
fn each_unreferenced_minted_metric_warns_and_referenced_metric_is_clean() {
    let root = setup(MODULE, ROLE_SPEC);
    let (report, _) = report(root.path());
    let warnings: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.reason == "target-id-unreferenced")
        .collect();
    assert_eq!(warnings.len(), 1, "one unused metric warns: {warnings:?}");
    assert!(warnings[0].message.contains("RS-M02"));
    assert!(!warnings[0].message.contains("RS-M01"));
}

#[trace("FR-050-AC-73")]
#[test]
fn cross_table_equalities_and_duplicate_ids_fail_bundle_validation() {
    let changed_query = ROLE_SPEC.replace("ledger.metric_one", "rubric@1#check-one");
    let duplicate_id = changed_query.replace(
        "RS-M02 | ledger.metric_unused",
        "RS-M01 | ledger.metric_unused",
    );
    let root = setup(MODULE, &duplicate_id);
    let registry = Registry::load_module(&root.path().join("module")).expect("module loads");
    let validation = validate(root.path(), &registry);
    let messages: Vec<_> = validation
        .errors
        .iter()
        .map(|finding| finding.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| { message.contains("RS-M01") && message.contains("duplicate") }),
        "duplicate id is an error naming the id: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| { message.contains("RS-M01") && message.contains("RS-R01-E1") }),
        "cross-table equality names both rows: {messages:?}"
    );
}

#[trace("FR-050-AC-73")]
#[test]
fn equal_trigger_with_distinct_metric_query_is_a_clean_control() {
    let role_spec = ROLE_SPEC.replace("ledger.metric_one", "ledger.shared_trigger");
    let root = setup(MODULE, &role_spec);
    let registry = Registry::load_module(&root.path().join("module")).expect("module loads");
    let validation = validate(root.path(), &registry);
    assert!(
        validation.errors.is_empty(),
        "Trigger is not compared: {:#?}",
        validation.errors
    );
}

#[trace("FR-050-AC-73")]
#[test]
fn table_assertion_declarations_with_unknown_targets_or_blank_columns_are_dropped() {
    for (module, expected) in [
        (
            MODULE.replace("left_target: role-metric", "left_target: missing"),
            "undeclared target 'missing'",
        ),
        (
            MODULE.replace("left_column: Query", "left_column: '  '"),
            "empty `left_column`",
        ),
    ] {
        let root = setup(&module, ROLE_SPEC);
        let registry = Registry::load_module(&root.path().join("module")).expect("module loads");
        assert!(
            registry
                .failures()
                .iter()
                .any(|failure| failure.reason.contains(expected)),
            "load failure should name the malformed assertion {expected:?}: {:?}",
            registry.failures()
        );
    }
}
