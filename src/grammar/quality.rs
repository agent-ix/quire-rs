//! Requirement-quality lints (FR-056) — the ISO 29148 quality characteristics,
//! as mechanical checks on the FR-042 grammar framework.
//!
//! The `ears` grammar judges whether a statement matches an EARS *pattern*; the
//! `ac` grammar judges the shape of an acceptance criterion. Neither asks
//! whether the statement is *well-formed as a requirement* — unambiguous,
//! verifiable, singular, and clear about who acts.
//!
//! These ship as a **separate grammar id** rather than as more `ears:` checks,
//! deliberately. They are a different judgement — 29148 quality rather than
//! EARS conformance — and FR-048 keys severity on `<grammar>:<check>`, so a
//! deployment can silence or promote the quality pack without touching pattern
//! conformance. Bundling them under `ears` would have made those two decisions
//! one lever.
//!
//! Every check is **advisory** (`Warning`) on arrival, per the FR-047-CON-1
//! pattern: promotion to `error` is a corpus sweep plus explicit sign-off, not
//! a default. CR-014 is the reason — a check that fires on half the corpus is a
//! check whose precision argument has to be written down either way, and the
//! one that was not measured first cost a retirement.

use std::sync::OnceLock;

use regex::Regex;

use super::ears::{abs_line, contains_shall, excerpt, normalize};
use super::{GrammarFinding, GrammarSeverity};
use crate::ast::QuireSection;
use crate::query;

/// Advisory on arrival (FR-056-CON-1).
const DEFAULT_SEVERITY: GrammarSeverity = GrammarSeverity::Warning;

/// The engine's built-in ambiguity lexicon: words whose presence in a normative
/// statement leaves a reader unable to say what would satisfy it.
///
/// Closed in the engine, **extended** by a module's `ambiguity_terms` registry
/// — the `vacuous_predicates` arrangement (ADR 0009), for the same reason: the
/// engine ships a defensible core and a deployment adds its own house words
/// without a code change.
///
/// Deliberately small. CR-014 retired `no-observable-outcome` because an open
/// set whose membership was *required* to earn a label reached ~13% sampled
/// precision; every term here is one where the ambiguity is in the word itself,
/// not in a guess about the sentence around it.
const BUILTIN_AMBIGUOUS: &[&str] = &[
    // Unquantified adequacy — the classic 29148 examples.
    "adequate",
    "appropriate",
    "sufficient",
    "reasonable",
    "acceptable",
    "as needed",
    "as appropriate",
    "as required",
    "if possible",
    "if practical",
    "where appropriate",
    // Unverifiable quality adjectives.
    "user-friendly",
    "easy to use",
    "robust",
    "flexible",
    "seamless",
    "efficient",
    "effective",
    "optimal",
    // Unbounded directions of travel — a target with no number.
    "minimize",
    "maximize",
    "optimize",
    "improve",
    "enhance",
    // Superlatives with no baseline.
    "best",
    "fastest",
    "state-of-the-art",
    // Coordination that leaves the obligation undetermined.
    "and/or",
    "etc",
    "and so on",
];

/// One matched ambiguous term and where it sits.
struct Stmt {
    text: String,
    line: Option<usize>,
}

/// Sections this grammar binds, per archetype. Identical to the `ears` binding
/// by design: the two grammars judge the same statements on different axes, and
/// a statement that is worth pattern-checking is worth quality-checking.
fn bound_sections(archetype: &str) -> &'static [(&'static str, Option<&'static str>)] {
    match archetype {
        "FR" => &[
            ("Description", None),
            ("Behavior", None),
            ("Constraints", Some("Constraint")),
        ],
        "NFR" => &[("Statement", None)],
        "StR" => &[("Stakeholder Need", None)],
        _ => &[],
    }
}

/// Check `doc` against the requirement-quality pack.
///
/// `ambiguous` is the merged module registry layered over [`BUILTIN_AMBIGUOUS`].
pub fn check(
    archetype: &str,
    doc: &crate::ast::QuireDocument,
    line_offset: usize,
    ambiguous: &AmbiguityTerms,
) -> Vec<GrammarFinding> {
    let mut findings = Vec::new();
    for (heading, table_column) in bound_sections(archetype) {
        let Some(section) = query::section(doc, heading) else {
            continue;
        };
        let stmts = match table_column {
            Some(col) => table_statements(section, line_offset, col),
            None => prose_statements(section, line_offset),
        };
        for stmt in &stmts {
            check_statement(stmt, ambiguous, &mut findings);
        }
    }
    findings
}

