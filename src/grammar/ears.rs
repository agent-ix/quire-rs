//! The EARS grammar (FR-042) — first grammar in the `iso-spec-core` bundle.
//!
//! EARS (Easy Approach to Requirements Syntax) classifies each normative
//! statement into one of six patterns and checks the clause ruleset. It runs on
//! the requirement-bearing sections of FR / NFR / StR (the `(archetype,
//! section)` binding below); all other archetypes and sections are ignored.
//!
//! Dialects (FR-042): an enumerated `The X SHALL:` stem followed by a numbered
//! response list is one statement (the list items carry no `shall`, so
//! segmentation never inflates the count); a `StR` statement accepts a
//! stakeholder subject; an `NFR` statement is not pushed toward triggers
//! (absence of a trigger is never a defect — no check penalises it).

use std::sync::OnceLock;

use regex::Regex;

use super::{GrammarFinding, GrammarLexicon, GrammarSeverity};
use crate::ast::QuireSection;
use crate::query;

/// The six EARS patterns plus the `Unclassifiable` catch-all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarsPattern {
    Ubiquitous,
    Event,
    State,
    Unwanted,
    Optional,
    Complex,
    Unclassifiable,
}

impl EarsPattern {
    /// Stable machine-readable label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ubiquitous => "ubiquitous",
            Self::Event => "event",
            Self::State => "state",
            Self::Unwanted => "unwanted",
            Self::Optional => "optional",
            Self::Complex => "complex",
            Self::Unclassifiable => "unclassifiable",
        }
    }
}

/// v1 default severity: advisory. Promotion to `Error` is a config/policy lever
/// (FR-042); the engine ships warning-only.
const DEFAULT_SEVERITY: GrammarSeverity = GrammarSeverity::Warning;

/// The `(archetype, section)` binding: which sections of each requirement-
/// bearing archetype carry normative statements. The bool marks a table-valued
/// section whose statements are the cells of the named column.
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

/// A normative statement extracted from a section, with its 1-based document
/// line.
struct Stmt {
    text: String,
    line: Option<usize>,
}

/// Check `doc` (an `archetype` document) against the EARS grammar, returning one
/// finding per clause violation and per unclassifiable statement.
pub fn check(
    archetype: &str,
    doc: &crate::ast::QuireDocument,
    line_offset: usize,
    lexicon: &GrammarLexicon,
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
        for stmt in stmts {
            check_statement(archetype, &stmt, lexicon, &mut findings);
        }
    }
    findings
}

fn check_statement(
    archetype: &str,
    stmt: &Stmt,
    lexicon: &GrammarLexicon,
    out: &mut Vec<GrammarFinding>,
) {
    let pattern = classify(&stmt.text);
    let label = Some(pattern.as_str().to_string());

    let push = |out: &mut Vec<GrammarFinding>, check: &str, message: String| {
        out.push(GrammarFinding {
            grammar: "ears".to_string(),
            check: check.to_string(),
            pattern: label.clone(),
            message,
            line: stmt.line,
            statement: excerpt(&stmt.text),
            severity: DEFAULT_SEVERITY,
        });
    };

    if pattern == EarsPattern::Unclassifiable {
        push(
            out,
            "unclassifiable",
            "statement does not match any EARS pattern (ubiquitous / event / state / unwanted / optional / complex)".to_string(),
        );
    }
    if shall_count(&stmt.text) > 1 {
        push(
            out,
            "non-singular",
            "statement contains more than one `shall`; split into one requirement per `shall`"
                .to_string(),
        );
    }
    if has_missing_subject(&stmt.text) {
        push(
            out,
            "missing-subject",
            "statement names no system/actor subject before `shall`".to_string(),
        );
    }
    if let Some(verb) = vague_verb(&stmt.text, lexicon) {
        push(
            out,
            "vague-response",
            format!("vague response verb `{verb}`; name a concrete, verifiable response"),
        );
    }
    // NFR dialect: absence of a trigger is never a defect, and no check
    // penalises it. The non-canonical-trigger check only fires on a *present*
    // non-EARS trigger, so it applies uniformly.
    let _ = archetype;
    if let Some(trigger) = non_canonical_trigger(&stmt.text) {
        push(
            out,
            "non-canonical-trigger",
            format!("non-EARS trigger `{trigger}`; use the canonical `When …` (event) or `While …` (state)"),
        );
    }
}

