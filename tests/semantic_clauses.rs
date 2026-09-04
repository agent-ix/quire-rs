//! FR-071 clause and operation extraction (TC-1622..TC-1626, TC-1629,
//! TC-1648). Plan-003 Task-019. Oracles: quoin `operations.md`,
//! `operations.expected.json`, `operations-cases.json`, and the
//! `config-version` golden span.

use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_rs::semantic::{
    compile_module_schema, extract_clauses, extract_operations, AvailabilityState, BundleIndex,
    SemanticContext, SemanticSeverity,
};
use quire_rs::Registry;
use serde_json::{json, Value};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn mapping(name: &str) -> String {
    fs::read_to_string(
        root()
            .join("tests/fixtures/semantic/quoin/mapping")
            .join(name),
    )
    .unwrap()
}

fn mapping_json(name: &str) -> Value {
    serde_json::from_str(&mapping(name)).unwrap()
}

fn bundle() -> BundleIndex {
    serde_json::from_slice(
        &fs::read(root().join("tests/fixtures/semantic/config-version.bundle.json")).unwrap(),
    )
    .unwrap()
}

fn context(path: &str) -> SemanticContext {
    let registry =
        Registry::load_module(&root().join("tests/fixtures/semantic/quoin/module-ok")).unwrap();
    let module = registry
        .semantic_module("spec-objects-fixture")
        .unwrap()
        .clone();
    SemanticContext::new(module, path, bundle())
        .with_source_identity("ix://agent-ix/config-service/spec")
}

fn gate(model: &str) -> jsonschema::JSONSchema {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://schemas.agent-ix.org/agent-ix/quire-rs/0.1.0/{model}Gate.json"),
        "$ref": format!("https://schemas.agent-ix.org/semantic-core/0.1.0/{model}.json")
    });
    compile_module_schema(
        &schema,
        &|_| None,
        "0.1.0",
        "https://schemas.agent-ix.org/agent-ix/quire-rs/",
    )
    .unwrap()
}

#[trace("TC-1622", "FR-071-AC-1")]
// golden clauses, operations, clauseText, and the config-version span.
#[test]
fn golden_operations_and_spans() {
    let raw = mapping("operations.md");
    let ctx = context("operations.md");
    let clauses = extract_clauses(&raw, &ctx);
    assert_eq!(
        clauses.availability.state,
        AvailabilityState::Available,
        "{:?}",
        clauses.diagnostics
    );
    let expected = mapping_json("operations.expected.json");
    assert_eq!(
        serde_json::to_value(clauses.clauses.as_ref().unwrap()).unwrap(),
        expected["clauses"]
    );
    assert_eq!(
        serde_json::to_value(&clauses.clause_text).unwrap(),
        expected["clauseText"]
    );
    let lines: Vec<&str> = raw.split('\n').collect();
    for clause in clauses.clauses.as_ref().unwrap() {
        let span = clause.source_span.as_ref().unwrap();
        let body = &clauses.clause_text[&clause.clause_id];
        assert_eq!(
            lines[span.start_line], body,
            "clauseText equals the fence body byte-for-byte"
        );
        assert!(lines[span.start_line - 1].starts_with("```ocl"));
        assert_eq!(lines[span.end_line.unwrap() - 1], "```");
        assert_eq!(span.end_column, Some(4));
    }
    let ops = extract_operations(&raw, &ctx, clauses.clauses.as_ref().unwrap());
    assert_eq!(
        ops.availability.state,
        AvailabilityState::Available,
        "{:?}",
        ops.diagnostics
    );
    assert_eq!(
        serde_json::to_value(ops.operations.as_ref().unwrap()).unwrap(),
        expected["operations"]
    );

    let cv = extract_clauses(
        &mapping("config-version.table.md"),
        &context("config-version.table.md"),
    );
    let expected = mapping_json("config-version.expected.json");
    assert_eq!(
        serde_json::to_value(cv.clauses.as_ref().unwrap()).unwrap(),
        expected["clauses"]
    );
    assert_eq!(
        serde_json::to_value(&cv.clause_text).unwrap(),
        expected["clauseText"]
    );
    assert!(cv.diagnostics.is_empty(), "{:?}", cv.diagnostics);
}

