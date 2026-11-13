//! The two-way relation coverage report.
//!
//! Every kind, verb and field below is authored in the fixture module
//! (`tests/fixtures/traceability/relation-report`); the engine knows none.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::corpus::{
    relation_report, validate_bundle, CorpusMember, LinkResolution, LinkedEntry, RelationReport,
    RelationReportError, RelationReportMember, RowState, Spec, Verification,
};
use quire_rs::coverage::{
    compute, CoverageMatrixCriterion, CoverageMatrixRequirement, CoverageMatrixStatus,
};
use quire_rs::symbols::{extract_tree, trace as sym_trace};
use quire_rs::{BundlePosture, Registry};
use tempfile::TempDir;

fn module(name: &str) -> Registry {
    Registry::load_module(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/traceability")
            .join(name),
    )
    .expect("fixture module loads")
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

fn relationships(rels: &[(&str, &str)]) -> String {
    if rels.is_empty() {
        return String::new();
    }
    let mut out = String::from("relationships:\n");
    for (verb, target) in rels {
        out.push_str(&format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"));
    }
    out
}

fn concern(id: &str, rels: &[(&str, &str)]) -> String {
    format!(
        "---\nid: {id}\ntype: Concern\ntitle: {id}\n{}---\n\n# {id}\n",
        relationships(rels)
    )
}

/// A `Rule` with `tier` (verbatim, or absent) and criteria `(id, method)`.
fn rule(id: &str, tier: Option<&str>, rels: &[(&str, &str)], criteria: &[(&str, &str)]) -> String {
    let tier = tier.map(|t| format!("tier: {t}\n")).unwrap_or_default();
    let mut text = format!(
        "---\nid: {id}\ntype: Rule\ntitle: {id}\n{tier}{}---\n\n# {id}\n",
        relationships(rels)
    );
    if !criteria.is_empty() {
        text.push_str(
            "\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|----|----------|--------------|\n",
        );
        for (ac, method) in criteria {
            text.push_str(&format!("| {ac} | The system shall hold. | {method} |\n"));
        }
    }
    text
}

fn simple(id: &str, ty: &str, rels: &[(&str, &str)]) -> String {
    format!(
        "---\nid: {id}\ntype: {ty}\ntitle: {id}\n{}---\n\n# {id}\n",
        relationships(rels)
    )
}

/// A repository on disk: documents under `spec/<id>.md`, and one test per
/// tagged criterion under `src/lib.rs`.
fn repo(docs: &[(&str, String)], tagged: &[&str]) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    for (id, text) in docs {
        write(dir.path(), &format!("spec/{id}.md"), text);
    }
    let mut src = String::from("#[cfg(test)]\nmod tests {\n");
    for (n, ac) in tagged.iter().enumerate() {
        src.push_str(&format!(
            "    #[trace(\"{ac}\")]\n    #[test]\n    fn t{n}() {{}}\n"
        ));
    }
    src.push_str("}\n");
    write(dir.path(), "src/lib.rs", &src);
    dir
}

/// The real coverage matrix of `dir`, then the report over it.
fn report_of(dir: &TempDir, registry: &Registry, names: &[&str]) -> RelationReport {
    let spec = Spec::from_path(&dir.path().join("spec"));
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = sym_trace::bind(&extract_tree(&dir.path().join("src")), &model);
    let coverage = compute(&spec, registry, &graph, dir.path()).expect("model declared");
    let names: Vec<String> = names.iter().map(|n| n.to_string()).collect();
    relation_report(
        &spec,
        registry,
        &[RelationReportMember {
            root: dir.path(),
            matrix: &coverage.coverage_matrix,
        }],
        &names,
    )
    .expect("report")
}

/// A synthetic matrix entry for `spec/<id>.md`.
fn entry(id: &str, statuses: &[CoverageMatrixStatus]) -> CoverageMatrixRequirement {
    CoverageMatrixRequirement {
        document: format!("spec/{id}.md"),
        criticality: None,
        criteria: statuses
            .iter()
            .enumerate()
            .map(|(n, status)| CoverageMatrixCriterion {
                id: format!("{id}-AC-{n}"),
                statement: "The system shall hold.".into(),
                method: None,
                binders: Vec::new(),
                record_binders: None,
                identity: None,
                status: *status,
            })
            .collect(),
    }
}

fn report_over(
    dir: &TempDir,
    registry: &Registry,
    matrix: &[CoverageMatrixRequirement],
) -> RelationReport {
    let spec = Spec::from_path(&dir.path().join("spec"));
    relation_report(
        &spec,
        registry,
        &[RelationReportMember {
            root: dir.path(),
            matrix,
        }],
        &[],
    )
    .expect("report")
}

fn section<'r>(report: &'r RelationReport, name: &str) -> &'r quire_rs::corpus::RelationSection {
    report
        .relations
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no section {name}"))
}

