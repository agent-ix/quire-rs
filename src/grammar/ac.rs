//! The acceptance-criteria grammar (`ac`, FR-047) — second grammar in the
//! `iso-spec-core` bundle.
//!
//! Where EARS ([`super::ears`]) checks the normative *statements* of a
//! requirement, `ac` checks its **binding criteria**: the `Criteria` column of
//! the table each archetype's module contract declares, plus any
//! `### <doc-id>-<kind>-N` supplement section whose body elaborates one row.
//! Which section and which sub-id kind is per-archetype — see [`BINDINGS`].
//!
//! Two things differ from the EARS grammar by design:
//!
//! - **No modal-verb filter.** Every non-empty criteria cell is a statement.
//!   An acceptance criterion with no `shall` is still a criterion, and the
//!   checks below are exactly the ones that catch such a cell.
//! - **Shape, not pattern.** A criterion is classified `assertion` (the
//!   canonical shape — CR-013), `obligation` or `given-when-then` (both
//!   recognized so their outcome clause is still checked, but steered by
//!   `non-canonical-shape`), or `unstructured`.
//! - **Mention is not use** (CR-017). A keyword inside an inline code span is
//!   quoted example data, so shape classification reads a masked copy of the
//!   statement while the vocabulary checks read the original.
//!
//! Every check ships advisory (`warning`); promotion is a per-check policy
//! lever (FR-048) gated on a corpus baseline sweep (FR-047-CON-1).

use std::sync::OnceLock;

use regex::Regex;

use super::ears;
use super::{GrammarFinding, GrammarSeverity, GrammarVocabularies};
use crate::ast::{QuireDocument, QuireSection};
use crate::query;

/// The shape of one acceptance criterion (FR-047, CR-013).
///
/// Classification is **structural**: it locates the outcome clause the checks
/// read. Only [`AcShape::Assertion`] is canonical — an acceptance criterion is
/// a verification statement, so the shape that carries the test oracle is the
/// canonical one. An obligation restates the requirement one level down and a
/// Given/When/Then cell is a second rendering of the same assertion; both are
/// steered by `non-canonical-shape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcShape {
    /// Asserts an outcome directly — **the canonical shape**. Its outcome
    /// clause is the whole statement.
    Assertion,
    /// Matches an EARS pattern, i.e. states an obligation rather than an
    /// observation. Its outcome clause is the response after the modal verb.
    Obligation,
    /// Given/When/Then — recognized so extraction can consume such cells, with
    /// the `Then` clause as the outcome.
    GivenWhenThen,
    /// No modal, no Given/When/Then structure, and no observable signal —
    /// nothing to test with.
    Unstructured,
}

impl AcShape {
    /// Stable machine-readable label, carried on the finding's `pattern`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Assertion => "assertion",
            Self::Obligation => "obligation",
            Self::GivenWhenThen => "given-when-then",
            Self::Unstructured => "unstructured",
        }
    }

    /// True when the shape is not the canonical assertion — what
    /// `non-canonical-shape` reports (`Unstructured` is reported by
    /// `unclassifiable` instead, so it is excluded here).
    fn is_non_canonical(self) -> bool {
        matches!(self, Self::Obligation | Self::GivenWhenThen)
    }
}

/// Rollout default: advisory (FR-047-CON-1). The FR-048 severity map is the
/// only promotion lever, and promotion is user-gated.
const DEFAULT_SEVERITY: GrammarSeverity = GrammarSeverity::Warning;

/// The column carrying the criterion text. One name across every archetype —
/// only the owning section and the sub-id kind differ.
const CRITERIA_COLUMN: &str = "Criteria";

/// The column carrying the criterion's own id (`FR-047-AC-3`). Read for the
/// FR-052 classification record's `row_id`; the checks do not use it.
const ID_COLUMN: &str = "ID";

/// Where an archetype's **binding criteria** live, and what sub-id kind their
/// supplement subsections use (CR-020).
///
/// The binding follows the module *contract*, not the corpus. `spec-artifacts-iso`
/// declares a binding-criteria table on exactly three archetypes:
///
/// | Archetype | Section | Sub-id | Reference the criterion is judged against |
/// |---|---|---|---|
/// | `FR`  | `Acceptance Criteria` | `-AC-` | the specification (verification) |
/// | `NFR` | `Acceptance Criteria` | `-AC-` | the specification (verification) |
/// | `StR` | `Validation Criteria` | `-VC-` | the stakeholder's real need (validation) |
///
/// `StR` keeps its own heading and sub-id kind deliberately: ISO/IEC/IEEE 29148
/// separates *validation* from *verification* at the artifact level — a
/// Stakeholder Requirements Specification is validated, a System/Software
/// Requirements Specification is verified. Only the table *shape* is unified.
///
/// **`US` and `IT` are absent, and that is the point.** CR-014 bound `ac` to
/// FR/NFR/US/StR/IT on a cell census, but a census measures what authors wrote,
/// not what the contract declares. A user story's `Acceptance Examples
/// (Illustrative)` are non-binding by design — its skeleton says so outright —
/// so reading them as criteria treats discovery context as verification
/// criteria and inflates every number that counts them. `IT` declares no
/// criteria table at all. A `## Acceptance Criteria` heading on a `US` is a
/// structural defect for the module's lint rules (`forbidden_section`), not an
/// input to this grammar.
const BINDINGS: &[(&str, &str, &str)] = &[
    ("FR", "Acceptance Criteria", "AC"),
    ("NFR", "Acceptance Criteria", "AC"),
    ("StR", "Validation Criteria", "VC"),
];

/// The criteria section and supplement sub-id kind for `archetype`, or `None`
/// when the archetype declares no binding-criteria table.
fn binding_for(archetype: &str) -> Option<(&'static str, &'static str)> {
    BINDINGS
        .iter()
        .find(|(name, _, _)| *name == archetype)
        .map(|(_, section, kind)| (*section, *kind))
}

/// One acceptance criterion with its 1-based document line.
pub(super) struct Stmt {
    pub(super) text: String,
    pub(super) line: Option<usize>,
    /// The criterion's own id (`FR-047-AC-3`) — the `ID` column of the
    /// criteria table, or the `### <doc-id>-<kind>-N` supplement heading.
    /// `None` when the source carries neither.
    ///
    /// The `ac` checks do not read it: a finding is located by line and
    /// statement. The FR-052 property classifier carries it onto its record,
    /// because without the criterion id a downstream generator cannot emit the
    /// tracking tag coverage reconciliation keys on.
    pub(super) row_id: Option<String>,
}

/// Every binding criterion of `doc`, in document order — the `Criteria` column
/// of the archetype's own binding-criteria section followed by its
/// `### <doc-id>-<kind>-N` supplement sections ([`BINDINGS`]). An archetype
/// with no binding-criteria table yields none (FR-047-AC-6).
///
/// Shared by [`check`] and the FR-052 property classifier
/// ([`super::property::classify_document`]) so the two surfaces cannot disagree
/// about which cells are criteria (FR-052-AC-9).
pub(super) fn statements(archetype: &str, doc: &QuireDocument, line_offset: usize) -> Vec<Stmt> {
    // CR-020: bind per the module contract, not the corpus census. CR-014
    // widened this to FR/NFR/US/StR/IT on cell counts, but outside FR those
    // cells were tables individual authors improvised — 20 StR cells and 69 US
    // cells across 199 repos. See [`BINDINGS`].
    let Some((criteria_section, sub_id_kind)) = binding_for(archetype) else {
        return Vec::new();
    };
    let mut stmts: Vec<Stmt> = Vec::new();
    if let Some(section) = query::section(doc, criteria_section) {
        stmts.extend(criteria_cells(section, line_offset));
    }
    for section in supplement_sections(&doc.sections, sub_id_kind) {
        stmts.extend(supplement_statements(section, line_offset, sub_id_kind));
    }
    stmts
}

