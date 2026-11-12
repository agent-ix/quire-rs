//! FR-058-AC-12..22 — the `where` field-value selector on a required relation
//! (TC-2248..TC-2257).
//!
//! The field name, the values, the kinds and the verbs all come from manifests
//! written here; nothing in the engine knows any of them (FR-058-CON-3).

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
use quire_rs::{validate_bundle_at, BundleFinding, BundlePosture, BundleReport, Registry};

fn fixture_module() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability/required-relations-where")
}

fn tmpdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("quire_reqrel_where_{tag}_{}", std::process::id()));
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

/// An NFR whose frontmatter carries `field_line` verbatim (or nothing) and an
/// optional outgoing edge.
fn nfr(id: &str, field_line: &str, edge: Option<(&str, &str)>) -> String {
    let relationships = edge
        .map(|(verb, target)| {
            format!("relationships:\n  - target: \"{target}\"\n    type: \"{verb}\"\n")
        })
        .unwrap_or_default();
    format!(
        "---\nid: {id}\ntype: NFR\ntitle: A quality\n{field_line}\n{relationships}---\n\n## Description\n\nThe system shall be so.\n"
    )
}

fn attr(value: &str) -> String {
    format!("quality_attribute: {value}")
}

fn threat(id: &str, edge: Option<(&str, &str)>) -> String {
    let relationships = edge
        .map(|(verb, target)| {
            format!("relationships:\n  - target: \"{target}\"\n    type: \"{verb}\"\n")
        })
        .unwrap_or_default();
    format!("---\nid: {id}\ntype: Threat\ntitle: A threat\n{relationships}---\n\n## Description\n\nA threat.\n")
}

fn registry() -> Registry {
    Registry::load_module(&fixture_module()).expect("load")
}

/// A module whose only relation is `relation_yaml`, over the fixture's schema.
fn module_with(tag: &str, relation_yaml: &str) -> PathBuf {
    let dir = tmpdir(&format!("mod_{tag}"));
    write(
        &dir,
        "schemas/nfr.schema.json",
        &fs::read_to_string(fixture_module().join("schemas/nfr.schema.json")).expect("schema"),
    );
    write(
        &dir,
        "schemas/threat.schema.json",
        &fs::read_to_string(fixture_module().join("schemas/threat.schema.json")).expect("schema"),
    );
    write(
        &dir,
        "manifest.yaml",
        &format!(
            "name: where-{tag}\nversion: 0.1.0\ndescription: d\narchetypes:\n- kind: NFR\n  name: NFR\n  frontmatter_schema_ref: schemas/nfr.schema.json\n- kind: Threat\n  name: Threat\n  frontmatter_schema_ref: schemas/threat.schema.json\ntraceability:\n  required_relations:\n  - name: sel\n    from: NFR\n    edges: [mitigates]\n    to: [Threat]\n    direction: outgoing\n    check: sel-check\n{relation_yaml}"
        ),
    );
    dir
}

fn findings<'r>(report: &'r BundleReport, reason: &str) -> Vec<&'r BundleFinding> {
    report
        .errors
        .iter()
        .chain(report.warnings.iter())
        .filter(|f| f.reason == reason)
        .collect()
}

fn run(root: &Path, registry: &Registry) -> BundleReport {
    validate_bundle_at(root, registry, BundlePosture::Okf)
}

/// Ids named by the findings of one check, sorted.
fn ids(report: &BundleReport, reason: &str) -> Vec<String> {
    let mut v: Vec<String> = findings(report, reason)
        .iter()
        .map(|f| f.message.split('\'').nth(1).unwrap_or("").to_string())
        .collect();
    v.sort();
    v
}

