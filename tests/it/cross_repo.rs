//! FR-085 — cross-repository edge resolution and corpus validation.
//!
//! Each repository is built on disk under `<DIR>/spec`, loaded as a corpus
//! member, and validated with the `(<DIR>/spec, <DIR>)` root pair.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::corpus::{
    validate_bundle, validate_corpus, CorpusError, CorpusMember, Resolution, Spec,
};
use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
use quire_rs::{BundleFinding, BundlePosture, BundleReport, Diagnostic, Registry};
use tempfile::TempDir;

/// A document: `relationships` are `(verb, target)` pairs, `body` free text.
fn doc(id: &str, ty: &str, relationships: &[(&str, &str)], body: &str) -> String {
    let mut front = format!("---\nid: {id}\ntype: {ty}\ntitle: {id}\n");
    if !relationships.is_empty() {
        front.push_str("relationships:\n");
        for (verb, target) in relationships {
            front.push_str(&format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"));
        }
    }
    format!("{front}---\n\n# {id}\n\n{body}\n")
}

/// A repository directory: `files` are `(path under spec/, text)`.
fn repo(files: &[(&str, String)]) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    for (rel, text) in files {
        let path = dir.path().join("spec").join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }
    dir
}

fn member(dir: &TempDir, identity: Option<&str>) -> CorpusMember {
    CorpusMember {
        identity: identity.map(str::to_string),
        load: quire_rs::corpus::walk::load_repo(&dir.path().join("spec")),
    }
}

fn corpus(members: &[(&TempDir, &str)]) -> Spec {
    Spec::from_members(members.iter().map(|(d, id)| member(d, Some(id))).collect())
        .expect("corpus builds")
}

fn registry() -> Registry {
    Registry::load_module(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/traceability/cross-repo"),
    )
    .expect("fixture module loads")
}

fn roots(dirs: &[&TempDir]) -> Vec<(PathBuf, PathBuf)> {
    dirs.iter()
        .map(|d| (d.path().join("spec"), d.path().to_path_buf()))
        .collect()
}

fn validate(corpus: &Spec, registry: &Registry, dirs: &[&TempDir]) -> BundleReport {
    let pairs = roots(dirs);
    let refs: Vec<(&Path, &Path)> = pairs
        .iter()
        .map(|(d, r)| (d.as_path(), r.as_path()))
        .collect();
    validate_corpus(corpus, registry, BundlePosture::Okf, &refs).expect("validates")
}

fn found<'r>(report: &'r BundleReport, reason: &str) -> Vec<&'r BundleFinding> {
    report
        .errors
        .iter()
        .chain(&report.warnings)
        .filter(|f| f.reason == reason)
        .collect()
}

/// o/b holds a hazard and a non-hazard note; o/a holds three requirements: one
/// that mitigates the hazard, one whose target id is absent from o/b, and one
/// whose target is the wrong kind.
fn two_repo_fixture() -> (TempDir, TempDir) {
    let a = repo(&[
        (
            "REQ-001.md",
            doc(
                "REQ-001",
                "Req",
                &[("mitigates", "ix://o/b/spec/HAZ-001")],
                "",
            ),
        ),
        (
            "REQ-002.md",
            doc(
                "REQ-002",
                "Req",
                &[("mitigates", "ix://o/b/spec/HAZ-404")],
                "",
            ),
        ),
        (
            "REQ-003.md",
            doc(
                "REQ-003",
                "Req",
                &[("mitigates", "ix://o/b/spec/NOTE-001")],
                "",
            ),
        ),
    ]);
    let b = repo(&[
        ("HAZ-001.md", doc("HAZ-001", "Haz", &[], "")),
        ("NOTE-001.md", doc("NOTE-001", "Note", &[], "")),
    ]);
    (a, b)
}