/// Check `doc` against the `ac` grammar. The grammar binds to the `Criteria`
/// column of the archetype's own binding-criteria section and to its
/// `### <doc-id>-<kind>-N` supplement sections ([`BINDINGS`]); every other
/// archetype and section yields nothing (FR-047-AC-6).
pub fn check(
    archetype: &str,
    doc: &QuireDocument,
    line_offset: usize,
    vocab: GrammarVocabularies<'_>,
) -> Vec<GrammarFinding> {
    let mut findings = Vec::new();
    for stmt in &statements(archetype, doc, line_offset) {
        check_statement(stmt, vocab, &mut findings);
    }
    findings
}

fn check_statement(stmt: &Stmt, vocab: GrammarVocabularies<'_>, out: &mut Vec<GrammarFinding>) {
    let shape = classify(&stmt.text, vocab);
    let label = Some(shape.as_str().to_string());

    let push = |out: &mut Vec<GrammarFinding>, check: &str, message: String| {
        out.push(GrammarFinding {
            grammar: "ac".to_string(),
            check: check.to_string(),
            pattern: label.clone(),
            message,
            line: stmt.line,
            statement: ears::excerpt(&stmt.text),
            severity: DEFAULT_SEVERITY,
        });
    };

    if shape == AcShape::Unstructured {
        push(
            out,
            "unclassifiable",
            "criterion states no predicate — a bare noun phrase asserts nothing that can be \
             tested"
                .to_string(),
        );
    }
    if shape.is_non_canonical() {
        push(
            out,
            "non-canonical-shape",
            format!(
                "criterion is {}-shaped; an acceptance criterion is a verification statement, so \
                 the canonical shape is a direct assertion of the outcome",
                shape.as_str()
            ),
        );
    }
    if obligation_count(&stmt.text, shape) > 1 {
        push(
            out,
            "non-singular",
            "criterion bundles more than one obligation; split into one criterion per obligation"
                .to_string(),
        );
    }

    // Checks 3 and 4 read the *outcome* clause: the whole statement for an
    // `assertion`, the response clause of an `obligation`, the `Then` clause of
    // a `given-when-then` cell (so a non-canonical cell's other checks still run
    // — FR-047-AC-10), and the whole cell when nothing structured it.
    let outcome = outcome_clause(&stmt.text, shape);
    if let Some(verb) = ears::vague_verb_in_clause(outcome, vocab.lexicon) {
        push(
            out,
            "vague-response",
            format!("vague outcome verb `{verb}`; name a concrete, verifiable outcome"),
        );
    }
    // CR-014: fire on membership of the *closed* vacuity set, not on absence
    // from the open-ended observable-verb space. Suppression reads the whole
    // statement rather than the sliced clause, because an obligation's concrete
    // signal often sits before the modal ("Repeated calls to `score_ip_encoded`
    // … SHALL not allocate heap memory").
    if vocab.vacuous.is_vacuous(outcome) && !carries_signal(&stmt.text, vocab) {
        push(
            out,
            "vacuous-outcome",
            "criterion asserts only that something works; name the observable result".to_string(),
        );
    }

    // ── Measurement instrument, not a shipped check (agent-ix/quire-rs#18) ──
    //
    // The check CR-014 retired, restored from `7a259d5` with the three
    // mechanical defects that inflated its original measurement already fixed:
    //
    //   1. irregular / `-y` inflections — fixed in `inflect` (grammar/mod.rs),
    //      so `written`, `built`, `sent`, `carries` now match their stems;
    //   2. obligation clause-slicing dropping a pre-modal concrete object —
    //      fixed here by reading the whole statement, as `vacuous-outcome`
    //      does, instead of `outcome_clause(...)`;
    //   3. supplement-prose segmentation producing truncated fragments —
    //      fixed in `split_sentences` (CR-017), shared by every check.
    //
    // The original predicate was `!is_observable(outcome, lexicon, observable)`
    // where `is_observable` is byte-for-byte today's `carries_signal`, so the
    // only deliberate change is the clause it reads.
    //
    // Behind the `remeasure` feature so no shipping build can emit it. See
    // `scripts/ac_corpus_sweep.py`.
    #[cfg(feature = "remeasure")]
    if !carries_signal(&stmt.text, vocab) {
        push(
            out,
            "no-observable-outcome",
            "outcome clause names no externally checkable result (no returned value, emitted \
             record, identifier, or bound)"
                .to_string(),
        );
    }
}

/// True when the statement carries something externally checkable: a
/// concrete-object signal (backticked identifier or numeric bound), a merged
/// lexicon term, or a declared observable-result verb.
fn carries_signal(statement: &str, vocab: GrammarVocabularies<'_>) -> bool {
    let normalized = ears::normalize(statement);
    re_concrete_object_signal().is_match(&normalized)
        || vocab.lexicon.contains_term(&normalized)
        || vocab.observable.contains_verb(&normalized)
}

/// A backticked span is a **mention** of a token, not a use of it: a criterion
/// reading ``a statement with two `shall` clauses yields one finding`` quotes
/// `shall` as example data — it does not itself impose an obligation. Grammar
/// **keyword** detection therefore reads a masked copy in which every closed
/// code span's contents are neutralized (CR-017).
///
/// The mask is deliberately not applied to the vocabulary checks: a backticked
/// lexicon term still suppresses `vague-response` (FR-043-AC-3), and a
/// backticked identifier still counts as a concrete-object signal (FR-042).
/// Replacing the contents with `x` rather than deleting the span preserves
/// both of those properties.
///
/// The mask is **byte-length-preserving** (each character becomes as many `x`
/// bytes as it occupied), so an offset found in the masked copy indexes the
/// original — [`outcome_clause`] relies on that.
pub(super) fn mask_code_spans(statement: &str) -> String {
    let mut out = String::with_capacity(statement.len());
    let mut rest = statement;
    while let Some(open) = rest.find('`') {
        let (before, from_open) = rest.split_at(open);
        out.push_str(before);
        // CommonMark: a run of N backticks is closed by the next run of exactly
        // N (CR-026). Reading only the first backtick made a ``double-tick``
        // span — the form used to quote a fragment that itself contains a code
        // span — degenerate into an empty span, leaving the quoted keywords
        // *inside* it unmasked and read as though they were used.
        let ticks = from_open.len() - from_open.trim_start_matches('`').len();
        let body = &from_open[ticks..];
        let Some(close) = find_closing_run(body, ticks) else {
            // An unbalanced run opens no span; the tail is ordinary prose.
            out.push_str(from_open);
            return out;
        };
        for _ in 0..ticks {
            out.push('`');
        }
        for c in body[..close].chars() {
            for _ in 0..c.len_utf8() {
                out.push('x');
            }
        }
        for _ in 0..ticks {
            out.push('`');
        }
        rest = &body[close + ticks..];
    }
    out.push_str(rest);
    out
}

/// The byte offset in `body` of the next backtick run of exactly `ticks`, or
/// `None` when the span is never closed. A longer run is *not* a closer — it is
/// content — which is what keeps a `` `nested` `` span inside a double-tick one
/// from ending it early.
fn find_closing_run(body: &str, ticks: usize) -> Option<usize> {
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let run = bytes[i..].iter().take_while(|&&b| b == b'`').count();
        if run == ticks {
            return Some(i);
        }
        i += run;
    }
    None
}

/// True when the statement has a **predicate** — a modal or copula, an inflected
/// or irregular verb form, a declared observable verb, a concrete signal, or an
/// elided-copula predication (CR-019). This is the structural question
/// `unclassifiable` asks after CR-014, and it is deliberately independent of any
/// domain vocabulary being complete.
///
/// On the corpus this is a weak test — 99.6% of cells satisfy it, 23.4% via the
/// `-s`/`-ed`/`-ing` branch alone, which cannot tell a verb from a plural noun.
/// FR-047 describes what it actually catches; do not read the name as a claim
/// that a finite verb was found (agent-ix/quire-rs#19).
fn has_predicate(statement: &str, vocab: GrammarVocabularies<'_>) -> bool {
    let normalized = ears::normalize(statement);
    carries_signal(&normalized, vocab)
        || re_modal_or_copula().is_match(&normalized)
        || re_inflected_verb().is_match(&normalized)
        || re_irregular_past().is_match(&normalized)
        || re_elided_copula().is_match(&normalized)
}

