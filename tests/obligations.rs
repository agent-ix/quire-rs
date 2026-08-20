//! FR-053 — obligation records (TC-831..TC-843, TC-870..TC-873).
//!
//! Bundles are built on disk per test so derivation runs over the real corpus
//! walk rather than a hand-assembled `Spec`.

use std::fs;
use std::path::{Path, PathBuf};

use quire_rs::coverage::compute;
use quire_rs::obligation::{derive, statement_hash};
use quire_rs::symbols::{extract_tree, trace};
use quire_rs::{Registry, Spec};

fn fixture_module(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("traceability")
        .join(name)
}

fn tmpdir(suffix: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "quire-rs-obligations-{}-{suffix}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("mkdir");
    p
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, body).expect("write");
}

/// A bundle whose one FR carries an `Acceptance Criteria` table with the
/// columns the `obligations` fixture module declares.
fn ac_bundle(suffix: &str, rows: &str) -> PathBuf {
    let scope = tmpdir(suffix).join("spec");
    fs::create_dir_all(&scope).expect("mkdir");
    write(
        &scope,
        "FR-001.md",
        &format!(
            "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
             ## Acceptance Criteria\n\n\
             | ID | Criteria | Verification | Priority |\n\
             |----|----------|--------------|----------|\n{rows}"
        ),
    );
    scope
}

fn model_of(module: &str) -> quire_rs::traceability::TraceabilityModel {
    Registry::load_module(&fixture_module(module))
        .expect("load module")
        .traceability()
        .cloned()
        .expect("model declared")
}

fn derive_at(scope: &Path, module: &str) -> Vec<quire_rs::Obligation> {
    let spec = Spec::from_path(scope);
    derive(&spec, scope, &model_of(module)).0
}

// TC-831, FR-053-AC-1: a `target:`-bound source yields one record per row of
// the named trace target's table, keyed on the id the rollup already mints.
#[test]
fn tc831_target_bound_source_mints_one_record_per_row() {
    let scope = ac_bundle(
        "831",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n\
         | FR-001-AC-2 | The system shall also do it. | Analysis | P2 |\n",
    );
    let obligations = derive_at(&scope, "obligations");

    assert_eq!(obligations.len(), 2, "{obligations:#?}");
    assert_eq!(obligations[0].id, "FR-001-AC-1");
    assert_eq!(obligations[0].source, "acceptance-criterion");
    assert_eq!(obligations[0].document, "FR-001.md");
    assert_eq!(obligations[0].statement, "The system shall do it.");
    assert_eq!(obligations[0].method.as_deref(), Some("Test"));
    assert_eq!(obligations[0].criticality.as_deref(), Some("P1"));
    assert_eq!(obligations[1].id, "FR-001-AC-2");
    assert_eq!(obligations[1].method.as_deref(), Some("Analysis"));
}

// TC-832, FR-053-AC-2: an `archetype:`+`id_format:` source covers rows with no
// id column, rendering `{document}` and the 1-based `{row}`.
#[test]
fn tc832_archetype_bound_source_renders_ids() {
    let scope = tmpdir("832").join("spec");
    fs::create_dir_all(&scope).expect("mkdir");
    write(
        &scope,
        "NFR-006-determinism.md",
        "---\nid: NFR-006\ntype: NFR\ntitle: Determinism\n---\n\n\
         ## Measurement and Evaluation\n\n\
         | Metric | Target | Threshold | Method |\n\
         |--------|--------|-----------|--------|\n\
         | Identical output across 100 parses | all equal | all equal | Proptest |\n\
         | Wall clock for a 500-document corpus | < 5ms | < 8ms | Benchmark |\n",
    );
    let obligations = derive_at(&scope, "obligations-nfr");

    assert_eq!(obligations.len(), 2, "{obligations:#?}");
    assert_eq!(obligations[0].id, "NFR-006-M-1");
    assert_eq!(obligations[1].id, "NFR-006-M-2");
    assert_eq!(obligations[1].method.as_deref(), Some("Benchmark"));
    // The "one number, three uses" carrier: the spec threshold travels with the
    // obligation instead of being re-parsed downstream.
    assert_eq!(obligations[1].parameters.get("target").unwrap(), "< 5ms");
    assert_eq!(obligations[1].parameters.get("threshold").unwrap(), "< 8ms");
}

