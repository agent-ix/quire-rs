//! FR-090 carried fields and FR-091 field-value rules.
//!
//! Every archetype, field, verb and value below is authored in the fixture
//! modules under `tests/fixtures/traceability/field-rules/` and belongs to no
//! real chain: a `Glyph` states a `tier`, a `Bridge` joins it to a `Prism`, and
//! the engine knows none of them. Corpora are built on disk per test so the
//! whole path runs: walk, resolve, then the declared check.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::corpus::{validate_bundle_at, validate_corpus, CorpusMember, Spec};
use quire_rs::coverage::{compute, CoverageReport};
use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
use quire_rs::symbols::{extract_tree, trace as sym_trace};
use quire_rs::{BundleFinding, BundlePosture, BundleReport, Registry};
use serde_json::{json, Value};
use tempfile::TempDir;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability/field-rules")
        .join(name)
}

/// The stating module and the allocating module, in the order given.
fn modules(names: &[&str]) -> Registry {
    let paths: Vec<PathBuf> = names.iter().map(|n| fixture(n)).collect();
    let refs: Vec<&Path> = paths.iter().map(PathBuf::as_path).collect();
    Registry::load_module_set(&refs).expect("modules load")
}

fn both() -> Registry {
    modules(&["glyph", "prism"])
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

/// A document with optional `tier` and `relationships`, plus an Acceptance
/// Criteria table so it yields a coverage-matrix entry.
fn doc(id: &str, ty: &str, extra: &str, rels: &[(&str, &str)]) -> String {
    let mut text = format!("---\nid: {id}\ntype: {ty}\ntitle: {id}\n{extra}");
    if !rels.is_empty() {
        text.push_str("relationships:\n");
        for (verb, target) in rels {
            text.push_str(&format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"));
        }
    }
    text.push_str(&format!(
        "---\n\n# {id}\n\n## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n\
         | {id}-AC-1 | The thing holds. |\n"
    ));
    text
}

fn tier(value: &str) -> String {
    format!("tier: {value}\n")
}

fn glyph(id: &str, value: Option<&str>, rels: &[(&str, &str)]) -> String {
    doc(id, "Glyph", &value.map(tier).unwrap_or_default(), rels)
}

fn prism(id: &str, value: Option<&str>) -> String {
    doc(id, "Prism", &value.map(tier).unwrap_or_default(), &[])
}

/// A bridge that bridges `glyphs` and rests on `prisms`.
fn bridge(id: &str, glyphs: &[&str], prisms: &[&str]) -> String {
    let mut rels: Vec<(&str, &str)> = glyphs.iter().map(|g| ("bridges", *g)).collect();
    rels.extend(prisms.iter().map(|p| ("rests_on", *p)));
    doc(id, "Bridge", "", &rels)
}

fn repo(docs: &[(&str, String)]) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    for (id, text) in docs {
        write(dir.path(), &format!("spec/{id}.md"), text);
    }
    dir
}

fn validate(dir: &TempDir, registry: &Registry) -> BundleReport {
    validate_bundle_at(dir.path(), registry, BundlePosture::Okf)
}

fn found<'r>(report: &'r BundleReport, reason: &str) -> Vec<&'r BundleFinding> {
    report
        .errors
        .iter()
        .chain(&report.warnings)
        .filter(|f| f.reason == reason)
        .collect()
}

fn coverage(dir: &TempDir, registry: &Registry) -> CoverageReport {
    let spec = Spec::from_path(&dir.path().join("spec"));
    let extraction = extract_tree(&dir.path().join("src"));
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = sym_trace::bind(&extraction, &model);
    compute(&spec, registry, &graph, dir.path()).expect("model declared")
}

/// The glyph module with `carried_fields` removed: the "before" the specs
/// compare against. (The schema reference goes too, as nothing reads it.)
fn bare_glyph() -> (TempDir, Registry) {
    let dir = TempDir::new().expect("tempdir");
    let manifest = fs::read_to_string(fixture("glyph").join("manifest.yaml")).expect("read");
    let (without, _) = manifest
        .split_once("  carried_fields:")
        .expect("fixture declares carried_fields");
    write(
        dir.path(),
        "manifest.yaml",
        &without.replace("  frontmatter_schema_ref: schemas/glyph.schema.json\n", ""),
    );
    let registry = Registry::load_module(dir.path()).expect("loads");
    (dir, registry)
}

/// A coverage payload with the FR-090 keys removed.
fn without_carried_keys(report: &CoverageReport) -> Value {
    let mut json = serde_json::to_value(report).expect("serializes");
    json.as_object_mut()
        .expect("object")
        .remove("carried_vocabulary");
    for requirement in json["coverage_matrix"].as_array_mut().into_iter().flatten() {
        requirement
            .as_object_mut()
            .expect("object")
            .remove("carried");
    }
    json
}

fn entry(report: &CoverageReport, document: &str) -> Value {
    serde_json::to_value(report).expect("serializes")["coverage_matrix"]
        .as_array()
        .expect("coverage_matrix")
        .iter()
        .find(|e| e["document"] == document)
        .unwrap_or_else(|| panic!("no entry for {document}"))
        .clone()
}

/// One bridged, consistent-or-not pair: GLY-001 -> BRG-001 -> PRI-001.
fn pair(glyph_tier: &str, prism_tier: &str) -> TempDir {
    repo(&[
        ("GLY-001", glyph("GLY-001", Some(glyph_tier), &[])),
        ("BRG-001", bridge("BRG-001", &["GLY-001"], &["PRI-001"])),
        ("PRI-001", prism("PRI-001", Some(prism_tier))),
    ])
}

// ── FR-090 ────────────────────────────────────────────────────────────────