fn row<'r>(report: &'r RelationReport, name: &str, id: &str) -> &'r quire_rs::corpus::RelationRow {
    section(report, name)
        .rows
        .iter()
        .find(|r| r.document.id == id)
        .unwrap_or_else(|| panic!("no row {id} in {name}"))
}

fn linked_ids(row: &quire_rs::corpus::RelationRow) -> Vec<String> {
    row.linked
        .iter()
        .map(|l| match l {
            LinkedEntry::Resolved { document, .. } => document.id.clone(),
            LinkedEntry::Dangling { reference, .. } => format!("dangling:{reference}"),
        })
        .collect()
}

use CoverageMatrixStatus as S;

#[trace("FR-084-AC-1")]
#[trace("FR-084-AC-14")]
#[test]
fn rows_are_uncovered_verified_and_unverified_from_the_real_matrix() {
    // CON-001 nothing; CON-002 addressed by a fully tagged rule; CON-003 by an
    // untagged one.
    let dir = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            ("CON-002", concern("CON-002", &[])),
            ("CON-003", concern("CON-003", &[])),
            (
                "RULE-001",
                rule(
                    "RULE-001",
                    None,
                    &[("addresses", "CON-002")],
                    &[("RULE-001-AC-1", "Test"), ("RULE-001-AC-2", "Test")],
                ),
            ),
            (
                "RULE-002",
                rule(
                    "RULE-002",
                    None,
                    &[("addresses", "CON-003")],
                    &[("RULE-002-AC-1", "Test")],
                ),
            ),
        ],
        &["RULE-001-AC-1", "RULE-001-AC-2"],
    );
    let report = report_of(&dir, &module("relation-report"), &["concern-is-addressed"]);
    let rows = &section(&report, "concern-is-addressed").rows;
    let got: Vec<_> = rows
        .iter()
        .map(|r| (r.document.id.as_str(), r.state, r.verification))
        .collect();
    assert_eq!(
        got,
        vec![
            ("CON-001", RowState::Uncovered, Verification::NotTracked),
            ("CON-002", RowState::Covered, Verification::Verified),
            ("CON-003", RowState::Covered, Verification::Unverified),
        ]
    );
}

#[trace("FR-084-AC-2")]
#[test]
fn linked_lists_accepted_counterparts_ordered_and_skips_other_verbs() {
    let dir = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            (
                "RULE-002",
                rule("RULE-002", None, &[("addresses", "CON-001")], &[]),
            ),
            (
                "RULE-001",
                rule("RULE-001", None, &[("addresses", "CON-001")], &[]),
            ),
            (
                "RULE-003",
                rule("RULE-003", None, &[("mentions", "CON-001")], &[]),
            ),
        ],
        &[],
    );
    let matrix = [
        entry("RULE-001", &[S::Tagged]),
        entry("RULE-002", &[S::Untagged]),
    ];
    let report = report_over(&dir, &module("relation-report"), &matrix);
    let r = row(&report, "concern-is-addressed", "CON-001");
    assert_eq!(linked_ids(r), vec!["RULE-001", "RULE-002"]);
    let LinkedEntry::Resolved { document, verb } = &r.linked[1] else {
        panic!("resolved entry")
    };
    assert_eq!(r.linked[1].resolution(), LinkResolution::Resolved);
    assert_eq!(document.archetype.as_deref(), Some("Rule"));
    assert_eq!(document.verification, Verification::Unverified);
    assert_eq!(verb, "addresses");
}