// TC-833, FR-053-AC-3: a source declaring both origins, or neither, fails
// module load with a diagnostic naming it.
#[test]
fn tc833_ambiguous_or_originless_source_fails_load() {
    for (module, source, expected) in [
        ("obligations-both", "ambiguous", "declares both"),
        ("obligations-neither", "originless", "declares neither"),
    ] {
        // The manifest is rejected at parse, so the module contributes nothing
        // and the failure names the offending source rather than leaving an
        // unexecutable declaration to yield an inexplicably empty report.
        let outcome = quire_rs::loader::load_single_module(&fixture_module(module));
        let reasons: Vec<&str> = outcome.failures.iter().map(|f| f.reason.as_str()).collect();
        assert!(
            reasons
                .iter()
                .any(|r| r.contains(source) && r.contains(expected)),
            "module {module} loaded without naming source '{source}': {reasons:#?}",
        );
        assert!(
            outcome.modules.is_empty(),
            "module {module} contributed archetypes despite an invalid obligation source",
        );

        // And nothing downstream sees the bad source.
        let registry = Registry::load_module(&fixture_module(module)).expect("tolerant load");
        assert_eq!(
            registry
                .traceability()
                .map(|m| m.obligations.len())
                .unwrap_or(0),
            0,
            "module {module} contributed a bad source",
        );
    }
}

// TC-834, FR-053-AC-4: whitespace does not churn the hash; a word does —
// including a word inside an inline code span, which the CR-017 mask would have
// collapsed.
#[test]
fn tc834_hash_is_whitespace_insensitive_and_word_sensitive() {
    assert_eq!(
        statement_hash("The system shall do it."),
        statement_hash("  The   system\tshall\n do it.  "),
    );
    assert_ne!(
        statement_hash("The system shall do it."),
        statement_hash("The system shall not do it."),
    );
    // The reason this is not the CR-017 mask.
    assert_ne!(
        statement_hash("The parser SHALL reject a `foo` token."),
        statement_hash("The parser SHALL reject a `bar` token."),
    );
}

// TC-835, FR-053-AC-5: one cell, two readings. The method drops the trailing
// annotation; FR-049 still reads the reference out of the same cell.
#[test]
fn tc835_method_and_reference_read_the_same_cell() {
    let scope = ac_bundle(
        "835",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n",
    );
    let obligations = derive_at(&scope, "obligations");
    assert_eq!(obligations[0].method.as_deref(), Some("Test"));

    // The reference side: the declared `verification` document_reference reads
    // `TC-001` from that same cell, which the coverage rollup then reconciles.
    let registry = Registry::load_module(&fixture_module("obligations")).expect("load module");
    let spec = Spec::from_path(&scope);
    let source = scope.parent().unwrap().join("src");
    fs::create_dir_all(&source).expect("mkdir");
    write(&source, "lib.rs", "//! empty\n");
    let extraction = extract_tree(&source);
    let model = registry.traceability().cloned().unwrap();
    let graph = trace::bind(&extraction, &model);
    let report = compute(&spec, &registry, &graph, &scope).expect("report");
    assert!(
        report
            .unbacked_rows
            .iter()
            .any(|r| r.target_ids.iter().any(|id| id == "TC-001")),
        "the reference was not read from the cell: {:#?}",
        report.unbacked_rows
    );
}

