//! Trace binding and graph relations (FR-051).
//!
//! Binds extracted symbols to spec trace ids using **only** the module-declared
//! trace-tag grammar ([`crate::traceability::TraceTagGrammar`]) — the engine
//! carries no tag forms of its own. Only framework-native markers bind, and they
//! are parsed statically from the source text (CR-215, FR-051-CON-3). An id
//! written anywhere else — a comment, a docstring, a test's own name — binds
//! nothing; the one advisory over comments and docstrings
//! ([`outside_marker_ids`]) reports it so a reader is told what to write instead.
//!
//! Nothing here executes, imports, or builds the extracted code
//! (FR-051-CON-1): binding reads the same text the adapters already read.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{stable_id, Symbol, SymbolExtraction, SymbolKind};
use crate::filament::{CoreGraphEdgeRef, CoreGraphNodeRef};
use crate::traceability::{SourceLanguage, TraceabilityModel};

/// One `verifies` relation: a test symbol verifies a spec trace id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiesRelation {
    /// Identity of the verifying symbol.
    pub symbol_id: String,
    pub symbol: String,
    pub path: String,
    /// The referenced trace id, exactly as the declared form yields it.
    pub trace_id: String,
    /// Name of the declared form that bound it.
    pub form: String,
    /// 1-based declaration line of the verifying symbol (#210) — what
    /// `untracked_symbols` renders, and a non-identity attribute like
    /// [`Symbol::line`](super::Symbol::line) itself.
    pub line: usize,
    /// 1-based UTF-8 byte column of the verifying symbol's own declaration
    /// start (CR-187, FR-051-AC-23), copied straight from the binding
    /// [`Symbol::column`].
    pub column: usize,
    /// The verifying symbol's own kind.
    pub kind: SymbolKind,
    /// Whether the verifying symbol is ignored (CR-187, FR-051-AC-27),
    /// copied straight from the binding [`Symbol::ignored`].
    pub ignored: bool,
}

/// One `implements` relation: a production symbol implements a requirement
/// (FR-062).
///
/// Deliberately a **different type** from [`VerifiesRelation`], not the same one
/// with a flag. The two answer different questions — *"would this test fail if
/// the behaviour broke"* versus *"is this the code the requirement is about"* —
/// and only the first is evidence. A shared type with a discriminator would put
/// one typo between scope and evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementsRelation {
    /// Identity of the implementing symbol.
    pub symbol_id: String,
    pub symbol: String,
    pub path: String,
    /// The requirement id this code implements.
    pub trace_id: String,
    /// Name of the declared form that bound it.
    pub form: String,
}

/// One textual `implements` use in Rust: a binding made by a Rust implements
/// form that declares no `syntax` (FR-050-AC-62).
///
/// One record per authored occurrence, carrying every id it binds, so a
/// two-id line yields one finding rather than one per id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyImplementsUse {
    pub path: String,
    pub symbol: String,
    /// 1-based line the textual `implements` use sits on.
    pub line: usize,
    /// The ids this occurrence binds, in authored order.
    pub trace_ids: Vec<String>,
    /// `leading_line` of the symbol that read it; an enclosing container reads
    /// the same text, and the greatest value is the innermost owner.
    pub owner_leading_line: usize,
}

/// A guarded form matched only part of an actual attribute candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialAttributeMatch {
    /// Module-declared form name.
    pub form: String,
    /// Repo-relative source path.
    pub path: String,
    /// Qualified owning production declaration.
    pub symbol: String,
    /// Actual 1-based start line of the attribute.
    pub line: usize,
    /// Complete verbatim candidate text.
    pub attribute: String,
}

/// A `typescript_wrapper` form matched the callee call of a curried call that
/// no wrapper chain attaches to a declarator, so it binds nothing (FR-062).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnattachedSyntax {
    /// Module-declared form name.
    pub form: String,
    /// Repo-relative source path.
    pub path: String,
    /// Actual 1-based start line of the candidate.
    pub line: usize,
    /// Verbatim candidate text.
    pub candidate: String,
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
    /// Candidates whose annotation block carries an id-shaped token, whether
    /// or not the declared grammar can read it. A bound candidate always
    /// counts as tagged. The block is the attached comments, attributes and
    /// decorators; the symbol's own declared name is never read (CR-215,
    /// FR-050-AC-44).
    pub tagged: usize,
    /// Candidates that minted at least one `verifies` relation.
    pub bound: usize,
    /// Every declared marker form consulted for this language — the list to
    /// check when `bound` is 0 and `candidates` is not.
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
    /// One candidate that carries an id-shaped token but bound nothing.
    /// This is the actionable split between an unread tag and a test that was
    /// never tagged at all (agent-ix/quire-rs#271).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unmatched_example: Option<UnboundSymbol>,
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

/// One evidence symbol the binding census counted as `tagged` but not `bound`:
/// it carries an id-shaped token and no declared form bound anything on it
/// (PLAT-875).
///
/// The census reports counts and one example per language; this is the whole
/// population, so a differential can report `tagged_not_bound` deltas. It is
/// distinct from [`UnmatchedTag`]: a symbol that bound through some other tag
/// leaves the census before its stray tokens are considered, so those tokens
/// appear in `unmatched_tags` but never here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaggedNotBound {
    /// Stable machine label — `rust`, `python`, `typescript`.
    pub language: String,
    /// Repo-relative, `/`-separated.
    pub path: String,
    /// 1-based line of the first id-shaped token, the line to open.
    pub line: usize,
    /// Qualified evidence-symbol name.
    pub symbol: String,
    /// The first id-shaped token found in the symbol's annotation block.
    pub trace_id: String,
}

/// One id-shaped token authored on an evidence symbol that no declared form
/// bound on that symbol (FR-050-AC-39, #362).
///
/// This is the row-addressable form of [`BindingCensus::unmatched_example`].
/// The engine owns the annotation boundary, so a downstream census can join
/// these records to authored rows without copying the language adapters or
/// mistaking id-shaped fixture data in a test body for a tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnmatchedTag {
    pub trace_id: String,
    /// Stable machine label — `rust`, `python`, `typescript`.
    pub language: String,
    /// Repo-relative, `/`-separated.
    pub path: String,
    /// 1-based line in the attached annotation block.
    pub line: usize,
    /// Qualified evidence-symbol name.
    pub symbol: String,
}

#[derive(Default)]
struct CensusAccumulator {
    candidates: usize,
    tagged: usize,
    bound: usize,
    unbound_example: Option<UnboundSymbol>,
    unmatched_example: Option<UnboundSymbol>,
}

impl CensusAccumulator {
    fn keep_lowest(slot: &mut Option<UnboundSymbol>, candidate: UnboundSymbol) {
        let better = match slot {
            None => true,
            Some(current) => (&candidate.path, candidate.line) < (&current.path, current.line),
        };
        if better {
            *slot = Some(candidate);
        }
    }

    fn observe(&mut self, bound: bool, generic_tag: Option<UnboundSymbol>, unbound: UnboundSymbol) {
        self.candidates += 1;
        if bound {
            self.bound += 1;
            self.tagged += 1;
            return;
        }

        Self::keep_lowest(&mut self.unbound_example, unbound);
        if let Some(example) = generic_tag {
            self.tagged += 1;
            Self::keep_lowest(&mut self.unmatched_example, example);
        }
    }

    fn finish(self, language: &str, model: &TraceabilityModel) -> BindingCensus {
        assert!(
            self.bound <= self.tagged && self.tagged <= self.candidates,
            "binding census invariant violated for {language}: bound {} <= tagged {} <= candidates {}",
            self.bound,
            self.tagged,
            self.candidates
        );
        BindingCensus {
            language: language.to_string(),
            candidates: self.candidates,
            tagged: self.tagged,
            bound: self.bound,
            forms: declared_forms(model, language),
            unbound_example: self.unbound_example,
            unmatched_example: self.unmatched_example,
        }
    }
}

/// A range — two id-shaped tokens joined by a bare `..`, in same-prefix,
/// differing-prefix, or short-suffix form — written inside a marker or a
/// textual `implements` use (CR-187, FR-050-AC-50, FR-051-AC-28). Binds no id
/// at all, including its own endpoints; this is the only record of the
/// occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeInTraceTag {
    pub path: String,
    pub symbol: String,
    /// 1-based line the range text sits on.
    pub line: usize,
    /// The literal range text, verbatim.
    pub range_text: String,
}

/// The symbol graph the coverage rollup and knowledge-graph ingestion consume.
/// Every collection is deterministically ordered (NFR-006).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SymbolGraph {
    pub verifies: Vec<VerifiesRelation>,
    /// A range written inside a marker or a textual `implements` use (CR-187,
    /// FR-050-AC-50, FR-051-AC-28) — binds no id, reported once per
    /// occurrence.
    pub range_diagnostics: Vec<RangeInTraceTag>,
    /// Requirement → production code (FR-062). **Never** consulted by
    /// [`Self::backed_trace_ids`]: this is scope, not evidence, and letting it
    /// back an acceptance criterion is exactly the coverage backdoor CR-061
    /// closed.
    pub implements: Vec<ImplementsRelation>,
    /// Textual Rust `implements` uses, for the FR-050-AC-62 coverage warning.
    /// Diagnostic input only: the same occurrences also bind in
    /// [`Self::implements`].
    pub legacy_implements: Vec<LegacyImplementsUse>,
    /// `(symbol id, symbol qualified name, file path)`.
    pub defined_in: Vec<(String, String, String)>,
    /// `(container qualified name, member qualified name, file path)`.
    pub contains: Vec<(String, String, String)>,
    pub diagnostics: Vec<TraceDiagnostic>,
    /// Rejected proper-substring matches, in declaration/form/source order.
    pub partial_attribute_matches: Vec<PartialAttributeMatch>,
    /// Rejected proper-substring matches of a Python or TypeScript structural
    /// form (FR-062), in declaration/form/source order.
    pub partial_syntax_matches: Vec<PartialAttributeMatch>,
    /// Wrapper calls a `typescript_wrapper` form names that attach to nothing
    /// (FR-062), in file/form/source order.
    pub unattached_syntax: Vec<UnattachedSyntax>,
    /// Per-language candidate/bound counts (FR-051-AC-19, CR-093). Ordered by
    /// language label, and present for every language the walk saw at least one
    /// evidence symbol in.
    pub binding_census: Vec<BindingCensus>,
    /// Generic id-shaped tokens on evidence-symbol annotation blocks that no
    /// declared form bound on that same symbol (FR-050-AC-39, #362).
    pub unmatched_tags: Vec<UnmatchedTag>,
    /// Every evidence symbol the census counted as tagged but not bound,
    /// sorted by `(language, path, line, symbol)` (PLAT-875). Read-only
    /// enumeration of the census population; it changes nothing that binds.
    pub tagged_not_bound: Vec<TaggedNotBound>,
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
    /// Languages the walk found source for but this build cannot extract,
    /// copied from [`SymbolExtraction::unavailable_languages`] so the coverage
    /// rollup — which sees only this graph — can refuse (FR-051-AC-29).
    pub unavailable_languages: Vec<crate::traceability::SourceLanguage>,
    /// Ids written in a comment or docstring attached to a declaration, which
    /// never binds (CR-215, FR-051-AC-49): the one advisory over non-marker
    /// trace text. Diagnostic input only — it binds nothing and moves no total.
    pub outside_marker: Vec<OutsideMarkerId>,
    /// Every generic id-shaped token found anywhere in the tree that is not
    /// part of a `verifies`/`implements` claim (FR-077-AC-2). Additive: a
    /// citation index over the same graph, computed after everything above
    /// it, reported alongside it rather than folded into it. No id-shaped
    /// token this engine finds is ever silently absent from all of
    /// `verifies`, `implements` and `mentions` at once.
    pub mentions: Vec<Mention>,
    /// Shell markers that matched a comment and bound nothing, and an
    /// `entry_paths` list that did not compile (FR-051-AC-39, AC-44).
    pub shell_findings: Vec<super::shell_trace::ShellFinding>,
}

/// What a comment or docstring id sits on, which decides what the advisory can
/// tell its author to write (FR-051-AC-22, AC-49).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutsideMarkerOwner {
    /// A symbol that binds trace ids (a test, benchmark or fuzz target): the
    /// finding names the marker to write.
    Evidence,
    /// A production function or container: a `trace` marker binds nothing
    /// there, so the finding names the `implements` text instead, or the kind
    /// alone.
    Production(SymbolKind),
    /// Comment text in no symbol's annotation block: a file header.
    Module,
}

/// One id-shaped token in a comment or docstring attached to a declaration
/// (CR-215, FR-051-AC-49). One per occurrence, an occurrence being one id on
/// one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutsideMarkerId {
    pub trace_id: String,
    pub language: SourceLanguage,
    pub path: String,
    /// 1-based line the token sits on.
    pub line: usize,
    /// The qualified symbol it is attached to, or the file path for a header.
    pub symbol: String,
    pub owner: OutsideMarkerOwner,
    /// The structural `implements` syntax the owning declaration admits
    /// (FR-051-AC-22, FR-062), when it has one.
    pub position: Option<crate::traceability::TraceMarkerSyntax>,
}