fn check_statement(stmt: &Stmt, ambiguous: &AmbiguityTerms, out: &mut Vec<GrammarFinding>) {
    let push = |out: &mut Vec<GrammarFinding>, check: &str, message: String| {
        out.push(GrammarFinding {
            grammar: "quality".to_string(),
            check: check.to_string(),
            pattern: None,
            message,
            line: stmt.line,
            statement: excerpt(&stmt.text),
            severity: DEFAULT_SEVERITY,
        });
    };

    // CR-017 parity: a term inside an inline code span is a **mention**, not a
    // use. `The manifest key \`optimize\` is rejected` is not an ambiguous
    // requirement, and reading the raw text would call it one.
    let masked = super::ac::mask_code_spans(&normalize(&stmt.text));

    if let Some(term) = ambiguous.first_match(&masked) {
        push(
            out,
            "ambiguous-term",
            format!(
                "ambiguous term `{term}`; state the quantity, threshold or condition that \
                 would satisfy this requirement"
            ),
        );
    }

    if let Some(verb) = agentless_passive(&masked) {
        push(
            out,
            "agentless-passive",
            format!(
                "`shall be {verb}` names no agent; say which component performs it, or the \
                 requirement allocates to nobody"
            ),
        );
    }

    if let Some((a, b)) = mixed_modal(&masked) {
        push(
            out,
            "mixed-modal",
            format!(
                "statement mixes `{a}` and `{b}`; one statement carries one obligation \
                 strength, or a reader cannot tell which part is binding"
            ),
        );
    }
}

/// The merged ambiguity lexicon: engine built-ins plus module-declared terms.
///
/// Terms are matched case-insensitively on **word boundaries**, so `optimize`
/// does not fire on `optimized`… which it should. The matcher therefore also
/// accepts a trailing inflection (`-s`, `-d`, `-ed`, `-ing`), the same
/// treatment `vacuous_predicates` gives its predicates.
#[derive(Debug, Clone, Default)]
pub struct AmbiguityTerms {
    terms: Vec<String>,
}

impl AmbiguityTerms {
    /// Build from module-declared terms layered **over** the built-ins, so a
    /// module extends the lexicon rather than replacing it.
    pub fn with_module_terms<'a>(module: impl Iterator<Item = &'a str>) -> Self {
        let mut terms: Vec<String> = BUILTIN_AMBIGUOUS.iter().map(|t| t.to_string()).collect();
        for term in module {
            let normalized = term.trim().to_lowercase();
            if !normalized.is_empty() && !terms.iter().any(|t| t == &normalized) {
                terms.push(normalized);
            }
        }
        // Longest first, so `as appropriate` reports as itself rather than as
        // the `appropriate` inside it.
        terms.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        Self { terms }
    }

    /// The built-in lexicon alone.
    pub fn builtin() -> Self {
        Self::with_module_terms(std::iter::empty())
    }

    /// The first declared term the statement uses, if any.
    fn first_match(&self, statement: &str) -> Option<&str> {
        let lower = statement.to_lowercase();
        self.terms
            .iter()
            .find(|term| contains_word(&lower, term))
            .map(String::as_str)
    }

    /// Number of terms in the merged lexicon (built-ins included).
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

/// Whole-word containment with a small inflection tail, ASCII-cased.
fn contains_word(haystack: &str, needle: &str) -> bool {
    let mut from = 0usize;
    while let Some(idx) = haystack[from..].find(needle) {
        let start = from + idx;
        let end = start + needle.len();
        let before_ok = start == 0
            || !haystack[..start]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric());
        // Allow a short inflection tail so `optimizes` / `minimized` fire.
        let tail: String = haystack[end..]
            .chars()
            .take_while(|c| c.is_alphabetic())
            .collect();
        let after_ok = matches!(tail.as_str(), "" | "s" | "d" | "ed" | "ing" | "es");
        if before_ok && after_ok {
            return true;
        }
        from = start + needle.len().max(1);
        if from >= haystack.len() {
            break;
        }
    }
    false
}

/// `shall be <past participle>` with no `by <agent>` anywhere after it.
///
/// The requirement says something must happen and allocates it to nobody, which
/// is the allocation defect 29148 names — not merely a style preference about
/// the passive voice. A trailing `by …` makes the agent explicit, so it passes.
fn agentless_passive(statement: &str) -> Option<String> {
    let lower = statement.to_lowercase();
    let caps = re_passive().captures(&lower)?;
    let verb = caps.get(1)?.as_str().to_string();
    let rest = &lower[caps.get(0)?.end()..];
    if re_by_agent().is_match(rest) {
        return None;
    }
    Some(verb)
}