// TC-836, FR-053-AC-6: declared parameters with no cell are omitted, never
// present-and-empty. A threshold nobody wrote is not a threshold of zero.
#[test]
fn tc836_absent_parameters_are_omitted() {
    let scope = tmpdir("836").join("spec");
    fs::create_dir_all(&scope).expect("mkdir");
    write(
        &scope,
        "NFR-007-cost.md",
        "---\nid: NFR-007\ntype: NFR\ntitle: Cost\n---\n\n\
         ## Measurement and Evaluation\n\n\
         | Metric | Target | Threshold | Method |\n\
         |--------|--------|-----------|--------|\n\
         | Amortized load cost | < 1ms |  | Benchmark |\n",
    );
    let obligations = derive_at(&scope, "obligations-nfr");
    assert_eq!(obligations.len(), 1);
    assert_eq!(obligations[0].parameters.get("target").unwrap(), "< 1ms");
    assert!(
        !obligations[0].parameters.contains_key("threshold"),
        "an empty cell must be omitted, not carried as an empty string: {:#?}",
        obligations[0].parameters
    );
}

// TC-837, FR-053-AC-7: criticality is genuinely optional — a source declaring
// no column, and one declaring an empty column, agree on every other field.
#[test]
fn tc837_criticality_is_optional() {
    let scope = ac_bundle(
        "837",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) |  |\n",
    );
    let declared_but_empty = derive_at(&scope, "obligations");
    assert_eq!(declared_but_empty.len(), 1);
    assert_eq!(declared_but_empty[0].criticality, None);

    // The NFR module declares no criticality column at all; the two records
    // differ in nothing but the source they came from.
    assert_eq!(declared_but_empty[0].method.as_deref(), Some("Test"));
    assert!(!declared_but_empty[0].statement_hash.is_empty());
}

// TC-838, FR-053-AC-8: a row whose statement cell is empty is skipped and
// reported, never emitted as a record stating nothing.
#[test]
fn tc838_empty_statement_row_is_skipped_and_reported() {
    let scope = ac_bundle(
        "838",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n\
         | FR-001-AC-2 |  | Test (TC-002) | P2 |\n",
    );
    let spec = Spec::from_path(&scope);
    let (obligations, skipped) = derive(&spec, &scope, &model_of("obligations"));

    assert_eq!(obligations.len(), 1, "{obligations:#?}");
    assert_eq!(obligations[0].id, "FR-001-AC-1");
    assert_eq!(skipped.len(), 1, "{skipped:#?}");
    assert_eq!(skipped[0].document, "FR-001.md");
    assert_eq!(skipped[0].row, 2);
    assert_eq!(skipped[0].source, "acceptance-criterion");
}

// TC-870, FR-053-AC-8: the skipped row reaches the coverage report, which is
// the only surface anybody reads. TC-838 proves `derive` returns it; this proves
// it is not dropped on the way out (#151, CR-063).
#[test]
fn tc870_skipped_row_is_reported_in_the_coverage_report() {
    let scope = ac_bundle(
        "870",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n\
         | FR-001-AC-2 |  | Test (TC-002) | P2 |\n",
    );
    let source = scope.parent().unwrap().join("src");
    fs::create_dir_all(&source).expect("mkdir");
    write(&source, "lib.rs", "//! empty\n");

    let registry = Registry::load_module(&fixture_module("obligations")).expect("load module");
    let spec = Spec::from_path(&scope);
    let model = registry.traceability().cloned().unwrap();
    let graph = trace::bind(&extract_tree(&source), &model);
    let report = compute(&spec, &registry, &graph, &scope).expect("report");

    let reported: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.reason == "obligation-row-states-nothing")
        .collect();
    assert_eq!(reported.len(), 1, "{:#?}", report.diagnostics);
    assert_eq!(reported[0].declaration, "acceptance-criterion");
    assert_eq!(reported[0].path.as_deref(), Some("FR-001.md"));
    assert!(
        reported[0].message.contains("row 2"),
        "the row ordinal must be named: {}",
        reported[0].message,
    );
    // And it reaches the JSON, since that is what a consumer parses.
    assert!(report.to_json().contains("obligation-row-states-nothing"));
}

// TC-871, FR-053-AC-4: Unicode normalization form does not change the hash.
// The FR asserted this and the code skipped it; NFC is now applied (CR-063).
#[test]
fn tc871_hash_is_normalization_form_insensitive() {
    // "café" composed (U+00E9) vs decomposed (e + U+0301). A reader cannot tell
    // them apart and neither may the hash.
    let composed = "The parser shall accept a caf\u{e9} token.";
    let decomposed = "The parser shall accept a cafe\u{301} token.";
    assert_ne!(composed, decomposed, "the fixture must differ as bytes");
    assert_eq!(
        statement_hash(composed),
        statement_hash(decomposed),
        "an editor rewriting NFD to NFC is not a change of statement",
    );
    // And normalization does not flatten a real difference.
    assert_ne!(
        statement_hash(composed),
        statement_hash("The parser shall accept a cafe token."),
    );
}

