//! NFR-140 â qualified trace ids in `verifies` markers bind the artifact of
//! the corpus member they name, never the same bare id of another member.
//! The acceptance criteria live in `agent-ix/filament-spec-language`; the
//! tests are traced to them by id.

use std::fs;
use std::path::{Path, PathBuf};

use quire_rs::corpus::{CorpusMember, Spec};
use quire_rs::coverage::{compute, CoverageReport};
use quire_rs::symbols::{extract_tree, trace as sym_trace};
use quire_rs::Registry;
use tempfile::TempDir;

const FSL: &str = "agent-ix/filament-spec-language";
const FSPEC: &str = "agent-ix/filament-specification";

fn fixture_module(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability")
        .join(name)
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

fn requirement(criteria: &[&str]) -> String {
    let mut rows = String::new();
    for id in criteria {
        rows.push_str(&format!("| {id} | The system shall do it. | Test |\n"));
    }
    format!(
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n{rows}"
    )
}

/// Two repositories holding the same bare id `FR-001-AC-1`; FSL tests tag it
/// in the forms under test. `fspec` also mints `FR-001-AC-3`, FSL does not.
fn world() -> TempDir {
    world_with(
        &[
            ("short_form", "FSpec-FR-001-AC-1"),
            (
                "long_form",
                "ix://agent-ix/filament-specification/FR-001-AC-2",
            ),
            ("unminted", "FSpec-FR-001-AC-99"),
            ("short_form_as_ignored", "FSpec-FR-001-AC-3"),
        ],
        false,
    )
}

/// `world` with chosen FSL `(test name, tag)` pairs (a name ending `ignored` is
/// `#[ignore]`d) and, optionally, a filament-specification test matrix whose row
/// `TC-001` traces to both of its criteria.
fn world_with(tests: &[(&str, &str)], matrix: bool) -> TempDir {
    let root = TempDir::new().expect("tempdir");
    if matrix {
        write(
            root.path(),
            "fspec/spec/tests.md",
            "---\nid: TM-001\ntype: TestMatrix\ntitle: Matrix\n---\n\n## Test Cases\n\n\
             | ID | Traces To | Status |\n|----|-----------|--------|\n\
             | TC-001 | FR-001-AC-1, FR-001-AC-2 | 🚧 |\n",
        );
    }
    write(
        root.path(),
        "fsl/spec/FR-001.md",
        &requirement(&["FR-001-AC-1", "FR-001-AC-2"]),
    );
    write(
        root.path(),
        "fspec/spec/FR-001.md",
        &requirement(&["FR-001-AC-1", "FR-001-AC-2", "FR-001-AC-3"]),
    );
    let mut src = String::from("//! fixture\n\n#[cfg(test)]\nmod tests {\n");
    for &(name, tag) in tests {
        let ignore = if name.ends_with("ignored") {
            "    #[ignore]\n"
        } else {
            ""
        };
        src.push_str(&format!(
            "    #[trace(\"{tag}\")]\n    #[test]\n{ignore}    fn {name}() {{}}\n"
        ));
    }
    src.push_str("}\n");
    write(root.path(), "fsl/src/lib.rs", &src);
    root
}

fn member(root: &Path, dir: &str, identity: &str) -> CorpusMember {
    CorpusMember {
        identity: Some(identity.to_string()),
        load: quire_rs::corpus::walk::load_repo(&root.join(dir).join("spec")),
    }
}

fn report(root: &Path, module: &str, members: Vec<CorpusMember>) -> CoverageReport {
    let registry = Registry::load_module(&fixture_module(module)).expect("load module");
    let spec = Spec::from_members(members).expect("corpus");
    let extraction = extract_tree(&root.join("fsl/src"));
    let model = registry.traceability().cloned().expect("model");
    let graph = sym_trace::bind(&extraction, &model);
    compute(&spec, &registry, &graph, root).expect("model declared")
}

fn both(root: &Path, module: &str) -> CoverageReport {
    report(
        root,
        module,
        vec![member(root, "fsl", FSL), member(root, "fspec", FSPEC)],
    )
}

fn binders(report: &CoverageReport, document: &str, id: &str) -> Vec<String> {
    report
        .coverage_matrix
        .iter()
        .filter(|r| r.document.starts_with(document))
        .flat_map(|r| &r.criteria)
        .filter(|c| c.id == id)
        .flat_map(|c| c.binders.iter().map(|b| b.qualified_name.clone()))
        .collect()
}

fn unresolved(report: &CoverageReport) -> Vec<(&str, &str)> {
    report
        .diagnostics
        .iter()
        .filter(|d| d.reason == "unresolved-qualified-trace-id")
        .map(|d| (d.value.as_deref().unwrap_or(""), d.message.as_str()))
        .collect()
}

// a short-form tag binds the criterion of the member the qualifier names and
// no criterion of FSL, which mints the same bare id.
#[test]
fn an_fspec_tag_binds_the_standard_row_and_not_the_fsl_row() {
    let root = world();
    let report = both(root.path(), "iso-obligations-qualified");
    assert_eq!(
        binders(&report, "fspec", "FR-001-AC-1"),
        vec!["tests::short_form"]
    );
    assert!(
        binders(&report, "fsl", "FR-001-AC-1").is_empty(),
        "FSL's own row must list no FSpec- test: {:?}",
        binders(&report, "fsl", "FR-001-AC-1")
    );
    assert!(binders(&report, "fsl", "FR-001-AC-2").is_empty());
    assert!(
        report.untracked_symbols.is_empty(),
        "{:?}",
        report.untracked_symbols
    );
}

// the qualified long form binds the same row as the short form.
#[test]
fn the_long_form_binds_the_same_row_as_the_short_form() {
    let root = world();
    let report = both(root.path(), "iso-obligations-qualified");
    assert_eq!(
        binders(&report, "fspec", "FR-001-AC-2"),
        vec!["tests::long_form"]
    );
}

// the binder's ignored state is carried like any other binder's.
#[test]
fn a_qualified_binder_keeps_its_ignored_state() {
    let root = world();
    let report = both(root.path(), "iso-obligations-qualified");
    let criterion = report
        .coverage_matrix
        .iter()
        .filter(|r| r.document.starts_with("fspec"))
        .flat_map(|r| &r.criteria)
        .find(|c| c.id == "FR-001-AC-3")
        .expect("criterion");
    assert!(criterion.binders.iter().all(|b| b.ignored));
    assert_eq!(
        criterion.status,
        quire_rs::coverage::CoverageMatrixStatus::TaggedByIgnoredTest
    );
}

// over FSL alone a qualified tag is a reference outside the corpus and binds
// no FSL row.
#[test]
fn over_fsl_alone_a_qualified_tag_is_outside_the_corpus() {
    let root = world();
    let report = report(
        root.path(),
        "iso-obligations-qualified",
        vec![member(root.path(), "fsl", FSL)],
    );
    assert!(binders(&report, "fsl", "FR-001-AC-1").is_empty());
    assert!(binders(&report, "fsl", "FR-001-AC-2").is_empty());
    let found = unresolved(&report);
    assert_eq!(found.len(), 4, "{found:?}");
    assert!(found
        .iter()
        .all(|(_, message)| message.contains("outside it")));
    assert!(report.untracked_symbols.is_empty());
}

// an id the named member does not mint is reported as unknown in that member.
#[test]
fn an_unminted_qualified_id_is_reported_as_unknown_in_the_member() {
    let root = world();
    let report = both(root.path(), "iso-obligations-qualified");
    let found = unresolved(&report);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "FSpec-FR-001-AC-99");
    assert!(
        found[0].1.contains("mints no id `FR-001-AC-99`"),
        "{}",
        found[0].1
    );
}