#[trace("FR-085-AC-1")]
#[test]
fn a_cross_member_edge_resolves_and_is_in_the_targets_incoming_set() {
    let (a, b) = two_repo_fixture();
    let corpus = corpus(&[(&a, "o/a"), (&b, "o/b")]);

    let out = corpus.outgoing("REQ-001");
    let edge = out
        .iter()
        .find(|e| e.edge_type == "mitigates")
        .expect("mitigates edge");
    assert_eq!(edge.resolution, Resolution::Resolved);
    assert_eq!(edge.source_repo.as_deref(), Some("o/a"));
    assert_eq!(edge.target_repo.as_deref(), Some("o/b"));

    let incoming = corpus.referencing("HAZ-001");
    let reached = incoming
        .iter()
        .find(|e| e.edge_type == "mitigates")
        .expect("in the target's incoming set");
    assert_eq!(reached.source, "REQ-001");
    assert_eq!(reached.source_repo.as_deref(), Some("o/a"));
}

#[trace("FR-085-AC-2")]
#[test]
fn a_cross_member_mitigation_satisfies_the_incoming_relation_and_needs_the_member() {
    let (a, b) = two_repo_fixture();
    let registry = registry();

    let both = validate(&corpus(&[(&a, "o/a"), (&b, "o/b")]), &registry, &[&a, &b]);
    let unmitigated: Vec<_> = found(&both, "unmitigated-haz");
    assert!(
        unmitigated.is_empty(),
        "HAZ-001 is mitigated: {unmitigated:?}"
    );

    // Without the mitigating member the hazard is unmitigated.
    let alone = validate(&corpus(&[(&b, "o/b")]), &registry, &[&b]);
    let unmitigated = found(&alone, "unmitigated-haz");
    assert_eq!(unmitigated.len(), 1, "{unmitigated:?}");
    assert!(unmitigated[0].message.contains("HAZ-001"));

    // Without o/b the requirement's edge leaves the corpus and cannot satisfy.
    let without_b = validate(&corpus(&[(&a, "o/a")]), &registry, &[&a]);
    let unmet: Vec<_> = found(&without_b, "req-no-haz")
        .into_iter()
        .map(|f| f.message.clone())
        .collect();
    assert!(unmet.iter().any(|m| m.contains("REQ-001")), "{unmet:?}");
}

#[trace("FR-085-AC-3")]
#[test]
fn a_cross_member_mitigation_satisfies_the_outgoing_relation() {
    let (a, b) = two_repo_fixture();
    let report = validate(&corpus(&[(&a, "o/a"), (&b, "o/b")]), &registry(), &[&a, &b]);
    let unmet: Vec<&str> = found(&report, "req-no-haz")
        .iter()
        .map(|f| f.message.as_str())
        .collect();
    assert!(!unmet.iter().any(|m| m.contains("'REQ-001'")), "{unmet:?}");
}

#[trace("FR-085-AC-4")]
#[test]
fn an_id_absent_from_a_loaded_member_is_dangling_and_named_as_authored() {
    let (a, b) = two_repo_fixture();
    let corpus = corpus(&[(&a, "o/a"), (&b, "o/b")]);

    let out = corpus.outgoing("REQ-002");
    let edge = out
        .iter()
        .find(|e| e.edge_type == "mitigates")
        .expect("mitigates edge");
    assert_eq!(edge.resolution, Resolution::Dangling);
    assert_eq!(edge.references, vec!["ix://o/b/spec/HAZ-404".to_string()]);

    let report = validate(&corpus, &registry(), &[&a, &b]);
    let unmet = found(&report, "req-no-haz");
    let named = unmet
        .iter()
        .find(|f| f.message.contains("'REQ-002'"))
        .expect("REQ-002 is unsatisfied");
    assert!(
        named.message.contains("'ix://o/b/spec/HAZ-404'"),
        "{}",
        named.message
    );
}

#[trace("FR-085-AC-5")]
#[test]
fn an_authority_no_member_declares_is_dangling_even_when_the_id_exists() {
    let a = repo(&[
        (
            "REQ-001.md",
            doc(
                "REQ-001",
                "Req",
                &[("mitigates", "ix://o/c/spec/HAZ-001")],
                "",
            ),
        ),
        ("HAZ-001.md", doc("HAZ-001", "Haz", &[], "")),
    ]);
    let b = repo(&[("HAZ-002.md", doc("HAZ-002", "Haz", &[], ""))]);
    let corpus = corpus(&[(&a, "o/a"), (&b, "o/b")]);
    assert!(corpus
        .outgoing("REQ-001")
        .iter()
        .all(|e| e.resolution == Resolution::Dangling));
}