#[trace("FR-090-AC-1", "FR-090-AC-2", "FR-090-AC-3", "FR-090-AC-6")]
#[test]
fn a_stated_value_is_carried_as_written_and_nothing_else_is() {
    let dir = repo(&[
        ("GLY-001", glyph("GLY-001", Some("soft"), &[])),
        ("GLY-002", glyph("GLY-002", None, &[])),
        ("GLY-003", doc("GLY-003", "Glyph", "tier: null\n", &[])),
        // Outside the enum: carried as written, left to schema validation.
        ("GLY-004", glyph("GLY-004", Some("glowing"), &[])),
        // Related to a document that states a value; infers nothing from it.
        (
            "GLY-005",
            glyph(
                "GLY-005",
                None,
                &[("traces_to", "GLY-001"), ("refines", "GLY-001")],
            ),
        ),
    ]);
    let report = coverage(&dir, &modules(&["glyph"]));

    let first = entry(&report, "spec/GLY-001.md");
    assert_eq!(first["carried"], json!({"tier": "soft"}));
    // No new top-level key on the entry.
    let mut keys: Vec<&String> = first.as_object().expect("object").keys().collect();
    keys.sort();
    assert_eq!(keys, ["carried", "criteria", "document"]);

    for unstated in ["GLY-002", "GLY-003", "GLY-005"] {
        let entry = entry(&report, &format!("spec/{unstated}.md"));
        assert!(entry.get("carried").is_none(), "{unstated}: {entry}");
    }
    assert_eq!(
        entry(&report, "spec/GLY-004.md")["carried"],
        json!({"tier": "glowing"})
    );
}

#[trace("FR-090-AC-4")]
#[test]
fn carried_and_criticality_are_independent_and_obligations_are_untouched() {
    let docs = [
        (
            "GLY-001",
            doc("GLY-001", "Glyph", "criticality: high\n", &[]),
        ),
        ("GLY-002", doc("GLY-002", "Glyph", "tier: dim\n", &[])),
        (
            "GLY-003",
            doc("GLY-003", "Glyph", "criticality: low\ntier: bright\n", &[]),
        ),
    ];
    let dir = repo(&docs);
    let with = coverage(&dir, &modules(&["glyph"]));

    let only_criticality = entry(&with, "spec/GLY-001.md");
    assert_eq!(only_criticality["criticality"], "high");
    assert!(only_criticality.get("carried").is_none());
    let only_carried = entry(&with, "spec/GLY-002.md");
    assert_eq!(only_carried["carried"], json!({"tier": "dim"}));
    assert!(only_carried.get("criticality").is_none());
    let both = entry(&with, "spec/GLY-003.md");
    assert_eq!(both["criticality"], "low");
    assert_eq!(both["carried"], json!({"tier": "bright"}));

    // The same module without `carried_fields`: obligation records identical.
    let (_bare, registry) = bare_glyph();
    let without = coverage(&dir, &registry);
    assert_eq!(
        serde_json::to_string(&with.obligations).expect("serializes"),
        serde_json::to_string(&without.obligations).expect("serializes")
    );
}

#[trace("FR-090-AC-5")]
#[test]
fn a_module_without_carried_fields_emits_neither_key() {
    let dir = repo(&[("GLY-001", glyph("GLY-001", Some("soft"), &[]))]);
    let bare = TempDir::new().expect("tempdir");
    write(
        bare.path(),
        "manifest.yaml",
        "name: bare\nversion: 0.1.0\ndescription: x\nartifact_types:\n- name: Glyph\n  \
         defaults:\n    id_pattern: GLY-{next:03d}\ntraceability:\n  trace_targets:\n  \
         - name: glyph-check\n    archetype: Glyph\n    section: Acceptance Criteria\n    \
         id_column: ID\n  obligations:\n  - name: glyph-check\n    target: glyph-check\n    \
         statement_column: Criteria\n",
    );
    let report = coverage(&dir, &Registry::load_module(bare.path()).expect("loads"));
    let json = serde_json::to_value(&report).expect("serializes");
    assert!(!json["coverage_matrix"]
        .as_array()
        .expect("matrix")
        .is_empty());
    assert!(!json.to_string().contains("carried"));
}

fn bad_module(entry: &str) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    write(
        dir.path(),
        "manifest.yaml",
        &format!(
            "name: bad\nversion: 0.1.0\ndescription: x\nartifact_types:\n- name: Glyph\n  \
             defaults:\n    id_pattern: GLY-{{next:03d}}\ntraceability:\n  carried_fields:\n{entry}"
        ),
    );
    dir
}

#[trace("FR-090-AC-7")]
#[test]
fn a_structurally_bad_carried_field_fails_the_declaring_module_naming_it() {
    for (entry, named) in [
        ("  - name: Tier\n    from: [Glyph]\n    key: tier\n", "Tier"),
        (
            "  - name: 'bad name'\n    from: [Glyph]\n    key: tier\n",
            "bad name",
        ),
        ("  - name: tier\n    from: [Glyph]\n    key: '  '\n", "tier"),
        ("  - name: tier\n    from: []\n    key: tier\n", "tier"),
    ] {
        let dir = bad_module(entry);
        let err = Registry::load_module_strict(dir.path())
            .err()
            .unwrap_or_else(|| panic!("{entry} must fail module load"));
        assert!(err.to_string().contains(named), "{entry}: {err}");
        assert!(err.to_string().contains("carried_fields"), "{entry}: {err}");
    }
}

#[trace("FR-090-AC-8")]
#[test]
fn an_entry_naming_nothing_reports_only_when_the_corpus_is_merged() {
    let dir = repo(&[("PRI-001", prism("PRI-001", None))]);
    // Prism alone: the key is a property of no `from` schema and no document
    // carries it. The module still loads.
    let alone = modules(&["prism"]);
    let report = validate(&dir, &alone);
    let dead = found(&report, "undeclared-relation-vocabulary");
    assert!(
        dead.iter()
            .any(|f| f.message.contains("carried_fields entry 'tier'")),
        "{dead:?}"
    );

    // Beside the module that declares Glyph, the merged entry is live.
    let merged = validate(&dir, &both());
    assert!(
        found(&merged, "undeclared-relation-vocabulary")
            .iter()
            .all(|f| !f.message.contains("carried_fields entry")),
        "{merged:?}"
    );

    // An archetype no loaded module declares.
    let ghost = bad_module("  - name: tier\n    from: [Ghost]\n    key: tier\n");
    let registry = Registry::load_module(ghost.path()).expect("loads");
    let report = validate(&dir, &registry);
    assert!(
        found(&report, "undeclared-relation-vocabulary")
            .iter()
            .any(|f| f.message.contains("'Ghost'") && f.message.contains("'tier'")),
        "{report:?}"
    );
}

