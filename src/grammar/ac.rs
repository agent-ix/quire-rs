//! The acceptance-criteria grammar (`ac`, FR-047) — second grammar in the
//! `iso-spec-core` bundle.
//!
//! Where EARS ([`super::ears`]) checks the normative *statements* of a
//! requirement, `ac` checks its *acceptance criteria*: the cells of the FR
//! `Acceptance Criteria` table's `Criteria` column, plus any
//! `### <doc-id>-AC-N` supplement section whose body elaborates one row.
//!
//! Two things differ from the EARS grammar by design:
//!
//! - **No modal-verb filter.** Every non-empty criteria cell is a statement.
//!   An acceptance criterion with no `shall` is still a criterion, and the
//!   checks below are exactly the ones that catch such a cell.
//! - **Shape, not pattern.** A criterion is classified `ears` (the canonical
//!   shape), `given-when-then` (recognized, but steered toward EARS via the
//!   `non-canonical-shape` check so property extraction can still consume
//!   legacy GWT cells), or `unclassifiable`.
//!
//! Every check ships advisory (`warning`); promotion is a per-check policy
//! lever (FR-048) gated on a corpus baseline sweep (FR-047-CON-1).

use std::sync::OnceLock;

use regex::Regex;

use super::ears;
use super::{GrammarFinding, GrammarLexicon, GrammarSeverity, ObservableVerbs};
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

/// The section heading carrying the criteria table, and the column within it.
const CRITERIA_SECTION: &str = "Acceptance Criteria";
const CRITERIA_COLUMN: &str = "Criteria";

/// One acceptance criterion with its 1-based document line.
struct Stmt {
    text: String,
    line: Option<usize>,
}

/// Check `doc` against the `ac` grammar. The grammar binds to the FR
/// `Acceptance Criteria` `Criteria` column and to `### <doc-id>-AC-N`
/// supplement sections; every other archetype and section yields nothing
/// (FR-047-AC-6).
pub fn check(
    archetype: &str,
    doc: &QuireDocument,
    line_offset: usize,
    lexicon: &GrammarLexicon,
    observable: &ObservableVerbs,
) -> Vec<GrammarFinding> {
    if archetype != "FR" {
        return Vec::new();
    }
    let mut stmts: Vec<Stmt> = Vec::new();
    if let Some(section) = query::section(doc, CRITERIA_SECTION) {
        stmts.extend(criteria_cells(section, line_offset));
    }
    for section in supplement_sections(&doc.sections) {
        stmts.extend(supplement_statements(section, line_offset));
    }

    let mut findings = Vec::new();
    for stmt in &stmts {
        check_statement(stmt, lexicon, observable, &mut findings);
    }
    findings
}

