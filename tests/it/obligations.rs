//! FR-053 — obligation records (TC-831..TC-843, TC-870..TC-873).
//!
//! Bundles are built on disk per test so derivation runs over the real corpus
//! walk rather than a hand-assembled `Spec`.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::coverage::compute;
use quire_rs::obligation::derive;
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

#[trace("FR-053-AC-1")]
// a `target:`-bound source yields one record per row of
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

#[trace("FR-053-AC-2")]
// an `archetype:`+`id_format:` source covers rows with no
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

#[trace("FR-053-AC-3")]
// a source declaring both origins, or neither, fails
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
        let error = Registry::load_module(&fixture_module(module))
            .expect_err("a bad obligation source fails the load");
        assert!(error.to_string().contains(source), "{error}");
    }
}

#[trace("FR-053-AC-5")]
// one cell, two readings. The method drops the trailing
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

#[trace("FR-053-AC-6")]
// declared parameters with no cell are omitted, never
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

#[trace("FR-053-AC-7")]
// criticality is genuinely optional — a source declaring
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
}

#[trace("FR-053-AC-8")]
// a row whose statement cell is empty is skipped and
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

#[trace("FR-053-AC-8")]
// the skipped row reaches the coverage report, which is
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

#[trace("FR-053-AC-9")]
// record order follows source DECLARATION order, not
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

#[trace("FR-053-AC-14")]
// an `exclude`d document states no obligation on EITHER
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

#[trace("FR-053-AC-9")]
// derivation is deterministic and ordered.
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

#[trace("FR-053-AC-10")]
// the classification record carries the obligation,
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

#[trace("FR-053-AC-11")]
// the coverage report carries the records, and a model
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

#[trace("FR-053-AC-13")]
// the nested obligation carries no id, statement or
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
}

#[trace("FR-053-AC-11")]
// the obligation carries the test-case ids its method
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

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryCases {
    issue_ref: String,
    cases: Vec<InventoryCase>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryCase {
    name: String,
    documents: std::collections::BTreeMap<String, String>,
    ids: Vec<String>,
    causes: Vec<String>,
}

fn selected_at(scope: &Path) -> quire_rs::obligation::inventory::SelectedInventory {
    quire_rs::obligation::inventory(
        scope,
        &model_of("obligations"),
        &["acceptance-criterion"],
        Default::default(),
    )
}

#[trace("FR-053-AC-16", "FR-053-AC-18", "FR-053-AC-19")]
#[test]
fn selected_inventory_declarative_controls() {
    use quire_rs::obligation::inventory::InventoryTerminal;
    let fixtures: InventoryCases = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/selected-inventory.json"
    ))
    .expect("inventory case data");
    assert_eq!(
        fixtures.issue_ref,
        "https://linear.app/agent-ix/issue/PLAT-1165"
    );
    let mut names = std::collections::BTreeSet::new();
    for case in fixtures.cases {
        assert!(
            names.insert(case.name.clone()),
            "duplicate case {}",
            case.name
        );
        let root = tempfile::tempdir().expect("case directory");
        for (path, text) in case.documents {
            write(root.path(), &path, &text);
        }
        let inventory = selected_at(root.path());
        let ids: Vec<_> = inventory
            .entries()
            .map(|entry| entry.obligation().id.clone())
            .collect();
        assert_eq!(ids, case.ids, "{} entries", case.name);
        let causes: Vec<_> = inventory
            .causes()
            .iter()
            .map(|cause| format!("{:?}", cause.code()))
            .collect();
        assert_eq!(causes, case.causes, "{} causes", case.name);
        assert_eq!(inventory.len(), ids.len(), "{} observed count", case.name);
        assert_eq!(
            inventory.terminal(),
            if causes.is_empty() {
                InventoryTerminal::Complete
            } else {
                InventoryTerminal::Incomplete
            },
            "{} terminal",
            case.name
        );
        for cause in inventory.causes() {
            assert_eq!(
                cause.document(),
                Some(
                    root.path()
                        .canonicalize()
                        .unwrap()
                        .join("FR-001.md")
                        .as_path()
                ),
                "{} cause locus",
                case.name
            );
        }
    }
}