#[trace("FR-090-AC-9", "FR-090-AC-13")]
#[test]
fn the_vocabulary_is_the_enum_in_schema_order_whichever_module_loads_first() {
    let dir = repo(&[
        ("GLY-001", glyph("GLY-001", Some("dim"), &[])),
        ("PRI-001", prism("PRI-001", Some("soft"))),
    ]);
    let expected = json!({"tier": ["dim", "soft", "bright"]});
    for order in [["glyph", "prism"], ["prism", "glyph"]] {
        let report = coverage(&dir, &modules(&order));
        let json = serde_json::to_value(&report).expect("serializes");
        assert_eq!(json["carried_vocabulary"], expected, "{order:?}");
        assert!(json["coverage_matrix"].is_array());
        assert!(found(
            &validate(&dir, &modules(&order)),
            "carried-vocabulary-mismatch"
        )
        .is_empty());
    }
    // A key with no `enum` in any `from` schema has no entry.
    let json = serde_json::to_value(coverage(&dir, &modules(&["prism"]))).expect("serializes");
    assert!(json.get("carried_vocabulary").is_none());
}

#[trace("FR-090-AC-13")]
#[test]
fn two_enums_that_differ_are_reported_and_yield_no_vocabulary() {
    let dir = repo(&[("GLY-001", glyph("GLY-001", Some("dim"), &[]))]);
    for order in [["glyph", "glint"], ["glint", "glyph"]] {
        let registry = modules(&order);
        let report = validate(&dir, &registry);
        let hits = found(&report, "carried-vocabulary-mismatch");
        assert_eq!(hits.len(), 1, "{order:?}: {hits:?}");
        for named in [
            "'tier'",
            "'Glyph'",
            "'Glint'",
            "field-rules-glyph",
            "field-rules-glint",
        ] {
            assert!(
                hits[0].message.contains(named),
                "{named}: {}",
                hits[0].message
            );
        }
        // The message is the same whichever module loaded first.
        let first = found(
            &validate(&dir, &modules(&["glyph", "glint"])),
            "carried-vocabulary-mismatch",
        )[0]
        .message
        .clone();
        assert_eq!(hits[0].message, first);
        let json = serde_json::to_value(coverage(&dir, &registry)).expect("serializes");
        assert!(json.get("carried_vocabulary").is_none(), "{json}");
    }
}

#[trace("FR-090-AC-10")]
#[test]
fn same_name_and_key_merge_to_a_union_and_a_different_key_fails_the_merge() {
    for order in [["glyph", "prism"], ["prism", "glyph"]] {
        let registry = modules(&order);
        let model = registry.traceability().expect("declared");
        assert_eq!(model.carried_fields.len(), 1);
        let mut from = model.carried_fields[0].from.clone();
        from.sort();
        assert_eq!(from, ["Glyph", "Prism"], "{order:?}");
    }

    // Both declaring modules fail, each naming the other, and the contradictory
    // model is not exposed.
    let registry =
        Registry::load_module_set(&[fixture("glyph").as_path(), fixture("rival").as_path()])
            .expect("loads tolerantly");
    assert!(registry.traceability().is_none(), "no contradictory model");
    let failures = registry.failures();
    assert_eq!(failures.len(), 2, "{failures:?}");
    for module in ["field-rules-glyph", "field-rules-rival"] {
        assert!(failures.iter().any(|f| f.module == module), "{module}");
    }
    for failure in failures {
        for named in ["tier", "field-rules-glyph", "field-rules-rival", "level"] {
            assert!(
                failure.reason.contains(named),
                "{named}: {}",
                failure.reason
            );
        }
    }
}

#[trace("FR-090-AC-11")]
#[test]
fn the_published_schema_admits_carried_and_carried_vocabulary_and_nothing_else() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas/output/coverage-v1.schema.json");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read")).expect("json");
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema)
        .expect("compiles");

    let dir = repo(&[("GLY-001", glyph("GLY-001", Some("soft"), &[]))]);
    let payload = serde_json::to_value(coverage(&dir, &both())).expect("serializes");
    assert!(payload["carried_vocabulary"].is_object());
    assert!(payload["coverage_matrix"][0]["carried"].is_object());
    let errors: Vec<String> = validator
        .iter_errors(&payload)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");

    let mut stray_entry = payload.clone();
    stray_entry["coverage_matrix"][0]["inferred"] = json!("x");
    assert!(!validator.is_valid(&stray_entry));
    let mut stray_report = payload;
    stray_report["inferred"] = json!("x");
    assert!(!validator.is_valid(&stray_report));
}

#[trace("FR-090-AC-12", "FR-091-AC-9")]
#[test]
fn running_leaves_documents_byte_identical_and_serializes_identically() {
    let dir = repo(&[
        ("GLY-001", glyph("GLY-001", Some("dim"), &[])),
        ("BRG-001", bridge("BRG-001", &["GLY-001"], &["PRI-001"])),
        ("PRI-001", prism("PRI-001", Some("bright"))),
        ("PRI-002", prism("PRI-002", None)),
    ]);
    let snapshot = |dir: &TempDir| -> Vec<(PathBuf, Vec<u8>)> {
        let mut files: Vec<_> = fs::read_dir(dir.path().join("spec"))
            .expect("read dir")
            .map(|e| e.expect("entry").path())
            .map(|p| (p.clone(), fs::read(&p).expect("read")))
            .collect();
        files.sort();
        files
    };
    let before = snapshot(&dir);
    let registry = both();
    let first = serde_json::to_string(&coverage(&dir, &registry)).expect("serializes");
    let second = serde_json::to_string(&coverage(&dir, &registry)).expect("serializes");
    let report = validate(&dir, &registry);
    assert_eq!(first, second);
    // A run without the keys serializes identically to itself, and equals the
    // run with them once the two keys are removed.
    let (_bare, bare) = bare_glyph();
    let bare_one = serde_json::to_string(&coverage(&dir, &bare)).expect("serializes");
    assert_eq!(
        bare_one,
        serde_json::to_string(&coverage(&dir, &bare)).expect("serializes")
    );
    assert_eq!(
        without_carried_keys(&coverage(&dir, &registry)),
        without_carried_keys(&coverage(&dir, &bare))
    );
    assert_eq!(before, snapshot(&dir), "no document was written");
    assert!(!report.warnings.is_empty(), "the corpus has findings");

    // No output carries a tier no document states.
    let stated = ["dim", "bright"];
    let json: Value = serde_json::from_str(&first).expect("json");
    for requirement in json["coverage_matrix"].as_array().expect("matrix") {
        if let Some(carried) = requirement.get("carried") {
            let value = carried["tier"].as_str().expect("string");
            assert!(stated.contains(&value), "{value}");
        }
    }
}

