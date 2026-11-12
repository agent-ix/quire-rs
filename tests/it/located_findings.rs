//! Located engine findings (PLAT-1478): locator lines (FR-033-AC-18,
//! TC-2300), located bundle findings (FR-049-AC-19..22, TC-2301..2303,
//! TC-2312) and located record validation (FR-069-AC-21..28,
//! TC-2304..2311).
//!
//! The record-validation modules are inline fixtures. `anomaly` takes the real
//! spec-artifacts-process Anomaly `yield_pattern` (`status`, `resolution`,
//! `safety_impact`, `description`, `assessment`) and adds a `closed_by`
//! locator; `ticket` is a second, unrelated archetype. The engine knows
//! neither.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::extract::locator::{eval_located, eval_locator_located};
use quire_rs::{
    parse_document, validate_bundle_at, validate_document, validate_document_in_registry,
    BundlePosture, BundleReport, Locator, LocatorPrimitive, Registry, ValidationResult,
};
use serde_json::{json, Value};

const DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

/// The 1-based line of the first line of `doc` containing `needle`.
fn line_of(doc: &str, needle: &str) -> usize {
    doc.lines()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("{needle:?} not in document"))
        + 1
}

// ───────────────────────── TC-2300: locator lines ─────────────────────────

fn primitive(value: Value) -> LocatorPrimitive {
    serde_json::from_value(value).expect("locator")
}

fn lines_of(doc: &str, value: Value) -> Vec<Option<usize>> {
    eval_located(&parse_document(doc), &primitive(value))
        .into_iter()
        .map(|l| l.line)
        .collect()
}

const LINED: &str = "---
id: D-1
meta:
  owner: alice
  deep:
    leaf: 1
tags:
  - a
  - b
flow: {x: 1,
  y: 2}
---

## Notes

- first
- second

## Code

```mermaid
graph TD
```

## Rows

| Id | Name |
|----|------|
| R1 | one  |
| xx | two  |
| R3 | three |
";

#[trace("TC-2300", "FR-033-AC-18")]
#[test]
fn tc2300_each_locator_kind_exposes_the_line_of_its_value() {
    let at = |needle: &str| Some(line_of(LINED, needle));

    // frontmatter_field: the line of the path's last key; a list value has the
    // key's line and no per-item line; a flow-style value too.
    assert_eq!(
        lines_of(LINED, json!({"from": "frontmatter_field", "path": ["id"]})),
        [at("id: D-1")]
    );
    assert_eq!(
        lines_of(
            LINED,
            json!({"from": "frontmatter_field", "path": ["meta", "deep", "leaf"]})
        ),
        [at("leaf: 1")]
    );
    assert_eq!(
        lines_of(
            LINED,
            json!({"from": "frontmatter_field", "path": ["tags"]})
        ),
        [at("tags:")]
    );
    assert_eq!(
        lines_of(
            LINED,
            json!({"from": "frontmatter_field", "path": ["flow", "y"]})
        ),
        [at("y: 2")]
    );

    // heading: the heading's own line.
    assert_eq!(
        lines_of(LINED, json!({"from": "heading", "level": 2})),
        [at("## Notes"), at("## Code"), at("## Rows")]
    );
    // section_body: the heading that owns the body.
    assert_eq!(
        lines_of(
            LINED,
            json!({"from": "section_body", "after_heading": "Notes"})
        ),
        [at("## Notes")]
    );
    // code_block: the opening fence.
    assert_eq!(
        lines_of(LINED, json!({"from": "code_block", "language": "mermaid"})),
        [at("```mermaid")]
    );
    assert_eq!(
        lines_of(
            LINED,
            json!({"from": "code_block", "under_section": "Code", "language": "mermaid"})
        ),
        [at("```mermaid")]
    );
    // list_item: the item, with and without a named section.
    for under in [json!({"under_section": "Notes"}), json!({})] {
        let mut locator = json!({"from": "list_item"});
        locator
            .as_object_mut()
            .unwrap()
            .extend(under.as_object().unwrap().clone());
        assert_eq!(lines_of(LINED, locator), [at("- first"), at("- second")]);
    }
    // table_row: each row's own line.
    assert_eq!(
        lines_of(
            LINED,
            json!({"from": "table_row", "under_section": "Rows", "column": "Id"})
        ),
        [at("| R1 |"), at("| xx |"), at("| R3 |")]
    );
}

#[trace("TC-2300", "FR-033-AC-18")]
#[test]
fn tc2300_the_line_travels_with_the_value_through_filtering() {
    let at = |needle: &str| Some(line_of(LINED, needle));
    // A `regex` that drops the middle row leaves the survivors on their lines.
    assert_eq!(
        lines_of(
            LINED,
            json!({"from": "table_row", "under_section": "Rows", "column": "Id", "regex": "^(R\\d)$"})
        ),
        [at("| R1 |"), at("| R3 |")]
    );
    // A column the table does not have contributes no value and so no line.
    assert!(lines_of(
        LINED,
        json!({"from": "table_row", "under_section": "Rows", "column": "Nope"})
    )
    .is_empty());
    // A fallback chain exposes the lines of the primitive that located.
    let chain: Locator = serde_json::from_value(json!([
        {"from": "section_body", "after_heading": "Absent"},
        {"from": "table_row", "under_section": "Rows", "column": "Name"},
    ]))
    .unwrap();
    let (located, position) = eval_locator_located(&parse_document(LINED), &chain);
    assert_eq!(position, 1);
    assert_eq!(
        located.iter().map(|l| l.line).collect::<Vec<_>>(),
        [at("| R1 |"), at("| xx |"), at("| R3 |")]
    );
}