/// Two different modal verbs in one statement.
///
/// `shall` + `should` leaves a reader unable to say which half is binding.
/// Reported as the pair rather than as "avoid `should`", because a `should`
/// statement on its own is a legitimate recommendation.
fn mixed_modal(statement: &str) -> Option<(&'static str, &'static str)> {
    let lower = statement.to_lowercase();
    let present: Vec<&'static str> = [
        ("shall", re_shall_w()),
        ("should", re_should()),
        ("may", re_may()),
        ("must", re_must()),
    ]
    .into_iter()
    .filter(|(_, re)| re.is_match(&lower))
    .map(|(name, _)| name)
    .collect();
    (present.len() >= 2).then(|| (present[0], present[1]))
}

// ─── Statement collection (mirrors `ears`, which owns the originals) ────────

fn prose_statements(section: &QuireSection, line_offset: usize) -> Vec<Stmt> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for (rel, raw_line) in section.content.lines().enumerate() {
        let trimmed = raw_line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.starts_with('>') || trimmed.starts_with('|') {
            continue;
        }
        let text = normalize(trimmed);
        if !contains_shall(&text.to_lowercase()) && !re_should().is_match(&text.to_lowercase()) {
            continue;
        }
        out.push(Stmt {
            text,
            line: Some(abs_line(section, rel, line_offset)),
        });
    }
    out
}

fn table_statements(section: &QuireSection, line_offset: usize, column: &str) -> Vec<Stmt> {
    let Some(table) = query::parse_table(&section.content) else {
        return Vec::new();
    };
    let Some(col_idx) = table
        .headers
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case(column))
    else {
        return Vec::new();
    };
    table
        .rows
        .iter()
        .filter_map(|row| {
            let cell = row.get(col_idx)?.trim();
            (!cell.is_empty()).then(|| Stmt {
                text: normalize(cell),
                line: Some(line_offset + section.start_line + 1),
            })
        })
        .collect()
}

// ─── Compiled once ──────────────────────────────────────────────────────────

fn re_passive() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\bshall\s+be\s+([a-z]+(?:ed|en))\b").expect("passive regex"))
}

fn re_by_agent() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\bby\s+(?:the\s+|a\s+|an\s+)?\w").expect("by-agent regex"))
}

fn re_shall_w() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\bshall\b").expect("shall regex"))
}

fn re_should() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\bshould\b").expect("should regex"))
}

fn re_may() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\bmay\b").expect("may regex"))
}

fn re_must() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\bmust\b").expect("must regex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_lexicon_matches_whole_words_and_inflections() {
        let lex = AmbiguityTerms::builtin();
        assert_eq!(
            lex.first_match("the system shall minimize latency"),
            Some("minimize")
        );
        assert_eq!(
            lex.first_match("the system minimizes latency"),
            Some("minimize")
        );
        // Not a substring hit: `optimizer` is a noun, not the unbounded verb.
        assert_eq!(lex.first_match("the optimizerbank shall run"), None);
    }

    #[test]
    fn longest_term_wins_so_the_report_names_what_was_written() {
        let lex = AmbiguityTerms::builtin();
        assert_eq!(
            lex.first_match("the system shall retry as appropriate"),
            Some("as appropriate"),
        );
    }

    #[test]
    fn module_terms_extend_rather_than_replace() {
        let lex = AmbiguityTerms::with_module_terms(["snappy"].into_iter());
        assert_eq!(lex.first_match("the ui shall feel snappy"), Some("snappy"));
        // The built-ins survive the extension.
        assert_eq!(lex.first_match("shall be adequate"), Some("adequate"));
        assert!(lex.len() > BUILTIN_AMBIGUOUS.len());
    }

    #[test]
    fn agentless_passive_needs_an_absent_agent() {
        assert_eq!(
            agentless_passive("the input shall be validated"),
            Some("validated".to_string()),
        );
        // An explicit agent is the whole point of the check, so it passes.
        assert_eq!(
            agentless_passive("the input shall be validated by the parser"),
            None,
        );
        assert_eq!(
            agentless_passive("the parser shall validate the input"),
            None
        );
    }

    #[test]
    fn mixed_modal_reports_the_pair() {
        assert_eq!(
            mixed_modal("the system shall retry and should log"),
            Some(("shall", "should"))
        );
        // One modal is not a defect: a `should` on its own is a recommendation.
        assert_eq!(mixed_modal("the system should log"), None);
        assert_eq!(mixed_modal("the system shall retry"), None);
    }
}