#[trace("FR-084-AC-3")]
#[trace("FR-084-AC-4")]
#[test]
fn reverse_view_rows_follow_the_selector_and_carry_the_authored_value() {
    let dir = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            ("RULE-001", rule("RULE-001", Some("gold"), &[], &[])),
            (
                "RULE-002",
                rule("RULE-002", Some("Gold"), &[("addresses", "CON-001")], &[]),
            ),
            ("RULE-003", rule("RULE-003", Some("bronze"), &[], &[])),
            ("RULE-004", rule("RULE-004", None, &[], &[])),
        ],
        &[],
    );
    let report = report_over(&dir, &module("relation-report"), &[]);
    let rows = &section(&report, "gold-rule-addresses").rows;
    let got: Vec<_> = rows
        .iter()
        .map(|r| (r.document.id.as_str(), r.state))
        .collect();
    assert_eq!(
        got,
        vec![
            ("RULE-001", RowState::Uncovered),
            ("RULE-002", RowState::Covered)
        ]
    );
    let selector = rows[1].selector.as_ref().expect("selector");
    assert_eq!(selector.field, "tier");
    assert_eq!(selector.value, serde_json::json!("Gold"));

    // A relation without `where` serializes no `selector` key.
    let json = serde_json::to_value(&report).expect("json");
    let forward = json["relations"][0]["rows"][0].as_object().expect("row");
    assert!(!forward.contains_key("selector"));
    assert!(json["relations"][1]["rows"][0]
        .as_object()
        .expect("row")
        .contains_key("selector"));
}

#[trace("FR-084-AC-5")]
#[test]
fn document_verification_derives_from_criterion_statuses() {
    let dir = repo(
        &[
            ("RULE-001", rule("RULE-001", Some("gold"), &[], &[])),
            ("RULE-002", rule("RULE-002", Some("gold"), &[], &[])),
            ("RULE-003", rule("RULE-003", Some("gold"), &[], &[])),
            ("RULE-004", rule("RULE-004", Some("gold"), &[], &[])),
            ("RULE-005", rule("RULE-005", Some("gold"), &[], &[])),
            ("RULE-006", rule("RULE-006", Some("gold"), &[], &[])),
            ("RULE-007", rule("RULE-007", Some("gold"), &[], &[])),
        ],
        &[],
    );
    let matrix = [
        entry("RULE-001", &[S::Tagged, S::QualifiedRecord]),
        entry("RULE-002", &[S::Tagged, S::Untagged]),
        entry("RULE-003", &[S::Untagged, S::TaggedByIgnoredTest]),
        entry("RULE-004", &[S::Retired, S::Tagged]),
        entry("RULE-005", &[S::Retired]),
        // RULE-006 has no matrix entry at all.
        entry("RULE-007", &[]),
    ];
    let report = report_over(&dir, &module("relation-report"), &matrix);
    let status = |id: &str| {
        row(&report, "gold-rule-addresses", id)
            .document
            .verification
    };
    assert_eq!(status("RULE-001"), Verification::Verified);
    assert_eq!(status("RULE-002"), Verification::Partial);
    assert_eq!(status("RULE-003"), Verification::Unverified);
    assert_eq!(status("RULE-004"), Verification::Verified);
    assert_eq!(status("RULE-005"), Verification::NotTracked);
    assert_eq!(status("RULE-006"), Verification::NotTracked);
    assert_eq!(status("RULE-007"), Verification::NotTracked);
    let counts = row(&report, "gold-rule-addresses", "RULE-004")
        .document
        .criteria;
    assert_eq!(
        (counts.backed, counts.unbacked, counts.not_measured),
        (1, 0, 0)
    );
}