/// Which of the three non-claim shapes a [`Mention`] is (FR-077-AC-2).
///
/// Every generic id-shaped token this engine finds that is not a
/// `verifies`/`implements` claim lands in exactly one of these — the
/// no-silent-drop invariant FR-077 exists to hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MentionBucket {
    /// The same shape [`BindingCensus::unmatched_example`] already names: an
    /// id-shaped token on an evidence symbol's own annotation block that no
    /// declared form bound.
    EvidenceNearMiss,
    /// The same shape [`OutsideMarkerId`] names on a production symbol: an id
    /// in a comment or docstring where no `verifies` marker can bind.
    ProductionOrphanTag,
    /// A generic id-shaped token matching no declared form at all — a plain
    /// comment, doc comment, or string-literal citation. The bucket that did
    /// not exist before FR-077: nothing scanned production-symbol bodies or
    /// file scope for form-agnostic text before this.
    Mention,
}

impl MentionBucket {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EvidenceNearMiss => "evidence_near_miss",
            Self::ProductionOrphanTag => "production_orphan_tag",
            Self::Mention => "mention",
        }
    }
}

/// One id-shaped token found in a scanned tree that is not part of a
/// `verifies`/`implements` claim (FR-077-AC-2).
///
/// Additive and read-only: nothing here changes `verifies`, `implements`,
/// `unmatched_tags`, `outside_marker` or any FR-050 coverage figure.
/// `mentions` is a citation index, not a second binder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mention {
    pub trace_id: String,
    /// Stable machine label — `rust`, `python`, `typescript`.
    pub language: String,
    /// Repo-relative, `/`-separated.
    pub path: String,
    /// The innermost symbol whose span contains this token, or `None` when
    /// no symbol does — file-scope prose (a header comment before the first
    /// declaration) is reported rather than left unscanned.
    pub symbol: Option<String>,
    /// The owning symbol's kind label, mirroring `symbol`.
    pub kind: Option<&'static str>,
    pub bucket: MentionBucket,
    /// 1-based line of the token itself — not a symbol's annotation line,
    /// the line the match is actually on, so a reader can put a cursor on it
    /// without a second tool call.
    pub line: usize,
    /// The trimmed source line the token was found on, for the same reason:
    /// so seeing it never costs a second read.
    pub excerpt: String,
}

/// The id kinds a loaded corpus can mint: the leading alphabetic prefix of
/// every id the declared trace targets minted (`TC` for `TC-001`, `FR` for
/// `FR-001-AC-1`), compared case-insensitively (CR-215, FR-051-AC-49).
///
/// The traceability model declares *which documents mint* ids, not what the ids
/// look like, so the kinds are read off the minted ids themselves: a prefix no
/// target mints (`CR-`, `PLAT-`, a bare `AC-`, `AGPL-`) is not a trace id and is
/// not advised, recorded as unmatched or reported as a mention. A corpus that
/// minted nothing declares no kind, and then every id-shaped token is admitted
/// (the generic pattern, unfiltered), because there is nothing to tell a trace
/// id from any other.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IdKinds(BTreeSet<String>);

impl IdKinds {
    /// The kinds of the given minted ids.
    pub fn from_minted<I, S>(ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self(
            ids.into_iter()
                .filter_map(|id| id_kind(id.as_ref()))
                .collect(),
        )
    }

    /// Whether `token` may be a trace id: its kind is minted, or no kind is
    /// declared at all.
    pub fn admits(&self, token: &str) -> bool {
        self.0.is_empty() || id_kind(token).is_some_and(|kind| self.0.contains(&kind))
    }
}

/// The leading alphabetic run of an id, upper-cased: `FR` for `FR-001-AC-1`.
fn id_kind(id: &str) -> Option<String> {
    let kind: String = id
        .trim()
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect();
    (!kind.is_empty()).then(|| kind.to_ascii_uppercase())
}

impl SymbolGraph {
    /// Drop every generic id-shaped record (`unmatched_tags`, `mentions`,
    /// `outside_marker`) whose kind the corpus does not mint (CR-215). Bound
    /// relations are untouched: a marker binds whatever it names.
    pub fn retain_id_kinds(&mut self, kinds: &IdKinds) {
        self.unmatched_tags.retain(|t| kinds.admits(&t.trace_id));
        self.mentions.retain(|m| kinds.admits(&m.trace_id));
        self.outside_marker.retain(|o| kinds.admits(&o.trace_id));
    }

    /// Trace ids backed by at least one symbol.
    pub fn backed_trace_ids(&self) -> BTreeSet<&str> {
        self.verifies.iter().map(|v| v.trace_id.as_str()).collect()
    }
}

/// Regexes declared by one traceability model, compiled once for its bind pass.
#[derive(Debug, Default)]
struct CompiledTracePatterns {
    markers: BTreeMap<String, Option<Regex>>,
    implements: BTreeMap<String, Option<Regex>>,
}

impl CompiledTracePatterns {
    fn new(model: &TraceabilityModel) -> Self {
        let markers = model
            .trace_tags
            .markers
            .iter()
            .map(|marker| (marker.pattern.clone(), Regex::new(&marker.pattern).ok()))
            .collect();
        let implements = model
            .trace_tags
            .implements
            .iter()
            .map(|marker| (marker.pattern.clone(), Regex::new(&marker.pattern).ok()))
            .collect();
        Self {
            markers,
            implements,
        }
    }

    fn marker(&self, pattern: &str) -> Option<&Regex> {
        self.markers.get(pattern)?.as_ref()
    }

