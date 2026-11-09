//! FR-051 shell adapter (CR-213, PLAT-362): TC-2101..TC-2113.

use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::coverage::compute;
use quire_rs::symbols::trace::{bind, SymbolGraph};
use quire_rs::symbols::{extract_file, extract_tree, extract_tree_scoped, Symbol, SymbolKind};
use quire_rs::traceability::{SourceLanguage, TraceabilityModel};
use quire_rs::{Registry, Spec};
use serde_json::json;

fn sh(path: &str, source: &str) -> quire_rs::symbols::SymbolExtraction {
    extract_file(path, SourceLanguage::Shell, source)
}

fn names(symbols: &[Symbol], kind: SymbolKind) -> Vec<&str> {
    symbols
        .iter()
        .filter(|s| s.kind == kind)
        .map(|s| s.qualified_name.as_str())
        .collect()
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, body).expect("write");
}

/// A model with one shell `markers` form, `# trace: ID[, ID]`, and optional
/// `entry_paths`.
fn model(entry_paths: Option<Vec<&str>>) -> TraceabilityModel {
    let mut form = json!({
        "name": "shell-trace-comment",
        "language": "shell",
        "syntax": "comment",
        "pattern": r"#\s*trace:\s*(.+)$",
    });
    if let Some(globs) = entry_paths {
        form["entry_paths"] = json!(globs);
    }
    let model: TraceabilityModel =
        serde_json::from_value(json!({ "trace_tags": { "markers": [form] } })).expect("model");
    model.validate().expect("valid model");
    model
}

fn verified(graph: &SymbolGraph) -> Vec<(&str, &str)> {
    graph
        .verifies
        .iter()
        .map(|v| (v.symbol.as_str(), v.trace_id.as_str()))
        .collect()
}

#[trace("TC-2101", "FR-051-AC-34")]
// the `.sh` extension is read and every symbol carries language shell;
// `.bash` and an extension-less `#!/bin/sh` file are not read.
#[test]
fn tc2101_only_sh_files_are_read() {
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), "a.sh", "f() { :; }\n");
    write(dir.path(), "b.bash", "g() { :; }\n");
    write(dir.path(), "run", "#!/bin/sh\nh() { :; }\n");
    let extraction = extract_tree(dir.path());
    assert!(extraction.symbols.iter().all(|s| s.path == "a.sh"));
    assert!(extraction
        .symbols
        .iter()
        .all(|s| s.language == SourceLanguage::Shell));
    assert_eq!(names(&extraction.symbols, SymbolKind::Function), ["f"]);
    assert_eq!(
        serde_json::to_value(SourceLanguage::Shell).expect("serialize"),
        json!("shell")
    );
}

#[trace("TC-2102", "FR-051-AC-35")]
// every function spelling mints one function named by itself.
#[test]
fn tc2102_function_spellings() {
    let src = "\
a() { :; }
function b { :; }
function c() { :; }
d()
{
  :
}
e() ( :; )
my-fn.v2:x() { :; }
outer() {
  inner() { :; }
}
";
    let extraction = sh("x.sh", src);
    let mut got = names(&extraction.symbols, SymbolKind::Function);
    got.sort();
    assert_eq!(
        got,
        ["a", "b", "c", "d", "e", "inner", "my-fn.v2:x", "outer"]
    );
    assert!(
        extraction.diagnostics.is_empty(),
        "{:?}",
        extraction.diagnostics
    );
    let inner = extraction
        .symbols
        .iter()
        .find(|s| s.qualified_name == "inner")
        .expect("inner");
    assert_eq!(inner.container.as_deref(), Some("outer"));
    let d = extraction
        .symbols
        .iter()
        .find(|s| s.qualified_name == "d")
        .expect("d");
    assert_eq!((d.line, d.end_line), (4, 7), "the span covers the body");
}