// the declaration is what makes `FSpec-` a qualifier. Without it the engine cannot
// know the prefix, so the tag is an ordinary unqualified symbol: it binds no row,
// bare ids included, and is reported in `untracked_symbols` (not as unresolved).
#[test]
fn without_the_declaration_an_fspec_tag_is_an_ordinary_untracked_symbol() {
    let root = world();
    let report = both(root.path(), "iso-obligations");
    assert!(binders(&report, "fspec", "FR-001-AC-1").is_empty());
    assert!(binders(&report, "fsl", "FR-001-AC-1").is_empty());
    assert!(report
        .untracked_symbols
        .iter()
        .any(|s| s.trace_id == "FSpec-FR-001-AC-1"));
}

// the declaration names a non-blank prefix and an `<org>/<repo>` identity.
#[test]
fn a_malformed_qualifier_declaration_is_refused_at_load() {
    use quire_rs::traceability::{TraceIdQualifier, TraceabilityModel};
    let model = |prefix: &str, repository: &str| TraceabilityModel {
        trace_id_qualifiers: vec![TraceIdQualifier {
            prefix: prefix.into(),
            repository: repository.into(),
        }],
        ..TraceabilityModel::default()
    };
    assert!(model("FSpec-", FSPEC).validate().is_empty());
    assert!(!model("", FSPEC).validate().is_empty());
    assert!(!model("FSpec-", "filament-specification")
        .validate()
        .is_empty());
    let overlapping = TraceabilityModel {
        trace_id_qualifiers: vec![
            TraceIdQualifier {
                prefix: "F-".into(),
                repository: "o/a".into(),
            },
            TraceIdQualifier {
                prefix: "F-X-".into(),
                repository: "o/b".into(),
            },
        ],
        ..TraceabilityModel::default()
    };
    assert_eq!(overlapping.validate().len(), 2);
}