#[trace("FR-085-AC-6")]
#[test]
fn a_wrong_kind_target_is_reported_as_a_kind_mismatch_across_and_within_members() {
    let (a, b) = two_repo_fixture();
    let report = validate(&corpus(&[(&a, "o/a"), (&b, "o/b")]), &registry(), &[&a, &b]);

    let mismatches = found(&report, "target-kind-mismatch");
    assert_eq!(mismatches.len(), 1, "{mismatches:?}");
    let message = &mismatches[0].message;
    for expected in [
        "REQ-003",
        "o/b:NOTE-001",
        "'Note'",
        "Haz",
        "req-mitigates",
        "mitigates",
    ] {
        assert!(
            message.contains(expected),
            "{expected} missing from {message}"
        );
    }
    // The mismatched edge does not satisfy the relation.
    assert!(found(&report, "req-no-haz")
        .iter()
        .any(|f| f.message.contains("'REQ-003'")));

    // The same finding within one member, naming no other repo.
    let same = repo(&[
        (
            "REQ-001.md",
            doc("REQ-001", "Req", &[("mitigates", "NOTE-001")], ""),
        ),
        ("NOTE-001.md", doc("NOTE-001", "Note", &[], "")),
    ]);
    let single = Spec::from_members(vec![member(&same, Some("o/a"))]).expect("corpus");
    let within = validate(&single, &registry(), &[&same]);
    let mismatches = found(&within, "target-kind-mismatch");
    assert_eq!(mismatches.len(), 1, "{mismatches:?}");
    assert!(!mismatches[0].message.contains("o/a:"));
}

#[trace("FR-085-AC-7")]
#[test]
fn an_unconstrained_to_has_no_kind_mismatch_and_the_finding_is_tunable() {
    // The same module plus a relation whose `to` is empty.
    let module = TempDir::new().expect("module dir");
    let manifest = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/traceability/cross-repo/manifest.yaml"),
    )
    .expect("manifest");
    fs::write(
        module.path().join("manifest.yaml"),
        format!(
            "{manifest}  - name: req-cites\n    from: Req\n    edges: [cites]\n    to: []\n    \
             direction: outgoing\n    check: req-no-cite\n"
        ),
    )
    .expect("write module");
    let open = Registry::load_module(module.path()).expect("module loads");
    let a = repo(&[
        (
            "REQ-001.md",
            doc("REQ-001", "Req", &[("cites", "NOTE-001")], ""),
        ),
        ("NOTE-001.md", doc("NOTE-001", "Note", &[], "")),
    ]);
    let single = Spec::from_members(vec![member(&a, Some("o/a"))]).expect("corpus");
    let report = validate(&single, &open, &[&a]);
    assert!(found(&report, "target-kind-mismatch").is_empty());
    let unmet: Vec<_> = found(&report, "req-no-cite");
    assert!(unmet.is_empty(), "the edge satisfies `to: []`: {unmet:?}");

    // Advisory by default, promotable through the FR-057 key.
    let (x, y) = two_repo_fixture();
    let both = corpus(&[(&x, "o/a"), (&y, "o/b")]);
    let default = validate(&both, &registry(), &[&x, &y]);
    assert!(default
        .errors
        .iter()
        .all(|f| f.reason != "target-kind-mismatch"));
    assert_eq!(
        default
            .warnings
            .iter()
            .filter(|f| f.reason == "target-kind-mismatch")
            .count(),
        1
    );

    let mut map = GrammarSeverityMap::new();
    map.insert(
        "trace:target-kind-mismatch".into(),
        GrammarSeverityLevel::Error,
    );
    let promoted = validate(&both, &registry().with_grammar_severity(map), &[&x, &y]);
    assert_eq!(
        promoted
            .errors
            .iter()
            .filter(|f| f.reason == "target-kind-mismatch")
            .count(),
        1
    );

    let mut map = GrammarSeverityMap::new();
    map.insert(
        "trace:target-kind-mismatch".into(),
        GrammarSeverityLevel::Off,
    );
    let off = validate(&both, &registry().with_grammar_severity(map), &[&x, &y]);
    assert!(found(&off, "target-kind-mismatch").is_empty());
}

