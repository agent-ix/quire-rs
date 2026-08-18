//! FR-058 — upward-trace completeness (TC-898..TC-903).
//!
//! Bundles are built on disk per test so the whole path runs: walk, resolve,
//! then the declared check. Every archetype, verb and direction in these
//! assertions comes from the fixture module's manifest — swapping the manifest
//! swaps the contract without touching the engine, which is the property
//! agent-ix/spec-objects-security#5 depends on.

use std::fs;
use std::path::{Path, PathBuf};

use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
use quire_rs::{validate_bundle_at, BundleFinding, BundlePosture, BundleReport, Registry};

fn fixture_module(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability")
        .join(name)
}

fn tmpdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("quire_reqrel_{tag}_{}", std::process::id()));
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

/// An FR whose frontmatter declares `relationships`, so the edge is harvested
/// from structured data rather than from prose.
fn fr(id: &str, rel: Option<(&str, &str)>) -> String {
    let relationships = match rel {
        Some((verb, target)) => {
            format!("relationships:\n  - target: \"{target}\"\n    type: \"{verb}\"\n")
        }
        None => String::new(),
    };
    format!("---\nid: {id}\ntype: FR\ntitle: A requirement\n{relationships}---\n\n## Description\n\nThe system shall do it.\n")
}

fn str_doc(id: &str) -> String {
    format!("---\nid: {id}\ntype: StR\ntitle: A need\n---\n\n## Stakeholder Need\n\nThe operator shall be able to do it.\n")
}

