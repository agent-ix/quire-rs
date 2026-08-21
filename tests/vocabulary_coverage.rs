//! FR-059 — declared-vocabulary coverage (TC-911..TC-916, and the CR-091
//! coverage-payload classification records, TC-962..TC-966).
//!
//! Bundles are built on disk per test so the whole path runs. The vocabulary in
//! every assertion below comes from the fixture module's NFR **frontmatter
//! schema**, never from the manifest and never from this file — swapping the
//! schema swaps the vocabulary with no engine change, which is the property
//! agent-ix/quire-rs#162 turns on.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
use quire_rs::{validate_bundle_at, BundleFinding, BundlePosture, BundleReport, Registry};

fn fixture_module(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability")
        .join(name)
}

fn tmpdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("quire_vocab_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("mkdir");
    p
}

fn write(root: &Path, rel: &str, body: &str) {
    fs::write(root.join(rel), body).expect("write");
}

fn nfr(id: &str, attribute: Option<&str>) -> String {
    let line = match attribute {
        Some(a) => format!("quality_attribute: {a}\n"),
        None => String::new(),
    };
    format!(
        "---\nid: {id}\ntype: NFR\ntitle: A constraint\n{line}---\n\n## Description\n\nIt holds.\n"
    )
}

fn validate(root: &Path, severity: Option<GrammarSeverityMap>) -> BundleReport {
    let registry = Registry::load_module(&fixture_module("vocabulary-coverage")).expect("load");
    let registry = match severity {
        Some(map) => registry.with_grammar_severity(map),
        None => registry,
    };
    validate_bundle_at(root, &registry, BundlePosture::Okf)
}

fn findings<'r>(report: &'r BundleReport, reason: &str) -> Vec<&'r BundleFinding> {
    report
        .errors
        .iter()
        .chain(report.warnings.iter())
        .filter(|f| f.reason == reason)
        .collect()
}

fn unowned(report: &BundleReport) -> Vec<String> {
    findings(report, "unowned-quality-characteristic")
        .iter()
        .filter_map(|f| {
            f.message
                .split('\'')
                .nth(1)
                .map(std::string::ToString::to_string)
        })
        .collect()
}