// ─── Classification ─────────────────────────────────────────────────────────

/// Classify one criterion into an [`AcShape`] (CR-013).
///
/// Structure first: a modal verb makes it an obligation (a `When … shall …`
/// criterion is an EARS event pattern, not a GWT trigger), then Given/When/Then.
/// What remains is an `assertion` when it carries an observable signal — the
/// canonical shape — and `unstructured` when it carries nothing at all.
pub fn classify(statement: &str, vocab: GrammarVocabularies<'_>) -> AcShape {
    // CR-017: shape is decided on the masked copy, so a criterion that *quotes*
    // a modal or a Given/When/Then keyword is not miscounted as using one.
    let masked = mask_code_spans(statement);
    if ears::classify(&masked) != ears::EarsPattern::Unclassifiable {
        return AcShape::Obligation;
    }
    if is_given_when_then(&masked) {
        return AcShape::GivenWhenThen;
    }
    // CR-014: a criterion with a predicate is an assertion, whatever verb it
    // uses; only a predicate-less cell is unstructured.
    if has_predicate(statement, vocab) {
        return AcShape::Assertion;
    }
    AcShape::Unstructured
}

/// A Given/When/Then criterion: a `Then`/result clause preceded by a
/// `Given`/`When` clause, in prose (`Given X, when Y, then Z`) or bullet form
/// (`- Given X` / `- When Y` / `- Then Z`, flattened into one cell or body).
fn is_given_when_then(statement: &str) -> bool {
    let lower = ears::normalize(statement).to_lowercase();
    let Some(then_at) = re_then().find(&lower) else {
        return false;
    };
    re_given_or_when()
        .find(&lower)
        .is_some_and(|m| m.start() < then_at.start())
}

// ─── Clause checks ──────────────────────────────────────────────────────────

/// The number of independent obligations in a criterion: its modal verbs, or —
/// only when it states its consequents without one — its `Then` clauses. The
/// positive/negative pair idiom (`X yields a finding; Y yields none`) states one
/// behaviour in two directions and counts as a single obligation
/// (FR-047-AC-3).
///
/// **`Then` counts only in a Given/When/Then criterion (CR-024).** Elsewhere
/// `then` sequences a *precedence chain* — ``resolves safeStorage first, then
/// `GITHUB_TOKEN`, then `undefined` `` — which states one resolution rule, not
/// one obligation per link. Counting it there scored four corpus criteria as
/// plural that carry no obligation at all (no modal verb anywhere in them).
/// Where the criterion has modals, they are the count: `max` let a narrative
/// `then` outvote them.
fn obligation_count(statement: &str, shape: AcShape) -> usize {
    // CR-017: quoted keywords are mentions, not obligations.
    let lower = ears::normalize(&mask_code_spans(statement)).to_lowercase();
    let modals = re_shall_word().find_iter(&lower).count();
    let count = if modals > 0 {
        modals
    } else if shape == AcShape::GivenWhenThen {
        re_then().find_iter(&lower).count()
    } else {
        0
    };
    if count == 2 && is_negated_restatement(&lower) {
        return 1;
    }
    count
}

/// The positive/negative pair idiom: two obligations whose second states the
/// same behaviour negatively, so the criterion is singular (FR-047-AC-3).
///
/// **The idiom is not tied to a separator (CR-024).** This rule originally
/// required the two halves to be split by `;` or ` while `, which missed the
/// form the corpus actually writes — ``SHALL render `skipped` … and SHALL NOT
/// execute`` — and left 19 singular criteria flagged. What identifies the
/// negative face is the second obligation, not the punctuation before it:
///
/// - its modal is directly negated (`SHALL NOT`, `SHALL never`), or
/// - its clause carries a negation marker (`` `github_url` SHALL be None ``,
///   `No Secret deletion SHALL occur`, `otherwise it SHALL be omitted`).
///
/// The clause is delimited at the last separator before the second modal so the
/// marker has to sit in *that* obligation — a `no` inside the first
/// obligation's own outcome does not suppress. Together with the `count == 2`
/// guard in [`obligation_count`], a criterion with three obligations is never
/// suppressed by this rule however it is worded.
fn is_negated_restatement(lower: &str) -> bool {
    let mut modals = re_shall_word().find_iter(lower);
    let (Some(first), Some(second)) = (modals.next(), modals.next()) else {
        // A `Then`-counted criterion has no modal to negate.
        return false;
    };
    if re_negated_modal().is_match(&lower[second.start()..]) {
        return true;
    }
    let clause_start = re_clause_separator()
        .find_iter(&lower[first.end()..second.start()])
        .last()
        .map_or(first.end(), |m| first.end() + m.end());
    re_negative_case().is_match(&lower[clause_start..])
}

/// The outcome clause a criterion is judged on: everything after the modal
/// verb for an `ears`-shaped criterion, everything after `Then` for a
/// `given-when-then`-shaped one, and the whole criterion otherwise (an
/// unclassifiable cell has no clause structure to slice).
fn outcome_clause(statement: &str, shape: AcShape) -> &str {
    // The keyword is located in the masked copy (CR-017) — a *quoted* modal must
    // not decide where the outcome starts — but the clause returned is the
    // original text, so the vocabulary checks still see the real words. The mask
    // preserves byte offsets, so the match indexes `statement` directly.
    let masked = mask_code_spans(statement);
    let tail = |m: Option<regex::Match<'_>>| m.map(|m| &statement[m.end()..]);
    match shape {
        AcShape::Obligation => tail(re_shall_word().find(&masked)).unwrap_or(statement),
        AcShape::GivenWhenThen => tail(re_then().find(&masked)).unwrap_or(statement),
        // An assertion states its outcome directly, and an unstructured cell has
        // no clause structure to slice — both are judged whole.
        AcShape::Assertion | AcShape::Unstructured => statement,
    }
}

// ─── Statement extraction ───────────────────────────────────────────────────

/// Every non-empty `Criteria` cell is one statement — no modal-verb filter
/// (FR-047-AC-2). A missing section, table, or column yields none.
///
/// The `ID` column is read alongside (FR-052): it carries the criterion's own
/// id onto the statement. A table declaring no `ID` column still yields its
/// criteria, with no id — the column is optional here, as it is for every
/// check.
fn criteria_cells(section: &QuireSection, line_offset: usize) -> Vec<Stmt> {
    let Some(table) = query::parse_table(&section.content) else {
        return Vec::new();
    };
    let column = |name: &str| {
        table
            .headers
            .iter()
            .position(|h| h.trim().eq_ignore_ascii_case(name))
    };
    let Some(col_idx) = column(CRITERIA_COLUMN) else {
        return Vec::new();
    };
    let id_idx = column(ID_COLUMN);
    table
        .rows
        .iter()
        .filter_map(|row| {
            let cell = row.get(col_idx)?.trim();
            if cell.is_empty() {
                return None;
            }
            Some(Stmt {
                text: cell.to_string(),
                line: ears::locate_line(section, cell, line_offset),
                row_id: id_idx
                    .and_then(|i| row.get(i))
                    .map(|id| id.trim())
                    .filter(|id| !id.is_empty())
                    .map(str::to_string),
            })
        })
        .collect()
}

/// Every `### <doc-id>-<kind>-N` section in the document tree — the supplement
/// body of one criteria row (e.g. `### FR-047-AC-1`, `### StR-001-VC-2`).
/// `kind` is the archetype's sub-id kind from [`BINDINGS`], so an `StR`
/// supplement is matched under `-VC-` and never under `-AC-` (CR-020).
fn supplement_sections<'a>(sections: &'a [QuireSection], kind: &str) -> Vec<&'a QuireSection> {
    let mut out = Vec::new();
    for s in sections {
        if re_supplement_heading(kind).is_match(s.heading.trim()) {
            out.push(s);
        }
        out.extend(supplement_sections(&s.children, kind));
    }
    out
}

