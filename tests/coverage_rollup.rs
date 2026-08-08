//! FR-050 — coverage reconciliation (TC-734..TC-740) and the FR-050-CON-2 /
//! FR-051-CON-1 static boundary audit (TC-756).

use std::fs;
use std::path::{Path, PathBuf};

use quire_rs::coverage::{compute, CoverageError, CoverageReport};
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
    p.push(format!("quire-rs-coverage-{}-{suffix}", std::process::id()));
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

/// An ISO-shaped bundle: one FR minting two AC ids, a matrix minting two TC
/// rows, and a Rust source tree whose tests carry `#[trace(...)]` markers.
struct Bundle {
    scope: PathBuf,
    source: PathBuf,
}

fn iso_bundle(suffix: &str, matrix_rows: &[(&str, &str, &str)], traced: &[&str]) -> Bundle {
    let root = tmpdir(suffix);
    let scope = root.join("spec");
    let source = root.join("src");
    fs::create_dir_all(&scope).expect("mkdir");
    fs::create_dir_all(&source).expect("mkdir");

    write(
        &scope,
        "FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test (TC-001) |\n\
         | FR-001-AC-2 | The system shall also do it. | Test (TC-002) |\n",
    );

    let mut matrix = String::from(
        "# Test Matrix\n\n## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n",
    );
    for (tc, traces_to, status) in matrix_rows {
        matrix.push_str(&format!("| {tc} | {traces_to} | {status} |\n"));
    }
    write(&scope, "tests.md", &matrix);

    let mut lib = String::from("//! Fixture source tree.\n\n#[cfg(test)]\nmod tests {\n");
    for (idx, id) in traced.iter().enumerate() {
        lib.push_str(&format!(
            "    #[trace(\"{id}\")]\n    #[test]\n    fn covers_{idx}() {{\n        let _ = 1;\n    }}\n"
        ));
    }
    lib.push_str("}\n");
    write(&source, "lib.rs", &lib);

    Bundle { scope, source }
}

/// Rewrite the bundle's FR with `cells` as its `Acceptance Criteria`, keeping
/// the ids and `Verification` references [`iso_bundle`] mints. Lets one test
/// choose whether the criteria classify as property-shaped or not.
fn rewrite_criteria(bundle: &Bundle, cells: &[&str]) {
    let mut md = String::from(
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n",
    );
    for (i, cell) in cells.iter().enumerate() {
        md.push_str(&format!(
            "| FR-001-AC-{} | {cell} | Test (TC-{:03}) |\n",
            i + 1,
            i + 1
        ));
    }
    write(&bundle.scope, "FR-001.md", &md);
}

fn report_for(bundle: &Bundle, module: &str) -> Result<CoverageReport, CoverageError> {
    let registry = Registry::load_module(&fixture_module(module)).expect("load module");
    let spec = Spec::from_path(&bundle.scope);
    let extraction = extract_tree(&bundle.source);
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = trace::bind(&extraction, &model);
    compute(&spec, &registry, &graph, &bundle.scope)
}

// TC-734 (FR-050-AC-3): a reference row whose trace target has no backing
// `verifies` relation appears in unbacked rows with the row id and target id.
#[test]
fn tc734_unbacked_rows() {
    let bundle = iso_bundle(
        "734",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        // Only TC-001 is bound by a test.
        &["TC-001"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let unbacked: Vec<&str> = report
        .unbacked_rows
        .iter()
        .filter_map(|r| r.row_id.as_deref())
        .collect();
    assert!(unbacked.contains(&"TC-002"), "unbacked: {unbacked:?}");
    assert!(!unbacked.contains(&"TC-001"));

    let row = report
        .unbacked_rows
        .iter()
        .find(|r| r.row_id.as_deref() == Some("TC-002"))
        .expect("TC-002 row");
    assert_eq!(row.reference, "traces-to");
    assert_eq!(row.document, "tests.md");
    assert!(row.target_ids.contains(&"TC-002".to_string()));
    assert!(row.target_ids.contains(&"FR-001-AC-2".to_string()));
}

// TC-735 (FR-050-AC-4): a `complete`-classed row with no backing symbol is a
// status lie; the same row with a backing symbol is not.
#[test]
fn tc735_status_lies() {
    // TC-002 claims ✅ but nothing binds it.
    let lying = iso_bundle(
        "735-lying",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
        ],
        &["TC-001"],
    );
    let report = report_for(&lying, "iso").expect("model declared");
    let lies: Vec<&str> = report
        .status_lies
        .iter()
        .filter_map(|l| l.row_id.as_deref())
        .collect();
    assert_eq!(lies, vec!["TC-002"]);
    assert_eq!(report.status_lies[0].status, "✅");

    // Bind it, and the lie disappears.
    let honest = iso_bundle(
        "735-honest",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
        ],
        &["TC-001", "TC-002"],
    );
    assert!(report_for(&honest, "iso")
        .expect("model declared")
        .status_lies
        .is_empty());

    // A pending row that is unbacked is *not* a lie — only `complete` is.
    let pending = iso_bundle("735-pending", &[("TC-001", "FR-001-AC-1", "🚧")], &[]);
    assert!(report_for(&pending, "iso")
        .expect("model declared")
        .status_lies
        .is_empty());
}