#[trace("FR-085-AC-8")]
#[test]
fn bare_ids_and_relative_links_resolve_only_within_the_sources_member() {
    let a = repo(&[
        (
            "REQ-001.md",
            doc(
                "REQ-001",
                "Req",
                &[("mitigates", "HAZ-001")],
                "see [other repo](../../b/spec/HAZ-001.md) and [local](./REQ-002.md)",
            ),
        ),
        ("REQ-002.md", doc("REQ-002", "Req", &[], "")),
    ]);
    let b = repo(&[("HAZ-001.md", doc("HAZ-001", "Haz", &[], ""))]);
    let corpus = corpus(&[(&a, "o/a"), (&b, "o/b")]);
    let out = corpus.outgoing("REQ-001");

    let bare = out
        .iter()
        .find(|e| e.edge_type == "mitigates")
        .expect("bare");
    assert_eq!(
        bare.resolution,
        Resolution::Dangling,
        "o/a holds no HAZ-001"
    );
    let by_link = |target: &str| {
        out.iter()
            .find(|e| e.edge_type == "references" && e.target.ends_with(target))
            .unwrap_or_else(|| panic!("{target}: {out:?}"))
            .resolution
    };
    assert_eq!(by_link("REQ-002"), Resolution::Resolved);
    assert_eq!(by_link("HAZ-001.md"), Resolution::Dangling);
}

#[trace("FR-085-AC-9")]
#[test]
fn two_members_declaring_one_identity_build_no_corpus() {
    let (a, b) = two_repo_fixture();
    let err = Spec::from_members(vec![member(&a, Some("o/x")), member(&b, Some("o/x"))])
        .expect_err("duplicate identity");
    assert_eq!(
        err,
        CorpusError::DuplicateIdentity {
            identity: "o/x".into()
        }
    );
    assert!(err.to_string().contains("o/x"));
}

#[trace("FR-085-AC-10")]
#[test]
fn an_identity_less_single_member_corpus_resolves_lexically() {
    let a = repo(&[
        (
            "REQ-001.md",
            doc(
                "REQ-001",
                "Req",
                &[("mitigates", "ix://anyone/anything/spec/HAZ-001")],
                "",
            ),
        ),
        ("HAZ-001.md", doc("HAZ-001", "Haz", &[], "")),
    ]);
    let spec = Spec::from_members(vec![member(&a, None)]).expect("one member needs no identity");
    let out = spec.outgoing("REQ-001");
    assert!(out.iter().all(|e| e.resolution == Resolution::Resolved));
    assert!(spec
        .referencing("HAZ-001")
        .iter()
        .any(|e| e.edge_type == "mitigates"));
}

#[trace("FR-085-AC-11")]
#[test]
fn classification_is_independent_of_member_order_and_reads_nothing_after_build() {
    let (a, b) = two_repo_fixture();
    let classify = |spec: &Spec| {
        let mut rows: Vec<(String, String, String, bool)> = spec
            .edges()
            .iter()
            .map(|e| {
                (
                    e.source_repo.clone().unwrap_or_default(),
                    e.source.clone(),
                    e.target.clone(),
                    e.resolution == Resolution::Resolved,
                )
            })
            .collect();
        rows.sort();
        rows
    };
    let ab = corpus(&[(&a, "o/a"), (&b, "o/b")]);
    let ba = corpus(&[(&b, "o/b"), (&a, "o/a")]);
    // `edges()` of an unscoped corpus is the whole set in either order.
    assert_eq!(classify(&ab), classify(&ba));
    assert_eq!(
        classify(&ab),
        classify(&corpus(&[(&a, "o/a"), (&b, "o/b")]))
    );

    // No filesystem read after construction.
    drop((a, b));
    assert!(ab
        .outgoing("REQ-001")
        .iter()
        .all(|e| e.resolution == Resolution::Resolved));
    assert!(ab.by_qualified_id("o/b", "HAZ-001").is_some());
}

