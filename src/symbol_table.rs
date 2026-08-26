//! The extracted symbol table, as the engine actually built it (FR-051, #309).
//!
//! # Why this surface exists
//!
//! No command reported which source symbols the scanner found. `extract` reads
//! *documents*; `coverage` reports what symbols *bound*, which is several
//! transformations downstream. So the only way to answer "how many declarations
//! does the scanner lose?" was to **reimplement the scanner** and diff.
//!
//! That was done three times while sizing `agent-ix/quire-rs#274`, with three
//! different answers:
//!
//! | attempt | files losing decls | declarations lost |
//! |---|---|---|
//! | first (bare names) | 51 | 387 |
//! | review (bare names) | 51 | 386 |
//! | second (qualified) | 295 | 5,263 |
//!
//! **A defect in the scanner cannot be sized by a reimplementation of the
//! scanner.** The ports disagree precisely where the original is wrong — the
//! loss figure is stable only under bare-name comparison, and everything else
//! moves by an order of magnitude depending on how the port qualifies names and
//! walks nested scopes, which is the part under test.
//!
//! # What it reports
//!
//! Every symbol, with the fields that decide what happens to it downstream:
//! its kind, whether that kind can bind a trace id, whether it can carry an
//! `implements` marker, and the trace ids it actually bound. Those four
//! together are the answer to almost every "why did my tag not count" question,
//! and they were previously only inferable from a rollup two layers away.
//!
//! Deterministic for identical input: the extraction is already ordered by
//! `(path, line, qualified_name)` (NFR-006), and nothing here re-sorts it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::symbols::trace::SymbolGraph;
use crate::symbols::SymbolExtraction;

/// One symbol, with everything that decides its fate downstream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRecord {
    /// Repo-relative, `/`-separated, so identities are platform-stable.
    pub path: String,
    /// Qualified path within the file — `outer::inner::name`, `Class.method`,
    /// or a registration's title. THE FIELD THE PORTS DISAGREED ON: two
    /// reimplementations of `python.rs` produced 386 and 5,263 lost
    /// declarations from the same tree because they qualified names
    /// differently. This is the engine's own answer.
    pub symbol: String,
    pub kind: String,
    pub language: String,
    /// 1-based declaration line.
    pub line: usize,
    /// 1-based first line of the attached annotation block — attributes,
    /// decorators, or leading comments. Equals `line` when nothing precedes.
    ///
    /// Reported because a marker that failed to match is written HERE, not at
    /// the declaration, and that is the line the reader has to edit (#256).
    pub leading_line: usize,
    /// 1-based last line of the body.
    pub end_line: usize,
    /// Qualified name of the containing symbol, when nested.
    pub container: Option<String>,
    /// The identity digest (FR-045): language, path, qualified name, kind. No
    /// positional attribute, so a symbol keeps its id when lines move.
    pub id: String,
    /// Whether this KIND can bind a trace id at all. The first thing to check
    /// when a row will not bind: tagging harder never helps a `Container`, and
    /// this is what `agent-ix/quire-rs#312` is about.
    pub binds_trace_ids: bool,
    /// Whether this KIND can carry an `implements` marker. The exact complement
    /// of `binds_trace_ids`, so no symbol reads as both.
    pub carries_implements: bool,
    /// Trace ids this symbol actually bound, deduplicated and ordered.
    ///
    /// EMPTY IS NOT THE SAME AS `binds_trace_ids: false`. An empty list on a
    /// binding kind is a test nothing tagged; an empty list on a non-binding
    /// kind is a symbol no tag could ever have reached. Reporting both fields
    /// is what separates authoring absence from an instrument that cannot see.
    pub trace_ids: Vec<String>,
    /// Requirement ids this symbol claims to implement (FR-062). Never
    /// evidence: this is scope, and letting it back a criterion is the coverage
    /// backdoor CR-061 closed.
    pub implements: Vec<String>,
}