#[trace("TC-2103", "FR-051-AC-36")]
// definition-shaped text that is data mints nothing and raises no diagnostic.
#[test]
fn tc2103_decoys_mint_no_function() {
    let src = "\
#!/bin/bash
# decoy() { :; }
echo 'sq() { :; }'
echo \"dq() { :; }\"
echo $'ansi() { :; }'
echo `bt() { :; }`
echo $(sub() { :; })
cat <<< 'here() { :; }'
y=$(g() { :; }; g)
cat <<EOF
heredoc() { :; }
EOF
cat <<'EOF'
quoted() { :; }
EOF
cat <<-EOF
\tdash() { :; }
EOF
case $x in
  pat) echo ok ;;
esac
{ echo group; }
y=${x:-}}
z=$(( 1 + 2 ))
real() { :; }
";
    for text in [src.to_string(), src.replace('\n', "\r\n")] {
        let extraction = sh("x.sh", &text);
        assert_eq!(
            names(&extraction.symbols, SymbolKind::Function),
            ["real"],
            "{:?}",
            extraction.diagnostics
        );
        assert!(
            extraction.diagnostics.is_empty(),
            "{:?}",
            extraction.diagnostics
        );
    }
}

#[trace("TC-2104", "FR-051-AC-37")]
// one entry_point per file, with or without a module; the label is part of
// the identity and of the symbol table's kind.
#[test]
fn tc2104_one_entry_point_per_file() {
    for src in ["", "f() { :; }\n", "echo hi\necho there\n"] {
        let extraction = sh("scripts/a.sh", src);
        let entries: Vec<&Symbol> = extraction
            .symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::EntryPoint)
            .collect();
        assert_eq!(entries.len(), 1, "{src:?}");
        let entry = entries[0];
        assert_eq!(entry.qualified_name, "entry_point");
        assert_eq!(entry.line, 1);
        assert_eq!(entry.end_line, src.lines().count().max(1));
        // The symbol set does not depend on a module: nothing here consults one.
        let with_module = bind(&extraction, &model(Some(vec!["scripts/**"])));
        assert!(with_module
            .defined_in
            .iter()
            .any(|(id, name, _)| id == &entry.id && name == "entry_point"));
    }
    // Identity is language, path, name and kind: stable per path, distinct across.
    let id_of = |path: &str| sh(path, "x\n").symbols[0].id.clone();
    assert_eq!(id_of("scripts/a.sh"), id_of("scripts/a.sh"));
    assert_ne!(id_of("scripts/a.sh"), id_of("scripts/b.sh"));
    // The symbol-table report lists it as a kind that binds and carries no
    // `implements`, and the FR-045 graph node carries the `entry_point` kind.
    let extraction = sh("scripts/a.sh", "f() { :; }\n");
    let report = quire_rs::symbol_table::build(&extraction, None);
    let record = report
        .symbols
        .iter()
        .find(|r| r.kind == "entry_point")
        .expect("entry_point record");
    assert!(record.binds_trace_ids && !record.carries_implements);
    let graph = bind(&extraction, &model(Some(vec!["scripts/**"])));
    let input = quire_rs::symbols::trace::SymbolRecordInput {
        org: "o".into(),
        repo_name: "r".into(),
        project_id: "p".into(),
        document_id: "d".into(),
        artifact_id: None,
    };
    let (nodes, _) =
        quire_rs::symbols::trace::graph_records(&extraction, &graph, &input).expect("records");
    assert!(
        nodes
            .iter()
            .any(|n| n.data_json.contains("\"kind\":\"entry_point\"")),
        "{nodes:?}"
    );
    assert!(SymbolKind::EntryPoint.binds_trace_ids());
    assert!(!SymbolKind::EntryPoint.carries_implements());
    assert_eq!(SymbolKind::EntryPoint.as_str(), "entry_point");
}