#[trace("FR-085-AC-12")]
#[test]
fn one_corpus_yields_each_of_the_three_outcomes_and_no_other() {
    let (a, b) = two_repo_fixture();
    let report = validate(&corpus(&[(&a, "o/a"), (&b, "o/b")]), &registry(), &[&a, &b]);

    let mut summary: Vec<(String, String)> = report
        .errors
        .iter()
        .chain(&report.warnings)
        .map(|f| {
            let subject = [
                "REQ-001", "REQ-002", "REQ-003", "HAZ-001", "NOTE-001", "HAZ-404",
            ]
            .into_iter()
            .find(|id| f.message.contains(&format!("'{id}'")) || f.message.contains(id))
            .unwrap_or("?");
            (f.reason.to_string(), subject.to_string())
        })
        .collect();
    summary.sort();
    assert_eq!(
        summary,
        vec![
            // the dangling edge: the reference itself (one per spelling of
            // the edge: the `relationships` entry and the `ix://` link
            // the frontmatter text also carries), and the relation
            ("dangling-reference".to_string(), "HAZ-404".to_string()),
            ("dangling-reference".to_string(), "HAZ-404".to_string()),
            // the wrong-kind edge
            ("req-no-haz".to_string(), "REQ-002".to_string()),
            ("req-no-haz".to_string(), "REQ-003".to_string()),
            ("target-kind-mismatch".to_string(), "REQ-003".to_string()),
            // the satisfied edge produces nothing: no REQ-001 or HAZ-001 row
        ]
    );
}

#[trace("FR-085-AC-13")]
#[test]
fn a_multi_member_corpus_requires_every_identity() {
    let (a, b) = two_repo_fixture();
    let err = Spec::from_members(vec![member(&a, Some("o/a")), member(&b, None)])
        .expect_err("missing identity");
    assert_eq!(err, CorpusError::MissingIdentity { member: 1 });
    let both = Spec::from_members(vec![member(&a, None), member(&b, None)]).expect_err("both");
    assert_eq!(both, CorpusError::MissingIdentity { member: 0 });
}

#[trace("FR-085-AC-14")]
#[test]
fn a_single_member_with_identity_resolves_only_its_own_authority() {
    let a = repo(&[
        (
            "REQ-001.md",
            doc(
                "REQ-001",
                "Req",
                &[
                    ("mitigates", "ix://o/a/spec/HAZ-001"),
                    ("cites", "ix://o/b/spec/HAZ-001"),
                ],
                "",
            ),
        ),
        ("HAZ-001.md", doc("HAZ-001", "Haz", &[], "")),
    ]);
    let spec = Spec::from_members(vec![member(&a, Some("o/a"))]).expect("corpus");
    let out = spec.outgoing("REQ-001");
    let of = |verb: &str| {
        out.iter()
            .find(|e| e.edge_type == verb)
            .expect(verb)
            .resolution
    };
    assert_eq!(of("mitigates"), Resolution::Resolved);
    assert_eq!(of("cites"), Resolution::Dangling);
}

#[trace("FR-085-AC-15", "FR-027-AC-10", "FR-025-AC-3")]
#[test]
fn an_id_held_by_two_members_is_not_a_duplicate_and_by_id_does_not_pick() {
    let a = repo(&[
        ("X.md", doc("X", "Req", &[], "")),
        ("Y.md", doc("Y", "Req", &[("cites", "X")], "")),
    ]);
    let b = repo(&[
        ("X.md", doc("X", "Haz", &[], "")),
        ("Z.md", doc("Z", "Haz", &[], "")),
    ]);
    let spec = corpus(&[(&a, "o/a"), (&b, "o/b")]);

    assert!(!spec
        .diagnostics()
        .iter()
        .any(|d| matches!(d, Diagnostic::DuplicateArtifactId { .. })));
    assert!(spec.by_id("X").is_none(), "ambiguous id answers nothing");
    assert!(
        spec.by_id("Z").is_some(),
        "an id held by one member answers"
    );
    let in_a = spec.by_qualified_id("o/a", "X").expect("o/a X");
    let in_b = spec.by_qualified_id("O/B", "X").expect("o/b X");
    assert_eq!(in_a.concept_type(), Some("Req"));
    assert_eq!(in_b.concept_type(), Some("Haz"));
    assert!(spec.by_qualified_id("o/c", "X").is_none());

    // A bare `X` in o/a resolves to o/a's X.
    let edge = &spec.edges().iter().find(|e| e.source == "Y").expect("edge");
    assert_eq!(edge.target_repo.as_deref(), Some("o/a"));

    // Within one member a repeated id is still a duplicate.
    let dup = repo(&[
        ("p/X.md", doc("X", "Req", &[], "")),
        ("q/X.md", doc("X", "Req", &[], "")),
    ]);
    let single = Spec::from_members(vec![member(&dup, Some("o/d"))]).expect("corpus");
    assert_eq!(
        single
            .diagnostics()
            .iter()
            .filter(|d| matches!(d, Diagnostic::DuplicateArtifactId { .. }))
            .count(),
        1
    );
}

