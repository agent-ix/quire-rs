//! NFR-021 boundary and compatibility gates (TC-1619, TC-1620, TC-1627,
//! TC-1628, TC-1640, TC-1641, TC-1642). Plan-003 Task-021. TC-1636 (WASM
//! parity) is external: agent-ix/quire-wasm#3 runs it against
//! `tests/fixtures/semantic/cases.json` + `cases.expected.json`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use ix_trace_rs::trace;
use quire_rs::semantic::{extract_clauses, BundleIndex, SemanticContext};
use quire_rs::Registry;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn semantic_sources() -> Vec<(String, String)> {
    let dir = root().join("src/semantic");
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") {
            out.push((
                path.file_name().unwrap().to_string_lossy().to_string(),
                fs::read_to_string(&path).unwrap(),
            ));
        }
    }
    out
}

fn code_lines(source: &str) -> impl Iterator<Item = &str> {
    source.lines().filter(|l| !l.trim_start().starts_with("//"))
}

#[trace("TC-1641", "NFR-021-AC-1")]
// no denylisted crate in the graph, no evaluator symbol over clause text.
#[test]
fn no_clause_parser_or_template_dependency() {
    let lock = fs::read_to_string(root().join("Cargo.lock")).unwrap();
    for dep in [
        "ocl",
        "sysml",
        "fret",
        "fretish",
        "tera",
        "handlebars",
        "minijinja",
        "askama",
        "liquid",
    ] {
        assert!(
            !lock.contains(&format!("name = \"{dep}\"")),
            "denylisted crate {dep}"
        );
    }
    for (name, source) in semantic_sources() {
        for line in code_lines(&source) {
            for sym in [
                "eval(",
                "parse_expr",
                "typecheck",
                "Tera",
                "Handlebars",
                "minijinja",
            ] {
                assert!(!line.contains(sym), "{name}: {sym} in `{line}`");
            }
        }
    }
    // The audit script is wired into `make audit-static`.
    let status = Command::new(root().join("scripts/audits/check_semantic_boundary.sh"))
        .status()
        .unwrap();
    assert!(status.success());
}

#[trace("TC-1642", "NFR-021-AC-2")]
// no network, process, or filesystem write on the semantic path; the only
// filesystem read is the module-file reader of the resolver.
#[test]
fn no_network_process_or_writes() {
    let mut reads = 0;
    for (name, source) in semantic_sources() {
        for line in code_lines(&source) {
            for sym in [
                "std::net",
                "std::process",
                "Command::new",
                "reqwest",
                "ureq",
                "git2",
                "fs::write",
                "File::create",
                "OpenOptions",
                "write_all(",
            ] {
                assert!(!line.contains(sym), "{name}: {sym} in `{line}`");
            }
            if line.contains("std::fs::read") {
                reads += 1;
                assert_eq!(
                    name, "resolver.rs",
                    "filesystem read outside the resolver: {name}"
                );
            }
        }
    }
    assert_eq!(reads, 1, "exactly one filesystem read on the semantic path");
    // `make ci` carries the wasm32 check (TC-1649).
    let makefile = fs::read_to_string(root().join("Makefile")).unwrap();
    assert!(makefile.contains("check-wasm:"));
    assert!(makefile
        .lines()
        .any(|l| l.starts_with("ci:") && l.contains("check-wasm")));
    assert!(
        makefile.contains("--target wasm32-unknown-unknown --no-default-features --features wasm")
    );
    let workflow = fs::read_to_string(root().join(".github/workflows/ci.yml")).unwrap();
    assert!(workflow.contains("targets: wasm32-unknown-unknown"));
}

#[trace("TC-1619", "FR-070-CON-1")]
// no brace-content or pattern parser: braces and patterns are carried opaque.
#[test]
fn fence_and_pattern_text_stay_opaque() {
    let properties = fs::read_to_string(root().join("src/semantic/properties.rs")).unwrap();
    for line in code_lines(&properties) {
        assert!(
            !line.contains("regex::") && !line.contains("Regex::new"),
            "pattern compiled: `{line}`"
        );
    }
    let cargo = fs::read_to_string(root().join("Cargo.toml")).unwrap();
    assert!(!cargo.contains("sysml"), "no SysML crate");
}

