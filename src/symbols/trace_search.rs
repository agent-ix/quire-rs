//! Trace search: structural forward/inverse lookup over the symbol graph
//! (FR-077).
//!
//! Pure indexing and matching over an already-computed [`SymbolGraph`] — no
//! regex, no trace-tag grammar of its own. Every id-shaped-text decision
//! (what counts as a claim, what counts as a citation, what a "form" is)
//! stays in [`super::trace`], which is where FR-051/FR-062/FR-077 all agree
//! it must live (FR-077-CON-1). This module only answers "given a query,
//! which of the already-classified rows match it."
//!
//! **Forward**: given a trace id, every symbol that claims it
//! (`verifies`/`implements`) and every symbol that merely cites it
//! (`mentions`). **Inverse**: given a file or a symbol, the same split in the
//! other direction. Both directions return the identical [`SearchResult`]
//! shape — `claims` (`verifies`/`implements`) and `citations` as separate
//! fields, never one list with a flag (FR-077-AC-1).
//!
//! Matching is **exact**, after [`super::trace::normalized_trace_id`]'s
//! existing case/punctuation normalization. There is no FR/AC/TC hierarchy
//! walk here: `traceability.rs` states outright that this engine knows
//! nothing of that vocabulary, so an id query for `FR-047` matches only
//! symbols claiming or citing the literal id `FR-047` (FR-077-AC-6). A
//! caller wanting the wider family is a `quire-cli` `--prefix` concern, not
//! this module's.

use super::trace::{
    normalized_trace_id, ImplementsRelation, Mention, SymbolGraph, VerifiesRelation,
};
use super::SymbolExtraction;

/// What a search asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// Forward: an exact trace id (`FR-047`, `FR-047-AC-2`, `TC-707`, …).
    Id(String),
    /// Inverse: one exact symbol, addressed the same way
    /// [`super::trace::graph_records`]'s own `ref` does —
    /// `{path}#{qualified_name}`.
    Symbol {
        path: String,
        qualified_name: String,
    },
    /// Inverse: a bare, unqualified symbol name. Ambiguous when more than one
    /// symbol in the tree shares it — see [`SearchResult::ambiguous_matches`].
    SymbolName(String),
    /// Inverse: every symbol in one file.
    File(String),
}

/// The result of one [`search`] call.
///
/// `verifies`/`implements` (claims) and `citations` are separate fields on
/// purpose (FR-077-AC-1): a caller reading `result.citations` and treating it
/// as evidence has to do that deliberately, not by missing a flag on a
/// homogeneous list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub query: Query,
    /// Test/benchmark/fuzz symbols claiming the queried id(s) as evidence.
    pub verifies: Vec<VerifiesRelation>,
    /// Production symbols claiming the queried id(s) as scope. Never
    /// evidence (CR-061) — reported here for the same reason it is reported
    /// anywhere: so a reader can see it without re-deriving it.
    pub implements: Vec<ImplementsRelation>,
    /// Non-claim mentions — near-miss tags, orphaned legacy tags, and plain
    /// citations in comments/docstrings/string literals (FR-077-AC-2).
    pub citations: Vec<Mention>,
    /// `path#qualified_name` for every candidate, populated only when a
    /// [`Query::SymbolName`] matched more than one symbol. Never a silent
    /// pick — the caller must re-query with one of these (FR-077-AC-4).
    pub ambiguous_matches: Vec<String>,
    /// `false` iff `verifies`, `implements`, `citations` and
    /// `ambiguous_matches` are all empty (FR-077-AC-3): a query that matched
    /// nothing still returns this full shape rather than an empty payload.
    pub resolved: bool,
}

/// `{path}#{qualified_name}`, matching [`super::trace::graph_records`]'s own
/// `ref` convention rather than inventing a second separator.
fn symbol_ref(path: &str, qualified_name: &str) -> String {
    format!("{path}#{qualified_name}")
}