// ── FR-091 field_consistency ──────────────────────────────────────────────

#[trace("FR-091-AC-1")]
#[test]
fn a_lower_value_than_the_allocated_target_names_every_document_and_both_values() {
    let dir = pair("dim", "bright");
    let report = validate(&dir, &both());
    let hits = found(&report, "tier-below-prism");
    assert_eq!(hits.len(), 1, "{hits:?}");
    let message = &hits[0].message;
    for named in ["GLY-001", "BRG-001", "PRI-001", "'dim'", "'bright'", "tier"] {
        assert!(message.contains(named), "{named}: {message}");
    }
    assert!(hits[0].pack.is_some());
    assert!(report.errors.is_empty(), "advisory by default: {report:?}");
}

#[trace("FR-091-AC-2")]
#[test]
fn an_equal_or_higher_value_than_the_target_is_not_reported() {
    for (from, to) in [("bright", "dim"), ("soft", "soft"), ("bright", "soft")] {
        let report = validate(&pair(from, to), &both());
        assert!(found(&report, "tier-below-prism").is_empty(), "{from} {to}");
    }
}

#[trace("FR-091-AC-3")]
#[test]
fn a_pair_where_either_side_states_nothing_is_not_compared() {
    let dir = repo(&[
        ("GLY-001", glyph("GLY-001", None, &[])),
        (
            "BRG-001",
            bridge("BRG-001", &["GLY-001", "GLY-002"], &["PRI-001", "PRI-002"]),
        ),
        ("GLY-002", glyph("GLY-002", Some("dim"), &[])),
        ("PRI-001", prism("PRI-001", Some("bright"))),
        ("PRI-002", prism("PRI-002", None)),
    ]);
    let report = validate(&dir, &both());
    let hits = found(&report, "tier-below-prism");
    // Only GLY-002 -> PRI-001 has both values.
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].message.contains("GLY-002") && hits[0].message.contains("PRI-001"));
}

#[trace("FR-091-AC-4")]
#[test]
fn each_outranking_target_is_one_finding_and_routes_to_one_target_are_one_pair() {
    let dir = repo(&[
        ("GLY-001", glyph("GLY-001", Some("soft"), &[])),
        (
            "BRG-001",
            bridge("BRG-001", &["GLY-001"], &["PRI-001", "PRI-002"]),
        ),
        // A second route to PRI-001. Its file name sorts AFTER BRG-001's while
        // its id sorts before, so walk order and id order disagree.
        ("ZZ-file", bridge("BRG-000", &["GLY-001"], &["PRI-001"])),
        ("PRI-001", prism("PRI-001", Some("bright"))),
        ("PRI-002", prism("PRI-002", Some("dim"))),
    ]);
    let report = validate(&dir, &both());
    let hits = found(&report, "tier-below-prism");
    assert_eq!(hits.len(), 1, "only the outranking part: {hits:?}");
    let message = &hits[0].message;
    assert!(message.contains("PRI-001") && !message.contains("PRI-002"));
    let (a, b) = (message.find("BRG-000"), message.find("BRG-001"));
    assert!(
        a.is_some() && b.is_some() && a < b,
        "lexical via order: {message}"
    );
}

#[trace("FR-091-AC-7")]
#[test]
fn a_value_outside_the_ranking_enum_is_reported_once_and_not_compared() {
    // Outside on the from side.
    let report = validate(&pair("glowing", "bright"), &both());
    let hits = found(&report, "undeclared-field-value");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].message.contains("GLY-001") && hits[0].message.contains("glowing"));
    assert!(found(&report, "tier-below-prism").is_empty());

    // Outside on the target side; the other side would otherwise be below.
    let report = validate(&pair("dim", "blazing"), &both());
    let hits = found(&report, "undeclared-field-value");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].message.contains("PRI-001") && hits[0].message.contains("blazing"));
    assert!(found(&report, "tier-below-prism").is_empty());

    // Once per document, however many pairs it takes part in.
    let dir = repo(&[
        ("GLY-001", glyph("GLY-001", Some("glowing"), &[])),
        (
            "BRG-001",
            bridge("BRG-001", &["GLY-001"], &["PRI-001", "PRI-002"]),
        ),
        ("PRI-001", prism("PRI-001", Some("bright"))),
        ("PRI-002", prism("PRI-002", Some("bright"))),
    ]);
    assert_eq!(
        found(&validate(&dir, &both()), "undeclared-field-value").len(),
        1
    );
}

#[trace("FR-091-AC-12")]
#[test]
fn an_inverse_authored_edge_and_a_pair_authored_both_ways_are_each_compared_once() {
    // The glyph authors the declared inverse toward the bridge; the bridge
    // authors nothing toward the glyph.
    let inverse = repo(&[
        (
            "GLY-001",
            glyph("GLY-001", Some("dim"), &[("bridged_by", "BRG-001")]),
        ),
        (
            "BRG-001",
            doc("BRG-001", "Bridge", "", &[("rests_on", "PRI-001")]),
        ),
        ("PRI-001", prism("PRI-001", Some("bright"))),
    ]);
    assert_eq!(
        found(&validate(&inverse, &both()), "tier-below-prism").len(),
        1
    );
    // Authored both ways.
    let twice = repo(&[
        (
            "GLY-001",
            glyph("GLY-001", Some("dim"), &[("bridged_by", "BRG-001")]),
        ),
        (
            "BRG-001",
            doc(
                "BRG-001",
                "Bridge",
                "",
                &[("bridges", "GLY-001"), ("rests_on", "PRI-001")],
            ),
        ),
        ("PRI-001", prism("PRI-001", Some("bright"))),
    ]);
    assert_eq!(
        found(&validate(&twice, &both()), "tier-below-prism").len(),
        1
    );
}