#[trace("FR-084-AC-6")]
#[test]
fn the_aggregate_ignores_not_tracked_documents_when_any_is_tracked() {
    let dir = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            ("CON-002", concern("CON-002", &[])),
            ("CON-003", concern("CON-003", &[])),
            (
                "RULE-001",
                rule(
                    "RULE-001",
                    None,
                    &[("addresses", "CON-001"), ("addresses", "CON-002")],
                    &[],
                ),
            ),
            (
                "RULE-002",
                rule(
                    "RULE-002",
                    None,
                    &[("addresses", "CON-002"), ("addresses", "CON-003")],
                    &[],
                ),
            ),
            (
                "RULE-003",
                rule("RULE-003", None, &[("addresses", "CON-002")], &[]),
            ),
            (
                "RULE-004",
                rule("RULE-004", None, &[("addresses", "CON-003")], &[]),
            ),
        ],
        &[],
    );
    // RULE-001 verified, RULE-002 unverified, RULE-003 verified, RULE-004 untracked.
    let matrix = [
        entry("RULE-001", &[S::Tagged]),
        entry("RULE-002", &[S::Untagged]),
        entry("RULE-003", &[S::Tagged]),
    ];
    let report = report_over(&dir, &module("relation-report"), &matrix);
    let agg = |id: &str| {
        let r = row(&report, "concern-is-addressed", id);
        (r.state, r.verification)
    };
    // Only a verified rule links to CON-001, and CON-003's rules are one
    // unverified and one not tracked.
    assert_eq!(agg("CON-001"), (RowState::Covered, Verification::Verified));
    assert_eq!(agg("CON-002"), (RowState::Covered, Verification::Partial));
    assert_eq!(
        agg("CON-003"),
        (RowState::Covered, Verification::Unverified)
    );

    // Linked only to untracked documents: covered and not-tracked.
    let only_untracked = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            (
                "RULE-001",
                rule("RULE-001", None, &[("addresses", "CON-001")], &[]),
            ),
        ],
        &[],
    );
    let report = report_over(&only_untracked, &module("relation-report"), &[]);
    let r = row(&report, "concern-is-addressed", "CON-001");
    assert_eq!(
        (r.state, r.verification),
        (RowState::Covered, Verification::NotTracked)
    );
}

#[trace("FR-084-AC-7")]
#[test]
fn reverse_view_aggregate_is_the_obliged_requirements_own_status() {
    let dir = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            (
                "RULE-001",
                rule("RULE-001", Some("gold"), &[("addresses", "CON-001")], &[]),
            ),
        ],
        &[],
    );
    let matrix = [entry("RULE-001", &[S::Tagged])];
    let report = report_over(&dir, &module("relation-report"), &matrix);
    let r = row(&report, "gold-rule-addresses", "RULE-001");
    assert_eq!(
        (r.state, r.verification),
        (RowState::Covered, Verification::Verified)
    );
    assert_eq!(linked_ids(r), vec!["CON-001"]);
}

#[trace("FR-084-AC-8")]
#[test]
fn a_module_without_an_obligations_source_reports_every_row_not_tracked() {
    let dir = repo(
        &[
            ("HAZ-001", simple("HAZ-001", "Haz", &[])),
            ("HAZ-002", simple("HAZ-002", "Haz", &[])),
            (
                "REQ-001",
                simple("REQ-001", "Req", &[("mitigates", "HAZ-002")]),
            ),
        ],
        &[],
    );
    // The cross-repo fixture declares relations but no `obligations:` source.
    let report = report_of(&dir, &module("cross-repo"), &[]);
    assert!(report.relations.iter().any(|s| !s.rows.is_empty()));
    for s in &report.relations {
        assert!(
            s.rows
                .iter()
                .all(|r| r.verification == Verification::NotTracked),
            "{}: {:?}",
            s.name,
            s.rows
        );
    }
}

#[trace("FR-084-AC-9")]
#[test]
fn naming_an_undeclared_relation_is_an_error_and_a_declared_one_is_alone() {
    let dir = repo(&[("CON-001", concern("CON-001", &[]))], &[]);
    let registry = module("relation-report");
    let spec = Spec::from_path(&dir.path().join("spec"));
    let member = [RelationReportMember {
        root: dir.path(),
        matrix: &[],
    }];
    let err = relation_report(
        &spec,
        &registry,
        &member,
        &["concern-is-addressed".into(), "no-such-relation".into()],
    )
    .expect_err("unknown relation");
    assert_eq!(
        err,
        RelationReportError::UnknownRelation {
            name: "no-such-relation".into()
        }
    );
    assert!(err.to_string().contains("no-such-relation"));

    let one = relation_report(&spec, &registry, &member, &["memo-cites".into()]).expect("report");
    assert_eq!(one.relations.len(), 1);
    assert_eq!(one.relations[0].name, "memo-cites");
}