// ─── Classification ─────────────────────────────────────────────────────────

/// Classify a single normative statement into an [`EarsPattern`].
pub fn classify(statement: &str) -> EarsPattern {
    let norm = normalize(statement);
    let lower = norm.to_lowercase();
    if !contains_shall(&lower) {
        return EarsPattern::Unclassifiable;
    }

    // Complex: two or more distinct trigger keywords present. Token-based
    // membership (no per-call regex compile — this runs once per statement).
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphabetic())
        .filter(|w| !w.is_empty())
        .collect();
    let triggers = ["when", "while", "where", "if"]
        .iter()
        .filter(|kw| words.contains(kw))
        .count();
    if triggers >= 2 {
        return EarsPattern::Complex;
    }

    let first = first_word(&lower);
    match first.as_str() {
        "when" => EarsPattern::Event,
        "while" => EarsPattern::State,
        "where" => EarsPattern::Optional,
        "if" => EarsPattern::Unwanted,
        _ => {
            // No leading trigger: ubiquitous when there is a subject before
            // `shall`, otherwise unclassifiable.
            if has_missing_subject(statement) {
                EarsPattern::Unclassifiable
            } else {
                EarsPattern::Ubiquitous
            }
        }
    }
}

// ─── Clause checks ──────────────────────────────────────────────────────────

fn shall_count(statement: &str) -> usize {
    re_shall().find_iter(&statement.to_lowercase()).count()
}

/// True when the statement names no subject before `shall` in its main clause.
/// The leading trigger clause (up to the first comma) is stripped first, and an
/// `if … then …` lead-in is reduced to its main clause, so `When X, the system
/// shall …` and `If X, then the system shall …` both keep `the system` as the
/// subject.
fn has_missing_subject(statement: &str) -> bool {
    let norm = normalize(statement);
    let lower = norm.to_lowercase();
    let main = main_clause(&lower);
    let Some(idx) = main.find("shall") else {
        return false; // no shall in main clause — not this check's concern
    };
    let before = main[..idx].trim();
    let before = before.strip_prefix("then").unwrap_or(before).trim();
    before.is_empty()
}

/// Detect a vague response verb — **object-aware** (FR-042) over the
/// module-supplied concrete `lexicon` (FR-043). A verb from the weak-verb set is
/// only vague when its *object* is abstract or absent: a mechanism/quantitative
/// qualifier (`handle X by Y`, `process within 16 ms`), a backticked identifier,
/// or a **lexicon term** (`provide an endpoint`, `support pagination`) states a
/// verifiable response and is suppressed. The engine carries no hardcoded
/// concrete-noun list — concrete terms come from the lexicon. `be able to` is
/// the one verb-intrinsic case (capability phrasing) and is always flagged.
fn vague_verb(statement: &str, lexicon: &GrammarLexicon) -> Option<String> {
    let lower = normalize(statement).to_lowercase();
    let caps = re_vague().captures(&lower)?;
    let m = caps.get(2)?;
    let verb = m.as_str().split_whitespace().collect::<Vec<_>>().join(" ");
    judge_vague_verb(&verb, &lower[m.end()..], lexicon)
}

/// The same object-aware judgement over a **bare outcome clause** — no `shall`
/// anchor. The `ac` grammar (FR-047) checks the response clause of an
/// `ears`-shaped criterion and the `Then` clause of a `given-when-then`-shaped
/// one, where the modal verb has already been consumed; this keeps one
/// vague-verb implementation across the two grammars (FR-047 check 3).
pub(super) fn vague_verb_in_clause(clause: &str, lexicon: &GrammarLexicon) -> Option<String> {
    let lower = normalize(clause).to_lowercase();
    let m = re_vague_bare().find(&lower)?;
    let verb = m.as_str().split_whitespace().collect::<Vec<_>>().join(" ");
    judge_vague_verb(&verb, &lower[m.end()..], lexicon)
}

