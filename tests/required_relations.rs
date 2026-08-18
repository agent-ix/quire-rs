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
        &fr("FR-001", Some(("satisfies", "StR-001"))),
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
        &fr("FR-001", Some(("satisfies", "StR-001"))),
    );
    write(
        &root,
        "FR-002.md",
        &fr("FR-002", Some(("satisfies", "StR-001"))),
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
fn tc900_a_need_nothing_satisfies_is_reported() {
    let root = tmpdir("900");
    write(&root, "StR-001.md", &str_doc("StR-001"));
    write(&root, "StR-002.md", &str_doc("StR-002"));
    write(
        &root,
        "FR-001.md",
        &fr("FR-001", Some(("satisfies", "StR-001"))),
    );

    let report = validate(&root, None);
    let hits = findings(&report, "unsatisfied-str");
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
        &fr("FR-001", Some(("satisfies", "StR-404"))),
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
// naming the path. A requirement that transitively derives from itself states
// nothing, and no per-document check can see it.
//
// The verb is `derives_from` — the vocabulary's declared decomposition-lineage
// edge (spec-artifacts-iso FR-004). `refines`, which an earlier draft used, is
// not in the vocabulary at all.
#[test]
fn tc902_a_derivation_cycle_is_reported_once() {
    let root = tmpdir("902");
    write(&root, "StR-001.md", &str_doc("StR-001"));
    write(
        &root,
        "FR-001.md",
        &fr("FR-001", Some(("derives_from", "FR-002"))),
    );
    write(
        &root,
        "FR-002.md",
        &fr("FR-002", Some(("derives_from", "FR-003"))),
    );
    write(
        &root,
        "FR-003.md",
        &fr("FR-003", Some(("derives_from", "FR-001"))),
    );

    let report = validate(&root, None);
    let hits = findings(&report, "cyclic-derives_from");
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
    assert_eq!(findings(&base, "unsatisfied-str").len(), 2);
    assert!(base.is_valid(), "advisory by default: {:?}", base.errors);

    // One off, the other untouched — the keys are per declaration.
    let mut map = GrammarSeverityMap::new();
    map.insert("trace:orphan-fr".into(), GrammarSeverityLevel::Off);
    let scoped = validate(&root, Some(map));
    assert_eq!(findings(&scoped, "orphan-fr").len(), 0, "switched off");
    assert_eq!(
        findings(&scoped, "unsatisfied-str").len(),
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
    for reason in ["orphan-fr", "unsatisfied-str", "cyclic-refines"] {
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
    assert_eq!(model.acyclic_edges, vec!["derives_from".to_string()]);
    assert!(!model.exclude.is_empty(), "exclude survived the merge");

    // And the same model read through `is_empty` — the gate that decides
    // whether `traceability()` returns anything at all.
    assert!(
        !model.is_empty(),
        "a model with only required relations must not read as empty"
    );
}

// TC-906 (FR-058-AC-10): a required relation that cannot be executed is
// rejected at load, not discovered as a corpus-wide false alarm.
//
// The two failure modes below are the reason this check exists at all. Neither
// is loud: both produce a *plausible* report rather than an error, so the
// declaration looks fine and the corpus looks broken.
//
//   * `edges: []` — no verb can satisfy the relation, so EVERY `from`
//     document is reported. On a real repository that is hundreds of findings
//     against documents that are perfectly well linked.
//   * a `check` token that cannot form a `trace:<check>` severity key — the
//     relation runs, but no `--severity` flag and no module override can ever
//     name it, so it cannot be tuned or switched off.
//
// Both are caught by `TraceabilityModel::validate`, which is the only place
// that can report them against the declaration rather than against the
// documents.
#[test]
fn tc906_an_unexecutable_relation_is_rejected_at_load() {
    use quire_rs::traceability::{RelationDirection, RequiredRelation, TraceabilityModel};

    let relation = |edges: Vec<&str>, check: &str| RequiredRelation {
        name: "upward".to_string(),
        from: "FR".to_string(),
        edges: edges.into_iter().map(str::to_string).collect(),
        to: vec!["StR".to_string()],
        direction: RelationDirection::Either,
        check: check.to_string(),
        exclude: vec![],
    };
    let model = |r: RequiredRelation| TraceabilityModel {
        required_relations: vec![r],
        ..TraceabilityModel::default()
    };

    // Baseline: the same declaration, well-formed, loads.
    model(relation(vec!["satisfies"], "orphan-fr"))
        .validate()
        .expect("a well-formed relation loads");

    let err = model(relation(vec![], "orphan-fr"))
        .validate()
        .expect_err("no accepted verb means nothing could ever satisfy it");
    assert!(
        err.contains("declares no edges") && err.contains("every 'FR' document would be reported"),
        "the error names the consequence, not just the empty field: {err}"
    );

    // A colon in the token would make `trace:a:b` ambiguous; whitespace breaks
    // the `--severity` entry the same way. Both must fail.
    for bad in ["orphan:fr", "orphan fr", ""] {
        let err = model(relation(vec!["satisfies"], bad))
            .validate()
            .expect_err("a check token that cannot form a severity key is rejected");
        assert!(
            err.contains("required_relations"),
            "the error names the declaration for token {bad:?}: {err}"
        );
    }

    // A blank verb in `acyclic_edges` would walk a graph no edge matches — the
    // cycle check would silently cover nothing while reading as declared.
    let err = TraceabilityModel {
        acyclic_edges: vec![String::new()],
        ..TraceabilityModel::default()
    }
    .validate()
    .expect_err("an empty acyclic verb checks nothing");
    assert!(err.contains("acyclic_edges"), "{err}");
}

// TC-907 (FR-058-AC-10): two relations cannot share a name.
//
// The name is what the finding is reported under and what a reader matches
// against the manifest. Two entries sharing one name make a report that cannot
// be traced back to the declaration that produced it.
#[test]
fn tc907_duplicate_relation_names_are_rejected() {
    use quire_rs::traceability::{RelationDirection, RequiredRelation, TraceabilityModel};

    let one = RequiredRelation {
        name: "upward".to_string(),
        from: "FR".to_string(),
        edges: vec!["satisfies".to_string()],
        to: vec![],
        direction: RelationDirection::Outgoing,
        check: "orphan-fr".to_string(),
        exclude: vec![],
    };
    let err = TraceabilityModel {
        required_relations: vec![one.clone(), one],
        ..TraceabilityModel::default()
    }
    .validate()
    .expect_err("duplicate names are rejected");
    assert!(
        err.contains("duplicate required_relations entry 'upward'"),
        "{err}"
    );
}

// TC-908 (FR-058-AC-11): a relation naming a kind nothing declares and no
// document is reports itself, instead of silently checking nothing.
//
// This is the failure this check exists for, and it is invisible without it.
// Measured on the fixture: changing `from: FR` to `from: FRR` leaves FR-001 —
// a genuine orphan with no upstream need — UNREPORTED, and the whole run comes
// back clean. A one-character slip disables the check, and from the outside
// that is indistinguishable from a bundle with nothing wrong.
//
// `TraceabilityModel::validate` cannot catch it: it runs per module at
// manifest-parse time, and a relation legitimately names kinds another module
// contributes. Only here are the merged registry and the walked bundle both in
// hand.
#[test]
fn tc908_a_relation_naming_a_dead_kind_reports_itself() {
    let root = tmpdir("908");
    write(&root, "StR-001.md", &str_doc("StR-001"));
    write(&root, "FR-001.md", &fr("FR-001", None));

    // The fixture module is well-formed, so nothing is dead and the orphan is
    // found the normal way.
    let clean = validate(&root, None);
    assert!(
        findings(&clean, "undeclared-relation-vocabulary").is_empty(),
        "a well-formed declaration reports no dead kind: {:?}",
        findings(&clean, "undeclared-relation-vocabulary")
    );
    assert_eq!(
        findings(&clean, "orphan-fr").len(),
        1,
        "the orphan is found while the declaration is intact"
    );

    // Now typo the archetype the relation selects on.
    let module = tmpdir("908_module");
    let manifest = fs::read_to_string(fixture_module("required-relations").join("manifest.yaml"))
        .expect("read fixture manifest")
        .replace("    from: FR\n", "    from: FRR\n");
    fs::write(module.join("manifest.yaml"), &manifest).expect("write");

    let registry = Registry::load_module(&module).expect("a typoed `from` still loads");
    let report = validate_bundle_at(&root, &registry, BundlePosture::Okf);

    // The orphan silently vanishes — this is the damage.
    assert!(
        findings(&report, "orphan-fr").is_empty(),
        "the typo really does disable the check, which is why the guard is needed"
    );
    // …and the declaration says so itself.
    let dead = findings(&report, "undeclared-relation-vocabulary");
    assert_eq!(dead.len(), 1, "one dead declaration: {dead:?}");
    assert!(
        dead[0].message.contains("FRR") && dead[0].message.contains("fr-has-upstream-need"),
        "names the dead kind and the declaration that carries it: {}",
        dead[0].message
    );

    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&module).ok();
}

// TC-909 (FR-058-AC-2): the `to` list accepts more than one upstream kind, so
// an FR hanging off a use case rather than a stakeholder requirement satisfies
// the same relation.
//
// The 29148 chain is stakeholder -> system -> software. Declaring only `StR`
// reported 6 of quire-rs's 38 FRs — a 16% false-positive rate that was a fault
// in the DECLARATION, not the engine, and fixing it was a manifest edit. The
// fixture accepted `US` in `to:` from the start but never declared the kind and
// no test exercised it, so the behaviour this comment describes was untested.
#[test]
fn tc909_an_fr_satisfying_a_use_case_is_not_an_orphan() {
    let root = tmpdir("909");
    write(
        &root,
        "US-001.md",
        "---\nid: US-001\ntype: US\ntitle: A use case\n---\n\n## Description\n\nThe operator does it.\n",
    );
    write(
        &root,
        "FR-001.md",
        &fr("FR-001", Some(("satisfies", "US-001"))),
    );
    write(&root, "FR-002.md", &fr("FR-002", None));

    let report = validate(&root, None);
    let hits = findings(&report, "orphan-fr");
    assert_eq!(
        hits.len(),
        1,
        "only the FR with no upstream need at all is reported: {hits:?}"
    );
    assert!(
        hits[0].message.contains("FR-002"),
        "the use-case-backed FR is not the one reported: {}",
        hits[0].message
    );
    fs::remove_dir_all(&root).ok();
}

// TC-910 (FR-058-AC-1): the finding reads as a sentence for both shapes of
// `to` — a constrained list and the `to: []` "any document" case.
//
// The first end-to-end run against `spec-objects-safety`, whose
// `hazard-has-mitigation` relation uses `to: []`, printed
// "nothing reaches 'HAZ-002' by 'mitigates' from any any document". The
// article was hardcoded in the template AND present in the noun phrase.
#[test]
fn tc910_the_finding_reads_as_a_sentence() {
    let root = tmpdir("910");
    write(&root, "StR-001.md", &str_doc("StR-001"));
    write(&root, "FR-001.md", &fr("FR-001", None));

    let report = validate(&root, None);
    for f in report.errors.iter().chain(report.warnings.iter()) {
        assert!(
            !f.message.contains("any any"),
            "doubled article in: {}",
            f.message
        );
    }
    // The constrained case still names the kinds it accepts.
    let orphan = findings(&report, "orphan-fr");
    assert_eq!(orphan.len(), 1);
    assert!(
        orphan[0].message.contains("any StR/US"),
        "names the accepted kinds with one article: {}",
        orphan[0].message
    );
    fs::remove_dir_all(&root).ok();
}
