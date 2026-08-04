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
        // Each term contributes both its singular and its plural form, so
        // corpus objects match regardless of number — including irregular
        // plurals (`policy`→`policies`, `box`→`boxes`).
        let mut forms: Vec<String> = Vec::new();
        for t in terms {
            let t = t.trim();
            if t.is_empty() {
                continue;
            }
            forms.push(regex::escape(t));
            forms.push(regex::escape(&pluralize(t)));
        }
        let matcher = if forms.is_empty() {
            None
        } else {
            Regex::new(&format!(r"(?i)\b({})\b", forms.join("|"))).ok()
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

/// Naive English pluralizer for lexicon terms: consonant+`y`→`ies`, sibilant
/// (`s`/`x`/`z`/`ch`/`sh`)→`es`, else `+s`. Lexicon terms are lowercase ASCII
/// nouns, so byte handling is safe.
fn pluralize(term: &str) -> String {
    let lower = term.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let n = bytes.len();
    if n >= 2 && bytes[n - 1] == b'y' && !b"aeiou".contains(&bytes[n - 2]) {
        format!("{}ies", &term[..n - 1])
    } else if lower.ends_with('s')
        || lower.ends_with('x')
        || lower.ends_with('z')
        || lower.ends_with("ch")
        || lower.ends_with("sh")
    {
        format!("{term}es")
    } else {
        format!("{term}s")
    }
}

/// The shared **all-default** severity map (FR-048-AC-7) for the registry-free
/// paths: every check keeps its default `warning` level.
pub fn default_severity() -> &'static GrammarSeverityMap {
    static DEFAULT: OnceLock<GrammarSeverityMap> = OnceLock::new();
    DEFAULT.get_or_init(GrammarSeverityMap::new)
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

/// A configured severity **level** for one `<grammar>:<check>` pair (FR-048).
///
/// Distinct from [`GrammarSeverity`], which is the severity a *finding*
/// carries: `Off` is a configuration state with no finding counterpart — an
/// `off`-mapped check is suppressed entirely (no warning, no error, no
/// `--summary` entry). Modules declare this vocabulary in their manifest
/// `grammar_severity` registry; the CLI `--severity` flag overrides it per key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrammarSeverityLevel {
    Off,
    Warning,
    Error,
}

impl GrammarSeverityLevel {
    /// Stable machine-readable string for surfaces (CLI, wheel).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// The merged per-check severity registry (FR-048): `<grammar>:<check>` →
/// level. `BTreeMap` because the merge order and any diagnostic derived from it
/// are observable (NFR-006). An absent key means `warning`, so the empty map is
/// the **all-default** map applied by the registry-free paths.
pub type GrammarSeverityMap = std::collections::BTreeMap<String, GrammarSeverityLevel>;

/// The registry key for a `(grammar, check)` pair — the `<grammar>:<check>`
/// form modules and the `--severity` CLI flag both author.
pub fn severity_key(grammar: &str, check: &str) -> String {
    format!("{grammar}:{check}")
}

/// The configured level for `(grammar, check)`, defaulting to `warning` when
/// the key is absent from `severity` (FR-048-AC-4).
pub fn severity_level(
    severity: &GrammarSeverityMap,
    grammar: &str,
    check: &str,
) -> GrammarSeverityLevel {
    severity
        .get(&severity_key(grammar, check))
        .copied()
        .unwrap_or(GrammarSeverityLevel::Warning)
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

/// Apply the merged `grammar_severity` map (FR-048) to freshly emitted
/// `findings`: each finding's severity is set from the map keyed by
/// `<grammar>:<check>`, defaulting to `warning` when the key is absent, and an
/// `off`-mapped finding is **dropped here** — before routing — so it appears
/// neither in `warnings`/`errors` nor in any downstream `--summary` histogram
/// (FR-048-AC-9). Input order is preserved (NFR-006).
pub fn apply_severity(
    findings: Vec<GrammarFinding>,
    severity: &GrammarSeverityMap,
) -> Vec<GrammarFinding> {
    findings
        .into_iter()
        .filter_map(
            |mut f| match severity_level(severity, &f.grammar, &f.check) {
                GrammarSeverityLevel::Off => None,
                GrammarSeverityLevel::Warning => {
                    f.severity = GrammarSeverity::Warning;
                    Some(f)
                }
                GrammarSeverityLevel::Error => {
                    f.severity = GrammarSeverity::Error;
                    Some(f)
                }
            },
        )
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(grammar: &str, check: &str) -> GrammarFinding {
        GrammarFinding {
            grammar: grammar.to_string(),
            check: check.to_string(),
            pattern: None,
            message: "m".to_string(),
            line: Some(1),
            statement: "s".to_string(),
            severity: GrammarSeverity::Warning,
        }
    }

    // TC-719 (FR-048-AC-4): a finding whose `<grammar>:<check>` key is absent
    // from the merged map defaults to `warning`.
    #[test]
    fn tc719_absent_key_defaults_to_warning() {
        let mut map = GrammarSeverityMap::new();
        // A sibling key is mapped; the finding's own key is not.
        map.insert("ac:unclassifiable".into(), GrammarSeverityLevel::Error);
        let out = apply_severity(vec![finding("ears", "vague-response")], &map);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].severity, GrammarSeverity::Warning);
    }

    // TC-752 (FR-048-AC-9): an `off`-mapped check records no finding at all,
    // while sibling checks of the same grammar still report.
    #[test]
    fn tc752_off_drops_the_finding_before_routing() {
        let mut map = GrammarSeverityMap::new();
        map.insert("ac:vague-response".into(), GrammarSeverityLevel::Off);
        let out = apply_severity(
            vec![
                finding("ac", "vague-response"),
                finding("ac", "non-singular"),
                finding("ears", "vague-response"),
            ],
            &map,
        );
        let keys: Vec<String> = out
            .iter()
            .map(|f| format!("{}:{}", f.grammar, f.check))
            .collect();
        assert_eq!(keys, vec!["ac:non-singular", "ears:vague-response"]);
    }

    // FR-048: an `error` mapping promotes the emitted finding's severity.
    #[test]
    fn error_level_promotes_finding_severity() {
        let mut map = GrammarSeverityMap::new();
        map.insert("ac:unclassifiable".into(), GrammarSeverityLevel::Error);
        let out = apply_severity(vec![finding("ac", "unclassifiable")], &map);
        assert_eq!(out[0].severity, GrammarSeverity::Error);
    }
}