    fn implements(&self, pattern: &str) -> Option<&Regex> {
        self.implements.get(pattern)?.as_ref()
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
/// first. Tagging harder never helps. A shell script's evidence symbol is its
/// `entry_point`, and it binds only under a declared `entry_paths` glob
/// ([`super::shell_trace`]).
pub fn bind(extraction: &SymbolExtraction, model: &TraceabilityModel) -> SymbolGraph {
    let patterns = CompiledTracePatterns::new(model);
    let mut graph = SymbolGraph {
        excluded_source_files: extraction.excluded_source_files,
        unavailable_languages: extraction.unavailable_languages.clone(),
        ..SymbolGraph::default()
    };
    // FR-051-AC-19: what was looked at, per language, and what bound. Keyed by
    // the stable language label so the census order is a property of the data
    // rather than of the walk (NFR-006).
    let mut census: BTreeMap<&'static str, CensusAccumulator> = BTreeMap::new();
    let shell = super::shell_trace::ShellBinder::new(model, &mut graph);
    // Ranges read on a production symbol, with what decides which symbol an
    // occurrence is reported against: a container's span covers every symbol
    // in its file, so the same occurrence is read more than once.
    let mut production_ranges: Vec<(RangeInTraceTag, SymbolKind, usize)> = Vec::new();

    for symbol in &extraction.symbols {
        graph.defined_in.push((
            symbol.id.clone(),
            symbol.qualified_name.clone(),
            symbol.path.clone(),
        ));
        if let Some(container) = &symbol.container {
            #[cfg(feature = "rust-symbols")]
            let container = if symbol.language == SourceLanguage::Rust {
                super::rust::graph_parent(container)
            } else {
                container.clone()
            };
            #[cfg(not(feature = "rust-symbols"))]
            let container = container.clone();
            graph.contains.push((
                container,
                symbol.qualified_name.clone(),
                symbol.path.clone(),
            ));
        }
        // Shell reads comment nodes, not the attached source text (PLAT-362).
        if symbol.language == SourceLanguage::Shell {
            shell.bind_symbol(symbol, &mut graph);
            continue;
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
            let implements_ranges = bind_implements(symbol, source, model, &patterns, &mut graph);
            production_ranges.extend(
                implements_ranges
                    .into_iter()
                    .map(|range| (range, symbol.kind, symbol.leading_line)),
            );
            graph.implements_candidates += 1;
            if graph.implements.len() > before {
                graph.implements_bound += 1;
            }
            // A `verifies` marker on a production symbol binds nothing, and a
            // range inside it binds nothing either; it is reported like any
            // other (FR-051-AC-28).
            let (_, ranges) = verifies_form_ids(symbol, source, model, &patterns);
            production_ranges.extend(
                ranges
                    .into_iter()
                    .map(|range| (range, symbol.kind, symbol.leading_line)),
            );
            continue;
        }
        if !symbol.kind.binds_trace_ids() {
            continue;
        }
        // Counted around the call rather than inside it: `bind_symbol` only
        // ever appends to `verifies`, so the length delta is exactly "did this
        // candidate bind" — and the count stays correct if the binder grows
        // another form.
        let generic_tags = generic_tags(symbol, source);
        let generic_tag_locus = generic_tags.first().map(|tag| UnboundSymbol {
            path: tag.path.clone(),
            line: tag.line,
            symbol: tag.symbol.clone(),
        });
        let first_generic = generic_tags.first().map(|tag| TaggedNotBound {
            language: tag.language.clone(),
            path: tag.path.clone(),
            line: tag.line,
            symbol: tag.symbol.clone(),
            trace_id: tag.trace_id.clone(),
        });
        let before = graph.verifies.len();
        let ranges_before = graph.range_diagnostics.len();
        bind_symbol(symbol, source, model, &patterns, &mut graph);
        let bound_on_symbol: BTreeSet<String> = graph.verifies[before..]
            .iter()
            .map(|relation| relation.trace_id.clone())
            .collect();
        // CR-187: a range's own endpoints must not resurface as generic
        // unmatched tags — the range finding replaces both `untracked_symbols`
        // and `unmatched_tags` for that text (FR-050-AC-50).
        let range_tokens: BTreeSet<String> = graph.range_diagnostics[ranges_before..]
            .iter()
            .flat_map(|r| r.range_text.split("..").map(|s| s.trim().to_string()))
            .collect();
        graph
            .unmatched_tags
            .extend(generic_tags.into_iter().filter(|tag| {
                !bound_on_symbol.contains(tag.trace_id.as_str())
                    && !range_tokens.contains(&tag.trace_id)
            }));
        if graph.verifies.len() == before {
            graph.tagged_not_bound.extend(first_generic);
        }
        let entry = census.entry(symbol.language.as_str()).or_default();
        entry.observe(
            graph.verifies.len() > before,
            generic_tag_locus,
            UnboundSymbol {
                path: symbol.path.clone(),
                line: symbol.leading_line,
                symbol: symbol.qualified_name.clone(),
            },
        );
    }

    bind_unattached_wrappers(extraction, model, &patterns, &mut graph);

    // Appending is cheap in the symbol loop. Sort once after every symbol has
    // been visited; sorting in `bind_symbol` would invalidate the per-symbol
    // append range used above and, more importantly, sort the growing graph N
    // times.
    graph
        .verifies
        .sort_by(|a, b| (&a.path, &a.symbol, &a.trace_id).cmp(&(&b.path, &b.symbol, &b.trace_id)));

    // A range an evidence symbol already reported is that symbol's: a container
    // spanning the test reads the same text and must not report it again.
    let reported: BTreeSet<RangeOccurrence> = graph
        .range_diagnostics
        .iter()
        .map(range_occurrence)
        .collect();

    shell.report_unbound(extraction, &mut graph);

    graph.suspicions = crate::skeptic::vacuous_property_suites(extraction);
    graph
        .suspicions
        .extend(crate::skeptic::oracle_copies_in(extraction));
    graph
        .suspicions
        .sort_by(|a, b| (&a.path, a.line, &a.symbol).cmp(&(&b.path, b.line, &b.symbol)));
    graph.binding_census = census
        .into_iter()
        .map(|(language, entry)| entry.finish(language, model))
        .collect();
    graph.unmatched_tags.sort_by(|a, b| {
        (&a.language, &a.path, a.line, &a.symbol, &a.trace_id).cmp(&(
            &b.language,
            &b.path,
            b.line,
            &b.symbol,
            &b.trace_id,
        ))
    });

    graph.implements.sort_by(|a, b| {
        (&a.path, &a.symbol, &a.trace_id, &a.form).cmp(&(&b.path, &b.symbol, &b.trace_id, &b.form))
    });
    graph.implements.dedup();
    // An authored line is one use, owned by the innermost symbol that reads
    // it: sort the innermost owner first within each (path, line) and keep it.
    graph.legacy_implements.sort_by(|a, b| {
        (
            &a.path,
            a.line,
            std::cmp::Reverse(a.owner_leading_line),
            &a.symbol,
            &a.trace_ids,
        )
            .cmp(&(
                &b.path,
                b.line,
                std::cmp::Reverse(b.owner_leading_line),
                &b.symbol,
                &b.trace_ids,
            ))
    });
    graph
        .legacy_implements
        .dedup_by(|later, kept| later.path == kept.path && later.line == kept.line);
    graph.tagged_not_bound.sort_by(|a, b| {
        (&a.language, &a.path, a.line, &a.symbol).cmp(&(&b.language, &b.path, b.line, &b.symbol))
    });
    graph.defined_in.sort();
    graph.contains.sort();
    graph
        .diagnostics
        .sort_by(|a, b| (&a.path, &a.symbol, &a.trace_id).cmp(&(&b.path, &b.symbol, &b.trace_id)));
    // Otherwise the innermost production symbol wins: greatest `leading_line`,
    // a container losing ties.
    let occurrence = range_occurrence;
    production_ranges.sort_by(|(a, a_kind, a_lead), (b, b_kind, b_lead)| {
        let container = |kind: &SymbolKind| *kind == SymbolKind::Container;
        (occurrence(a), b_lead, container(a_kind)).cmp(&(occurrence(b), a_lead, container(b_kind)))
    });
    production_ranges.dedup_by(|(a, _, _), (b, _, _)| occurrence(a) == occurrence(b));
    graph.range_diagnostics.extend(
        production_ranges
            .into_iter()
            .map(|(range, _, _)| range)
            .filter(|range| !reported.contains(&occurrence(range))),
    );

    // CR-187: one record per occurrence, deterministically ordered.
    graph.range_diagnostics.sort_by(|a, b| {
        (&a.path, a.line, &a.symbol, &a.range_text).cmp(&(
            &b.path,
            b.line,
            &b.symbol,
            &b.range_text,
        ))
    });
    // Two declared forms can read the identical range on one line; the dedup
    // key equals the sort key, so those records are adjacent and collapse.
    graph.range_diagnostics.dedup_by(|a, b| {
        a.path == b.path && a.line == b.line && a.symbol == b.symbol && a.range_text == b.range_text
    });

    // Computed last and additively, both reading the graph built above: the
    // advisory over comment and docstring ids (FR-051-AC-49) and the citation
    // index (FR-077-AC-2).
    graph.outside_marker = outside_marker_ids(extraction, &graph);
    graph.mentions = find_mentions(extraction, &graph);

    graph
}

/// Every generic id-shaped token in `extraction` that is not already part of
/// a `verifies`/`implements` claim (FR-077-AC-2).
///
/// One pass per FILE over the whole source text, not per symbol: scanning
/// per-symbol would re-read every line once per enclosing container
/// (a citation pass that also wants file-scope, un-owned text does the
/// attribution itself instead).
/// Each match is attributed to the innermost symbol in that file whose span
/// contains it, using the same "highest `leading_line` wins, a `Container`
/// loses ties" rule already used for the advisory — or to no symbol at
/// all, when the token sits outside every declared span (a file header, a
/// trailing comment after the last item).
///
/// Deliberately reuses [`generic_id_pattern`] rather than declaring a second
/// id-shaped pattern (FR-077-CON-1), and deliberately does **not** mask string
/// literals (FR-077-CON-3): hiding an id the caller asked about because it
/// happens to sit inside a string literal is the failure mode this pass exists
/// to avoid. (The advisory masks them, FR-051-AC-24: it tells an author what to
/// write, which data is not.)
fn find_mentions(extraction: &SymbolExtraction, graph: &SymbolGraph) -> Vec<Mention> {
    // Claimed (symbol identity, normalized trace id) pairs — a match here is
    // not a citation, it is the claim itself, and reporting it twice would
    // make every bound test also read as citing itself.
    let mut claimed: BTreeSet<(&str, String)> = BTreeSet::new();
    for relation in &graph.verifies {
        claimed.insert((
            relation.symbol_id.as_str(),
            normalized_trace_id(&relation.trace_id),
        ));
    }
    for relation in &graph.implements {
        claimed.insert((
            relation.symbol_id.as_str(),
            normalized_trace_id(&relation.trace_id),
        ));
    }

    // (path, symbol qualified name, normalized trace id) already named by the
    // narrower existing passes — consulted only to choose the more specific
    // bucket label. A miss here still reports as `Mention`, never drops the
    // token: at worst this assigns the coarser of two honest labels.
    let evidence_near_miss: BTreeSet<(&str, &str, String)> = graph
        .unmatched_tags
        .iter()
        .map(|t| {
            (
                t.path.as_str(),
                t.symbol.as_str(),
                normalized_trace_id(&t.trace_id),
            )
        })
        .collect();
    let production_orphan: BTreeSet<(&str, &str, String)> = graph
        .outside_marker
        .iter()
        .filter(|t| matches!(t.owner, OutsideMarkerOwner::Production(_)))
        .map(|t| {
            (
                t.path.as_str(),
                t.symbol.as_str(),
                normalized_trace_id(&t.trace_id),
            )
        })
        .collect();

    // CR-187, FR-051-AC-28: a range's own endpoints
    // contribute to neither `unmatched_tags` nor `mentions` — the range
    // finding is the only record of the occurrence. Keyed by (path, line,
    // normalized endpoint), since this pass reads one physical line at a
    // time the same way `RangeInTraceTag::line` was computed.
    let range_endpoints = range_endpoints(graph);

    let pattern = generic_id_pattern();
    let mut mentions = Vec::new();

    for file in &extraction.files {
        let symbols_in_file: Vec<&Symbol> = extraction
            .symbols
            .iter()
            .filter(|s| s.path == file.path)
            .collect();

        for (idx, text) in file.source.lines().enumerate() {
            let line = idx + 1;
            for matched in pattern.find_iter(text) {
                let trace_id = matched.as_str().to_string();

                if range_endpoints.contains(&(
                    file.path.as_str(),
                    line,
                    normalized_trace_id(&trace_id),
                )) {
                    continue;
                }

                let owner = innermost_symbol_at(&symbols_in_file, line);

                if let Some(symbol) = owner {
                    if claimed.contains(&(symbol.id.as_str(), normalized_trace_id(&trace_id))) {
                        continue;
                    }
                }

                let bucket = match owner {
                    Some(symbol)
                        if evidence_near_miss.contains(&(
                            file.path.as_str(),
                            symbol.qualified_name.as_str(),
                            normalized_trace_id(&trace_id),
                        )) =>
                    {
                        MentionBucket::EvidenceNearMiss
                    }
                    Some(symbol)
                        if production_orphan.contains(&(
                            file.path.as_str(),
                            symbol.qualified_name.as_str(),
                            normalized_trace_id(&trace_id),
                        )) =>
                    {
                        MentionBucket::ProductionOrphanTag
                    }
                    _ => MentionBucket::Mention,
                };

                mentions.push(Mention {
                    trace_id,
                    language: file.language.as_str().to_string(),
                    path: file.path.clone(),
                    symbol: owner.map(|s| s.qualified_name.clone()),
                    kind: owner.map(|s| s.kind.as_str()),
                    bucket,
                    line,
                    excerpt: text.trim().to_string(),
                });
            }
        }
    }

    mentions.sort_by(|a, b| {
        (&a.path, a.line, &a.trace_id, &a.symbol).cmp(&(&b.path, b.line, &b.trace_id, &b.symbol))
    });
    mentions
}

/// The endpoints of every range finding, keyed by `(path, line, normalized id)`
/// (FR-051-AC-28): a range is the only record of its occurrence, so its
/// endpoints are claimed by no other channel.
fn range_endpoints(graph: &SymbolGraph) -> BTreeSet<(&str, usize, String)> {
    graph
        .range_diagnostics
        .iter()
        .flat_map(|r| {
            r.range_text
                .split("..")
                .map(move |token| (r.path.as_str(), r.line, normalized_trace_id(token.trim())))
        })
        .collect()
}

/// Every id-shaped token in a comment or docstring attached to a declaration,
/// which never binds (CR-215, FR-051-AC-22, AC-24, AC-49, AC-50).
///
/// The advisory replaces the textual trace forms with one question: *is there
/// an id here that a marker should carry?* It reads each file's annotation
/// channel ([`annotation_channel`]): comments everywhere, a Python docstring
/// because it is a string literal by syntax, and nothing else — string
/// literals, code and a symbol's body are blanked, so an id a file carries as
/// data is not reported. A symbol's annotation block is the lines from its
/// first attribute, decorator or leading comment to its declaration line, and a
/// Python docstring directly after the signature; the lines above the first
/// symbol are the file's header. Where several symbols span one token the
/// innermost owns it, and a container loses a tie.
///
/// One finding per occurrence, an occurrence being one id on one line. It fires
/// whether or not the id is also backed (AC-50), binds nothing, and is
/// additive to `unmatched_tags` and `mentions`. An id an `implements` form read
/// belongs to FR-050-AC-62 and a range endpoint to the range finding, so
/// neither is reported here.
fn outside_marker_ids(extraction: &SymbolExtraction, graph: &SymbolGraph) -> Vec<OutsideMarkerId> {
    let implements_read: BTreeSet<(&str, String)> = graph
        .implements
        .iter()
        .map(|r| (r.symbol_id.as_str(), normalized_trace_id(&r.trace_id)))
        .collect();
    let range_endpoints = range_endpoints(graph);
    let pattern = generic_id_pattern();

    let mut by_path: BTreeMap<&str, Vec<&Symbol>> = BTreeMap::new();
    for symbol in &extraction.symbols {
        by_path
            .entry(symbol.path.as_str())
            .or_default()
            .push(symbol);
    }

    let mut found = Vec::new();
    for file in &extraction.files {
        let Some(channel) = annotation_channel(&file.source, file.language) else {
            continue;
        };
        let channel_lines: Vec<&str> = channel.lines().collect();
        let code = (file.language == SourceLanguage::Python)
            .then(|| mask_source_string_contents(&file.source, file.language));
        let code_lines: Vec<&str> = code
            .as_deref()
            .map(str::lines)
            .into_iter()
            .flatten()
            .collect();
        let symbols = by_path
            .get(file.path.as_str())
            .map_or(&[][..], Vec::as_slice);

        // The synthetic whole-file module container of the Python and TypeScript
        // adapters owns no annotation block of its own: its tokens are the
        // header's.
        let is_module = |s: &Symbol| {
            file.language != SourceLanguage::Rust
                && s.kind == SymbolKind::Container
                && s.container.is_none()
                && s.line == 1
                && s.leading_line == 1
        };
        let header_end = symbols
            .iter()
            .filter(|s| !is_module(s))
            .map(|s| s.leading_line)
            .min()
            .unwrap_or(channel_lines.len() + 1)
            - 1;
        let header_owner = symbols
            .iter()
            .find(|s| is_module(s))
            .map_or(file.path.as_str(), |s| s.qualified_name.as_str());

        let mut owned = Owned::new();
        claim(
            &mut owned,
            &channel_lines,
            pattern,
            1..=header_end,
            (0, false),
            None,
        );
        for symbol in symbols.iter().filter(|s| !is_module(s)) {
            let rank = (symbol.leading_line, symbol.kind != SymbolKind::Container);
            claim(
                &mut owned,
                &channel_lines,
                pattern,
                symbol.leading_line..=symbol.line,
                rank,
                Some(symbol),
            );
            if file.language == SourceLanguage::Python {
                if let Some(docstring) = python_docstring_lines(symbol, &code_lines, &channel_lines)
                {
                    claim(
                        &mut owned,
                        &channel_lines,
                        pattern,
                        docstring,
                        rank,
                        Some(symbol),
                    );
                }
            }
        }

        for ((line, trace_id), (_, owner)) in owned {
            let normalized = normalized_trace_id(trace_id);
            if range_endpoints.contains(&(file.path.as_str(), line, normalized.clone())) {
                continue;
            }
            let position = owner.and_then(|symbol| symbol.position);
            let (symbol, kind) = match owner {
                Some(symbol) => {
                    if implements_read.contains(&(symbol.id.as_str(), normalized)) {
                        continue;
                    }
                    let kind = if symbol.kind.binds_trace_ids() {
                        OutsideMarkerOwner::Evidence
                    } else {
                        OutsideMarkerOwner::Production(symbol.kind)
                    };
                    (symbol.qualified_name.clone(), kind)
                }
                None => (header_owner.to_string(), OutsideMarkerOwner::Module),
            };
            found.push(OutsideMarkerId {
                trace_id: trace_id.to_string(),
                language: file.language,
                path: file.path.clone(),
                line,
                symbol,
                owner: kind,
                position,
            });
        }
    }
    found.sort_by(|a, b| (&a.path, a.line, &a.trace_id).cmp(&(&b.path, b.line, &b.trace_id)));
    found
}

/// How strongly a symbol owns an annotation token: the greater `leading_line`
/// is the innermost declaration, and a container loses a tie.
type Rank = (usize, bool);

/// `(line, id)` → the best owner so far, `None` being the file header.
type Owned<'a> = BTreeMap<(usize, &'a str), (Rank, Option<&'a Symbol>)>;

/// Record every id-shaped token on `lines` of the annotation channel as owned by
/// `owner` unless a stronger owner already holds it.
fn claim<'a>(
    owned: &mut Owned<'a>,
    channel_lines: &[&'a str],
    pattern: &Regex,
    lines: std::ops::RangeInclusive<usize>,
    rank: Rank,
    owner: Option<&'a Symbol>,
) {
    for line in lines {
        let Some(text) = line.checked_sub(1).and_then(|i| channel_lines.get(i)) else {
            continue;
        };
        for matched in pattern.find_iter(text) {
            let slot = owned
                .entry((line, matched.as_str()))
                .or_insert((rank, owner));
            if rank > slot.0 {
                *slot = (rank, owner);
            }
        }
    }
}

/// The lines of the docstring that opens the body of a Python symbol, if it has
/// one: the first statement after the signature, a triple-quoted literal
/// (FR-051-AC-24). A literal further down is body text, not an annotation.
///
/// `code_lines` is the file with string contents blanked and code kept, which is
/// what makes the signature's end findable: the first line at bracket depth zero
/// must end in `:`; a one-line `def f(): ...` has no body line to read.
fn python_docstring_lines(
    symbol: &Symbol,
    code_lines: &[&str],
    channel_lines: &[&str],
) -> Option<std::ops::RangeInclusive<usize>> {
    let mut depth: i64 = 0;
    let mut signature_end = None;
    for line in symbol.line..=symbol.end_line {
        let code = code_lines.get(line.checked_sub(1)?)?;
        let code = code.split('#').next().unwrap_or_default();
        for ch in code.chars() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }
        if depth <= 0 {
            if code.trim_end().ends_with(':') {
                signature_end = Some(line);
            }
            break;
        }
    }
    let mut first = signature_end? + 1;
    while code_lines.get(first - 1)?.trim().is_empty() {
        first += 1;
    }
    if first > symbol.end_line {
        return None;
    }
    let opener = channel_lines.get(first - 1)?.trim_start();
    let opener = opener.trim_start_matches(|c: char| "rRbBuUfF".contains(c));
    let delimiter = ["\"\"\"", "'''"]
        .into_iter()
        .find(|delimiter| opener.starts_with(delimiter))?;
    // Closed on its own line, or on a later one.
    if opener[delimiter.len()..].contains(delimiter) {
        return Some(first..=first);
    }
    let last = (first + 1..=symbol.end_line)
        .find(|line| {
            channel_lines
                .get(line - 1)
                .is_some_and(|text| text.contains(delimiter))
        })
        .unwrap_or(symbol.end_line);
    Some(first..=last)
}

/// The innermost symbol in `symbols` (already filtered to one file) whose
/// span contains `line`, or `None` when no symbol's span does.
///
/// Greatest `leading_line` wins (the more deeply nested declaration
/// starts later), and at equal `leading_line` a [`SymbolKind::Container`]
/// loses — a container's span starts at the top of the file, so ties there
/// are ties against something that is never the more specific answer.
fn innermost_symbol_at<'a>(symbols: &[&'a Symbol], line: usize) -> Option<&'a Symbol> {
    symbols
        .iter()
        .filter(|s| s.leading_line <= line && line <= s.end_line)
        .max_by_key(|s| {
            let non_container = usize::from(!matches!(s.kind, super::SymbolKind::Container));
            (s.leading_line, non_container)
        })
        .copied()
}