#[trace("FR-084-AC-10")]
#[trace("FR-084-AC-11")]
#[test]
fn an_empty_selection_has_a_zero_summary_and_counts_sum_to_the_rows() {
    let dir = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            ("CON-002", concern("CON-002", &[])),
            ("CON-003", concern("CON-003", &[])),
            (
                "RULE-001",
                rule("RULE-001", None, &[("addresses", "CON-002")], &[]),
            ),
            (
                "RULE-002",
                rule("RULE-002", None, &[("addresses", "CON-003")], &[]),
            ),
        ],
        &[],
    );
    let matrix = [
        entry("RULE-001", &[S::Tagged]),
        entry("RULE-002", &[S::Untagged]),
    ];
    let report = report_over(&dir, &module("relation-report"), &matrix);
    // No Memo exists: the relation is present with zero rows.
    let memo = section(&report, "memo-cites");
    assert!(memo.rows.is_empty());
    assert_eq!(memo.summary.rows, 0);
    assert_eq!(memo.summary.state.covered + memo.summary.state.uncovered, 0);

    let s = section(&report, "concern-is-addressed").summary;
    assert_eq!(s.rows, 3);
    assert_eq!((s.state.uncovered, s.state.covered), (1, 2));
    assert_eq!(
        (
            s.verification.verified,
            s.verification.partial,
            s.verification.unverified,
            s.verification.not_tracked
        ),
        (1, 0, 1, 1)
    );
    assert_eq!(s.state.uncovered + s.state.covered, s.rows);
    let v = s.verification;
    assert_eq!(
        v.verified + v.partial + v.unverified + v.not_tracked,
        s.rows
    );
}

/// Two members holding every serialized shape: a selector row with a resolved
/// cross-member link, a dangling edge and a kind mismatch; forward rows from
/// both members; open-relation links (resolved, untyped, dangling).
fn two_member_corpus() -> (TempDir, TempDir, Spec) {
    let a = repo(
        &[
            (
                "RULE-001",
                rule(
                    "RULE-001",
                    Some("Gold"),
                    &[
                        ("addresses", "ix://o/b/spec/CON-001"),
                        ("addresses", "ghost"),
                        ("addresses", "RULE-002"),
                    ],
                    &[],
                ),
            ),
            ("RULE-002", rule("RULE-002", None, &[], &[])),
            ("CON-009", concern("CON-009", &[])),
            (
                "MEMO-001",
                simple(
                    "MEMO-001",
                    "Memo",
                    &[
                        ("cites", "ghost-b"),
                        ("cites", "ghost-a"),
                        ("cites", "ix://o/b/spec/UNT-001"),
                        ("cites", "ix://o/b/spec/CON-001"),
                    ],
                ),
            ),
        ],
        &[],
    );
    let b = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            (
                "RULE-005",
                rule("RULE-005", None, &[("addresses", "CON-001")], &[]),
            ),
            (
                "UNT-001",
                "---\nid: UNT-001\ntitle: untyped\n---\n\n# UNT-001\n".into(),
            ),
        ],
        &[],
    );
    let corpus = Spec::from_members(
        [(&a, "o/a"), (&b, "o/b")]
            .iter()
            .map(|(d, id)| CorpusMember {
                identity: Some((*id).to_string()),
                load: quire_rs::corpus::walk::load_repo(&d.path().join("spec")),
            })
            .collect(),
    )
    .expect("corpus");
    (a, b, corpus)
}

#[trace("FR-084-AC-12")]
#[trace("FR-084-AC-2")]
#[test]
fn serialization_is_exactly_the_stated_shape_and_order_across_members() {
    let (a, b, corpus) = two_member_corpus();
    let matrix = [entry("RULE-001", &[S::Tagged])];
    let render = || {
        let report = relation_report(
            &corpus,
            &module("relation-report"),
            &[
                RelationReportMember {
                    root: a.path(),
                    matrix: &matrix,
                },
                RelationReportMember {
                    root: b.path(),
                    matrix: &[],
                },
            ],
            &[],
        )
        .expect("report");
        serde_json::to_string(&report).expect("json")
    };
    let first = render();
    assert_eq!(first, render(), "byte-identical across runs");
    assert_eq!(first, EXPECTED_TWO_MEMBER_JSON);
}