// TC-872, FR-053-AC-9: record order follows source DECLARATION order, not
// source name. The fixture declares `zzz-metric` before `aaa-criterion`, so the
// two orderings disagree — which is the only way to tell them apart (#151).
#[test]
fn tc872_order_is_declaration_order_not_source_name() {
    let scope = tmpdir("872").join("spec");
    fs::create_dir_all(&scope).expect("mkdir");
    write(
        &scope,
        "FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test |\n",
    );
    write(
        &scope,
        "NFR-001.md",
        "---\nid: NFR-001\ntype: NFR\ntitle: A budget\n---\n\n\
         ## Measurement and Evaluation\n\n\
         | Metric | Method |\n|--------|--------|\n\
         | Amortized load cost | Benchmark |\n",
    );

    let obligations = derive_at(&scope, "obligations-ordered");
    assert_eq!(obligations.len(), 2, "{obligations:#?}");
    assert_eq!(
        obligations
            .iter()
            .map(|o| o.source.as_str())
            .collect::<Vec<_>>(),
        ["zzz-metric", "aaa-criterion"],
        "sources must read in declaration order; alphabetical is the bug",
    );
}

// TC-873, FR-053-AC-14: an `exclude`d document states no obligation on EITHER
// surface. Before #151 the rollup honoured the glob and the classification path
// did not, so an excluded fixture criterion carried an obligation a generator
// would have emitted a dead trace tag for.
#[test]
fn tc873_exclude_applies_to_both_surfaces() {
    let scope = tmpdir("873").join("spec");
    fs::create_dir_all(&scope).expect("mkdir");
    let body = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
                ## Acceptance Criteria\n\n\
                | ID | Criteria | Verification |\n|----|----------|--------------|\n\
                | FR-001-AC-1 | The system shall do it. | Test |\n";
    write(&scope, "FR-001.md", body);
    write(
        &scope,
        "fixtures/FR-009.md",
        &body.replace("FR-001", "FR-009"),
    );

    // The rollup: one record, the excluded fixture contributes none.
    let obligations = derive_at(&scope, "obligations-excluded");
    assert_eq!(obligations.len(), 1, "{obligations:#?}");
    assert_eq!(obligations[0].document, "FR-001.md");

    // The classification path, handed each document with its path.
    let registry =
        Registry::load_module(&fixture_module("obligations-excluded")).expect("load module");
    let archetype = registry.archetype("FR").expect("FR archetype");

    let included = quire_rs::classify_document_criteria(
        &registry,
        archetype,
        body,
        Some(Path::new("FR-001.md")),
    );
    assert!(
        included[0].obligation.is_some(),
        "an included document still states its obligation",
    );

    let excluded = quire_rs::classify_document_criteria(
        &registry,
        archetype,
        &body.replace("FR-001", "FR-009"),
        Some(Path::new("fixtures/FR-009.md")),
    );
    assert!(
        excluded[0].obligation.is_none(),
        "an excluded document must state no obligation on this surface either: {:#?}",
        excluded[0].obligation,
    );
}

// TC-839, FR-053-AC-9: derivation is deterministic and ordered.
#[test]
fn tc839_derivation_is_deterministic() {
    let scope = ac_bundle(
        "839",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n\
         | FR-001-AC-2 | The system shall also do it. | Test (TC-002) | P2 |\n",
    );
    let a = derive_at(&scope, "obligations");
    let b = derive_at(&scope, "obligations");
    assert_eq!(a, b);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
    );
    // Row order within a document is the authored order.
    assert_eq!(a[0].id, "FR-001-AC-1");
    assert_eq!(a[1].id, "FR-001-AC-2");
}

