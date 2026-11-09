//! FR-051-AC-49 / AC-50 / AC-22 — the one advisory over non-marker trace text:
//! an id written in a comment or docstring never binds, and the coverage report
//! says what to write instead (TC-2161, TC-2162, TC-2163, TC-2164, TC-2165;
//! CR-215, PLAT-847).

use ix_trace_rs::trace;
use quire_rs::coverage::{CoverageDiagnostic, CoverageReport};
use quire_rs::symbols::trace;
use serde_json::{json, Value};

const REASON: &str = "trace-id-outside-marker";

fn rust_marker(template: Option<&str>) -> Value {
    let mut form = json!({
        "name": "rust-trace-attribute", "language": "rust",
        "pattern": r#"#\[trace\(((?:\s*"[^"]*"\s*,?)+)\)\]"#,
    });
    if let Some(template) = template {
        form["template"] = template.into();
    }
    form
}

fn python_marker() -> Value {
    json!({
        "name": "pytest-trace-marker", "language": "python",
        "pattern": r#"@pytest\.mark\.trace\(((?:\s*"[^"]*"\s*,?)+)\)"#,
        "template": "@pytest.mark.trace({ids})",
    })
}

fn typescript_marker() -> Value {
    json!({
        "name": "ts-trace-helper", "language": "typescript",
        "pattern": r#"\btrace\(((?:\s*"[^"]*"\s*,?)+)\)"#,
    })
}

fn attribute_form(template: Option<&str>) -> Value {
    let mut form = json!({
        "name": "rust-implements-attribute", "language": "rust",
        "pattern": r#"(?s)#\[\s*implements\s*\((.*?)\)\s*\]"#,
        "syntax": "rust_attribute",
    });
    if let Some(template) = template {
        form["template"] = template.into();
    }
    form
}

fn textual_implements() -> Value {
    json!({
        "name": "rust-implements-line", "language": "rust",
        "pattern": r"(?m)^\s*///\s*Implements:\s*([^\n]+)",
    })
}

struct Run {
    report: CoverageReport,
    verifies: usize,
    unmatched: usize,
}

/// Load a module declaring exactly the given marker and implements forms, run
/// coverage over `files` and return the report.
fn run(markers: Vec<Value>, implements: Vec<Value>, files: &[(&str, &str)]) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let module = dir.path().join("module");
    std::fs::create_dir_all(&module).unwrap();
    let manifest = json!({
        "name": "outside-marker-fixture", "version": "0.1.0", "artifact_types": [],
        "traceability": { "trace_tags": { "markers": markers, "implements": implements } }
    });
    std::fs::write(
        module.join("manifest.yaml"),
        serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();
    let registry = quire_rs::Registry::load_module(&module).unwrap();
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let src = dir.path().join("src");
    std::fs::create_dir(&src).unwrap();
    for (name, text) in files {
        std::fs::write(src.join(name), text).unwrap();
    }
    let graph = trace::bind(
        &quire_rs::symbols::extract_tree(&src),
        registry.traceability().unwrap(),
    );
    let spec = quire_rs::Spec::from_path(&src);
    let report = quire_rs::coverage::compute(&spec, &registry, &graph, &src).unwrap();
    Run {
        verifies: graph.verifies.len(),
        unmatched: graph.unmatched_tags.len(),
        report,
    }
}

fn findings(report: &CoverageReport) -> Vec<&CoverageDiagnostic> {
    report
        .diagnostics
        .iter()
        .filter(|d| d.reason == REASON)
        .collect()
}

const RUST_SOURCE: &str = "// FR-007-AC-1 first\n\
// FR-007-AC-2 second, FR-007-AC-2 again\n\
// FR-007-AC-3 third\n\
#[test]\n\
fn parses() {}\n";

const PYTHON_SOURCE: &str = concat!(
    "def test_parses():\n",
    "    \"\"\"FR-007-AC-1: the docstring.\"\"\"\n",
    "    pass\n",
);

