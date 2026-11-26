//! The source walk, collision attribution and identity, and the
//! ignored-file count (PLAT-1793). Scenarios are declarative cases under
//! `tests/fixtures/corpus_cases/plat-1793-*.json`; this file only materializes
//! and grades them.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::coverage::compute;
use quire_rs::symbols::trace as bind_trace;
use quire_rs::symbols::trace_search::{search, Query};
use quire_rs::symbols::{extract_file, extract_tree, extract_tree_scoped, SymbolExtraction};
use quire_rs::traceability::{SourceLanguage, TraceMarkerForm, TraceabilityModel};
use quire_rs::{symbol_table, Registry, Spec};
use serde_json::Value;

fn cases(json: &str) -> Vec<Value> {
    let file: Value = serde_json::from_str(json).expect("case file parses");
    file["cases"].as_array().expect("cases array").clone()
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|v| v.as_str().expect("string").to_string())
        .collect()
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

fn materialize(case: &Value) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    // The tempdir's own name starts with `.`; nest so the code root's
    // ancestors are never read as part of the case. `{outside}` in a link
    // target names a directory beside the tree, outside every code root.
    let top = dir.path().join("top");
    let outside = dir.path().join("outside");
    for (rel, body) in case["input"]["files"].as_object().expect("files") {
        write(&top, rel, body.as_str().expect("body"));
    }
    for (link, target) in case["input"]["symlinks"].as_object().into_iter().flatten() {
        let at = top.join(link);
        fs::create_dir_all(at.parent().expect("parent")).expect("mkdir");
        let target = target.as_str().expect("target");
        let target = target.replace("{outside}", &outside.to_string_lossy());
        std::os::unix::fs::symlink(target, at).expect("symlink");
    }
    let root = match case["input"]["root"].as_str() {
        Some(sub) => top.join(sub),
        None => top,
    };
    (dir, root)
}

fn walk(case: &Value, root: &Path) -> SymbolExtraction {
    let globs = strings(&case["input"]["source_exclude"]);
    let doc_root = case["input"]["document_root"].as_str().map(PathBuf::from);
    let dirs: Vec<&Path> = doc_root.iter().map(PathBuf::as_path).collect();
    extract_tree_scoped(root, &dirs, &globs)
}

fn sorted_symbol_files(out: &SymbolExtraction) -> Vec<String> {
    let set: BTreeSet<&str> = out.symbols.iter().map(|s| s.path.as_str()).collect();
    set.into_iter().map(str::to_string).collect()
}

#[trace("FR-051-AC-54", "FR-051-AC-55", "FR-050-AC-69")]
#[test]
fn source_walk_cases_hold() {
    let all = cases(include_str!(
        "../fixtures/corpus_cases/plat-1793-source-walk.json"
    ));
    assert!(all.len() >= 16);
    for case in &all {
        let name = case["name"].as_str().expect("name");
        assert_eq!(case["issue_ref"], "PLAT-1793", "{name}");
        let (_dir, root) = materialize(case);
        let out = walk(case, &root);
        let expect = &case["expect"];

        assert_eq!(
            sorted_symbol_files(&out),
            strings(&expect["symbol_files"]),
            "{name}: files yielding symbols"
        );
        if let Some(n) = expect["ignored_source_files"].as_u64() {
            assert_eq!(out.ignored_source_files as u64, n, "{name}: ignored");
        }
        if let Some(n) = expect["excluded_source_files"].as_u64() {
            assert_eq!(out.excluded_source_files as u64, n, "{name}: excluded");
        }
        if expect.get("symlink_diagnostics").is_some() {
            let links: Vec<&quire_rs::symbols::SymbolDiagnostic> = out
                .diagnostics
                .iter()
                .filter(|d| d.reason.starts_with("symlink-not-followed"))
                .collect();
            let paths: Vec<String> = links.iter().map(|d| d.path.clone()).collect();
            assert_eq!(
                paths,
                strings(&expect["symlink_diagnostics"]),
                "{name}: symlink diagnostics"
            );
            assert!(
                links
                    .iter()
                    .all(|d| d.severity == quire_rs::symbols::DiagnosticSeverity::Warning),
                "{name}: severity"
            );
        }
    }
}