#[trace("TC-2300", "FR-033-AC-18")]
#[test]
fn tc2300_a_locator_that_located_nothing_exposes_no_line() {
    for locator in [
        json!({"from": "frontmatter_field", "path": ["nope"]}),
        json!({"from": "heading", "level": 4}),
        json!({"from": "section_body", "after_heading": "Absent"}),
        json!({"from": "code_block", "language": "rust"}),
        json!({"from": "list_item", "under_section": "Absent"}),
        json!({"from": "table_row", "under_section": "Absent"}),
    ] {
        assert!(lines_of(LINED, locator.clone()).is_empty(), "{locator}");
    }
    assert!(lines_of(
        "# no frontmatter\n",
        json!({"from": "frontmatter_field", "path": ["id"]})
    )
    .is_empty());
}

#[trace("TC-2300", "FR-033-AC-18")]
#[test]
fn tc2300_an_assert_failure_and_the_exposed_line_are_the_same_number() {
    let manifest = "name: asserts
version: 0.1.0
artifact_types:
- name: Doc
  body_extraction:
    yield_pattern:
      match:
        status: { from: frontmatter_field, path: [status], assert: { choices: [good] } }
        code: { from: frontmatter_field, path: [code], assert: { id_pattern: '^X-' } }
        kind: { from: section_body, after_heading: Kind, assert: { choices: [alpha] } }
        note: { from: section_body, after_heading: Note, assert: { matches: '^fine' } }
        item: { from: list_item, under_section: Items, assert: { choices: [fine] } }
        title: { from: heading, level: 3, assert: { choices: [Known] } }
        rows: { from: table_row, under_section: Rows, assert: { column_choices: { Id: [R1] } } }
";
    let registry = Registry::from_inline_parts(manifest.as_bytes(), &BTreeMap::new()).unwrap();
    let doc = "---\ntype: Doc\nstatus: bad\ncode: Y-1\n---\n\n## Kind\n\nbeta\n\n## Note\n\nodd\n\n## Items\n\n- fine\n- nope\n\n### Odd\n\n## Rows\n\n| Id |\n|----|\n| R1 |\n| R9 |\n";
    let result = validate_document_in_registry(&registry, registry.archetype("Doc").unwrap(), doc);
    let failing: Vec<Option<usize>> = result.errors.iter().map(|e| e.line).collect();
    let parsed = parse_document(doc);
    // The line exposed for the one failing value of each locator.
    let exposed =
        |locator: Value, value: usize| eval_located(&parsed, &primitive(locator))[value].line;
    let expected = [
        exposed(json!({"from": "frontmatter_field", "path": ["status"]}), 0),
        exposed(json!({"from": "frontmatter_field", "path": ["code"]}), 0),
        exposed(json!({"from": "section_body", "after_heading": "Kind"}), 0),
        exposed(json!({"from": "section_body", "after_heading": "Note"}), 0),
        exposed(json!({"from": "list_item", "under_section": "Items"}), 1),
        exposed(json!({"from": "heading", "level": 3}), 0),
        exposed(
            json!({"from": "table_row", "under_section": "Rows", "column": "Id"}),
            1,
        ),
    ];
    assert_eq!(failing, expected, "{:?}", result.errors);
    assert_eq!(expected[0], Some(line_of(doc, "status: bad")));
    assert_eq!(expected[4], Some(line_of(doc, "- nope")));
}

// ───────────────── TC-2301..2303: located trace findings ─────────────────

fn traceability_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("traceability")
        .join(name)
}

fn bundle_dir(suffix: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("quire-rs-located-{}-{suffix}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn write(root: &Path, rel: &str, body: &str) {
    fs::write(root.join(rel), body).unwrap();
}

/// Left/Right target documents plus `links.md` made of `links`.
fn kinds_bundle(suffix: &str, links: &str) -> PathBuf {
    let root = bundle_dir(suffix);
    write(
        &root,
        "left.md",
        "---\nid: LD-001\ntype: Left\ntitle: Left\n---\n\n## Items\n\n| Item |\n|------|\n| L-1 |\n",
    );
    write(
        &root,
        "right.md",
        "---\nid: RD-001\ntype: Right\ntitle: Right\n---\n\n## Items\n\n| Item |\n|------|\n| R-1 |\n",
    );
    write(&root, "links.md", links);
    root
}

fn kinds_registry() -> Registry {
    Registry::load_module(&traceability_fixture("kinds")).unwrap()
}

fn with_reason<'a>(report: &'a BundleReport, reason: &str) -> Vec<&'a quire_rs::BundleFinding> {
    report
        .errors
        .iter()
        .chain(report.warnings.iter())
        .filter(|f| f.reason == reason)
        .collect()
}

const LINKS_HEAD: &str = "---\nid: LK-001\ntype: Link\ntitle: Links\n---\n\n## Links\n\n| Link | Class | Target |\n|------|-------|--------|\n";