const EXPECTED_TWO_MEMBER_JSON: &str = r#"{"relations":[{"name":"concern-is-addressed","check":"unaddressed-concern","direction":"incoming","summary":{"rows":2,"state":{"uncovered":1,"covered":1},"verification":{"verified":1,"partial":0,"unverified":0,"not-tracked":1}},"rows":[{"relation":"concern-is-addressed","check":"unaddressed-concern","direction":"incoming","document":{"id":"CON-009","path":"spec/CON-009.md","archetype":"Concern","repo":"o/a","verification":"not-tracked","criteria":{"backed":0,"unbacked":0,"not_measured":0}},"state":"uncovered","verification":"not-tracked","linked":[],"dangling":[],"kind_mismatch":[]},{"relation":"concern-is-addressed","check":"unaddressed-concern","direction":"incoming","document":{"id":"CON-001","path":"spec/CON-001.md","archetype":"Concern","repo":"o/b","verification":"not-tracked","criteria":{"backed":0,"unbacked":0,"not_measured":0}},"state":"covered","verification":"verified","linked":[{"id":"RULE-001","path":"spec/RULE-001.md","archetype":"Rule","repo":"o/a","verification":"verified","criteria":{"backed":1,"unbacked":0,"not_measured":0},"verb":"addresses","resolution":"resolved"},{"id":"RULE-005","path":"spec/RULE-005.md","archetype":"Rule","repo":"o/b","verification":"not-tracked","criteria":{"backed":0,"unbacked":0,"not_measured":0},"verb":"addresses","resolution":"resolved"}],"dangling":[],"kind_mismatch":[]}]},{"name":"gold-rule-addresses","check":"gold-rule-idle","direction":"outgoing","summary":{"rows":1,"state":{"uncovered":0,"covered":1},"verification":{"verified":1,"partial":0,"unverified":0,"not-tracked":0}},"rows":[{"relation":"gold-rule-addresses","check":"gold-rule-idle","direction":"outgoing","document":{"id":"RULE-001","path":"spec/RULE-001.md","archetype":"Rule","repo":"o/a","verification":"verified","criteria":{"backed":1,"unbacked":0,"not_measured":0}},"selector":{"field":"tier","value":"Gold"},"state":"covered","verification":"verified","linked":[{"id":"CON-001","path":"spec/CON-001.md","archetype":"Concern","repo":"o/b","verification":"not-tracked","criteria":{"backed":0,"unbacked":0,"not_measured":0},"verb":"addresses","resolution":"resolved"}],"dangling":[{"reference":"ghost","verb":"addresses"}],"kind_mismatch":[{"id":"RULE-002","repo":"o/a","archetype":"Rule","verb":"addresses","accepted":["Concern"]}]}]},{"name":"memo-cites","check":"memo-uncited","direction":"outgoing","summary":{"rows":1,"state":{"uncovered":0,"covered":1},"verification":{"verified":0,"partial":0,"unverified":0,"not-tracked":1}},"rows":[{"relation":"memo-cites","check":"memo-uncited","direction":"outgoing","document":{"id":"MEMO-001","path":"spec/MEMO-001.md","archetype":"Memo","repo":"o/a","verification":"not-tracked","criteria":{"backed":0,"unbacked":0,"not_measured":0}},"state":"covered","verification":"not-tracked","linked":[{"id":"CON-001","path":"spec/CON-001.md","archetype":"Concern","repo":"o/b","verification":"not-tracked","criteria":{"backed":0,"unbacked":0,"not_measured":0},"verb":"cites","resolution":"resolved"},{"id":"UNT-001","path":"spec/UNT-001.md","repo":"o/b","verification":"not-tracked","criteria":{"backed":0,"unbacked":0,"not_measured":0},"verb":"cites","resolution":"resolved"},{"reference":"ghost-a","verb":"cites","resolution":"dangling","verification":"not-tracked","criteria":{"backed":0,"unbacked":0,"not_measured":0}},{"reference":"ghost-b","verb":"cites","resolution":"dangling","verification":"not-tracked","criteria":{"backed":0,"unbacked":0,"not_measured":0}}],"dangling":[{"reference":"ghost-a","verb":"cites"},{"reference":"ghost-b","verb":"cites"}],"kind_mismatch":[]}]},{"name":"note-cites-rule","check":"note-uncited","direction":"outgoing","summary":{"rows":0,"state":{"uncovered":0,"covered":0},"verification":{"verified":0,"partial":0,"unverified":0,"not-tracked":0}},"rows":[]}]}"#;

#[trace("FR-084-AC-18")]
#[test]
fn a_module_declaring_no_relation_yields_an_empty_report() {
    let dir = repo(&[("CON-001", concern("CON-001", &[]))], &[]);
    let spec = Spec::from_path(&dir.path().join("spec"));
    let report = relation_report(
        &spec,
        &module("iso-obligations"),
        &[RelationReportMember {
            root: dir.path(),
            matrix: &[],
        }],
        &[],
    )
    .expect("an empty set of relations is not an error");
    assert_eq!(
        serde_json::to_string(&report).expect("json"),
        "{\"relations\":[]}"
    );
}