fn check_statement(
    stmt: &Stmt,
    lexicon: &GrammarLexicon,
    observable: &ObservableVerbs,
    out: &mut Vec<GrammarFinding>,
) {
    let shape = classify(&stmt.text, lexicon, observable);
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
            "criterion names neither an input nor an observable outcome; there is nothing to test \
             against"
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
    if obligation_count(&stmt.text) > 1 {
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
    if let Some(verb) = ears::vague_verb_in_clause(outcome, lexicon) {
        push(
            out,
            "vague-response",
            format!("vague outcome verb `{verb}`; name a concrete, verifiable outcome"),
        );
    }
    if !is_observable(outcome, lexicon, observable) {
        push(
            out,
            "no-observable-outcome",
            "outcome clause names no externally checkable result (no returned value, emitted \
             record, identifier, or bound)"
                .to_string(),
        );
    }
}

// ─── Classification ─────────────────────────────────────────────────────────

/// Classify one criterion into an [`AcShape`] (CR-013).
///
/// Structure first: a modal verb makes it an obligation (a `When … shall …`
/// criterion is an EARS event pattern, not a GWT trigger), then Given/When/Then.
/// What remains is an `assertion` when it carries an observable signal — the
/// canonical shape — and `unstructured` when it carries nothing at all.
pub fn classify(
    statement: &str,
    lexicon: &GrammarLexicon,
    observable: &ObservableVerbs,
) -> AcShape {
    if ears::classify(statement) != ears::EarsPattern::Unclassifiable {
        return AcShape::Obligation;
    }
    if is_given_when_then(statement) {
        return AcShape::GivenWhenThen;
    }
    if is_observable(statement, lexicon, observable) {
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

/// The number of independent obligations in a criterion: `shall` occurrences
/// or `Then` clauses, whichever the criterion uses. The positive/negative pair
/// idiom (`X yields a finding; Y yields none`) states one behaviour in two
/// directions and counts as a single obligation (FR-047-AC-3).
fn obligation_count(statement: &str) -> usize {
    let lower = ears::normalize(statement).to_lowercase();
    let count = re_shall_word()
        .find_iter(&lower)
        .count()
        .max(re_then().find_iter(&lower).count());
    if count == 2 && is_positive_negative_pair(&lower) {
        return 1;
    }
    count
}

/// The `X yields a finding; Y yields none` idiom: two halves separated by `;`
/// (or ` while `), the second stating the negative case of the same behaviour.
fn is_positive_negative_pair(lower: &str) -> bool {
    let halves: Vec<&str> = lower.split(';').flat_map(|h| h.split(" while ")).collect();
    halves.len() == 2 && re_negative_case().is_match(halves[1])
}

/// The outcome clause a criterion is judged on: everything after the modal
/// verb for an `ears`-shaped criterion, everything after `Then` for a
/// `given-when-then`-shaped one, and the whole criterion otherwise (an
/// unclassifiable cell has no clause structure to slice).
fn outcome_clause(statement: &str, shape: AcShape) -> &str {
    let tail = |m: Option<regex::Match<'_>>| m.map(|m| &statement[m.end()..]);
    match shape {
        AcShape::Obligation => tail(re_shall_word().find(statement)).unwrap_or(statement),
        AcShape::GivenWhenThen => tail(re_then().find(statement)).unwrap_or(statement),
        // An assertion states its outcome directly, and an unstructured cell has
        // no clause structure to slice — both are judged whole.
        AcShape::Assertion | AcShape::Unstructured => statement,
    }
}

/// True when the outcome clause names an externally checkable result: a
/// concrete-object signal per FR-042 (a backticked identifier, a numeric or
/// comparison bound, or a merged-lexicon term) or an observable-result verb
/// from the module-data vocabulary (FR-047-AC-5/AC-12).
fn is_observable(clause: &str, lexicon: &GrammarLexicon, observable: &ObservableVerbs) -> bool {
    let normalized = ears::normalize(clause);
    re_concrete_object_signal().is_match(&normalized)
        || lexicon.contains_term(&normalized)
        || observable.contains_verb(&normalized)
}

// ─── Statement extraction ───────────────────────────────────────────────────

/// Every non-empty `Criteria` cell is one statement — no modal-verb filter
/// (FR-047-AC-2). A missing section, table, or column yields none.
fn criteria_cells(section: &QuireSection, line_offset: usize) -> Vec<Stmt> {
    let Some(table) = query::parse_table(&section.content) else {
        return Vec::new();
    };
    let Some(col_idx) = table
        .headers
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case(CRITERIA_COLUMN))
    else {
        return Vec::new();
    };
    table
        .rows
        .iter()
        .filter_map(|row| row.get(col_idx))
        .map(|cell| cell.trim())
        .filter(|cell| !cell.is_empty())
        .map(|cell| Stmt {
            text: cell.to_string(),
            line: ears::locate_line(section, cell, line_offset),
        })
        .collect()
}

/// Every `### <doc-id>-AC-N` section in the document tree — the supplement
/// body of one criteria row (e.g. `### FR-047-AC-1`).
fn supplement_sections(sections: &[QuireSection]) -> Vec<&QuireSection> {
    let mut out = Vec::new();
    for s in sections {
        if re_ac_supplement_heading().is_match(s.heading.trim()) {
            out.push(s);
        }
        out.extend(supplement_sections(&s.children));
    }
    out
}

/// Segment a supplement body into statements: one per sentence of prose, with
/// fenced code blocks and blockquotes skipped per the FR-042 skip rules
/// (FR-047-AC-11). No modal-verb filter, as with the criteria cells.
fn supplement_statements(section: &QuireSection, line_offset: usize) -> Vec<Stmt> {
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
        let line = ears::strip_list_marker(trimmed);
        for fragment in line.split(". ") {
            let text = fragment.trim();
            if text.is_empty() {
                continue;
            }
            out.push(Stmt {
                text: text.to_string(),
                line: Some(ears::abs_line(section, rel, line_offset)),
            });
        }
    }
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
fn re_negative_case() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)\b(none|no|not|never|neither|nothing|zero|without)\b")
            .expect("negative-case regex")
    })
}