fn cross_member_report(second: Option<&TempDir>) -> BundleReport {
    let a = repo(&[
        ("GLY-001", glyph("GLY-001", Some("dim"), &[])),
        (
            "BRG-001",
            doc(
                "BRG-001",
                "Bridge",
                "",
                &[
                    ("bridges", "GLY-001"),
                    ("rests_on", "ix://o/b/spec/PRI-001"),
                ],
            ),
        ),
    ]);
    let member = |dir: &TempDir, identity: &str| CorpusMember {
        identity: Some(identity.to_string()),
        load: quire_rs::corpus::walk::load_repo(&dir.path().join("spec")),
    };
    let mut members = vec![member(&a, "o/a")];
    let mut dirs = vec![&a];
    if let Some(b) = second {
        members.push(member(b, "o/b"));
        dirs.push(b);
    }
    let corpus = Spec::from_members(members).expect("corpus");
    let roots: Vec<(PathBuf, PathBuf)> = dirs
        .iter()
        .map(|d| (d.path().join("spec"), d.path().to_path_buf()))
        .collect();
    let refs: Vec<(&Path, &Path)> = roots
        .iter()
        .map(|(d, r)| (d.as_path(), r.as_path()))
        .collect();
    validate_corpus(&corpus, &both(), BundlePosture::Okf, &refs).expect("validates")
}

fn cross_member(second: Option<&TempDir>) -> usize {
    found(&cross_member_report(second), "tier-below-prism").len()
}

#[trace("FR-091-AC-7")]
#[test]
fn an_out_of_enum_target_in_another_member_is_reported_once_by_its_own_member() {
    let b = repo(&[("PRI-001", prism("PRI-001", Some("blazing")))]);
    let report = cross_member_report(Some(&b));
    let hits = found(&report, "undeclared-field-value");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].repo.as_deref(), Some("o/b"));
    assert!(hits[0].path.starts_with(b.path()), "{:?}", hits[0].path);
    assert!(found(&report, "tier-below-prism").is_empty());
}

#[trace("FR-091-AC-13")]
#[test]
fn dangling_wrong_kind_and_unloaded_targets_join_nothing_and_a_loaded_member_does() {
    // Dangling target.
    let dangling = repo(&[
        ("GLY-001", glyph("GLY-001", Some("dim"), &[])),
        ("BRG-001", bridge("BRG-001", &["GLY-001"], &["PRI-404"])),
    ]);
    assert!(found(&validate(&dangling, &both()), "tier-below-prism").is_empty());

    // Target of a kind outside `to`: a Pier states a high tier.
    let wrong_kind = repo(&[
        ("GLY-001", glyph("GLY-001", Some("dim"), &[])),
        ("BRG-001", bridge("BRG-001", &["GLY-001"], &["PIR-001"])),
        ("PIR-001", doc("PIR-001", "Pier", &tier("bright"), &[])),
    ]);
    assert!(found(&validate(&wrong_kind, &both()), "tier-below-prism").is_empty());

    // A target in a member that is not loaded, then in one that is.
    assert_eq!(cross_member(None), 0);
    let b = repo(&[("PRI-001", prism("PRI-001", Some("bright")))]);
    assert_eq!(cross_member(Some(&b)), 1);
}

#[trace("FR-091-AC-14")]
#[test]
fn findings_are_ordered_by_check_path_via_and_target_whatever_the_read_order() {
    let build = |order: &[&str]| {
        let mut docs = vec![
            ("GLY-001", glyph("GLY-001", Some("dim"), &[])),
            ("GLY-002", glyph("GLY-002", Some("soft"), &[])),
            (
                "BRG-001",
                bridge(
                    "BRG-001",
                    &["GLY-001", "GLY-002"],
                    &["PRI-002", "PRI-001", "PRI-003"],
                ),
            ),
            ("PRI-001", prism("PRI-001", Some("bright"))),
            ("PRI-002", prism("PRI-002", None)),
            ("PRI-003", prism("PRI-003", Some("nonsense"))),
        ];
        docs.sort_by_key(|(id, _)| order.iter().position(|o| o == id));
        repo(&docs)
    };
    let forward = [
        "GLY-001", "GLY-002", "BRG-001", "PRI-001", "PRI-002", "PRI-003",
    ];
    let mut backward = forward;
    backward.reverse();
    let registry = both();
    let sequence = |report: &BundleReport| -> Vec<(String, String)> {
        report
            .warnings
            .iter()
            .filter(|f| {
                [
                    "tier-below-prism",
                    "unassigned-tier",
                    "undeclared-tier-value",
                    "undeclared-field-value",
                ]
                .contains(&f.reason.as_ref())
            })
            .map(|f| (f.reason.to_string(), f.message.clone()))
            .collect()
    };
    let one = sequence(&validate(&build(&forward), &registry));
    let two = sequence(&validate(&build(&backward), &registry));
    assert_eq!(one, two);
    let checks: Vec<&str> = one.iter().map(|(c, _)| c.as_str()).collect();
    let mut sorted = checks.clone();
    sorted.sort_unstable();
    assert_eq!(checks, sorted, "ordered by check first");
    // The pair that reports an out-of-enum value is emitted before the pair
    // that reports a lower value, so emission order alone is not report order.
    assert!(checks.contains(&"undeclared-field-value") && checks.contains(&"tier-below-prism"));
    let below: Vec<&String> = one
        .iter()
        .filter(|(c, _)| c == "tier-below-prism")
        .map(|(_, m)| m)
        .collect();
    assert!(
        below[0].contains("GLY-001") && below[1].contains("GLY-002"),
        "{below:?}"
    );
}