/// Mutation sensitivity: the walk is not vacuously green — the same tree
/// differs when its ignore file is removed, and the count follows.
#[trace("FR-051-AC-55")]
#[test]
fn removing_the_ignore_file_changes_the_result() {
    let all = cases(include_str!(
        "../fixtures/corpus_cases/plat-1793-source-walk.json"
    ));
    let case = all
        .iter()
        .find(|c| c["name"] == "gitignore-removes-source-without-git")
        .expect("case");
    let (_dir, root) = materialize(case);
    let with = extract_tree(&root);
    fs::remove_file(root.join(".gitignore")).expect("remove");
    let without = extract_tree(&root);
    assert_eq!(with.ignored_source_files, 2);
    assert_eq!(without.ignored_source_files, 0);
    assert!(sorted_symbol_files(&without).len() > sorted_symbol_files(&with).len());
    // Un-ignoring yields the file's symbols.
    assert!(sorted_symbol_files(&without).contains(&"ignored_tests/a.rs".to_string()));
}

/// The walk is identical however the machine is configured: a user-global
/// ignore file under `HOME` / `XDG_CONFIG_HOME` is never read.
#[trace("FR-051-AC-55")]
#[test]
fn extraction_is_independent_of_home_and_xdg_config_home() {
    if let Ok(root) = std::env::var("QUIRE_WALK_PROBE_ROOT") {
        let out = extract_tree(Path::new(&root));
        println!(
            "PROBE:{:?}:{}:END",
            sorted_symbol_files(&out),
            out.ignored_source_files
        );
        return;
    }
    let tree = tempfile::tempdir().expect("tempdir");
    write(tree.path(), "lib.rs", "fn a() {}\n");
    write(tree.path(), "tests/t.rs", "fn t() {}\n");
    let home = tempfile::tempdir().expect("home");
    write(home.path(), ".config/git/ignore", "*.rs\n");
    write(
        home.path(),
        ".gitconfig",
        "[core]\n\texcludesFile = ~/.config/git/ignore\n",
    );

    let probe = |home: &Path| -> String {
        let out = std::process::Command::new(std::env::current_exe().expect("exe"))
            .args([
                "--exact",
                "source_walk::extraction_is_independent_of_home_and_xdg_config_home",
                "--nocapture",
            ])
            .env("QUIRE_WALK_PROBE_ROOT", tree.path())
            .env("HOME", home)
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .output()
            .expect("probe runs");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        stdout
            .split_once("PROBE:")
            .and_then(|(_, output)| output.split_once(":END"))
            .map(|(probe, _)| probe.to_string())
            .unwrap_or_else(|| panic!("probe output missing from child stdout:\n{stdout}"))
    };
    let empty = tempfile::tempdir().expect("empty home");
    let hostile = probe(home.path());
    assert_eq!(hostile, probe(empty.path()));
    assert!(
        hostile.contains("lib.rs") && hostile.contains("tests/t.rs"),
        "{hostile}"
    );
}