/// The segment-preserving form of the engine's trace-id fold: the maximal runs
/// of ASCII alphanumerics in `value`, uppercased, in order (PLAT-898).
///
/// This is the one implementation of the fold. [`normalized_trace_id`] is
/// `trace_id_segments(value).concat()`, so the two cannot disagree. A consumer
/// that needs boundary-aware prefix matching (`FR-047` matches `FR-047-AC-9`
/// but not `FR-0470`) compares segments rather than the concatenation, which
/// discards exactly the boundaries that comparison needs.
pub fn trace_id_segments(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(str::to_uppercase)
        .collect()
}

pub(crate) fn normalized_trace_id(value: &str) -> String {
    trace_id_segments(value).concat()
}

/// The generic, form-agnostic id-shaped pattern: `AB-123` or
/// `AB-123-CD-456`. Shared by [`generic_tags`] (evidence-symbol near-miss
/// detection, FR-050) and [`find_mentions`] (whole-file citation detection,
/// FR-077) so the two passes never drift onto two different notions of
/// "id-shaped" — one compiled pattern, two call sites, per FR-077-CON-1.
fn generic_id_pattern() -> &'static Regex {
    static GENERIC_ID: OnceLock<Regex> = OnceLock::new();
    GENERIC_ID.get_or_init(|| {
        Regex::new(r"(?i)\b[A-Z]{2,4}-[0-9]+(?:-[A-Z]+-[0-9]+)?\b")
            .expect("generic trace-id pattern compiles")
    })
}

/// Find every generic id-shaped token in the symbol's attached annotation
/// block.
///
/// This deliberately does not reuse the declared trace grammar: that would
/// reproduce the blindness being measured. It also stops at the declaration
/// line, so an id mentioned in a test body is data used by the test rather than
/// evidence that the test itself was tagged, and it blanks the declaration's own
/// name there: a name is not an annotation, so an id a title or identifier
/// carries adds to no census count (CR-215, FR-050-AC-44).
fn generic_tags(symbol: &Symbol, source: &str) -> Vec<UnmatchedTag> {
    let pattern = generic_id_pattern();
    let start = symbol.leading_line.saturating_sub(1);
    let count = symbol.line.saturating_sub(start).max(1);
    let name = symbol.bare_name();
    let mut tags = Vec::new();
    for (index, text) in source.lines().enumerate().skip(start).take(count) {
        let line = index + 1;
        let mut scanned = text.to_string();
        if line == symbol.line {
            for name in [symbol.qualified_name.as_str(), name] {
                if !name.is_empty() {
                    scanned = scanned.replacen(name, &" ".repeat(name.len()), 1);
                }
            }
        }
        tags.extend(pattern.find_iter(&scanned).map(|matched| UnmatchedTag {
            trace_id: matched.as_str().to_string(),
            language: symbol.language.as_str().to_string(),
            path: symbol.path.clone(),
            line,
            symbol: symbol.qualified_name.clone(),
        }));
    }
    tags
}

/// Every declared marker form the binder consults for `language`
/// (FR-051-AC-19).
fn declared_forms(model: &TraceabilityModel, language: &str) -> Vec<String> {
    model
        .trace_tags
        .markers
        .iter()
        .filter(|m| m.language.as_str() == language)
        .map(|m| m.name.clone())
        .collect()
}