/// Shared object-awareness: given a matched weak `verb` and the `remainder` of
/// the clause after it, decide whether the response is verifiable. `be able to`
/// is the one verb-intrinsic case (capability phrasing) and is always vague.
fn judge_vague_verb(verb: &str, remainder: &str, lexicon: &GrammarLexicon) -> Option<String> {
    if verb == "be able to" {
        return Some(verb.to_string()); // capability-not-behavior — always vague
    }
    if re_concrete_qualifier().is_match(remainder) {
        return None; // mechanism / numeric / ordering / backtick qualifier → concrete
    }
    if re_vague_quality().is_match(remainder) {
        return Some(verb.to_string()); // abstract quality/manner object → vague
    }
    if lexicon.contains_term(remainder) {
        return None; // a module-declared concrete term → verifiable
    }
    Some(verb.to_string()) // bare / generic response → vague
}

fn non_canonical_trigger(statement: &str) -> Option<String> {
    let lower = normalize(statement).to_lowercase();
    if !contains_shall(&lower) {
        return None;
    }
    let first = first_word(&lower);
    matches!(
        first.as_str(),
        "on" | "upon" | "after" | "before" | "once" | "during"
    )
    .then_some(first)
}

// ─── Statement extraction ───────────────────────────────────────────────────

/// Segment a prose section into normative statements. v1 is line-based: each
/// kept line is split on sentence boundaries and fragments bearing a modal verb
/// become statements. A statement wrapped across multiple source lines is
/// therefore segmented at its first line — this can *under*-count (a second
/// `shall` or vague verb on a continuation line is missed), never over-flag a
/// continuation line (those carry no `shall`). Conservative for an advisory
/// check; paragraph-joining is a planned refinement.
fn prose_statements(section: &QuireSection, line_offset: usize) -> Vec<Stmt> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for (rel, raw_line) in section.content.lines().enumerate() {
        let trimmed = raw_line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.starts_with('>') || trimmed.starts_with('#') {
            continue;
        }
        let line = strip_list_marker(trimmed);
        for fragment in line.split(". ") {
            if contains_shall(&fragment.to_lowercase()) {
                out.push(Stmt {
                    text: fragment.trim().to_string(),
                    line: Some(abs_line(section, rel, line_offset)),
                });
            }
        }
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
        .filter_map(|row| row.get(col_idx))
        .filter(|cell| contains_shall(&cell.to_lowercase()))
        .map(|cell| Stmt {
            text: cell.trim().to_string(),
            line: locate_line(section, cell, line_offset),
        })
        .collect()
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Strip markdown decoration that would confuse subject/trigger detection:
/// links `[t](u)` → `t` and bold `**`. Backticks are **kept** — a backticked
/// identifier in the response object (`provide \`CodeBlockEditor\``) is the most
/// concrete object there is, and the object-aware vague-response check treats it
/// as a concrete signal (FR-042).
/// Run to a **fixpoint** (CR-069). One pass is not enough: the two rewrites can
/// feed each other, so stripping bold can *synthesize* a link the link pass has
/// already run past. `[]**()` normalizes to `[]()` in one pass and to `""` in
/// two — and the authored shape that reaches it, `[FR-001]**(see note)**`, would
/// have left raw markdown link syntax in the text the grammar reads as prose.
///
/// **[RAN]** before changing it: zero documents across the 237 `~/dev` spec
/// bundles contain the triggering shape (`]**(` occurs twice, both with a space
/// before the paren, which does not form a link). So this closes a latent defect
/// and moves no current output.
///
/// Each pass strictly shortens the string — a link drops its brackets and
/// destination, a bold marker drops two bytes — so the loop terminates on its
/// own. The bound is a guard against pathological input on the fuzz surface,
/// not the termination argument.
pub(super) fn normalize(s: &str) -> String {
    const MAX_PASSES: usize = 16;
    let mut out = s.to_string();
    for pass in 0..MAX_PASSES {
        let once = re_link().replace_all(&out, "$1").replace("**", "");
        if once == out {
            return out;
        }
        out = once;
        // Reaching the ceiling means the termination argument above is wrong,
        // not that the input was unusual — each pass strictly shortens, so 16
        // passes require a 16-byte-longer input than any statement has. Fail in
        // tests rather than silently returning a non-fixpoint (CR-072).
        debug_assert!(
            pass + 1 < MAX_PASSES,
            "ears::normalize did not converge in {MAX_PASSES} passes: {s:?}"
        );
    }
    out
}

pub(super) fn excerpt(s: &str) -> String {
    let s = s.trim();
    if s.chars().count() > 160 {
        let truncated: String = s.chars().take(157).collect();
        format!("{truncated}…")
    } else {
        s.to_string()
    }
}

pub(super) fn strip_list_marker(line: &str) -> &str {
    let t = line.trim_start();
    if let Some(rest) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
        return rest;
    }
    // numbered list: "1. ", "12. "
    if let Some(pos) = t.find(". ") {
        if t[..pos].chars().all(|c| c.is_ascii_digit()) && !t[..pos].is_empty() {
            return &t[pos + 2..];
        }
    }
    t
}