fn model() -> TraceabilityModel {
    let mut model = TraceabilityModel::default();
    for (list, name, pattern) in [
        (true, "rust-trace-attr", r#"#\[trace\("([^"]+)"\)\]"#),
        (
            false,
            "rust-implements-attr",
            r#"#\[implements\("([^"]+)"\)\]"#,
        ),
    ] {
        let form = TraceMarkerForm {
            syntax: None,
            name: name.to_string(),
            language: SourceLanguage::Rust,
            pattern: pattern.to_string(),
            template: None,
            entry_paths: None,
        };
        if list {
            model.trace_tags.markers.push(form);
        } else {
            model.trace_tags.implements.push(form);
        }
    }
    model
}

#[trace("FR-051-AC-56")]
#[test]
fn colliding_declarations_are_attributed_individually() {
    let all = cases(include_str!(
        "../fixtures/corpus_cases/plat-1793-collision-attribution.json"
    ));
    assert!(all.len() >= 5);
    for case in &all {
        let name = case["name"].as_str().expect("name");
        assert_eq!(case["issue_ref"], "PLAT-1793", "{name}");
        let source = case["input"]["source"].as_str().expect("source");
        let extraction = extract_file("src/a.rs", SourceLanguage::Rust, source);
        let graph = bind_trace::bind(&extraction, &model());
        let report = symbol_table::build(&extraction, Some(&graph));
        let expect = &case["expect"];

        for (key, pick) in [("records", false), ("implements_records", true)] {
            let want = expect[key].as_array().into_iter().flatten();
            let mut got: BTreeMap<usize, Vec<String>> = BTreeMap::new();
            for record in report.symbols.iter().filter(|r| {
                r.symbol == "t" || r.symbol == "p" || r.symbol == "one" || r.symbol == "two"
            }) {
                let ids = if pick {
                    &record.implements
                } else {
                    &record.trace_ids
                };
                got.insert(record.line, ids.clone());
            }
            for entry in want {
                let line = entry["line"].as_u64().expect("line") as usize;
                let ids = strings(if pick {
                    &entry["ids"]
                } else {
                    &entry["trace_ids"]
                });
                assert_eq!(got.get(&line), Some(&ids), "{name}: {key} at line {line}");
            }
        }
        let backed: Vec<String> = graph
            .backed_trace_ids()
            .into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!(backed, strings(&expect["backed"]), "{name}: backed");

        if let Some(ids) = expect.get("inverse_ids") {
            let found = search(
                &graph,
                &extraction,
                &Query::Symbol {
                    path: "src/a.rs".into(),
                    qualified_name: "t".into(),
                },
            )
            .expect("available");
            let mut claimed: Vec<String> =
                found.verifies.iter().map(|v| v.trace_id.clone()).collect();
            claimed.sort();
            assert_eq!(claimed, strings(ids), "{name}: inverse query");
        }
        let reasons: Vec<&str> = extraction
            .diagnostics
            .iter()
            .map(|d| d.reason.as_str())
            .collect();
        for want in strings(&expect["diagnostic_reasons"]) {
            assert!(
                reasons.iter().any(|r| r.starts_with(&want)),
                "{name}: {reasons:?}"
            );
        }
        for absent in strings(&expect["absent_diagnostic_reasons"]) {
            assert!(
                !reasons.iter().any(|r| r.starts_with(&absent)),
                "{name}: {reasons:?}"
            );
        }
    }
}

fn tree_extraction(files: &Value) -> (tempfile::TempDir, SymbolExtraction) {
    let dir = tempfile::tempdir().expect("tempdir");
    for (rel, body) in files.as_object().expect("files") {
        write(dir.path(), rel, body.as_str().expect("body"));
    }
    let out = extract_tree(dir.path());
    (dir, out)
}

#[trace("FR-051-AC-57")]
#[test]
fn identity_survives_unrelated_edits_and_only_positions_move() {
    let all = cases(include_str!(
        "../fixtures/corpus_cases/plat-1793-identity.json"
    ));
    for case in &all {
        let name = case["name"].as_str().expect("name");
        let (_b, before) = tree_extraction(&case["input"]["before"]);
        let (_a, after) = tree_extraction(&case["input"]["after"]);
        let expect = &case["expect"];
        let model = model();
        let g_before = bind_trace::bind(&before, &model);
        let g_after = bind_trace::bind(&after, &model);

        let relations = |g: &bind_trace::SymbolGraph, id: &str| -> Vec<String> {
            let mut v: Vec<String> = g
                .verifies
                .iter()
                .filter(|r| r.symbol_id == id)
                .map(|r| r.trace_id.clone())
                .collect();
            v.sort();
            v
        };
        for file in strings(&expect["identical_files"]) {
            let pick = |out: &SymbolExtraction| -> Vec<_> {
                out.symbols
                    .iter()
                    .filter(|s| s.path == file)
                    .cloned()
                    .collect()
            };
            assert_eq!(pick(&before), pick(&after), "{name}: {file} byte-identical");
        }
        for (symbol, want) in expect["kept"].as_object().expect("kept") {
            let find = |out: &SymbolExtraction| {
                out.symbols
                    .iter()
                    .find(|s| s.qualified_name == *symbol)
                    .cloned()
                    .expect("symbol")
            };
            let (b, a) = (find(&before), find(&after));
            assert_eq!(
                (&b.id, b.kind, &b.qualified_name, b.language, &b.path),
                (&a.id, a.kind, &a.qualified_name, a.language, &a.path),
                "{name}: {symbol} identity"
            );
            let want: Vec<String> = strings(want);
            assert_eq!(relations(&g_before, &b.id), want, "{name}: {symbol} before");
            assert_eq!(relations(&g_after, &a.id), want, "{name}: {symbol} after");
            let shift = expect["line_shift"][symbol].as_u64().expect("shift") as usize;
            assert_eq!(a.line, b.line + shift, "{name}: {symbol} line shift");
        }
        let names = |out: &SymbolExtraction| -> BTreeSet<String> {
            out.symbols
                .iter()
                .map(|s| s.qualified_name.clone())
                .collect()
        };
        let added: BTreeSet<String> = names(&after).difference(&names(&before)).cloned().collect();
        assert_eq!(
            added,
            strings(&expect["added"]).into_iter().collect(),
            "{name}: added"
        );
    }
}

fn iso_bundle(source_files: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let scope = dir.path().join("spec");
    let source = dir.path().join("src");
    write(
        &scope,
        "FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test (TC-001) |\n",
    );
    write(
        &scope,
        "tests.md",
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n# Test Matrix\n\n\
         ## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-001 | FR-001-AC-1 | ✅ |\n",
    );
    for (rel, body) in source_files {
        write(&source, rel, body);
    }
    (dir, scope, source)
}

fn report_json(scope: &Path, source: &Path) -> (quire_rs::coverage::CoverageReport, String) {
    let module = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/traceability/iso");
    let registry = Registry::load_module(&module).expect("module");
    let spec = Spec::from_path(scope);
    let model = registry.traceability().cloned().expect("model");
    let graph = bind_trace::bind(&extract_tree(source), &model);
    let report = compute(&spec, &registry, &graph, scope).expect("report");
    let json = report.to_json();
    (report, json)
}

const TAGGED: &str =
    "#[cfg(test)]\nmod tests {\n    #[trace(\"TC-001\")]\n    #[test]\n    fn covers() {}\n}\n";

#[trace("FR-050-AC-69")]
#[test]
fn ignored_source_count_reaches_the_report_and_is_absent_when_zero() {
    let (_d, scope, source) = iso_bundle(&[
        (".gitignore", "hidden_tests/\n"),
        ("lib.rs", TAGGED),
        ("hidden_tests/more.rs", TAGGED),
    ]);
    let (report, json) = report_json(&scope, &source);
    assert_eq!(report.ignored_source_files, 1);
    assert_eq!(report.excluded_source_files, 0, "counted in at most one");
    assert!(json.contains("\"ignored_source_files\": 1"), "{json}");

    let (_d, scope, source) = iso_bundle(&[("lib.rs", TAGGED), ("more/more.rs", TAGGED)]);
    let (report, json) = report_json(&scope, &source);
    assert_eq!(report.ignored_source_files, 0);
    assert!(
        !json.contains("ignored_source_files"),
        "key omitted, never 0"
    );
}

/// A marker naming an id no declared target mints still binds, answers a
/// forward query and is an untracked symbol; fixing it changes only that
/// symbol.
#[trace("FR-051-AC-57")]
#[test]
fn a_stale_marker_binds_and_is_reported_untracked() {
    let stale = "#[cfg(test)]\nmod tests {\n    #[trace(\"TC-999\")]\n    #[test]\n    fn covers() {}\n\n    #[trace(\"TC-001\")]\n    #[test]\n    fn healthy() {}\n}\n";
    let (_d, scope, source) = iso_bundle(&[("lib.rs", stale)]);
    let (report, _) = report_json(&scope, &source);
    let untracked: Vec<&str> = report
        .untracked_symbols
        .iter()
        .map(|s| s.trace_id.as_str())
        .collect();
    assert_eq!(untracked, vec!["TC-999"]);

    let extraction = extract_tree(&source);
    let module = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/traceability/iso");
    let registry = Registry::load_module(&module).expect("module");
    let graph = bind_trace::bind(&extraction, registry.traceability().expect("model"));
    let found = search(&graph, &extraction, &Query::Id("TC-999".into())).expect("available");
    assert_eq!(found.verifies.len(), 1, "the stale marker is still bound");

    // Correcting the marker changes only that symbol's relation and entry.
    fs::write(source.join("lib.rs"), stale.replace("TC-999", "TC-001")).expect("write");
    let (fixed, _) = report_json(&scope, &source);
    assert!(fixed.untracked_symbols.is_empty());
}

/// Apply a case's `after` step to a materialized tree.
fn apply_after(dir: &tempfile::TempDir, root: &Path, after: &Value) {
    for (rel, body) in after["files"].as_object().into_iter().flatten() {
        write(root, rel, body.as_str().expect("body"));
    }
    for (rel, body) in after["outside_files"].as_object().into_iter().flatten() {
        write(
            &dir.path().join("outside"),
            rel,
            body.as_str().expect("body"),
        );
    }
    for rel in strings(&after["remove"]) {
        fs::remove_file(root.join(rel)).expect("remove");
    }
}

/// A link's target, present or absent or deleted, never changes the
/// extraction apart from what the target's own symbols are (FR-051-AC-54).
#[trace("FR-051-AC-54")]
#[test]
fn symlink_targets_never_change_the_extraction() {
    let all = cases(include_str!(
        "../fixtures/corpus_cases/plat-1793-source-walk.json"
    ));
    let mut ran = 0;
    for case in all.iter().filter(|c| c.get("after").is_some()) {
        let name = case["name"].as_str().expect("name");
        let (dir, root) = materialize(case);
        let before = walk(case, &root);
        apply_after(&dir, &root, &case["after"]);
        let after = walk(case, &root);
        let expect = &case["expect_after"];
        if expect["identical"] == true {
            assert_eq!(before, after, "{name}");
        }
        let removed = strings(&expect["removed_symbol_files"]);
        if !removed.is_empty() {
            let kept = |out: &SymbolExtraction| -> Vec<_> {
                out.symbols
                    .iter()
                    .filter(|s| !removed.contains(&s.path))
                    .cloned()
                    .collect()
            };
            assert_eq!(kept(&before), kept(&after), "{name}: other symbols");
            assert!(
                before.symbols.iter().any(|s| removed.contains(&s.path)),
                "{name}"
            );
            assert!(
                after.symbols.iter().all(|s| !removed.contains(&s.path)),
                "{name}"
            );
            assert_eq!(before.diagnostics, after.diagnostics, "{name}: diagnostics");
            assert_eq!(
                (before.ignored_source_files, before.excluded_source_files),
                (after.ignored_source_files, after.excluded_source_files),
                "{name}: counts"
            );
        }
        ran += 1;
    }
    assert_eq!(ran, 2);
}

/// `.git` presence never changes what an ignore file removes (FR-051-AC-55).
#[trace("FR-051-AC-55")]
#[test]
fn a_git_directory_does_not_change_the_ignore_result() {
    let all = cases(include_str!(
        "../fixtures/corpus_cases/plat-1793-source-walk.json"
    ));
    let run = |name: &str| {
        let case = all.iter().find(|c| c["name"] == name).expect("case");
        let (_dir, root) = materialize(case);
        extract_tree(&root)
    };
    assert_eq!(
        run("gitignore-removes-source-without-git"),
        run("gitignore-removes-source-with-git")
    );
}