#[trace("TC-911", "FR-059-AC-1")]
// a declared value no document claims is reported, and a
// claimed one is not.
//
// This is the all-functional-requirements failure mode: a bundle can be 100%
// AC-covered and still carry no requirement anywhere for a whole quality
// characteristic. No per-document check can see it — every document is
// individually fine; what is wrong is the SET.
#[test]
fn tc911_a_value_no_document_claims_is_reported() {
    let root = tmpdir("911");
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));
    write(&root, "NFR-002.md", &nfr("NFR-002", Some("security")));

    let report = validate(&root, None);
    let mut missing = unowned(&report);
    missing.sort();
    assert_eq!(
        missing,
        vec!["safety".to_string(), "usability".to_string()],
        "exactly the two unclaimed values of the four declared"
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-912", "FR-059-AC-2")]
// the vocabulary is READ from the archetype's frontmatter
// schema, not restated in the manifest.
//
// The fixture manifest names an archetype and a field and lists no values at
// all. If the engine were reading a manifest list this test could not pass,
// because there is nothing there to read — and a second list in the manifest is
// exactly the drift CR-015 closed.
#[test]
fn tc912_the_vocabulary_comes_from_the_schema() {
    let manifest = fs::read_to_string(fixture_module("vocabulary-coverage").join("manifest.yaml"))
        .expect("read manifest");
    for value in ["reliability", "security", "usability", "safety"] {
        assert!(
            !manifest.contains(value),
            "the manifest must not restate '{value}' — the schema declares it"
        );
    }

    let root = tmpdir("912");
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));
    let report = validate(&root, None);
    let mut missing = unowned(&report);
    missing.sort();
    assert_eq!(
        missing,
        vec![
            "safety".to_string(),
            "security".to_string(),
            "usability".to_string()
        ],
        "all four schema values are known to the engine"
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-913", "FR-059-AC-3")]
// a written "not applicable" record covers a value.
//
// A check that cannot accept an answer forces one of two bad outcomes: a
// permanent false finding, or a fabricated requirement written to silence it.
// "This is a CLI that controls no physical process, so it has no safety
// characteristic" is a real answer and the bundle should be able to hold it.
#[test]
fn tc913_a_justified_absence_covers_the_value() {
    let root = tmpdir("913");
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));
    write(&root, "NFR-002.md", &nfr("NFR-002", Some("security")));
    write(&root, "NFR-003.md", &nfr("NFR-003", Some("usability")));

    let before = unowned(&validate(&root, None));
    assert_eq!(before, vec!["safety".to_string()]);

    write(
        &root,
        "spec.md",
        "---\nid: SPEC-001\ntype: Spec\ntitle: The product\n\
         quality_attributes_not_applicable:\n  - safety\n---\n\n## Description\n\n\
         A CLI that controls no physical process.\n",
    );
    assert!(
        unowned(&validate(&root, None)).is_empty(),
        "a recorded N/A covers the value"
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-914", "FR-059-AC-4")]
// the justification may live on ANY document, not only on
// one of the archetype being counted.
//
// "This product has no safety characteristic" is a statement about the product.
// Its natural home is the spec or master-requirements document, not one of the
// requirements it is a statement about — and requiring it there would mean
// authoring an NFR to say an NFR is unnecessary.
#[test]
fn tc914_the_justification_need_not_be_on_the_counted_archetype() {
    let root = tmpdir("914");
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));
    write(&root, "NFR-002.md", &nfr("NFR-002", Some("security")));
    write(&root, "NFR-003.md", &nfr("NFR-003", Some("usability")));
    // Type `Spec`, not `NFR` — a different archetype entirely.
    write(
        &root,
        "spec.md",
        "---\nid: SPEC-001\ntype: Spec\ntitle: The product\n\
         quality_attributes_not_applicable:\n  - safety\n---\n\n## Description\n\nx\n",
    );

    assert!(
        unowned(&validate(&root, None)).is_empty(),
        "a Spec document justifies an NFR-projected value"
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-915", "FR-059-AC-5")]
// the finding carries its own `trace:<check>` severity
// key, so a repository tunes it without touching any other corpus check.
#[test]
fn tc915_the_check_is_independently_tunable() {
    let root = tmpdir("915");
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));

    // Advisory by default: findings land as warnings and the bundle stays valid.
    let report = validate(&root, None);
    assert!(!unowned(&report).is_empty());
    assert!(
        findings(&report, "unowned-quality-characteristic")
            .iter()
            .all(|f| !report.errors.contains(*f)),
        "new corpus checks ship advisory"
    );

    let mut off = GrammarSeverityMap::new();
    off.insert(
        "trace:unowned-quality-characteristic".to_string(),
        GrammarSeverityLevel::Off,
    );
    assert!(
        unowned(&validate(&root, Some(off))).is_empty(),
        "mapping the key off drops the finding"
    );

    let mut error = GrammarSeverityMap::new();
    error.insert(
        "trace:unowned-quality-characteristic".to_string(),
        GrammarSeverityLevel::Error,
    );
    let promoted = validate(&root, Some(error));
    assert!(
        !promoted.errors.is_empty(),
        "mapping the key to error promotes it"
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-916", "FR-059-AC-7")]
// a declaration naming a field the schema gives no enum
// reports itself rather than silently covering everything.
//
// The CR-075 failure mode in a second place: with no vocabulary there is
// nothing to be unowned, so the check would pass every bundle forever and look
// exactly like success.
#[test]
fn tc916_a_declaration_with_no_vocabulary_reports_itself() {
    let module = tmpdir("916_module");
    let source = fixture_module("vocabulary-coverage");
    fs::create_dir_all(module.join("schemas")).expect("mkdir");
    fs::copy(
        source.join("schemas/nfr.schema.json"),
        module.join("schemas/nfr.schema.json"),
    )
    .expect("copy schema");
    let manifest = fs::read_to_string(source.join("manifest.yaml"))
        .expect("read")
        .replace("    field: quality_attribute\n", "    field: not_a_field\n");
    fs::write(module.join("manifest.yaml"), manifest).expect("write");

    let root = tmpdir("916");
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));

    let registry = Registry::load_module(&module).expect("a bad field still loads");
    let report = validate_bundle_at(&root, &registry, BundlePosture::Okf);

    assert!(
        unowned(&report).is_empty(),
        "with no vocabulary the check reports no unowned value — which is the danger"
    );
    let dead = findings(&report, "undeclared-coverage-vocabulary");
    assert_eq!(
        dead.len(),
        1,
        "…so the declaration reports itself: {dead:?}"
    );
    assert!(
        dead[0].message.contains("not_a_field") && dead[0].message.contains("NFR"),
        "names the field and the archetype: {}",
        dead[0].message
    );

    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&module).ok();
}