#[trace("TC-2105", "FR-051-AC-38")]
// the entry point binds only under a matching glob; a function never binds.
#[test]
fn tc2105_eligibility() {
    let tagged = "#!/bin/sh\n# trace: FR-001-AC-1\nf() { :; }\n";
    let inside = bind(
        &sh("scripts/a.sh", tagged),
        &model(Some(vec!["scripts/**"])),
    );
    assert_eq!(verified(&inside), [("entry_point", "FR-001-AC-1")]);
    assert_eq!(inside.verifies[0].form, "shell-trace-comment");
    assert_eq!(inside.verifies[0].kind, SymbolKind::EntryPoint);

    let outside = bind(&sh("other/a.sh", tagged), &model(Some(vec!["scripts/**"])));
    assert!(outside.verifies.is_empty());

    let function_tag = "set -e\n# trace: FR-001-AC-2\nf() { :; }\n";
    let graph = bind(
        &sh("scripts/a.sh", function_tag),
        &model(Some(vec!["scripts/**"])),
    );
    assert!(
        graph.verifies.is_empty(),
        "a shell function binds no verifies: {:?}",
        graph.verifies
    );
}

#[trace("TC-2106", "FR-051-AC-39")]
// glob semantics: source_exclude dialect, code-root relative, a union across
// forms, source_exclude first, and an uncompilable pattern refused.
#[test]
fn tc2106_entry_paths_globs() {
    // Anchored at the code root: `scripts/**` does not reach `src/scripts/`.
    let tagged = "# trace: FR-001-AC-1\n";
    let nested = bind(
        &sh("src/scripts/a.sh", tagged),
        &model(Some(vec!["scripts/**"])),
    );
    assert!(nested.verifies.is_empty());

    // Union across two forms.
    let two: TraceabilityModel = serde_json::from_value(json!({ "trace_tags": { "markers": [
        { "name": "one", "language": "shell", "syntax": "comment",
          "pattern": r"#\s*trace:\s*(.+)$", "entry_paths": ["a/**"] },
        { "name": "two", "language": "shell", "syntax": "comment",
          "pattern": r"#\s*check:\s*(.+)$", "entry_paths": ["b/**"] },
    ]}}))
    .expect("model");
    two.validate().expect("valid");
    for path in ["a/x.sh", "b/x.sh"] {
        let graph = bind(&sh(path, tagged), &two);
        assert_eq!(graph.verifies.len(), 1, "{path}");
    }

    // source_exclude is applied first: an excluded file mints no symbol.
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), "scripts/a.sh", tagged);
    let excluded = extract_tree_scoped(dir.path(), &[], &["scripts/**".to_string()]);
    assert!(excluded.symbols.is_empty());
    assert_eq!(excluded.excluded_source_files, 1);

    // Load refuses a pattern that does not compile, naming the key.
    let bad: TraceabilityModel = serde_json::from_value(json!({ "trace_tags": { "markers": [
        { "name": "one", "language": "shell", "syntax": "comment",
          "pattern": r"#\s*trace:\s*(.+)$", "entry_paths": ["a/[**"] },
    ]}}))
    .expect("shape");
    let err = bad.validate().expect_err("uncompilable");
    assert!(err.contains("entry_paths"), "{err}");

    // An extraction handed the uncompiled glob applies none and says so.
    let graph = bind(&sh("a/x.sh", tagged), &bad);
    assert!(graph.verifies.is_empty());
    let not_applied: Vec<&str> = graph
        .diagnostics
        .iter()
        .filter(|d| d.reason.starts_with("entry_paths not applied"))
        .map(|d| d.path.as_str())
        .collect();
    assert_eq!(not_applied, ["a/[**"]);
}

