//! FR-050 — coverage reconciliation (TC-734..TC-740) and the FR-050-CON-2 /
//! FR-051-CON-1 static boundary audit (TC-756).

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
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

    // CR-062: the matrix is reached by archetype, so it must declare one. It
    // was frontmatter-less while `document:` binding read it off-corpus.
    let mut matrix = String::from(
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n\
         # Test Matrix\n\n## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n",
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

#[trace("TC-734", "FR-050-AC-3")]
// a reference row whose trace target has no backing
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

#[trace("TC-735", "FR-050-AC-4")]
// a `complete`-classed row with no backing symbol is a
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

#[trace("TC-736", "FR-050-AC-5")]
// a symbol whose trace tag resolves to no declared
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

#[trace("TC-737", "FR-050-AC-6")]
// per-minting-document counts, summing to the totals.
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

#[trace("TC-738", "FR-050-AC-7")]
// repeated runs over identical inputs emit (Property)
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

#[trace("TC-739", "FR-050-AC-8")]
// a non-ISO vocabulary gets a correct rollup from its
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
        // CR-062: reached by archetype, so it declares one — `CheckRegister`,
        // the alt module's own evidence archetype, not the ISO `TestMatrix`.
        "---\nid: CR-001\ntype: CheckRegister\ntitle: Check Register\n---\n\n\
         # Check Register\n\n## Checks\n\n| Check | Covers | State |\n|-------|--------|-------|\n\
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

#[trace("TC-740", "FR-050-AC-9")]
// with no declared model, coverage fails with a distinct
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

#[trace("TC-788", "FR-052-AC-10", "FR-050-AC-13")]
// the CR-028 criteria rollup — one entry
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

    // One entry per minting document. These fixture matrices carry no
    // frontmatter, so under CR-044 membership they are not documents and bind
    // no criteria — `FR-001.md` is the only contributor. (When #74 removes the
    // `document:` origin they gain `type: TestMatrix` and this inverts.)
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

#[trace("TC-788", "FR-050-AC-13")]
// a corpus binding criteria of which *none* (continued)
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

#[trace("TC-788", "FR-050-AC-13")]
// a corpus binding no criteria carries an (continued)
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

#[trace("TC-756", "FR-050-CON-2", "FR-051-CON-1")]
// static boundary audit over the coverage
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

#[trace("TC-758", "FR-050-AC-10")]
// a status cell with a trailing note classes by its
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

#[trace("TC-759", "FR-050-AC-11")]
// a declared column vocabulary is exposed on the
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

/// A bundle in the shape the CR-038 scoping exists for: a canonical
/// `spec/tests.md` reached by `document:` binding rather than by the walk —
/// it carries no frontmatter, so CR-044 membership excludes it — a second
/// in-corpus matrix, and a
/// **fixture** matrix under `fixtures/` whose rows are test data — including one
/// reusing a real id, which is what turns a fixture into a phantom backed row.
fn scoped_bundle(suffix: &str) -> Bundle {
    let bundle = iso_bundle(
        suffix,
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        &["TC-001"],
    );
    // In-corpus matrix: a real claim, and the reason `archetype:` binding is
    // wanted at all.
    write(
        &bundle.scope,
        "matrix.md",
        "---\nid: TM-002\ntype: TestMatrix\ntitle: Extra matrix\n---\n\n\
         ## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-003 | FR-001-AC-1 | 🚧 |\n",
    );
    // Fixture matrix: deliberately reuses TC-001, so admitting it mints a
    // second TC-001 that the real test appears to back.
    write(
        &bundle.scope,
        "fixtures/bad-matrix.md",
        "---\nid: TM-900\ntype: TestMatrix\ntitle: Deliberately malformed fixture\n---\n\n\
         ## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-001 | FR-001-AC-1 | ✅ |\n\
         | TC-900 | FR-001-AC-2 | ✅ |\n",
    );
    bundle
}

#[trace("TC-801", "FR-050-AC-15")]
// a declaration excluding `fixtures/**` contributes no
// rows from a matching document; the same corpus without the exclusion reports
// the fixture's rows as real.
#[test]
fn tc801_excluded_documents_contribute_no_rows() {
    let bundle = scoped_bundle("801");

    let scoped = report_for(&bundle, "scoped").expect("model declared");
    let scoped_docs: Vec<&str> = scoped.groups.iter().map(|g| g.document.as_str()).collect();
    assert!(
        !scoped_docs.contains(&"fixtures/bad-matrix.md"),
        "the fixture matrix must mint nothing: {scoped_docs:?}"
    );
    assert!(
        !scoped
            .unbacked_rows
            .iter()
            .any(|r| r.document == "fixtures/bad-matrix.md"),
        "an excluded document cannot produce reference rows"
    );
    assert!(
        !scoped
            .status_lies
            .iter()
            .any(|l| l.row_id.as_deref() == Some("TC-900")),
        "TC-900 exists only in the fixture: {:?}",
        scoped.status_lies
    );

    // The control: identical corpus, exclusion removed.
    let unscoped = report_for(&bundle, "unscoped").expect("model declared");
    let unscoped_docs: Vec<&str> = unscoped
        .groups
        .iter()
        .map(|g| g.document.as_str())
        .collect();
    assert!(
        unscoped_docs.contains(&"fixtures/bad-matrix.md"),
        "without the exclusion the fixture is admitted: {unscoped_docs:?}"
    );
    // The phantom the exclusion exists to prevent: a fixture row reusing TC-001
    // reads as backed, because the real test bound that id.
    let phantom = unscoped
        .groups
        .iter()
        .find(|g| g.document == "fixtures/bad-matrix.md")
        .expect("fixture group");
    assert_eq!(
        phantom.backed, 1,
        "the fixture's reused id reads as backed: {phantom:?}"
    );
    assert!(unscoped.totals.total > scoped.totals.total);
}

#[trace("TC-826")]
// a model-level `exclude:` scopes the (FR-050-AC-13 / AC-15, CR-060)
// **criteria walk**, which has no declaration of its own to hang an exclusion
// on and so had none at all before this.
//
// Deliberately malformed fixture data was minting nothing and referencing
// nothing — correct — while still contributing to `criteria` and to both
// totals, inflating the denominator, and being body-parsed despite the
// declaration saying it is not corpus data (agent-ix/quire-rs#124).
#[test]
fn tc826_model_level_exclusion_scopes_the_criteria_walk() {
    let bundle = iso_bundle(
        "826-model-scoped",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &["TC-001"],
    );
    // A fixture FR: same archetype, same grammar binding, deliberately reusing
    // a real AC id and referencing a TC nothing mints.
    write(
        &bundle.scope,
        "fixtures/FR-900.md",
        "---\nid: FR-900\ntype: FR\ntitle: Deliberately malformed fixture\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall always hold it. | Test (TC-666) |\n",
    );

    let scoped = report_for(&bundle, "model-scoped").expect("model declared");
    assert!(
        !scoped
            .criteria
            .iter()
            .any(|c| c.document.contains("fixtures/")),
        "an excluded document contributed criteria: {:?}",
        scoped.criteria
    );
    assert!(
        !scoped
            .groups
            .iter()
            .any(|g| g.document.contains("fixtures/")),
        "and mints no ids: {:?}",
        scoped.groups
    );
    assert!(
        !scoped.to_json().contains("TC-666"),
        "and its references are never read"
    );

    // The exclusion scopes **traceability**, not membership: the fixture is
    // still a document in the corpus, and `validate_bundle` still schema- and
    // grammar-checks it. Being outside the rollup is not a licence to be
    // malformed in ways nobody reports.
    let spec = Spec::from_path(&bundle.scope);
    assert!(
        spec.by_id("FR-900").is_some(),
        "an excluded document is still loaded and still validated"
    );

    // The control: the same corpus under a model declaring no exclusion at all.
    // Every count the fixture inflates is visible here.
    let unscoped = report_for(&bundle, "iso").expect("model declared");
    assert!(
        unscoped
            .criteria
            .iter()
            .any(|c| c.document.contains("fixtures/")),
        "without the exclusion the fixture is counted: {:?}",
        unscoped.criteria
    );
    assert!(
        unscoped.totals.criteria > scoped.totals.criteria,
        "the fixture inflates the criteria denominator: {:?} vs {:?}",
        unscoped.totals,
        scoped.totals
    );
    assert!(
        unscoped.totals.property_shaped > scoped.totals.property_shaped,
        "and the property-shaped numerator with it"
    );
}

#[trace("TC-830", "FR-050-AC-15")]
// ONE archetype-bound entry mints from every (CR-062)
// matrix in the corpus, whatever each one is called. This is what replaced
// enumeration: the retired `document:` form needed an entry per filename and
// still reached nothing nested.
#[test]
fn tc830_one_archetype_entry_mints_from_every_matrix_filename() {
    let bundle = scoped_bundle("830");
    let report = report_for(&bundle, "scoped").expect("model declared");

    let documents: Vec<&str> = report
        .groups
        .iter()
        .filter(|g| g.target == "test-case")
        .map(|g| g.document.as_str())
        .collect();
    // Two different filenames, one declaration — and `scoped` declares exactly
    // one `test-case` entry, so neither is reached by a path.
    assert!(
        documents.contains(&"tests.md"),
        "the canonical filename must mint: {documents:?}"
    );
    assert!(
        documents.contains(&"matrix.md"),
        "a differently-named matrix must mint from the same entry: {documents:?}"
    );

    // Rows from both documents reconcile against the same target kind: TC-003
    // lives in `matrix.md` and nothing binds it.
    assert!(
        report
            .unbacked_rows
            .iter()
            .any(|r| r.document == "matrix.md" && r.row_id.as_deref() == Some("TC-003")),
        "unbacked: {:?}",
        report.unbacked_rows
    );
}

/// A matrix whose rows carry a `Type` column, so a declaration can exempt the
/// methods that mint no source symbol.
fn typed_matrix_bundle(suffix: &str) -> Bundle {
    let bundle = iso_bundle(suffix, &[], &["TC-001"]);
    let mut matrix = String::from(
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n\
         # Test Matrix\n\n## Test Cases\n\n| ID | Type | Traces To | Status |\n\
         |----|------|-----------|--------|\n",
    );
    for (tc, ty, traces, status) in [
        // Backed by a real test: never a lie, exempt or not.
        ("TC-001", "Unit", "FR-001-AC-1", "✅"),
        // Unbacked, and its method could have produced a symbol → a lie.
        ("TC-002", "Unit", "FR-001-AC-2", "✅"),
        // Unbacked, but an agent-behaviour eval has no symbol to tag.
        ("TC-003", "Eval", "FR-001-AC-2", "✅"),
        // Same, by inspection.
        ("TC-004", "Inspection", "FR-001-AC-2", "✅"),
    ] {
        matrix.push_str(&format!("| {tc} | {ty} | {traces} | {status} |\n"));
    }
    write(&bundle.scope, "tests.md", &matrix);
    bundle
}

#[trace("TC-805", "FR-050-AC-16")]
// a row whose declared `Type` names a method that mints
// no source symbol is reported as a no-symbol row rather than a status lie —
// and only when the module declares the vocabulary.
#[test]
fn tc805_no_source_symbol_rows_are_not_status_lies() {
    let bundle = typed_matrix_bundle("805");

    let report = report_for(&bundle, "no-symbol").expect("model declared");
    let lies: Vec<&str> = report
        .status_lies
        .iter()
        .filter_map(|l| l.row_id.as_deref())
        .collect();
    let exempt: Vec<&str> = report
        .no_symbol_rows
        .iter()
        .filter_map(|r| r.row_id.as_deref())
        .collect();

    // The unbacked row whose method could have produced a symbol is still a lie.
    assert_eq!(lies, ["TC-002"], "lies: {lies:?}");
    // The two whose declared method cannot are explained instead.
    assert_eq!(exempt, ["TC-003", "TC-004"], "exempt: {exempt:?}");
    assert_eq!(report.no_symbol_rows[0].test_type, "Eval");
    assert_eq!(report.no_symbol_rows[1].test_type, "Inspection");

    // Exemption changes the verdict, never the facts: every unbacked row is
    // still listed as unbacked, and the counts are untouched.
    let unbacked: Vec<&str> = report
        .unbacked_rows
        .iter()
        .filter_map(|r| r.row_id.as_deref())
        .collect();
    for id in ["TC-002", "TC-003", "TC-004"] {
        assert!(unbacked.contains(&id), "unbacked: {unbacked:?}");
    }
    assert!(!unbacked.contains(&"TC-001"));
}

#[trace("TC-805", "FR-050-AC-16", "FR-050-AC-7")]
// a module declaring no `no_source_symbol`
// vocabulary reports exactly as before — the same rows are lies, and the report
// serializes without the new key.
#[test]
fn tc805_undeclared_vocabulary_changes_nothing() {
    let bundle = typed_matrix_bundle("805-undeclared");
    let report = report_for(&bundle, "iso").expect("model declared");

    let lies: Vec<&str> = report
        .status_lies
        .iter()
        .filter_map(|l| l.row_id.as_deref())
        .collect();
    assert!(
        lies.contains(&"TC-003") && lies.contains(&"TC-004"),
        "without the declaration the eval rows are ordinary lies: {lies:?}"
    );
    assert!(report.no_symbol_rows.is_empty());
    assert!(
        !report.to_json().contains("no_symbol_rows"),
        "an undeclared vocabulary must leave the JSON byte-identical"
    );
}

#[trace("TC-818", "FR-050-AC-18")]
// coverage parses bodies only for the (CR-049)
// archetypes its declared model names. Selection is decided on the header
// tier (frontmatter `type`), never by filename, before any body touch —
// the declaration the engine used to discard now bounds what is parsed.
#[test]
fn tc818_coverage_parses_only_declared_archetype_bodies() {
    let bundle = iso_bundle("818", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    // Documents whose archetypes no trace target, document reference, or
    // grammar binding names: real corpus members, never body-parsed here.
    write(
        &bundle.scope,
        "ADR-001.md",
        "---\nid: ADR-001\ntype: ADR\n---\n# adr\n\n## Context\n\nwords.\n",
    );
    write(
        &bundle.scope,
        "NOTE-001.md",
        "---\nid: NOTE-001\ntype: Note\n---\n# note\n\n## Body\n\nwords.\n",
    );
    // The falsifier (CR-054): an undeclared type in a file NAMED like a
    // declared one. With the undeclared documents also named unlike the
    // declared ones, a filename-driven engine passed this test — the
    // fixture could not tell "decided on the header tier" from "decided by
    // the FR-* filename", which is the claim FR-050-AC-18 actually makes.
    write(
        &bundle.scope,
        "FR-002.md",
        "---\nid: FR-002\ntype: Note\n---\n# note\n\n## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n| FR-002-AC-1 | never minted |\n",
    );

    let registry = Registry::load_module(&fixture_module("iso")).expect("load module");
    let spec = Spec::from_path(&bundle.scope);
    let extraction = extract_tree(&bundle.source);
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = trace::bind(&extraction, &model);
    let report = compute(&spec, &registry, &graph, &bundle.scope).expect("report");
    assert!(report.totals.total > 0, "the declared model must mint");

    // Declared archetype: the FR's body was read — its AC table minted ids.
    assert!(
        spec.by_id("FR-001").expect("FR-001").body_is_parsed(),
        "the declared archetype's body must have been parsed"
    );
    // Undeclared archetypes: bodies stay unparsed through the whole rollup —
    // including the one whose FILENAME matches the declared archetype's.
    for id in ["ADR-001", "NOTE-001", "FR-002"] {
        assert!(
            !spec.by_id(id).expect(id).body_is_parsed(),
            "{id} is not declared by the model; its body was parsed during coverage"
        );
    }
    // And nothing it holds reached the report: a filename-driven engine would
    // have minted its AC row.
    assert!(
        !report.to_json().contains("FR-002"),
        "an undeclared type in an FR-named file must mint nothing: {}",
        report.to_json()
    );
}

#[trace("TC-822", "FR-050-AC-19")]
// a model that loads and (CR-054, amended CR-059)
// selects nothing is reported, not silently accepted. Both shapes reach the
// report: an archetype no document has (a typo in the declaration), and a
// declared auxiliary document that did not resolve to rows — the CR-045
// silent-un-minting class, where the ids simply vanish and the only symptom is
// a distant count. The fixture's document is absent, and this model mints
// nothing, which is when an absence is the thing to check first.
#[test]
fn tc822_declarations_that_select_nothing_are_reported() {
    let bundle = iso_bundle(
        "822-selects-nothing",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &[],
    );
    let report = report_for(&bundle, "fails-open").expect("model declared");

    assert_eq!(
        report.totals.total, 0,
        "the fixture's whole point is that it mints nothing"
    );

    let reasons: Vec<&str> = report
        .diagnostics
        .iter()
        .map(|d| d.reason.as_str())
        .collect();
    assert!(
        reasons.contains(&"archetype-matches-nothing"),
        "an archetype no document has must be reported: {:?}",
        report.diagnostics
    );
    // CR-062: `archetype-matches-nothing` is the only scan diagnostic. The
    // `absent-declared-document` and `unreadable-declared-document` reasons went
    // with the `document:` form — there is no declared path left to be absent,
    // and a minting document that cannot be *read* is now the walk's
    // `DocumentUnreadable`, which is strictly better than the silent `None` the
    // off-corpus reader returned.
    assert_eq!(
        reasons,
        vec![
            "archetype-matches-nothing",
            "archetype-matches-nothing",
            "archetype-matches-nothing"
        ],
        "every declaration selects nothing, each for the same reason: {:?}",
        report.diagnostics
    );

    let archetype = report
        .diagnostics
        .iter()
        .find(|d| d.declaration == "acceptance-criterion")
        .expect("present");
    assert!(
        archetype.message.contains("FRR"),
        "names the archetype nothing has: {}",
        archetype.message
    );

    let target = report
        .diagnostics
        .iter()
        .find(|d| d.declaration == "test-case")
        .expect("present");
    assert!(
        target.message.contains("TestMatrixx"),
        "a misspelled matrix archetype is reported the same way: {}",
        target.message
    );
    assert_eq!(
        target.path, None,
        "a declaration-level fault has no document to point at"
    );

    // Order is a property of the model, not of the walk (NFR-006).
    let again = report_for(&bundle, "fails-open").expect("model declared");
    assert_eq!(report.diagnostics, again.diagnostics);
}

#[trace("TC-822", "FR-050-AC-19")]
// and a model whose declarations all select (CR-054)
// something reports no diagnostics at all — the key is absent from the JSON,
// so FR-050-AC-7 byte-identity holds for every repo without the defect.
#[test]
fn tc822_a_healthy_model_reports_no_diagnostics_and_no_key() {
    let bundle = iso_bundle(
        "822-healthy",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &["TC-001"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    assert!(
        report.diagnostics.is_empty(),
        "a model selecting normally must report nothing: {:?}",
        report.diagnostics
    );
    assert!(
        !report.to_json().contains("diagnostics"),
        "an empty diagnostics list must leave the JSON byte-identical"
    );
}

#[trace("TC-822", "FR-050-AC-19")]
// excluding every document of a declared (CR-054)
// archetype is a deliberate act, not a missing archetype — the count that
// decides is taken before `exclude` applies.
#[test]
fn tc822_excluding_every_match_is_not_a_missing_archetype() {
    let bundle = iso_bundle(
        "822-excluded",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &["TC-001"],
    );
    let report = report_for(&bundle, "scoped").expect("model declared");

    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| d.reason == "archetype-matches-nothing"),
        "an exclusion is not a typo: {:?}",
        report.diagnostics
    );
}