/// Implements: FR-062
/// Bind a production symbol to the requirements it implements (FR-062).
///
/// Structurally simpler than [`bind_symbol`] because there is no precedence
/// between forms. A requirement named twice by two forms yields one relation
/// after the dedup in [`bind`].
///
/// A range in an `implements` marker binds nothing either, including its own
/// endpoints (CR-187, FR-051-AC-28); it is returned for [`bind`] to report,
/// once per occurrence, against the innermost symbol that carries it.
fn bind_implements(
    symbol: &Symbol,
    source: &str,
    model: &TraceabilityModel,
    patterns: &CompiledTracePatterns,
    graph: &mut SymbolGraph,
) -> Vec<RangeInTraceTag> {
    let mut ranges = Vec::new();
    if model.trace_tags.implements.is_empty() {
        return ranges;
    }
    let span = symbol.attached_source(source);
    for marker in &model.trace_tags.implements {
        if marker.language != symbol.language {
            continue;
        }
        let Some(re) = patterns.implements(&marker.pattern) else {
            continue;
        };
        let candidates: Vec<(&str, usize)> = match marker.syntax {
            Some(crate::traceability::TraceMarkerSyntax::RustAttribute) => symbol
                .attributes
                .iter()
                .map(|attribute| (attribute.text.as_str(), attribute.line))
                .collect(),
            // A structural form reads only the candidates of a declaration whose
            // position is its own: a decorator form never reads a wrapper's
            // callee calls, nor the reverse (FR-062).
            Some(
                structural @ (crate::traceability::TraceMarkerSyntax::PythonDecorator
                | crate::traceability::TraceMarkerSyntax::TypescriptDecorator
                | crate::traceability::TraceMarkerSyntax::TypescriptWrapper),
            ) => {
                if symbol.position == Some(structural) {
                    symbol
                        .attributes
                        .iter()
                        .map(|attribute| (attribute.text.as_str(), attribute.line))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            None | Some(crate::traceability::TraceMarkerSyntax::Comment) => {
                vec![(span.as_str(), symbol.leading_line)]
            }
        };
        for (candidate, line) in candidates {
            let mut partial = false;
            for caps in re.captures_iter(candidate) {
                if marker.syntax.is_some()
                    && !caps.get(0).is_some_and(|matched| {
                        matched.start() == 0 && matched.end() == candidate.len()
                    })
                {
                    partial = true;
                    continue;
                }
                let Some(args) = caps.get(1) else { continue };
                let mut bound_ids = Vec::new();
                for trace_id in marker_ids(args.as_str()) {
                    if let Some(mut range) = marker_range(symbol, candidate, &caps, &trace_id) {
                        range.line = line + range.line.saturating_sub(symbol.leading_line);
                        ranges.push(range);
                        continue;
                    }
                    bound_ids.push(trace_id.clone());
                    graph.implements.push(ImplementsRelation {
                        symbol_id: symbol.id.clone(),
                        symbol: symbol.qualified_name.clone(),
                        path: symbol.path.clone(),
                        trace_id,
                        form: marker.name.clone(),
                    });
                }
                if marker.syntax.is_none()
                    && marker.language == SourceLanguage::Rust
                    && !bound_ids.is_empty()
                {
                    // The authored line of the ids, not of the match: a pattern
                    // opening with `^\s*` starts on a preceding blank line.
                    let offset = caps
                        .get(1)
                        .map_or(0, |ids| candidate[..ids.start()].matches('\n').count());
                    graph.legacy_implements.push(LegacyImplementsUse {
                        path: symbol.path.clone(),
                        symbol: symbol.qualified_name.clone(),
                        line: line + offset,
                        trace_ids: bound_ids,
                        owner_leading_line: symbol.leading_line,
                    });
                }
            }
            if partial {
                let finding = PartialAttributeMatch {
                    form: marker.name.clone(),
                    path: symbol.path.clone(),
                    symbol: symbol.qualified_name.clone(),
                    line,
                    attribute: candidate.to_string(),
                };
                if marker.syntax == Some(crate::traceability::TraceMarkerSyntax::RustAttribute) {
                    graph.partial_attribute_matches.push(finding);
                } else {
                    graph.partial_syntax_matches.push(finding);
                }
            }
        }
    }
    ranges
}

/// Report each curried call a declared `typescript_wrapper` form names in full
/// but that no wrapper chain attaches to a declarator (FR-062). It binds nothing;
/// a call written as a decorator is never among these candidates.
fn bind_unattached_wrappers(
    extraction: &SymbolExtraction,
    model: &TraceabilityModel,
    patterns: &CompiledTracePatterns,
    graph: &mut SymbolGraph,
) {
    for file in &extraction.files {
        if file.unattached_calls.is_empty() {
            continue;
        }
        for marker in &model.trace_tags.implements {
            if marker.language != file.language
                || marker.syntax != Some(crate::traceability::TraceMarkerSyntax::TypescriptWrapper)
            {
                continue;
            }
            let Some(re) = patterns.implements(&marker.pattern) else {
                continue;
            };
            for call in &file.unattached_calls {
                let whole = re.captures_iter(&call.text).any(|caps| {
                    caps.get(0)
                        .is_some_and(|m| m.start() == 0 && m.end() == call.text.len())
                });
                if whole {
                    graph.unattached_syntax.push(UnattachedSyntax {
                        form: marker.name.clone(),
                        path: file.path.clone(),
                        line: call.line,
                        candidate: call.text.clone(),
                    });
                }
            }
        }
    }
}

/// Blank the *contents* of Rust string literals, preserving byte length and
/// line structure so downstream offset arithmetic is unaffected.
///
/// Comments are left intact: a `"` inside a comment must not open a string, or
/// an apostrophe-quote in prose would mask every line that follows. With
/// `keep_code` false everything that is neither a comment nor a string
/// delimiter is blanked too, leaving the annotation channel alone (the advisory
/// of FR-051-AC-49 reads nothing else).
fn mask_rust_string_contents(span: &str, keep_code: bool) -> String {
    let mut out = String::with_capacity(span.len());
    let mut block_depth: u32 = 0;
    let mut raw_hashes: Option<usize> = None;
    let mut in_string = false;

    // PLAT-181: each line is walked by byte index and decoded in place, so no
    // a char buffer is built per line. Every lookahead below is for an ASCII
    // character, so a byte comparison is exact and a multi-byte character is
    // stepped over whole (`len_utf8`), which keeps byte offsets unshifted.
    for line in span.split_inclusive('\n') {
        let bytes = line.as_bytes();
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

        while i < line.len() {
            let Some(c) = line[i..].chars().next() else {
                break;
            };
            let width = c.len_utf8();

            if let Some(hashes) = raw_hashes {
                if c == '"' && bytes[i + 1..].iter().take(hashes).all(|h| *h == b'#') {
                    raw_hashes = None;
                    out.push(c);
                    let present = hashes.min(bytes.len() - i - 1);
                    for _ in 0..present {
                        out.push('#');
                    }
                    i += 1 + hashes;
                    continue;
                }
                blank(c, &mut out);
                i += width;
                continue;
            }

            if in_string {
                if c == '\\' {
                    blank(c, &mut out);
                    let next = line[i + 1..].chars().next();
                    if let Some(n) = next {
                        blank(n, &mut out);
                    }
                    i += 1 + next.map_or(1, char::len_utf8);
                    continue;
                }
                if c == '"' {
                    in_string = false;
                    out.push(c);
                } else {
                    blank(c, &mut out);
                }
                i += width;
                continue;
            }

            if block_depth > 0 {
                if c == '/' && bytes.get(i + 1) == Some(&b'*') {
                    block_depth += 1;
                    out.push_str("/*");
                    i += 2;
                    continue;
                }
                if c == '*' && bytes.get(i + 1) == Some(&b'/') {
                    block_depth -= 1;
                    out.push_str("*/");
                    i += 2;
                    continue;
                }
                out.push(c);
                i += width;
                continue;
            }

            // A line comment runs to end of line and is kept verbatim.
            if c == '/' && bytes.get(i + 1) == Some(&b'/') {
                out.push_str(&line[i..]);
                break;
            }
            if c == '/' && bytes.get(i + 1) == Some(&b'*') {
                block_depth = 1;
                out.push_str("/*");
                i += 2;
                continue;
            }
            // `r"…"` / `r#"…"#` / `br#"…"#`
            if (c == 'r' || c == 'b') && !prev_is_ident(line, i) {
                let mut j = i;
                if bytes[j] == b'b' {
                    j += 1;
                }
                if bytes.get(j) == Some(&b'r') {
                    let start = j + 1;
                    let hashes = bytes[start..].iter().take_while(|h| **h == b'#').count();
                    if bytes.get(start + hashes) == Some(&b'"') {
                        raw_hashes = Some(hashes);
                        out.push_str(&line[i..=start + hashes]);
                        i = start + hashes + 1;
                        continue;
                    }
                }
            }
            // A char literal holding a quote (`'"'`, `'\"'`) is not a string.
            if c == '\'' {
                let literal_len = if bytes[i..].starts_with(b"'\"'") {
                    3
                } else if bytes[i..].starts_with(b"'\\\"'") {
                    4
                } else {
                    0
                };
                if literal_len > 0 {
                    if keep_code {
                        out.push_str(&line[i..i + literal_len]);
                    } else {
                        out.push_str(&" ".repeat(literal_len));
                    }
                    i += literal_len;
                    continue;
                }
            }
            if c == '"' {
                in_string = true;
                out.push(c);
                i += 1;
                continue;
            }
            if keep_code {
                out.push(c);
            } else {
                blank(c, &mut out);
            }
            i += width;
        }
    }
    out
}

/// True when the char before `i` could continue an identifier, so an `r` there
/// is part of a name (`str"` never opens a raw string) rather than a prefix.
fn prev_is_ident(line: &str, i: usize) -> bool {
    line[..i]
        .chars()
        .next_back()
        .is_some_and(|prev| prev.is_alphanumeric() || prev == '_')
}

/// Blank the *contents* of TypeScript and Python string literals, preserving
/// byte length and line structure exactly as [`mask_rust_string_contents`] does.
///
/// A SEPARATE FUNCTION rather than a generalisation of the Rust one. Rust's
/// raw-string form has no analogue in either language and its handling is the
/// delicate part of that lexer; folding three languages into one loop would put
/// the two at risk of each other for no gain. This engine already keeps a
/// symbol extractor per language for the same reason.
///
/// Comments are left INTACT: `//` and block comments for TypeScript, `#` for
/// Python. With `keep_docstrings`, a Python triple-quoted literal that opens its
/// line is kept too: Python has no doc-comment syntax, so its annotation channel
/// IS a string literal (FR-051-AC-24). A literal opened mid-line (`x = """`) is an
/// assigned value and is masked like any other string. With `keep_code` false
/// everything that is neither a comment, a kept docstring nor a string delimiter
/// is blanked too, leaving the annotation channel alone.
fn mask_script_string_contents(
    span: &str,
    language: SourceLanguage,
    keep_docstrings: bool,
    keep_code: bool,
) -> String {
    let python = matches!(language, SourceLanguage::Python);
    let mut out = String::with_capacity(span.len());
    // The delimiter that opened the string we are inside, if any. A triple
    // quote is held as all three characters so a lone quote inside it does not
    // close it — the shape #274 was about, one layer down.
    let mut open: Option<&'static str> = None;
    let mut block_comment = false;
    // Whether the literal currently open is a DOCSTRING — Python's doc channel,
    // preserved like a comment.
    let mut doc_string = false;

    for line in span.split_inclusive('\n') {
        // PLAT-181: walked by byte index and decoded in place; no per-line
        // char buffer. Every lookahead is for an ASCII character, so byte
        // comparison is exact and a multi-byte char is stepped over whole.
        let bytes = line.as_bytes();
        // Everything before byte `i` is whitespace exactly while `i` has not
        // passed the first non-whitespace byte. Computed once per line rather
        // than rescanned at every character (quadratic on a long indented line).
        let first_non_ws = line
            .find(|x: char| !x.is_whitespace())
            .unwrap_or(line.len());
        let mut i = 0;
        let blank = |c: char, out: &mut String| {
            if c == '\n' || c == '\r' || !c.is_ascii() {
                out.push(c);
            } else {
                out.push(' ');
            }
        };

        while i < line.len() {
            let Some(c) = line[i..].chars().next() else {
                break;
            };
            let width = c.len_utf8();
            // Whether everything before this position on the line is
            // whitespace — the declaration's own `^\s*` anchor for the
            // docstring form.
            let opens_line = i <= first_non_ws;

            if block_comment {
                if c == '*' && bytes.get(i + 1) == Some(&b'/') {
                    block_comment = false;
                    out.push_str("*/");
                    i += 2;
                    continue;
                }
                out.push(c);
                i += width;
                continue;
            }

            if let Some(delim) = open {
                // An escape consumes the next character, so an escaped quote
                // never closes the literal.
                if c == '\\' && !doc_string {
                    blank(c, &mut out);
                    let next = line[i + 1..].chars().next();
                    if let Some(n) = next {
                        blank(n, &mut out);
                    }
                    i += 1 + next.map_or(1, char::len_utf8);
                    continue;
                }
                if line[i..].starts_with(delim) {
                    out.push_str(delim);
                    i += delim.len();
                    open = None;
                    doc_string = false;
                    continue;
                }
                if doc_string {
                    out.push(c);
                } else {
                    blank(c, &mut out);
                }
                i += width;
                continue;
            }

            // Line comments run to end of line and are kept verbatim.
            if (python && c == '#') || (!python && c == '/' && bytes.get(i + 1) == Some(&b'/')) {
                out.push_str(&line[i..]);
                break;
            }
            if !python && c == '/' && bytes.get(i + 1) == Some(&b'*') {
                block_comment = true;
                out.push_str("/*");
                i += 2;
                continue;
            }

            // TRIPLE QUOTES FIRST, or the single form opens and immediately
            // closes on its own second character and the body stays unmasked.
            let triple = if python {
                ["\"\"\"", "'''"]
                    .into_iter()
                    .find(|delim| line[i..].starts_with(delim))
            } else {
                None
            };
            if let Some(delim) = triple {
                out.push_str(delim);
                // A DOCSTRING IS PYTHON'S COMMENT. The test is POSITIONAL: a
                // literal whose opener has only whitespace before it on its
                // line is documentation; one opened mid-line is a value.
                if keep_docstrings && opens_line {
                    doc_string = true;
                }
                open = Some(delim);
                i += 3;
                continue;
            }
            if c == '"' || c == '\'' || (!python && c == '`') {
                out.push(c);
                open = Some(match c {
                    '"' => "\"",
                    '\'' => "'",
                    _ => "`",
                });
                i += 1;
                continue;
            }
            if keep_code {
                out.push(c);
            } else {
                blank(c, &mut out);
            }
            i += width;
        }
    }
    out
}

/// The annotation channel of a whole file: comments everywhere and Python
/// docstrings, with every other character blanked (FR-051-AC-24). Byte length
/// and line count are preserved, so a line in the result is the line in the
/// original.
///
/// `None` for shell: its tags are read from comment nodes and its comments are
/// FR-051-AC-44's, not this advisory's.
fn annotation_channel(source: &str, language: SourceLanguage) -> Option<String> {
    match language {
        SourceLanguage::Rust => Some(mask_rust_string_contents(source, false)),
        SourceLanguage::Typescript | SourceLanguage::Python => {
            Some(mask_script_string_contents(source, language, true, false))
        }
        SourceLanguage::Shell => None,
    }
}

/// Blank every string's contents for a static check that reads code rather
/// than an annotation channel. Unlike [`annotation_channel`], this keeps the
/// code and preserves no Python docstring.
pub(crate) fn mask_source_string_contents(span: &str, language: SourceLanguage) -> String {
    match language {
        SourceLanguage::Rust => mask_rust_string_contents(span, true),
        SourceLanguage::Typescript | SourceLanguage::Python => {
            mask_script_string_contents(span, language, false, true)
        }
        SourceLanguage::Shell => span.to_string(),
    }
}

/// Every trace id a declared **verifies** marker attaches to this symbol's
/// span, with the form that attached it.
///
/// Returns every match, including duplicates: deduplication is a *binding*
/// decision (FR-051-AC-6) and belongs to the caller that binds.
///
/// A range — two id-shaped tokens joined by a bare `..`, same-prefix,
/// differing-prefix, or short-suffix (CR-187, FR-050-AC-50, FR-051-AC-28) —
/// binds no id: it is filtered out of the returned id list and reported instead
/// in the second return value, once per occurrence. A marker's argument list
/// already yields a range as ONE literal token, so the range is simply the
/// whole id-shaped token failing the id shape and matching the range shape
/// instead. Filtered here rather than spliced out of the raw argument text,
/// which would leave behind an empty quoted pair (`""`) that a naive
/// comma-split misreads as a literal empty id.
fn verifies_form_ids(
    symbol: &Symbol,
    source: &str,
    model: &TraceabilityModel,
    patterns: &CompiledTracePatterns,
) -> (Vec<(String, String)>, Vec<RangeInTraceTag>) {
    let span = symbol.attached_source(source);
    let mut out = Vec::new();
    let mut ranges = Vec::new();
    for marker in &model.trace_tags.markers {
        if marker.language != symbol.language {
            continue;
        }
        let Some(re) = patterns.marker(&marker.pattern) else {
            continue;
        };
        for caps in re.captures_iter(&span) {
            let Some(args) = caps.get(1) else { continue };
            for trace_id in marker_ids(args.as_str()) {
                if let Some(range) = marker_range(symbol, &span, &caps, &trace_id) {
                    ranges.push(range);
                    continue;
                }
                out.push((trace_id, marker.name.clone()));
            }
        }
    }
    (out, ranges)
}

/// Whole-token range pattern (CR-187, FR-050-AC-50, FR-051-AC-28): two
/// id-shaped tokens joined by a bare `..`, or an id-shaped token followed by
/// `..` and a bare short-suffix number. Anchored, because it tests a single
/// token a grammar has already isolated — a marker's own id — never a raw
/// span.
pub(crate) fn range_pattern() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)^\s*[A-Z]{2,4}-[0-9]+(?:-[A-Z]+-[0-9]+)?\s*\.\.\s*([A-Z]{2,4}-[0-9]+(?:-[A-Z]+-[0-9]+)?|[0-9]+)\s*$",
        )
        .expect("range pattern compiles")
    })
}

/// The range a marker's argument token spells, if it is one (CR-187,
/// FR-051-AC-28), located on the line the token itself sits on, so a marker
/// wrapped over several lines still reports the line `find_mentions` reads
/// the endpoints from.
fn marker_range(
    symbol: &Symbol,
    span: &str,
    caps: &regex::Captures<'_>,
    trace_id: &str,
) -> Option<RangeInTraceTag> {
    if !range_pattern().is_match(trace_id) {
        return None;
    }
    let start = caps
        .get(1)
        .and_then(|args| args.as_str().find(trace_id).map(|at| args.start() + at))
        .or_else(|| caps.get(0).map(|m| m.start()))
        .unwrap_or(0);
    Some(RangeInTraceTag {
        path: symbol.path.clone(),
        symbol: symbol.qualified_name.clone(),
        line: symbol.leading_line + span[..start].matches('\n').count(),
        range_text: trace_id.to_string(),
    })
}

/// What makes two range records one occurrence: the same text on the same
/// line of the same file, whichever symbol read it.
type RangeOccurrence = (String, usize, String);

fn range_occurrence(range: &RangeInTraceTag) -> RangeOccurrence {
    (range.path.clone(), range.line, range.range_text.clone())
}

fn bind_symbol(
    symbol: &Symbol,
    source: &str,
    model: &TraceabilityModel,
    patterns: &CompiledTracePatterns,
    graph: &mut SymbolGraph,
) {
    // Every attachment of a trace id to this symbol, in discovery order. The id
    // is bound once no matter how many forms attached it (FR-051-AC-6).
    let mut attachments: BTreeMap<String, Vec<VerifiesRelation>> = BTreeMap::new();
    let (forms, ranges) = verifies_form_ids(symbol, source, model, patterns);
    graph.range_diagnostics.extend(ranges);
    for (trace_id, form) in forms {
        attachments
            .entry(trace_id.clone())
            .or_default()
            .push(VerifiesRelation {
                symbol_id: symbol.id.clone(),
                symbol: symbol.qualified_name.clone(),
                path: symbol.path.clone(),
                trace_id,
                form,
                line: symbol.line,
                column: symbol.column,
                kind: symbol.kind,
                ignored: symbol.ignored,
            });
    }

    // One relation per trace id: an id attached more than once yields exactly
    // one diagnostic naming every form that attached it (FR-045 edge-dedup
    // convention).
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
        // `else` arm is unreachable — expressed as an `if let` rather than an
        // `expect` so there is no panic path in library code at all.
        if let Some(first) = candidates.into_iter().next() {
            graph.verifies.push(first);
        }
    }
}