#[trace("TC-2107", "FR-051-AC-40")]
// the tag channel is the comment block above a function, and the first header
// run before the first command for the entry point.
#[test]
fn tc2107_tag_channel() {
    let src = "\
#!/usr/bin/env bash

# trace: FR-001-AC-1
# trace: FR-001-AC-2

# trace: FR-001-AC-3
set -euo pipefail
# trace: FR-001-AC-4

# implements: FR-009
# trace: FR-001-AC-5
func() { :; }
";
    let extraction = sh("scripts/a.sh", src);
    let graph = bind(&extraction, &model(Some(vec!["scripts/**"])));
    let ids: Vec<&str> = graph.verifies.iter().map(|v| v.trace_id.as_str()).collect();
    // First run only: AC-3 sits after a blank line, AC-4 after the first command.
    assert_eq!(ids, ["FR-001-AC-1", "FR-001-AC-2"]);
    let func = extraction
        .symbols
        .iter()
        .find(|s| s.qualified_name == "func")
        .expect("func");
    let channel: Vec<usize> = func.tag_comments.iter().map(|c| c.line).collect();
    assert_eq!(
        channel,
        [10, 11],
        "the contiguous block above, a blank line ends it"
    );
    assert_eq!(func.leading_line, 10);
}

#[trace("TC-2108", "FR-051-AC-41")]
// only a comment node opens a comment.
#[test]
fn tc2108_only_comment_nodes_are_comments() {
    let src = "\
# trace: FR-001-AC-1
echo \"# trace: FR-002-AC-2\" $# ${#x} ${x#pat} a#b http://h/p#trace:FR-003-AC-3
cat <<EOF
# trace: FR-004-AC-4
EOF
";
    let extraction = sh("scripts/a.sh", src);
    let comments: Vec<usize> = extraction.files[0]
        .comments
        .iter()
        .map(|c| c.line)
        .collect();
    assert_eq!(comments, [1]);
    let graph = bind(&extraction, &model(Some(vec!["scripts/**"])));
    assert_eq!(verified(&graph), [("entry_point", "FR-001-AC-1")]);
}

#[trace("TC-2109", "FR-051-AC-42")]
// shell binds only through a declared shell comment form.
#[test]
fn tc2109_no_declared_form_mints_nothing() {
    let src = "# trace: FR-001-AC-1\n";
    let none = TraceabilityModel::default();
    assert!(bind(&sh("scripts/a.sh", src), &none).verifies.is_empty());
    // A non-shell form does not stand in for it.
    let rust_only: TraceabilityModel = serde_json::from_value(json!({ "trace_tags": { "markers": [
        { "name": "r", "language": "rust", "pattern": r"#\s*trace:\s*(.+)$" },
    ]}}))
    .expect("model");
    assert!(bind(&sh("scripts/a.sh", src), &rust_only)
        .verifies
        .is_empty());

    // A shell form without `syntax: comment` is refused at load, naming it...
    let no_syntax = json!({ "trace_tags": { "markers": [
        { "name": "plain", "language": "shell", "pattern": r"#\s*trace:\s*(.+)$",
          "entry_paths": ["scripts/**"] },
    ]}});
    let model: TraceabilityModel = serde_json::from_value(no_syntax).expect("shape");
    let err = model.validate().expect_err("shell form without syntax");
    assert!(
        err.contains("plain") && err.contains("syntax: comment"),
        "{err}"
    );
    // ...and a model that skipped validation binds nothing through it.
    assert!(bind(&sh("scripts/a.sh", src), &model).verifies.is_empty());
}

