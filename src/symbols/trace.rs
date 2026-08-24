//! Trace binding and graph relations (FR-051).
//!
//! Binds extracted symbols to spec trace ids using **only** the module-declared
//! trace-tag grammar ([`crate::traceability::TraceTagGrammar`]) — the engine
//! carries no tag forms of its own. Framework-native markers are the canonical
//! form and are parsed statically from the source text; the textual forms the
//! `gap-analysis` workflow greps today bind as a recognized `legacy` class,
//! carry `legacy` provenance, and yield a mechanical rewrite suggestion where
//! the target marker declares a template.
//!
//! Nothing here executes, imports, or builds the extracted code
//! (FR-051-CON-1): binding reads the same text the adapters already read.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{stable_id, Symbol, SymbolExtraction};
use crate::filament::{CoreGraphEdgeRef, CoreGraphNodeRef};
use crate::traceability::{SourceLanguage, TraceabilityModel};

/// How a trace binding was authored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TraceProvenance {
    /// A framework-native marker — the canonical form.
    Canonical,
    /// A textual legacy form, read during migration only (FR-051-CON-3).
    Legacy,
}

impl TraceProvenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::Legacy => "legacy",
        }
    }
}

/// One `verifies` relation: a test symbol verifies a spec trace id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiesRelation {
    /// Identity digest of the verifying symbol.
    pub symbol_id: String,
    pub symbol: String,
    pub path: String,
    /// The referenced trace id, exactly as the declared form yields it.
    pub trace_id: String,
    pub provenance: TraceProvenance,
    /// Name of the declared form that bound it.
    pub form: String,
    /// 1-based declaration line of the verifying symbol (#210) — what
    /// `untracked_symbols` renders, and a non-identity attribute like
    /// [`Symbol::line`](super::Symbol::line) itself.
    pub line: usize,
}

/// One `implements` relation: a production symbol implements a requirement
/// (FR-062).
///
/// Deliberately a **different type** from [`VerifiesRelation`], not the same one
/// with a flag. The two answer different questions — *"would this test fail if
/// the behaviour broke"* versus *"is this the code the requirement is about"* —
/// and only the first is evidence. A shared type with a discriminator would put
/// one typo between scope and evidence.
///
/// Carries no `provenance`: there is no legacy `implements` form to migrate
/// from, and inventing the field would suggest one exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementsRelation {
    /// Identity digest of the implementing symbol.
    pub symbol_id: String,
    pub symbol: String,
    pub path: String,
    /// The requirement id this code implements.
    pub trace_id: String,
    /// Name of the declared form that bound it.
    pub form: String,
}

/// A mechanical marker-rewrite suggestion for a legacy binding (FR-051-AC-11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteSuggestion {
    pub path: String,
    pub symbol: String,
    /// 1-based line the legacy form sits on.
    pub line: usize,
    pub from_form: String,
    pub to_marker: String,
    /// The marker text to insert, rendered from the marker's template.
    pub suggestion: String,
}

/// A duplicate binding, dropped to one relation (FR-045 edge-dedup).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceDiagnostic {
    pub path: String,
    pub symbol: String,
    pub trace_id: String,
    pub reason: String,
}

/// What the binder looked at, and what bound, for one language (FR-051-AC-19,
/// CR-093).
///
/// The engine already knew both numbers — it walks the candidates to match them
/// — and reported neither, so a corpus whose tag convention matches no declared
/// pattern produced a low coverage percentage that reads exactly like missing
/// tests. Measured on `agent-ix/filament-ide-rs`: 1,292 Rust candidates, 0
/// bound, reported as `555/2389 rows backed (23%)` with no other signal.
///
/// `bound` counts **symbols**, not relations: a test carrying five ids is one
/// bound candidate. The question this answers is "could the binder read this
/// repository's convention at all", and one symbol binding many ids is no more
/// evidence of that than one binding a single id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingCensus {
    /// Stable machine label — `rust`, `python`, `typescript`.
    pub language: String,
    /// Evidence symbols the binder examined: those whose kind
    /// ([`SymbolKind::binds_trace_ids`](super::SymbolKind::binds_trace_ids))
    /// admits a trace tag at all. A container or a production function is not a
    /// candidate and never was, so counting it here would make every repository
    /// look half-unbound.
    pub candidates: usize,
    /// Candidates that minted at least one `verifies` relation.
    pub bound: usize,
    /// Every declared form consulted for this language, marker names first,
    /// then legacy — the list to check when `bound` is 0 and `candidates` is
    /// not.
    pub forms: Vec<String>,
    /// One candidate that bound nothing — a place to look (#256).
    ///
    /// The census is a count, and a count cannot be opened. `no-symbol-bound`
    /// named the LANGUAGE (`rust`) and nothing else, so a reader with 1,292
    /// unbound symbols was told which of three languages to search. This is the
    /// lowest `(path, line)` among the unbound, so it is deterministic
    /// (NFR-006) and it is a real symbol the reader can put a cursor on.
    ///
    /// `None` when every candidate bound, which is also when the diagnostic
    /// does not fire. Skipped rather than serialized as `null`, matching
    /// `specific_shaped` and `grounding`: FR-050-AC-7 byte-identity means a
    /// report from a corpus with nothing unbound must not change shape because
    /// a later engine learned to name one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unbound_example: Option<UnboundSymbol>,
}

/// Where one unbound evidence symbol is (#256).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnboundSymbol {
    /// Repo-relative, `/`-separated.
    pub path: String,
    /// 1-based line of the symbol's annotation block, not its `fn` line: a
    /// marker that failed to match is written where the annotation is, and
    /// that is the line the reader has to edit.
    pub line: usize,
    /// Qualified name, so the reader can confirm they are looking at the same
    /// symbol after the file moves.
    pub symbol: String,
}

/// The symbol graph the coverage rollup and knowledge-graph ingestion consume.
/// Every collection is deterministically ordered (NFR-006).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SymbolGraph {
    pub verifies: Vec<VerifiesRelation>,
    /// Requirement → production code (FR-062). **Never** consulted by
    /// [`Self::backed_trace_ids`]: this is scope, not evidence, and letting it
    /// back an acceptance criterion is exactly the coverage backdoor CR-061
    /// closed.
    pub implements: Vec<ImplementsRelation>,
    /// `(symbol id, symbol qualified name, file path)`.
    pub defined_in: Vec<(String, String, String)>,
    /// `(container qualified name, member qualified name, file path)`.
    pub contains: Vec<(String, String, String)>,
    pub rewrites: Vec<RewriteSuggestion>,
    pub diagnostics: Vec<TraceDiagnostic>,
    /// Per-language candidate/bound counts (FR-051-AC-19, CR-093). Ordered by
    /// language label, and present for every language the walk saw at least one
    /// evidence symbol in.
    pub binding_census: Vec<BindingCensus>,
    /// Production symbols examined for an `implements` marker — the complement
    /// of [`BindingCensus::candidates`] (FR-063-AC-4, CR-094).
    ///
    /// The denominator [`Self::implements`] is drawn from. Without it the
    /// relation count is a bare number: 16 relations reads the same whether 20
    /// production symbols were examined or 20,000.
    pub implements_candidates: usize,
    /// Production symbols that carried at least one declared `implements`
    /// marker. Counts symbols, not relations, for the same reason
    /// [`BindingCensus::bound`] does.
    pub implements_bound: usize,
    /// Evidence symbols whose assertions may never run (FR-064-AC-1, CR-100).
    ///
    /// Computed here rather than one layer up because this is where the
    /// extraction and the symbol kinds are both in hand — and because minting
    /// a fact and exposing it one release later is the CR-076/CR-080 shape this
    /// repository has now paid for four times.
    pub suspicions: Vec<crate::skeptic::Suspicion>,
    /// Source files a declared `source_exclude` glob removed from the walk,
    /// copied from [`SymbolExtraction::excluded_source_files`] so the coverage
    /// rollup — which sees only this graph — can report it (FR-050-AC-24,
    /// #215).
    pub excluded_source_files: usize,
}