// TC-736 (FR-050-AC-5): a symbol whose trace tag resolves to no declared
// target or row appears in untracked symbols with its file and symbol name.
#[test]
fn tc736_untracked_symbols() {
    let bundle = iso_bundle(
        "736",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &["TC-001", "TC-999"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let untracked: Vec<&str> = report
        .untracked_symbols
        .iter()
        .map(|s| s.trace_id.as_str())
        .collect();
    assert_eq!(untracked, vec!["TC-999"]);
    let entry = &report.untracked_symbols[0];
    assert_eq!(entry.path, "lib.rs");
    assert!(entry.symbol.contains("covers_1"));
}

// TC-737 (FR-050-AC-6): per-minting-document counts, summing to the totals.
#[test]
fn tc737_per_group_counts_sum_to_totals() {
    let bundle = iso_bundle(
        "737",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        &["TC-001", "FR-001-AC-1"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let fr = report
        .groups
        .iter()
        .find(|g| g.document == "FR-001.md")
        .expect("FR group");
    assert_eq!(fr.target, "acceptance-criterion");
    assert_eq!(fr.total, 2, "two AC ids minted");
    assert_eq!(fr.backed, 1, "only FR-001-AC-1 is bound");

    let matrix = report
        .groups
        .iter()
        .find(|g| g.document == "tests.md")
        .expect("matrix group");
    assert_eq!(matrix.total, 2);
    assert_eq!(matrix.backed, 1);

    assert_eq!(
        report.totals.total,
        report.groups.iter().map(|g| g.total).sum::<usize>()
    );
    assert_eq!(
        report.totals.backed,
        report.groups.iter().map(|g| g.backed).sum::<usize>()
    );
    assert_eq!(report.totals.total, 4);
    assert_eq!(report.totals.backed, 2);
}

// TC-738 (FR-050-AC-7, Property): repeated runs over identical inputs emit
// byte-identical JSON.
#[test]
fn tc738_report_json_is_byte_identical() {
    let bundle = iso_bundle(
        "738",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
            ("TC-003", "FR-001-AC-2", "❌"),
        ],
        &["TC-001", "TC-999"],
    );
    let first = report_for(&bundle, "iso")
        .expect("model declared")
        .to_json();
    assert!(first.contains("unbacked_rows"));
    for _ in 0..8 {
        assert_eq!(
            first,
            report_for(&bundle, "iso")
                .expect("model declared")
                .to_json(),
            "coverage JSON must be byte-identical across runs"
        );
    }
}

// TC-739 (FR-050-AC-8): a non-ISO vocabulary gets a correct rollup from its
// own declaration, with no engine change.
#[test]
fn tc739_non_iso_model_rolls_up() {
    let root = tmpdir("739");
    let scope = root.join("rules");
    let source = root.join("src");
    fs::create_dir_all(&scope).expect("mkdir");
    fs::create_dir_all(&source).expect("mkdir");

    write(
        &scope,
        "R-001.md",
        "---\nid: R-001\ntype: Rule\ntitle: A rule\n---\n\n## Clauses\n\n\
         | Clause | Evidence |\n|--------|----------|\n\
         | R-001-C-1 | checked by C-001 |\n\
         | R-001-C-2 | checked by C-002 |\n",
    );
    write(
        &scope,
        "checks.md",
        "# Check Register\n\n## Checks\n\n| Check | Covers | State |\n|-------|--------|-------|\n\
         | C-001 | R-001-C-1 | done |\n\
         | C-002 | R-001-C-2 | done |\n",
    );
    // The alt module declares `#[covers(...)]`, not `#[trace(...)]`.
    write(
        &source,
        "lib.rs",
        "#[cfg(test)]\nmod tests {\n    #[covers(\"C-001\")]\n    #[test]\n    fn one() {\n        let _ = 1;\n    }\n}\n",
    );

    let bundle = Bundle { scope, source };
    let report = report_for(&bundle, "alt").expect("model declared");

    // Clause + check groups both counted from the alt declaration.
    assert_eq!(report.totals.total, 4);
    assert_eq!(report.totals.backed, 1, "only C-001 is covered");
    let clauses = report
        .groups
        .iter()
        .find(|g| g.target == "clause")
        .expect("clause group");
    assert_eq!(clauses.total, 2);
    // C-002's row claims `done` while nothing covers it — a status lie in the
    // alt vocabulary, with no ISO status value anywhere in play.
    let lies: Vec<&str> = report
        .status_lies
        .iter()
        .filter_map(|l| l.row_id.as_deref())
        .collect();
    assert_eq!(lies, vec!["C-002"]);
    assert_eq!(report.status_lies[0].status, "done");
}

// TC-740 (FR-050-AC-9): with no declared model, coverage fails with a distinct
// diagnostic naming the missing declaration — never an empty report.
#[test]
fn tc740_no_model_is_a_distinct_diagnostic() {
    let bundle = iso_bundle("740", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    let module = tmpdir("740-module");
    fs::write(
        module.join("manifest.yaml"),
        "name: no-model\nartifact_types:\n- name: FR\n",
    )
    .expect("write manifest");

    let registry = Registry::load_module(&module).expect("load module");
    let spec = Spec::from_path(&bundle.scope);
    let graph = trace::bind(&extract_tree(&bundle.source), &Default::default());
    let err = compute(&spec, &registry, &graph, &bundle.scope).expect_err("must not report");

    assert_eq!(err, CoverageError::ModelUndeclared);
    assert!(err.to_string().contains("traceability"));
}

// TC-788 (FR-052-AC-10, FR-050-AC-13): the CR-028 criteria rollup — one entry
// per document binding criteria plus the two new totals, and byte-identical
// serialization across runs.
#[test]
fn tc788_criteria_counts_and_totals() {
    let bundle = iso_bundle(
        "788",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        &["TC-001"],
    );
    // A universally quantified criterion and a specific-scenario one, so the
    // entry carries a property-shaped count that is neither 0 nor the total.
    rewrite_criteria(
        &bundle,
        &[
            "A finding whose key is absent from the merged map defaults to warning.",
            "The loader emits a `Duplicate` diagnostic for the second declaration.",
        ],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    // One entry per minting document. `tests.md` is not a corpus document and
    // binds no criteria, so `FR-001.md` is the only contributor.
    assert_eq!(
        report
            .criteria
            .iter()
            .map(|c| c.document.as_str())
            .collect::<Vec<_>>(),
        vec!["FR-001.md"],
    );
    let entry = &report.criteria[0];
    assert_eq!(entry.archetype, "FR");
    assert_eq!(entry.criteria, 2, "both AC rows are binding criteria");
    assert_eq!(
        entry.property_shaped, 1,
        "only the quantified criterion is extractable: {:?}",
        entry.by_property
    );
    // The histogram accounts for every criterion exactly once.
    assert_eq!(
        entry.by_property.values().sum::<usize>(),
        entry.criteria,
        "by_property: {:?}",
        entry.by_property
    );

    // The totals are the sum over the entries — the same relation FR-050-AC-6
    // states for backed/total — and they are present as a pair.
    assert_eq!(
        report.totals.criteria,
        Some(report.criteria.iter().map(|c| c.criteria).sum::<usize>())
    );
    assert_eq!(
        report.totals.property_shaped,
        Some(
            report
                .criteria
                .iter()
                .map(|c| c.property_shaped)
                .sum::<usize>()
        )
    );
    assert_eq!(report.totals.criteria, Some(2));
    assert_eq!(report.totals.property_shaped, Some(1));

    // …and the classification is data, not a verdict: it moves nothing in the
    // reconciliation the report already carried.
    assert_eq!(report.totals.total, 4);
    assert_eq!(report.totals.backed, 1);

    // FR-050-AC-7 still holds over the enlarged payload.
    let first = report.to_json();
    assert!(first.contains("\"criteria\""));
    assert!(first.contains("property_shaped"));
    for _ in 0..8 {
        assert_eq!(
            first,
            report_for(&bundle, "iso")
                .expect("model declared")
                .to_json(),
            "coverage JSON must stay byte-identical across runs"
        );
    }
}

// TC-788 (FR-050-AC-13, continued): a corpus binding criteria of which *none*
// are property-shaped emits `property_shaped: 0` — present and zero, never
// absent. The two totals move as a pair, so a JSON consumer computing the
// extraction ratio divides by a number rather than by `undefined`, in exactly
// the corpus most worth reporting on (CR-020: criteria validated by
// demonstration legitimately score zero).
#[test]
fn tc788_zero_property_shaped_is_emitted_not_omitted() {
    let bundle = iso_bundle("788-zero", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    // Two specific scenarios: binding criteria, neither quantified.
    rewrite_criteria(
        &bundle,
        &[
            "The loader emits a `Duplicate` diagnostic for the second declaration.",
            "The report lists one row per declared module.",
        ],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    assert_eq!(report.criteria.len(), 1);
    assert_eq!(report.criteria[0].criteria, 2);
    assert_eq!(report.criteria[0].property_shaped, 0);
    assert_eq!(report.totals.criteria, Some(2));
    assert_eq!(
        report.totals.property_shaped,
        Some(0),
        "zero is a value, not an absence"
    );

    let json = report.to_json();
    let value: serde_json::Value = serde_json::from_str(&json).expect("parses");
    let totals: Vec<&str> = value["totals"]
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        totals,
        vec!["backed", "criteria", "property_shaped", "total"]
    );
    assert_eq!(value["totals"]["property_shaped"], serde_json::json!(0));
    assert!(
        json.contains("\"property_shaped\": 0"),
        "the key must be written, not skipped: {json}"
    );

    let restored: CoverageReport = serde_json::from_str(&json).expect("round-trips");
    assert_eq!(restored, report);
}

// TC-788 (FR-050-AC-13, continued): a corpus binding no criteria carries an
// empty list, and its JSON is byte-for-byte what an engine predating the
// field would have written — the CR-028 keys are absent, not zero-valued.
#[test]
fn tc788_no_criteria_corpus_is_unchanged() {
    let root = tmpdir("788-none");
    let scope = root.join("rules");
    let source = root.join("src");
    fs::create_dir_all(&scope).expect("mkdir");
    fs::create_dir_all(&source).expect("mkdir");

    // The alt archetype declares no `grammar_ref`, so nothing binds criteria.
    write(
        &scope,
        "R-001.md",
        "---\nid: R-001\ntype: Rule\ntitle: A rule\n---\n\n## Clauses\n\n\
         | Clause | Evidence |\n|--------|----------|\n\
         | R-001-C-1 | checked by C-001 |\n",
    );
    write(
        &scope,
        "checks.md",
        "# Check Register\n\n## Checks\n\n| Check | Covers | State |\n|-------|--------|-------|\n\
         | C-001 | R-001-C-1 | done |\n",
    );
    write(&source, "lib.rs", "//! No traced symbols.\n");

    let bundle = Bundle { scope, source };
    let report = report_for(&bundle, "alt").expect("model declared");

    assert!(report.criteria.is_empty(), "{:?}", report.criteria);
    assert_eq!(report.totals.criteria, None);
    assert_eq!(report.totals.property_shaped, None);

    let json = report.to_json();
    assert!(
        !json.contains("criteria"),
        "an absent field, not an empty one: {json}"
    );
    assert!(!json.contains("property_shaped"), "{json}");
    // The payload is exactly the pre-CR-028 key set.
    let value: serde_json::Value = serde_json::from_str(&json).expect("parses");
    let keys: Vec<&str> = value
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    // `serde_json::Value` holds its object as a map, so the comparison is over
    // the key *set*, not the (separately asserted) emitted order.
    assert_eq!(
        keys,
        vec![
            "groups",
            "status_lies",
            "totals",
            "unbacked_rows",
            "untracked_symbols"
        ]
    );
    let totals: Vec<&str> = value["totals"]
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(totals, vec!["backed", "total"]);

    // A report written by an older engine still round-trips through the
    // derived `Deserialize` — what `#[serde(default)]` buys.
    let restored: CoverageReport = serde_json::from_str(&json).expect("round-trips");
    assert_eq!(restored, report);
}

// TC-756 (FR-050-CON-2, FR-051-CON-1): static boundary audit over the coverage
// and symbol modules — no network or service I/O, and no execution of the code
// the symbols were extracted from. Mirrors the TC-690 pattern.
#[test]
fn tc756_coverage_and_symbol_modules_have_no_forbidden_surface() {
    const SOURCES: &[(&str, &str)] = &[
        ("src/coverage.rs", include_str!("../src/coverage.rs")),
        ("src/symbols/mod.rs", include_str!("../src/symbols/mod.rs")),
        (
            "src/symbols/rust.rs",
            include_str!("../src/symbols/rust.rs"),
        ),
        (
            "src/symbols/python.rs",
            include_str!("../src/symbols/python.rs"),
        ),
        (
            "src/symbols/typescript.rs",
            include_str!("../src/symbols/typescript.rs"),
        ),
        (
            "src/symbols/trace.rs",
            include_str!("../src/symbols/trace.rs"),
        ),
        (
            "src/corpus/declared_tables.rs",
            include_str!("../src/corpus/declared_tables.rs"),
        ),
        (
            "src/corpus/trace_refs.rs",
            include_str!("../src/corpus/trace_refs.rs"),
        ),
    ];
    // Reading local files is explicitly in scope ("inputs are the corpus, the
    // registry, and local source trees"), so `std::fs` is allowed here — what
    // must never appear is a network, service, or *execution* surface.
    const FORBIDDEN: &[&str] = &[
        "std::net",
        "TcpStream",
        "UdpSocket",
        "reqwest",
        "hyper",
        // `tokio` alone would trip on the Rust adapter's documentation of the
        // `#[tokio::test]` attribute it classifies — the surface that matters
        // is an actual runtime dependency.
        "use tokio",
        "tokio::runtime",
        "tokio::spawn",
        "Command",
        "std::process",
        "libloading",
        "dlopen",
        "PGlite",
        "pglite",
        "CloudManager",
        "cloud_manager",
        "embedding",
    ];
    for (name, source) in SOURCES {
        for needle in FORBIDDEN {
            assert!(
                !source.contains(needle),
                "{name} must not reference {needle:?}: the coverage/symbol boundary performs no \
                 network or service I/O and never executes extracted code \
                 (FR-050-CON-2, FR-051-CON-1)"
            );
        }
    }
}

// ── CR-015: status classification, declared vocabularies, cell normalization ──

/// A module manifest declaring only what a test needs from the model.
fn model_from(yaml: &str) -> quire_rs::traceability::TraceabilityModel {
    let module = tmpdir(&format!("cr015-{}", yaml.len()));
    fs::write(module.join("manifest.yaml"), yaml).expect("write manifest");
    Registry::load_module(&module)
        .expect("load module")
        .traceability()
        .cloned()
        .expect("declared model")
}

// TC-758 (FR-050-AC-10): a status cell with a trailing note classes by its
// leading marker, and a declared `retired` value classes retired.
#[test]
fn tc758_status_classes_by_leading_marker() {
    use quire_rs::traceability::StatusClass;
    let model = model_from(
        "name: m\ntraceability:\n  status:\n    column: Status\n    complete: [\"✅\"]\n    \
         pending: [\"🚧\"]\n    failed: [\"❌\"]\n    retired: [\"⛔\"]\n",
    );
    let status = model.status.as_ref().expect("status vocabulary");

    // Bare markers still class exactly.
    assert_eq!(status.class_of("✅"), StatusClass::Complete);
    // …and a marker carrying the reason classes the same, keeping the note.
    assert_eq!(status.class_of("✅ Complete"), StatusClass::Complete);
    assert_eq!(
        status.class_of("🚧 implementation in progress"),
        StatusClass::Pending
    );
    assert_eq!(
        status.class_of("⛔ RETIRED — render removed"),
        StatusClass::Retired
    );
    // An undeclared value is still unknown, not silently absorbed.
    assert_eq!(status.class_of("Done"), StatusClass::Unknown);
    // A word vocabulary keeps working, and a longer word is not a prefix match.
    let words = model_from(
        "name: w\ntraceability:\n  status:\n    column: State\n    complete: [\"done\"]\n",
    );
    let words = words.status.as_ref().unwrap();
    assert_eq!(words.class_of("done"), StatusClass::Complete);
    assert_eq!(words.class_of("done, verified"), StatusClass::Complete);
    assert_eq!(words.class_of("doneish"), StatusClass::Unknown);
}

// TC-759 (FR-050-AC-11): a declared column vocabulary is exposed on the
// Registry, and is the same list a matrix contract would validate against.
#[test]
fn tc759_declared_column_vocabulary() {
    let module = tmpdir("cr015-vocab");
    fs::write(
        module.join("manifest.yaml"),
        "name: m\ntraceability:\n  vocabularies:\n    test_type: [Unit, Integration, E2E, \
         Property, pg_test]\n",
    )
    .expect("write manifest");
    let registry = Registry::load_module(&module).expect("load module");

    assert_eq!(
        registry.column_vocabulary("test_type"),
        ["Unit", "Integration", "E2E", "Property", "pg_test"]
    );
    // An undeclared column reports empty rather than a guessed default.
    assert!(registry.column_vocabulary("priority").is_empty());
    let bare = Registry::load_module(&fixture_module("alt")).expect("load module");
    assert!(bare.column_vocabulary("test_type").is_empty());
}