#[trace("TC-2110", "FR-051-AC-43")]
// load-time refusals, and the accepted shell implements form.
#[test]
fn tc2110_load_refusals() {
    let load = |value: serde_json::Value| -> Result<(), String> {
        let model: TraceabilityModel = serde_json::from_value(value).map_err(|e| e.to_string())?;
        model.validate()
    };
    let pattern = r"#\s*trace:\s*(.+)$";

    let err = load(json!({ "trace_tags": { "markers": [
        { "name": "r", "language": "rust", "syntax": "comment", "pattern": pattern } ]}}))
    .expect_err("comment off shell");
    assert!(err.contains("comment"), "{err}");

    let err = load(json!({ "trace_tags": { "bogus": [] }})).expect_err("unknown key");
    assert!(err.contains("bogus"), "{err}");

    // CR-215: the retired `legacy` list is refused by name, like any unknown key.
    let err = load(json!({ "trace_tags": { "legacy": [
        { "name": "l", "language": "rust", "pattern": pattern, "rewrite_to": "r" } ]}}))
    .expect_err("legacy key");
    assert!(err.contains("legacy"), "{err}");

    let err = load(json!({ "trace_tags": { "markers": [
        { "name": "r", "language": "rust", "pattern": pattern, "entry_paths": ["a/**"] } ]}}))
    .expect_err("entry_paths off shell");
    assert!(err.contains("entry_paths"), "{err}");

    let err = load(json!({ "trace_tags": { "implements": [
        { "name": "s", "language": "shell", "syntax": "comment", "pattern": pattern,
          "entry_paths": ["a/**"] } ]}}))
    .expect_err("entry_paths on implements");
    assert!(err.contains("entry_paths"), "{err}");

    load(json!({ "trace_tags": { "implements": [
        { "name": "s", "language": "shell", "syntax": "comment", "pattern": pattern } ]}}))
    .expect("a shell implements form is accepted");

    // The implements form binds on a function and not on the entry point.
    let model: TraceabilityModel = serde_json::from_value(json!({ "trace_tags": { "implements": [
        { "name": "s", "language": "shell", "syntax": "comment",
          "pattern": r"#\s*implements:\s*(.+)$" } ]}}))
    .expect("model");
    let src = "# implements: FR-001\nf() { :; }\n";
    let graph = bind(&sh("scripts/a.sh", src), &model);
    assert_eq!(graph.implements.len(), 1);
    assert_eq!(graph.implements[0].symbol, "f");
    assert!(graph.verifies.is_empty());
}

#[trace("TC-2111", "FR-051-AC-44")]
// a marker that matched and bound nothing is reported with its reason.
#[test]
fn tc2111_non_binding_markers_are_reported() {
    let src = "\
#!/bin/sh
# trace: FR-001-AC-1
set -e
# trace: FR-001-AC-2

# trace: FR-001-AC-3
f() { :; }
";
    let outside = bind(&sh("other/a.sh", src), &model(Some(vec!["scripts/**"])));
    let got: Vec<(&str, usize, &str)> = outside
        .shell_findings
        .iter()
        .map(|f| (f.reason, f.line, f.value.as_str()))
        .collect();
    assert_eq!(
        got,
        [
            ("outside-entry-paths", 2, "FR-001-AC-1"),
            ("not-in-tag-channel", 4, "FR-001-AC-2"),
            ("function-not-evidence", 6, "FR-001-AC-3"),
        ]
    );
    // AC-44: "no census count moves" — shell is not in the binding census.
    assert!(
        outside.binding_census.iter().all(|c| c.language != "shell"),
        "{:?}",
        outside.binding_census
    );
    // A range binds nothing wherever it sits and leaves a record (AC-28): in
    // an ineligible header, a function's block, and any other comment.
    let ranges = "\
#!/bin/sh
# trace: FR-001-AC-1..FR-001-AC-3
set -e
# trace: FR-002-AC-1..FR-002-AC-3

# trace: FR-003-AC-1..FR-003-AC-3
f() { :; }
";
    let graph = bind(&sh("other/a.sh", ranges), &model(Some(vec!["scripts/**"])));
    let got: Vec<(usize, &str, &str)> = graph
        .range_diagnostics
        .iter()
        .map(|r| (r.line, r.symbol.as_str(), r.range_text.as_str()))
        .collect();
    assert_eq!(
        got,
        [
            (2, "entry_point", "FR-001-AC-1..FR-001-AC-3"),
            (4, "entry_point", "FR-002-AC-1..FR-002-AC-3"),
            (6, "f", "FR-003-AC-1..FR-003-AC-3"),
        ]
    );
    assert!(graph.verifies.is_empty() && graph.shell_findings.is_empty());

    // Inside a glob only the two that cannot bind remain.
    let inside = bind(&sh("scripts/a.sh", src), &model(Some(vec!["scripts/**"])));
    let got: Vec<&str> = inside.shell_findings.iter().map(|f| f.reason).collect();
    assert_eq!(got, ["not-in-tag-channel", "function-not-evidence"]);
}