impl SymbolGraph {
    /// Trace ids backed by at least one symbol.
    pub fn backed_trace_ids(&self) -> BTreeSet<&str> {
        self.verifies.iter().map(|v| v.trace_id.as_str()).collect()
    }
}

/// Bind `extraction` to trace ids per the module-declared grammar, and mint the
/// `verifies` / `defined_in` / `contains` relations.
///
/// Only **leaf evidence symbols** carry trace bindings — a test function, a
/// benchmark, or a fuzz target ([`super::SymbolKind::binds_trace_ids`]). Binding
/// containers would let a `mod tests` block inherit every marker nested inside
/// it, and binding plain functions would turn production doc comments that cite
/// an acceptance criterion into backing for it (CR-061).
///
/// If you are here because a row will not bind: check the symbol's *kind*
/// first. Tagging harder never helps, and a shell audit mints no symbol at all
/// — `language_of` reads Rust, Python and TypeScript.
pub fn bind(extraction: &SymbolExtraction, model: &TraceabilityModel) -> SymbolGraph {
    let mut graph = SymbolGraph {
        excluded_source_files: extraction.excluded_source_files,
        ..SymbolGraph::default()
    };
    // FR-051-AC-19: what was looked at, per language, and what bound. Keyed by
    // the stable language label so the census order is a property of the data
    // rather than of the walk (NFR-006).
    // candidates, bound, and the lowest-positioned unbound candidate (#256).
    let mut census: BTreeMap<&'static str, (usize, usize, Option<UnboundSymbol>)> = BTreeMap::new();

    for symbol in &extraction.symbols {
        graph.defined_in.push((
            symbol.id.clone(),
            symbol.qualified_name.clone(),
            symbol.path.clone(),
        ));
        if let Some(container) = &symbol.container {
            graph.contains.push((
                container.clone(),
                symbol.qualified_name.clone(),
                symbol.path.clone(),
            ));
        }
        let Some(source) = extraction.source_of(&symbol.path) else {
            continue;
        };
        // A production symbol carries `implements`, an evidence symbol carries
        // `verifies`, and the kinds are complements — so no symbol can be read
        // as both and a mis-declared pattern binds nothing rather than binding
        // the wrong relation.
        if symbol.kind.carries_implements() {
            // Counted around the call for the same reason the evidence census
            // is: `bind_implements` only appends, so the delta is exactly "did
            // this production symbol carry a marker".
            let before = graph.implements.len();
            bind_implements(symbol, source, model, &mut graph);
            graph.implements_candidates += 1;
            if graph.implements.len() > before {
                graph.implements_bound += 1;
            }
            continue;
        }
        if !symbol.kind.binds_trace_ids() {
            continue;
        }
        // Counted around the call rather than inside it: `bind_symbol` only
        // ever appends to `verifies`, so the length delta is exactly "did this
        // candidate bind" — and the count stays correct if the binder grows
        // another form.
        let before = graph.verifies.len();
        bind_symbol(symbol, source, model, &mut graph);
        let entry = census.entry(symbol.language.as_str()).or_default();
        entry.0 += 1;
        if graph.verifies.len() > before {
            entry.1 += 1;
        } else {
            // Keep the LOWEST (path, line), not the first walked: the walk
            // order is the scan's and a report that changes when a file is
            // renamed is a report nobody can diff (NFR-006).
            let candidate = UnboundSymbol {
                path: symbol.path.clone(),
                line: symbol.leading_line,
                symbol: symbol.qualified_name.clone(),
            };
            let better = match &entry.2 {
                None => true,
                Some(current) => (&candidate.path, candidate.line) < (&current.path, current.line),
            };
            if better {
                entry.2 = Some(candidate);
            }
        }
    }

    graph.suspicions = crate::skeptic::vacuous_property_suites(extraction);
    graph.binding_census = census
        .into_iter()
        .map(
            |(language, (candidates, bound, unbound_example))| BindingCensus {
                language: language.to_string(),
                candidates,
                bound,
                forms: declared_forms(model, language),
                unbound_example,
            },
        )
        .collect();

    graph.implements.sort_by(|a, b| {
        (&a.path, &a.symbol, &a.trace_id, &a.form).cmp(&(&b.path, &b.symbol, &b.trace_id, &b.form))
    });
    graph.implements.dedup();
    graph.defined_in.sort();
    graph.contains.sort();
    graph.rewrites.sort_by(|a, b| {
        (&a.path, a.line, &a.symbol, &a.from_form).cmp(&(&b.path, b.line, &b.symbol, &b.from_form))
    });
    graph
        .diagnostics
        .sort_by(|a, b| (&a.path, &a.symbol, &a.trace_id).cmp(&(&b.path, &b.symbol, &b.trace_id)));
    graph
}

/// Every declared form the binder consults for `language`, markers before
/// legacy forms — the order [`bind_symbol`] tries them in (FR-051-AC-19).
///
/// A legacy form declaring no `language` applies to all of them, which is the
/// same rule `bind_symbol` uses; naming it under each language is what makes
/// the list answer "which patterns had a chance at this file".
fn declared_forms(model: &TraceabilityModel, language: &str) -> Vec<String> {
    let applies = |declared: Option<SourceLanguage>| match declared {
        Some(l) => l.as_str() == language,
        None => true,
    };
    model
        .trace_tags
        .markers
        .iter()
        .filter(|m| applies(Some(m.language)))
        .map(|m| m.name.clone())
        .chain(
            model
                .trace_tags
                .legacy
                .iter()
                .filter(|l| applies(l.language))
                .map(|l| l.name.clone()),
        )
        .collect()
}

/// Implements: FR-062
/// Bind a production symbol to the requirements it implements (FR-062).
///
/// Structurally simpler than [`bind_symbol`] because there is nothing to
/// reconcile: no legacy forms, so no provenance and no rewrite suggestions, and
/// no precedence between forms. A requirement named twice by two forms yields
/// one relation after the dedup in [`bind`].
fn bind_implements(
    symbol: &Symbol,
    source: &str,
    model: &TraceabilityModel,
    graph: &mut SymbolGraph,
) {
    if model.trace_tags.implements.is_empty() {
        return;
    }
    let span = symbol.attached_source(source);
    for marker in &model.trace_tags.implements {
        if marker.language != symbol.language {
            continue;
        }
        let Some(re) = compile(&marker.pattern) else {
            continue;
        };
        for caps in re.captures_iter(&span) {
            let Some(args) = caps.get(1) else { continue };
            for trace_id in marker_ids(args.as_str()) {
                graph.implements.push(ImplementsRelation {
                    symbol_id: symbol.id.clone(),
                    symbol: symbol.qualified_name.clone(),
                    path: symbol.path.clone(),
                    trace_id,
                    form: marker.name.clone(),
                });
            }
        }
    }
}

