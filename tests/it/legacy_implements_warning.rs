//! FR-050-AC-62 — a textual Rust `implements` use warns when the loaded module
//! declares the attribute form (TC-2090, CR-209).

use ix_trace_rs::trace;
use quire_rs::coverage::{CoverageDiagnostic, CoverageReport};
use quire_rs::symbols::trace;
use serde_json::{json, Value};

const REASON: &str = "legacy-implements-form";
const LEGACY_PATTERN: &str = r"(?m)^\s*///\s*Implements:\s*([^\n]+)";
const ATTRIBUTE_PATTERN: &str = r#"(?s)#\[\s*implements\s*\((.*?)\)\s*\]"#;

fn legacy_form(name: &str, language: &str, pattern: &str) -> Value {
    json!({ "name": name, "language": language, "pattern": pattern })
}

fn attribute_form(name: &str, template: Option<&str>) -> Value {
    let mut form = json!({
        "name": name, "language": "rust", "pattern": ATTRIBUTE_PATTERN,
        "syntax": "rust_attribute"
    });
    if let Some(template) = template {
        form["template"] = template.into();
    }
    form
}

/// Load a module declaring exactly `forms` as its implements forms, run
/// coverage over `files` and return the graph's implements count and report.
fn run(forms: Vec<Value>, files: &[(&str, &str)]) -> (usize, CoverageReport) {
    let dir = tempfile::tempdir().unwrap();
    let module = dir.path().join("module");
    std::fs::create_dir_all(&module).unwrap();
    let manifest = json!({
        "name": "legacy-implements-fixture", "version": "0.1.0", "artifact_types": [],
        "traceability": { "trace_tags": { "implements": forms } }
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
    (graph.implements.len(), report)
}

fn findings(report: &CoverageReport) -> Vec<&CoverageDiagnostic> {
    report
        .diagnostics
        .iter()
        .filter(|d| d.reason == REASON)
        .collect()
}

const LEGACY_SOURCE: &str = "// header\n\
/// Parses the manifest.\n\
/// Implements: FR-007-AC-2\n\
pub fn parse() {}\n\
\n\
/// Implements: FR-007-AC-1, FR-007-AC-2\n\
pub fn parse_both() {}\n";

#[trace("TC-2090", "FR-050-AC-62")]
#[test]
fn a_legacy_use_warns_with_the_canonical_text_and_changes_nothing_else() {
    let forms = vec![
        legacy_form("textual", "rust", LEGACY_PATTERN),
        attribute_form("attr", Some("#[implements({ids})]")),
    ];
    let (bound, report) = run(forms, &[("lib.rs", LEGACY_SOURCE)]);

    let found = findings(&report);
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(found[0].path.as_deref(), Some("lib.rs"));
    assert_eq!(found[0].line, Some(3));
    assert_eq!(
        found[0].value.as_deref(),
        Some(r#"#[implements("FR-007-AC-2")]"#)
    );
    assert!(found[0].message.contains("parse"), "{}", found[0].message);
    assert_eq!(found[1].line, Some(6));
    assert_eq!(
        found[1].value.as_deref(),
        Some(r#"#[implements("FR-007-AC-1", "FR-007-AC-2")]"#)
    );
    assert!(found[1].message.contains("parse_both"));

    // The same source under a module without the attribute form: identical
    // binding and counts, no finding.
    let (control_bound, control) = run(
        vec![legacy_form("textual", "rust", LEGACY_PATTERN)],
        &[("lib.rs", LEGACY_SOURCE)],
    );
    assert_eq!(bound, control_bound);
    assert_eq!(bound, 3);
    assert_eq!(report.implements, control.implements);
    assert_eq!(report.totals, control.totals);
    assert!(findings(&control).is_empty());
}

#[trace("TC-2090", "FR-050-AC-62")]
#[test]
fn renaming_every_form_changes_no_finding() {
    let run_named = |legacy: &str, attr: &str| {
        let forms = vec![
            legacy_form(legacy, "rust", LEGACY_PATTERN),
            attribute_form(attr, Some("#[implements({ids})]")),
        ];
        run(forms, &[("lib.rs", LEGACY_SOURCE)]).1
    };
    let original = run_named("textual", "attr");
    let renamed = run_named("zzz-one", "zzz-two");
    let strip = |report: &CoverageReport| -> Vec<(String, Option<usize>, Option<String>, String)> {
        findings(report)
            .iter()
            .map(|d| (d.reason.clone(), d.line, d.value.clone(), d.message.clone()))
            .collect()
    };
    assert_eq!(strip(&original).len(), 2);
    assert_eq!(strip(&original), strip(&renamed));
}

#[trace("TC-2090", "FR-050-AC-62")]
#[test]
fn an_attribute_form_without_a_template_yields_no_finding() {
    let forms = vec![
        legacy_form("textual", "rust", LEGACY_PATTERN),
        attribute_form("attr", None),
    ];
    let (bound, report) = run(forms, &[("lib.rs", LEGACY_SOURCE)]);
    assert_eq!(bound, 3, "the legacy use still binds");
    assert!(findings(&report).is_empty());
}

#[trace("TC-2090", "FR-050-AC-62")]
#[test]
fn the_first_declared_template_renders_the_replacement() {
    let forms = vec![
        legacy_form("textual", "rust", LEGACY_PATTERN),
        attribute_form("no-template", None),
        attribute_form("first", Some("#[implements({ids})]")),
        attribute_form("second", Some("#[other({ids})]")),
    ];
    let (_, report) = run(forms, &[("lib.rs", LEGACY_SOURCE)]);
    assert_eq!(
        findings(&report)[0].value.as_deref(),
        Some(r#"#[implements("FR-007-AC-2")]"#)
    );
}

#[trace("TC-2090", "FR-050-AC-62")]
#[test]
fn an_attribute_binding_yields_no_finding() {
    let forms = vec![
        legacy_form("textual", "rust", LEGACY_PATTERN),
        attribute_form("attr", Some("#[implements({ids})]")),
    ];
    let source = "#[implements(\"FR-007-AC-2\")]\npub fn parse() {}\n";
    let (bound, report) = run(forms, &[("lib.rs", source)]);
    assert_eq!(bound, 1);
    assert!(findings(&report).is_empty());
}

#[trace("TC-2090", "FR-050-AC-62")]
#[test]
fn non_rust_source_yields_no_finding() {
    let forms = vec![
        legacy_form(
            "python-textual",
            "python",
            r"(?m)^\s*#\s*Implements:\s*([^\n]+)",
        ),
        attribute_form("attr", Some("#[implements({ids})]")),
    ];
    let source = "# Implements: FR-007-AC-2\ndef parse():\n    pass\n";
    let (bound, report) = run(forms, &[("lib.py", source)]);
    assert!(bound > 0, "the Python use still binds");
    assert!(findings(&report).is_empty());
}

#[trace("TC-2090", "FR-050-AC-62")]
#[test]
fn a_nested_use_yields_one_finding_naming_the_innermost_member() {
    let source = "pub mod inner {\n\
    /// Implements: FR-007-AC-1\n\
    pub fn in_mod() {}\n\
}\n\
\n\
pub trait Shape {\n\
    /// Implements: FR-007-AC-2\n\
    fn in_trait(&self);\n\
}\n\
\n\
pub fn outer() {\n\
    /// Implements: FR-007-AC-3\n\
    fn in_fn() {}\n\
    in_fn();\n\
}\n\
\n\
pub mod spaced {\n\
\n\
    /// Implements: FR-007-AC-4\n\
    pub fn after_blank() {}\n\
\n\
    /// Implements: FR-007-AC-5\n\
    #[inline]\n\
    #[must_use]\n\
    pub fn with_attributes() {}\n\
}\n";
    let forms = vec![
        legacy_form("textual", "rust", LEGACY_PATTERN),
        attribute_form("attr", Some("#[implements({ids})]")),
    ];
    let (_, report) = run(forms, &[("lib.rs", source)]);
    let found = findings(&report);
    let at: Vec<(Option<usize>, bool)> = found
        .iter()
        .map(|d| {
            let member = [
                "in_mod",
                "in_trait",
                "in_fn",
                "after_blank",
                "with_attributes",
            ]
            .iter()
            .any(|name| d.message.contains(&format!("{name}` at")));
            (d.line, member)
        })
        .collect();
    assert_eq!(
        at,
        vec![
            (Some(2), true),
            (Some(7), true),
            (Some(12), true),
            (Some(19), true),
            (Some(22), true)
        ],
        "{found:?}"
    );
}