#[trace("FR-091-AC-15", "FR-091-CON-1")]
#[test]
fn neutral_names_yield_the_same_findings_so_the_engine_holds_none() {
    let neutral = |id: &str, ty: &str, rank: Option<&str>, rels: &[(&str, &str)]| {
        doc(
            id,
            ty,
            &rank
                .map(|r| format!("rank_field: {r}\n"))
                .unwrap_or_default(),
            rels,
        )
    };
    let dir = repo(&[
        (
            "AL-001",
            neutral("AL-001", "alpha", Some("r1"), &[("links_to", "BE-001")]),
        ),
        (
            "AL-002",
            neutral("AL-002", "alpha", Some("r4"), &[("links_to", "BE-001")]),
        ),
        ("BE-001", neutral("BE-001", "beta", Some("r3"), &[])),
        ("BE-002", neutral("BE-002", "beta", Some("r9"), &[])),
        ("GA-001", neutral("GA-001", "gamma", Some("r1"), &[])),
    ]);
    let report = validate(&dir, &modules(&["neutral"]));
    let below = found(&report, "rank-below");
    assert_eq!(below.len(), 1, "{below:?}");
    assert!(below[0].message.contains("AL-001") && below[0].message.contains("BE-001"));
    let outside = found(&report, "rank-outside");
    assert_eq!(outside.len(), 1, "{outside:?}");
    assert!(outside[0].message.contains("BE-002") && outside[0].message.contains("r9"));
}

fn temp_module(manifest: &str) -> (TempDir, Registry) {
    let dir = TempDir::new().expect("tempdir");
    write(dir.path(), "manifest.yaml", manifest);
    let registry = Registry::load_module(dir.path()).expect("loads");
    (dir, registry)
}

#[trace("FR-091-AC-16")]
#[test]
fn an_order_from_field_with_no_enum_reports_and_compares_nothing() {
    let (_dir, registry) = temp_module(
        "name: no-enum\nversion: 0.1.0\ndescription: x\n\
         artifact_types:\n- name: alpha\n  defaults:\n    id_pattern: AL-{next:03d}\n\
         - name: beta\n  defaults:\n    id_pattern: BE-{next:03d}\n\
         traceability:\n  field_consistency:\n  - name: unranked\n    from: [alpha]\n    \
         field: rank_field\n    edges: [links_to]\n    direction: outgoing\n    to: [beta]\n    \
         to_field: rank_field\n    order_from: { archetype: alpha, field: rank_field }\n    \
         rule: not_below\n    check: rank-below\n",
    );
    let docs = repo(&[
        (
            "AL-001",
            doc(
                "AL-001",
                "alpha",
                "rank_field: r1\n",
                &[("links_to", "BE-001")],
            ),
        ),
        ("BE-001", doc("BE-001", "beta", "rank_field: r2\n", &[])),
    ]);
    let report = validate(&docs, &registry);
    let hits = found(&report, "undeclared-relation-vocabulary");
    assert!(
        hits.iter().any(|f| f.message.contains("'unranked'")),
        "{hits:?}"
    );
    assert!(found(&report, "rank-below").is_empty());
}

// ── FR-091 required_fields and vocabulary ─────────────────────────────────

#[trace("FR-091-AC-5")]
#[test]
fn a_part_stating_no_value_is_reported_once_naming_it_and_the_field() {
    let dir = repo(&[
        ("PRI-001", prism("PRI-001", None)),
        ("PRI-002", prism("PRI-002", Some("soft"))),
        ("PRI-003", doc("PRI-003", "Prism", "tier: null\n", &[])),
        ("PRI-004", doc("PRI-004", "Prism", "tier: ''\n", &[])),
    ]);
    let report = validate(&dir, &both());
    let hits = found(&report, "unassigned-tier");
    let named: Vec<bool> = ["PRI-001", "PRI-003", "PRI-004"]
        .iter()
        .map(|id| hits.iter().filter(|f| f.message.contains(id)).count() == 1)
        .collect();
    assert_eq!(hits.len(), 3, "{hits:?}");
    assert!(named.iter().all(|n| *n), "{hits:?}");
    assert!(hits.iter().all(|f| f.message.contains("`tier`")));
    assert!(hits.iter().all(|f| !f.message.contains("PRI-002")));
}

#[trace("FR-091-AC-6")]
#[test]
fn a_module_declaring_none_of_the_three_yields_no_finding() {
    let dir = repo(&[
        ("GLY-001", glyph("GLY-001", Some("dim"), &[])),
        ("PRI-001", prism("PRI-001", None)),
    ]);
    // Glyph alone declares carried fields but none of the three rules.
    let report = validate(&dir, &modules(&["glyph"]));
    for reason in [
        "tier-below-prism",
        "unassigned-tier",
        "undeclared-tier-value",
        "undeclared-field-value",
    ] {
        assert!(found(&report, reason).is_empty(), "{reason}");
    }
    // Output identical to the same module without the declarations.
    let (_bare, bare) = bare_glyph();
    assert_eq!(report, validate(&dir, &bare));
    let with = coverage(&dir, &modules(&["glyph"]));
    let without = coverage(&dir, &bare);
    assert_eq!(without_carried_keys(&with), without_carried_keys(&without));
    assert!(found(&validate(&dir, &bare), "undeclared-relation-vocabulary").is_empty());
}

#[trace("FR-091-AC-8")]
#[test]
fn findings_are_advisory_tunable_and_a_module_may_ship_a_check_off() {
    let dir = repo(&[("PRI-001", prism("PRI-001", None))]);
    let registry = both();
    let report = validate(&dir, &registry);
    assert_eq!(found(&report, "unassigned-tier").len(), 1);
    assert!(report.errors.is_empty());

    let mut map = GrammarSeverityMap::new();
    map.insert(
        "trace:unassigned-tier".to_string(),
        GrammarSeverityLevel::Off,
    );
    let off = validate(&dir, &registry.with_grammar_severity(map));
    assert!(found(&off, "unassigned-tier").is_empty());

    let mut map = GrammarSeverityMap::new();
    map.insert(
        "trace:unassigned-tier".to_string(),
        GrammarSeverityLevel::Error,
    );
    let promoted = validate(&dir, &registry.with_grammar_severity(map));
    assert_eq!(promoted.errors.len(), 1);

    // The module's own default: shipped off, so a project enables it.
    let shipped = validate(&dir, &modules(&["quiet"]));
    assert!(found(&shipped, "unassigned-tier").is_empty());
    let mut map = GrammarSeverityMap::new();
    map.insert(
        "trace:unassigned-tier".to_string(),
        GrammarSeverityLevel::Warning,
    );
    let enabled = validate(&dir, &modules(&["quiet"]).with_grammar_severity(map));
    assert_eq!(found(&enabled, "unassigned-tier").len(), 1);
}