/// The main clause: everything after the first comma when the statement leads
/// with a trigger keyword, else the whole statement.
fn main_clause(lower: &str) -> &str {
    let first = first_word(lower);
    if matches!(first.as_str(), "when" | "while" | "where" | "if") {
        if let Some(idx) = lower.find(", ") {
            return lower[idx + 2..].trim_start();
        }
    }
    lower
}

fn first_word(lower: &str) -> String {
    lower
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c: char| !c.is_ascii_alphabetic())
        .to_string()
}

pub(super) fn contains_shall(lower: &str) -> bool {
    re_shall().is_match(lower)
}

pub(super) fn abs_line(section: &QuireSection, rel: usize, line_offset: usize) -> usize {
    line_offset + section.start_line + rel + 1
}

/// Find the document line of a table cell by locating its text within the
/// section content; falls back to the section heading line.
pub(super) fn locate_line(section: &QuireSection, cell: &str, line_offset: usize) -> Option<usize> {
    let needle = cell.trim();
    for (rel, raw_line) in section.content.lines().enumerate() {
        if !needle.is_empty() && raw_line.contains(needle) {
            return Some(abs_line(section, rel, line_offset));
        }
    }
    Some(abs_line(section, 0, line_offset))
}

// ─── Compile-once regexes ───────────────────────────────────────────────────

fn re_shall() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)\bshall\b").expect("shall regex"))
}

fn re_link() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\[([^\]]*)\]\([^)]*\)").expect("link regex"))
}

/// The weak-verb alternation shared by the `shall`-anchored EARS check and the
/// bare-clause `ac` check — one verb set, two grammars (FR-047).
const VAGUE_VERBS: &str =
    r"support|handle|manage|process|deal\s+with|provide|enable|be\s+able\s+to";

fn re_vague() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(&format!(r"(?i)\bshall\s+(not\s+)?({VAGUE_VERBS})\b")).expect("vague regex")
    })
}

/// The same weak verbs without the `shall` anchor, for outcome-clause checking.
fn re_vague_bare() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(&format!(r"(?i)\b({VAGUE_VERBS})\b")).expect("vague-bare regex"))
}