/// Split prose into sentences on `". "`, ignoring a boundary that falls **inside
/// an inline code span** (CR-017): an embedded example
/// (``` `EXPLAIN … SELECT ... ORDER BY col LIMIT 10` ```) is one quoted token,
/// and cutting it in half produced a statement whose backticks no longer
/// balanced — which then hid the modal that followed it.
fn split_sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut in_code = false;
    let mut start = 0;
    let bytes = text.as_bytes();
    for (i, c) in text.char_indices() {
        match c {
            // An unbalanced backtick opens no span — the same rule
            // `mask_code_spans` applies, so the two readers cannot disagree
            // about where a span is. Without it a stray backtick would swallow
            // every remaining sentence boundary in the paragraph.
            '`' if in_code || text[i + 1..].contains('`') => in_code = !in_code,
            '.' if !in_code && bytes.get(i + 1) == Some(&b' ') => {
                out.push(&text[start..i]);
                start = i + 2;
            }
            _ => {}
        }
    }
    out.push(&text[start..]);
    out
}

/// Segment a supplement body into statements: one per sentence of prose, with
/// fenced code blocks and blockquotes skipped per the FR-042 skip rules
/// (FR-047-AC-11). No modal-verb filter, as with the criteria cells.
///
/// Every statement carries the supplement's own criterion id, read from the
/// `### <doc-id>-<kind>-N` heading `kind` matched the section under (FR-052).
fn supplement_statements(section: &QuireSection, line_offset: usize, kind: &str) -> Vec<Stmt> {
    let row_id = re_supplement_heading(kind)
        .find(section.heading.trim())
        .map(|m| m.as_str().to_string());
    let mut out = Vec::new();
    let mut in_fence = false;
    // CR-014: join wrapped prose before segmenting. Splitting per source line
    // truncated statements mid-clause ("During ambuild in the default build
    // mode, the native graph builder SHALL use"), which the ecosystem fit check
    // surfaced as a large share of the findings. A paragraph is a run of
    // non-blank lines; its first line carries the reported position.
    let mut paragraph: Vec<&str> = Vec::new();
    let mut paragraph_line: usize = 0;

    let flush = |paragraph: &mut Vec<&str>, start: usize, out: &mut Vec<Stmt>| {
        if paragraph.is_empty() {
            return;
        }
        let joined = paragraph.join(" ");
        paragraph.clear();
        for fragment in split_sentences(&joined) {
            let text = fragment.trim();
            if text.is_empty() {
                continue;
            }
            out.push(Stmt {
                text: text.to_string(),
                line: Some(start),
                row_id: row_id.clone(),
            });
        }
    };

    for (rel, raw_line) in section.content.lines().enumerate() {
        let trimmed = raw_line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            flush(&mut paragraph, paragraph_line, &mut out);
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.starts_with('>') || trimmed.starts_with('#') {
            flush(&mut paragraph, paragraph_line, &mut out);
            continue;
        }
        let line = ears::strip_list_marker(trimmed);
        if line.trim().is_empty() {
            flush(&mut paragraph, paragraph_line, &mut out);
            continue;
        }
        // A list marker starts a new statement even without a blank line.
        if line != trimmed {
            flush(&mut paragraph, paragraph_line, &mut out);
        }
        if paragraph.is_empty() {
            paragraph_line = ears::abs_line(section, rel, line_offset);
        }
        paragraph.push(line);
    }
    flush(&mut paragraph, paragraph_line, &mut out);
    out
}

// ─── Compile-once regexes ───────────────────────────────────────────────────

fn re_then() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)\bthen\b").expect("then regex"))
}

fn re_given_or_when() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)\b(given|when)\b").expect("given/when regex"))
}

fn re_shall_word() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)\bshall\b").expect("shall regex"))
}

/// The negative half of the positive/negative pair idiom.
///
/// Bare `not` is deliberately absent (CR-024): it is the commonest word in a
/// criterion's *condition* (`when the record is not found …`), so reading it as
/// a negative face suppressed real pairs of obligations. A directly negated
/// modal is recognized by [`re_negated_modal`] instead, which cannot be
/// confused with a condition.
fn re_negative_case() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)\b(none|no|never|neither|nothing|zero|without|otherwise)\b")
            .expect("negative-case regex")
    })
}

/// A directly negated obligation — `SHALL NOT`, `SHALL never` — anchored at the
/// modal it negates (CR-024).
fn re_negated_modal() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)^shall\s+(not|never)\b").expect("negated-modal regex"))
}

/// Where one obligation's clause ends and the next begins. `otherwise` is
/// absent on purpose — it *marks* the negative case rather than delimiting it,
/// and is read by [`re_negative_case`].
fn re_clause_separator() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)(;|\.\s|,\s|\s+and\s+|\s+but\s+|\s+while\s+)")
            .expect("clause-separator regex")
    })
}

/// `### <doc-id>-AC-N` — an acceptance-criterion supplement heading.
fn re_supplement_heading(kind: &str) -> &'static Regex {
    static AC: OnceLock<Regex> = OnceLock::new();
    static VC: OnceLock<Regex> = OnceLock::new();
    let build = |k: &str| {
        Regex::new(&format!(r"(?i)^[A-Za-z]+-\d+-{k}-\d+\b")).expect("supplement heading regex")
    };
    // The kinds come from the closed `BINDINGS` table, so one `OnceLock` each
    // keeps the compile-once property without a lock or a map.
    match kind {
        "VC" => VC.get_or_init(|| build("VC")),
        _ => AC.get_or_init(|| build("AC")),
    }
}

/// A modal or copula — the clearest predicate marker.
pub(super) fn re_modal_or_copula() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)\b(shall|should|must|may|can|cannot|will|won't|would|does|doesn't|do|don't|is|are|was|were|be|been|being|has|have|had)\b",
        )
        .expect("modal/copula regex")
    })
}

/// An inflected verb form (`-s`, `-ed`, `-ing`) — a weak but broad predicate
/// signal that needs no vocabulary to be complete.
pub(super) fn re_inflected_verb() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)\b\w+(s|ed|ing)\b").expect("inflected-verb regex"))
}

/// A predication whose copula is **elided** (CR-019): an existential or
/// quantifier head (`No refresh_token in response body`, `Only one active token
/// per user`) or a predicative adjective (`No credential-material field
/// present`, `Loki datasource visible in Grafana`, `Lockout response identical
/// to normal failure`). Each asserts something perfectly testable while
/// containing no inflected verb, no modal, and no concrete signal, so before
/// this branch they classified `unstructured` and drew a spurious
/// `unclassifiable` finding — the largest false-positive class in the check
/// (14 of a 40-cell sample).
///
/// This is a **closed** set used to *suppress* a finding, not an open one whose
/// membership is required — the distinction CR-014 turned on. It sits in the
/// engine rather than in module data because, like [`re_modal_or_copula`], it is
/// English grammar, not domain vocabulary: an author's project glossary should
/// not have to declare that `present` is an adjective.
fn re_elided_copula() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)(^\s*(no|none|neither|only|exactly|at most|at least)\b|\b(present|absent|missing|visible|hidden|identical|independent|equal|equivalent|unique|empty|non-empty|consistent|idempotent|deterministic|available|unavailable|immutable|atomic|monotonic|stable|reachable|unreachable|distinct|sorted|ordered|unchanged)\b)",
        )
        .expect("elided-copula regex")
    })
}

/// Irregular past/participle forms the `-ed` rule cannot reach.
pub(super) fn re_irregular_past() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)\b(built|sent|written|wrote|kept|made|found|held|ran|begun|threw|thrown|caught|left|lost|shown|drawn|gave|given|took|taken|saw|seen|set|put|cut|read)\b",
        )
        .expect("irregular-past regex")
    })
}