#[trace("FR-091-AC-17")]
#[test]
fn a_value_outside_the_carried_vocabulary_is_reported_exactly() {
    let dir = repo(&[
        ("PRI-001", prism("PRI-001", Some("class-9"))),
        ("PRI-002", prism("PRI-002", Some("Bright"))),
        ("PRI-003", prism("PRI-003", Some("bright"))),
        ("PRI-004", prism("PRI-004", None)),
        ("PRI-005", doc("PRI-005", "Prism", "tier: 3\n", &[])),
    ]);
    let report = validate(&dir, &both());
    let hits = found(&report, "undeclared-tier-value");
    assert_eq!(hits.len(), 3, "{hits:?}");
    for (id, value) in [
        ("PRI-001", "class-9"),
        ("PRI-002", "Bright"),
        ("PRI-005", "3"),
    ] {
        assert!(
            hits.iter().any(|f| f.message.contains(id)
                && f.message.contains(&format!("'{value}'"))
                && f.message.contains("tier")),
            "{id}: {hits:?}"
        );
    }
}

#[trace("FR-091-AC-18")]
#[test]
fn a_vocabulary_entry_with_no_carried_vocabulary_reports_and_finds_no_value() {
    let dir = repo(&[("PRI-001", prism("PRI-001", Some("class-9")))]);
    // Prism alone: `tier` has no enum anywhere, so no carried_vocabulary entry.
    let report = validate(&dir, &modules(&["prism"]));
    let dead = found(&report, "undeclared-relation-vocabulary");
    assert!(
        dead.iter()
            .any(|f| f.message.contains("'prism-tier-in-vocabulary'")),
        "{dead:?}"
    );
    assert!(found(&report, "undeclared-tier-value").is_empty());

    // The finding set does not depend on which module loaded first.
    let messages = |names: &[&str]| -> Vec<String> {
        let report = validate(&dir, &modules(names));
        let mut all: Vec<String> = report
            .errors
            .iter()
            .chain(&report.warnings)
            .filter(|f| f.pack.is_some())
            .map(|f| format!("{}|{}", f.reason, f.message))
            .collect();
        all.sort();
        all
    };
    // Both modules declare the carried name and neither has an enum: the
    // state the AC names, under both load orders.
    let first = messages(&["prism", "loose"]);
    assert_eq!(first, messages(&["loose", "prism"]));
    assert!(
        first
            .iter()
            .any(|m| m.starts_with("undeclared-relation-vocabulary|")
                && m.contains("prism-tier-in-vocabulary")),
        "{first:?}"
    );
    assert!(first
        .iter()
        .all(|m| !m.starts_with("undeclared-tier-value")));
    assert_eq!(messages(&["glyph", "prism"]), messages(&["prism", "glyph"]));
}

// ── FR-091-AC-10 / AC-11 ──────────────────────────────────────────────────

fn rules_module(section: &str, entry: &str) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    write(
        dir.path(),
        "manifest.yaml",
        &format!(
            "name: bad\nversion: 0.1.0\ndescription: x\nartifact_types:\n- name: alpha\n  \
             defaults:\n    id_pattern: AL-{{next:03d}}\ntraceability:\n  {section}:\n{entry}"
        ),
    );
    dir
}

const GOOD_CONSISTENCY: &str = "  - name: entry\n    from: [alpha]\n    field: f\n    \
    edges: [links_to]\n    direction: outgoing\n    to: [alpha]\n    to_field: f\n    \
    order_from: { archetype: alpha, field: f }\n    rule: not_below\n    check: below\n";

#[trace("FR-091-AC-10")]
#[test]
fn a_structurally_bad_declaration_fails_the_declaring_module_naming_the_entry() {
    let load = |dir: &TempDir| Registry::load_module_strict(dir.path()).map(|_| ());
    let ok = rules_module("field_consistency", GOOD_CONSISTENCY);
    load(&ok).expect("the baseline loads");

    for (from, to) in [
        ("from: [alpha]", "from: []"),
        ("edges: [links_to]", "edges: []"),
        ("to: [alpha]", "to: []"),
        ("direction: outgoing", "direction: sideways"),
        ("rule: not_below", "rule: not_above"),
        ("check: below", "check: 'a:b'"),
        ("field: f\n", "field: ''\n"),
        (
            "edges: [links_to]\n",
            "via: { edges: [x], direction: incoming }\n    edges: [links_to]\n",
        ),
        (
            "edges: [links_to]\n",
            "via: { archetype: alpha, direction: incoming }\n    edges: [links_to]\n",
        ),
        (
            "edges: [links_to]\n",
            "via: { archetype: alpha, edges: [x], direction: up }\n    edges: [links_to]\n",
        ),
    ] {
        let entry = GOOD_CONSISTENCY.replacen(from, to, 1);
        let err = load(&rules_module("field_consistency", &entry))
            .err()
            .unwrap_or_else(|| panic!("{to} must fail module load"));
        assert!(err.to_string().contains("'entry'"), "{to}: {err}");
    }

    let blank = GOOD_CONSISTENCY.replacen("name: entry", "name: ''", 1);
    assert!(load(&rules_module("field_consistency", &blank)).is_err());
    let duplicate = format!("{GOOD_CONSISTENCY}{GOOD_CONSISTENCY}");
    let err = load(&rules_module("field_consistency", &duplicate)).expect_err("duplicate");
    assert!(err.to_string().contains("'entry'"), "{err}");

    let required = "  - name: needs\n    from: [alpha]\n    field: f\n    check: unset\n";
    load(&rules_module("required_fields", required)).expect("required baseline");
    for bad in [
        required.replace("from: [alpha]", "from: []"),
        required.replace("check: unset", "check: ''"),
        format!("{required}{required}"),
    ] {
        let err = load(&rules_module("required_fields", &bad)).expect_err("bad required");
        assert!(err.to_string().contains("required_fields"), "{err}");
    }

    let vocabulary = "  - name: held\n    from: [alpha]\n    field: f\n    carried: c\n    \
                      check: outside\n";
    load(&rules_module("vocabulary", vocabulary)).expect("vocabulary baseline");
    for bad in [
        vocabulary.replace("carried: c\n", "carried: ''\n"),
        vocabulary.replace("    carried: c\n", ""),
        vocabulary.replace("field: f", "field: ''"),
    ] {
        let err = load(&rules_module("vocabulary", &bad)).expect_err("bad vocabulary");
        assert!(err.to_string().contains("'held'"), "{err}");
    }
}