#[trace("TC-917", "FR-059-AC-6")]
// a module declaring no vocabulary coverage produces
// byte-identical output to one that never heard of this FR.
//
// The same guarantee FR-058-AC-8 makes. A new corpus pack that changed the
// report for modules which never opted in would make every existing consumer's
// baseline move on upgrade.
#[test]
fn tc917_a_module_declaring_nothing_sees_no_change() {
    let root = tmpdir("917");
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));
    write(&root, "NFR-002.md", &nfr("NFR-002", None));

    // The required-relations fixture declares relations but no vocabulary
    // coverage — so this FR must contribute nothing to its report.
    let registry = Registry::load_module(&fixture_module("required-relations")).expect("load");
    let report = validate_bundle_at(&root, &registry, BundlePosture::Okf);

    for reason in [
        "unowned-quality-characteristic",
        "undeclared-coverage-vocabulary",
    ] {
        assert!(
            findings(&report, reason).is_empty(),
            "a module declaring no vocabulary_coverage sees no {reason} finding"
        );
    }
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-918", "FR-059-AC-8")]
// an empty projection is ONE finding, not one per value.
//
// Measured over 243 `~/dev` bundles during the FR-059 fit-check: 90 carry no
// NFR document at all. Reporting each of the 12 declared characteristics as
// unowned turned that single fact into 1080 of the sweep's 2792 findings —
// every one saying "no document claims security", "no document claims safety",
// when what is true is "nothing here projects onto this vocabulary".
//
// The count fell to 1802 as a RESULT of reporting the fact at the granularity
// it has. It is not a widening to lower the count: no bundle that was reported
// stops being reported, and the specific statement replaces twelve vaguer ones.
#[test]
fn tc918_an_empty_projection_is_one_finding() {
    let root = tmpdir("918");
    // A bundle with documents, but none of the projected archetype.
    write(
        &root,
        "spec.md",
        "---\nid: SPEC-001\ntype: Spec\ntitle: The product\n---\n\n## Description\n\nx\n",
    );

    let report = validate(&root, None);
    let hits = findings(&report, "unowned-quality-characteristic");
    assert_eq!(
        hits.len(),
        1,
        "one fact, one finding — not one per declared value: {hits:?}"
    );
    assert!(
        hits[0].message.contains("no 'NFR' document projects onto"),
        "states the fact that is true: {}",
        hits[0].message
    );
    // The count is still reported, so nothing is hidden by the collapse.
    assert!(
        hits[0].message.contains('4'),
        "still says how many values are unowned: {}",
        hits[0].message
    );

    // And a bundle WITH the archetype still reports per value.
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));
    assert_eq!(
        findings(&validate(&root, None), "unowned-quality-characteristic").len(),
        3,
        "once something projects, each unowned value is named"
    );
    fs::remove_dir_all(&root).ok();
}

// ─── CR-091 (#179): the coverage payload classifies every declared value ───
//
// The warnings above report only the RESIDUE — the unowned set. A consumer
// (quoin FR-037 is the filed one) also needs to tell an owned value from an
// excused one, and today the only way is to open every document in the bundle
// and parse its frontmatter — a second frontmatter reader in a tool whose whole
// discipline is that quire is the parser. These tests pin the per-value records
// on `quire coverage --json`.

fn coverage_report_for(scope: &Path, registry: &Registry) -> quire_rs::coverage::CoverageReport {
    let src = scope.join("no-source");
    fs::create_dir_all(&src).expect("mkdir");
    let spec = quire_rs::Spec::from_path(scope);
    let model = registry.traceability().cloned().expect("model");
    let graph = quire_rs::symbols::trace::bind(&quire_rs::symbols::extract_tree(&src), &model);
    quire_rs::coverage::compute(&spec, registry, &graph, scope).expect("report")
}

fn coverage_report(scope: &Path) -> quire_rs::coverage::CoverageReport {
    let registry = Registry::load_module(&fixture_module("vocabulary-coverage")).expect("load");
    coverage_report_for(scope, &registry)
}

/// The three-state bundle every classification test reads: `reliability` is
/// claimed, `safety` is excused on a Spec document, `security` and `usability`
/// are neither.
fn classified_bundle(tag: &str) -> PathBuf {
    let root = tmpdir(tag);
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));
    write(
        &root,
        "spec.md",
        "---\nid: SPEC-001\ntype: Spec\ntitle: The product\n\
         quality_attributes_not_applicable:\n  - safety\n---\n\n## Description\n\n\
         A CLI that controls no physical process.\n",
    );
    root
}