/// Ids as quoted literals joined by `, `, the argument list of a rendered
/// marker (`"FR-1-AC-1", "FR-1-AC-2"`).
pub(crate) fn quoted_id_list(ids: &[String]) -> String {
    ids.iter()
        .map(|id| format!("\"{id}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The trace ids inside a marker's argument list. Quoted arguments are the
/// authored form (`trace("TC-1", "FR-2-AC-3")`); an unquoted argument list is
/// taken whole, so a bare-identifier marker form still binds.
pub(crate) fn marker_ids(args: &str) -> Vec<String> {
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
/// org/repo. Ordering is deterministic and ids are stable, so repeated
/// emission over an identical tree is byte-identical (FR-051-AC-10).
pub fn graph_records(
    extraction: &SymbolExtraction,
    graph: &SymbolGraph,
    input: &SymbolRecordInput,
) -> Result<(Vec<CoreGraphNodeRef>, Vec<CoreGraphEdgeRef>), super::ExtractionUnavailable> {
    extraction.require_available()?;
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
    Ok((nodes, edges))
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
    use crate::traceability::TraceMarkerForm;
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
                "#[trace(\"TC-577\", \"NFR-002-AC-4\")]\n",
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
        // target's span is the whole file. A leading-annotation walk cannot
        // reach across non-comment lines.
        let fuzz = crate::symbols::extract_file(
            "fuzz/fuzz_targets/f.rs",
            SourceLanguage::Rust,
            concat!(
                "#![no_main]\n",
                "#[trace(\"TC-579\")]\n",
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
                "#[trace(\"TC-741\")]\n",
                "mod thing {\n",
                "    #[trace(\"FR-051-AC-1\")]\n",
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
                "test.describe(\"warning default\", () => {\n",
                "  trace(\"TC-001\"); // Playwright spelling, on the SUITE.\n",
                "  test(\"defaults every finding to warning\", () => {});\n",
                "});\n",
                "\n",
                "describe(\"plain suite\", () => {\n",
                "  trace(\"TC-002\"); // bare spelling, on the SUITE.\n",
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
                "describe(\"warning default\", () => {\n",
                "  trace(\"TC-001\"); // on the BLOCK.\n",
                "  it(\"defaults every finding to warning\", () => {\n",
                "    expect(1 + 1).toBe(2);\n",
                "  });\n",
                "});\n",
                "\n",
                "it(\"names the declaration on every finding\", () => {\n",
                "  trace(\"TC-002\"); // the same form, on the `it`.\n",
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

    #[trace("TC-894", "FR-051-AC-6")]
    // a tag-shaped snippet inside a Rust string literal is data,
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
        let (nodes, edges) = graph_records(&extraction, &graph, &record_input()).unwrap();

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
            let (nodes, edges) = graph_records(&extraction, &graph, &record_input()).unwrap();
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
        model
    }

    /// A one-file Python extraction, so a test about WHERE a tag sits does not
    /// depend on a fixture tree that other tests also assert against.
    fn py(source: &str) -> SymbolExtraction {
        crate::symbols::extract_file("src/m.py", SourceLanguage::Python, source)
    }

    /// The advisory's findings as `(line, id, symbol)` triples, in report order.
    fn advised(graph: &SymbolGraph) -> Vec<(usize, &str, &str)> {
        graph
            .outside_marker
            .iter()
            .map(|f| (f.line, f.trace_id.as_str(), f.symbol.as_str()))
            .collect()
    }

    #[trace("TC-1044", "FR-051-AC-22")]
    // an id in the annotation block of a symbol whose kind cannot bind is
    // reported naming the id, the symbol and its kind, and the payload is
    // otherwise unmoved.
    #[test]
    fn tc1044_an_id_on_a_symbol_that_cannot_bind_is_reported() {
        // `normalize_severity` is a plain `Function`, so `binds_trace_ids()` is
        // false: the comment reaches no channel at all, and the census cannot
        // say so — a non-binding symbol is missing from `candidates` rather than
        // counted as unbound.
        let graph = bind(
            &py("# TC-001: warning default.\ndef normalize_severity(f):\n    return 1\n"),
            &iso_model(),
        );
        assert_eq!(advised(&graph), [(1, "TC-001", "normalize_severity")]);
        let finding = &graph.outside_marker[0];
        assert_eq!(
            finding.owner,
            OutsideMarkerOwner::Production(SymbolKind::Function)
        );
        assert_eq!(finding.path, "src/m.py");
        assert_eq!(finding.language, SourceLanguage::Python);

        // And the payload is otherwise unmoved: nothing binds, and the symbol
        // does not become a census candidate.
        assert!(graph.verifies.is_empty(), "the id still binds nothing");
        assert!(
            graph.binding_census.is_empty(),
            "a non-binding symbol does not become a census candidate"
        );
    }

    #[trace("TC-1046", "FR-051-AC-22")]
    // where several symbols span one id the INNERMOST is named, because the
    // module is where the id is not; a container loses a tie.
    #[test]
    fn tc1046_the_innermost_symbol_spanning_an_id_is_the_one_named() {
        // The file's module container and `normalize_severity` both span this
        // comment. The fix a reader needs names the function.
        let graph = bind(
            &py("import os\n\n\n# TC-001: warning default.\ndef normalize_severity(f):\n    return 1\n"),
            &iso_model(),
        );
        assert_eq!(advised(&graph), [(4, "TC-001", "normalize_severity")]);

        // On line 1 the module container and the function share `leading_line`;
        // the container loses the tie.
        let graph = bind(
            &py("# TC-001: warning default.\ndef normalize_severity(f):\n    return 1\n"),
            &iso_model(),
        );
        assert_eq!(advised(&graph), [(1, "TC-001", "normalize_severity")]);

        // A method's comment belongs to the method, not to its class.
        let graph = bind(
            &rs("mod tests {\n    // TC-001: on the test.\n    #[test]\n    fn t() {\n    }\n}\n"),
            &iso_model(),
        );
        assert_eq!(advised(&graph), [(2, "TC-001", "tests::t")]);
        assert_eq!(graph.outside_marker[0].owner, OutsideMarkerOwner::Evidence);
    }

    #[trace("TC-1081", "FR-051-AC-22")]
    // An id quoted inside comment prose remains visible to the advisory, while
    // the same prose with a redacted non-id stays silent: the check still
    // depends on an id rather than on quoted text, and no caller-specific mask
    // is introduced.
    #[test]
    fn tc1081_a_quoted_id_in_prose_remains_visible() {
        let quoted = bind(
            &py("# The banner quotes `# FR-001-AC-1: Model Resolution`.\ndef normalize_severity(f):\n    return 1\n"),
            &iso_model(),
        );
        assert_eq!(advised(&quoted), [(1, "FR-001-AC-1", "normalize_severity")]);

        let redacted = bind(
            &py("# The banner quotes `# FR-NNN: Model Resolution`.\ndef normalize_severity(f):\n    return 1\n"),
            &iso_model(),
        );
        assert!(redacted.outside_marker.is_empty());
    }

    #[trace("TC-2165", "FR-051-AC-22", "FR-051-AC-49")]
    // A canonical marker on a production function, a container or a file
    // yields nothing; a comment id on the same function is reported with its
    // kind; the same id twice on a line is one finding; `unmatched_tags` is
    // unchanged by the advisory.
    #[test]
    fn tc2165_a_marker_is_never_reported_and_a_comment_id_is_once_per_line() {
        let marker_only = bind(
            &rs(concat!(
                "#[trace(\"FR-007-AC-1\")]\n",
                "mod holder {\n",
                "    #[trace(\"FR-007-AC-1\")]\n",
                "    pub fn resolve() {}\n",
                "}\n",
            )),
            &iso_model(),
        );
        assert!(
            marker_only.outside_marker.is_empty(),
            "a marker is an attribute, never a comment: {:?}",
            marker_only.outside_marker
        );

        let graph = bind(
            &rs(concat!(
                "// FR-007-AC-1 header prose.\n",
                "\n",
                "mod holder {\n",
                "    /// FR-007-AC-1 and again FR-007-AC-1.\n",
                "    pub fn resolve() {}\n",
                "}\n",
            )),
            &iso_model(),
        );
        let owners: Vec<(usize, OutsideMarkerOwner)> = graph
            .outside_marker
            .iter()
            .map(|f| (f.line, f.owner))
            .collect();
        assert_eq!(
            owners,
            [
                (1, OutsideMarkerOwner::Module),
                (4, OutsideMarkerOwner::Production(SymbolKind::Function)),
            ],
            "a header is a module, a function is named with its kind, and the id \
             written twice on one line is one finding"
        );
        assert_eq!(graph.outside_marker[0].symbol, "src/lib.rs");
        assert_eq!(graph.outside_marker[1].symbol, "holder::resolve");
        assert!(graph.verifies.is_empty() && graph.implements.is_empty());
    }

    #[trace("TC-2163", "FR-051-AC-50", "FR-051-AC-24")]
    // The advisory fires whether or not the id is also backed, stays silent for
    // a body and a string literal outside the annotation channel, yields
    // nothing for an all-marker tree, and goes quiet once retagged.
    #[test]
    fn tc2163_the_advisory_fires_on_backed_ids_and_ignores_bodies_and_strings() {
        let graph = bind(
            &py(concat!(
                "# TC-741: also a comment.\n",
                "@pytest.mark.trace(\"TC-741\")\n",
                "def test_backed():\n",
                "    # TC-902 in the body\n",
                "    fixture = \"TC-903\"\n",
                "    text = \"\"\"\n",
                "TC-904\n",
                "\"\"\"\n",
                "    assert fixture\n",
            )),
            &iso_model(),
        );
        assert!(graph.verifies.iter().any(|v| v.trace_id == "TC-741"));
        assert_eq!(
            advised(&graph),
            [(1, "TC-741", "test_backed")],
            "a backed id is still reported; a body comment, a string and a \
             triple-quoted value are not"
        );

        // Retagged: the comment goes, the finding goes.
        let tidy = bind(
            &py("@pytest.mark.trace(\"TC-741\")\ndef test_backed():\n    assert True\n"),
            &iso_model(),
        );
        assert!(tidy.outside_marker.is_empty());
    }

    #[trace("TC-2161", "FR-051-AC-49")]
    // The finding is additive to `unmatched_tags` and `mentions`: neither is
    // replaced or suppressed, and one finding is made per id per line.
    #[test]
    fn tc2161_the_advisory_is_additive_to_unmatched_tags_and_mentions() {
        let graph = bind(
            &rs(concat!(
                "// FR-001-AC-1 first\n",
                "// FR-001-AC-2 second\n",
                "// FR-001-AC-3 third\n",
                "#[test]\n",
                "fn t() {}\n",
            )),
            &iso_model(),
        );
        assert_eq!(
            advised(&graph),
            [
                (1, "FR-001-AC-1", "t"),
                (2, "FR-001-AC-2", "t"),
                (3, "FR-001-AC-3", "t"),
            ],
            "three ids on three lines are three findings"
        );
        for id in ["FR-001-AC-1", "FR-001-AC-2", "FR-001-AC-3"] {
            assert!(
                graph.unmatched_tags.iter().any(|t| t.trace_id == id),
                "{id}"
            );
            assert!(graph.mentions.iter().any(|m| m.trace_id == id), "{id}");
        }
        assert!(graph.verifies.is_empty());
    }

    #[trace("TC-2161", "FR-051-AC-49")]
    // The advisory covers Rust, Python and TypeScript comment channels.
    #[test]
    fn tc2161_the_advisory_reads_each_languages_comment_channel() {
        let ts = crate::symbols::extract_file(
            "src/m.test.ts",
            SourceLanguage::Typescript,
            "/** TC-001: doc comment */\nit(\"t\", () => {});\n",
        );
        for (label, extraction) in [
            ("rust", rs("/// TC-001: doc\n#[test]\nfn t() {}\n")),
            ("python", py("# TC-001: line\ndef test_t():\n    pass\n")),
            (
                "python docstring",
                py("def test_t():\n    \"\"\"TC-001: docstring.\"\"\"\n    pass\n"),
            ),
            ("typescript", ts),
        ] {
            let graph = bind(&extraction, &iso_model());
            assert_eq!(
                graph.outside_marker.len(),
                1,
                "{label}: {:?}",
                graph.outside_marker
            );
            assert_eq!(graph.outside_marker[0].trace_id, "TC-001", "{label}");
            assert_eq!(
                graph.outside_marker[0].owner,
                OutsideMarkerOwner::Evidence,
                "{label}"
            );
        }
    }

    #[trace("TC-2164", "FR-050-AC-44")]
    // A test whose name embeds a trace id adds nothing to `tagged` or `bound`.
    #[test]
    fn tc2164_an_id_in_a_name_adds_nothing_to_the_census() {
        let ts = crate::symbols::extract_file(
            "src/m.test.ts",
            SourceLanguage::Typescript,
            "it(\"TC-901 binds nothing\", () => {});\n",
        );
        for (label, extraction) in [
            ("rust", rs("#[test]\nfn tc_901_named() {}\n")),
            ("python", py("def test_tc_901_named():\n    pass\n")),
            ("typescript", ts),
        ] {
            let graph = bind(&extraction, &iso_model());
            let census = &graph.binding_census[0];
            assert_eq!(
                (census.candidates, census.tagged, census.bound),
                (1, 0, 0),
                "{label}"
            );
            assert!(graph.verifies.is_empty(), "{label}");
            assert!(graph.tagged_not_bound.is_empty(), "{label}");
            assert!(
                graph.outside_marker.is_empty(),
                "{label}: a name is not advised"
            );
        }
    }

    #[trace("TC-2160", "FR-051-AC-11")]
    // Per language, an id in a docstring, a doc comment or a line comment, a
    // `Trace:` line, or the test's own name mints no `verifies` relation; the
    // same tests with a marker bind; the relation carries no provenance.
    #[test]
    fn tc2160_only_a_marker_binds_in_every_language() {
        let ts = |source: &str| {
            crate::symbols::extract_file("src/m.test.ts", SourceLanguage::Typescript, source)
        };
        let unmarked = [
            (
                "rust",
                rs(concat!(
                    "// Trace: FR-901-AC-1\n",
                    "#[test]\n",
                    "fn tc901_named() {\n    // TC-902\n}\n",
                    "\n",
                    "/// TC-903: a doc comment\n",
                    "#[test]\n",
                    "fn plain() {}\n",
                )),
            ),
            (
                "python",
                py(concat!(
                    "# Trace: FR-901-AC-1\n",
                    "def test_tc_901_named():\n    # TC-902\n    pass\n",
                    "\n",
                    "def test_docstring():\n    \"\"\"TC-903: a docstring\"\"\"\n    pass\n",
                )),
            ),
            (
                "typescript",
                ts(concat!(
                    "// Trace: FR-901-AC-1\n",
                    "it(\"TC-901 named\", () => {\n  // TC-902\n});\n",
                    "\n",
                    "/** TC-903: a doc comment */\n",
                    "it(\"plain\", () => {});\n",
                )),
            ),
        ];
        for (label, extraction) in &unmarked {
            let graph = bind(extraction, &iso_model());
            assert!(graph.verifies.is_empty(), "{label}: {:?}", graph.verifies);
            assert!(graph.binding_census.iter().all(|c| c.bound == 0), "{label}");
        }

        let marked = [
            (
                "rust",
                rs("#[trace(\"TC-901\")]\n#[test]\nfn tc901_named() {}\n"),
            ),
            (
                "python",
                py("@pytest.mark.trace(\"TC-901\")\ndef test_tc_901_named():\n    pass\n"),
            ),
            (
                "typescript",
                ts("it(\"named\", () => {\n  trace(\"TC-901\");\n});\n"),
            ),
        ];
        for (label, extraction) in &marked {
            let model = iso_model();
            let graph = bind(extraction, &model);
            assert_eq!(
                graph
                    .verifies
                    .iter()
                    .map(|v| v.trace_id.as_str())
                    .collect::<Vec<_>>(),
                ["TC-901"],
                "{label}"
            );
            // The emitted edge carries the form and nothing else.
            let (_, edges) = graph_records(extraction, &graph, &record_input()).unwrap();
            let verifies = edges.iter().find(|e| e.edge_type == "verifies").unwrap();
            let data: Value = serde_json::from_str(&verifies.data_json).unwrap();
            assert_eq!(
                data.as_object().unwrap().keys().collect::<Vec<_>>(),
                ["form"],
                "{label}"
            );
        }
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
            assert!(
                entry.bound <= entry.tagged && entry.tagged <= entry.candidates,
                "{} preserves bound <= tagged <= candidates",
                entry.language
            );
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

    #[trace("TC-1074", "FR-050-AC-39", "FR-066-AC-3", "FR-066-AC-6")]
    // every unread authored id is row-addressable without widening the
    // declared binder or scanning test bodies (#362).
    #[test]
    fn tc1074_unmatched_annotation_ids_are_exposed_per_symbol() {
        let graph = bind(
            &py(concat!(
                "@pytest.mark.trace(\"TC-741\")\n",
                "# malformed marker names TC-404\n",
                "def test_mixed_annotation():\n",
                "    fixture_data = \"TC-999\"\n",
            )),
            &iso_model(),
        );

        assert!(
            graph.verifies.iter().any(|relation| {
                relation.symbol == "test_mixed_annotation" && relation.trace_id == "TC-741"
            }),
            "the declared sibling proves the symbol can otherwise bind"
        );
        assert_eq!(
            graph.unmatched_tags,
            vec![UnmatchedTag {
                trace_id: "TC-404".to_string(),
                language: "python".to_string(),
                path: "src/m.py".to_string(),
                line: 2,
                symbol: "test_mixed_annotation".to_string(),
            }],
            "the bound sibling and body-only fixture id are both absent"
        );
        assert!(graph.unmatched_tags.windows(2).all(|records| {
            (
                &records[0].language,
                &records[0].path,
                records[0].line,
                &records[0].symbol,
                &records[0].trace_id,
            ) <= (
                &records[1].language,
                &records[1].path,
                records[1].line,
                &records[1].symbol,
                &records[1].trace_id,
            )
        }));
    }

    #[test]
    fn plat875_tagged_not_bound_enumerates_the_census_population() {
        let graph = bind(
            &py(concat!(
                "# malformed marker names TC-404\n",
                "def test_tagged_only():\n",
                "    pass\n",
                "\n",
                "@pytest.mark.trace(\"TC-741\")\n",
                "# stray prose naming TC-405\n",
                "def test_bound_with_stray():\n",
                "    pass\n",
                "\n",
                "def test_untagged():\n",
                "    pass\n",
            )),
            &iso_model(),
        );
        assert_eq!(
            graph.tagged_not_bound,
            vec![TaggedNotBound {
                language: "python".to_string(),
                path: "src/m.py".to_string(),
                line: 1,
                symbol: "test_tagged_only".to_string(),
                trace_id: "TC-404".to_string(),
            }],
            "bound-with-stray and untagged symbols are not in the population"
        );
        // The enumeration and the census count agree.
        let census = graph
            .binding_census
            .iter()
            .find(|entry| entry.language == "python")
            .expect("python census");
        assert_eq!(
            census.tagged - census.bound,
            graph.tagged_not_bound.len(),
            "accessor must enumerate exactly the census's tagged-but-unbound count"
        );
        // The stray token on the bound symbol stays in the other bucket.
        assert!(graph.unmatched_tags.iter().any(|t| t.trace_id == "TC-405"));
    }

    #[test]
    fn plat898_segments_concat_is_the_normalized_fold() {
        assert_eq!(trace_id_segments("fr_047-ac-1"), ["FR", "047", "AC", "1"]);
        assert_eq!(trace_id_segments("FR-047-"), ["FR", "047"]);
        assert_eq!(trace_id_segments("FR047"), ["FR047"]);
        assert_eq!(trace_id_segments("FR-0\u{e9}47"), ["FR", "0", "47"]);
        assert!(trace_id_segments("--").is_empty());
        for id in [
            "FR-047",
            "fr_047",
            "FR047",
            "FR-047-AC-9",
            "FR-0470",
            "FR-0\u{e9}47",
            "",
            "..",
        ] {
            assert_eq!(
                trace_id_segments(id).concat(),
                normalized_trace_id(id),
                "fold disagreement on {id:?}"
            );
        }
        // Segment-aware prefix: the decoy the concatenation cannot reject.
        let prefix = trace_id_segments("FR-047");
        assert!(trace_id_segments("FR-047-AC-9").starts_with(&prefix));
        assert!(!trace_id_segments("FR-0470").starts_with(&prefix));
    }

    /// PLAT-181: masked output is byte-identical for multi-byte characters in
    /// strings, comments and docstrings; ASCII is blanked, multi-byte kept.
    #[test]
    fn plat181_masks_preserve_multibyte_and_byte_offsets() {
        let rust =
            "let s = \"h\u{e9}llo \u{2192} x\"; // \u{fc}ber TC-1\nlet r = r#\"a\u{e9}\"b\"#;\n";
        let masked = mask_rust_string_contents(rust, true);
        assert_eq!(
            masked,
            "let s = \" \u{e9}    \u{2192}  \"; // \u{fc}ber TC-1\nlet r = r#\" \u{e9}  \"#;\n"
        );
        assert_eq!(masked.len(), rust.len());

        let ts = "const a = \"x\u{e9}y\"; // \u{e9} \"q\"\nconst b = `m\u{2192}n`; /* \u{e9}\n\"z\" */\n";
        let masked = mask_script_string_contents(ts, SourceLanguage::Typescript, false, true);
        assert_eq!(
            masked,
            "const a = \" \u{e9} \"; // \u{e9} \"q\"\nconst b = ` \u{2192} `; /* \u{e9}\n\"z\" */\n"
        );
        assert_eq!(masked.len(), ts.len());

        let py = "x = \"\"\"a\u{e9}b\"\"\"\n    \"\"\"doc \u{e9} TC-2\"\"\"\ny = 'k\u{2192}'  # \u{e9}\n";
        let masked = mask_script_string_contents(py, SourceLanguage::Python, true, true);
        assert_eq!(
            masked,
            "x = \"\"\" \u{e9} \"\"\"\n    \"\"\"doc \u{e9} TC-2\"\"\"\ny = ' \u{2192}'  # \u{e9}\n"
        );
        assert_eq!(masked.len(), py.len());
    }

    /// PLAT-181 / PLAT-1248: masking is linear. A long whitespace-led line used
    /// to re-scan its whole prefix at every character.
    #[test]
    fn plat181_masking_a_long_whitespace_line_is_linear() {
        let line = format!("{}\"x\" y\n", " ".repeat(300_000));
        let start = std::time::Instant::now();
        let masked = mask_script_string_contents(&line, SourceLanguage::Python, true, true);
        let elapsed = start.elapsed();
        assert_eq!(masked.len(), line.len());
        assert!(
            elapsed < std::time::Duration::from_secs(2),
            "masking 300k chars took {elapsed:?}; it is quadratic"
        );
    }

    /// A one-file Rust extraction, mirroring [`py`] above.
    fn rs(source: &str) -> SymbolExtraction {
        crate::symbols::extract_file("src/lib.rs", SourceLanguage::Rust, source)
    }

    #[trace("TC-1888", "FR-077-AC-2")]
    #[test]
    fn tc1888_every_id_shaped_token_lands_in_exactly_one_bucket() {
        // The no-silent-drop invariant FR-077 exists to hold: over a fixture
        // carrying one of each of the five shapes this engine can find, every
        // id lands in exactly one of {verifies, implements, mentions} — never
        // zero (dropped) and never more than one (double-counted).
        //
        // Residual gap: "exactly one" is checked against three hardcoded
        // channel names (verifies/implements/mentions). A future bucket that
        // ABSORBS a shape from one of these three — leaving an id in zero of
        // them — trips this test as written. A future bucket that instead
        // DUPLICATES an existing shape, landing an id in a fourth channel
        // *as well as* one of these three, would not: this test only counts
        // membership in {verifies, implements, mentions}, not exhaustiveness
        // over every channel the engine might ever grow. Widening this check
        // to catch that needs an explicit list of all known channels, kept
        // current by hand — not attempted here.
        let mut model = iso_model();
        model.trace_tags.implements.push(TraceMarkerForm {
            syntax: None,
            name: "rust-implements-line".to_string(),
            language: SourceLanguage::Rust,
            pattern: r"(?m)^\s*//\s*Implements:\s*(.+)$".to_string(),
            template: None,
            entry_paths: None,
        });

        let source = "\
#[trace(\"FR-901\")]\n\
#[test]\n\
fn tc_verifies() {\n\
    assert!(true);\n\
}\n\
\n\
// see FR-903, unrelated near-miss\n\
#[test]\n\
fn tc_near_miss() {\n\
    assert!(true);\n\
}\n\
\n\
// Implements: FR-902\n\
fn fn_implements() {\n\
}\n\
\n\
// Trace: FR-904\n\
fn fn_orphan_tag() {\n\
}\n\
\n\
fn fn_plain_mention() {\n\
    // See FR-905 for background.\n\
}\n\
";
        let extraction = rs(source);
        let graph = bind(&extraction, &model);

        // Each id appears in exactly one of the three channels.
        let ids = ["FR-901", "FR-902", "FR-903", "FR-904", "FR-905"];
        for id in ids {
            let in_verifies = graph.verifies.iter().any(|v| v.trace_id == id);
            let in_implements = graph.implements.iter().any(|i| i.trace_id == id);
            let in_mentions = graph.mentions.iter().any(|m| m.trace_id == id);
            let hits = [in_verifies, in_implements, in_mentions]
                .into_iter()
                .filter(|hit| *hit)
                .count();
            assert_eq!(
                hits, 1,
                "{id} must land in exactly one of {{verifies, implements, mentions}}: \
                 verifies={in_verifies} implements={in_implements} mentions={in_mentions}"
            );
        }

        // And the specific buckets are the RIGHT ones, not just "some bucket":
        assert!(graph.verifies.iter().any(|v| v.trace_id == "FR-901"));
        assert!(graph.implements.iter().any(|i| i.trace_id == "FR-902"));
        let bucket_of = |id: &str| {
            graph
                .mentions
                .iter()
                .find(|m| m.trace_id == id)
                .map(|m| m.bucket)
        };
        assert_eq!(bucket_of("FR-903"), Some(MentionBucket::EvidenceNearMiss));
        assert_eq!(
            bucket_of("FR-904"),
            Some(MentionBucket::ProductionOrphanTag)
        );
        assert_eq!(bucket_of("FR-905"), Some(MentionBucket::Mention));
    }

    /// FR-051-AC-28: a range in an `implements` marker binds
    /// no relation at all, including its own endpoints, and is reported
    /// under `range-in-trace-tag` — the same rule `#[trace(...)]` already
    /// gets, applied to `bind_implements`'s own marker loop.
    #[trace("TC-1936", "FR-051-AC-28")]
    #[test]
    fn tc1936_a_range_in_an_implements_marker_binds_nothing() {
        let mut model = iso_model();
        model.trace_tags.implements.push(TraceMarkerForm {
            syntax: None,
            name: "rust-implements-attribute".to_string(),
            language: SourceLanguage::Rust,
            pattern: r#"#\[implements\(((?:\s*"[^"]*"\s*,?)+)\)\]"#.to_string(),
            template: None,
            entry_paths: None,
        });
        let graph = bind(
            &rs("#[implements(\"FR-001..FR-003\")]\nfn resolve_scope() {\n}\n"),
            &model,
        );
        assert!(
            graph.implements.is_empty(),
            "the range must mint no implements relation: {:?}",
            graph.implements
        );
        assert_eq!(
            graph
                .range_diagnostics
                .iter()
                .map(|r| (r.range_text.as_str(), r.line, r.symbol.as_str()))
                .collect::<Vec<_>>(),
            [("FR-001..FR-003", 1, "resolve_scope")],
            "the range is reported once, on its line, naming its symbol"
        );
        assert!(
            !graph
                .mentions
                .iter()
                .any(|m| m.trace_id == "FR-001" || m.trace_id == "FR-003"),
            "neither endpoint resurfaces as a mention: {:?}",
            graph.mentions
        );
    }

    /// FR-051-AC-28: a range's own endpoints must not
    /// resurface as citations in `mentions` either — "the range finding is
    /// the only record of it".
    #[trace("TC-1936", "FR-051-AC-28")]
    #[test]
    fn tc1936_range_endpoints_do_not_leak_into_mentions() {
        let graph = bind(
            &rs(
                "#[trace(\"FR-001-AC-1..FR-001-AC-2\", \"FR-001-AC-3..4\")]\n#[test]\nfn t() {\n}\n",
            ),
            &iso_model(),
        );
        for id in ["FR-001-AC-1", "FR-001-AC-2", "FR-001-AC-3"] {
            assert!(
                !graph.mentions.iter().any(|m| m.trace_id == id),
                "{id} is a range endpoint and must not appear as a mention: {:?}",
                graph.mentions
            );
        }
        assert_eq!(
            graph.range_diagnostics.len(),
            2,
            "both ranges on the line are reported: {:?}",
            graph.range_diagnostics
        );

        // A marker wrapped over several lines: the range reports the line
        // its own text sits on, and its endpoints still stay out of mentions.
        let graph = bind(
            &rs("#[trace(\n    \"FR-001-AC-1..FR-001-AC-2\",\n)]\n#[test]\nfn t() {\n}\n"),
            &iso_model(),
        );
        assert_eq!(
            graph
                .range_diagnostics
                .iter()
                .map(|r| r.line)
                .collect::<Vec<_>>(),
            [2]
        );
        assert!(graph.mentions.is_empty(), "{:?}", graph.mentions);
    }

    /// CR-187: a range inside a marker that continues past it — only the range
    /// token is inert, and the explicitly named id after it still binds.
    #[trace("TC-1937", "FR-051-AC-28", "FR-050-AC-50")]
    #[test]
    fn tc1937_a_marker_continuing_past_a_range_still_binds_the_rest() {
        let graph = bind(
            &rs("#[trace(\"FR-034-AC-1..FR-034-AC-3\", \"FR-034-AC-7\")]\n#[test]\nfn t() {\n}\n"),
            &iso_model(),
        );
        assert_eq!(
            graph
                .verifies
                .iter()
                .map(|v| v.trace_id.as_str())
                .collect::<Vec<_>>(),
            ["FR-034-AC-7"],
            "the explicitly named id after the range still binds; the range's own \
             endpoints do not"
        );
        assert!(
            !graph
                .unmatched_tags
                .iter()
                .any(|t| t.trace_id == "FR-034-AC-7"),
            "a bound id must not also read as an unmatched tag: {:?}",
            graph.unmatched_tags
        );

        // Ids before, between and after two ranges: every named id binds,
        // neither range's endpoints do, and both ranges report.
        let graph = bind(
            &rs(concat!(
                "#[trace(\"FR-034-AC-9\", \"FR-034-AC-1..FR-034-AC-3\", \"FR-034-AC-7\", ",
                "\"FR-034-AC-10..12\", \"FR-034-AC-20\")]\n#[test]\nfn t() {\n}\n"
            )),
            &iso_model(),
        );
        let bound: Vec<&str> = graph.verifies.iter().map(|v| v.trace_id.as_str()).collect();
        assert_eq!(
            bound,
            ["FR-034-AC-20", "FR-034-AC-7", "FR-034-AC-9"],
            "exactly the explicitly named ids bind"
        );
        let ranges: Vec<&str> = graph
            .range_diagnostics
            .iter()
            .map(|r| r.range_text.as_str())
            .collect();
        assert_eq!(ranges, ["FR-034-AC-1..FR-034-AC-3", "FR-034-AC-10..12"]);
    }

    /// A marker range above a production function binds nothing, is reported
    /// once against that function (not the module container that also spans
    /// it), and keeps its endpoints out of `mentions`.
    #[trace("TC-1937", "FR-051-AC-28")]
    #[test]
    fn tc1937_a_marker_range_on_a_production_function_is_reported() {
        for (label, extraction) in [
            (
                "rust",
                rs("#[trace(\"FR-001-AC-1..FR-001-AC-3\")]\npub fn prod() {\n}\n"),
            ),
            (
                "python",
                py("import os\n\n\n@pytest.mark.trace(\"FR-001-AC-1..FR-001-AC-3\")\ndef prod():\n    return 1\n"),
            ),
        ] {
            let graph = bind(&extraction, &iso_model());
            assert_eq!(
                graph
                    .range_diagnostics
                    .iter()
                    .map(|r| (r.range_text.as_str(), r.symbol.as_str()))
                    .collect::<Vec<_>>(),
                [("FR-001-AC-1..FR-001-AC-3", "prod")],
                "{label}: reported once, against the function"
            );
            assert!(
                !graph
                    .mentions
                    .iter()
                    .any(|m| m.trace_id == "FR-001-AC-1" || m.trace_id == "FR-001-AC-3"),
                "{label}: no endpoint is a mention: {:?}",
                graph.mentions
            );
        }
    }

    /// A marker range on a test inside a container is the test's: one range
    /// record naming the test.
    #[trace("TC-1937", "FR-051-AC-28")]
    #[test]
    fn tc1937_a_marker_range_on_a_contained_test_is_the_tests() {
        let ts = |source: &str| {
            crate::symbols::extract_file("src/m.test.ts", SourceLanguage::Typescript, source)
        };
        for (label, extraction, test) in [
            (
                "rust",
                rs(concat!(
                    "mod tests {\n",
                    "    #[trace(\"FR-001-AC-1..FR-001-AC-3\")]\n",
                    "    #[test]\n",
                    "    fn t() {\n",
                    "    }\n",
                    "}\n",
                )),
                "tests::t",
            ),
            (
                "python",
                py(concat!(
                    "import os\n",
                    "\n",
                    "\n",
                    "class TestThing:\n",
                    "    @pytest.mark.trace(\"FR-001-AC-1..FR-001-AC-3\")\n",
                    "    def test_t(self):\n",
                    "        assert 1\n",
                )),
                "TestThing.test_t",
            ),
            (
                "typescript",
                ts(concat!(
                    "import { it } from \"vitest\";\n",
                    "\n",
                    "describe(\"s\", () => {\n",
                    "  it(\"t\", () => {\n",
                    "    trace(\"FR-001-AC-1..FR-001-AC-3\");\n",
                    "  });\n",
                    "});\n",
                )),
                "t",
            ),
        ] {
            let graph = bind(&extraction, &iso_model());
            assert_eq!(
                graph
                    .range_diagnostics
                    .iter()
                    .map(|r| (r.range_text.as_str(), r.symbol.as_str()))
                    .collect::<Vec<_>>(),
                [("FR-001-AC-1..FR-001-AC-3", test)],
                "{label}: one record, naming the test"
            );
        }
    }

    /// When a container and a function start on the same line and both span a
    /// range on a production symbol, the function is named: a container loses
    /// ties.
    #[trace("TC-1937", "FR-051-AC-28")]
    #[test]
    fn tc1937_a_container_loses_a_tie_for_a_production_range() {
        let graph = bind(
            &py("@pytest.mark.trace(\"FR-001-AC-1..FR-001-AC-3\")\ndef prod():\n    return 1\n"),
            &iso_model(),
        );
        assert_eq!(
            graph
                .range_diagnostics
                .iter()
                .map(|r| r.symbol.as_str())
                .collect::<Vec<_>>(),
            ["prod"]
        );
    }

    /// FR-051-AC-28 for the textual `implements` forms: a range in
    /// `/// Implements: FR-001..FR-003` (Rust) and `# Implements: ...` (Python)
    /// binds no id, mints no relation for either endpoint, and yields one
    /// range finding on the line its text sits on.
    #[trace("TC-2166", "FR-051-AC-28")]
    #[test]
    fn tc2166_a_range_in_a_textual_implements_use_binds_nothing() {
        let mut model = iso_model();
        model.trace_tags.implements.push(TraceMarkerForm {
            syntax: None,
            name: "rust-implements-line".to_string(),
            language: SourceLanguage::Rust,
            pattern: r"(?m)^\s*///\s*Implements:\s*([^\n]+)".to_string(),
            template: None,
            entry_paths: None,
        });
        model.trace_tags.implements.push(TraceMarkerForm {
            syntax: None,
            name: "python-implements-line".to_string(),
            language: SourceLanguage::Python,
            pattern: r"(?m)^\s*#\s*Implements:\s*([^\n]+)".to_string(),
            template: None,
            entry_paths: None,
        });
        for (label, extraction) in [
            (
                "rust",
                rs("/// Implements: FR-001..FR-003\npub fn resolve_scope() {}\n"),
            ),
            (
                "python",
                py("# Implements: FR-001..FR-003\ndef resolve_scope():\n    return 1\n"),
            ),
        ] {
            let graph = bind(&extraction, &model);
            assert!(
                graph.implements.is_empty(),
                "{label}: the range mints no relation: {:?}",
                graph.implements
            );
            assert!(graph.verifies.is_empty(), "{label}");
            assert_eq!(
                graph
                    .range_diagnostics
                    .iter()
                    .map(|r| (r.range_text.as_str(), r.line, r.symbol.as_str()))
                    .collect::<Vec<_>>(),
                [("FR-001..FR-003", 1, "resolve_scope")],
                "{label}: one finding naming the literal range text"
            );
            assert!(
                graph.outside_marker.is_empty(),
                "{label}: the range finding is the only record of it: {:?}",
                graph.outside_marker
            );
        }
    }
}

#[cfg(test)]
mod mask_tests {
    use super::*;
    use ix_trace_rs::trace;

    /// The annotation channel of `src`: comments and Python docstrings kept,
    /// strings and code blanked.
    fn channel(src: &str, language: SourceLanguage) -> String {
        annotation_channel(src, language).expect("not shell")
    }

    #[trace("TC-1055", "FR-051-AC-24")]
    #[test]
    fn tc1055_an_id_inside_a_string_is_masked_in_every_language() {
        // `agent-ix/quire-rs#323`. A trace id a file carries as DATA is not an
        // authored tag, and reporting it sends a reader to rewrite data. The
        // mask was gated on `SourceLanguage::Rust`; it covers every language.
        let rust = channel("let s = \"// TC-002\";\n", SourceLanguage::Rust);
        assert!(!rust.contains("TC-002"), "a Rust string is data: {rust:?}");
        let python = channel(
            "x = \"\"\"\n    Trace: TC-002\n\"\"\"\n",
            SourceLanguage::Python,
        );
        assert!(
            !python.contains("TC-002"),
            "an assigned literal is data, not a tag: {python:?}"
        );
        let ts = channel(
            "const x = `\n  Trace: TC-002\n`;\n",
            SourceLanguage::Typescript,
        );
        assert!(
            !ts.contains("TC-002"),
            "a template literal is data, not a tag: {ts:?}"
        );
    }

    #[trace("TC-1056", "FR-051-AC-24")]
    #[test]
    fn tc1056_a_python_docstring_is_the_annotation_channel_there() {
        // Python has no doc-comment syntax: its annotation channel IS a string
        // literal, where Rust's is `///` and TypeScript's is a block comment.
        let doc = channel(
            "def test_x():\n    \"\"\"TC-001: the declared severity.\"\"\"\n    pass\n",
            SourceLanguage::Python,
        );
        assert!(
            doc.contains("TC-001"),
            "a docstring is Python's comment: {doc:?}"
        );
        // A TypeScript registration title is an ordinary string: its name is not
        // an annotation.
        let title = channel(
            "it(\"TC-001 binds\", () => {});\n",
            SourceLanguage::Typescript,
        );
        assert!(!title.contains("TC-001"), "{title:?}");
        // And the exemption is POSITIONAL: the same text assigned mid-line is
        // masked.
        let assigned = channel(
            "    x = \"\"\"TC-001: not documentation.\"\"\"\n",
            SourceLanguage::Python,
        );
        assert!(
            !assigned.contains("TC-001"),
            "opened mid-line, so it is a value: {assigned:?}"
        );
    }

    #[trace("TC-1057", "FR-051-AC-24")]
    #[test]
    fn tc1057_comments_survive_and_offsets_do_not_shift() {
        // Comments are the annotation channel everywhere. Byte length and line
        // count are preserved so a reported `file:line` is a line in the
        // original.
        for (language, src) in [
            (
                SourceLanguage::Python,
                "def test_x():\n    # Trace: TC-001\n    y = \"Trace: TC-002\"\n",
            ),
            (
                SourceLanguage::Rust,
                "fn t() {\n    // Trace: TC-001\n    let y = \"Trace: TC-002\";\n}\n",
            ),
            (
                SourceLanguage::Typescript,
                "function t() {\n  // Trace: TC-001\n  const y = \"Trace: TC-002\";\n}\n",
            ),
        ] {
            let out = channel(src, language);
            assert!(
                out.contains("TC-001"),
                "{language:?}: comment preserved: {out:?}"
            );
            assert!(
                !out.contains("TC-002"),
                "{language:?}: string masked: {out:?}"
            );
            assert_eq!(out.len(), src.len(), "{language:?}: byte length preserved");
            assert_eq!(out.lines().count(), src.lines().count(), "{language:?}");
        }
    }

    #[trace("TC-1055", "FR-051-AC-24")]
    #[test]
    fn a_char_literal_holding_a_quote_does_not_open_a_string() {
        // `'"'` is a char literal. Read as a string opener it would swallow the
        // comment on the next line.
        let out = channel("let q = '\"';\n// TC-001\n", SourceLanguage::Rust);
        assert!(out.contains("TC-001"), "{out:?}");
    }
}