fn validate(root: &Path, severity: Option<GrammarSeverityMap>) -> BundleReport {
    let registry = Registry::load_module(&fixture_module("required-relations")).expect("load");
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

// TC-898 (FR-058-AC-1): an FR with no upstream edge to a declared kind is
// reported; one that has the edge is not. This is the orphan-requirement case —
// a feature nobody asked for — and it is the only analysis class that finds a
// *missing* requirement rather than an unverified one.
#[test]
fn tc898_an_fr_with_no_upstream_need_is_reported() {
    let root = tmpdir("898");
    write(&root, "StR-001.md", &str_doc("StR-001"));
    write(
        &root,
        "FR-001.md",
        &fr("FR-001", Some(("implements", "StR-001"))),
    );
    write(&root, "FR-002.md", &fr("FR-002", None));

    let report = validate(&root, None);
    let hits = findings(&report, "orphan-fr");
    assert_eq!(hits.len(), 1, "exactly the orphan is reported: {hits:?}");
    assert!(
        hits[0].message.contains("FR-002"),
        "names the orphan: {}",
        hits[0].message
    );
    assert!(
        hits[0].message.contains("fr-has-upstream-need"),
        "names the declaration that asked for it: {}",
        hits[0].message
    );
    fs::remove_dir_all(&root).ok();
}

// TC-899 (FR-058-AC-2): any one of the declared verbs satisfies the relation —
// a module that accepts `implements` or `refines` says so once rather than
// declaring the relation twice.
#[test]
fn tc899_any_declared_verb_satisfies_the_relation() {
    let root = tmpdir("899");
    write(&root, "StR-001.md", &str_doc("StR-001"));
    write(
        &root,
        "FR-001.md",
        &fr("FR-001", Some(("implements", "StR-001"))),
    );
    write(
        &root,
        "FR-002.md",
        &fr("FR-002", Some(("refines", "StR-001"))),
    );
    // A verb the declaration does not accept does not satisfy it.
    write(
        &root,
        "FR-003.md",
        &fr("FR-003", Some(("references", "StR-001"))),
    );

    let report = validate(&root, None);
    let hits = findings(&report, "orphan-fr");
    assert_eq!(hits.len(), 1, "only the undeclared verb fails: {hits:?}");
    assert!(hits[0].message.contains("FR-003"), "{}", hits[0].message);
    fs::remove_dir_all(&root).ok();
}

// TC-900 (FR-058-AC-3): the `incoming` direction reads the same declaration
// the other way — a stated need nothing implements is a need nobody built.
#[test]
fn tc900_a_need_nothing_implements_is_reported() {
    let root = tmpdir("900");
    write(&root, "StR-001.md", &str_doc("StR-001"));
    write(&root, "StR-002.md", &str_doc("StR-002"));
    write(
        &root,
        "FR-001.md",
        &fr("FR-001", Some(("implements", "StR-001"))),
    );

    let report = validate(&root, None);
    let hits = findings(&report, "unimplemented-str");
    assert_eq!(hits.len(), 1, "exactly the unbuilt need: {hits:?}");
    assert!(hits[0].message.contains("StR-002"), "{}", hits[0].message);
    fs::remove_dir_all(&root).ok();
}

// TC-901 (FR-058-AC-4): a **dangling** edge does not satisfy a relation whose
// targets are constrained. The target is not in the bundle, so nothing can say
// what archetype it is — and accepting it would let a typo satisfy the
// requirement it broke.
#[test]
fn tc901_a_dangling_edge_does_not_satisfy_a_constrained_relation() {
    let root = tmpdir("901");
    write(
        &root,
        "FR-001.md",
        &fr("FR-001", Some(("implements", "StR-404"))),
    );

    let report = validate(&root, None);
    let hits = findings(&report, "orphan-fr");
    assert_eq!(
        hits.len(),
        1,
        "a typo'd target must not count as satisfaction: {hits:?}"
    );
    fs::remove_dir_all(&root).ok();
}

// TC-902 (FR-058-AC-5): a cycle over a declared acyclic verb is reported once,
// naming the path. A requirement that transitively refines itself states
// nothing, and no per-document check can see it.
#[test]
fn tc902_a_refines_cycle_is_reported_once() {
    let root = tmpdir("902");
    write(&root, "StR-001.md", &str_doc("StR-001"));
    write(
        &root,
        "FR-001.md",
        &fr("FR-001", Some(("refines", "FR-002"))),
    );
    write(
        &root,
        "FR-002.md",
        &fr("FR-002", Some(("refines", "FR-003"))),
    );
    write(
        &root,
        "FR-003.md",
        &fr("FR-003", Some(("refines", "FR-001"))),
    );

    let report = validate(&root, None);
    let hits = findings(&report, "cyclic-refines");
    assert_eq!(
        hits.len(),
        1,
        "a three-node cycle is one finding, not three: {hits:?}"
    );
    let message = &hits[0].message;
    for id in ["FR-001", "FR-002", "FR-003"] {
        assert!(message.contains(id), "path names {id}: {message}");
    }
    fs::remove_dir_all(&root).ok();
}

// TC-903 (FR-058-AC-6/AC-7): each declared relation carries its own
// `trace:<check>` severity key, so a module tunes them independently — and
// FR-058's findings ship advisory, tunable by the FR-057 registry.
#[test]
fn tc903_each_relation_is_independently_tunable() {
    let root = tmpdir("903");
    write(&root, "StR-001.md", &str_doc("StR-001"));
    write(&root, "StR-002.md", &str_doc("StR-002"));
    write(&root, "FR-001.md", &fr("FR-001", None));

    // Unconfigured: both fire, and under Okf both are warnings.
    let base = validate(&root, None);
    assert_eq!(findings(&base, "orphan-fr").len(), 1);
    assert_eq!(findings(&base, "unimplemented-str").len(), 2);
    assert!(base.is_valid(), "advisory by default: {:?}", base.errors);

    // One off, the other untouched — the keys are per declaration.
    let mut map = GrammarSeverityMap::new();
    map.insert("trace:orphan-fr".into(), GrammarSeverityLevel::Off);
    let scoped = validate(&root, Some(map));
    assert_eq!(findings(&scoped, "orphan-fr").len(), 0, "switched off");
    assert_eq!(
        findings(&scoped, "unimplemented-str").len(),
        2,
        "its sibling is untouched"
    );

    // And promotion works through the same key.
    let mut map = GrammarSeverityMap::new();
    map.insert("trace:orphan-fr".into(), GrammarSeverityLevel::Error);
    let promoted = validate(&root, Some(map));
    assert!(
        !promoted.is_valid(),
        "error promotes out of the warning tier"
    );
    assert_eq!(
        promoted
            .errors
            .iter()
            .filter(|f| f.reason == "orphan-fr")
            .count(),
        1
    );

    fs::remove_dir_all(&root).ok();
}

// TC-904 (FR-058-AC-8): a module declaring neither key is a no-op — the report
// is byte-identical to one from a module that never heard of FR-058.
#[test]
fn tc904_a_module_declaring_nothing_sees_no_change() {
    let root = tmpdir("904");
    write(&root, "FR-001.md", &fr("FR-001", None));
    write(&root, "StR-001.md", &str_doc("StR-001"));

    let plain = Registry::load_module(&fixture_module("plain")).expect("load plain");
    let report = validate_bundle_at(&root, &plain, BundlePosture::Okf);
    for reason in ["orphan-fr", "unimplemented-str", "cyclic-refines"] {
        assert!(
            findings(&report, reason).is_empty(),
            "{reason} must not fire for an undeclared module"
        );
    }
    fs::remove_dir_all(&root).ok();
}

// TC-905 (FR-058-CON-1): every field of the declared model survives a merge.
//
// This exists because adding `required_relations` broke **two** hand-maintained
// per-field functions at once — `TraceabilityModel::is_empty`, which decides
// whether a model counts as declared at all, and `merge_traceability`, which
// combines models across modules. Neither is an exhaustive `match`, so the
// compiler said nothing and the new check silently never ran: a module whose
// whole model was "every FR must trace to a StR" had that model discarded.
//
// A field added to the model and to nothing else fails here rather than being
// dropped on the floor.
#[test]
fn tc905_every_declared_field_survives_the_merge() {
    let registry = Registry::load_module(&fixture_module("required-relations")).expect("load");
    let model = registry
        .traceability()
        .expect("a model declaring only required relations is still a declared model");

    assert_eq!(
        model.required_relations.len(),
        2,
        "required_relations survived the merge"
    );
    assert_eq!(model.acyclic_edges, vec!["refines".to_string()]);
    assert!(!model.exclude.is_empty(), "exclude survived the merge");

    // And the same model read through `is_empty` — the gate that decides
    // whether `traceability()` returns anything at all.
    assert!(
        !model.is_empty(),
        "a model with only required relations must not read as empty"
    );
}