#[trace("TC-2301", "FR-049-AC-19")]
#[test]
fn tc2301_a_dangling_id_is_located_at_its_referencing_row() {
    let links = format!(
        "{LINKS_HEAD}| K-0 | north | L-9 |\n| K-1 | north | L-1 |\n| K-2 | north | L-8 and L-9 |\n| K-3 | north | L-9 |\n"
    );
    let root = kinds_bundle("2301", &links);
    let report = validate_bundle_at(&root, &kinds_registry(), BundlePosture::Strict);
    let dangling = with_reason(&report, "dangling-trace-reference");
    let found: Vec<(&str, Option<usize>)> = dangling
        .iter()
        .map(|f| {
            let id = ["L-8", "L-9"]
                .into_iter()
                .find(|id| f.message.contains(&format!("'{id}'")))
                .unwrap();
            (id, f.line)
        })
        .collect();
    let row = |key: &str| Some(line_of(&links, key));
    // The same id dangling on three rows is three findings; both unresolved
    // ids of one multi-id cell carry that row's line.
    assert_eq!(found.len(), 4, "{dangling:?}");
    for expected in [
        ("L-9", row("| K-0 |")),
        ("L-8", row("| K-2 |")),
        ("L-9", row("| K-2 |")),
        ("L-9", row("| K-3 |")),
    ] {
        assert!(found.contains(&expected), "{expected:?} not in {found:?}");
    }
    // Every other pack finding stays unlocated.
    for finding in report.errors.iter().chain(report.warnings.iter()) {
        if finding.reason != "dangling-trace-reference" && finding.pack.is_some() {
            assert_eq!(finding.line, None, "{finding:?}");
        }
    }
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2302", "FR-049-AC-20")]
#[test]
fn tc2302_a_kind_mismatch_is_located_at_its_own_row_below_other_tables() {
    let links = format!(
        "{LINKS_HEAD}| K-0 | north | L-1 |\n\n## More\n\n| Other | Note |\n|-------|------|\n| x | y |\n\n## Links\n\n| Link | Class | Target |\n|------|-------|--------|\n| K-1 | north | R-1 |\n"
    );
    let root = kinds_bundle("2302", &links);
    let report = validate_bundle_at(&root, &kinds_registry(), BundlePosture::Strict);
    let mismatch = with_reason(&report, "reference-kind-mismatch");
    assert_eq!(mismatch.len(), 1, "{:?}", report.errors);
    assert_eq!(mismatch[0].line, Some(line_of(&links, "| K-1 |")));
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2303", "FR-049-AC-21")]
#[test]
fn tc2303_located_findings_are_deterministic_and_follow_the_modules_own_declaration() {
    let links = format!(
        "{LINKS_HEAD}| K-0 | north | L-9 |\n| K-1 | north | R-1 |\n| K-2 | south | L-9 |\n"
    );
    let root = kinds_bundle("2303", &links);
    let registry = kinds_registry();
    let first = validate_bundle_at(&root, &registry, BundlePosture::Strict);
    for _ in 0..3 {
        assert_eq!(
            validate_bundle_at(&root, &registry, BundlePosture::Strict),
            first
        );
    }
    // Ordered (path, id, reason, message, line): the two L-9 rows come out in
    // line order.
    let lines: Vec<usize> = with_reason(&first, "dangling-trace-reference")
        .iter()
        .map(|f| f.line.unwrap())
        .collect();
    assert_eq!(
        lines,
        [line_of(&links, "| K-0 |"), line_of(&links, "| K-2 |")]
    );
    fs::remove_dir_all(&root).ok();

    // A module declaring other column names and a different table position is
    // located by its own declaration.
    let module = bundle_dir("2303-module");
    write(
        &module,
        "manifest.yaml",
        "name: refs-alt\nversion: 0.1.0\nartifact_types:\n- name: Goal\n- name: Memo\ntraceability:\n  trace_targets:\n  - name: goals\n    archetype: Goal\n    section: Goals\n    id_column: Goal\n  document_references:\n  - name: cites\n    archetype: Memo\n    section: Citations\n    column: Cites\n    pattern: '\\b(G\\d+)\\b'\n    targets: [goals]\n",
    );
    let docs = bundle_dir("2303-docs");
    write(
        &docs,
        "goal.md",
        "---\nid: GL-1\ntype: Goal\ntitle: g\n---\n\n## Goals\n\n| Goal |\n|------|\n| G1 |\n",
    );
    let memo = "---\nid: MM-1\ntype: Memo\ntitle: m\n---\n\n## Intro\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n## Citations\n\n| Why | Cites |\n|-----|-------|\n| fine | G1 |\n| lost | G7 |\n| again | G7 |\n";
    write(&docs, "memo.md", memo);
    let registry = Registry::load_module(&module).unwrap();
    let report = validate_bundle_at(&docs, &registry, BundlePosture::Strict);
    let dangling = with_reason(&report, "dangling-trace-reference");
    // No `row_id_column`: the two findings differ by line alone, and the same
    // id dangling on two rows is two findings (FR-049-AC-19).
    let mut found: Vec<Option<usize>> = dangling.iter().map(|f| f.line).collect();
    found.sort();
    assert_eq!(
        found,
        [
            Some(line_of(memo, "| lost |")),
            Some(line_of(memo, "| again |"))
        ],
        "{:?}",
        report.errors
    );
    fs::remove_dir_all(&module).ok();
    fs::remove_dir_all(&docs).ok();
}

// ─────────────────────── record-validation fixtures ───────────────────────

fn inline(manifest: &str, schemas: &[(&str, &str)]) -> Registry {
    let schemas: BTreeMap<String, String> = schemas
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Registry::from_inline_parts(manifest.as_bytes(), &schemas).expect("inline registry")
}

fn validate(registry: &Registry, archetype: &str, doc: &str) -> ValidationResult {
    validate_document_in_registry(registry, registry.archetype(archetype).unwrap(), doc)
}

/// `(line, message)` of each `semantic.record-invalid` error, in report order.
fn record_invalid(result: &ValidationResult) -> Vec<(Option<usize>, String)> {
    result
        .errors
        .iter()
        .filter(|e| e.message.starts_with("semantic.record-invalid"))
        .map(|e| (e.line, e.message.clone()))
        .collect()
}

fn record_lines(result: &ValidationResult) -> Vec<Option<usize>> {
    record_invalid(result).into_iter().map(|(l, _)| l).collect()
}

const SEMANTIC: &str = "semantic:\n  contract_version: 1.0.0\n  package: agent-ix/record-fixture\n  exports: [Anomaly]\n  targets: [json-schema, markdown]\n";

/// The real Anomaly `yield_pattern` of spec-artifacts-process plus `closed_by`.
const ANOMALY_MATCH: &str = "    yield_pattern:
      match:
        status:
          from: frontmatter_field
          path: [status]
          required: true
          assert:
            choices: [identified, assessed, resolved, closed]
        closed_by:
          from: frontmatter_field
          path: [closed_by]
          required: false
        resolution:
          from: frontmatter_field
          path: [resolution]
          required: false
          assert:
            choices: [fixed, not-a-defect, duplicate, wont-fix]
        safety_impact:
          from: frontmatter_field
          path: [safety_impact]
          required: true
          assert:
            choices: [unassessed, none, impacting]
        description:
          from: section_body
          after_heading: Description
          required: true
        assessment:
          from: table_row
          under_section: Safety Impact Assessment
          required: false
          multiple: true
          assert:
            columns: [Kind, Target, Impact, Rationale]
            min_rows: 1
            column_choices:
              Kind: [Requirement, Hazard, None]
              Impact: [Affects, Mitigates, Introduces, Unaffected, None]
            column_patterns:
              Rationale: '\\S'
";

fn anomaly_manifest(record_validation: &str, semantic: &str) -> String {
    format!(
        "name: record-fixture\nversion: 0.1.0\nartifact_types:\n- name: Anomaly\n  body_extraction:\n{ANOMALY_MATCH}{record_validation}{semantic}"
    )
}

const ANOMALY_RV: &str =
    "  record_validation:\n    schema: schemas/AnomalyFields.json\n    locate_absence_at: status\n";

/// The conditional record schema: closed needs `closed_by` and `resolution`,
/// resolved needs `resolution`, and anything assessed or later needs an
/// assessment. It describes the yield record, whose keys are the `match` keys.
fn anomaly_schema() -> String {
    json!({
        "$schema": DRAFT,
        "type": "object",
        "properties": {
            "status": {"enum": ["identified", "assessed", "resolved", "closed"]},
            "safety_impact": {"enum": ["unassessed", "none", "impacting"]}
        },
        "allOf": [
            {"if": {"properties": {"status": {"const": "closed"}}, "required": ["status"]},
             "then": {"required": ["closed_by", "resolution"]}},
            {"if": {"properties": {"status": {"const": "resolved"}}, "required": ["status"]},
             "then": {"required": ["resolution"]}},
            {"if": {"properties": {"status": {"enum": ["assessed", "resolved", "closed"]}}, "required": ["status"]},
             "then": {"required": ["assessment"]}}
        ]
    })
    .to_string()
}

fn anomaly_registry() -> Registry {
    let registry = inline(
        &anomaly_manifest(ANOMALY_RV, SEMANTIC),
        &[("schemas/AnomalyFields.json", &anomaly_schema())],
    );
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    registry
}

const ASSESSMENT: &str = "\n## Safety Impact Assessment\n\n| Kind | Target | Impact | Rationale |\n|------|--------|--------|-----------|\n| None | n/a | None | nothing |\n";

fn anomaly_doc(frontmatter: &str, assessed: bool) -> String {
    format!(
        "---\nid: AN-001\ntype: Anomaly\n{frontmatter}safety_impact: none\n---\n\n## Description\n\nIt broke.\n{}",
        if assessed { ASSESSMENT } else { "" }
    )
}

// ──────────────────────── TC-2304 / TC-2305: load ────────────────────────

#[trace("TC-2304", "FR-069-AC-21")]
#[test]
fn tc2304_record_validation_loads_and_is_recorded_beside_data_schema() {
    let data = json!({"$schema": DRAFT, "title": "declaration", "type": "object"}).to_string();
    let manifest = format!(
        "name: record-fixture\nversion: 0.1.0\nartifact_types:\n- name: Anomaly\n  data_schema:\n    schema: schemas/Data.json\n  body_extraction:\n{ANOMALY_MATCH}{ANOMALY_RV}{SEMANTIC}"
    );
    let registry = inline(
        &manifest,
        &[
            ("schemas/Data.json", &data),
            ("schemas/AnomalyFields.json", &anomaly_schema()),
        ],
    );
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let archetype = registry.archetype("Anomaly").unwrap();
    let declared = archetype.record_validation.as_ref().expect("recorded");
    assert_eq!(declared.locate_absence_at, "status");
    assert!(declared.schema["allOf"].is_array());
    // `data_schema` still validates the declaration record only.
    assert_eq!(
        archetype.data_schema.as_ref().unwrap()["title"],
        "declaration"
    );

    // The same manifest without the declaration loads as before and checks no
    // record.
    let without = format!(
        "name: record-fixture\nversion: 0.1.0\nartifact_types:\n- name: Anomaly\n  body_extraction:\n{ANOMALY_MATCH}{SEMANTIC}"
    );
    let registry = inline(&without, &[]);
    assert!(registry.failures().is_empty());
    assert!(registry
        .archetype("Anomaly")
        .unwrap()
        .record_validation
        .is_none());
    let result = validate(&registry, "Anomaly", &anomaly_doc("status: closed\n", true));
    assert!(record_invalid(&result).is_empty(), "{:?}", result.errors);
}

/// A module with a refused `Bad` archetype and a valid `Good` one.
fn refused(bad_body: &str, bad_rv: &str, semantic: &str) -> Registry {
    let manifest = format!(
        "name: record-fixture\nversion: 0.1.0\nartifact_types:\n- name: Good\n  body_extraction:\n    yield_pattern:\n      match:\n        status: {{ from: frontmatter_field, path: [status], required: true }}\n  record_validation:\n    schema: schemas/Fields.json\n    locate_absence_at: status\n- name: Bad\n{bad_body}{bad_rv}{semantic}"
    );
    let fields = json!({"$schema": DRAFT, "type": "object"}).to_string();
    inline(&manifest, &[("schemas/Fields.json", &fields)])
}

const BAD_MATCH: &str = "  body_extraction:\n    yield_pattern:\n      match:\n        status: { from: frontmatter_field, path: [status], required: true }\n";
const GOOD_SEMANTIC: &str = "semantic:\n  contract_version: 1.0.0\n  package: agent-ix/record-fixture\n  exports: [Good]\n  targets: [json-schema, markdown]\n";

#[trace("TC-2305", "FR-069-AC-22")]
#[test]
fn tc2305_each_malformed_declaration_is_refused_at_its_manifest_path() {
    let cases: [(&str, &str, &str, &str); 11] = [
        (
            "unknown sibling key",
            BAD_MATCH,
            "  record_validation:\n    schema: schemas/Fields.json\n    locate_absence_at: status\n    extra: 1\n",
            "semantic.invalid-value: artifact_types[Bad].record_validation:",
        ),
        (
            "iterate_over only",
            "  body_extraction:\n    yield_pattern:\n      iterate_over: { section_path: [Items], kind: heading }\n      per_match:\n        name: { from: heading, level: 3 }\n",
            "  record_validation:\n    schema: schemas/Fields.json\n    locate_absence_at: name\n",
            "semantic.invalid-value: artifact_types[Bad].record_validation:",
        ),
        (
            "no yield_pattern",
            "",
            "  record_validation:\n    schema: schemas/Fields.json\n    locate_absence_at: status\n",
            "semantic.invalid-value: artifact_types[Bad].record_validation:",
        ),
        (
            "schema absent",
            BAD_MATCH,
            "  record_validation:\n    locate_absence_at: status\n",
            "semantic.invalid-value: artifact_types[Bad].record_validation.schema:",
        ),
        (
            "schema unresolvable",
            BAD_MATCH,
            "  record_validation:\n    schema: schemas/Missing.json\n    locate_absence_at: status\n",
            "semantic.data-schema-missing: artifact_types[Bad].record_validation.schema:",
        ),
        (
            "schema escapes the module",
            BAD_MATCH,
            "  record_validation:\n    schema: ../Fields.json\n    locate_absence_at: status\n",
            "semantic.data-schema-escape: artifact_types[Bad].record_validation.schema:",
        ),
        (
            "locate_absence_at absent",
            BAD_MATCH,
            "  record_validation:\n    schema: schemas/Fields.json\n",
            "semantic.invalid-value: artifact_types[Bad].record_validation.locate_absence_at:",
        ),
        (
            "locate_absence_at empty",
            BAD_MATCH,
            "  record_validation:\n    schema: schemas/Fields.json\n    locate_absence_at: ''\n",
            "semantic.invalid-value: artifact_types[Bad].record_validation.locate_absence_at:",
        ),
        (
            "locate_absence_at not a string",
            BAD_MATCH,
            "  record_validation:\n    schema: schemas/Fields.json\n    locate_absence_at: 7\n",
            "semantic.invalid-value: artifact_types[Bad].record_validation.locate_absence_at:",
        ),
        (
            "locate_absence_at not a match key",
            BAD_MATCH,
            "  record_validation:\n    schema: schemas/Fields.json\n    locate_absence_at: nope\n",
            "semantic.invalid-value: artifact_types[Bad].record_validation.locate_absence_at:",
        ),
        (
            "not a mapping",
            BAD_MATCH,
            "  record_validation: schemas/Fields.json\n",
            "semantic.invalid-value: artifact_types[Bad].record_validation:",
        ),
    ];
    for (name, body, rv, expected) in cases {
        let registry = refused(body, rv, GOOD_SEMANTIC);
        let reasons: Vec<&str> = registry
            .failures()
            .iter()
            .filter(|f| f.archetype == "Bad")
            .map(|f| f.reason.as_str())
            .collect();
        assert!(
            reasons.iter().any(|r| r.starts_with(expected)),
            "{name}: {reasons:?}"
        );
        assert!(
            registry.archetype("Good").is_some(),
            "{name}: a valid declaration in the same module is not refused"
        );
        assert!(
            registry.failures().iter().all(|f| f.archetype == "Bad"),
            "{name}: {:?}",
            registry.failures()
        );
    }

    // A module without a semantic block refuses the declaration.
    let registry = refused(
        BAD_MATCH,
        "  record_validation:\n    schema: schemas/Fields.json\n    locate_absence_at: status\n",
        "",
    );
    assert!(
        registry.failures().iter().any(|f| f
            .reason
            .starts_with("semantic.invalid-value: artifact_types[Bad].record_validation:")),
        "{:?}",
        registry.failures()
    );
}

// ─────────────────── TC-2306 .. TC-2310: Anomaly checks ───────────────────

#[trace("TC-2306", "FR-069-AC-23")]
#[test]
fn tc2306_every_validator_error_is_one_finding_in_validator_order() {
    let registry = anomaly_registry();
    let doc = anomaly_doc("status: closed\n", true);
    let result = validate(&registry, "Anomaly", &doc);
    let found = record_invalid(&result);
    assert_eq!(found.len(), 2, "{:?}", result.errors);
    // The schema's `required` order: closed_by, then resolution.
    assert!(found[0].1.contains("closed_by"), "{found:?}");
    assert!(found[1].1.contains("resolution"), "{found:?}");
    assert!(found[0].1.contains("[Anomaly]"));

    // The check is deterministic.
    assert_eq!(validate(&registry, "Anomaly", &doc), result);

    // A conforming document has none; the type-only entry point has none even
    // for the failing document.
    let conforming = anomaly_doc(
        "status: closed\nclosed_by: AN-002\nresolution: fixed\n",
        true,
    );
    assert!(record_invalid(&validate(&registry, "Anomaly", &conforming)).is_empty());
    let type_only = validate_document(registry.archetype("Anomaly").unwrap(), &doc);
    assert!(
        record_invalid(&type_only).is_empty(),
        "{:?}",
        type_only.errors
    );
}

#[trace("TC-2306", "FR-069-AC-23")]
#[test]
fn tc2306_an_object_archetype_that_declares_it_is_checked_too() {
    let manifest = "name: object-fixture
version: 0.1.0
artifact_types:
- name: Plain
object_types:
- name: Thing
  body_extraction:
    yield_pattern:
      match:
        state: { from: frontmatter_field, path: [state], required: true }
        owner: { from: frontmatter_field, path: [owner], required: false }
  record_validation:
    schema: schemas/ThingFields.json
    locate_absence_at: state
semantic:
  contract_version: 1.0.0
  package: agent-ix/object-fixture
  exports: [Thing]
  targets: [json-schema, markdown]
";
    let schema = json!({"$schema": DRAFT, "type": "object", "required": ["owner"]}).to_string();
    let registry = inline(manifest, &[("schemas/ThingFields.json", &schema)]);
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let doc = "---\ntype: Plain\nobject: Thing\nstate: open\n---\n\nbody\n";
    let result = validate(&registry, "Plain", doc);
    assert_eq!(
        record_lines(&result),
        [Some(line_of(doc, "state:"))],
        "{:?}",
        result.errors
    );
}

#[trace("TC-2307", "FR-069-AC-24")]
#[test]
fn tc2307_an_absence_is_located_at_the_absence_locators_value() {
    let registry = anomaly_registry();
    // Conditional absences, both at the `status:` line.
    for (status, expected) in [("closed", 2), ("resolved", 1)] {
        let doc = anomaly_doc(&format!("status: {status}\n"), true);
        let found = record_lines(&validate(&registry, "Anomaly", &doc));
        assert_eq!(
            found,
            vec![Some(line_of(&doc, "status:")); expected],
            "{status}"
        );
    }

    // Outright `required` and an absence locator that located nothing: line 1.
    let ticket = ticket_registry("owner");
    let doc = ticket_doc("state: open\n", "");
    let result = validate(&ticket, "Ticket", &doc);
    assert_eq!(record_lines(&result), [Some(1)], "{:?}", result.errors);
    let ticket = ticket_registry("state");
    let result = validate(&ticket, "Ticket", &doc);
    assert_eq!(
        record_lines(&result),
        [Some(line_of(&doc, "state:"))],
        "{:?}",
        result.errors
    );
}

#[trace("TC-2309", "FR-069-AC-26")]
#[test]
fn tc2309_one_defect_is_one_finding() {
    let registry = anomaly_registry();

    // A `required: true` locator that located nothing: its own finding and no
    // record finding.
    let doc =
        "---\nid: AN-001\ntype: Anomaly\nsafety_impact: none\n---\n\n## Description\n\nIt broke.\n";
    let result = validate(&registry, "Anomaly", doc);
    assert!(record_invalid(&result).is_empty(), "{:?}", result.errors);
    assert_eq!(
        result
            .errors
            .iter()
            .filter(|e| e.message.contains("required 'status'"))
            .count(),
        1,
        "{:?}",
        result.errors
    );

    // A value failing the locator's `choices` and the schema's `enum` is the
    // assert finding only.
    let doc = anomaly_doc("status: bogus\n", true);
    let result = validate(&registry, "Anomaly", &doc);
    assert!(record_invalid(&result).is_empty(), "{:?}", result.errors);
    assert_eq!(
        result
            .errors
            .iter()
            .filter(|e| e.message.contains("'status'"))
            .count(),
        1,
        "{:?}",
        result.errors
    );

    // A conditional absence has no locator finding and is reported.
    let doc = anomaly_doc("status: resolved\n", true);
    assert_eq!(
        record_invalid(&validate(&registry, "Anomaly", &doc)).len(),
        1
    );
}

#[trace("TC-2310", "FR-069-AC-27")]
#[test]
fn tc2310_the_anomaly_fixture_reports_conditionals_at_the_status_line() {
    let registry = anomaly_registry();

    let closed = anomaly_doc("status: closed\n", true);
    assert_eq!(
        record_lines(&validate(&registry, "Anomaly", &closed)),
        [
            Some(line_of(&closed, "status:")),
            Some(line_of(&closed, "status:"))
        ]
    );

    let assessed = anomaly_doc("status: assessed\n", false);
    let result = validate(&registry, "Anomaly", &assessed);
    assert_eq!(
        record_lines(&result),
        [Some(line_of(&assessed, "status:"))],
        "{:?}",
        result.errors
    );

    let valid = anomaly_doc(
        "status: closed\nclosed_by: AN-002\nresolution: wont-fix\n",
        true,
    );
    let result = validate(&registry, "Anomaly", &valid);
    assert!(result.is_valid, "{:?}", result.errors);
}

// ─────────────────── TC-2308 / TC-2311: the Ticket module ───────────────────

fn ticket_registry(locate_absence_at: &str) -> Registry {
    let manifest = format!(
        "name: ticket-fixture
version: 0.1.0
artifact_types:
- name: Ticket
  body_extraction:
    yield_pattern:
      match:
        state: {{ from: frontmatter_field, path: [state], required: true }}
        owner: {{ from: frontmatter_field, path: [owner], required: false }}
        meta: {{ from: frontmatter_field, path: [meta], required: false }}
        extra: {{ from: frontmatter_field, path: [extra], required: false }}
        labels: {{ from: list_item, under_section: Labels, regex: '^([A-Za-z]+)$', multiple: true, required: false }}
        refs: {{ from: table_row, under_section: Refs, column: Ref, multiple: true, required: false }}
  record_validation:
    schema: schemas/TicketFields.json
    locate_absence_at: {locate_absence_at}
semantic:
  contract_version: 1.0.0
  package: agent-ix/ticket-fixture
  exports: [Ticket]
  targets: [json-schema, markdown]
"
    );
    let schema = json!({
        "$schema": DRAFT,
        "type": "object",
        "required": ["owner"],
        "additionalProperties": false,
        "properties": {
            "state": {"enum": ["open", "done"]},
            "owner": {"type": "string"},
            "meta": {"type": "object", "properties": {"size": {"type": "integer"}}},
            "labels": {"type": "array", "items": {"pattern": "^[a-z]+$"}},
            "refs": {"type": "array", "items": {"pattern": "^[a-z]+$"}}
        }
    })
    .to_string();
    let registry = inline(&manifest, &[("schemas/TicketFields.json", &schema)]);
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    registry
}

fn ticket_doc(frontmatter: &str, body: &str) -> String {
    format!("---\ntype: Ticket\n{frontmatter}---\n\n# Ticket\n{body}")
}

#[trace("TC-2308", "FR-069-AC-25")]
#[test]
fn tc2308_a_present_but_invalid_value_is_located_at_its_own_line() {
    let registry = ticket_registry("state");

    // A scalar.
    let doc = ticket_doc("owner: ann\nstate: weird\n", "");
    let result = validate(&registry, "Ticket", &doc);
    assert_eq!(
        record_lines(&result),
        [Some(line_of(&doc, "state:"))],
        "{:?}",
        result.errors
    );

    // A `multiple: true` element after a `regex` drop (`!!skip`) and a
    // placeholder drop (`{{ref}}`): each is at the line of the value that
    // became it, not at its index in the record array.
    let body = "\n## Labels\n\n- alpha\n- !!skip\n- BAD\n- gamma\n\n## Refs\n\n| Ref |\n|-----|\n| {{ref}} |\n| fine |\n| WORSE |\n";
    let doc = ticket_doc("owner: ann\nstate: open\n", body);
    let result = validate(&registry, "Ticket", &doc);
    assert_eq!(
        record_lines(&result),
        [
            Some(line_of(&doc, "- BAD")),
            Some(line_of(&doc, "| WORSE |"))
        ],
        "{:?}",
        result.errors
    );

    // A value with no line of its own: the nested `size` is at `meta:`.
    let doc = ticket_doc("owner: ann\nstate: open\nmeta:\n  size: big\n", "");
    let result = validate(&registry, "Ticket", &doc);
    assert_eq!(
        record_lines(&result),
        [Some(line_of(&doc, "meta:"))],
        "{:?}",
        result.errors
    );

    // An error with no property of its own (`additionalProperties`) is at the
    // absence locator's line.
    let doc = ticket_doc("owner: ann\nstate: open\nextra: 1\n", "");
    let result = validate(&registry, "Ticket", &doc);
    let found = record_invalid(&result);
    assert_eq!(found.len(), 1, "{:?}", result.errors);
    assert_eq!(found[0].0, Some(line_of(&doc, "state:")), "{found:?}");
}

#[trace("TC-2311", "FR-069-AC-28")]
#[test]
fn tc2311_another_archetype_locates_at_its_own_state_line_and_repeats_identically() {
    let registry = ticket_registry("state");
    for frontmatter in [
        "state: open\n",
        "owner:\nstate: done\n",
        "state: done\nextra: x\n",
    ] {
        let doc = ticket_doc(frontmatter, "");
        let first = validate(&registry, "Ticket", &doc);
        assert!(!record_invalid(&first).is_empty(), "{frontmatter}");
        for finding in record_invalid(&first) {
            assert_eq!(finding.0, Some(line_of(&doc, "state:")), "{frontmatter}");
        }
        for _ in 0..3 {
            assert_eq!(validate(&registry, "Ticket", &doc), first);
        }
    }
}

// ───────────────────── TC-2312: bridging into the bundle ─────────────────────

#[trace("TC-2312", "FR-049-AC-22")]
#[test]
fn tc2312_bridged_findings_keep_the_line_of_the_document_result() {
    let registry = anomaly_registry();
    let root = bundle_dir("2312");
    let located = anomaly_doc("status: closed\n", true);
    write(&root, "located.md", &located);
    // A result with an unlocated error: the required description is missing.
    let unlocated = "---\nid: AN-002\ntype: Anomaly\nstatus: identified\nsafety_impact: none\n---\n\nno sections\n";
    write(&root, "unlocated.md", unlocated);
    // A result carrying a line from another check: a duplicated heading.
    let duplicate = format!(
        "{}\n## Description\n\nagain.\n",
        anomaly_doc("status: identified\n", false)
    );
    write(&root, "duplicate.md", &duplicate);

    let report = validate_bundle_at(&root, &registry, BundlePosture::Strict);
    let by_file = |file: &str| -> Vec<&quire_rs::BundleFinding> {
        report
            .errors
            .iter()
            .filter(|f| f.path.ends_with(file))
            .collect()
    };

    // The record finding reaches the bundle at the line it has from
    // `validate_document_in_registry`.
    let direct = validate(&registry, "Anomaly", &located);
    let bridged: Vec<Option<usize>> = by_file("located.md")
        .iter()
        .filter(|f| f.message.starts_with("semantic.record-invalid"))
        .map(|f| f.line)
        .collect();
    assert_eq!(bridged, record_lines(&direct));
    assert_eq!(bridged.len(), 2);

    // Every bridged error carries exactly the line of its document result.
    for (file, text) in [
        ("located.md", located.as_str()),
        ("unlocated.md", unlocated),
        ("duplicate.md", duplicate.as_str()),
    ] {
        let result = validate(&registry, "Anomaly", text);
        let bridged: Vec<(String, Option<usize>)> = by_file(file)
            .iter()
            .filter(|f| f.pack.is_none())
            .map(|f| (f.message.clone(), f.line))
            .collect();
        for error in &result.errors {
            assert!(
                bridged.contains(&(error.message.clone(), error.line)),
                "{file}: {error:?} not in {bridged:?}"
            );
        }
    }
    assert!(by_file("unlocated.md").iter().any(|f| f.line.is_none()));
    let duplicate_line = by_file("duplicate.md")
        .iter()
        .find(|f| f.reason == "duplicate-heading")
        .and_then(|f| f.line);
    assert_eq!(duplicate_line, Some(duplicate.lines().count() - 2));

    // Findings of the engine's own packs stay unlocated.
    for finding in report.errors.iter().chain(report.warnings.iter()) {
        if finding.pack.is_some() {
            assert_eq!(finding.line, None, "{finding:?}");
        }
    }
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2312", "FR-049-AC-22")]
#[test]
fn tc2312_a_bridged_warning_keeps_its_line() {
    let registry = inline(
        "name: warn-fixture\nversion: 0.1.0\nartifact_types:\n- name: FR\n  grammar_ref: iso-spec-core\n",
        &[],
    );
    let doc = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n## Description\n\nThe system shall provide adequate throughput.\n";
    let direct = validate(&registry, "FR", doc);
    let direct_lines: Vec<Option<usize>> = direct.warnings.iter().map(|w| w.line).collect();
    assert!(
        direct_lines.contains(&Some(line_of(doc, "adequate"))),
        "{:?}",
        direct.warnings
    );

    let root = bundle_dir("2312-warn");
    write(&root, "FR-001.md", doc);
    let report = validate_bundle_at(&root, &registry, BundlePosture::Strict);
    let mut bridged: Vec<Option<usize>> = report
        .warnings
        .iter()
        .filter(|f| f.pack.is_none())
        .map(|f| f.line)
        .collect();
    let mut direct_lines = direct_lines;
    bridged.sort();
    direct_lines.sort();
    assert_eq!(bridged, direct_lines);
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2307", "FR-069-AC-24")]
#[test]
fn tc2307_no_located_value_validates_the_empty_record() {
    // Every locator is optional and none located anything: FR-011 yields no
    // record, the empty record is validated, and its absence is reported at
    // the first line.
    let manifest = "name: empty-fixture
version: 0.1.0
artifact_types:
- name: Note
  body_extraction:
    yield_pattern:
      match:
        owner: { from: frontmatter_field, path: [owner], required: false }
  record_validation:
    schema: schemas/NoteFields.json
    locate_absence_at: owner
semantic:
  contract_version: 1.0.0
  package: agent-ix/empty-fixture
  exports: [Note]
  targets: [json-schema, markdown]
";
    let schema = json!({"$schema": DRAFT, "type": "object", "required": ["owner"]}).to_string();
    let registry = inline(manifest, &[("schemas/NoteFields.json", &schema)]);
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let result = validate(&registry, "Note", "---\ntype: Note\n---\n\nbody\n");
    assert_eq!(record_lines(&result), [Some(1)], "{:?}", result.errors);
}

#[trace("TC-2300", "FR-033-AC-18")]
#[test]
fn tc2300_a_duplicated_frontmatter_key_follows_the_value_the_parser_keeps() {
    let doc = "---\nid: first\nname: x\nid: last\n---\n";
    let located = eval_located(
        &parse_document(doc),
        &primitive(json!({"from": "frontmatter_field", "path": ["id"]})),
    );
    assert_eq!(located.len(), 1);
    assert_eq!(located[0].value, json!("last"));
    assert_eq!(located[0].line, Some(4));
}