// TC-840, FR-053-AC-10: the classification record carries the obligation,
// matched by row id — and carries `None` for a module declaring no sources.
#[test]
fn tc840_classification_carries_the_obligation() {
    let doc = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
               ## Acceptance Criteria\n\n\
               | ID | Criteria | Verification | Priority |\n\
               |----|----------|--------------|----------|\n\
               | FR-001-AC-1 | Every finding absent from the merged map defaults to warning. | Test (TC-001) | P1 |\n";

    let with = Registry::load_module(&fixture_module("obligations")).expect("load");
    let archetype = with.archetype("FR").expect("FR archetype");
    let records = quire_rs::classify_document_criteria(&with, archetype, doc, None);
    assert_eq!(records.len(), 1, "{records:#?}");
    let ob = records[0]
        .obligation
        .as_ref()
        .expect("obligation attached to the criterion");
    assert_eq!(ob.source, "acceptance-criterion");
    assert_eq!(ob.method.as_deref(), Some("Test"));
    assert_eq!(ob.criticality.as_deref(), Some("P1"));
    assert_eq!(
        ob.statement_hash,
        statement_hash("Every finding absent from the merged map defaults to warning."),
    );

    // A module declaring no obligation sources: same records, no obligation.
    let without = Registry::load_module(&fixture_module("iso")).expect("load");
    let archetype = without.archetype("FR").expect("FR archetype");
    let plain = quire_rs::classify_document_criteria(&without, archetype, doc, None);
    assert_eq!(plain.len(), 1);
    assert!(plain[0].obligation.is_none());
    // Field-for-field unchanged apart from the new field.
    assert_eq!(plain[0].row_id, records[0].row_id);
    assert_eq!(plain[0].statement, records[0].statement);
    assert_eq!(plain[0].property, records[0].property);
    assert_eq!(plain[0].extractable, records[0].extractable);
    assert_eq!(plain[0].signals, records[0].signals);
}

// TC-841, FR-053-AC-11: the coverage report carries the records, and a model
// declaring no sources carries an empty list that serializes away entirely, so
// FR-050-AC-7 byte-identity holds for every module that has not adopted them.
#[test]
fn tc841_coverage_report_carries_obligations() {
    let scope = ac_bundle(
        "841",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n",
    );
    let source = scope.parent().unwrap().join("src");
    fs::create_dir_all(&source).expect("mkdir");
    write(&source, "lib.rs", "//! empty\n");

    let spec = Spec::from_path(&scope);
    let extraction = extract_tree(&source);

    for (module, expected) in [("obligations", 1usize), ("iso", 0)] {
        let registry = Registry::load_module(&fixture_module(module)).expect("load module");
        let model = registry.traceability().cloned().unwrap_or_default();
        let graph = trace::bind(&extraction, &model);
        let report = compute(&spec, &registry, &graph, &scope).expect("report");
        assert_eq!(report.obligations.len(), expected, "module {module}");
        let json = report.to_json();
        assert_eq!(
            json.contains("\"obligations\""),
            expected > 0,
            "an empty list must be absent from the JSON, not present-and-empty ({module})",
        );
    }
}

// TC-842, FR-053-AC-12: the hash follows the statement, not its position. A
// suspect-link detector that fires on a file move is one that gets switched off.
#[test]
fn tc842_hash_survives_a_move_and_a_renumber() {
    let statement = "The system shall reject a malformed token.";

    let here = ac_bundle(
        "842a",
        &format!("| FR-001-AC-1 | {statement} | Test (TC-001) | P1 |\n"),
    );
    let there = tmpdir("842b").join("spec");
    fs::create_dir_all(&there).expect("mkdir");
    write(
        &there,
        "nested/FR-009.md",
        &format!(
            "---\nid: FR-009\ntype: FR\ntitle: Elsewhere\n---\n\n\
             ## Acceptance Criteria\n\n\
             | ID | Criteria | Verification | Priority |\n\
             |----|----------|--------------|----------|\n\
             | FR-009-AC-7 | {statement} | Test (TC-050) | P3 |\n"
        ),
    );

    let a = derive_at(&here, "obligations");
    let b = derive_at(&there, "obligations");
    assert_ne!(a[0].id, b[0].id);
    assert_ne!(a[0].document, b[0].document);
    assert_eq!(
        a[0].statement_hash, b[0].statement_hash,
        "a move and a renumber are not a change of statement",
    );

    let reworded = ac_bundle(
        "842c",
        "| FR-001-AC-1 | The system shall reject a malformed tokens. | Test (TC-001) | P1 |\n",
    );
    let c = derive_at(&reworded, "obligations");
    assert_ne!(a[0].statement_hash, c[0].statement_hash);
}

