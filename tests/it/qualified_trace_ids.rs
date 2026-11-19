//! NFR-140 — qualified trace ids in `verifies` markers bind the artifact of
//! the corpus member they name, never the same bare id of another member.
//! The acceptance criteria live in `agent-ix/filament-spec-language`; the
//! tests are traced to them by id.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
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
    let root = TempDir::new().expect("tempdir");
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
    let tests = [
        ("short_form", "FSpec-FR-001-AC-1"),
        (
            "long_form",
            "ix://agent-ix/filament-specification/FR-001-AC-2",
        ),
        ("unminted", "FSpec-FR-001-AC-99"),
        ("short_form_as_ignored", "FSpec-FR-001-AC-3"),
    ];
    let mut src = String::from("//! fixture\n\n#[cfg(test)]\nmod tests {\n");
    for (name, tag) in tests {
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

#[trace("NFR-140-AC-1")]
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

#[trace("NFR-140-AC-2")]
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

#[trace("NFR-140-AC-2")]
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

#[trace("NFR-140-AC-2")]
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

#[trace("NFR-140-AC-2")]
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

#[trace("NFR-140-AC-3")]
// without the declared qualifier no FSpec- tag binds any row, bare ids
// included; the tag is an untracked id.
#[test]
fn without_the_declaration_an_fspec_tag_binds_nothing() {
    let root = world();
    let report = both(root.path(), "iso-obligations");
    assert!(binders(&report, "fspec", "FR-001-AC-1").is_empty());
    assert!(binders(&report, "fsl", "FR-001-AC-1").is_empty());
    assert!(report
        .untracked_symbols
        .iter()
        .any(|s| s.trace_id == "FSpec-FR-001-AC-1"));
}

#[trace("NFR-140-AC-3")]
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
    assert!(model("FSpec-", FSPEC).validate().is_ok());
    assert!(model("", FSPEC).validate().is_err());
    assert!(model("FSpec-", "filament-specification")
        .validate()
        .is_err());
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
    assert!(overlapping.validate().is_err());
}