/// A concrete-object signal per FR-042: a backticked identifier, a digit, or a
/// comparison/percentage bound.
fn re_concrete_object_signal() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[0-9<>%`]").expect("concrete-object-signal regex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str) -> QuireDocument {
        crate::parse_document(text)
    }

    use crate::grammar::{GrammarLexicon, ObservableVerbs, VacuousPredicates};

    fn empty_lex() -> GrammarLexicon {
        GrammarLexicon::empty()
    }

    fn lex(terms: &[&str]) -> GrammarLexicon {
        GrammarLexicon::from_terms(terms.iter().copied())
    }

    fn verbs() -> ObservableVerbs {
        ObservableVerbs::default()
    }

    fn vacuities() -> VacuousPredicates {
        VacuousPredicates::default()
    }

    /// The vocabularies most tests use: empty lexicon, built-in verb and
    /// vacuity sets. The property idioms are the built-ins too — no `ac` check
    /// reads them (FR-052-CON-1).
    fn vocab<'a>(
        lexicon: &'a GrammarLexicon,
        observable: &'a ObservableVerbs,
        vacuous: &'a VacuousPredicates,
    ) -> GrammarVocabularies<'a> {
        GrammarVocabularies {
            lexicon,
            observable,
            vacuous,
            idioms: crate::grammar::property::default_property_idioms(),
            ambiguous: crate::grammar::default_ambiguity_terms(),
        }
    }

    /// An FR whose `Acceptance Criteria` table carries `cells` as rows.
    fn ac_doc(cells: &[&str]) -> QuireDocument {
        let mut md = String::from(
            "---\nid: FR-001\ntype: FR\n---\n## Acceptance Criteria\n\n\
             | ID | Criteria | Verification |\n|----|----------|--------------|\n",
        );
        for (i, cell) in cells.iter().enumerate() {
            md.push_str(&format!("| FR-001-AC-{} | {cell} | Test |\n", i + 1));
        }
        doc(&md)
    }

    fn findings(cells: &[&str]) -> Vec<GrammarFinding> {
        check(
            "FR",
            &ac_doc(cells),
            0,
            vocab(&empty_lex(), &verbs(), &vacuities()),
        )
    }

    fn count(cells: &[&str], check_id: &str) -> usize {
        findings(cells)
            .iter()
            .filter(|f| f.check == check_id)
            .count()
    }

    // TC-707 (FR-047-AC-1, CR-013): the four shapes — `assertion` is canonical,
    // `obligation` and `given-when-then` are recognized renderings, and a cell
    // with no structure and no observable signal is `unstructured` and yields
    // one `unclassifiable` finding.
    #[test]
    fn tc707_shape_classification() {
        let shape = |text: &str| classify(text, vocab(&empty_lex(), &verbs(), &vacuities()));

        // Canonical: asserts an outcome directly.
        assert_eq!(
            shape("The parser returns the parsed record."),
            AcShape::Assertion
        );
        // A concrete-object signal is enough of an outcome to be an assertion.
        assert_eq!(
            shape("The merged map contains `ac:unclassifiable`."),
            AcShape::Assertion
        );
        // A modal makes it an obligation, not an assertion.
        assert_eq!(
            shape("The system shall reject the request with `403`."),
            AcShape::Obligation
        );
        assert_eq!(
            shape("Given a valid token, when the user submits, then the request is accepted."),
            AcShape::GivenWhenThen
        );
        // No predicate at all — a bare noun phrase (CR-014).
        assert_eq!(shape("Structural evaluation"), AcShape::Unstructured);
        assert_eq!(shape("Type Check"), AcShape::Unstructured);
        // …whereas a vacuous-but-predicated cell is an assertion; its defect is
        // `vacuous-outcome`, not shape.
        assert_eq!(shape("Navigation works"), AcShape::Assertion);

        // `unclassifiable` fires on the unstructured cell only.
        assert_eq!(count(&["Structural evaluation"], "unclassifiable"), 1);
        assert_eq!(count(&["Navigation works"], "unclassifiable"), 0);
        for clean in [
            "The parser returns the parsed record.",
            "The system shall reject the request with `403`.",
            "Given a token, when the user submits, then `202` is returned.",
        ] {
            assert_eq!(count(&[clean], "unclassifiable"), 0, "{clean}");
        }
    }

    // TC-708 (FR-047-AC-2): a modal-free cell is still segmented and checked;
    // an empty cell yields no statement at all.
    #[test]
    fn tc708_every_non_empty_cell_is_a_statement() {
        // No `shall` anywhere — EARS would segment nothing; `ac` checks it.
        let f = findings(&["It all works end to end."]);
        assert!(!f.is_empty(), "modal-free cell must still be checked");

        // An empty cell yields nothing (no unclassifiable, no outcome finding).
        assert!(findings(&[""]).is_empty());
        assert!(findings(&["   "]).is_empty());
    }

    // TC-709 (FR-047-AC-3): two obligations yield exactly one `non-singular`;
    // the positive/negative pair idiom yields none.
    #[test]
    fn tc709_non_singular_and_pair_idiom() {
        assert_eq!(
            count(
                &["The system shall emit `A` and shall persist `B`."],
                "non-singular"
            ),
            1
        );
        assert_eq!(
            count(
                &[
                    "Given a token, when it expires, then `401` is returned, and then the \
                   session is cleared."
                ],
                "non-singular"
            ),
            1
        );
        // Pair idiom: one behaviour, both directions.
        assert_eq!(
            count(
                &[
                    "A conflicting redeclaration yields a `Duplicate` finding; an identical \
                   one yields none."
                ],
                "non-singular"
            ),
            0
        );
        assert_eq!(
            count(
                &["The system shall emit `A` for a match; it shall emit none otherwise."],
                "non-singular"
            ),
            0
        );
    }

    // TC-775 (FR-047-AC-15, CR-024): the pair idiom is recognized by the second
    // obligation, not by the separator joining the two.
    #[test]
    fn tc775_pair_idiom_is_not_separator_bound() {
        // Directly negated second modal — the dominant corpus form.
        assert_eq!(
            count(
                &["When `enabled === false`, the task shall render `skipped` and shall not execute"],
                "non-singular"
            ),
            0
        );
        // Negation inside the second obligation's clause, either side of its modal.
        assert_eq!(
            count(
                &[
                    "Given a non-GitHub remote, the system shall set `git_url` but `github_url` \
                   shall be None"
                ],
                "non-singular"
            ),
            0
        );
        assert_eq!(
            count(
                &["The command shall reject with exit code 2. No Secret deletion shall occur."],
                "non-singular"
            ),
            0
        );
        assert_eq!(
            count(
                &["The field shall be present in the body; otherwise it shall be omitted"],
                "non-singular"
            ),
            0
        );

        // Two *positive* obligations joined the same way still flag.
        assert_eq!(
            count(
                &["The system shall emit `A` and shall persist `B`."],
                "non-singular"
            ),
            1
        );
        // `not` inside the condition is not a negative face: the second
        // obligation is positive and independent.
        assert_eq!(
            count(
                &[
                    "When the record is not found the API shall return `404` and shall log the \
                   attempt"
                ],
                "non-singular"
            ),
            1
        );
        // Three obligations are never suppressed, however worded.
        assert_eq!(
            count(
                &[
                    "The value shall be processed in bounded time. The verifier shall return it \
                   preserved; it shall not hang."
                ],
                "non-singular"
            ),
            1
        );
    }

    // TC-776 (FR-047-AC-16, CR-024): `then` separates obligations only in a
    // Given/When/Then criterion that states no modal.
    #[test]
    fn tc776_then_counts_only_in_a_gwt_criterion() {
        // A precedence chain: no modal, no Given/When — one resolution rule.
        assert_eq!(
            count(
                &[
                    "`getGithubToken` resolves safeStorage first, then `GITHUB_TOKEN`, then \
                   `undefined`."
                ],
                "non-singular"
            ),
            0
        );
        assert_eq!(
            count(
                &[
                    "`--config-root`, then `IX_HOME`, then `~/.ix` select the config root in that \
                   precedence"
                ],
                "non-singular"
            ),
            0
        );
        // A genuine Given/When/Then cell with two consequents still flags.
        assert_eq!(
            count(
                &[
                    "Given a token, when it expires, then `401` is returned, and then the session \
                   is cleared."
                ],
                "non-singular"
            ),
            1
        );
    }

    // TC-777 (FR-047-AC-17, CR-025): a vacuous predicate that is also a common
    // noun does not fire on the noun.
    #[test]
    fn tc777_vacuous_predicate_does_not_fire_on_a_noun() {
        assert_eq!(
            count(
                &["A spec requirement node can be traversed to the code functions that implement \
                   it and the tests that verify it"],
                "vacuous-outcome"
            ),
            0
        );
        // The qualified predicate is still vacuous.
        assert_eq!(
            count(
                &["In-process metric collection functions independently of exporters"],
                "vacuous-outcome"
            ),
            1
        );
    }

    // TC-778 (FR-047-AC-18, CR-026): a double-backtick span masks its whole
    // body, including the keywords and the single-tick spans inside it.
    #[test]
    fn tc778_double_backtick_spans_are_masked() {
        // The CR-024 example quoted as a fragment containing its own code span.
        let quoting = "The pair idiom is recognized by the second obligation: \
                       ``the task SHALL render `skipped` and SHALL NOT execute`` yields no \
                       finding.";
        assert_eq!(count(&[quoting], "non-singular"), 0);
        assert_eq!(count(&[quoting], "non-canonical-shape"), 0);
        assert_eq!(
            classify(quoting, vocab(&empty_lex(), &verbs(), &vacuities())),
            AcShape::Assertion
        );

        // A longer run does not close a shorter one, and an unbalanced run
        // opens no span — the unquoted modal after it still counts.
        assert_eq!(
            count(
                &[
                    "An unterminated ``span leaves the rest as prose, so the system shall reject \
                   it."
                ],
                "non-canonical-shape"
            ),
            1
        );
    }

    // TC-710 (FR-047-AC-4): a vague outcome verb over an abstract object flags;
    // the same cell with the object in the merged lexicon does not.
    #[test]
    fn tc710_vague_response_reuses_the_lexicon() {
        let cell = "The system shall support pagination.";
        let vague = |lexicon: &GrammarLexicon| {
            check(
                "FR",
                &ac_doc(&[cell]),
                0,
                vocab(lexicon, &verbs(), &vacuities()),
            )
            .iter()
            .filter(|f| f.check == "vague-response")
            .count()
        };
        assert_eq!(vague(&empty_lex()), 1);
        assert_eq!(vague(&lex(&["pagination"])), 0);
    }

    // TC-711 (FR-047-AC-5, CR-014): a cell headed by a vacuous predicate with
    // nothing else to check fires `vacuous-outcome`; the same predicate
    // alongside a concrete signal, a lexicon term, or an observable verb does
    // not. The check detects membership of a *closed* vacuity set rather than
    // absence from the open-ended observable-verb space.
    #[test]
    fn tc711_vacuous_outcome() {
        for vacuous in [
            "Navigation works",
            "PostgreSQL backend works",
            "Built-in conditions work correctly",
            "The result is correct",
        ] {
            assert_eq!(count(&[vacuous], "vacuous-outcome"), 1, "{vacuous}");
        }

        // Suppressed by a concrete-object signal…
        assert_eq!(
            count(&["Navigation works through `Router`"], "vacuous-outcome"),
            0
        );
        // …by an observable-result verb…
        assert_eq!(
            count(
                &["Pagination works and returns the next cursor"],
                "vacuous-outcome"
            ),
            0
        );
        // …and by a merged-lexicon term.
        assert_eq!(
            check(
                "FR",
                &ac_doc(&["Pagination works"]),
                0,
                vocab(&lex(&["pagination"]), &verbs(), &vacuities()),
            )
            .iter()
            .filter(|f| f.check == "vacuous-outcome")
            .count(),
            0
        );

        // A substantive criterion never fires, whatever verb it uses — the
        // ecosystem fit check found the old allowlist flagging these.
        for fine in [
            "Semantic search ranks by relevance",
            "Cache does not exceed max_size entries",
            "A partially-built index is cleaned up on recovery",
            "Volumes are correctly mounted into the container",
        ] {
            assert_eq!(count(&[fine], "vacuous-outcome"), 0, "{fine}");
        }
    }

    // TC-712 (FR-047-AC-6, CR-014): the grammar runs on the `Acceptance
    // Criteria` `Criteria` column of every requirement archetype that carries
    // one, plus `### <doc-id>-AC-N` supplements — but on nothing else. An FR
    // `Constraints` cell and an NFR `Statement` remain EARS territory.
    #[test]
    fn tc712_binding() {
        let constraints = doc("---\nid: FR-001\ntype: FR\n---\n## Constraints\n\n\
             | ID | Constraint | Type | Validation |\n|----|------------|------|------------|\n\
             | FR-001-CON-1 | Structural evaluation | Operational | Inspection |\n");
        assert!(check(
            "FR",
            &constraints,
            0,
            vocab(&empty_lex(), &verbs(), &vacuities())
        )
        .is_empty());

        let nfr_statement =
            doc("---\nid: NFR-001\ntype: NFR\n---\n## Statement\n\nStructural evaluation\n");
        assert!(check(
            "NFR",
            &nfr_statement,
            0,
            vocab(&empty_lex(), &verbs(), &vacuities())
        )
        .is_empty());

        // CR-020: the archetypes whose module contract declares a
        // binding-criteria table, each under its own heading.
        for (archetype, section, kind) in [
            ("FR", "Acceptance Criteria", "AC"),
            ("NFR", "Acceptance Criteria", "AC"),
        ] {
            let md = format!(
                "---\nid: {archetype}-001\ntype: {archetype}\n---\n## {section}\n\n\
                 | ID | Criteria | Verification |\n|----|----------|--------------|\n\
                 | {archetype}-001-{kind}-1 | Structural evaluation | Test |\n"
            );
            let found = check(
                archetype,
                &doc(&md),
                0,
                vocab(&empty_lex(), &verbs(), &vacuities()),
            );
            assert!(
                found.iter().any(|f| f.check == "unclassifiable"),
                "{archetype} criteria must be checked"
            );
        }

        // StR's criteria live under `## Validation Criteria` with a
        // `Validation` methods column and `-VC-` ids — the ISO 29148
        // validation/verification split is kept, only the table shape is
        // unified (spec-artifacts-iso#9).
        let str_doc = doc(
            "---\nid: StR-001\ntype: StR\n---\n## Validation Criteria\n\n\
             | ID | Criteria | Validation |\n|----|----------|------------|\n\
             | StR-001-VC-1 | Structural evaluation | Demonstration |\n",
        );
        assert!(
            check(
                "StR",
                &str_doc,
                0,
                vocab(&empty_lex(), &verbs(), &vacuities())
            )
            .iter()
            .any(|f| f.check == "unclassifiable"),
            "StR validation criteria must be checked under their own heading"
        );

        // …and an StR `## Acceptance Criteria` section is NOT its binding
        // section, so it contributes nothing here. Flagging it is the module's
        // `forbidden_section` lint job, not this grammar's.
        let str_wrong_heading = doc(
            "---\nid: StR-001\ntype: StR\n---\n## Acceptance Criteria\n\n\
             | ID | Criteria | Verification |\n|----|----------|--------------|\n\
             | StR-001-AC-1 | Structural evaluation | Test |\n",
        );
        assert!(check(
            "StR",
            &str_wrong_heading,
            0,
            vocab(&empty_lex(), &verbs(), &vacuities())
        )
        .is_empty());

        // US and IT declare no binding-criteria table. A US's acceptance
        // examples are non-binding by design, so even an improvised
        // `## Acceptance Criteria` table on one is not read as criteria —
        // reading it would treat discovery context as verification criteria.
        for archetype in ["US", "IT"] {
            let md = format!(
                "---\nid: {archetype}-001\ntype: {archetype}\n---\n## Acceptance Criteria\n\n\
                 | ID | Criteria | Verification |\n|----|----------|--------------|\n\
                 | {archetype}-001-AC-1 | Structural evaluation | Test |\n"
            );
            assert!(
                check(
                    archetype,
                    &doc(&md),
                    0,
                    vocab(&empty_lex(), &verbs(), &vacuities())
                )
                .is_empty(),
                "{archetype} declares no binding-criteria table and must not be read"
            );
        }

        // An archetype with no AC table contributes nothing.
        let adr = doc("---\nid: ADR-001\ntype: ADR\n---\n## Decision\n\nStructural evaluation\n");
        assert!(check("ADR", &adr, 0, vocab(&empty_lex(), &verbs(), &vacuities())).is_empty());

        // The same prose inside a supplement section IS checked.
        let supplement = doc(
            "---\nid: FR-001\ntype: FR\n---\n## Notes\n\n### FR-001-AC-1\n\nStructural evaluation\n",
        );
        assert!(!check(
            "FR",
            &supplement,
            0,
            vocab(&empty_lex(), &verbs(), &vacuities())
        )
        .is_empty());

        // A supplement is matched under the archetype's own sub-id kind: StR
        // supplements are `-VC-`, and an `-AC-` heading on an StR is not one.
        let vc_supplement = doc(
            "---\nid: StR-001\ntype: StR\n---\n## Notes\n\n### StR-001-VC-1\n\nStructural evaluation\n",
        );
        assert!(!check(
            "StR",
            &vc_supplement,
            0,
            vocab(&empty_lex(), &verbs(), &vacuities())
        )
        .is_empty());
        let ac_supplement_on_str = doc(
            "---\nid: StR-001\ntype: StR\n---\n## Notes\n\n### StR-001-AC-1\n\nStructural evaluation\n",
        );
        assert!(check(
            "StR",
            &ac_supplement_on_str,
            0,
            vocab(&empty_lex(), &verbs(), &vacuities())
        )
        .is_empty());
    }

    // TC-713 (FR-047-AC-7): finding fields — grammar, check id, excerpt, line,
    // shape, severity.
    #[test]
    fn tc713_finding_fields() {
        let f = findings(&["Structural evaluation"]);
        let u = f.iter().find(|x| x.check == "unclassifiable").unwrap();
        assert_eq!(u.grammar, "ac");
        // `pattern` carries the detected *shape*; the check id names the defect.
        assert_eq!(u.pattern.as_deref(), Some("unstructured"));
        assert_eq!(u.statement, "Structural evaluation");
        assert!(u.line.is_some());
        assert_eq!(u.severity, GrammarSeverity::Warning);
    }

    // TC-751 (FR-047-AC-10): a GWT cell yields one `non-canonical-shape` while
    // still classifying `given-when-then`, and its other checks run on the
    // `Then` clause; an EARS cell yields none.
    #[test]
    fn tc751_non_canonical_shape() {
        // GWT: flagged, still classified `given-when-then`, other checks run on
        // the `Then` clause.
        let gwt = "Given a request, when it is unauthenticated, then it works correctly.";
        assert_eq!(count(&[gwt], "non-canonical-shape"), 1);
        assert!(findings(&[gwt])
            .iter()
            .all(|x| x.pattern.as_deref() == Some("given-when-then")));
        // The `Then` clause is vacuous ("it works correctly") → the outcome
        // check still runs on it.
        assert_eq!(count(&[gwt], "vacuous-outcome"), 1);
        assert_eq!(
            count(
                &["Given a request, when it is unauthenticated, then the API returns `401`."],
                "vacuous-outcome"
            ),
            0
        );

        // Obligation: also flagged (CR-013 — an AC states an observation, not an
        // obligation), still classified `obligation`, checks run on the response
        // clause after the modal.
        let obligation = "The system shall reject the request with `403`.";
        assert_eq!(count(&[obligation], "non-canonical-shape"), 1);
        assert!(findings(&[obligation])
            .iter()
            .all(|x| x.pattern.as_deref() == Some("obligation")));
        assert_eq!(count(&[obligation], "vacuous-outcome"), 0);

        // The canonical assertion yields none.
        assert_eq!(
            count(
                &["The parser returns the parsed record."],
                "non-canonical-shape"
            ),
            0
        );
        // …nor does an unstructured cell: that is `unclassifiable`'s business.
        assert_eq!(count(&["Structural evaluation"], "non-canonical-shape"), 0);
    }

    // TC-763 (FR-047-AC-14): a predication whose copula is elided is a
    // predicate. An existential/quantifier head or a predicative adjective
    // asserts something testable while carrying no inflected verb, no modal and
    // no concrete signal, so before CR-019 it drew a spurious `unclassifiable`
    // finding — the check's largest false-positive class (agent-ix/quire-rs#19).
    #[test]
    fn tc763_elided_copula_is_a_predicate() {
        let (lx, vb, vc) = (empty_lex(), verbs(), vacuities());
        let v = vocab(&lx, &vb, &vc);

        // Existential / quantifier heads.
        for s in [
            "No refresh_token in response body",
            "No credential-material field present",
            "Only one active password_reset token per user at any time",
        ] {
            assert_eq!(count(&[s], "unclassifiable"), 0, "{s}");
            assert_eq!(classify(s, v), AcShape::Assertion, "{s}");
        }

        // Predicative adjectives with the copula elided.
        for s in [
            "Loki datasource visible in Grafana",
            "Lockout response identical to normal failure",
            "Password-reset request response time independent of email existence",
        ] {
            assert_eq!(count(&[s], "unclassifiable"), 0, "{s}");
            assert_eq!(classify(s, v), AcShape::Assertion, "{s}");
        }

        // The true positives are untouched: a heading, a bare noun phrase and a
        // dangling prose fragment authored into a `Criteria` column still have
        // no predicate of any kind.
        for s in [
            "Structural evaluation",
            "Key Generation",
            "Sparse Checkout",
            "Feature Detection",
            "Current validation note:",
        ] {
            assert_eq!(count(&[s], "unclassifiable"), 1, "{s}");
            assert_eq!(classify(s, v), AcShape::Unstructured, "{s}");
        }
    }

    // TC-761 (FR-047-AC-13): a keyword inside a code span is a *mention*, not a
    // use — it decides neither the shape nor the obligation count — while the
    // vocabulary checks still read the real words inside the span (CR-017).
    #[test]
    fn tc761_quoted_keywords_are_mentions() {
        // A criterion about the grammar, quoting `shall` as example data. It is
        // a canonical assertion of the checker's behaviour, not an obligation.
        let quoting_modal = "A statement with two `shall` clauses yields exactly one \
                             `non-singular` finding; an enumerated `The X SHALL:` stem \
                             followed by a numbered list yields none.";
        assert_eq!(count(&[quoting_modal], "non-canonical-shape"), 0);
        assert_eq!(count(&[quoting_modal], "non-singular"), 0);
        assert_eq!(
            classify(quoting_modal, vocab(&empty_lex(), &verbs(), &vacuities())),
            AcShape::Assertion
        );

        // Same for a quoted Given/When/Then.
        let quoting_gwt = "A `Given`/`When`/`Then` cell is classified `given-when-then`.";
        assert_eq!(count(&[quoting_gwt], "non-canonical-shape"), 0);

        // An *unquoted* keyword still means what it says — masking must not
        // disarm the checks themselves.
        assert_eq!(
            count(
                &["The system shall reject the request with `403`."],
                "non-canonical-shape"
            ),
            1
        );
        assert_eq!(
            count(
                &["Given a request, when it is stale, then the API returns `410`."],
                "non-canonical-shape"
            ),
            1
        );

        // The signal checks keep reading the span: masking neutralizes keywords
        // for shape detection only, so a backticked identifier still counts as a
        // concrete-object signal and suppresses `vacuous-outcome` (FR-042).
        assert_eq!(count(&["It all works."], "vacuous-outcome"), 1);
        assert_eq!(count(&["`Navigation` works."], "vacuous-outcome"), 0);
        // …and a merged lexicon term inside a span still suppresses
        // `vague-response` (FR-043-AC-3).
        let vague = "The system shall support pagination.";
        assert_eq!(count(&[vague], "vague-response"), 1);
        let f = check(
            "FR",
            &ac_doc(&["The system shall support `pagination`."]),
            0,
            vocab(&lex(&["pagination"]), &verbs(), &vacuities()),
        );
        assert_eq!(f.iter().filter(|x| x.check == "vague-response").count(), 0);

        // Sentence segmentation does not cut inside a code span: an SQL example
        // containing `. ` stays one statement, so the modal that follows it is
        // still outside a span and still classifies `obligation`.
        let d = doc(
            "---\nid: FR-001\ntype: FR\n---\n## Notes\n\n### FR-001-AC-1\n\n\
             `EXPLAIN (FORMAT JSON) SELECT ... ORDER BY col LIMIT 10` on an indexed table \
             SHALL include a `\"Stats\"` group.\n",
        );
        let f = check("FR", &d, 0, vocab(&empty_lex(), &verbs(), &vacuities()));
        assert_eq!(
            f.iter()
                .filter(|x| x.check == "non-canonical-shape")
                .count(),
            1,
            "the modal after an embedded example must still be seen: {f:?}"
        );

        // …and an unbalanced backtick does not swallow the following sentence
        // boundaries either: both readers agree that it opens no span.
        let d = doc(
            "---\nid: FR-001\ntype: FR\n---\n## Notes\n\n### FR-001-AC-1\n\n\
             The loader emits a `partial identifier. The system shall reject it.\n",
        );
        let f = check("FR", &d, 0, vocab(&empty_lex(), &verbs(), &vacuities()));
        assert!(
            f.iter()
                .any(|x| x.statement.starts_with("The system shall")),
            "the second sentence must still be its own statement: {f:?}"
        );

        // An unbalanced backtick opens no span, so nothing is masked.
        assert_eq!(
            count(
                &["The system shall return a `partial identifier."],
                "non-canonical-shape"
            ),
            1
        );
    }

    // TC-754 (FR-047-AC-11): fenced blocks and blockquotes inside a supplement
    // section are skipped; the surrounding prose is still checked.
    #[test]
    fn tc754_supplement_skip_rules() {
        let d = doc(
            "---\nid: FR-001\ntype: FR\n---\n## Notes\n\n### FR-001-AC-1\n\n\
             The loader emits a `Duplicate` diagnostic.\n\n\
             ```\nIt all works end to end.\n```\n\n\
             > It all works in quotes.\n",
        );
        let f = check("FR", &d, 0, vocab(&empty_lex(), &verbs(), &vacuities()));
        assert!(
            !f.iter().any(|x| x.statement.contains("works end to end")),
            "fenced content must not be segmented"
        );
        assert!(
            !f.iter().any(|x| x.statement.contains("works in quotes")),
            "blockquote content must not be segmented"
        );
        // The surrounding prose IS segmented and checked — it carries a
        // backticked identifier and an observable verb, so it is a clean
        // canonical assertion and yields nothing.
        assert!(
            f.iter()
                .all(|x| !x.statement.contains("emits a `Duplicate`")),
            "canonical supplement prose must be clean: {f:?}"
        );

        // A vacuous supplement line proves the prose really is segmented.
        let vacuous =
            doc("---\nid: FR-001\ntype: FR\n---\n## Notes\n\n### FR-001-AC-1\n\nIt all works.\n");
        assert!(check(
            "FR",
            &vacuous,
            0,
            vocab(&empty_lex(), &verbs(), &vacuities())
        )
        .iter()
        .any(|x| x.check == "vacuous-outcome"));
    }

    // TC-757 (FR-047-AC-12, CR-014): both vocabularies are module data — a
    // module-added observable verb suppresses `vacuous-outcome`, a module-added
    // vacuous predicate extends the vacuity set, and with no declaration both
    // built-in sets apply unchanged.
    #[test]
    fn tc757_vocabularies_are_module_data() {
        // ── observable verbs suppress ──
        let cell = "Navigation works once the pipeline surfaces the outcome";
        let flagged = |observable: &ObservableVerbs| {
            check(
                "FR",
                &ac_doc(&[cell]),
                0,
                vocab(&empty_lex(), observable, &vacuities()),
            )
            .iter()
            .filter(|f| f.check == "vacuous-outcome")
            .count()
        };
        // Built-in defaults only: `surfaces` is not an observable-result verb.
        assert_eq!(flagged(&ObservableVerbs::default()), 1);
        // A module declaring it extends the vocabulary and suppresses.
        assert_eq!(
            flagged(&ObservableVerbs::with_module_verbs(
                ["surface"].iter().copied()
            )),
            0
        );
        // The built-ins survive a module declaration (lowest precedence, not
        // replaced): `returns` still suppresses.
        assert_eq!(
            check(
                "FR",
                &ac_doc(&["Navigation works and returns the next cursor"]),
                0,
                vocab(
                    &empty_lex(),
                    &ObservableVerbs::with_module_verbs(["surface"].iter().copied()),
                    &vacuities(),
                ),
            )
            .iter()
            .filter(|f| f.check == "vacuous-outcome")
            .count(),
            0
        );

        // ── vacuous predicates extend ──
        // No observable verb in the sentence, so only the vacuity vocabulary decides.
        let acceptable = "The outcome is acceptable";
        let vacuity_flagged = |vacuous: &VacuousPredicates| {
            check(
                "FR",
                &ac_doc(&[acceptable]),
                0,
                vocab(&empty_lex(), &verbs(), vacuous),
            )
            .iter()
            .filter(|f| f.check == "vacuous-outcome")
            .count()
        };
        // Built-ins alone do not know `is acceptable`.
        assert_eq!(vacuity_flagged(&VacuousPredicates::default()), 0);
        // A module declaring it does.
        assert_eq!(
            vacuity_flagged(&VacuousPredicates::with_module_predicates(
                ["is acceptable"].iter().copied()
            )),
            1
        );
    }
}