#[trace("FR-085-AC-16")]
#[test]
fn validate_corpus_resolves_edges_across_members() {
    let (a, b) = two_repo_fixture();
    let registry = registry();
    let refs = |report: &BundleReport| -> Vec<String> {
        found(report, "dangling-reference")
            .iter()
            .map(|f| f.message.clone())
            .collect()
    };
    let with_b = refs(&validate(
        &corpus(&[(&a, "o/a"), (&b, "o/b")]),
        &registry,
        &[&a, &b],
    ));
    assert!(!with_b.iter().any(|m| m.contains("HAZ-001")), "{with_b:?}");
    assert!(!with_b.iter().any(|m| m.contains("NOTE-001")), "{with_b:?}");

    // o/b loaded but no longer holding the hazard: the miss is dangling.
    let b_without = repo(&[("NOTE-001.md", doc("NOTE-001", "Note", &[], ""))]);
    let without_hazard = refs(&validate(
        &corpus(&[(&a, "o/a"), (&b_without, "o/b")]),
        &registry,
        &[&a, &b_without],
    ));
    assert!(
        without_hazard.iter().any(|m| m.contains("HAZ-001")),
        "{without_hazard:?}"
    );
}

#[trace("FR-085-AC-17")]
#[test]
fn a_miss_in_a_loaded_member_is_dangling_but_an_unloaded_authority_is_not() {
    let a = repo(&[(
        "REQ-001.md",
        doc(
            "REQ-001",
            "Req",
            &[],
            "[in b](ix://o/b/spec/HAZ-404) and [elsewhere](ix://o/c/spec/HAZ-405)",
        ),
    )]);
    let b = repo(&[("HAZ-001.md", doc("HAZ-001", "Haz", &[], ""))]);
    let report = validate(&corpus(&[(&a, "o/a"), (&b, "o/b")]), &registry(), &[&a, &b]);
    let messages: Vec<&str> = found(&report, "dangling-reference")
        .iter()
        .map(|f| f.message.as_str())
        .collect();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("HAZ-404"));
}

#[trace("FR-085-AC-18")]
#[test]
fn identity_matching_folds_ascii_case() {
    let (a, b) = two_repo_fixture();
    // `ix://o/b/...` resolves to the member declared `O/B`.
    let spec = corpus(&[(&a, "o/a"), (&b, "O/B")]);
    let edge = spec
        .outgoing("REQ-001")
        .into_iter()
        .find(|e| e.edge_type == "mitigates")
        .expect("edge");
    assert_eq!(edge.resolution, Resolution::Resolved);
    assert_eq!(edge.target_repo.as_deref(), Some("O/B"));

    let err = Spec::from_members(vec![member(&a, Some("o/b")), member(&b, Some("O/B"))])
        .expect_err("case-only difference is a duplicate");
    assert!(matches!(err, CorpusError::DuplicateIdentity { .. }));
}