/// Blank the *contents* of Rust string literals, preserving byte length and
/// line structure so downstream offset arithmetic is unaffected.
///
/// A symbol's attached source includes its body, so any file that carries
/// trace-shaped text inside a string literal binds ids that were never
/// authored as tags. This engine's own suite is full of them — it feeds
/// tag-shaped snippets to the extractor as fixtures, and every one of those
/// bound. (No example is spelled out here: a comment containing a real tag
/// would itself bind, which is the same defect one level up.)
///
/// Applied to the **legacy textual forms only**. Canonical markers put their
/// ids inside string literals by design (`#[trace("TC-707")]`), so masking
/// before matching them would suppress exactly the form the grammar prefers.
///
/// Comments are left intact — that is where legacy tags live. A `"` inside a
/// comment must not open a string, or an apostrophe-quote in prose would mask
/// every line that follows.
fn mask_rust_string_contents(span: &str) -> String {
    let mut out = String::with_capacity(span.len());
    let mut block_depth: u32 = 0;
    let mut raw_hashes: Option<usize> = None;
    let mut in_string = false;

    for line in span.split_inclusive('\n') {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        // A masked char becomes a space, but only when it is ASCII — a
        // multi-byte char is replaced by itself so byte offsets never shift.
        let blank = |c: char, out: &mut String| {
            if c == '\n' || c == '\r' || !c.is_ascii() {
                out.push(c);
            } else {
                out.push(' ');
            }
        };

        while i < chars.len() {
            let c = chars[i];

            if let Some(hashes) = raw_hashes {
                if c == '"' && chars[i + 1..].iter().take(hashes).all(|h| *h == '#') {
                    raw_hashes = None;
                    out.push(c);
                    for h in chars[i + 1..].iter().take(hashes) {
                        out.push(*h);
                    }
                    i += 1 + hashes;
                    continue;
                }
                blank(c, &mut out);
                i += 1;
                continue;
            }

            if in_string {
                if c == '\\' {
                    blank(c, &mut out);
                    if let Some(n) = chars.get(i + 1) {
                        blank(*n, &mut out);
                    }
                    i += 2;
                    continue;
                }
                if c == '"' {
                    in_string = false;
                    out.push(c);
                } else {
                    blank(c, &mut out);
                }
                i += 1;
                continue;
            }

            if block_depth > 0 {
                if c == '/' && chars.get(i + 1) == Some(&'*') {
                    block_depth += 1;
                    out.push_str("/*");
                    i += 2;
                    continue;
                }
                if c == '*' && chars.get(i + 1) == Some(&'/') {
                    block_depth -= 1;
                    out.push_str("*/");
                    i += 2;
                    continue;
                }
                out.push(c);
                i += 1;
                continue;
            }

            // A line comment runs to end of line and is kept verbatim: legacy
            // tags live here.
            if c == '/' && chars.get(i + 1) == Some(&'/') {
                for rest in &chars[i..] {
                    out.push(*rest);
                }
                break;
            }
            if c == '/' && chars.get(i + 1) == Some(&'*') {
                block_depth = 1;
                out.push_str("/*");
                i += 2;
                continue;
            }
            // `r"…"` / `r#"…"#` / `br#"…"#`
            if (c == 'r' || c == 'b') && !prev_is_ident(&chars, i) {
                let mut j = i;
                if chars[j] == 'b' {
                    j += 1;
                }
                if chars.get(j) == Some(&'r') {
                    let start = j + 1;
                    let hashes = chars[start..].iter().take_while(|h| **h == '#').count();
                    if chars.get(start + hashes) == Some(&'"') {
                        raw_hashes = Some(hashes);
                        for k in &chars[i..=start + hashes] {
                            out.push(*k);
                        }
                        i = start + hashes + 1;
                        continue;
                    }
                }
            }
            if c == '"' {
                in_string = true;
                out.push(c);
                i += 1;
                continue;
            }
            out.push(c);
            i += 1;
        }
    }
    out
}

/// True when the char before `i` could continue an identifier, so an `r` there
/// is part of a name (`str"` never opens a raw string) rather than a prefix.
fn prev_is_ident(chars: &[char], i: usize) -> bool {
    i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_')
}

fn bind_symbol(symbol: &Symbol, source: &str, model: &TraceabilityModel, graph: &mut SymbolGraph) {
    let span = symbol.attached_source(source);
    // Every attachment of a trace id to this symbol, in discovery order
    // (canonical markers first, then legacy forms). The id is bound once no
    // matter how many forms attached it (FR-051-AC-6).
    let mut attachments: BTreeMap<String, Vec<VerifiesRelation>> = BTreeMap::new();
    let mut record = |relation: VerifiesRelation| {
        attachments
            .entry(relation.trace_id.clone())
            .or_default()
            .push(relation);
    };

    // ── Canonical markers ──
    for marker in &model.trace_tags.markers {
        if marker.language != symbol.language {
            continue;
        }
        let Some(re) = compile(&marker.pattern) else {
            continue;
        };
        for caps in re.captures_iter(&span) {
            let Some(args) = caps.get(1) else { continue };
            for trace_id in marker_ids(args.as_str()) {
                record(VerifiesRelation {
                    symbol_id: symbol.id.clone(),
                    symbol: symbol.qualified_name.clone(),
                    path: symbol.path.clone(),
                    trace_id,
                    provenance: TraceProvenance::Canonical,
                    form: marker.name.clone(),
                    line: symbol.line,
                });
            }
        }
    }

    // ── Legacy textual forms ──
    //
    // Matched against a span whose Rust string *contents* are blanked. A
    // legacy tag is comment text; trace-shaped characters inside a string
    // literal are data the file happens to carry, and binding them invents
    // coverage nobody authored. Byte length and line breaks are preserved, so
    // the rewrite-suggestion offset arithmetic below is unaffected.
    let legacy_span = match symbol.language {
        SourceLanguage::Rust => mask_rust_string_contents(&span),
        _ => span.clone(),
    };
    for legacy in &model.trace_tags.legacy {
        if legacy.language.is_some_and(|l| l != symbol.language) {
            continue;
        }
        let Some(re) = compile(&legacy.pattern) else {
            continue;
        };
        for caps in re.captures_iter(&legacy_span) {
            let trace_ids = legacy_ids(&caps, legacy.id_format.as_deref());
            if trace_ids.is_empty() {
                continue;
            }
            for trace_id in &trace_ids {
                record(VerifiesRelation {
                    symbol_id: symbol.id.clone(),
                    symbol: symbol.qualified_name.clone(),
                    path: symbol.path.clone(),
                    trace_id: trace_id.clone(),
                    provenance: TraceProvenance::Legacy,
                    form: legacy.name.clone(),
                    line: symbol.line,
                });
            }
            // A rewrite suggestion is emitted only when the target marker
            // declares an authoring template — FR-051's "where derivable".
            // One match is one authored line, so a match carrying a list of
            // ids yields one suggestion naming all of them, never one
            // conflicting single-id rewrite per id (FR-051-AC-16).
            if let Some(marker) = legacy
                .rewrite_to
                .as_ref()
                .and_then(|name| model.trace_tags.markers.iter().find(|m| &m.name == name))
            {
                if let Some(template) = &marker.template {
                    let line = symbol.leading_line
                        + legacy_span[..caps.get(0).map_or(0, |m| m.start())]
                            .matches('\n')
                            .count();
                    let ids = trace_ids
                        .iter()
                        .map(|id| format!("\"{id}\""))
                        .collect::<Vec<_>>()
                        .join(", ");
                    graph.rewrites.push(RewriteSuggestion {
                        path: symbol.path.clone(),
                        symbol: symbol.qualified_name.clone(),
                        line,
                        from_form: legacy.name.clone(),
                        to_marker: marker.name.clone(),
                        suggestion: template.replace("{ids}", &ids),
                    });
                }
            }
        }
    }

    // One relation per trace id: a canonical marker wins over a legacy form,
    // and an id attached more than once yields exactly one diagnostic naming
    // every form that attached it (FR-045 edge-dedup convention).
    for (trace_id, candidates) in attachments {
        if candidates.len() > 1 {
            let forms: Vec<&str> = candidates.iter().map(|c| c.form.as_str()).collect();
            graph.diagnostics.push(TraceDiagnostic {
                path: symbol.path.clone(),
                symbol: symbol.qualified_name.clone(),
                trace_id: trace_id.clone(),
                reason: format!(
                    "trace id attached {} times (forms: {}); one relation minted",
                    candidates.len(),
                    forms.join(", ")
                ),
            });
        }
        // A key only exists because something was pushed under it, so the
        // `else` arm is unreachable — expressed as a match rather than an
        // `expect` so there is no panic path in library code at all.
        if let Some(winner) = candidates
            .iter()
            .find(|c| c.provenance == TraceProvenance::Canonical)
            .or_else(|| candidates.first())
        {
            graph.verifies.push(winner.clone());
        }
    }
    graph
        .verifies
        .sort_by(|a, b| (&a.path, &a.symbol, &a.trace_id).cmp(&(&b.path, &b.symbol, &b.trace_id)));
}