#[cfg(test)]
mod props_metamorphic {
    use proptest::prelude::*;

    /// Backtick-dense strings, so runs of every length and unbalanced tails are
    /// reached. A `.*` strategy essentially never produces a closed code span.
    fn tickish() -> impl Strategy<Value = String> {
        let tok = prop_oneof![
            Just("`".to_string()),
            Just("``".to_string()),
            Just("```".to_string()),
            Just("\n".to_string()),
            "[a-z]{1,3}".prop_map(|s| s),
        ];
        proptest::collection::vec(tok, 0..14).prop_map(|v| v.concat())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(4000))]

        /// TC-894 (FR-047-AC-18, CR-017/CR-026, CR-069, Property): masking is
        /// idempotent — a masked span masks to itself.
        #[test]
        fn tc894_mask_code_spans_is_idempotent(s in tickish()) {
            let once = super::mask_code_spans(&s);
            prop_assert_eq!(super::mask_code_spans(&once), once.clone(), "input={:?}", s);
        }

        /// TC-894 (FR-047-AC-18, CR-069, Property): masking preserves byte
        /// length. `outcome_clause` locates a keyword in the masked copy and
        /// slices the *original* with that offset, so any drift here would
        /// silently mis-slice rather than fail loudly.
        #[test]
        fn tc894_mask_code_spans_preserves_byte_length(s in tickish()) {
            prop_assert_eq!(super::mask_code_spans(&s).len(), s.len(), "input={:?}", s);
        }

        /// TC-894 (FR-047-AC-18, CR-069, Property): masking never *introduces* a
        /// backtick, and every backtick it keeps is at the offset it already
        /// occupied — the output's backtick positions are a subset of the
        /// input's. Equality would be wrong: a backtick inside a span is span
        /// content and is masked like any other byte (`` ```\n`\n``` `` is the
        /// witness). Subset is the invariant that matters, because a mask that
        /// could synthesize a delimiter would change which spans a second pass
        /// sees, and idempotence would be luck rather than structure.
        #[test]
        fn tc894_mask_code_spans_never_introduces_a_backtick(s in tickish()) {
            let out = super::mask_code_spans(&s);
            let positions = |t: &str| -> std::collections::BTreeSet<usize> {
                t.bytes().enumerate().filter(|(_, b)| *b == b'`').map(|(i, _)| i).collect()
            };
            let (before, after) = (positions(&s), positions(&out));
            prop_assert!(after.is_subset(&before), "input={:?} masked={:?}", s, out);
        }
    }
}