#[trace("TC-2112", "FR-051-AC-45")]
// an unparseable file is a per-file diagnostic naming the line; siblings extract.
#[test]
fn tc2112_unparseable_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), "ok.sh", "f() { :; }\n");
    write(dir.path(), "bad.sh", "echo ok\nf() { echo \"unterminated\n");
    let extraction = extract_tree(dir.path());
    assert_eq!(
        extraction.diagnostics.len(),
        1,
        "{:?}",
        extraction.diagnostics
    );
    assert_eq!(extraction.diagnostics[0].path, "bad.sh");
    assert!(
        extraction.diagnostics[0].reason.starts_with("line 2:"),
        "{}",
        extraction.diagnostics[0].reason
    );
    assert!(extraction.symbols.iter().all(|s| s.path == "ok.sh"));
    assert_eq!(names(&extraction.symbols, SymbolKind::Function), ["f"]);
}

#[trace("TC-2113", "FR-051-AC-46")]
// end to end: a tagged script under a glob backs its row; outside it, nothing.
#[test]
fn tc2113_audit_script_backs_its_static_row() {
    let root = tempfile::tempdir().expect("tempdir");
    let scope = root.path().join("spec");
    write(
        &scope,
        "FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Static (TC-001) |\n",
    );
    write(
        &scope,
        "tests.md",
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n# Test Matrix\n\n\
         ## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-001 | FR-001-AC-1 | ✅ |\n",
    );
    let script = "#!/usr/bin/env bash\n# audit\n# trace: TC-001\nset -euo pipefail\n";
    write(root.path(), "scripts/audits/check.sh", script);
    write(root.path(), "tools/check.sh", script);

    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/traceability/iso");
    let registry = Registry::load_module(&module).expect("load module");
    let spec = Spec::from_path(&scope);
    let report_for = |entry_paths: &str| {
        let mut model = registry.traceability().cloned().expect("model");
        model.trace_tags.markers.push(
            serde_json::from_value(json!({
                "name": "shell-trace-comment", "language": "shell", "syntax": "comment",
                "pattern": r"#\s*trace:\s*(.+)$", "entry_paths": [entry_paths],
            }))
            .expect("form"),
        );
        let extraction = extract_tree_scoped(root.path(), &[Path::new("spec")], &[]);
        let graph = bind(&extraction, &model);
        compute(&spec, &registry, &graph, &scope).expect("model declared")
    };

    let backed = report_for("scripts/audits/*.sh");
    assert_eq!(backed.totals.backed, 1, "{:?}", backed.unbacked_rows);
    assert!(
        backed.unbacked_rows.is_empty(),
        "{:?}",
        backed.unbacked_rows
    );
    // The same marker in the second script sits outside every glob.
    assert!(
        backed
            .diagnostics
            .iter()
            .any(|d| d.reason == "outside-entry-paths"
                && d.path.as_deref() == Some("tools/check.sh")),
        "{:?}",
        backed.diagnostics
    );

    let unbacked = report_for("nowhere/**");
    assert_eq!(unbacked.totals.backed, 0);
    assert!(unbacked
        .unbacked_rows
        .iter()
        .any(|r| r.row_id.as_deref() == Some("TC-001")));
    assert_eq!(
        unbacked
            .diagnostics
            .iter()
            .filter(|d| d.reason == "outside-entry-paths")
            .count(),
        2
    );
}