#[trace("TC-2248", "FR-058-AC-12", "FR-058-AC-15")]
#[test]
fn tc2248_where_selects_by_field_value_and_narrows_only_the_obliged_side() {
    let root = tmpdir("2248");
    // Outgoing: selected+no edge (reported), selected+edge (clean),
    // outside values with and without edge (never reported).
    write(&root, "T-001.md", &threat("T-001", None));
    write(&root, "N-001.md", &nfr("N-001", &attr("security"), None));
    write(
        &root,
        "N-002.md",
        &nfr("N-002", &attr("safety"), Some(("mitigates", "T-001"))),
    );
    write(&root, "N-003.md", &nfr("N-003", &attr("usability"), None));
    write(
        &root,
        "N-004.md",
        &nfr("N-004", &attr("usability"), Some(("mitigates", "T-001"))),
    );
    // Incoming relation (security only): a Threat whose own NFR-side field is
    // irrelevant still satisfies it; the source here is a Threat, so `where`
    // never filters it. N-005 is targeted by T-002, N-001 by nothing.
    write(
        &root,
        "N-005.md",
        &nfr("N-005", &attr("security"), Some(("mitigates", "T-001"))),
    );
    write(
        &root,
        "T-002.md",
        &threat("T-002", Some(("mitigated_by", "N-005"))),
    );

    let report = run(&root, &registry());
    assert_eq!(
        ids(&report, "unjustified-classified-nfr"),
        vec!["N-001"],
        "{report:?}"
    );
    assert_eq!(
        ids(&report, "untargeted-security-nfr"),
        vec!["N-001"],
        "N-005 is targeted by T-002, N-001 by nothing: {report:?}"
    );
    fs::remove_dir_all(&root).ok();

    // `to` stays constrained exactly as without `where`: an edge to a document
    // of another kind, or to nothing in the bundle, does not satisfy; and an
    // incoming edge from a non-`to` kind does not satisfy the incoming relation.
    let root = tmpdir("2248b");
    write(&root, "N-010.md", &nfr("N-010", &attr("usability"), None));
    write(
        &root,
        "N-011.md",
        &nfr("N-011", &attr("security"), Some(("mitigates", "N-010"))),
    );
    write(
        &root,
        "N-012.md",
        &nfr("N-012", &attr("safety"), Some(("mitigates", "T-404"))),
    );
    write(
        &root,
        "N-013.md",
        &nfr("N-013", &attr("usability"), Some(("mitigated_by", "N-011"))),
    );
    let report = run(&root, &registry());
    assert_eq!(
        ids(&report, "unjustified-classified-nfr"),
        vec!["N-011", "N-012"],
        "{report:?}"
    );
    assert_eq!(
        ids(&report, "untargeted-security-nfr"),
        vec!["N-011"],
        "an NFR-sourced edge is not a Threat: {report:?}"
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2249", "FR-058-AC-13")]
#[test]
fn tc2249_absent_null_empty_or_non_string_values_are_not_selected() {
    let root = tmpdir("2249");
    let cases = [
        ("N-001", String::new()),
        ("N-002", "quality_attribute: null".to_string()),
        ("N-003", "quality_attribute: \"\"".to_string()),
        ("N-004", "quality_attribute: 7".to_string()),
        ("N-005", "quality_attribute: [security]".to_string()),
        ("N-006", "quality_attribute: {a: security}".to_string()),
    ];
    for (id, line) in &cases {
        write(&root, &format!("{id}.md"), &nfr(id, line, None));
    }
    for posture in [BundlePosture::Okf, BundlePosture::Strict] {
        let report = validate_bundle_at(&root, &registry(), posture);
        for reason in [
            "unjustified-classified-nfr",
            "untargeted-security-nfr",
            "reliability-nfr-orphan",
            "undeclared-relation-vocabulary",
        ] {
            assert!(
                findings(&report, reason).is_empty(),
                "{reason} must not fire: {:?}",
                findings(&report, reason)
            );
        }
    }
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2250", "FR-058-AC-14")]
#[test]
fn tc2250_matching_trims_and_folds_ascii_case_on_both_sides() {
    let root = tmpdir("2250");
    write(
        &root,
        "N-001.md",
        &nfr("N-001", "quality_attribute: \" Security \"", None),
    );
    write(&root, "N-002.md", &nfr("N-002", &attr("SAFETY"), None));
    write(&root, "N-003.md", &nfr("N-003", &attr("secur1ty"), None));
    write(&root, "N-004.md", &nfr("N-004", &attr("security-x"), None));

    let report = run(&root, &registry());
    assert_eq!(
        ids(&report, "unjustified-classified-nfr"),
        vec!["N-001", "N-002"]
    );

    // Entries fold too: `values: [" Security "]` selects `security`.
    let dir = module_with(
        "2250",
        "    where: { field: quality_attribute, values: [\" SECURITY \"] }\n",
    );
    let reg = Registry::load_module(&dir).expect("load");
    write(&root, "N-005.md", &nfr("N-005", &attr("security"), None));
    let report = run(&root, &reg);
    assert!(ids(&report, "sel-check").contains(&"N-005".to_string()));
    assert!(!ids(&report, "sel-check").contains(&"N-002".to_string()));
    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&dir).ok();
}

#[trace("TC-2251", "FR-058-AC-16")]
#[test]
fn tc2251_exclude_exempts_a_selected_document() {
    let root = tmpdir("2251");
    write(
        &root,
        "legacy/N-001.md",
        &nfr("N-001", &attr("reliability"), None),
    );
    write(&root, "N-002.md", &nfr("N-002", &attr("reliability"), None));
    let report = run(&root, &registry());
    assert_eq!(ids(&report, "reliability-nfr-orphan"), vec!["N-002"]);
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2252", "FR-058-AC-17")]
#[test]
fn tc2252_dead_from_kind_reports_with_and_without_where() {
    let root = tmpdir("2252");
    write(&root, "N-001.md", &nfr("N-001", &attr("security"), None));
    let mut messages = Vec::new();
    for (tag, where_yaml) in [
        ("2252a", ""),
        (
            "2252b",
            "    where: { field: quality_attribute, values: [security] }\n",
        ),
    ] {
        let dir = module_with(tag, where_yaml);
        let manifest = fs::read_to_string(dir.join("manifest.yaml")).expect("read");
        write(
            &dir,
            "manifest.yaml",
            &manifest.replace("from: NFR", "from: NFRR"),
        );
        let report = run(&root, &Registry::load_module(&dir).expect("load"));
        let dead: Vec<_> = findings(&report, "undeclared-relation-vocabulary")
            .into_iter()
            .filter(|f| f.message.contains("from 'NFRR'"))
            .map(|f| f.message.clone())
            .collect();
        assert_eq!(dead.len(), 1, "{tag}: {report:?}");
        messages.push(dead[0].clone());
        fs::remove_dir_all(&dir).ok();
    }
    assert_eq!(messages[0], messages[1], "text is unchanged by `where`");
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2253", "FR-058-AC-18")]
#[test]
fn tc2253_where_findings_are_advisory_and_tunable_by_their_own_key() {
    let root = tmpdir("2253");
    write(&root, "N-001.md", &nfr("N-001", &attr("safety"), None));
    for posture in [BundlePosture::Okf, BundlePosture::Strict] {
        let report = validate_bundle_at(&root, &registry(), posture);
        assert_eq!(findings(&report, "unjustified-classified-nfr").len(), 1);
        assert!(report.is_valid(), "advisory: {:?}", report.errors);
    }
    let with = |level| {
        let mut map = GrammarSeverityMap::new();
        map.insert("trace:unjustified-classified-nfr".into(), level);
        run(&root, &registry().with_grammar_severity(map))
    };
    let off = with(GrammarSeverityLevel::Off);
    assert!(findings(&off, "unjustified-classified-nfr").is_empty());
    let promoted = with(GrammarSeverityLevel::Error);
    assert!(!promoted.is_valid());
    assert_eq!(
        promoted
            .errors
            .iter()
            .filter(|f| f.reason == "unjustified-classified-nfr")
            .count(),
        1
    );
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2254", "FR-058-AC-19")]
#[test]
fn tc2254_a_malformed_where_fails_module_load_naming_relation_and_where() {
    let bad = [
        "    where: security\n",
        "    where: null\n",
        "    where: { values: [security] }\n",
        "    where: { field: \"  \", values: [security] }\n",
        "    where: { field: 3, values: [security] }\n",
        "    where: { field: quality_attribute }\n",
        "    where: { field: quality_attribute, values: [] }\n",
        "    where: { field: quality_attribute, values: security }\n",
        "    where: { field: quality_attribute, values: [security, \" \"] }\n",
        "    where: { field: quality_attribute, values: [security, 4] }\n",
        "    where: { field: quality_attribute, values: [security, \" SECURITY\"] }\n",
        "    where: { field: quality_attribute, values: [security], extra: 1 }\n",
    ];
    for (n, yaml) in bad.iter().enumerate() {
        let dir = module_with(&format!("2254_{n}"), yaml);
        let err = Registry::load_module(&dir)
            .err()
            .unwrap_or_else(|| panic!("{yaml:?} must be rejected"))
            .to_string();
        assert!(
            err.contains("'sel'") && err.contains("`where`"),
            "{yaml:?} -> {err}"
        );
        fs::remove_dir_all(&dir).ok();
    }
}

#[trace("TC-2255", "FR-058-AC-20")]
#[test]
fn tc2255_the_finding_names_the_selection_and_unselected_text_is_unchanged() {
    let root = tmpdir("2255");
    write(
        &root,
        "N-001.md",
        &nfr("N-001", "quality_attribute: \" Security \"", None),
    );
    let report = run(&root, &registry());
    let hit = findings(&report, "unjustified-classified-nfr");
    assert_eq!(hit.len(), 1);
    let m = &hit[0].message;
    for part in [
        "N-001",
        "classified-nfr-needs-threat",
        "quality_attribute",
        "' Security '",
    ] {
        assert!(m.contains(part), "{part} missing from {m}");
    }

    // Without `where` the text is the pre-change sentence exactly.
    let dir = module_with("2255", "");
    let report = run(&root, &Registry::load_module(&dir).expect("load"));
    let hit = findings(&report, "sel-check");
    assert_eq!(hit.len(), 1);
    assert_eq!(
        hit[0].message,
        "'N-001' has no 'mitigates' edge to any Threat — declared by 'sel'"
    );
    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&dir).ok();
}

#[trace("TC-2256", "FR-058-AC-21")]
#[test]
fn tc2256_no_where_selects_everything_and_runs_are_deterministic() {
    let root = tmpdir("2256");
    write(&root, "N-002.md", &nfr("N-002", &attr("usability"), None));
    write(&root, "N-001.md", &nfr("N-001", "", None));
    let dir = module_with("2256", "");
    let reg = Registry::load_module(&dir).expect("load");
    assert_eq!(ids(&run(&root, &reg), "sel-check"), vec!["N-001", "N-002"]);

    write(&root, "N-003.md", &nfr("N-003", &attr("security"), None));
    write(&root, "N-004.md", &nfr("N-004", &attr("safety"), None));
    let first = format!("{:?}", run(&root, &registry()));
    for _ in 0..3 {
        assert_eq!(first, format!("{:?}", run(&root, &registry())));
    }
    let hits = findings(&run(&root, &registry()), "unjustified-classified-nfr")
        .iter()
        .map(|f| f.path.clone())
        .collect::<Vec<_>>();
    let mut sorted = hits.clone();
    sorted.sort();
    assert_eq!(hits, sorted, "document order");
    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&dir).ok();
}

#[trace("TC-2257", "FR-058-AC-22")]
#[test]
fn tc2257_a_where_that_selects_nothing_reports_itself() {
    let root = tmpdir("2257");
    write(&root, "N-001.md", &nfr("N-001", &attr("security"), None));
    write(&root, "N-002.md", &nfr("N-002", "owner: bob", None));
    let dead = |report: &BundleReport| -> Vec<String> {
        findings(report, "undeclared-relation-vocabulary")
            .iter()
            .map(|f| f.message.clone())
            .collect()
    };
    let load = |tag: &str, yaml: &str| {
        let dir = module_with(tag, yaml);
        let reg = Registry::load_module(&dir).expect("load");
        fs::remove_dir_all(&dir).ok();
        reg
    };

    // Live field and entries: nothing, even differently cased or padded.
    let live = load(
        "2257a",
        "    where: { field: quality_attribute, values: [Security, \" safety \"] }\n",
    );
    assert!(
        dead(&run(&root, &live)).is_empty(),
        "{:?}",
        dead(&run(&root, &live))
    );

    // Misspelt field: neither in the schema nor carried by a document.
    let bad_field = load(
        "2257b",
        "    where: { field: quality_atribute, values: [security] }\n",
    );
    let msgs = dead(&run(&root, &bad_field));
    assert_eq!(msgs.len(), 1, "{msgs:?}");
    assert!(msgs[0].contains("'sel'") && msgs[0].contains("quality_atribute"));

    // Field carried by a document but absent from the schema is live; its
    // values are unchecked because no enum exists.
    let carried = load("2257c", "    where: { field: owner, values: [anyone] }\n");
    assert!(dead(&run(&root, &carried)).is_empty());

    // Entry outside the enum and held by no document.
    let bad_value = load(
        "2257d",
        "    where: { field: quality_attribute, values: [security, secrity] }\n",
    );
    let msgs = dead(&run(&root, &bad_value));
    assert_eq!(msgs.len(), 1, "{msgs:?}");
    assert!(msgs[0].contains("'sel'") && msgs[0].contains("secrity"));

    // Advisory under both postures.
    for posture in [BundlePosture::Okf, BundlePosture::Strict] {
        let r = validate_bundle_at(&root, &bad_field, posture);
        assert!(r.is_valid(), "{:?}", r.errors);
    }
    // Outside the enum but held by a document: live by the second route.
    write(
        &root,
        "N-003.md",
        &nfr("N-003", "quality_attribute: \" Secrity \"", None),
    );
    assert!(dead(&run(&root, &bad_value)).is_empty());

    // "Held" compares folded too: a padded, upper-case entry is live through a
    // document holding the lower-case value.
    let folded_held = load(
        "2257e",
        "    where: { field: quality_attribute, values: [security, SECRITY] }\n",
    );
    assert!(dead(&run(&root, &folded_held)).is_empty());

    fs::remove_dir_all(&root).ok();
}