/// A file the extractor could not read, with the reason it gave.
///
/// Extraction degrades per file — one unparseable file never aborts the tree
/// (FR-051-CON-2) — so a report that omitted these would show a smaller symbol
/// table with nothing saying why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileDiagnostic {
    pub path: String,
    pub reason: String,
}

/// Per-language totals, so a reader sees the shape of the table before the rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageCount {
    pub language: String,
    pub symbols: usize,
    pub files: usize,
    /// Symbols of a kind that CAN bind a trace id — the denominator any
    /// binding rate is drawn from.
    pub binding_kinds: usize,
    /// Of those, how many bound at least one id.
    pub bound: usize,
}

/// The extracted symbol table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolTableReport {
    pub symbols: Vec<SymbolRecord>,
    pub by_language: Vec<LanguageCount>,
    pub diagnostics: Vec<FileDiagnostic>,
    /// Files a declared `source_exclude` glob removed from the walk. A count,
    /// because what the operator needs is to notice that an over-broad glob
    /// subtracted real backing — which otherwise reads exactly like tests that
    /// were never written (FR-050-AC-24).
    pub excluded_source_files: usize,
    pub files: usize,
}

/// Build the report from an extraction and, optionally, the binding graph.
///
/// `graph` is separate because the two are separate questions. Without it the
/// report answers "what did the scanner find", which is #309's actual ask and
/// needs no declared model. With it, every record also carries the ids that
/// bound — and that half REQUIRES a module, since which forms exist is a
/// declaration.
pub fn build(extraction: &SymbolExtraction, graph: Option<&SymbolGraph>) -> SymbolTableReport {
    // Keyed on the symbol's identity digest rather than its name: two symbols
    // can share a qualified name across files, and a name-keyed join would
    // silently merge their tags.
    let mut verifies: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut implements: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    if let Some(graph) = graph {
        for relation in &graph.verifies {
            verifies
                .entry(relation.symbol_id.as_str())
                .or_default()
                .push(relation.trace_id.clone());
        }
        for relation in &graph.implements {
            implements
                .entry(relation.symbol_id.as_str())
                .or_default()
                .push(relation.trace_id.clone());
        }
    }
    let tidy = |mut ids: Vec<String>| {
        ids.sort();
        ids.dedup();
        ids
    };

    let symbols: Vec<SymbolRecord> = extraction
        .symbols
        .iter()
        .map(|symbol| SymbolRecord {
            path: symbol.path.clone(),
            symbol: symbol.qualified_name.clone(),
            kind: symbol.kind.as_str().to_string(),
            language: symbol.language.as_str().to_string(),
            line: symbol.line,
            leading_line: symbol.leading_line,
            end_line: symbol.end_line,
            container: symbol.container.clone(),
            id: symbol.id.clone(),
            binds_trace_ids: symbol.kind.binds_trace_ids(),
            carries_implements: symbol.kind.carries_implements(),
            trace_ids: tidy(verifies.remove(symbol.id.as_str()).unwrap_or_default()),
            implements: tidy(implements.remove(symbol.id.as_str()).unwrap_or_default()),
        })
        .collect();

    // Keyed by an owned label rather than a borrow of `symbols`: the records
    // are moved into the report below, and a borrowed key would tie the census
    // to the lifetime of the thing it summarises.
    let mut counts: BTreeMap<String, LanguageCount> = BTreeMap::new();
    for record in &symbols {
        let entry = counts
            .entry(record.language.clone())
            .or_insert_with(|| LanguageCount {
                language: record.language.clone(),
                symbols: 0,
                files: 0,
                binding_kinds: 0,
                bound: 0,
            });
        entry.symbols += 1;
        if record.binds_trace_ids {
            entry.binding_kinds += 1;
            if !record.trace_ids.is_empty() {
                entry.bound += 1;
            }
        }
    }
    for file in &extraction.files {
        if let Some(entry) = counts.get_mut(file.language.as_str()) {
            entry.files += 1;
        }
    }

    SymbolTableReport {
        symbols,
        by_language: counts.into_values().collect(),
        diagnostics: extraction
            .diagnostics
            .iter()
            .map(|d| FileDiagnostic {
                path: d.path.clone(),
                reason: d.reason.clone(),
            })
            .collect(),
        excluded_source_files: extraction.excluded_source_files,
        files: extraction.files.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::extract_file;
    use crate::traceability::SourceLanguage;

    /// A Python file whose test class holds one tagged method and one bare one.
    fn python_tree() -> SymbolExtraction {
        extract_file(
            "tests/test_thing.py",
            SourceLanguage::Python,
            "class TestThing:\n\
             \x20   @pytest.mark.trace(\"TC-001\")\n\
             \x20   def test_covered(self):\n\
             \x20       assert True\n\
             \x20   def test_bare(self):\n\
             \x20       assert True\n",
        )
    }

    #[test]
    fn tc1052_reports_the_qualified_name_the_engine_built() {
        // TC-1052
        // THE FIELD THE PORTS DISAGREED ON. Two reimplementations of
        // `python.rs` produced 386 and 5,263 lost declarations over the same
        // tree, because they qualified nested names differently — and that is
        // the part of the scanner under test. A defect in the scanner cannot be
        // sized by a reimplementation of the scanner (#309).
        let report = build(&python_tree(), None);
        let names: Vec<&str> = report.symbols.iter().map(|s| s.symbol.as_str()).collect();
        assert!(
            names.contains(&"TestThing.test_covered"),
            "expected the engine's own qualified name, got {names:?}"
        );
        // The container is reported too, so a reader can see the nesting the
        // qualification came from rather than parsing it back out of the name.
        let covered = report
            .symbols
            .iter()
            .find(|s| s.symbol == "TestThing.test_covered")
            .expect("the tagged method");
        assert_eq!(covered.container.as_deref(), Some("TestThing"));
        assert_eq!(covered.path, "tests/test_thing.py");
    }

    #[test]
    fn tc1053_kind_capability_is_reported_beside_the_symbol() {
        // TC-1053
        // The first thing to check when a row will not bind, and previously
        // only inferable from a coverage rollup two layers away. Tagging harder
        // never helps a `Container`, which is the whole of #312 — and the two
        // flags are complements, so no symbol reads as both evidence and scope.
        let report = build(&python_tree(), None);
        for symbol in &report.symbols {
            assert_ne!(
                symbol.binds_trace_ids, symbol.carries_implements,
                "{} is both or neither: the kinds are complements (CR-061)",
                symbol.symbol
            );
        }
        let class = report
            .symbols
            .iter()
            .find(|s| s.symbol == "TestThing")
            .expect("the test class");
        assert!(!class.binds_trace_ids, "a container cannot bind a trace id");
        let method = report
            .symbols
            .iter()
            .find(|s| s.symbol == "TestThing.test_covered")
            .expect("the tagged method");
        assert!(method.binds_trace_ids);
    }

    #[test]
    fn tc1054_no_graph_means_not_asked_rather_than_not_tagged() {
        // TC-1054
        // An unbound run and a repository nobody tagged produce the SAME empty
        // `trace_ids`. Telling them apart is the point of the surface, so the
        // per-language census reports `binding_kinds` — the population a rate
        // is drawn from — separately from `bound`, and `bound` stays 0 when
        // nothing was asked.
        let extraction = python_tree();
        let unasked = build(&extraction, None);
        let python = unasked
            .by_language
            .iter()
            .find(|l| l.language == "python")
            .expect("python census");
        assert_eq!(
            python.bound, 0,
            "nothing was bound because nothing was asked"
        );
        assert!(
            python.binding_kinds >= 2,
            "both test methods are of a binding kind, whatever bound: {python:?}"
        );
        // And the symbols are all present regardless — the walk is the walk.
        assert_eq!(python.symbols, unasked.symbols.len());
        assert!(unasked.symbols.iter().all(|s| s.trace_ids.is_empty()));
    }
}