#[trace("FR-053-AC-15")]
#[test]
fn selected_inventory_validates_the_real_request() {
    use quire_rs::obligation::inventory::{
        InventoryCauseCode, InventoryCauseDetail, InventoryTerminal, SelectionFailure,
    };
    let root = tempfile::tempdir().unwrap();
    let model = model_of("obligations");
    for (names, reason) in [
        (vec![], SelectionFailure::Empty),
        (vec!["unknown"], SelectionFailure::UnknownSource),
        (
            vec!["acceptance-criterion", "acceptance-criterion"],
            SelectionFailure::DuplicateSource,
        ),
    ] {
        let result =
            quire_rs::obligation::inventory(root.path(), &model, &names, Default::default());
        assert_eq!(result.terminal(), InventoryTerminal::Incomplete);
        assert!(result.selection().is_none());
        assert_eq!(result.causes().len(), 1);
        assert_eq!(
            result.causes()[0].code(),
            InventoryCauseCode::InvalidSelection
        );
        assert!(
            matches!(result.causes()[0].detail(), InventoryCauseDetail::Selection { reason: observed, .. } if *observed == reason)
        );
        assert_eq!(result.request().source_names(), names);
    }
    let mut invalid = model.clone();
    invalid.obligations[0].target = Some("unknown-target".into());
    let result = quire_rs::obligation::inventory(
        root.path(),
        &invalid,
        &["acceptance-criterion"],
        Default::default(),
    );
    assert_eq!(result.terminal(), InventoryTerminal::Incomplete);
    assert!(result.selection().is_none());
    assert_eq!(result.causes()[0].code(), InventoryCauseCode::InvalidModel);
    assert!(
        matches!(result.causes()[0].detail(), InventoryCauseDetail::Model(reason) if reason == &invalid.validate().unwrap_err())
    );
    let valid = selected_at(root.path());
    assert_eq!(valid.terminal(), InventoryTerminal::Complete);
    let selection = valid.selection().unwrap();
    assert_eq!(selection.root(), root.path().canonicalize().unwrap());
    assert_eq!(
        selection.sources()[0].declaration().name,
        "acceptance-criterion"
    );
    assert_eq!(
        selection.sources()[0].target(),
        Some(&model.trace_targets[0])
    );
}

#[trace("FR-053-AC-16", "FR-053-AC-17", "FR-053-AC-21")]
#[test]
fn selected_inventory_borrowed_identity_changes_with_authored_source() {
    use quire_rs::obligation::inventory::{
        CriterionOrigin, InventoryLookupError, InventoryTerminal,
    };
    let scope = ac_bundle("selected-identity", "| FR-001-AC-1 | First criterion. | Test (TC-001) | P1 |\n| FR-001-AC-2 | Unmapped criterion. | Analysis | P2 |\n");
    let first = selected_at(&scope);
    assert_eq!(first.terminal(), InventoryTerminal::Complete);
    let entries: Vec<_> = first.entries().collect();
    let key = entries[1].identity();
    assert!(matches!(
        key.origin(),
        CriterionOrigin::AuthoredRow {
            authored_id: "FR-001-AC-2"
        }
    ));
    assert_eq!(key.source_name(), "acceptance-criterion");
    assert_eq!(key.document(), "FR-001.md");
    assert!(std::ptr::eq(key.selection(), first.selection().unwrap()));
    assert_eq!(entries[1].obligation().statement, "Unmapped criterion.");
    assert_eq!(entries[1].obligation().method.as_deref(), Some("Analysis"));
    assert!(entries[1].obligation().target_ids.is_empty());
    assert_eq!(entries[0].obligation().target_ids, ["TC-001"]);
    assert_eq!(entries[1].row(), Some(2));
    assert!(entries[1].line().is_some());
    assert!(std::ptr::eq(
        first.lookup(&key).unwrap().obligation(),
        entries[1].obligation()
    ));
    let equal = selected_at(&scope);
    assert_eq!(first.selection(), equal.selection());
    assert_eq!(
        equal.lookup(&key).unwrap().obligation(),
        entries[1].obligation()
    );
    let path = scope.join("FR-001.md");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        text.replace("Unmapped criterion.", "Changed owner statement."),
    )
    .unwrap();
    let changed = selected_at(&scope);
    assert_eq!(changed.len(), 2);
    assert_eq!(
        changed.entries().nth(1).unwrap().obligation().statement,
        "Changed owner statement."
    );
    assert_eq!(
        changed.lookup(&key).unwrap_err(),
        InventoryLookupError::SelectionMismatch
    );
    fs::write(
        &path,
        text.replace(
            "| FR-001-AC-1 | First criterion. | Test (TC-001) | P1 |\n",
            "",
        ),
    )
    .unwrap();
    let deleted = selected_at(&scope);
    assert_eq!(deleted.len(), 1);
    assert_eq!(
        deleted.entries().next().unwrap().obligation().id,
        "FR-001-AC-2"
    );
    assert_eq!(
        deleted.lookup(&key).unwrap_err(),
        InventoryLookupError::SelectionMismatch
    );
    assert_eq!(
        deleted
            .lookup_key(
                "acceptance-criterion",
                "FR-001.md",
                CriterionOrigin::AuthoredRow {
                    authored_id: "FR-001-AC-1"
                }
            )
            .unwrap_err(),
        InventoryLookupError::UnknownIdentity
    );
    fs::write(
        &path,
        text.replace(
            "| FR-001-AC-2 | Unmapped criterion. | Analysis | P2 |\n",
            "",
        ),
    )
    .unwrap();
    let other_deleted = selected_at(&scope);
    assert_eq!(other_deleted.terminal(), InventoryTerminal::Complete);
    assert_eq!(other_deleted.len(), 1);
    assert_eq!(
        other_deleted.entries().next().unwrap().obligation().id,
        "FR-001-AC-1"
    );
    let foreign = ac_bundle(
        "selected-foreign",
        "| FR-001-AC-2 | Unmapped criterion. | Analysis | P2 |\n",
    );
    assert_eq!(
        selected_at(&foreign).lookup(&key).unwrap_err(),
        InventoryLookupError::SelectionMismatch
    );
}