fn artifact(invariants: &str, operations: &str) -> String {
    let mut s = String::from("---\nid: FR-901\ntitle: Ops\nobject: entity\n---\n# FR-901\n\n## Properties\n\n| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| id | UUID | 1 | identity |\n\n");
    if !invariants.is_empty() {
        s.push_str("## Invariants\n\n");
        s.push_str(invariants);
        s.push('\n');
    }
    if !operations.is_empty() {
        s.push_str("\n## Operations\n\n");
        s.push_str(operations);
        s.push('\n');
    }
    s
}

fn case_lines(md: &str, needle: &str) -> usize {
    md.split('\n')
        .position(|l| l.starts_with(needle))
        .map(|i| i + 1)
        .unwrap()
}

fn expected_line(md: &str, locus: &str) -> usize {
    match locus {
        "fence" => case_lines(md, "```"),
        "heading" => case_lines(md, "### "),
        "second-clause" | "second-heading" => md
            .split('\n')
            .enumerate()
            .filter(|(_, l)| l.starts_with("### "))
            .nth(1)
            .map(|(i, _)| i + 1)
            .unwrap(),
        "post-line" => case_lines(md, "Post:"),
        "second-occurrence" => case_lines(md, "Clause:"),
        other => panic!("locus {other}"),
    }
}

fn run_case(case: &Value) -> (Vec<quire_rs::semantic::SemanticDiagnostic>, String) {
    let invariants = case["invariants"].as_str().unwrap_or("");
    let operations = case["operations"].as_str().unwrap_or("");
    let md = artifact(invariants, operations);
    let ctx = context("case.md");
    let clauses = extract_clauses(&md, &ctx);
    let mut diagnostics = clauses.diagnostics.clone();
    let ops = extract_operations(&md, &ctx, clauses.clauses.as_deref().unwrap_or(&[]));
    diagnostics.extend(ops.diagnostics);
    (diagnostics, md)
}

fn assert_case(case: &Value) {
    let id = case["id"].as_str().unwrap();
    let (diagnostics, md) = run_case(case);
    for d in case["diagnostics"].as_array().unwrap() {
        let code = d["code"].as_str().unwrap();
        let severity = match d["severity"].as_str().unwrap() {
            "advisory" => SemanticSeverity::Advisory,
            "warning" => SemanticSeverity::Warning,
            _ => SemanticSeverity::Error,
        };
        let line = expected_line(&md, d["locus"].as_str().unwrap());
        assert!(
            diagnostics
                .iter()
                .any(|x| x.code == code && x.severity == severity && x.line == Some(line)),
            "{id}: expected {code} {severity:?} at {line}; got {diagnostics:?}"
        );
    }
}

fn cases(prefix: &str) -> Vec<Value> {
    mapping_json("operations-cases.json")["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["id"].as_str().unwrap().starts_with(prefix))
        .cloned()
        .collect()
}

#[trace("TC-1623", "FR-071-AC-2")]
// language cases; ocl yields no advisory.
#[test]
fn language_cases() {
    let fence = cases("fence-");
    assert_eq!(fence.len(), 5);
    for case in &fence {
        assert_case(case);
    }
    let (diagnostics, _) =
        run_case(&json!({ "id": "ocl", "invariants": "### immutable\n\n```ocl\nx\n```\n" }));
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    // unchecked languages are lossy but available
    let md = artifact("### immutable\n\n```fretish\nx\n```\n", "");
    let out = extract_clauses(&md, &context("f.md"));
    assert_eq!(out.availability.state, AvailabilityState::Available);
    assert!(out.availability.lossy);
}

#[trace("TC-1624", "FR-071-AC-3")]
// structural cases: duplicate id, non-identifier, inline+external, ownerless
// fence, bodiless heading, two fences, unterminated, external-only.
#[test]
fn structural_cases() {
    for id in [
        "duplicate-clause-id",
        "clause-id-not-identifier",
        "inline-and-external",
    ] {
        let case = cases(id)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{id}"));
        assert_case(&case);
    }
    let ctx = context("s.md");
    let ownerless = artifact("```ocl\nx\n```\n", "");
    let out = extract_clauses(&ownerless, &ctx);
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.code == "semantic.clause-without-id"
                && d.line == Some(case_lines(&ownerless, "```"))),
        "{:?}",
        out.diagnostics
    );
    let bodiless = artifact("### immutable\n\nprose only\n", "");
    let out = extract_clauses(&bodiless, &ctx);
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.code == "semantic.clause-missing-body"
                && d.line == Some(case_lines(&bodiless, "### "))),
        "{:?}",
        out.diagnostics
    );
    let two = artifact("### immutable\n\n```ocl\na\n```\n\n```ocl\nb\n```\n", "");
    let out = extract_clauses(&two, &ctx);
    let second = two
        .split('\n')
        .enumerate()
        .filter(|(_, l)| l.starts_with("```ocl"))
        .nth(1)
        .unwrap()
        .0
        + 1;
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.code == "semantic.clause-multiple-bodies" && d.line == Some(second)),
        "{:?}",
        out.diagnostics
    );
    let open = artifact("### immutable\n\n```ocl\nnever closed\n", "");
    let out = extract_clauses(&open, &ctx);
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.code == "semantic.clause-fence-unterminated"),
        "{:?}",
        out.diagnostics
    );
    let external = artifact("### immutable\n\nClause: ./clauses.md#immutable\n", "");
    let out = extract_clauses(&external, &ctx);
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.code == "semantic.clause-external-unsupported"
                && d.severity == SemanticSeverity::Advisory),
        "{:?}",
        out.diagnostics
    );
    assert_eq!(
        out.clauses.as_ref().map(Vec::len),
        Some(0),
        "no entry, file never read"
    );
    // A deeper heading belongs to its clause and does not split it.
    let nested = artifact("### immutable\n\n#### note\n\n```ocl\nx\n```\n", "");
    let out = extract_clauses(&nested, &ctx);
    assert_eq!(
        out.clauses.as_ref().map(Vec::len),
        Some(1),
        "{:?}",
        out.diagnostics
    );
}