const TYPESCRIPT_SOURCE: &str = "// FR-007-AC-1: on the registration\n\
it(\"parses\", () => {});\n";

#[trace("TC-2161", "FR-051-AC-49")]
#[test]
fn a_comment_or_docstring_id_warns_with_the_text_rendered_from_the_template() {
    let forms = vec![rust_marker(Some("#[trace({ids})]")), python_marker()];
    let run = run(
        forms,
        vec![],
        &[("lib.rs", RUST_SOURCE), ("test_m.py", PYTHON_SOURCE)],
    );

    let found = findings(&run.report);
    let at: Vec<(&str, Option<usize>, Option<&str>)> = found
        .iter()
        .map(|d| (d.path.as_deref().unwrap(), d.line, d.value.as_deref()))
        .collect();
    assert_eq!(
        at,
        vec![
            ("lib.rs", Some(1), Some(r#"#[trace("FR-007-AC-1")]"#)),
            ("lib.rs", Some(2), Some(r#"#[trace("FR-007-AC-2")]"#)),
            ("lib.rs", Some(3), Some(r#"#[trace("FR-007-AC-3")]"#)),
            (
                "test_m.py",
                Some(2),
                Some(r#"@pytest.mark.trace("FR-007-AC-1")"#)
            ),
        ],
        "three ids on three lines are three findings; one id twice on a line is one: {found:?}"
    );
    assert!(found[0].message.contains("parses"), "{}", found[0].message);
    assert!(found[0].message.contains("FR-007-AC-1"));
    assert!(found[0].message.contains("lib.rs:1"));

    // Additive: the same tokens still reach `unmatched_tags`, nothing binds and
    // no total moves.
    assert_eq!(run.verifies, 0);
    // One record per token on the Rust block (the id written twice is two
    // records, as before); the census block never read a docstring.
    assert_eq!(run.unmatched, 4);
    assert_eq!(run.report.totals.backed, 0);
}

#[trace("TC-2162", "FR-051-AC-49")]
#[test]
fn the_advisory_is_silent_or_names_the_form_when_nothing_canonical_can_be_written() {
    // No marker form for Python: nothing to write, so no finding for it.
    let rust_only = run(
        vec![rust_marker(Some("#[trace({ids})]"))],
        vec![],
        &[("lib.rs", RUST_SOURCE), ("test_m.py", PYTHON_SOURCE)],
    );
    let paths: Vec<_> = findings(&rust_only.report)
        .iter()
        .map(|d| d.path.clone().unwrap())
        .collect();
    assert!(paths.iter().all(|p| p == "lib.rs"), "{paths:?}");

    // A form with no template: the finding names the form instead.
    let untemplated = run(
        vec![typescript_marker()],
        vec![],
        &[("m.test.ts", TYPESCRIPT_SOURCE)],
    );
    let found = findings(&untemplated.report);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].value.as_deref(), Some("ts-trace-helper"));
    assert!(
        found[0].message.contains("ts-trace-helper"),
        "{}",
        found[0].message
    );

    // No marker form declared at all (only an implements form): no finding.
    let none = run(
        vec![],
        vec![textual_implements()],
        &[("lib.rs", RUST_SOURCE)],
    );
    assert!(findings(&none.report).is_empty());
}

#[trace("TC-2162", "FR-051-AC-49")]
#[test]
fn an_id_an_implements_form_read_is_left_to_its_own_warning() {
    let source = "/// Implements: FR-007-AC-2\n\
                  /// Also mentions FR-007-AC-9.\n\
                  pub fn parse() {}\n";
    let run = run(
        vec![rust_marker(Some("#[trace({ids})]"))],
        vec![
            textual_implements(),
            attribute_form(Some("#[implements({ids})]")),
        ],
        &[("lib.rs", source)],
    );
    let ids: Vec<_> = findings(&run.report).iter().map(|d| d.line).collect();
    assert_eq!(
        ids,
        vec![Some(2)],
        "the id the implements form read is FR-050-AC-62's; the other is advised"
    );
    assert_eq!(
        run.report
            .diagnostics
            .iter()
            .filter(|d| d.reason == "legacy-implements-form")
            .count(),
        1
    );
}

#[trace("TC-2163", "FR-051-AC-50")]
#[test]
fn the_advisory_fires_on_a_backed_id_and_is_silent_for_an_all_marker_tree() {
    let backed = "// FR-007-AC-1: still a comment.\n\
                  #[trace(\"FR-007-AC-1\")]\n\
                  #[test]\n\
                  fn parses() {\n    let s = \"FR-007-AC-9\";\n    // FR-007-AC-8\n    let _ = s;\n}\n";
    let run_backed = run(
        vec![rust_marker(Some("#[trace({ids})]"))],
        vec![],
        &[("lib.rs", backed)],
    );
    assert_eq!(run_backed.verifies, 1, "the marker binds");
    let found = findings(&run_backed.report);
    assert_eq!(
        found.len(),
        1,
        "a body comment and a string are not advised: {found:?}"
    );
    assert_eq!(found[0].line, Some(1));

    let tidy = "#[trace(\"FR-007-AC-1\")]\n#[test]\nfn parses() {}\n";
    let run_tidy = run(
        vec![rust_marker(Some("#[trace({ids})]"))],
        vec![],
        &[("lib.rs", tidy)],
    );
    assert!(findings(&run_tidy.report).is_empty());
}

#[trace("TC-2165", "FR-051-AC-22", "FR-051-AC-49")]
#[test]
fn a_production_symbol_is_told_the_implements_attribute_and_a_module_its_kind() {
    let source = "// FR-007-AC-1 header prose.\n\
                  \n\
                  /// FR-007-AC-2 on production.\n\
                  pub fn parse() {}\n";
    let with_template = run(
        vec![rust_marker(Some("#[trace({ids})]"))],
        vec![
            textual_implements(),
            attribute_form(Some("#[implements({ids})]")),
        ],
        &[("lib.rs", source)],
    );
    let found = findings(&with_template.report);
    assert_eq!(found.len(), 2, "{found:?}");
    // The module header: kind alone.
    assert_eq!(found[0].line, Some(1));
    assert_eq!(found[0].value.as_deref(), Some("module"));
    assert!(!found[0].message.contains("#["), "{}", found[0].message);
    // The production function: the first rust_attribute implements template,
    // never the textual form, and no `#[trace(...)]`.
    assert_eq!(found[1].line, Some(3));
    assert_eq!(
        found[1].value.as_deref(),
        Some(r#"#[implements("FR-007-AC-2")]"#)
    );
    assert!(
        !found[1].message.contains("#[trace"),
        "{}",
        found[1].message
    );

    // No attribute form declaring a template: the kind alone.
    let without = run(
        vec![rust_marker(Some("#[trace({ids})]"))],
        vec![textual_implements(), attribute_form(None)],
        &[("lib.rs", source)],
    );
    let found = findings(&without.report);
    assert_eq!(found[1].value.as_deref(), Some("function"));
    assert!(!found[1].message.contains("#["), "{}", found[1].message);
}

#[trace("TC-2164", "FR-050-AC-44")]
#[test]
fn an_id_in_a_test_name_adds_nothing_and_raises_no_name_diagnostic() {
    let source = "#[test]\nfn tc_901_named() {}\n";
    let run = run(
        vec![rust_marker(Some("#[trace({ids})]"))],
        vec![],
        &[("lib.rs", source)],
    );
    let census = &run.report.binding_census[0];
    assert_eq!(
        (census.candidates, census.tagged, census.bound),
        (1, 0, 0),
        "the name adds nothing to tagged or bound"
    );
    assert!(
        !run.report
            .diagnostics
            .iter()
            .any(|d| d.reason == "marker-form-mismatch"
                || (d.reason == "hollow-denominator"
                    && d.value.as_deref().is_some_and(|v| v.contains("self_named")))),
        "{:?}",
        run.report.diagnostics
    );
}