#[trace("FR-053-AC-19")]
#[test]
fn selected_inventory_duplicate_lookup_retains_every_locus() {
    use quire_rs::obligation::inventory::{
        InventoryCauseCode, InventoryCauseDetail, InventoryLookupError,
    };
    let scope = ac_bundle("selected-duplicates", "| FR-001-AC-1 | One. | Analysis | |\n| FR-001-AC-1 | Two. | Analysis | |\n| FR-001-AC-2 | Unique. | Analysis | |\n");
    let result = selected_at(&scope);
    assert_eq!(result.len(), 3);
    let entries: Vec<_> = result.entries().collect();
    assert_eq!(
        result.lookup(&entries[0].identity()).unwrap_err(),
        InventoryLookupError::AmbiguousIdentity
    );
    assert_eq!(
        result.lookup(&entries[1].identity()).unwrap_err(),
        InventoryLookupError::AmbiguousIdentity
    );
    assert_eq!(
        result
            .lookup(&entries[2].identity())
            .unwrap()
            .obligation()
            .statement,
        "Unique."
    );
    let cause = &result.causes()[0];
    assert_eq!(cause.code(), InventoryCauseCode::DuplicateIdentity);
    assert_eq!(
        cause.document(),
        Some(scope.canonicalize().unwrap().join("FR-001.md").as_path())
    );
    assert_eq!(cause.row(), Some(1));
    assert_eq!(cause.line(), entries[0].line());
    assert!(entries[0].line() < entries[1].line());
    assert!(
        matches!(cause.detail(), InventoryCauseDetail::Duplicate { authored_id, rows } if authored_id == "FR-001-AC-1" && rows == &[1,2])
    );
}

#[trace("FR-053-AC-20")]
#[test]
fn selected_inventory_stops_without_mistaking_a_prefix_for_exhaustion() {
    use quire_rs::obligation::inventory::{
        InventoryCauseCode, InventoryCauseDetail, InventoryOptions, InventoryTerminal,
    };
    use std::error::Error;
    let scope = ac_bundle(
        "selected-stops",
        "| FR-001-AC-1 | One. | Analysis | |\n| FR-001-AC-2 | Two. | Analysis | |\n",
    );
    let model = model_of("obligations");
    for (limit, count, terminal) in [
        (Some(0), 0, InventoryTerminal::Incomplete),
        (Some(1), 1, InventoryTerminal::Incomplete),
        (Some(2), 2, InventoryTerminal::Incomplete),
        (Some(3), 2, InventoryTerminal::Complete),
        (None, 2, InventoryTerminal::Complete),
    ] {
        let result = quire_rs::obligation::inventory(
            &scope,
            &model,
            &["acceptance-criterion"],
            InventoryOptions {
                stop_limit: limit,
                ..Default::default()
            },
        );
        assert_eq!(result.len(), count);
        assert_eq!(result.terminal(), terminal);
        if terminal == InventoryTerminal::Incomplete {
            assert_eq!(result.causes().len(), 1);
            assert_eq!(result.causes()[0].code(), InventoryCauseCode::LimitReached);
            assert!(
                matches!(result.causes()[0].detail(), InventoryCauseDetail::Limit { limit: observed } if Some(*observed) == limit)
            );
        } else {
            assert!(result.causes().is_empty());
        }
    }
    for after in [0, 1, 2] {
        let interrupt = |emitted| {
            (emitted >= after).then(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "original requested stop",
                )) as quire_rs::obligation::inventory::InventoryInterruption
            })
        };
        let result = quire_rs::obligation::inventory(
            &scope,
            &model,
            &["acceptance-criterion"],
            InventoryOptions {
                interrupt: Some(&interrupt),
                ..Default::default()
            },
        );
        assert_eq!(result.len(), after);
        assert_eq!(result.terminal(), InventoryTerminal::Incomplete);
        assert_eq!(result.causes().len(), 1);
        let cause = &result.causes()[0];
        assert_eq!(cause.code(), InventoryCauseCode::Interrupted);
        let original = cause
            .source()
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap();
        assert_eq!(original.kind(), std::io::ErrorKind::Interrupted);
        assert_eq!(original.to_string(), "original requested stop");
    }
    let empty = tempfile::tempdir().unwrap();
    let stopped = quire_rs::obligation::inventory(
        empty.path(),
        &model,
        &["acceptance-criterion"],
        InventoryOptions {
            stop_limit: Some(0),
            ..Default::default()
        },
    );
    assert_eq!(stopped.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(
        selected_at(empty.path()).terminal(),
        InventoryTerminal::Complete
    );
}