#[trace("TC-1625", "FR-071-AC-4")]
// operations: dangling post, duplicate operation, non-identifier heading,
// Pre lists, no-table params.
#[test]
fn operation_cases() {
    for id in ["dangling-post", "duplicate-operation"] {
        let case = cases(id)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{id}"));
        assert_case(&case);
    }
    let ctx = context("o.md");
    let md = artifact(
        "### a\n\n```ocl\nx\n```\n\n### b\n\n```ocl\ny\n```\n",
        "### not-ok\n\nReturns: String[1]\n\n### fine\n\nPre: a, b\n",
    );
    let clauses = extract_clauses(&md, &ctx);
    let ops = extract_operations(&md, &ctx, clauses.clauses.as_ref().unwrap());
    assert!(
        ops.diagnostics
            .iter()
            .any(|d| d.code == "semantic.operation-name-not-identifier"
                && d.line == Some(case_lines(&md, "### not-ok"))),
        "{:?}",
        ops.diagnostics
    );
    assert_eq!(
        ops.availability.state,
        AvailabilityState::Unavailable,
        "one erroring operation makes the kind unavailable"
    );
    let md = artifact(
        "### a\n\n```ocl\nx\n```\n\n### b\n\n```ocl\ny\n```\n",
        "### fine\n\nPre: a, b\nPost: b\n",
    );
    let clauses = extract_clauses(&md, &ctx);
    let ops = extract_operations(&md, &ctx, clauses.clauses.as_ref().unwrap());
    let op = &ops
        .operations
        .as_ref()
        .unwrap_or_else(|| panic!("{:?}", ops.diagnostics))[0];
    assert_eq!(op.params.len(), 0);
    assert_eq!(
        op.pre
            .iter()
            .map(|c| c.clause_id.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(op.pre[0].language, "ocl");
    assert!(op.pre[0].source_span.is_none());
    assert!(op.returns.is_none());
    // A unit on Returns is refused.
    let md = artifact("", "### op\n\nReturns: Duration [ms][1]\n");
    let ops = extract_operations(&md, &ctx, &[]);
    assert!(
        ops.diagnostics
            .iter()
            .any(|d| d.code == "agent-ix.semantic-core.UNIT_ON_RETURNS"),
        "{:?}",
        ops.diagnostics
    );
}

#[trace("TC-1626", "FR-071-AC-5")]
// produced entries validate; absent sections are not_applicable; one
// erroring entry makes the kind unavailable with no partial array.
#[test]
fn validation_and_states() {
    let raw = mapping("operations.md");
    let ctx = context("operations.md");
    let clauses = extract_clauses(&raw, &ctx);
    let clause_gate = gate("ClauseRef");
    for c in clauses.clauses.as_ref().unwrap() {
        assert!(
            clause_gate.is_valid(&serde_json::to_value(c).unwrap()),
            "{c:?}"
        );
    }
    let ops = extract_operations(&raw, &ctx, clauses.clauses.as_ref().unwrap());
    let op_gate = gate("OperationDecl");
    for o in ops.operations.as_ref().unwrap() {
        assert!(op_gate.is_valid(&serde_json::to_value(o).unwrap()), "{o:?}");
    }
    let none = artifact("", "");
    assert_eq!(
        extract_clauses(&none, &ctx).availability.state,
        AvailabilityState::NotApplicable
    );
    assert_eq!(
        extract_operations(&none, &ctx, &[]).availability.state,
        AvailabilityState::NotApplicable
    );
    let mixed = artifact(
        "### ok\n\n```ocl\nx\n```\n\n### bad\n\n```tla\ny\n```\n",
        "",
    );
    let out = extract_clauses(&mixed, &ctx);
    assert_eq!(out.availability.state, AvailabilityState::Unavailable);
    assert!(out
        .availability
        .reason
        .as_ref()
        .unwrap()
        .starts_with("entry-errors"));
    assert!(out.clauses.is_none());
    assert!(out.clause_text.is_empty());
}

#[trace("TC-1648", "FR-071-AC-7")]
// defaulted vs supplied source identity.
#[test]
fn source_identity_default() {
    let raw = mapping("operations.md");
    let registry =
        Registry::load_module(&root().join("tests/fixtures/semantic/quoin/module-ok")).unwrap();
    let module = registry
        .semantic_module("spec-objects-fixture")
        .unwrap()
        .clone();
    let ctx = SemanticContext::new(module.clone(), "operations.md", bundle())
        .with_scope("config-service");
    let out = extract_clauses(&raw, &ctx);
    let span = out.clauses.as_ref().unwrap()[0]
        .source_span
        .as_ref()
        .unwrap();
    assert_eq!(span.source_identity, "ix://local/config-service/spec");
    assert_eq!(
        out.diagnostics
            .iter()
            .filter(|d| d.code == "semantic.source-identity-defaulted")
            .count(),
        1
    );
    let ctx = SemanticContext::new(module, "operations.md", bundle())
        .with_source_identity("ix://agent-ix/config-service/spec");
    let out = extract_clauses(&raw, &ctx);
    assert_eq!(
        out.clauses.as_ref().unwrap()[0]
            .source_span
            .as_ref()
            .unwrap()
            .source_identity,
        "ix://agent-ix/config-service/spec"
    );
    assert!(out
        .diagnostics
        .iter()
        .all(|d| d.code != "semantic.source-identity-defaulted"));
}

fn body_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-zA-Z0-9 (),.:=<>@'-]{0,40}",
        "[a-zA-Z0-9 ]{0,10}`[a-zA-Z0-9 ]{0,10}",
        "[a-zA-Z0-9 ]{0,10}\r\n[a-zA-Z0-9 ]{0,10}",
        "[a-zA-Z0-9 ]{0,5}\n``\n[a-zA-Z0-9 ]{0,5}",
        "\\PC{0,30}",
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[trace("TC-1629", "FR-071-AC-6")]
    // generated bodies round-trip byte-for-byte with spans on the fences.
    #[test]
    fn fence_bodies_round_trip(body in body_strategy(), tilde in any::<bool>(), extra in 0usize..3) {
        // A body that could close the fence is escaped by using a longer fence.
        let fence_char = if tilde { "~" } else { "`" };
        let fence = fence_char.repeat(3 + extra);
        let md = format!("---\nid: X\nobject: entity\n---\n## Invariants\n\n### inv\n\n{fence}ocl\n{body}\n{fence}\n");
        let ctx = context("p.md");
        let out = extract_clauses(&md, &ctx);
        // The body may itself contain a closing run; then the fence closes
        // early and the rest is outside — still a well-formed outcome.
        if let Some(clauses) = &out.clauses {
            let clause = &clauses[0];
            let span = clause.source_span.as_ref().unwrap();
            let lines: Vec<&str> = md.split('\n').collect();
            prop_assert!(lines[span.start_line - 1].starts_with(&fence));
            let close = span.end_line.unwrap();
            prop_assert!(lines[close - 1].trim_end_matches('\r').chars().all(|c| c.to_string() == fence_char));
            let expected_body = lines[span.start_line..close - 1].join("\n");
            prop_assert_eq!(&out.clause_text["inv"], &expected_body);
            prop_assert_eq!(span.end_column, Some(lines[close - 1].trim_end_matches('\r').len() + 1));
        } else {
            prop_assert!(out.diagnostics.iter().any(|d| d.is_error()), "{:?}", out.diagnostics);
        }
    }
}