#[trace("FR-085-AC-19")]
#[test]
fn every_finding_names_the_member_of_its_source_document() {
    let (a, b) = two_repo_fixture();
    let report = validate(&corpus(&[(&a, "o/a"), (&b, "o/b")]), &registry(), &[&a, &b]);
    for reason in ["req-no-haz", "target-kind-mismatch", "dangling-reference"] {
        let hits = found(&report, reason);
        assert!(!hits.is_empty(), "{reason}");
        assert!(
            hits.iter().all(|f| f.repo.as_deref() == Some("o/a")),
            "{reason}: {hits:?}"
        );
    }

    // A hazard finding is attributed to o/b.
    let lonely = repo(&[("HAZ-009.md", doc("HAZ-009", "Haz", &[], ""))]);
    let report = validate(
        &corpus(&[(&a, "o/a"), (&lonely, "o/l")]),
        &registry(),
        &[&a, &lonely],
    );
    let hazard = found(&report, "unmitigated-haz");
    assert_eq!(hazard.len(), 1);
    assert_eq!(hazard[0].repo.as_deref(), Some("o/l"));

    // A member with no identity carries none.
    let single = Spec::from_members(vec![member(&lonely, None)]).expect("corpus");
    let report = validate(&single, &registry(), &[&lonely]);
    let hazard = found(&report, "unmitigated-haz");
    assert_eq!(hazard.len(), 1);
    assert_eq!(hazard[0].repo, None);
}

#[trace("FR-085-AC-20")]
#[test]
fn a_single_identity_less_member_validates_as_validate_bundle_does() {
    let (a, _b) = two_repo_fixture();
    let registry = registry();
    let single = Spec::from_members(vec![member(&a, None)]).expect("corpus");
    let via_corpus = validate(&single, &registry, &[&a]);
    let via_bundle = validate_bundle(
        &single,
        &registry,
        BundlePosture::Okf,
        &a.path().join("spec"),
        a.path(),
    );
    assert_eq!(via_corpus, via_bundle);
    assert!(
        !via_bundle.warnings.is_empty(),
        "the comparison is not vacuous"
    );
}

#[trace("FR-085-AC-21")]
#[test]
fn model_paths_resolve_against_each_members_own_reference_root() {
    // Both members keep an unmitigated requirement under `spec/fixtures/`,
    // which the model excludes by a path relative to the member's repository
    // root. Each member's own root excludes its own copy.
    let fixture = || {
        repo(&[
            ("fixtures/REQ-900.md", doc("REQ-900", "Req", &[], "")),
            (
                "REQ-001.md",
                doc("REQ-001", "Req", &[("cites", "REQ-900")], ""),
            ),
        ])
    };
    let (a, b) = (fixture(), fixture());
    let spec = corpus(&[(&a, "o/a"), (&b, "o/b")]);
    let registry = registry();

    let own = validate(&spec, &registry, &[&a, &b]);
    let unmet = found(&own, "req-no-haz");
    assert!(
        !unmet.iter().any(|f| f.message.contains("REQ-900")),
        "{unmet:?}"
    );
    assert_eq!(unmet.len(), 2, "REQ-001 in each member: {unmet:?}");

    // Control: a member handed another member's root does not borrow the
    // exclusion, so its own fixture is reported.
    let pairs = [
        (a.path().join("spec"), a.path().to_path_buf()),
        (b.path().join("spec"), a.path().to_path_buf()),
    ];
    let refs: Vec<(&Path, &Path)> = pairs
        .iter()
        .map(|(d, r)| (d.as_path(), r.as_path()))
        .collect();
    let crossed = validate_corpus(&spec, &registry, BundlePosture::Okf, &refs).expect("validates");
    let crossed_unmet = found(&crossed, "req-no-haz");
    assert!(
        crossed_unmet.iter().any(|f| f.message.contains("REQ-900")),
        "{crossed_unmet:?}"
    );
}

#[test]
fn validate_corpus_needs_one_root_pair_per_member() {
    let (a, b) = two_repo_fixture();
    let spec = corpus(&[(&a, "o/a"), (&b, "o/b")]);
    let only = (a.path().join("spec"), a.path().to_path_buf());
    let err = validate_corpus(
        &spec,
        &registry(),
        BundlePosture::Okf,
        &[(only.0.as_path(), only.1.as_path())],
    )
    .expect_err("one pair for two members");
    assert_eq!(
        err,
        CorpusError::RootCount {
            members: 2,
            roots: 1
        }
    );
}