/// The trace ids inside a marker's argument list. Quoted arguments are the
/// authored form (`trace("TC-1", "FR-2-AC-3")`); an unquoted argument list is
/// taken whole, so a bare-identifier marker form still binds.
fn marker_ids(args: &str) -> Vec<String> {
    let quoted: Vec<String> = re_quoted()
        .captures_iter(args)
        .filter_map(|c| (1..=2).find_map(|g| c.get(g)))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if !quoted.is_empty() {
        return quoted;
    }
    args.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// The trace ids a legacy match yields (FR-051-AC-16). Capture group 1 is
/// comma-split the way [`marker_ids`] splits a marker's argument list, so a
/// form whose pattern admits a list (`// Trace: FR-1-AC-1, FR-1-AC-2`) binds
/// every id it carries rather than only the first.
///
/// A form declaring `id_format` renders the template over the captures and is
/// never split: the template's inputs are structural captures (`TC-{1}` over a
/// function name), which cannot carry a list.
fn legacy_ids(caps: &regex::Captures<'_>, id_format: Option<&str>) -> Vec<String> {
    match id_format {
        None => caps.get(1).map_or_else(Vec::new, |m| {
            m.as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }),
        Some(template) => {
            let mut out = template.to_string();
            for (idx, group) in caps.iter().enumerate().skip(1) {
                out = out.replace(&format!("{{{idx}}}"), group.map_or("", |m| m.as_str()));
            }
            vec![out]
        }
    }
}

/// Compile a module-declared pattern. Patterns are validated at module load
/// (FR-050-AC-2), so a failure here means the caller built a model in memory;
/// binding skips the form rather than panicking.
fn compile(pattern: &str) -> Option<Regex> {
    Regex::new(pattern).ok()
}

fn re_quoted() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"'([^']*)'|"([^"]*)""#).expect("quoted-arg regex"))
}

/// Caller-supplied identity for FR-045 record emission. `org`/`repo_name`
/// normalize every `ref` per FR-045-CON-4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolRecordInput {
    pub project_id: String,
    pub document_id: String,
    pub artifact_id: Option<String>,
    pub org: String,
    pub repo_name: String,
}

/// Emit the symbol graph as FR-045 graph records: symbols as node records,
/// relations as edge records, every `ref` normalized under the caller's
/// org/repo. Ordering is deterministic and ids are stable digests, so repeated
/// emission over an identical tree is byte-identical (FR-051-AC-10).
pub fn graph_records(
    extraction: &SymbolExtraction,
    graph: &SymbolGraph,
    input: &SymbolRecordInput,
) -> (Vec<CoreGraphNodeRef>, Vec<CoreGraphEdgeRef>) {
    let mut nodes: Vec<CoreGraphNodeRef> = Vec::new();
    for symbol in &extraction.symbols {
        let name = symbol_ref_name(symbol);
        let ref_ = normalize_ref(&input.org, &input.repo_name, &name);
        let mut data = Map::new();
        data.insert(
            "language".into(),
            Value::String(symbol.language.as_str().into()),
        );
        data.insert("path".into(), Value::String(symbol.path.clone()));
        data.insert(
            "qualifiedName".into(),
            Value::String(symbol.qualified_name.clone()),
        );
        data.insert("kind".into(), Value::String(symbol.kind.as_str().into()));
        data.insert("line".into(), Value::Number(symbol.line.into()));
        data.insert("symbolId".into(), Value::String(symbol.id.clone()));
        nodes.push(CoreGraphNodeRef {
            id: stable_id(&[&input.project_id, "graph-node", &ref_]),
            project_id: input.project_id.clone(),
            document_id: input.document_id.clone(),
            artifact_id: input.artifact_id.clone(),
            object_type: "source_symbol".to_string(),
            name,
            ref_,
            data_json: json_string(&Value::Object(data)),
            updated_at: None,
        });
    }

    let mut edges: Vec<CoreGraphEdgeRef> = Vec::new();
    let mut push_edge =
        |source: String, target: String, edge_type: &str, data: Map<String, Value>| {
            let source_ref = normalize_ref(&input.org, &input.repo_name, &source);
            let target_ref = normalize_ref(&input.org, &input.repo_name, &target);
            edges.push(CoreGraphEdgeRef {
                id: stable_id(&[
                    &input.project_id,
                    "graph-edge",
                    &source_ref,
                    &target_ref,
                    edge_type,
                ]),
                project_id: input.project_id.clone(),
                source_ref,
                target_ref,
                edge_type: edge_type.to_string(),
                data_json: json_string(&Value::Object(data)),
                updated_at: None,
            });
        };

    for relation in &graph.verifies {
        let mut data = Map::new();
        data.insert(
            "provenance".into(),
            Value::String(relation.provenance.as_str().into()),
        );
        data.insert("form".into(), Value::String(relation.form.clone()));
        push_edge(
            format!("{}#{}", relation.path, relation.symbol),
            relation.trace_id.clone(),
            "verifies",
            data,
        );
    }
    for (_, symbol, path) in &graph.defined_in {
        push_edge(
            format!("{path}#{symbol}"),
            path.clone(),
            "defined_in",
            Map::new(),
        );
    }
    for (container, member, path) in &graph.contains {
        push_edge(
            format!("{path}#{container}"),
            format!("{path}#{member}"),
            "contains",
            Map::new(),
        );
    }
    (nodes, edges)
}

/// The `ref` name of a symbol: `<path>#<qualified name>`, which is unique per
/// tree and carries no line or formatting information.
fn symbol_ref_name(symbol: &Symbol) -> String {
    format!("{}#{}", symbol.path, symbol.qualified_name)
}

fn normalize_ref(org: &str, repo_name: &str, value: &str) -> String {
    if value.starts_with("ix://") {
        value.to_string()
    } else {
        format!("ix://{org}/{repo_name}/{value}")
    }
}

fn json_string(value: &Value) -> String {
    serde_json::to_string(value).expect("serializing serde_json::Value cannot fail")
}

/// The language a symbol was extracted from, exposed for consumers that group
/// relations per language.
pub fn language_of(symbol: &Symbol) -> SourceLanguage {
    symbol.language
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::extract_tree;
    use crate::Registry;
    use ix_trace_rs::trace;

    fn fixture_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    fn iso_model() -> TraceabilityModel {
        Registry::load_module(&fixture_root().join("traceability").join("iso"))
            .expect("load")
            .traceability()
            .cloned()
            .expect("declared model")
    }

    fn alt_model() -> TraceabilityModel {
        Registry::load_module(&fixture_root().join("traceability").join("alt"))
            .expect("load")
            .traceability()
            .cloned()
            .expect("declared model")
    }

    fn symbols() -> SymbolExtraction {
        extract_tree(&fixture_root().join("symbols"))
    }

    fn relations_for<'g>(graph: &'g SymbolGraph, symbol: &str) -> Vec<&'g VerifiesRelation> {
        graph
            .verifies
            .iter()
            .filter(|v| v.symbol == symbol)
            .collect()
    }

    #[trace("TC-744", "FR-051-AC-4")]
    // each canonical marker form binds statically — one
    // `verifies` relation per attached trace id, no code executed.
    #[test]
    fn tc744_canonical_markers_bind_statically() {
        let graph = bind(&symbols(), &iso_model());

        // Rust `#[trace("TC-741")]`.
        let rust = relations_for(&graph, "tests::tc741_extracts");
        assert!(rust
            .iter()
            .any(|r| r.trace_id == "TC-741" && r.form == "rust-trace-attribute"));

        // Python `@pytest.mark.trace("TC-741", "FR-051-AC-1")` — one relation
        // per attached id.
        let py = relations_for(&graph, "test_parses_config");
        let mut ids: Vec<&str> = py.iter().map(|r| r.trace_id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["FR-051-AC-1", "TC-741"]);
        assert!(py
            .iter()
            .all(|r| r.provenance == TraceProvenance::Canonical));

        // A method marker binds to the method, not to its class.
        assert!(relations_for(&graph, "TestService.test_rejects_empty")
            .iter()
            .any(|r| r.trace_id == "TC-743"));
        assert!(relations_for(&graph, "TestService").is_empty());
    }

    #[trace("TC-745", "FR-051-AC-5")]
    // marker forms are module data — a different
    // declaration binds by its own forms, and no declared forms mints nothing.
    #[test]
    fn tc745_forms_are_module_data() {
        let extraction = crate::symbols::extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            "#[covers(\"R-1-C-2\")]\n#[test]\nfn checks_it() {\n    assert!(true);\n}\n",
        );
        // The alt module declares `#[covers(...)]`, not `#[trace(...)]`.
        let alt = bind(&extraction, &alt_model());
        assert_eq!(alt.verifies.len(), 1);
        assert_eq!(alt.verifies[0].trace_id, "R-1-C-2");
        assert_eq!(alt.verifies[0].form, "alt-marker");

        // The ISO declaration knows nothing of `#[covers(...)]`.
        assert!(bind(&extraction, &iso_model()).verifies.is_empty());

        // No declared forms at all → zero `verifies` relations, while the
        // structural relations still mint.
        let none = bind(&extraction, &TraceabilityModel::default());
        assert!(none.verifies.is_empty());
        assert!(!none.defined_in.is_empty());
    }

    #[trace("TC-746", "FR-051-AC-6")]
    // a trace id attached more than once to one symbol
    // mints one relation and one diagnostic.
    #[test]
    fn tc746_duplicate_binding_is_deduped_with_diagnostic() {
        let extraction = crate::symbols::extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            "#[trace(\"TC-900\")]\n#[trace(\"TC-900\")]\n#[test]\nfn tc900_thing() {\n    let _ = 1;\n}\n",
        );
        let graph = bind(&extraction, &iso_model());
        let dupes: Vec<&VerifiesRelation> = graph
            .verifies
            .iter()
            .filter(|v| v.trace_id == "TC-900")
            .collect();
        assert_eq!(dupes.len(), 1, "one relation only");
        assert_eq!(
            graph
                .diagnostics
                .iter()
                .filter(|d| d.trace_id == "TC-900")
                .count(),
            1
        );
        // Marker plus legacy test-name form for the same id also dedups, and
        // the canonical provenance wins.
        assert_eq!(dupes[0].provenance, TraceProvenance::Canonical);
    }

    #[trace("TC-828", "FR-051-AC-17")]
    // a benchmark and a fuzz target are leaf (CR-061)
    // evidence and bind; a container and a plain production function do not.
    //
    // The two exclusions are for two different reasons and both matter. A
    // container would let a `mod tests` block inherit every marker nested
    // inside it — the original FR-051 reason. A plain function is production
    // code whose doc comments routinely cite the acceptance criteria they
    // implement, so binding those would manufacture backing out of prose.
    #[test]
    fn tc828_benchmarks_and_fuzz_targets_bind_but_production_code_does_not() {
        let bench = crate::symbols::extract_file(
            "benches/parse.rs",
            SourceLanguage::Rust,
            concat!(
                "/// TC-577, NFR-002-AC-4: the validate-document bench.\n",
                "fn bench_validate(c: &mut Criterion) {\n",
                "    let _ = c;\n",
                "}\n",
                "criterion_group!(benches, bench_validate);\n",
            ),
        );
        let graph = bind(&bench, &iso_model());
        let mut ids: Vec<&str> = graph.verifies.iter().map(|v| v.trace_id.as_str()).collect();
        ids.sort();
        assert_eq!(
            ids,
            vec!["NFR-002-AC-4", "TC-577"],
            "a registered bench binds every id on the line"
        );

        // The tag sits at the top of the file, separated from the invocation by
        // `use` statements and a helper — so it binds only because the fuzz
        // target's span is the whole file. `leading_block` cannot reach across
        // non-comment lines.
        let fuzz = crate::symbols::extract_file(
            "fuzz/fuzz_targets/f.rs",
            SourceLanguage::Rust,
            concat!(
                "#![no_main]\n",
                "// TC-579: must never panic on arbitrary input.\n",
                "\n",
                "use libfuzzer_sys::fuzz_target;\n",
                "\n",
                "fn registry() -> usize {\n",
                "    1\n",
                "}\n",
                "\n",
                "fuzz_target!(|data: &[u8]| {\n",
                "    let _ = data;\n",
                "});\n",
            ),
        );
        let graph = bind(&fuzz, &iso_model());
        assert!(
            graph.verifies.iter().any(|v| v.trace_id == "TC-579"),
            "the file is the target's annotation block: {:?}",
            graph.verifies
        );

        // Production code citing an AC in a doc comment backs nothing, and
        // neither does the module that contains it.
        let production = crate::symbols::extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            concat!(
                "// TC-741: this module implements it.\n",
                "mod thing {\n",
                "    /// FR-051-AC-1: resolves the archetype.\n",
                "    pub fn resolve() -> usize {\n",
                "        1\n",
                "    }\n",
                "}\n",
            ),
        );
        assert!(
            bind(&production, &iso_model()).verifies.is_empty(),
            "prose in production code is not evidence"
        );
    }

    #[trace("TC-1042", "FR-051-AC-21")]
    // a name chain that NAMES a suite is a suite,
    // wherever the suite word sits in it (CR-121).
    //
    // `test.describe(` matched `TEST_NAMES` first, because the `.modifier`
    // window AC-18 opens for `it.each([…])(…)` swallows `.describe` as an
    // ordinary modifier — so one construct had two spellings with OPPOSITE
    // classifications. The bare spelling minted a `Container` binding nothing;
    // the Playwright spelling minted a `TestFunction` that bound its header tag
    // and entered the census, which is the negative AC-21 asserts.
    //
    // 120 such headers exist across two corpus repositories and 79 carry an id
    // in their title, so `spec-artifacts-process#68` declaring a TypeScript
    // test-name form would have bound all 79 as evidence on the spelling the
    // spec calls grouping. That is why this landed BEFORE sap#68 rather than
    // after it (#322).
    #[test]
    fn tc1042_a_chain_that_names_a_suite_is_a_suite() {
        let extraction = crate::symbols::extract_file(
            "src/coverage.test.ts",
            SourceLanguage::Typescript,
            concat!(
                "// TC-001: FR-001-AC-1 — Playwright spelling, on the SUITE header.\n",
                "test.describe(\"warning default\", () => {\n",
                "  test(\"defaults every finding to warning\", () => {});\n",
                "});\n",
                "\n",
                "// TC-002: FR-001-AC-2 — bare spelling, on the SUITE header.\n",
                "describe(\"plain suite\", () => {\n",
                "  it(\"names the declaration on every finding\", () => {});\n",
                "});\n",
            ),
        );

        let kind = |name: &str| {
            extraction
                .symbols
                .iter()
                .find(|s| s.qualified_name == name)
                .unwrap_or_else(|| panic!("{name} registers"))
                .kind
        };
        // One construct, two spellings, ONE classification.
        assert_eq!(
            kind("warning default"),
            crate::symbols::SymbolKind::Container
        );
        assert_eq!(kind("plain suite"), crate::symbols::SymbolKind::Container);
        // And the tests inside both still register as tests.
        assert_eq!(
            kind("defaults every finding to warning"),
            crate::symbols::SymbolKind::TestFunction
        );

        // Neither header tag binds, which is what AC-21 asserts and what the
        // Playwright spelling used to contradict.
        let graph = bind(&extraction, &iso_model());
        let bound: Vec<&str> = graph.verifies.iter().map(|v| v.trace_id.as_str()).collect();
        assert!(
            !bound.contains(&"TC-001") && !bound.contains(&"TC-002"),
            "a tag on a suite header binds nothing, in EITHER spelling: {bound:?}"
        );
    }

    #[trace("TC-1040", "FR-051-AC-21")]
    // a trace tag on a SUITE HEADER binds nothing, (CR-119)
    // and the suite is not a binding candidate.
    //
    // This is the semantics `agent-ix/quire-rs#273` had to declare rather than
    // let fall out of a kind change, and it is declared as **unchanged**: a
    // suite groups evidence and is not evidence, so widening `verifies` to
    // reach it would widen it to every `SymbolKind::Container` — including the
    // file's own module, whose span is line 1 to EOF. Every column-0 comment id
    // in every TypeScript and Python test file would then back a row, which is
    // the CR-061 prohibition arrived at from the other side.
    //
    // What #273 changed is that the tag now lands on the SUITE rather than on
    // the module: a locus a reader can act on. Where the tag then goes — today,
    // nowhere at all — is `agent-ix/quire-rs#312`, and this test pins the
    // "nowhere" so that ticket cannot land without moving it.
    #[test]
    fn tc1040_a_tag_on_a_suite_header_binds_nothing_and_is_not_a_candidate() {
        let extraction = crate::symbols::extract_file(
            "src/coverage.test.ts",
            SourceLanguage::Typescript,
            concat!(
                "// TC-001: FR-001-AC-1 — on the BLOCK header.\n",
                "describe(\"warning default\", () => {\n",
                "  it(\"defaults every finding to warning\", () => {\n",
                "    expect(1 + 1).toBe(2);\n",
                "  });\n",
                "});\n",
                "\n",
                "// TC-002: FR-001-AC-2 — the same form, on the `it`.\n",
                "it(\"names the declaration on every finding\", () => {\n",
                "  expect(2 + 2).toBe(4);\n",
                "});\n",
            ),
        );
        let suite = extraction
            .symbols
            .iter()
            .find(|s| s.qualified_name == "warning default")
            .expect("the suite is a symbol at all — that is what #273 landed");
        assert_eq!(suite.kind, crate::symbols::SymbolKind::Container);

        let graph = bind(&extraction, &iso_model());

        // The tag on the header reaches NEITHER channel. `verifies` refuses it
        // by kind; `untracked_symbols` is built one layer up from
        // `graph.verifies`, so a tag that never bound cannot appear there
        // either.
        assert!(
            relations_for(&graph, "warning default").is_empty(),
            "a suite is not evidence: {:?}",
            graph.verifies,
        );
        let ids: Vec<&str> = graph.verifies.iter().map(|v| v.trace_id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["TC-002"],
            "only the tag on the `it` binds — TC-001 is nowhere",
        );

        // And the suite is missing from the DENOMINATOR rather than counted as
        // an unbound candidate, which is why the census reads healthy while
        // TC-001 comes back unbacked. Asserted here so #312 has a pinned
        // "before" to move.
        let census = graph
            .binding_census
            .iter()
            .find(|c| c.language == "typescript")
            .expect("a census for the language the walk saw");
        assert_eq!(
            (census.candidates, census.bound),
            (2, 1),
            "two `it` registrations are the candidates; the suite is not one",
        );

        // It is counted as a production symbol instead — the branch a
        // `Container` actually takes at `carries_implements()`. That is the
        // double-count #312 has to settle before a suite could enter both.
        assert_eq!(
            graph.implements_candidates, 2,
            "the file's module container and the suite container — it read 1 \
             before #273, and the suite is the one that was added",
        );
        assert_eq!(graph.implements_bound, 0);
    }

    #[trace("TC-753", "FR-051-AC-11")]
    // legacy textual forms still bind, carry `legacy`
    // provenance, and yield a mechanical rewrite suggestion where derivable.
    #[test]
    fn tc753_legacy_forms_bind_with_provenance_and_rewrite() {
        let extraction = crate::symbols::extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            "// Trace: FR-051-AC-11\n#[test]\nfn tc753_legacy() {\n    // TC-800\n    let _ = 1;\n}\n",
        );
        let graph = bind(&extraction, &iso_model());
        let by_id = |id: &str| {
            graph
                .verifies
                .iter()
                .find(|v| v.trace_id == id)
                .unwrap_or_else(|| panic!("no relation for {id}"))
        };
        // `Trace:` line, line-comment id, and the test name that carries an id.
        assert_eq!(by_id("FR-051-AC-11").provenance, TraceProvenance::Legacy);
        assert_eq!(by_id("TC-800").provenance, TraceProvenance::Legacy);
        assert_eq!(by_id("TC-753").form, "test-name-id");

        // A rewrite suggestion is derivable for the forms whose target marker
        // declares a template.
        let suggestion = graph
            .rewrites
            .iter()
            .find(|r| r.from_form == "trace-line")
            .expect("rewrite suggestion");
        assert_eq!(suggestion.to_marker, "rust-trace-attribute");
        assert_eq!(suggestion.suggestion, "#[trace(\"FR-051-AC-11\")]");
        assert!(suggestion.line >= 1);
    }

    #[trace("TC-894", "FR-051-AC-6")]
    // a legacy tag inside a Rust string literal is data,
    // not a marker. This engine's own suite feeds tag-shaped snippets to the
    // extractor as fixtures, and before the mask every one of them bound and
    // showed up as an untracked symbol. The ids below sit outside any declared
    // range so this test cannot itself mint coverage.
    #[test]
    fn tc894_a_trace_tag_inside_a_string_literal_does_not_bind() {
        let extraction = crate::symbols::extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            "#[test]\n\
             fn holds_a_fixture() {\n\
             \x20   let fixture = \"// TC-901, FR-901-AC-1: prose\";\n\
             \x20   let _ = fixture;\n\
             }\n",
        );
        let graph = bind(&extraction, &iso_model());
        let ids: Vec<&str> = graph.verifies.iter().map(|v| v.trace_id.as_str()).collect();
        assert!(
            !ids.contains(&"TC-901") && !ids.contains(&"FR-901-AC-1"),
            "ids inside a string literal must not bind; got {ids:?}"
        );
    }

    #[trace("TC-895", "FR-051-AC-6")]
    // the mask must not reach the canonical marker, whose
    // ids are string literals by design. Masking before matching it would
    // suppress exactly the form the grammar prefers.
    #[test]
    fn tc895_the_canonical_marker_still_binds_its_string_arguments() {
        let extraction = crate::symbols::extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            "#[trace(\"TC-902\", \"FR-902-AC-1\")]\n\
             #[test]\n\
             fn carries_a_canonical_marker() {\n\
             \x20   let _ = 1;\n\
             }\n",
        );
        let graph = bind(&extraction, &iso_model());
        let ids: Vec<&str> = graph.verifies.iter().map(|v| v.trace_id.as_str()).collect();
        assert!(
            ids.contains(&"TC-902") && ids.contains(&"FR-902-AC-1"),
            "the canonical marker's string arguments must still bind; got {ids:?}"
        );
    }

    #[trace("TC-806", "FR-051-AC-16")]
    // a legacy form yields every id its match carries,
    // and a form declaring `id_format` is unchanged.
    #[test]
    fn tc806_legacy_comma_list_binds_every_id() {
        let extraction = crate::symbols::extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            "// Trace: FR-001-AC-1, FR-001-AC-2, FR-001-AC-4\n\
             #[test]\n\
             fn tc806_legacy_list() {\n    // TC-033, TC-034\n    let _ = 1;\n}\n",
        );
        let graph = bind(&extraction, &iso_model());
        let ids: Vec<&str> = graph.verifies.iter().map(|v| v.trace_id.as_str()).collect();

        // Every id on both lines binds, not just the first of each.
        assert_eq!(
            ids,
            vec![
                "FR-001-AC-1",
                "FR-001-AC-2",
                "FR-001-AC-4",
                "TC-033",
                "TC-034",
                "TC-806",
            ]
        );
        assert!(graph
            .verifies
            .iter()
            .filter(|v| v.trace_id != "TC-806")
            .all(|v| v.provenance == TraceProvenance::Legacy));

        // The widened patterns must not re-match a trailing id on its own: one
        // relation per id, so no dedup diagnostic fires.
        assert!(
            graph.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            graph.diagnostics
        );

        // One authored line is one rewrite suggestion naming all of its ids —
        // never one conflicting single-id rewrite per id.
        let listed: Vec<&RewriteSuggestion> = graph
            .rewrites
            .iter()
            .filter(|r| r.from_form == "trace-line")
            .collect();
        assert_eq!(listed.len(), 1);
        assert_eq!(
            listed[0].suggestion,
            "#[trace(\"FR-001-AC-1\", \"FR-001-AC-2\", \"FR-001-AC-4\")]"
        );

        // `test-name-id` declares `id_format`, so it renders one id as before.
        let by_name: Vec<&VerifiesRelation> = graph
            .verifies
            .iter()
            .filter(|v| v.form == "test-name-id")
            .collect();
        assert_eq!(by_name.len(), 1);
        assert_eq!(by_name[0].trace_id, "TC-806");
    }

    #[trace("TC-806", "FR-051-AC-16")]
    // irregular spacing and a trailing comma yield the
    // ids only — never an empty id.
    #[test]
    fn tc806_legacy_list_tolerates_irregular_separators() {
        let extraction = crate::symbols::extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            "// Trace: FR-002-AC-1,  FR-002-AC-2 ,\n#[test]\nfn checks_it() {\n    let _ = 1;\n}\n",
        );
        let graph = bind(&extraction, &iso_model());
        let ids: Vec<&str> = graph.verifies.iter().map(|v| v.trace_id.as_str()).collect();
        assert_eq!(ids, vec!["FR-002-AC-1", "FR-002-AC-2"]);
        assert!(graph.verifies.iter().all(|v| !v.trace_id.is_empty()));
    }

    #[trace("TC-748", "FR-051-AC-8")]
    // `defined_in` links every symbol to its file and
    // `contains` links containers to members, deterministically ordered.
    #[test]
    fn tc748_structural_relations() {
        let extraction = symbols();
        let graph = bind(&extraction, &iso_model());

        assert_eq!(graph.defined_in.len(), extraction.symbols.len());
        for symbol in &extraction.symbols {
            assert!(graph.defined_in.iter().any(|(id, name, path)| {
                id == &symbol.id && name == &symbol.qualified_name && path == &symbol.path
            }));
        }
        assert!(graph.contains.iter().any(|(container, member, path)| {
            container == "tests" && member == "tests::tc741_extracts" && path == "rust/lib.rs"
        }));

        let mut sorted = graph.defined_in.clone();
        sorted.sort();
        assert_eq!(graph.defined_in, sorted);
        let mut sorted_contains = graph.contains.clone();
        sorted_contains.sort();
        assert_eq!(graph.contains, sorted_contains);
    }

    fn record_input() -> SymbolRecordInput {
        SymbolRecordInput {
            project_id: "project-1".into(),
            document_id: "doc-1".into(),
            artifact_id: Some("artifact-1".into()),
            org: "agent-ix".into(),
            repo_name: "quire-rs".into(),
        }
    }

    #[trace("TC-747", "FR-051-AC-7")]
    // emitted records match the FR-045 graph-record
    // shapes with normalized `ref` values.
    #[test]
    fn tc747_records_match_fr045_shapes() {
        let extraction = symbols();
        let graph = bind(&extraction, &iso_model());
        let (nodes, edges) = graph_records(&extraction, &graph, &record_input());

        assert!(!nodes.is_empty() && !edges.is_empty());
        let node = nodes
            .iter()
            .find(|n| n.name == "rust/lib.rs#tests::tc741_extracts")
            .expect("node for the traced test");
        assert_eq!(node.project_id, "project-1");
        assert_eq!(node.document_id, "doc-1");
        assert_eq!(node.artifact_id.as_deref(), Some("artifact-1"));
        assert_eq!(node.object_type, "source_symbol");
        assert_eq!(
            node.ref_,
            "ix://agent-ix/quire-rs/rust/lib.rs#tests::tc741_extracts"
        );
        assert!(node.updated_at.is_none());
        let data: Value = serde_json::from_str(&node.data_json).expect("data json");
        assert_eq!(data["kind"], "test_function");
        assert_eq!(data["language"], "rust");

        // Serialized keys are the FR-045 camelCase record shape.
        let json = serde_json::to_value(node).expect("serialize");
        for key in [
            "id",
            "projectId",
            "documentId",
            "artifactId",
            "objectType",
            "name",
            "ref",
            "dataJson",
        ] {
            assert!(json.get(key).is_some(), "node record is missing {key}");
        }

        let verifies = edges
            .iter()
            .find(|e| e.edge_type == "verifies")
            .expect("a verifies edge");
        assert!(verifies.source_ref.starts_with("ix://agent-ix/quire-rs/"));
        assert!(verifies.target_ref.starts_with("ix://agent-ix/quire-rs/"));
        let edge_json = serde_json::to_value(verifies).expect("serialize");
        for key in [
            "id",
            "projectId",
            "sourceRef",
            "targetRef",
            "edgeType",
            "dataJson",
        ] {
            assert!(edge_json.get(key).is_some(), "edge record is missing {key}");
        }
        assert!(edges.iter().any(|e| e.edge_type == "defined_in"));
        assert!(edges.iter().any(|e| e.edge_type == "contains"));
    }

    #[trace("TC-750", "FR-051-AC-10")]
    // repeated extraction + binding over an (Property)
    // identical tree emits byte-identical JSON and identical record ids.
    #[test]
    fn tc750_repeated_extraction_is_byte_identical() {
        let model = iso_model();
        let render = || {
            let extraction = symbols();
            let graph = bind(&extraction, &model);
            let (nodes, edges) = graph_records(&extraction, &graph, &record_input());
            serde_json::to_string(&(nodes, edges)).expect("serialize")
        };
        let first = render();
        for _ in 0..8 {
            assert_eq!(first, render(), "records must be byte-identical");
        }
    }

    /// Every declared form rewritten to a pattern nothing can match, so the
    /// binder walks the same candidates and binds none — the shape
    /// `agent-ix/filament-ide-rs` was in.
    fn unmatchable(mut model: TraceabilityModel) -> TraceabilityModel {
        for marker in &mut model.trace_tags.markers {
            marker.pattern = "ZZZ_NO_SUCH_MARKER_FORM".to_string();
        }
        for legacy in &mut model.trace_tags.legacy {
            legacy.pattern = "ZZZ_NO_SUCH_LEGACY_FORM".to_string();
        }
        model
    }

    #[trace("TC-982", "FR-051-AC-19")]
    // the binder reports what it looked at and what
    // bound, per language, so a corpus whose convention matches no declared
    // pattern is distinguishable from one with no tests.
    #[test]
    fn tc982_binding_census_counts_candidates_and_bound_per_language() {
        let extraction = symbols();
        let bound = bind(&extraction, &iso_model());

        assert!(
            !bound.binding_census.is_empty(),
            "a tree with evidence symbols reports a census"
        );

        // Ordered by the stable language label, so the census is a property of
        // the data rather than of the walk (NFR-006).
        let languages: Vec<&str> = bound
            .binding_census
            .iter()
            .map(|c| c.language.as_str())
            .collect();
        let mut sorted = languages.clone();
        sorted.sort_unstable();
        assert_eq!(languages, sorted, "census order is deterministic");

        // `candidates` counts evidence symbols and nothing else: a container or
        // a production function was never eligible, so counting it would make
        // every repository look half-unbound.
        for entry in &bound.binding_census {
            let expected = extraction
                .symbols
                .iter()
                .filter(|s| s.language.as_str() == entry.language && s.kind.binds_trace_ids())
                .count();
            assert_eq!(
                entry.candidates, expected,
                "{} candidates count evidence symbols",
                entry.language
            );
            assert!(entry.bound <= entry.candidates);
            assert!(
                !entry.forms.is_empty(),
                "{} names the forms that had a chance",
                entry.language
            );
        }

        // `bound` counts symbols, not relations: the Python fixture binds two
        // ids from one marker and must still count once.
        for entry in &bound.binding_census {
            let relations = bound
                .verifies
                .iter()
                .filter(|v| {
                    extraction
                        .symbols
                        .iter()
                        .any(|s| s.id == v.symbol_id && s.language.as_str() == entry.language)
                })
                .count();
            assert!(
                entry.bound <= relations,
                "{}: {} bound symbols cannot exceed {} relations",
                entry.language,
                entry.bound,
                relations
            );
        }

        let rust = bound
            .binding_census
            .iter()
            .find(|c| c.language == "rust")
            .expect("the fixture tree has rust evidence symbols");
        assert!(rust.bound > 0, "the ISO model reads the fixture's tags");

        // The same tree, the same candidates, every declared pattern rewritten
        // to match nothing: the counts separate "no tests" from "cannot read
        // the convention", which is the whole point.
        let blind = bind(&extraction, &unmatchable(iso_model()));
        assert_eq!(
            blind.binding_census.len(),
            bound.binding_census.len(),
            "the same languages are reported either way"
        );
        for (before, after) in bound.binding_census.iter().zip(&blind.binding_census) {
            assert_eq!(before.language, after.language);
            assert_eq!(
                before.candidates, after.candidates,
                "{}: the candidate walk does not depend on the patterns",
                before.language
            );
            assert_eq!(after.bound, 0, "{}: nothing binds", after.language);
        }
        assert!(blind.verifies.is_empty());
    }
}