/// `### <doc-id>-AC-N` — an acceptance-criterion supplement heading.
fn re_ac_supplement_heading() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)^[A-Za-z]+-\d+-AC-\d+\b").expect("ac supplement heading regex")
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

    fn empty_lex() -> GrammarLexicon {
        GrammarLexicon::empty()
    }

    fn lex(terms: &[&str]) -> GrammarLexicon {
        GrammarLexicon::from_terms(terms.iter().copied())
    }

    fn verbs() -> ObservableVerbs {
        ObservableVerbs::default()
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
        check("FR", &ac_doc(cells), 0, &empty_lex(), &verbs())
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
        let shape = |text: &str| classify(text, &empty_lex(), &verbs());

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
        // No modal, no Given/When/Then, no observable signal.
        assert_eq!(shape("It all works end to end."), AcShape::Unstructured);

        // `unclassifiable` fires on the unstructured cell only.
        assert_eq!(count(&["It all works end to end."], "unclassifiable"), 1);
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

    // TC-710 (FR-047-AC-4): a vague outcome verb over an abstract object flags;
    // the same cell with the object in the merged lexicon does not.
    #[test]
    fn tc710_vague_response_reuses_the_lexicon() {
        let cell = "The system shall support pagination.";
        let vague = |lexicon: &GrammarLexicon| {
            check("FR", &ac_doc(&[cell]), 0, lexicon, &verbs())
                .iter()
                .filter(|f| f.check == "vague-response")
                .count()
        };
        assert_eq!(vague(&empty_lex()), 1);
        assert_eq!(vague(&lex(&["pagination"])), 0);
    }

    // TC-711 (FR-047-AC-5): an outcome clause with neither a concrete-object
    // signal nor an observable verb flags; returned values, emitted records and
    // exit codes do not.
    #[test]
    fn tc711_no_observable_outcome() {
        assert_eq!(
            count(&["The import works correctly."], "no-observable-outcome"),
            1
        );
        assert_eq!(
            count(
                &["The parser returns the parsed record."],
                "no-observable-outcome"
            ),
            0
        );
        assert_eq!(
            count(
                &["The loader emits a `DuplicateEdgeType` diagnostic."],
                "no-observable-outcome"
            ),
            0
        );
        assert_eq!(
            count(&["The command exits with code 1."], "no-observable-outcome"),
            0
        );
    }

    // TC-712 (FR-047-AC-6): the `ac` grammar runs only on its bindings — the FR
    // Criteria column and `### <doc-id>-AC-N` supplements. An FR `Constraints`
    // cell and an NFR `Statement` are not `ac` territory.
    #[test]
    fn tc712_binding() {
        let constraints = doc("---\nid: FR-001\ntype: FR\n---\n## Constraints\n\n\
             | ID | Constraint | Type | Validation |\n|----|------------|------|------------|\n\
             | FR-001-CON-1 | It all works end to end. | Operational | Inspection |\n");
        assert!(check("FR", &constraints, 0, &empty_lex(), &verbs()).is_empty());

        let nfr = doc("---\nid: NFR-001\ntype: NFR\n---\n## Statement\n\nIt all works.\n");
        assert!(check("NFR", &nfr, 0, &empty_lex(), &verbs()).is_empty());

        // The same prose inside a supplement section IS checked.
        let supplement =
            doc("---\nid: FR-001\ntype: FR\n---\n## Notes\n\n### FR-001-AC-1\n\nIt all works.\n");
        assert!(!check("FR", &supplement, 0, &empty_lex(), &verbs()).is_empty());
    }

    // TC-713 (FR-047-AC-7): finding fields — grammar, check id, excerpt, line,
    // shape, severity.
    #[test]
    fn tc713_finding_fields() {
        let f = findings(&["It all works end to end."]);
        let u = f.iter().find(|x| x.check == "unclassifiable").unwrap();
        assert_eq!(u.grammar, "ac");
        // `pattern` carries the detected *shape*; the check id names the defect.
        assert_eq!(u.pattern.as_deref(), Some("unstructured"));
        assert_eq!(u.statement, "It all works end to end.");
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
        assert_eq!(count(&[gwt], "no-observable-outcome"), 1);
        assert_eq!(
            count(
                &["Given a request, when it is unauthenticated, then the API returns `401`."],
                "no-observable-outcome"
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
        assert_eq!(count(&[obligation], "no-observable-outcome"), 0);

        // The canonical assertion yields none.
        assert_eq!(
            count(
                &["The parser returns the parsed record."],
                "non-canonical-shape"
            ),
            0
        );
        // …nor does an unstructured cell: that is `unclassifiable`'s business.
        assert_eq!(
            count(&["It all works end to end."], "non-canonical-shape"),
            0
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
        let f = check("FR", &d, 0, &empty_lex(), &verbs());
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
        assert!(check("FR", &vacuous, 0, &empty_lex(), &verbs())
            .iter()
            .any(|x| x.check == "unclassifiable"));
    }

    // TC-757 (FR-047-AC-12): the observable-verb vocabulary is module data — a
    // module-added verb suppresses `no-observable-outcome`, and with no module
    // declaration the built-in defaults apply unchanged.
    #[test]
    fn tc757_observable_verbs_are_module_data() {
        let cell = "The pipeline surfaces the review outcome.";
        let flagged = |observable: &ObservableVerbs| {
            check("FR", &ac_doc(&[cell]), 0, &empty_lex(), observable)
                .iter()
                .filter(|f| f.check == "no-observable-outcome")
                .count()
        };
        // Built-in defaults only: `surfaces` is not an observable-result verb.
        assert_eq!(flagged(&ObservableVerbs::default()), 1);
        // A module declaring it extends the vocabulary.
        assert_eq!(
            flagged(&ObservableVerbs::with_module_verbs(
                ["surface"].iter().copied()
            )),
            0
        );
        // The built-ins survive the module declaration (lowest precedence, not
        // replaced): `returns` still counts.
        assert_eq!(
            check(
                "FR",
                &ac_doc(&["The parser returns the record."]),
                0,
                &empty_lex(),
                &ObservableVerbs::with_module_verbs(["surface"].iter().copied()),
            )
            .iter()
            .filter(|f| f.check == "no-observable-outcome")
            .count(),
            0
        );
    }
}
