//! Requirement-grammar checking (FR-042).
//!
//! A *grammar* evaluates the natural-language requirement statements inside
//! requirement-bearing artifacts. This is a posture distinct from declarative
//! lint ([`crate::lint`], FR-036) and structural validation
//! ([`crate::validate_document`], FR-032): a grammar classifies and checks the
//! *prose of individual normative statements*, and its findings carry a
//! [`GrammarSeverity`] that routes them into the validation result.
//!
//! Archetypes bind to a grammar **bundle** via the manifest `grammar_ref`
//! field (e.g. `grammar_ref: iso-spec-core`). The framework holds a registry of
//! bundles by name and dispatches the right sub-grammar by
//! `(archetype, section)` inside the bundle. v1 ships the `iso-spec-core`
//! bundle implementing **EARS** for FR/NFR/StR normative sections; GWT (for
//! acceptance criteria) and the `US` story grammar register onto the same
//! framework later.

pub mod ears;

use std::sync::OnceLock;

use regex::Regex;

use crate::ast::QuireDocument;

/// A precompiled matcher over the merged module `lexicon` (FR-043): the set of
/// accepted **concrete terms** the EARS vague-response check (FR-042) treats as
/// verifiable objects. Built once from the `Registry`'s merged lexicon; the
/// engine carries no hardcoded concrete-noun list. An empty lexicon (the
/// type-only `validate_document` path, or `check_grammar` without a module)
/// matches nothing — generic mechanism/bound/backtick suppression still applies.
#[derive(Debug, Default)]
pub struct GrammarLexicon {
    matcher: Option<Regex>,
}

impl GrammarLexicon {
    /// Build a matcher from concrete terms (the merged lexicon keys). Terms are
    /// matched case-insensitively at word boundaries. An empty term set yields a
    /// matcher that recognises nothing.
    pub fn from_terms<'a, I: IntoIterator<Item = &'a str>>(terms: I) -> Self {
        let escaped: Vec<String> = terms
            .into_iter()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(regex::escape)
            .collect();
        let matcher = if escaped.is_empty() {
            None
        } else {
            Regex::new(&format!(r"(?i)\b({})\b", escaped.join("|"))).ok()
        };
        Self { matcher }
    }

    /// The empty lexicon — recognises no concrete term.
    pub fn empty() -> Self {
        Self { matcher: None }
    }

    /// True when `text` contains an accepted concrete term (word-boundary).
    pub fn contains_term(&self, text: &str) -> bool {
        self.matcher.as_ref().is_some_and(|r| r.is_match(text))
    }
}

/// The shared empty lexicon for the registry-free paths.
pub fn empty_lexicon() -> &'static GrammarLexicon {
    static EMPTY: OnceLock<GrammarLexicon> = OnceLock::new();
    EMPTY.get_or_init(GrammarLexicon::empty)
}

/// Severity of a grammar finding. `Warning` is advisory (never fails
/// validation); `Error` blocks. Severity is **policy**: a deployment may
/// promote a grammar from advisory to enforcing without a code change — the
/// migration lever for adopting EARS across the corpus (FR-042).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammarSeverity {
    Warning,
    Error,
}

impl GrammarSeverity {
    /// Stable machine-readable string for surfaces (CLI, wheel).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// One grammar diagnostic against a single normative statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarFinding {
    /// Grammar bundle that produced the finding (e.g. `ears`).
    pub grammar: String,
    /// Stable check id (e.g. `non-singular`, `vague-response`).
    pub check: String,
    /// Classification label for the statement, when the grammar classifies it
    /// (e.g. an EARS pattern: `ubiquitous`, `event`, …, `unclassifiable`).
    pub pattern: Option<String>,
    /// Human-readable message.
    pub message: String,
    /// 1-based document line of the offending statement, when known.
    pub line: Option<usize>,
    /// The offending statement excerpt.
    pub statement: String,
    /// Severity (advisory `Warning` by default in v1).
    pub severity: GrammarSeverity,
}

/// Run the grammar bundle named `grammar_ref` against `doc`, dispatching by
/// `archetype` (the **resolved** archetype name, so a `--archetype` override on
/// a typeless document is still checked). An unknown bundle name yields no
/// findings — grammar checking is advisory by construction and never errors on
/// an unrecognised binding (a module may declare a `grammar_ref` this engine
/// build does not implement).
///
/// `line_offset` is the number of leading lines (frontmatter + fence) before
/// the document body, so findings can carry 1-based document line numbers.
pub fn check_document_grammar(
    grammar_ref: &str,
    archetype: &str,
    doc: &QuireDocument,
    line_offset: usize,
    lexicon: &GrammarLexicon,
) -> Vec<GrammarFinding> {
    match grammar_ref {
        "iso-spec-core" => ears::check(archetype, doc, line_offset, lexicon),
        _ => Vec::new(),
    }
}