#[trace("FR-084-AC-19")]
#[test]
fn a_member_input_count_that_differs_from_the_corpus_is_an_error() {
    let (a, _b, corpus) = two_member_corpus();
    let err = relation_report(
        &corpus,
        &module("relation-report"),
        &[RelationReportMember {
            root: a.path(),
            matrix: &[],
        }],
        &[],
    )
    .expect_err("one input for two members");
    assert_eq!(
        err,
        RelationReportError::MemberCount {
            members: 2,
            supplied: 1
        }
    );
    assert!(err.to_string().contains('2') && err.to_string().contains('1'));
}

#[trace("FR-084-AC-13")]
#[test]
fn reporting_changes_no_document_and_raises_no_finding() {
    let dir = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            ("RULE-001", rule("RULE-001", Some("gold"), &[], &[])),
            (
                "MEMO-001",
                simple("MEMO-001", "Memo", &[("cites", "ghost")]),
            ),
        ],
        &[],
    );
    let registry = module("relation-report");
    let snapshot = |dir: &TempDir| {
        let mut out = Vec::new();
        for entry in fs::read_dir(dir.path().join("spec")).expect("dir") {
            let path = entry.expect("entry").path();
            out.push((path.clone(), fs::read(path).expect("read")));
        }
        out.sort();
        out
    };
    let findings = |dir: &TempDir| {
        let spec = Spec::from_path(&dir.path().join("spec"));
        let report = validate_bundle(
            &spec,
            &registry,
            BundlePosture::Okf,
            &dir.path().join("spec"),
            dir.path(),
        );
        let mut all: Vec<String> = report
            .errors
            .iter()
            .chain(&report.warnings)
            .map(|f| format!("{}:{}", f.reason, f.message))
            .collect();
        all.sort();
        all
    };
    let (before_docs, before) = (snapshot(&dir), findings(&dir));
    let _ = report_over(&dir, &registry, &[]);
    assert_eq!(snapshot(&dir), before_docs);
    assert_eq!(findings(&dir), before);
    assert!(!before.is_empty(), "the fixture raises findings to compare");
}

#[trace("FR-084-AC-15")]
#[test]
fn method_without_symbol_criteria_are_not_measured() {
    let dir = repo(
        &[
            ("RULE-001", rule("RULE-001", Some("gold"), &[], &[])),
            ("RULE-002", rule("RULE-002", Some("gold"), &[], &[])),
            ("RULE-003", rule("RULE-003", Some("gold"), &[], &[])),
        ],
        &[],
    );
    let matrix = [
        entry("RULE-001", &[S::Tagged, S::MethodWithoutSymbol]),
        entry("RULE-002", &[S::MethodWithoutSymbol]),
        entry("RULE-003", &[S::Untagged, S::MethodWithoutSymbol]),
    ];
    let report = report_over(&dir, &module("relation-report"), &matrix);
    let doc = |id: &str| row(&report, "gold-rule-addresses", id).document.clone();
    assert_eq!(doc("RULE-001").verification, Verification::Verified);
    assert_eq!(doc("RULE-001").criteria.not_measured, 1);
    assert_eq!(doc("RULE-002").verification, Verification::NotTracked);
    assert_eq!(doc("RULE-003").verification, Verification::Unverified);
}

#[trace("FR-084-AC-15")]
#[test]
fn a_real_inspection_criterion_is_not_measured_by_the_matrix() {
    let dir = repo(
        &[
            ("CON-001", concern("CON-001", &[])),
            (
                "RULE-001",
                rule(
                    "RULE-001",
                    Some("gold"),
                    &[("addresses", "CON-001")],
                    &[("RULE-001-AC-1", "Test"), ("RULE-001-AC-2", "Inspection")],
                ),
            ),
        ],
        &["RULE-001-AC-1"],
    );
    let report = report_of(&dir, &module("relation-report"), &[]);
    let r = row(&report, "gold-rule-addresses", "RULE-001");
    assert_eq!(r.verification, Verification::Verified);
    assert_eq!(r.document.criteria.not_measured, 1);
}