#[trace("TC-1620", "FR-070-CON-2")]
// resolution reads only the BundleIndex and loaded modules.
#[test]
fn resolution_touches_no_filesystem_or_network() {
    for name in [
        "properties.rs",
        "context.rs",
        "clauses.rs",
        "surface.rs",
        "scan.rs",
        "decl.rs",
    ] {
        let source = fs::read_to_string(root().join("src/semantic").join(name)).unwrap();
        for line in code_lines(&source) {
            assert!(
                !line.contains("std::fs")
                    && !line.contains("read_dir")
                    && !line.contains("std::net"),
                "{name}: `{line}`"
            );
        }
    }
}

#[trace("TC-1627", "FR-071-CON-1")]
// no clause tokenizer, parser, or evaluator; arbitrary text round-trips.
#[test]
fn clause_text_is_only_copied() {
    let clauses = fs::read_to_string(root().join("src/semantic/clauses.rs")).unwrap();
    for line in code_lines(&clauses) {
        for sym in [
            "tokenize",
            "Lexer",
            "Parser::",
            "eval(",
            "evaluate(",
            "typecheck",
        ] {
            assert!(!line.contains(sym), "`{line}`");
        }
    }
    let registry =
        Registry::load_module(&root().join("tests/fixtures/semantic/quoin/module-ok")).unwrap();
    let module = registry
        .semantic_module("spec-objects-fixture")
        .unwrap()
        .clone();
    let ctx = SemanticContext::new(module, "x.md", BundleIndex::default())
        .with_source_identity("ix://agent-ix/x/spec");
    let body = "context X inv y: self.a->forAll(b | b.c <> null) and \"weird\" ✓ {braces} `ticks`";
    let md = format!(
        "---\nid: X\nobject: entity\n---\n## Invariants\n\n### inv\n\n```ocl\n{body}\n```\n"
    );
    let out = extract_clauses(&md, &ctx);
    assert_eq!(out.clause_text["inv"], body);
}

#[trace("TC-1628", "FR-071-CON-2")]
// spans agree with the code_block scanner on every fixture; the parser
// golden is untouched (the parser_golden suite pins it).
#[test]
fn spans_agree_with_the_code_block_scanner() {
    let registry =
        Registry::load_module(&root().join("tests/fixtures/semantic/quoin/module-ok")).unwrap();
    let module = registry
        .semantic_module("spec-objects-fixture")
        .unwrap()
        .clone();
    for name in [
        "config-version.table.md",
        "config-version.fence.md",
        "operations.md",
    ] {
        let raw = fs::read_to_string(
            root()
                .join("tests/fixtures/semantic/quoin/mapping")
                .join(name),
        )
        .unwrap();
        let ctx = SemanticContext::new(module.clone(), name, BundleIndex::default())
            .with_source_identity("ix://agent-ix/x/spec");
        let out = extract_clauses(&raw, &ctx);
        let doc = quire_rs::parse_document(&raw);
        // The code_block locator's scanner, whole-document, ocl only: the
        // fixtures carry ocl fences under `## Invariants` clauses only.
        let blocks = quire_rs::extract_diagrams(&doc, Some("ocl"));
        let clauses = out.clauses.as_ref().unwrap();
        assert_eq!(blocks.len(), clauses.len(), "{name}: block count");
        for (block, clause) in blocks.iter().zip(clauses) {
            assert_eq!(
                block.source, out.clause_text[&clause.clause_id],
                "{name}: {}",
                clause.clause_id
            );
            assert_eq!(block.language, clause.language);
        }
    }
    // Parser output is unaffected: the golden suite in tests/parser_golden.rs
    // compares parse_document byte for byte; here, only that it still parses
    // the fixtures without a semantic-specific path.
    let raw =
        fs::read_to_string(root().join("tests/fixtures/semantic/quoin/mapping/operations.md"))
            .unwrap();
    let doc = quire_rs::parse_document(&raw);
    assert!(quire_rs::section(&doc, "Operations").is_some());
}

#[trace("TC-1640", "FR-072-CON-2")]
// the surface renders nothing, generates nothing, writes nothing.
#[test]
fn surface_returns_values_only() {
    for name in ["surface.rs", "python_entry.rs"] {
        let source = fs::read_to_string(root().join("src/semantic").join(name)).unwrap();
        for line in code_lines(&source) {
            for sym in [
                "fs::",
                "File::",
                "render",
                "template",
                "codegen",
                "std::process",
            ] {
                assert!(
                    !line.to_lowercase().contains(&sym.to_lowercase()),
                    "{name}: `{line}`"
                );
            }
        }
    }
}