#[trace("FR-091-AC-11")]
#[test]
fn an_entry_naming_another_modules_archetype_loads_and_reports_when_that_module_is_absent() {
    // The module that owns the entries loads on its own.
    Registry::load_module_strict(&fixture("prism")).expect("loads without the stating module");

    let dir = pair("dim", "bright");
    // Without the stating module the merged corpus reports the entry.
    let alone = validate(&dir, &modules(&["prism"]));
    let dead = found(&alone, "undeclared-relation-vocabulary");
    assert!(
        dead.iter()
            .any(|f| f.message.contains("'glyph-tier-below-prism'")),
        "{dead:?}"
    );
    assert!(found(&alone, "tier-below-prism").is_empty());

    // With both it works.
    let together = validate(&dir, &both());
    assert_eq!(found(&together, "tier-below-prism").len(), 1);
    assert!(found(&together, "undeclared-relation-vocabulary").is_empty());
}

// ── narrowing: exclude, where, the via filter, the model-level exclusion ──

fn alpha_module(extra_model: &str, required: &str) -> (TempDir, Registry) {
    let dir = TempDir::new().expect("tempdir");
    write(
        dir.path(),
        "manifest.yaml",
        &format!(
            "name: narrow\nversion: 0.1.0\ndescription: x\nartifact_types:\n- name: alpha\n  \
             defaults:\n    id_pattern: AL-{{next:03d}}\ntraceability:\n{extra_model}  \
             required_fields:\n{required}"
        ),
    );
    let registry = Registry::load_module(dir.path()).expect("loads");
    (dir, registry)
}

#[trace("FR-091-AC-5")]
#[test]
fn required_fields_exclude_where_and_the_model_exclusion_narrow_the_obliged() {
    let (_module, registry) = alpha_module(
        "  exclude: ['spec/skip/**']\n",
        "  - name: needs\n    from: [alpha]\n    field: f\n    check: unset\n    \
         exclude: ['spec/legacy/**']\n    where: { field: kind, values: [live] }\n",
    );
    let at = |path: &str, id: &str, extra: &str| (path.to_string(), doc(id, "alpha", extra, &[]));
    let docs = [
        at("spec/obliged.md", "AL-001", "kind: live\n"),
        at("spec/draft.md", "AL-002", "kind: draft\n"),
        at("spec/legacy/old.md", "AL-003", "kind: live\n"),
        at("spec/skip/fixture.md", "AL-004", "kind: live\n"),
        at("spec/stated.md", "AL-005", "kind: live\nf: x\n"),
    ];
    let dir = TempDir::new().expect("tempdir");
    for (path, text) in &docs {
        write(dir.path(), path, text);
    }
    let report = validate(&dir, &registry);
    let hits = found(&report, "unset");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].message.contains("AL-001"), "{hits:?}");
}

#[trace("FR-091-AC-5")]
#[test]
fn a_where_that_selects_nothing_reports_under_undeclared_relation_vocabulary() {
    let dead_field = alpha_module(
        "",
        "  - name: needs\n    from: [alpha]\n    field: f\n    check: unset\n    \
         where: { field: nosuch, values: [x] }\n",
    );
    let dir = repo(&[("AL-001", doc("AL-001", "alpha", "f: x\n", &[]))]);
    let report = validate(&dir, &dead_field.1);
    assert!(
        found(&report, "undeclared-relation-vocabulary")
            .iter()
            .any(|f| f.message.contains("'needs'") && f.message.contains("nosuch")),
        "{report:?}"
    );

    // A value outside the field's enum that no document holds; a value that
    // differs only by case or whitespace is live.
    let dir = repo(&[("GLY-001", glyph("GLY-001", Some("dim"), &[]))]);
    let entry = |values: &str| {
        let (module, _) = alpha_module(
            "",
            &format!(
                "  - name: needs\n    from: [Glyph]\n    field: tier\n    check: unset\n    \
                 where: {{ field: tier, values: [{values}] }}\n"
            ),
        );
        let registry =
            Registry::load_module_set(&[fixture("glyph").as_path(), module.path()]).expect("loads");
        (module, validate(&dir, &registry))
    };
    let (_m, dead) = entry("shiny");
    assert!(found(&dead, "undeclared-relation-vocabulary")
        .iter()
        .any(|f| f.message.contains("'shiny'")));
    let (_m, live) = entry("' DIM '");
    assert!(found(&live, "undeclared-relation-vocabulary").is_empty());
}

#[trace("FR-091-AC-1")]
#[test]
fn the_via_hop_reads_only_documents_of_its_archetype_and_the_model_exclusion_applies() {
    // A Pier, not a Bridge, joins the glyph to the prism: no pair.
    let pier = repo(&[
        ("GLY-001", glyph("GLY-001", Some("dim"), &[])),
        (
            "PIR-001",
            doc(
                "PIR-001",
                "Pier",
                "",
                &[("bridges", "GLY-001"), ("rests_on", "PRI-001")],
            ),
        ),
        ("PRI-001", prism("PRI-001", Some("bright"))),
    ]);
    assert!(found(&validate(&pier, &both()), "tier-below-prism").is_empty());

    // The model-level exclusion removes a `from` document from every rule.
    let (_module, registry) = alpha_module(
        "  exclude: ['spec/skip/**']\n",
        "  - name: needs\n    from: [alpha]\n    field: f\n    check: unset\n",
    );
    let dir = TempDir::new().expect("tempdir");
    write(
        dir.path(),
        "spec/skip/x.md",
        &doc("AL-001", "alpha", "", &[]),
    );
    write(dir.path(), "spec/y.md", &doc("AL-002", "alpha", "", &[]));
    let hits = found(&validate(&dir, &registry), "unset").len();
    assert_eq!(hits, 1);
}