/// A mechanism, numeric, or ordering qualifier after the verb — the response is
/// concrete/measurable (`handle X by Y`, `process within 16 ms`, `process only
/// push events`). Calibrated against the corpus (FR-042 object-aware tuning).
fn re_concrete_qualifier() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)\b(by|using|via|per|through|according to|that|which|within|before|after|only|not)\b|\bin order\b|\bidempotent|[0-9<>%`]",
        )
        .expect("concrete-qualifier regex")
    })
}

/// An abstract quality noun or unmeasurable manner — genuine vagueness even with
/// a grammatical object (`provide flexibility`, `handle errors gracefully`).
fn re_vague_quality() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)\b(flexibility|extensibility|robustness|scalability|usability|reliability|maintainability|interoperability|capabilit(y|ies)|functionality|complexity|seamless(ly)?|gracefully|appropriately|efficiently|properly|correctly|reliably|acceptable|reasonable|sufficient|adequate|user-friendly|intuitive|performant|flexible|robust|scalable)\b|\bas (needed|appropriate|required|possible)\b",
        )
        .expect("vague-quality regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    fn doc(text: &str) -> crate::ast::QuireDocument {
        crate::parse_document(text)
    }

    fn empty() -> GrammarLexicon {
        GrammarLexicon::empty()
    }

    fn lex(terms: &[&str]) -> GrammarLexicon {
        GrammarLexicon::from_terms(terms.iter().copied())
    }

    #[trace("TC-657", "FR-042-AC-1")]
    // every pattern classifies; no-match → unclassifiable.
    #[test]
    fn tc657_classification() {
        assert_eq!(
            classify("The system shall emit a structured log."),
            EarsPattern::Ubiquitous
        );
        assert_eq!(
            classify("When the token expires, the system shall refresh it."),
            EarsPattern::Event
        );
        assert_eq!(
            classify("While under load, the system shall shed traffic."),
            EarsPattern::State
        );
        assert_eq!(
            classify("If the request is unauthenticated, then the system shall reject it."),
            EarsPattern::Unwanted
        );
        assert_eq!(
            classify("Where billing is enabled, the system shall meter usage."),
            EarsPattern::Optional
        );
        assert_eq!(
            classify("While idle, when a job arrives, the system shall start it."),
            EarsPattern::Complex
        );
        assert_eq!(
            classify("This paragraph has no normative verb."),
            EarsPattern::Unclassifiable
        );
    }

    #[trace("TC-658", "FR-042-AC-2")]
    // non-singular fires on 2 shalls; enumerated stem doesn't.
    #[test]
    fn tc658_non_singular() {
        let d = doc("---\ntype: FR\n---\n## Description\n\nThe service shall mint a token and shall carry it downstream.\n");
        let f = check("FR", &d, 0, &empty());
        assert_eq!(f.iter().filter(|x| x.check == "non-singular").count(), 1);

        let enumerated = doc("---\ntype: FR\n---\n## Description\n\nThe function shall:\n\n1. Validate the input.\n2. Render the output.\n3. Return the result.\n");
        let f2 = check("FR", &enumerated, 0, &empty());
        assert_eq!(f2.iter().filter(|x| x.check == "non-singular").count(), 0);
    }

    #[trace("TC-659", "FR-042-AC-3")]
    // missing-subject fires; StR stakeholder subject doesn't.
    #[test]
    fn tc659_missing_subject() {
        assert!(has_missing_subject("shall be created lazily on first use"));
        assert!(!has_missing_subject(
            "The operator shall be able to resync a repository"
        ));
        let str_doc = doc("---\ntype: StR\n---\n## Stakeholder Need\n\nThe operator shall reconcile a repository on demand.\n");
        let f = check("StR", &str_doc, 0, &empty());
        assert_eq!(f.iter().filter(|x| x.check == "missing-subject").count(), 0);
    }

    #[trace("TC-660", "FR-042-AC-4")]
    // vague-response fires; passive does not; the check
    // is object-aware (a concrete object or mechanism qualifier suppresses it).
    #[test]
    fn tc660_vague_and_passive() {
        // Empty lexicon: only the engine-generic signals (mechanism/bound/
        // backtick) suppress; bare/abstract objects flag.
        let count = |text: &str| {
            check("FR", &doc(text), 0, &empty())
                .iter()
                .filter(|x| x.check == "vague-response")
                .count()
        };
        let fr = |body: &str| format!("---\ntype: FR\n---\n## Description\n\n{body}\n");

        // Bare / abstract object → vague.
        assert_eq!(
            count(&fr("The system shall support publishing to registries.")),
            1
        );
        assert_eq!(count(&fr("The system shall provide flexibility.")), 1);
        // `be able to` is capability phrasing — always vague.
        assert_eq!(count(&fr("The user shall be able to export the data.")), 1);

        // Object-aware suppression via engine-generic signals (no lexicon needed):
        // a backticked identifier, an ordering qualifier, a mechanism + bound.
        assert_eq!(
            count(&fr(
                "The system shall provide `CodeBlockEditor`, a fenced editor."
            )),
            0
        );
        assert_eq!(
            count(&fr("The webhook handler shall process only push events.")),
            0
        );
        assert_eq!(
            count(&fr(
                "The system shall handle 404 errors by displaying a toast."
            )),
            0
        );

        // Passive voice is not a vague verb at all.
        assert_eq!(
            count(&fr("Messages shall be included from newest to oldest.")),
            0
        );
    }

    fn fr_doc(body: &str) -> String {
        format!("---\ntype: FR\n---\n## Description\n\n{body}\n")
    }

    fn vague_count(text: &str, lexicon: &GrammarLexicon) -> usize {
        check("FR", &doc(text), 0, lexicon)
            .iter()
            .filter(|x| x.check == "vague-response")
            .count()
    }

    #[trace("TC-669", "FR-043-AC-3")]
    // a lexicon term suppresses; a different term re-flags.
    #[test]
    fn tc669_lexicon_term_suppresses() {
        let d = fr_doc("The system shall support pagination.");
        assert_eq!(vague_count(&d, &lex(&["pagination"])), 0); // term present → suppressed
        assert_eq!(vague_count(&d, &lex(&["cursor"])), 1); // wrong term → still flagged
        let ep = fr_doc("The system shall provide an endpoint for review runs.");
        assert_eq!(vague_count(&ep, &lex(&["endpoint"])), 0);
    }

    #[trace("TC-670", "FR-043-AC-4")]
    // under an empty lexicon a bare domain noun flags —
    // proving no hardcoded concrete-noun list remains in the engine.
    #[test]
    fn tc670_empty_lexicon_flags_bare_noun() {
        assert_eq!(
            vague_count(&fr_doc("The system shall support pagination."), &empty()),
            1
        );
        assert_eq!(
            vague_count(
                &fr_doc("The system shall provide an endpoint for review runs."),
                &empty()
            ),
            1
        );
    }

    #[trace("TC-671", "FR-043-AC-5")]
    // backtick / mechanism / bound suppression does not
    // depend on the lexicon — it still holds under an empty lexicon.
    #[test]
    fn tc671_generic_suppression_lexicon_independent() {
        let fr = |body: &str| format!("---\ntype: FR\n---\n## Description\n\n{body}\n");
        let count = |text: &str| {
            check("FR", &doc(text), 0, &empty())
                .iter()
                .filter(|x| x.check == "vague-response")
                .count()
        };
        assert_eq!(count(&fr("The system shall provide `RepoSource`.")), 0); // backtick
        assert_eq!(count(&fr("The system shall handle errors by retrying.")), 0); // mechanism
        assert_eq!(count(&fr("The system shall process up to 100 items.")), 0); // bound
                                                                                // …but an abstract object still flags regardless of lexicon.
        assert_eq!(count(&fr("The system shall provide flexibility.")), 1);
    }

    // A lexicon term suppresses its plural — including irregular plurals
    // (FR-043 plural matching; corpus objects are frequently plural).
    #[test]
    fn lexicon_matches_plurals() {
        let fr = |body: &str| format!("---\ntype: FR\n---\n## Description\n\n{body}\n");
        let count = |text: &str, lexicon: &GrammarLexicon| {
            check("FR", &doc(text), 0, lexicon)
                .iter()
                .filter(|x| x.check == "vague-response")
                .count()
        };
        assert_eq!(
            count(
                &fr("The system shall support endpoints."),
                &lex(&["endpoint"])
            ),
            0
        ); // -s
        assert_eq!(
            count(&fr("The system shall enforce policies."), &lex(&["policy"])),
            0
        ); // -ies
        assert_eq!(
            count(&fr("The system shall provide boxes."), &lex(&["box"])),
            0
        ); // -es
    }

    #[trace("TC-661", "FR-042-AC-5")]
    // non-canonical trigger fires; NFR no-trigger does not.
    #[test]
    fn tc661_non_canonical_trigger() {
        let latent = doc("---\ntype: FR\n---\n## Description\n\nOn startup, the service shall perform a full scan.\n");
        let f = check("FR", &latent, 0, &empty());
        assert_eq!(
            f.iter()
                .filter(|x| x.check == "non-canonical-trigger")
                .count(),
            1
        );

        let nfr = doc("---\ntype: NFR\n---\n## Statement\n\nThe system shall sustain p95 latency under 200ms.\n");
        let f2 = check("NFR", &nfr, 0, &empty());
        assert_eq!(
            f2.iter()
                .filter(|x| x.check == "non-canonical-trigger")
                .count(),
            0
        );
    }

    #[trace("TC-662", "FR-042-AC-6")]
    // grammar runs only on bound (archetype, section) pairs.
    #[test]
    fn tc662_binding() {
        // A vague statement parked in Dependencies (not bound) yields nothing.
        let d =
            doc("---\ntype: FR\n---\n## Dependencies\n\nThe system shall support everything.\n");
        assert!(check("FR", &d, 0, &empty()).is_empty());
        // An IT document binds no sections at all.
        let it = doc("---\ntype: IT\n---\n## Objective\n\nThe system shall support everything.\n");
        assert!(check("IT", &it, 0, &empty()).is_empty());
    }

    #[trace("TC-664", "FR-042-AC-8")]
    // findings carry excerpt, line, pattern, severity.
    #[test]
    fn tc664_finding_fields() {
        let d = doc("---\ntype: FR\n---\n## Description\n\nThe system shall support publishing.\n");
        let f = check("FR", &d, 0, &empty());
        let v = f.iter().find(|x| x.check == "vague-response").unwrap();
        assert!(!v.statement.is_empty());
        assert!(v.line.is_some());
        assert_eq!(v.pattern.as_deref(), Some("ubiquitous"));
        assert_eq!(v.severity, GrammarSeverity::Warning);
    }

    #[trace("TC-665", "FR-042-AC-9")]
    // fenced/quote/reference content is not a statement.
    #[test]
    fn tc665_skips() {
        let d = doc("---\ntype: FR\n---\n## Description\n\n```\nThe system shall support publishing.\n```\n\n> The system shall support quoting.\n");
        assert!(check("FR", &d, 0, &empty()).is_empty());
    }
}

#[cfg(test)]
mod props_metamorphic {
    use proptest::prelude::*;

    /// Strings built from the tokens `normalize` rewrites, so the generator
    /// actually reaches the interaction between the two rewrites. A plain `.*`
    /// strategy passed 2,000 cases against the pre-CR-069 single-pass form —
    /// it essentially never produces `[](...)` or `**`.
    fn markdownish() -> impl Strategy<Value = String> {
        let tok = prop_oneof![
            Just("[".to_string()),
            Just("]".to_string()),
            Just("(".to_string()),
            Just(")".to_string()),
            Just("**".to_string()),
            Just("*".to_string()),
            Just("`".to_string()),
            "[a-z]{1,3}".prop_map(|s| s),
        ];
        proptest::collection::vec(tok, 0..14).prop_map(|v| v.concat())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(4000))]

        /// TC-891, FR-042 (CR-069, Property): EARS normalization is idempotent.
        /// Its counterexample before the fixpoint loop was `[]**()`, which
        /// normalized to `[]()` and then to `""`.
        #[test]
        fn tc891_ears_normalize_is_idempotent(s in markdownish()) {
            let once = super::normalize(&s);
            prop_assert_eq!(super::normalize(&once), once.clone(), "input={:?}", s);
        }

        /// TC-891, FR-042 (CR-069, Property): and it never lengthens its input,
        /// which is what makes the fixpoint loop terminate.
        #[test]
        fn tc891_ears_normalize_never_grows(s in markdownish()) {
            prop_assert!(super::normalize(&s).len() <= s.len(), "input={:?}", s);
        }
    }
}

#[cfg(test)]
mod cr069_regressions {
    /// TC-891: regression (FR-042, CR-069) — the generator's minimized witness.
    /// Stripping `**` synthesized a link the link pass had already run past, so
    /// one pass left `[]()` — raw markdown link syntax in the text the grammar
    /// reads as prose — and a second pass reduced it to `""`.
    #[test]
    fn tc891_bold_removal_can_synthesize_a_link() {
        assert_eq!(super::normalize("[]**()"), "");
        // The authored shape that reaches it.
        assert_eq!(super::normalize("[FR-001]**(see note)**"), "FR-001");
        // A link that was already a link is unaffected — one pass was enough.
        assert_eq!(super::normalize("**[label](url)**"), "label");
    }
}