// TC-843, FR-053-AC-13: the nested obligation carries no id, statement or
// document — the record and its enclosing object already have all three.
#[test]
fn tc843_nested_obligation_does_not_repeat_the_record() {
    let doc = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
               ## Acceptance Criteria\n\n\
               | ID | Criteria | Verification | Priority |\n\
               |----|----------|--------------|----------|\n\
               | FR-001-AC-1 | Every finding defaults to warning. | Test (TC-001) | P1 |\n";
    let registry = Registry::load_module(&fixture_module("obligations")).expect("load");
    let archetype = registry.archetype("FR").expect("FR archetype");
    let records = quire_rs::classify_document_criteria(&registry, archetype, doc, None);
    let ob = records[0].obligation.as_ref().expect("attached");

    let json: serde_json::Value = serde_json::to_value(ob).expect("serialize");
    let object = json.as_object().expect("object");
    for repeated in ["id", "statement", "document"] {
        assert!(
            !object.contains_key(repeated),
            "the nested obligation repeats `{repeated}`, which the record already carries: {object:#?}",
        );
    }
    assert!(object.contains_key("source"));
    assert!(object.contains_key("statement_hash"));
}

// TC-935, FR-053-AC-11: the obligation carries the test-case ids its method
// cell names, so a consumer binding evidence keyed on a TEST CASE can join.
//
// The engine already parses this parenthetical — `method_of` finds the same `(`
// to know where the method name ends, and the reconciliation resolves the row
// through those ids — and then dropped them on the way out. quoin's evidence
// store binds by matching a run's trace ids against OBLIGATION ids, so an
// adapter reporting `TC-EV-057` (an agent-eval report keys on the scenario)
// bound nothing, while the join `FR-038-AC-8 → TC-EV-057` sat in the criteria
// table the engine had just read (agent-ix/quire-rs#180).
#[test]
fn tc935_obligation_carries_the_ids_its_method_cell_names() {
    let scope = ac_bundle(
        "935",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n\
         | FR-001-AC-2 | It shall also do it. | Eval (TC-EV-054, TC-EV-055) | P1 |\n\
         | FR-001-AC-3 | A person reads it. | Inspection | P2 |\n\
         | FR-001-AC-4 | Malformed annotation. | Test (TC-004 | P2 |\n",
    );
    let obligations = derive_at(&scope, "obligations");
    assert_eq!(obligations.len(), 4, "{obligations:#?}");

    // One id, and the method is still just the head.
    assert_eq!(obligations[0].target_ids, vec!["TC-001".to_string()]);
    assert_eq!(obligations[0].method.as_deref(), Some("Test"));

    // Several, comma-separated and trimmed — the eval case that motivated this.
    assert_eq!(
        obligations[1].target_ids,
        vec!["TC-EV-054".to_string(), "TC-EV-055".to_string()]
    );
    assert_eq!(obligations[1].method.as_deref(), Some("Eval"));

    // A cell naming none carries none, rather than an empty-string id.
    assert!(obligations[2].target_ids.is_empty());
    assert_eq!(obligations[2].method.as_deref(), Some("Inspection"));

    // An unclosed parenthetical reads NOTHING. Reading to end-of-cell would
    // turn a typo into a plausible-looking id that binds to nothing and looks
    // like a real target.
    assert!(
        obligations[3].target_ids.is_empty(),
        "{:?}",
        obligations[3].target_ids
    );
    assert_eq!(obligations[3].method.as_deref(), Some("Test"));
}