#[trace("FR-084-AC-16")]
#[test]
fn dangling_edges_satisfy_an_open_relation_and_never_a_constrained_one() {
    let dir = repo(
        &[
            (
                "MEMO-001",
                simple("MEMO-001", "Memo", &[("cites", "ghost")]),
            ),
            (
                "NOTE-001",
                simple("NOTE-001", "Note", &[("cites", "ghost")]),
            ),
            ("CON-001", concern("CON-001", &[])),
            (
                "NOTE-002",
                simple("NOTE-002", "Note", &[("cites", "CON-001")]),
            ),
        ],
        &[],
    );
    let registry = module("relation-report");
    let report = report_over(&dir, &registry, &[]);

    let memo = row(&report, "memo-cites", "MEMO-001");
    assert_eq!(memo.state, RowState::Covered);
    assert_eq!(memo.verification, Verification::NotTracked);
    assert_eq!(linked_ids(memo), vec!["dangling:ghost"]);
    assert_eq!(memo.dangling.len(), 1);
    assert_eq!(memo.dangling[0].reference, "ghost");
    let json = serde_json::to_value(memo).expect("json");
    assert_eq!(json["linked"][0]["resolution"], "dangling");
    assert!(json["linked"][0].get("id").is_none());

    let note = row(&report, "note-cites-rule", "NOTE-001");
    assert_eq!(note.state, RowState::Uncovered);
    assert!(note.linked.is_empty());
    assert_eq!(note.dangling.len(), 1);

    // A resolved target of the wrong archetype is a kind mismatch.
    let wrong = row(&report, "note-cites-rule", "NOTE-002");
    assert_eq!(wrong.state, RowState::Uncovered);
    assert!(wrong.linked.is_empty());
    assert_eq!(wrong.kind_mismatch.len(), 1);
    assert_eq!(wrong.kind_mismatch[0].id, "CON-001");
    assert_eq!(wrong.kind_mismatch[0].archetype.as_deref(), Some("Concern"));
    assert_eq!(wrong.kind_mismatch[0].accepted, vec!["Rule".to_string()]);

    // FR-058 agrees with the states.
    let spec = Spec::from_path(&dir.path().join("spec"));
    let validated = validate_bundle(
        &spec,
        &registry,
        BundlePosture::Okf,
        &dir.path().join("spec"),
        dir.path(),
    );
    let reasons: Vec<String> = validated
        .warnings
        .iter()
        .filter(|f| f.reason == "memo-uncited" || f.reason == "note-uncited")
        .map(|f| format!("{}", f.reason))
        .collect();
    assert_eq!(reasons, vec!["note-uncited", "note-uncited"]);
}

#[trace("FR-084-AC-2")]
#[test]
fn rows_carry_the_member_identity_and_link_across_members() {
    let a = repo(
        &[(
            "RULE-001",
            rule(
                "RULE-001",
                Some("gold"),
                &[("addresses", "ix://o/b/spec/CON-001")],
                &[],
            ),
        )],
        &[],
    );
    let b = repo(&[("CON-001", concern("CON-001", &[]))], &[]);
    let corpus = Spec::from_members(
        [(&a, "o/a"), (&b, "o/b")]
            .iter()
            .map(|(d, id)| CorpusMember {
                identity: Some((*id).to_string()),
                load: quire_rs::corpus::walk::load_repo(&d.path().join("spec")),
            })
            .collect(),
    )
    .expect("corpus");
    let matrix = [entry("RULE-001", &[S::Tagged])];
    let report = relation_report(
        &corpus,
        &module("relation-report"),
        &[
            RelationReportMember {
                root: a.path(),
                matrix: &matrix,
            },
            RelationReportMember {
                root: b.path(),
                matrix: &[],
            },
        ],
        &[],
    )
    .expect("report");
    let hazard_like = row(&report, "concern-is-addressed", "CON-001");
    assert_eq!(hazard_like.document.repo.as_deref(), Some("o/b"));
    assert_eq!(hazard_like.state, RowState::Covered);
    assert_eq!(hazard_like.verification, Verification::Verified);
    let LinkedEntry::Resolved { document, .. } = &hazard_like.linked[0] else {
        panic!("resolved")
    };
    assert_eq!(document.repo.as_deref(), Some("o/a"));
    assert_eq!(document.path, "spec/RULE-001.md");
}