/// How much structural confidence a record's `language` earns (FR-077-AC-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// Extracted over a real tree-sitter AST — Rust only, PLAT-843.
    Structural,
    /// Extracted by the pre-PLAT-851 line/indentation-structural scanner —
    /// Python and TypeScript, and inherits that scanner's known risk classes
    /// (PLAT-163 abandoned-file, PLAT-305 `#[ignore]` ordering, PLAT-69
    /// multiline-attribute truncation, PLAT-234 formatter sensitivity).
    LineHeuristic,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Structural => "structural",
            Self::LineHeuristic => "line_heuristic",
        }
    }
}

/// The single documented mapping every relation/mention record's `language`
/// resolves confidence through (FR-077-AC-5) — one function, not a field
/// duplicated onto every reused claim/citation type, so the mapping cannot
/// drift between record kinds or go stale when a language's extractor
/// changes without this function changing too.
pub fn language_confidence(language: &str) -> Confidence {
    match language {
        "rust" => Confidence::Structural,
        _ => Confidence::LineHeuristic,
    }
}

/// The language of the symbol `verifies`/`implements` identifies by
/// `symbol_id`.
///
/// [`VerifiesRelation`]/[`ImplementsRelation`] are reused from `trace.rs`
/// unmodified (FR-077's own stated boundary) and carry no `language` field of
/// their own — [`Mention`] does, because it is new. A caller wanting
/// `language`/[`language_confidence`] for a claim record joins it back to the
/// symbol that made it, through the same `extraction` the search already
/// took; this is that join, done once rather than once per caller.
pub fn symbol_language<'a>(extraction: &'a SymbolExtraction, symbol_id: &str) -> Option<&'a str> {
    extraction
        .symbols
        .iter()
        .find(|s| s.id == symbol_id)
        .map(|s| s.language.as_str())
}

/// Answer `query` against `graph`/`extraction`.
///
/// `extraction` is needed only for [`Query::SymbolName`]'s ambiguity check —
/// `graph`'s own relations/mentions already carry `path`/`symbol` for every
/// other query shape.
pub fn search(graph: &SymbolGraph, extraction: &SymbolExtraction, query: &Query) -> SearchResult {
    match query {
        Query::Id(id) => search_id(graph, id.clone()),
        Query::Symbol {
            path,
            qualified_name,
        } => search_symbol(graph, query.clone(), path, qualified_name),
        Query::File(path) => search_file(graph, query.clone(), path),
        Query::SymbolName(name) => search_symbol_name(graph, extraction, name.clone()),
    }
}

fn finish(
    query: Query,
    verifies: Vec<VerifiesRelation>,
    implements: Vec<ImplementsRelation>,
    citations: Vec<Mention>,
    ambiguous_matches: Vec<String>,
) -> SearchResult {
    let resolved = !(verifies.is_empty()
        && implements.is_empty()
        && citations.is_empty()
        && ambiguous_matches.is_empty());
    SearchResult {
        query,
        verifies,
        implements,
        citations,
        ambiguous_matches,
        resolved,
    }
}

fn search_id(graph: &SymbolGraph, id: String) -> SearchResult {
    let target = normalized_trace_id(&id);
    let verifies: Vec<VerifiesRelation> = graph
        .verifies
        .iter()
        .filter(|v| normalized_trace_id(&v.trace_id) == target)
        .cloned()
        .collect();
    let implements: Vec<ImplementsRelation> = graph
        .implements
        .iter()
        .filter(|i| normalized_trace_id(&i.trace_id) == target)
        .cloned()
        .collect();
    let citations: Vec<Mention> = graph
        .mentions
        .iter()
        .filter(|m| normalized_trace_id(&m.trace_id) == target)
        .cloned()
        .collect();
    finish(Query::Id(id), verifies, implements, citations, Vec::new())
}

fn search_symbol(
    graph: &SymbolGraph,
    query: Query,
    path: &str,
    qualified_name: &str,
) -> SearchResult {
    let verifies: Vec<VerifiesRelation> = graph
        .verifies
        .iter()
        .filter(|v| v.path == path && v.symbol == qualified_name)
        .cloned()
        .collect();
    let implements: Vec<ImplementsRelation> = graph
        .implements
        .iter()
        .filter(|i| i.path == path && i.symbol == qualified_name)
        .cloned()
        .collect();
    let citations: Vec<Mention> = graph
        .mentions
        .iter()
        .filter(|m| m.path == path && m.symbol.as_deref() == Some(qualified_name))
        .cloned()
        .collect();
    finish(query, verifies, implements, citations, Vec::new())
}