// a qualified tag naming a whole document binds nothing a matrix reads, so it is
// reported as unresolved, as the bare tag is reported untracked.
#[test]
fn a_qualified_document_id_is_reported_not_silently_accepted() {
    let root = world_with(&[("doc_level", "FSpec-FR-001")], false);
    let report = both(root.path(), "iso-obligations-qualified");
    let found = unresolved(&report);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "FSpec-FR-001");
    // the bare control: the same id written bare is untracked
    let bare = world_with(&[("doc_level", "FR-001")], false);
    let bare_report = both(bare.path(), "iso-obligations-qualified");
    assert!(bare_report
        .untracked_symbols
        .iter()
        .any(|s| s.trace_id == "FR-001"));
}

fn unbacked_matrix_row(report: &CoverageReport) -> bool {
    report
        .unbacked_rows
        .iter()
        .any(|r| r.document.starts_with("fspec") && r.row_id.as_deref() == Some("TC-001"))
}

// a qualified tag backs a reference row by the bare rule: every criterion the
// row traces to must be backed, not any one of them.
#[test]
fn a_qualified_tag_backs_a_reference_row_only_when_every_criterion_is_backed() {
    let partial = world_with(&[("only_ac1", "FSpec-FR-001-AC-1")], true);
    let report = both(partial.path(), "iso-obligations-qualified");
    assert!(
        unbacked_matrix_row(&report),
        "one of two criteria must leave the row unbacked: {:?}",
        report.unbacked_rows
    );

    let full = world_with(
        &[("ac1", "FSpec-FR-001-AC-1"), ("ac2", "FSpec-FR-001-AC-2")],
        true,
    );
    let report = both(full.path(), "iso-obligations-qualified");
    assert!(!unbacked_matrix_row(&report), "{:?}", report.unbacked_rows);
}

// a qualified tag backs the minted target of the member it names, not FSL's.
#[test]
fn a_qualified_tag_backs_the_minted_target_of_its_own_member() {
    let root = world();
    let report = both(root.path(), "iso-obligations-qualified");
    let backed = |document: &str, id: &str| {
        report
            .minted_targets
            .iter()
            .find(|m| m.document.starts_with(document) && m.id == id)
            .map(|m| m.backed)
    };
    assert_eq!(backed("fspec", "FR-001-AC-1"), Some(true));
    assert_eq!(backed("fsl", "FR-001-AC-1"), Some(false));
    assert_eq!(backed("fsl", "FR-001-AC-2"), Some(false));
}

// over FSL alone the only report about a qualified tag is the unresolved one.
#[test]
fn over_fsl_alone_only_the_unresolved_diagnostic_names_the_tag() {
    let root = world();
    let report = report(
        root.path(),
        "iso-obligations-qualified",
        vec![member(root.path(), "fsl", FSL)],
    );
    let others: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.reason != "unresolved-qualified-trace-id")
        .filter(|d| {
            d.value
                .as_deref()
                .is_some_and(|v| v.contains("FSpec-") || v.contains("ix://"))
                || d.message.contains("FSpec-")
        })
        .collect();
    assert!(others.is_empty(), "{others:?}");
}