#[trace("TC-962", "FR-059-AC-9")]
// a claimed value is an `owned` record naming the claiming
// document — a fact the warning stream never carries, because a covered value
// produces no warning at all.
#[test]
fn tc962_a_claimed_value_is_an_owned_record() {
    let root = classified_bundle("962");
    let report = coverage_report(&root);

    let owned: Vec<_> = report
        .vocabulary_coverage
        .iter()
        .filter(|r| r.state == quire_rs::coverage::VocabularyValueState::Owned)
        .collect();
    assert_eq!(owned.len(), 1, "{:#?}", report.vocabulary_coverage);
    let record = owned[0];
    assert_eq!(record.value, "reliability");
    assert_eq!(record.vocabulary, "quality-characteristics");
    assert_eq!(record.archetype, "NFR");
    assert_eq!(record.field, "quality_attribute");
    assert_eq!(record.check, "unowned-quality-characteristic");
    assert_eq!(
        record.documents,
        vec!["NFR-001.md".to_string()],
        "who owns it, and where, without a second frontmatter reader"
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-963", "FR-059-AC-9")]
// an excused value is an `excused` record naming the
// excusing document. Owned and excused are very different facts — one means a
// requirement exists, the other means somebody wrote the value into the
// justified-absence field and the check went quiet — and the flat warning
// stream collapses them into the same silence.
#[test]
fn tc963_an_excused_value_is_an_excused_record_naming_the_excuser() {
    let root = classified_bundle("963");
    let report = coverage_report(&root);

    let excused: Vec<_> = report
        .vocabulary_coverage
        .iter()
        .filter(|r| r.state == quire_rs::coverage::VocabularyValueState::Excused)
        .collect();
    assert_eq!(excused.len(), 1, "{:#?}", report.vocabulary_coverage);
    assert_eq!(excused[0].value, "safety");
    assert_eq!(
        excused[0].documents,
        vec!["spec.md".to_string()],
        "\"who excused this, and where\" is answerable from the record — the \
         document is the Spec, not the projected archetype"
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-964", "FR-059-AC-9")]
// a value nothing claims and nothing excuses is an
// `unowned` record with no documents, and the record set covers every declared
// value exactly once, in the schema's own order.
#[test]
fn tc964_an_unclaimed_unexcused_value_is_an_unowned_record() {
    let root = classified_bundle("964");
    let report = coverage_report(&root);

    let states: Vec<(&str, &quire_rs::coverage::VocabularyValueState)> = report
        .vocabulary_coverage
        .iter()
        .map(|r| (r.value.as_str(), &r.state))
        .collect();
    use quire_rs::coverage::VocabularyValueState as S;
    assert_eq!(
        states,
        vec![
            ("reliability", &S::Owned),
            ("security", &S::Unowned),
            ("usability", &S::Unowned),
            ("safety", &S::Excused),
        ],
        "one record per declared value, in the schema enum's order"
    );
    for record in report
        .vocabulary_coverage
        .iter()
        .filter(|r| r.state == S::Unowned)
    {
        assert!(
            record.documents.is_empty(),
            "nothing claims it, so nothing to point at: {record:#?}"
        );
    }
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-965", "FR-059-AC-9")]
// a module declaring no `vocabulary_coverage` serializes
// with no `vocabulary_coverage` key at all — the FR-050-AC-7 byte-identity
// every additive payload key before this one has kept.
#[test]
fn tc965_a_module_declaring_nothing_serializes_no_key() {
    let root = tmpdir("965");
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));
    let registry = Registry::load_module(&fixture_module("required-relations")).expect("load");
    let report = coverage_report_for(&root, &registry);

    assert!(report.vocabulary_coverage.is_empty());
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).expect("valid JSON");
    assert!(
        !json
            .as_object()
            .unwrap()
            .contains_key("vocabulary_coverage"),
        "an empty record set must be omitted, not serialized as `[]`"
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-966", "FR-059-AC-10")]
// a declaration whose field yields no enum reports itself
// on the coverage surface too, under the same `undeclared-coverage-vocabulary`
// token the bundle warning uses. Without this, a dead declaration and an
// undeclared one read identically in the payload — no records either way —
// which is the CR-075 silence in a third place.
#[test]
fn tc966_a_dead_declaration_is_a_coverage_diagnostic() {
    let module = tmpdir("966_module");
    let source = fixture_module("vocabulary-coverage");
    fs::create_dir_all(module.join("schemas")).expect("mkdir");
    fs::copy(
        source.join("schemas/nfr.schema.json"),
        module.join("schemas/nfr.schema.json"),
    )
    .expect("copy schema");
    let manifest = fs::read_to_string(source.join("manifest.yaml"))
        .expect("read")
        .replace("    field: quality_attribute\n", "    field: not_a_field\n");
    fs::write(module.join("manifest.yaml"), manifest).expect("write");

    let root = tmpdir("966");
    write(&root, "NFR-001.md", &nfr("NFR-001", Some("reliability")));
    let registry = Registry::load_module(&module).expect("a bad field still loads");
    let report = coverage_report_for(&root, &registry);

    assert!(
        report.vocabulary_coverage.is_empty(),
        "with no vocabulary there is nothing to classify"
    );
    let dead: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.reason == "undeclared-coverage-vocabulary")
        .collect();
    assert_eq!(dead.len(), 1, "{:#?}", report.diagnostics);
    assert_eq!(dead[0].declaration, "quality-characteristics");
    assert!(
        dead[0].message.contains("not_a_field") && dead[0].message.contains("NFR"),
        "names the field and the archetype: {}",
        dead[0].message
    );

    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&module).ok();
}