#[trace("FR-053-AC-18", "FR-053-AC-20")]
#[test]
fn selected_inventory_filesystem_failures_retain_original_causes_and_rows() {
    use quire_rs::obligation::inventory::{
        InventoryCauseCode, InventoryCauseDetail, InventoryTerminal,
    };
    use std::error::Error;
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("missing");
    let result = selected_at(&missing);
    assert_eq!(result.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(result.causes()[0].code(), InventoryCauseCode::RootMissing);
    assert_eq!(result.causes()[0].document(), Some(missing.as_path()));
    let original = result.causes()[0]
        .source()
        .unwrap()
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .unwrap();
    assert_eq!(original.kind(), std::io::ErrorKind::NotFound);
    write(root.path(), "file", "root is a file");
    assert_eq!(
        selected_at(&root.path().join("file")).causes()[0].code(),
        InventoryCauseCode::RootNotDirectory
    );
    let scope = root.path().join("spec");
    fs::create_dir(&scope).unwrap();
    write(&scope,"FR-001.md","---\nid: FR-001\ntype: FR\n---\n\n## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n| FR-001-AC-1 | Healthy partial row. |\n");
    fs::write(scope.join("FR-002.md"), [0xff]).unwrap();
    let partial = selected_at(&scope);
    assert_eq!(partial.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(partial.len(), 1);
    assert_eq!(
        partial.entries().next().unwrap().obligation().statement,
        "Healthy partial row."
    );
    let cause = &partial.causes()[0];
    assert_eq!(cause.code(), InventoryCauseCode::DocumentUnreadable);
    let unreadable = scope.canonicalize().unwrap().join("FR-002.md");
    assert_eq!(cause.document(), Some(unreadable.as_path()));
    assert!(
        matches!(cause.detail(),InventoryCauseDetail::Diagnostic(quire_rs::Diagnostic::DocumentUnreadable { path, reason }) if path == &unreadable && reason == &fs::read_to_string(path).unwrap_err().to_string())
    );
    fs::remove_file(scope.join("FR-002.md")).unwrap();
    assert_eq!(selected_at(&scope).terminal(), InventoryTerminal::Complete);
}

#[cfg(unix)]
#[trace("FR-053-AC-18", "FR-053-AC-20")]
#[test]
fn selected_inventory_real_permission_and_walk_failures_have_healthy_controls() {
    use quire_rs::obligation::inventory::{InventoryCauseCode, InventoryTerminal};
    use std::error::Error;
    use std::os::unix::fs::PermissionsExt;
    struct Restore(std::path::PathBuf, fs::Permissions);
    impl Drop for Restore {
        fn drop(&mut self) {
            fs::set_permissions(&self.0, self.1.clone()).expect("restore permissions");
        }
    }
    let root = tempfile::tempdir().unwrap();
    let text="---\nid: FR-001\ntype: FR\n---\n\n## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n| FR-001-AC-1 | Readable row. |\n";
    write(root.path(), "FR-001.md", text);
    let hidden = root.path().join("unreadable");
    fs::create_dir(&hidden).unwrap();
    let mode = fs::metadata(&hidden).unwrap().permissions();
    let restore = Restore(hidden.clone(), mode);
    fs::set_permissions(&hidden, fs::Permissions::from_mode(0o0)).unwrap();
    assert_eq!(
        fs::read_dir(&hidden).unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied,
        "control requires a real permission boundary"
    );
    let partial = selected_at(root.path());
    assert_eq!(partial.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(partial.len(), 1);
    let cause = partial
        .causes()
        .iter()
        .find(|cause| cause.code() == InventoryCauseCode::WalkFailed)
        .unwrap();
    assert_eq!(
        cause.document(),
        Some(hidden.canonicalize().unwrap().as_path())
    );
    let original = cause
        .source()
        .unwrap()
        .source()
        .unwrap()
        .downcast_ref::<ignore::Error>()
        .unwrap();
    assert!(original
        .io_error()
        .is_some_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied));
    let root_failure = selected_at(&hidden);
    assert_eq!(root_failure.terminal(), InventoryTerminal::Incomplete);
    assert!(!root_failure.causes().is_empty());
    drop(restore);
    assert_eq!(
        selected_at(root.path()).terminal(),
        InventoryTerminal::Complete
    );
    let path = root.path().join("FR-001.md");
    let restore = Restore(path.clone(), fs::metadata(&path).unwrap().permissions());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
    assert_eq!(
        fs::read(&path).unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    let unreadable = selected_at(root.path());
    assert_eq!(unreadable.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(
        unreadable.causes()[0].code(),
        InventoryCauseCode::DocumentUnreadable
    );
    let expected = path.canonicalize().unwrap();
    assert_eq!(unreadable.causes()[0].document(), Some(expected.as_path()));
    drop(restore);
    assert_eq!(
        selected_at(root.path()).terminal(),
        InventoryTerminal::Complete
    );
}

#[trace("FR-053-AC-21", "FR-053-AC-22")]
#[test]
fn selected_inventory_identity_exclusions_and_order_are_owner_facts() {
    use quire_rs::obligation::inventory::{
        CriterionOrigin, InventoryLookupError, InventoryTerminal,
    };
    let scope = ac_bundle("selected-order", "| FR-001-AC-1 | One. | Analysis | |\n");
    let first_doc = fs::read_to_string(scope.join("FR-001.md")).unwrap();
    write(&scope, "FR-002.md", &first_doc.replace("FR-001", "FR-002"));
    write(&scope, "ignored.md", "not frontmatter");
    write(
        &scope,
        "nonmember.md",
        "---\nid: NFR-001\ntype: NFR\n---\n\nValid nonmember.\n",
    );
    let mut model = model_of("obligations");
    model.exclude = vec!["ignored.md".into()];
    let mut other = model.obligations[0].clone();
    other.name = "a-source".into();
    model.obligations.push(other);
    let scan = |model: &quire_rs::traceability::TraceabilityModel| {
        quire_rs::obligation::inventory(
            &scope,
            model,
            &["a-source", "acceptance-criterion"],
            Default::default(),
        )
    };
    let first = scan(&model);
    assert_eq!(first.terminal(), InventoryTerminal::Complete);
    let order: Vec<_> = first
        .entries()
        .map(|entry| {
            (
                entry.identity().source_name().to_owned(),
                entry.identity().document().to_owned(),
                entry.obligation().id.clone(),
            )
        })
        .collect();
    assert_eq!(
        order,
        vec![
            (
                "acceptance-criterion".into(),
                "FR-001.md".into(),
                "FR-001-AC-1".into()
            ),
            (
                "acceptance-criterion".into(),
                "FR-002.md".into(),
                "FR-002-AC-1".into()
            ),
            ("a-source".into(), "FR-001.md".into(), "FR-001-AC-1".into()),
            ("a-source".into(), "FR-002.md".into(), "FR-002-AC-1".into())
        ]
    );
    let again = scan(&model);
    assert_eq!(first.selection(), again.selection());
    assert_eq!(first.terminal(), again.terminal());
    assert_eq!(
        first
            .entries()
            .map(|entry| entry.obligation())
            .collect::<Vec<_>>(),
        again
            .entries()
            .map(|entry| entry.obligation())
            .collect::<Vec<_>>()
    );
    let key = first.entries().next().unwrap().identity();
    fs::write(scope.join("ignored.md"), [0xff]).unwrap();
    write(
        &scope,
        "nonmember.md",
        "---\nid: NFR-001\ntype: NFR\n---\n\nEdited nonmember.\n",
    );
    write(scope.parent().unwrap(), "outside.md", "outside root");
    let irrelevant = scan(&model);
    assert_eq!(first.selection(), irrelevant.selection());
    assert!(irrelevant.lookup(&key).is_ok());
    write(
        &scope,
        "FR-001.md",
        &(first_doc.clone() + "\n## Unrelated Section\n\nChanged selected bytes.\n"),
    );
    assert_eq!(
        scan(&model).lookup(&key).unwrap_err(),
        InventoryLookupError::SelectionMismatch
    );
    write(&scope, "FR-001.md", &first_doc);
    let mut contract = model.clone();
    contract.obligations[0]
        .exclude
        .push("never-matches.md".into());
    let changed = scan(&contract);
    assert_eq!(changed.len(), first.len());
    assert_eq!(
        changed.lookup(&key).unwrap_err(),
        InventoryLookupError::SelectionMismatch
    );
    contract.obligations[0].exclude = vec!["FR-002.md".into()];
    contract.obligations[1].exclude = vec!["FR-002.md".into()];
    let excluded = scan(&contract);
    assert_eq!(excluded.len(), 2);
    assert!(excluded
        .entries()
        .all(|entry| entry.identity().document() == "FR-001.md"));
    assert_eq!(excluded.terminal(), InventoryTerminal::Complete);
    write(
        &scope,
        "nonmember.md",
        &first_doc.replace("FR-001", "FR-003"),
    );
    assert_eq!(scan(&model).len(), 6);
    assert_eq!(
        scan(&model).lookup(&key).unwrap_err(),
        InventoryLookupError::SelectionMismatch
    );
    assert_eq!(
        first
            .lookup_key(
                "wrong-source",
                "FR-001.md",
                CriterionOrigin::AuthoredRow {
                    authored_id: "FR-001-AC-1"
                }
            )
            .unwrap_err(),
        InventoryLookupError::UnknownIdentity
    );
}

#[trace("FR-053-AC-17")]
#[test]
fn selected_inventory_rendered_rows_never_claim_authored_ids() {
    use quire_rs::obligation::inventory::{CriterionOrigin, InventoryTerminal};
    let root = tempfile::tempdir().unwrap();
    let head="---\nid: NFR-001\ntype: NFR\n---\n\n## Measurement and Evaluation\n\n| Metric | Target | Threshold | Method |\n|--------|--------|-----------|--------|\n";
    let first = "| First measurement. | 10 | 20 | Analysis |\n";
    let second = "| Second measurement. | 30 | 40 | Test |\n";
    write(
        root.path(),
        "NFR-001.md",
        &(head.to_owned() + first + second),
    );
    let model = model_of("obligations-nfr");
    let scan = || {
        quire_rs::obligation::inventory(root.path(), &model, &["nfr-metric"], Default::default())
    };
    let before = scan();
    assert_eq!(before.terminal(), InventoryTerminal::Complete);
    let row = before.entries().nth(1).unwrap();
    assert!(matches!(
        row.identity().origin(),
        CriterionOrigin::RenderedRow {
            rendered_id: "NFR-001-M-2",
            row: 2
        }
    ));
    assert_eq!(
        row.obligation()
            .parameters
            .get("target")
            .map(String::as_str),
        Some("30")
    );
    assert_eq!(
        before
            .lookup(&row.identity())
            .unwrap()
            .obligation()
            .statement,
        "Second measurement."
    );
    write(root.path(), "NFR-001.md", &(head.to_owned() + second));
    let after = scan();
    assert_eq!(after.len(), 1);
    let row = after.entries().next().unwrap();
    assert!(matches!(
        row.identity().origin(),
        CriterionOrigin::RenderedRow {
            rendered_id: "NFR-001-M-1",
            row: 1
        }
    ));
    assert_eq!(row.obligation().statement, "Second measurement.");
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigurationInventoryCases {
    issue_ref: String,
    cases: Vec<ConfigurationInventoryCase>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigurationInventoryCase {
    name: String,
    rows: String,
    count: usize,
    unusable_rows: Vec<usize>,
    tuples: Option<usize>,
}

#[trace("FR-053-AC-17", "FR-053-AC-19")]
#[test]
fn selected_inventory_configuration_rows_follow_the_real_minter() {
    use quire_rs::obligation::inventory::{CriterionOrigin, InventoryCauseCode, InventoryTerminal};
    use quire_rs::traceability::{CombinatorialColumns, TraceabilityModel};
    let mut source = model_of("obligations").obligations.remove(0);
    source.name = "configuration-space".into();
    source.target = None;
    source.archetype = Some("FR".into());
    source.section = Some("Configuration Dimensions".into());
    source.id_format = Some("{document}-COMB".into());
    source.statement_column = "Dimension".into();
    source.method_column = None;
    source.criticality_column = None;
    source.combinatorial = Some(CombinatorialColumns {
        dimension_column: "Dimension".into(),
        values_column: "Values".into(),
        excludes_column: Some("Excludes".into()),
        strength: 2,
    });
    let model = TraceabilityModel {
        obligations: vec![source],
        ..Default::default()
    };
    let cases: ConfigurationInventoryCases = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/selected-inventory-configurations.json"
    ))
    .unwrap();
    assert_eq!(
        cases.issue_ref,
        "https://linear.app/agent-ix/issue/PLAT-1165"
    );
    let mut names = std::collections::BTreeSet::new();
    for case in cases.cases {
        assert!(names.insert(case.name.clone()));
        let root = tempfile::tempdir().unwrap();
        let text=format!("---\nid: FR-001\ntype: FR\n---\n\n## Configuration Dimensions\n\n| Dimension | Values | Excludes |\n|-----------|--------|----------|\n{}",case.rows);
        write(root.path(), "FR-001.md", &text);
        let inventory = quire_rs::obligation::inventory(
            root.path(),
            &model,
            &["configuration-space"],
            Default::default(),
        );
        assert_eq!(inventory.len(), case.count, "{} count", case.name);
        assert_eq!(
            inventory.terminal(),
            if case.unusable_rows.is_empty() {
                InventoryTerminal::Complete
            } else {
                InventoryTerminal::Incomplete
            },
            "{} terminal",
            case.name
        );
        let rows: Vec<_> = inventory
            .causes()
            .iter()
            .map(|cause| {
                assert_eq!(
                    cause.code(),
                    InventoryCauseCode::UnusableRow,
                    "{} cause",
                    case.name
                );
                assert_eq!(
                    cause.document(),
                    Some(
                        root.path()
                            .canonicalize()
                            .unwrap()
                            .join("FR-001.md")
                            .as_path()
                    )
                );
                assert!(cause.line().is_some());
                cause.row().unwrap()
            })
            .collect();
        assert_eq!(rows, case.unusable_rows, "{} row locus", case.name);
        let legacy = derive(&Spec::from_path(root.path()), root.path(), &model);
        assert_eq!(
            inventory
                .entries()
                .map(|entry| entry.obligation())
                .collect::<Vec<_>>(),
            legacy.0.iter().collect::<Vec<_>>(),
            "{} actual producer equality",
            case.name
        );
        if let Some(tuples) = case.tuples {
            let entry = inventory.entries().next().unwrap();
            assert!(matches!(
                entry.identity().origin(),
                CriterionOrigin::CombinatorialTable {
                    rendered_id: "FR-001-COMB"
                }
            ));
            assert_eq!(entry.row(), None);
            assert_eq!(
                entry.obligation().parameters.get("tuples"),
                Some(&tuples.to_string())
            );
            assert_eq!(
                inventory.lookup(&entry.identity()).unwrap().obligation(),
                entry.obligation()
            );
        }
    }
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "FR-001.md",
        "---\nid: FR-001\ntype: FR\n---\n\n## Context\nNo optional inference.\n",
    );
    let missing = quire_rs::obligation::inventory(
        root.path(),
        &model,
        &["configuration-space"],
        Default::default(),
    );
    assert_eq!(
        missing.causes()[0].code(),
        InventoryCauseCode::MissingSection
    );
    assert_eq!(missing.terminal(), InventoryTerminal::Incomplete);
}

#[trace("FR-053-AC-23")]
#[test]
fn selected_inventory_preserves_legacy_consumers_without_evidence_verdicts() {
    use quire_rs::obligation::inventory::{InventoryCauseCode, InventoryTerminal};
    let scope=ac_bundle("selected-legacy","| FR-001-AC-1 | The system shall preserve its statement. | Test (TC-001) | P1 |\n| FR-001-AC-2 | | Analysis | |\n");
    let model = model_of("obligations");
    let spec = Spec::from_path(&scope);
    let before = derive(&spec, &scope, &model);
    assert_eq!(before.0.len(), 1);
    assert_eq!(before.0[0].id, "FR-001-AC-1");
    assert_eq!(
        before.0[0].statement,
        "The system shall preserve its statement."
    );
    assert_eq!(before.1.len(), 1);
    assert_eq!(before.1[0].row, 2);
    let registry = Registry::load_module(&fixture_module("obligations")).unwrap();
    let extraction = extract_tree(&scope);
    let graph = trace::bind(&extraction, &model);
    let coverage_before = compute(&spec, &registry, &graph, &scope).unwrap();
    let doc = quire_rs::parse_document(&fs::read_to_string(scope.join("FR-001.md")).unwrap());
    let document_before =
        quire_rs::obligation::for_document(&model, "FR", &doc, Some(Path::new("FR-001.md")));
    let document_text = fs::read_to_string(scope.join("FR-001.md")).unwrap();
    let archetype = registry.archetype("FR").unwrap();
    let classification_before = quire_rs::classify_document_criteria(
        &registry,
        archetype,
        &document_text,
        Some(Path::new("FR-001.md")),
    );
    let inventory = selected_at(&scope);
    assert_eq!(inventory.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(
        inventory.causes()[0].code(),
        InventoryCauseCode::UnusableRow
    );
    assert_eq!(
        inventory
            .entries()
            .map(|entry| entry.obligation())
            .collect::<Vec<_>>(),
        before.0.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        classification_before,
        quire_rs::classify_document_criteria(
            &registry,
            archetype,
            &document_text,
            Some(Path::new("FR-001.md"))
        )
    );
    let after = derive(&spec, &scope, &model);
    assert_eq!(before, after);
    assert_eq!(
        document_before,
        quire_rs::obligation::for_document(&model, "FR", &doc, Some(Path::new("FR-001.md")))
    );
    assert_eq!(
        serde_json::to_value(&coverage_before).unwrap(),
        serde_json::to_value(compute(&spec, &registry, &graph, &scope).unwrap()).unwrap()
    );
    let record = serde_json::to_value(inventory.entries().next().unwrap().obligation()).unwrap();
    assert_eq!(
        record,
        serde_json::json!({"source":"acceptance-criterion","id":"FR-001-AC-1","document":"FR-001.md","statement":"The system shall preserve its statement.","method":"Test","criticality":"P1","target_ids":["TC-001"]})
    );
    let missing = scope.join("absent");
    let tolerated = derive(&Spec::from_path(&missing), &missing, &model);
    assert!(tolerated.0.is_empty());
    assert!(tolerated.1.is_empty());
    assert_eq!(
        selected_at(&missing).terminal(),
        InventoryTerminal::Incomplete
    );
}

// PLAT-104 / quire-rs#428: quoin reads exactly `id` and `statement` from each
// entry of `coverage --json`'s `obligations[]` through a deserialize-only
// projection with both fields required. Renaming or dropping either is a
// breaking change for that consumer, so the serialized payload pins both.
#[test]
fn coverage_json_obligations_carry_id_and_statement() {
    let scope = ac_bundle(
        "plat104",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n\
         | FR-001-AC-2 | The system shall log it. | Analysis | P2 |\n",
    );
    let registry = Registry::load_module(&fixture_module("obligations")).expect("load module");
    let spec = Spec::from_path(&scope);
    let source = scope.parent().unwrap().join("src");
    fs::create_dir_all(&source).expect("mkdir");
    write(&source, "lib.rs", "//! empty\n");
    let extraction = extract_tree(&source);
    let model = registry.traceability().cloned().unwrap();
    let graph = trace::bind(&extraction, &model);
    let report = compute(&spec, &registry, &graph, &scope).expect("report");

    let json: serde_json::Value = serde_json::from_str(&report.to_json()).expect("valid JSON");
    let entries = json["obligations"].as_array().expect("obligations array");
    assert_eq!(entries.len(), 2, "{entries:#?}");
    let pairs: Vec<(&str, &str)> = entries
        .iter()
        .map(|e| {
            (
                e["id"].as_str().expect("`id` must be a string"),
                e["statement"]
                    .as_str()
                    .expect("`statement` must be a string"),
            )
        })
        .collect();
    assert_eq!(
        pairs,
        [
            ("FR-001-AC-1", "The system shall do it."),
            ("FR-001-AC-2", "The system shall log it."),
        ]
    );
}

const LEGEND: &str = "\nSymbols.\n\n| Symbol | Meaning |\n|--------|---------|\n| x | y |\n";

#[trace("FR-049-AC-29", "FR-049-AC-30")]
#[test]
fn every_table_under_an_obligation_section_derives_and_a_legend_raises_no_cause() {
    use quire_rs::obligation::inventory::InventoryTerminal;
    let scope = ac_bundle(
        "1500a",
        "| FR-001-AC-1 | The system shall do it. | Test (TC-001) | P1 |\n",
    );
    let fr = scope.join("FR-001.md");
    let mut text = fs::read_to_string(&fr).expect("read");
    text.push_str(LEGEND);
    text.push_str(
        "\nMore.\n\n| ID | Criteria | Verification | Priority |\n\
         |----|----------|--------------|----------|\n\
         | FR-001-AC-2 | The system shall also do it. | Analysis | P2 |\n",
    );
    // A fenced table is an example and derives nothing (FR-049-AC-27).
    text.push_str(
        "\n```\n| ID | Criteria | Verification | Priority |\n\
         |----|----------|--------------|----------|\n\
         | FR-001-AC-9 | Example. | Test | P3 |\n```\n",
    );
    fs::write(&fr, text).expect("write");

    let ids: Vec<_> = derive_at(&scope, "obligations")
        .into_iter()
        .map(|o| o.id)
        .collect();
    assert_eq!(ids, ["FR-001-AC-1", "FR-001-AC-2"]);

    let inventory = selected_at(&scope);
    let inv_ids: Vec<_> = inventory
        .entries()
        .map(|entry| entry.obligation().id.clone())
        .collect();
    assert_eq!(inv_ids, ids);
    assert!(inventory.causes().is_empty(), "{:?}", inventory.causes());
    assert_eq!(inventory.terminal(), InventoryTerminal::Complete);
}

#[trace("FR-049-AC-30")]
#[test]
fn inventory_reports_one_missing_column_cause_when_no_table_has_it() {
    let scope = tmpdir("1500b").join("spec");
    fs::create_dir_all(&scope).expect("mkdir");
    let grid = "| ID | Notes |\n|----|-------|\n| FR-001-AC-1 | n |\n";
    write(
        &scope,
        "FR-001.md",
        &format!(
            "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n## Acceptance Criteria\n\n{grid}\nmore\n\n{grid}"
        ),
    );
    let inventory = selected_at(&scope);
    let causes: Vec<_> = inventory.causes().iter().map(|c| c.code()).collect();
    assert_eq!(
        causes,
        [quire_rs::obligation::inventory::InventoryCauseCode::MissingColumn]
    );
}