fn search_file(graph: &SymbolGraph, query: Query, path: &str) -> SearchResult {
    let verifies: Vec<VerifiesRelation> = graph
        .verifies
        .iter()
        .filter(|v| v.path == path)
        .cloned()
        .collect();
    let implements: Vec<ImplementsRelation> = graph
        .implements
        .iter()
        .filter(|i| i.path == path)
        .cloned()
        .collect();
    // Every mention in the file, including file-scope ones owned by no
    // symbol (`symbol: None`) — a whole-file inverse query is exactly the
    // case those exist for.
    let citations: Vec<Mention> = graph
        .mentions
        .iter()
        .filter(|m| m.path == path)
        .cloned()
        .collect();
    finish(query, verifies, implements, citations, Vec::new())
}

fn search_symbol_name(
    graph: &SymbolGraph,
    extraction: &SymbolExtraction,
    name: String,
) -> SearchResult {
    let mut candidates: Vec<(&str, &str)> = extraction
        .symbols
        .iter()
        .filter(|s| s.bare_name() == name)
        .map(|s| (s.path.as_str(), s.qualified_name.as_str()))
        .collect();
    candidates.sort();
    candidates.dedup();

    match candidates.as_slice() {
        [] => finish(
            Query::SymbolName(name),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        [(path, qualified_name)] => {
            let (path, qualified_name) = (path.to_string(), qualified_name.to_string());
            search_symbol(graph, Query::SymbolName(name), &path, &qualified_name)
        }
        many => {
            let ambiguous_matches: Vec<String> = many
                .iter()
                .map(|(path, qualified_name)| symbol_ref(path, qualified_name))
                .collect();
            finish(
                Query::SymbolName(name),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                ambiguous_matches,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::extract_file;
    use crate::symbols::trace::bind;
    use crate::traceability::{SourceLanguage, TraceMarkerForm, TraceabilityModel};
    use crate::Registry;
    use ix_trace_rs::trace;

    fn fixture_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    /// The shared ISO fixture module (`tests/fixtures/traceability/iso`) with
    /// a Rust `Implements:` line-comment form added — the ISO fixture itself
    /// declares no `implements` forms (see `trace.rs`'s own
    /// `tc939_implements_reaches_the_v1_contract...` test for the identical
    /// pattern, one module reused across a dozen tests rather than a fixture
    /// per test).
    fn model_with_implements() -> TraceabilityModel {
        let mut model = Registry::load_module(&fixture_root().join("traceability").join("iso"))
            .expect("load")
            .traceability()
            .cloned()
            .expect("declared model");
        model.trace_tags.implements.push(TraceMarkerForm {
            name: "rust-implements-line".to_string(),
            language: SourceLanguage::Rust,
            pattern: r"(?m)^\s*//\s*Implements:\s*(.+)$".to_string(),
            template: None,
        });
        model
    }

    /// One file carrying, of the same id `FR-900`: a `verifies` claim (a
    /// tagged test), an `implements` claim (a marked production function),
    /// and a plain-language citation (a comment on a THIRD function that
    /// matches neither declared form — generic id-shaped text only).
    const FORWARD_FIXTURE: &str = "\
#[trace(\"FR-900\")]\n\
#[test]\n\
fn tc001_verifies() {\n\
    assert!(true);\n\
}\n\
\n\
// Implements: FR-900\n\
fn implementing_fn() {\n\
}\n\
\n\
fn citing_helper() {\n\
    // See FR-900 for background, not a tag.\n\
}\n\
";

    fn forward_graph() -> (SymbolExtraction, SymbolGraph) {
        let extraction = extract_file("src/lib.rs", SourceLanguage::Rust, FORWARD_FIXTURE);
        let graph = bind(&extraction, &model_with_implements());
        (extraction, graph)
    }

    #[trace("TC-1887")]
    #[test]
    fn tc1887_forward_id_query_separates_claims_from_citations() {
        // ACs: FR-077-AC-1
        let (extraction, graph) = forward_graph();
        let result = search(&graph, &extraction, &Query::Id("FR-900".to_string()));
        assert!(result.resolved);

        // The claim: exactly one `verifies` relation, and it is the tagged
        // test — `TestFunction`/`Benchmark`/`FuzzTarget` are the only kinds
        // `verifies` ever binds, so no other symbol in the fixture can appear
        // here regardless of what else matches the generic pattern.
        assert_eq!(result.verifies.len(), 1, "{:?}", result.verifies);
        assert_eq!(result.verifies[0].symbol, "tc001_verifies");

        // The claim: the marked production function claims it too.
        assert!(
            result
                .implements
                .iter()
                .any(|i| i.symbol == "implementing_fn" && i.trace_id == "FR-900"),
            "expected implementing_fn among implements claims: {:?}",
            result.implements
        );

        // The citation: `citing_helper`'s plain comment is not a declared
        // form on either channel, so it must surface as a citation — and it
        // must never ALSO appear as a claim, which is the structural
        // guarantee FR-077-AC-1 exists to make unmisreadable.
        assert!(
            result
                .citations
                .iter()
                .any(|m| m.symbol.as_deref() == Some("citing_helper")),
            "expected citing_helper among citations: {:?}",
            result.citations
        );
        assert!(
            !result.verifies.iter().any(|v| v.symbol == "citing_helper"),
            "a citation must never also appear as a verifies claim"
        );
        assert!(
            !result
                .implements
                .iter()
                .any(|i| i.symbol == "citing_helper"),
            "a citation must never also appear as an implements claim"
        );
        assert!(
            !result
                .citations
                .iter()
                .any(|m| m.symbol.as_deref() == Some("tc001_verifies")),
            "a bound claim must never also appear as a citation"
        );
        assert!(
            !result
                .citations
                .iter()
                .any(|m| m.symbol.as_deref() == Some("implementing_fn")),
            "a bound claim must never also appear as a citation"
        );
    }

    #[trace("TC-1889")]
    #[test]
    fn tc1889_unresolved_id_returns_full_shape_not_empty() {
        // ACs: FR-077-AC-3
        let (extraction, graph) = forward_graph();
        let result = search(
            &graph,
            &extraction,
            &Query::Id("FR-999-NOWHERE".to_string()),
        );
        assert!(!result.resolved);
        assert!(result.verifies.is_empty());
        assert!(result.implements.is_empty());
        assert!(result.citations.is_empty());
        assert!(result.ambiguous_matches.is_empty());
    }

    #[trace("TC-1893")]
    #[test]
    fn tc1893_exact_match_only_no_hierarchy_expansion() {
        // ACs: FR-077-AC-6
        // The hazard AC-6 names is WIDENING: a query for the parent `FR-900`
        // wrongly including a child `FR-900-AC-1` claim. A fixture that only
        // ever tags `FR-900` cannot exercise that — there is nothing wider to
        // wrongly include, so `.all(|v| v.trace_id == "FR-900")` passes
        // whether or not the engine would have widened. This fixture tags
        // BOTH, on two different symbols, so the property is checked by
        // exclusion of a real, present, differently-tagged claim.
        let source = "\
#[trace(\"FR-900\")]\n\
#[test]\n\
fn tc_parent() {\n\
    assert!(true);\n\
}\n\
\n\
#[trace(\"FR-900-AC-1\")]\n\
#[test]\n\
fn tc_child() {\n\
    assert!(true);\n\
}\n";
        let extraction = extract_file("src/lib.rs", SourceLanguage::Rust, source);
        let graph = bind(&extraction, &model_with_implements());

        let result = search(&graph, &extraction, &Query::Id("FR-900".to_string()));
        assert_eq!(result.verifies.len(), 1, "{:?}", result.verifies);
        assert_eq!(result.verifies[0].symbol, "tc_parent");
        assert!(
            !result.verifies.iter().any(|v| v.symbol == "tc_child"),
            "a query for the parent id FR-900 must not widen to include the child \
             FR-900-AC-1 claim: {:?}",
            result.verifies
        );

        let child_result = search(&graph, &extraction, &Query::Id("FR-900-AC-1".to_string()));
        assert_eq!(
            child_result.verifies.len(),
            1,
            "{:?}",
            child_result.verifies
        );
        assert_eq!(child_result.verifies[0].symbol, "tc_child");
    }

    // Both `helper`s are NESTED (`mod tests { ... }`, qualified_name
    // `tests::helper`) — deliberately not top-level, so a bare-name query
    // exercises the actual `bare_name()` split (the last `::`-segment) rather
    // than a shape where `qualified_name == bare name` already by accident.
    // Each carries its OWN, DIFFERENT claim, so the exact-ref query
    // (`tc1891`) has something real to prove it returns one file's claim and
    // not the other's, rather than passing on two symbols that bind nothing
    // at all.
    // Two levels of nesting — `mod` then `fn` (PLAT-845: a `fn` is now a
    // container for its own body too, the same `::` mechanism `mod` already
    // used) — so `qualified_name` carries TWO `::` separators
    // (`tests::outer::helper`). `bare_name()` must take the LAST segment via
    // `rsplit_once`, not the first: a fixture with only one `::` cannot tell
    // a correct `rsplit_once` apart from a buggy `split_once` that happens
    // to agree with it at depth 1.
    const AMBIGUOUS_FIXTURE_A: &str = "\
mod tests {\n\
    fn outer() {\n\
        #[trace(\"FR-910\")]\n\
        #[test]\n\
        fn helper() {\n\
            assert!(true);\n\
        }\n\
    }\n\
}\n";
    const AMBIGUOUS_FIXTURE_B: &str = "\
mod tests {\n\
    fn outer() {\n\
        #[trace(\"FR-911\")]\n\
        #[test]\n\
        fn helper() {\n\
            assert!(true);\n\
        }\n\
    }\n\
}\n";

    fn ambiguous_graph() -> (SymbolExtraction, SymbolGraph) {
        let mut a = extract_file("src/a.rs", SourceLanguage::Rust, AMBIGUOUS_FIXTURE_A);
        let b = extract_file("src/b.rs", SourceLanguage::Rust, AMBIGUOUS_FIXTURE_B);
        a.symbols.extend(b.symbols);
        a.files.extend(b.files);
        let graph = bind(&a, &model_with_implements());
        (a, graph)
    }

    #[trace("TC-1890")]
    #[test]
    fn tc1890_ambiguous_bare_name_lists_every_candidate() {
        // ACs: FR-077-AC-4
        let (extraction, graph) = ambiguous_graph();
        let result = search(
            &graph,
            &extraction,
            &Query::SymbolName("helper".to_string()),
        );
        assert!(result.resolved);
        let mut matches = result.ambiguous_matches.clone();
        matches.sort();
        assert_eq!(
            matches,
            vec![
                "src/a.rs#tests::outer::helper".to_string(),
                "src/b.rs#tests::outer::helper".to_string()
            ],
            "expected every candidate named by its QUALIFIED ref, none picked silently \
             (B1: a bare name must match the nested symbol's LAST `::`-segment even \
             across two levels of nesting, not fail to match at all)"
        );
        assert!(result.verifies.is_empty());
        assert!(result.implements.is_empty());
        assert!(result.citations.is_empty());
    }

    #[trace("TC-1891")]
    #[test]
    fn tc1891_exact_symbol_ref_returns_only_that_symbol() {
        // ACs: FR-077-AC-4
        let (extraction, graph) = ambiguous_graph();
        let result = search(
            &graph,
            &extraction,
            &Query::Symbol {
                path: "src/a.rs".to_string(),
                qualified_name: "tests::outer::helper".to_string(),
            },
        );
        assert!(result.ambiguous_matches.is_empty());
        // a.rs's OWN claim, and not b.rs's same-named symbol's claim — this
        // is the property the exact-ref form exists to guarantee, and
        // without it these assertions would equally pass if `search_symbol`
        // dropped its `path`/`qualified_name` filters and returned the whole
        // graph.
        assert_eq!(result.verifies.len(), 1, "{:?}", result.verifies);
        assert_eq!(result.verifies[0].trace_id, "FR-910");
        assert_eq!(result.verifies[0].path, "src/a.rs");
        assert!(
            !result.verifies.iter().any(|v| v.trace_id == "FR-911"),
            "must not return b.rs's same-named symbol's claim: {:?}",
            result.verifies
        );
    }

    #[trace("TC-1892")]
    #[test]
    fn tc1892_language_confidence_is_honest_per_language() {
        // ACs: FR-077-AC-5
        assert_eq!(language_confidence("rust"), Confidence::Structural);
        assert_eq!(language_confidence("python"), Confidence::LineHeuristic);
        assert_eq!(language_confidence("typescript"), Confidence::LineHeuristic);

        // A genuinely MIXED-language extraction — Rust and Python, each with
        // its own claim — not a Rust-only fixture asserted against hardcoded
        // string literals unrelated to what was actually extracted. Built by
        // hand rather than reusing `forward_graph()`'s single-language
        // fixture, because the property this AC states is cross-language
        // honesty, and nothing here can demonstrate that over one language.
        let rust = extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            "#[trace(\"FR-930\")]\n#[test]\nfn tc_rust_claim() {\n    assert!(true);\n}\n",
        );
        let python = extract_file(
            "tests/test_thing.py",
            SourceLanguage::Python,
            "class TestThing:\n    @pytest.mark.trace(\"FR-931\")\n    def test_thing(self):\n        assert True\n",
        );
        let mut combined = rust;
        combined.symbols.extend(python.symbols);
        combined.files.extend(python.files);
        let graph = bind(&combined, &model_with_implements());

        let rust_result = search(&graph, &combined, &Query::Id("FR-930".to_string()));
        let python_result = search(&graph, &combined, &Query::Id("FR-931".to_string()));

        // Non-emptiness asserted FIRST and separately: `.all()` over an empty
        // iterator is vacuously true, so if `find_mentions`/`bind` stopped
        // producing a claim entirely, an `.all()`-only check would still go
        // green. Neither of these may be empty for the property below to
        // mean anything.
        assert_eq!(rust_result.verifies.len(), 1, "{:?}", rust_result.verifies);
        assert_eq!(
            python_result.verifies.len(),
            1,
            "{:?}",
            python_result.verifies
        );

        // The join that actually exists: `symbol_language` reads the owning
        // symbol's real, extracted language back off `symbol_id`, and
        // `language_confidence` maps THAT — not a field on the relation
        // itself, which `VerifiesRelation` does not carry (FR-077-AC-5's own
        // wording).
        let rust_language = symbol_language(&combined, &rust_result.verifies[0].symbol_id)
            .expect("rust symbol found");
        assert_eq!(rust_language, "rust");
        assert_eq!(language_confidence(rust_language), Confidence::Structural);

        let python_language = symbol_language(&combined, &python_result.verifies[0].symbol_id)
            .expect("python symbol found");
        assert_eq!(python_language, "python");
        assert_eq!(
            language_confidence(python_language),
            Confidence::LineHeuristic
        );
    }

    #[trace("TC-1894")]
    #[test]
    fn tc1894_string_literal_content_is_not_masked_from_citations() {
        // Constraints: FR-077-CON-3
        let source = "fn holds_a_fixture() -> &'static str {\n    \"marker id FR-900 lives in this string\"\n}\n";
        let extraction = extract_file("src/lib.rs", SourceLanguage::Rust, source);
        let graph = bind(&extraction, &model_with_implements());
        assert!(
            graph
                .mentions
                .iter()
                .any(|m| m.trace_id == "FR-900" && m.symbol.as_deref() == Some("holds_a_fixture")),
            "an id inside a string literal must still surface as a citation, not be masked: {:?}",
            graph.mentions
        );
    }
}
