//! Declarative coverage computation (FR-050).
//!
//! The FR→AC→TC→test reconciliation the `gap-analysis` workflow performs by
//! grepping, computed generically: the engine knows nothing of "FR", "AC", or
//! "TC". Everything it reconciles — which documents mint trace ids, which rows
//! reference them, what a status value means — comes from the module-declared
//! traceability model ([`crate::traceability`]), so a module with an entirely
//! different vocabulary gets the same rollup with no engine change.
//!
//! **Data, not judgment.** The report states what is backed, what is not, and
//! what does not reconcile. Verdict policy (PASS / CONDITIONAL / FAIL), review
//! gating, and SpecReview authoring stay in the consuming workflow
//! (FR-050-CON-1).
//!
//! Inputs are the corpus, the registry, and local source trees: no network or
//! service I/O, and no execution of the code the symbols came from
//! (FR-050-CON-2, FR-051-CON-1).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::corpus::declared_tables;
use crate::corpus::spec::Spec;
use crate::grammar::GroundingCounts;
use crate::metric::{Metric, MetricShape};
use crate::obligation::Obligation;
use crate::registry::Registry;
use crate::symbols::trace::{BindingCensus, SymbolGraph, UnmatchedTag};
use crate::traceability::TraceabilityModel;

mod binding_diagnostics;
mod declarations;
mod diagnostics;
mod reconcile;

use declarations::criteria_counts;
use diagnostics::{catch_all_documents, hollow_denominators, uncatalogued_methods};
use reconcile::reconcile;

/// Why a coverage run produced no report.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoverageError {
    /// No active module declares a `traceability:` model, so there is nothing
    /// to reconcile against — a distinct diagnostic, never an empty report
    /// (FR-050-AC-9).
    #[error(
        "no active module declares a `traceability:` model; coverage needs one \
         (declare trace targets, document references, and a status vocabulary \
         in the module manifest)"
    )]
    ModelUndeclared,
}

/// A declared reference row whose trace target has no backing `verifies`
/// relation from any source symbol (FR-050-AC-3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnbackedRow {
    /// The declaration that produced the row.
    pub reference: String,
    /// Document the row lives in, relative to the scope root.
    pub document: String,
    /// The row's own id, when the declaration names a `row_id_column`.
    pub row_id: Option<String>,
    /// Trace ids the row is answerable for — its own id plus the ids it
    /// references — none of which any symbol backs.
    pub target_ids: Vec<String>,
    /// 1-based document line of the matrix row (#210). Optional on the wire so
    /// a payload from an engine predating the field still deserializes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

/// A row whose status classes as `complete` while nothing backs it
/// (FR-050-AC-4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusLie {
    pub reference: String,
    pub document: String,
    pub row_id: Option<String>,
    /// The authored status value.
    pub status: String,
    pub target_ids: Vec<String>,
    /// 1-based document line of the matrix row (#210).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

/// An unbacked row whose declared verification method mints no source symbol
/// (FR-050-AC-16, CR-041) — an eval, an inspection, a demonstration.
///
/// Reported separately from a status lie, because a lie is a row claiming
/// evidence it does not have, and this is a row whose own declared method makes
/// a source tag impossible. Which methods those are is module-declared; the
/// engine has no opinion about which verification produces code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoSymbolRow {
    pub reference: String,
    pub document: String,
    pub row_id: Option<String>,
    /// The declared test-type value that exempts the row.
    pub test_type: String,
    pub target_ids: Vec<String>,
    /// 1-based document line of the matrix row (#210).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

/// A reference row whose authored status classes outside the module's declared
/// status vocabulary (FR-050-AC-21, CR-083).
///
/// [`crate::traceability::StatusClass::Unknown`] is reported rather than
/// discarded. It remains separate from a status lie because the model supplies
/// no class to evaluate (CR-083).
///
/// This is a **vocabulary-drift backstop**, not a verdict on the row. It carries
/// the authored string verbatim so the reader can see which value drifted; the
/// class is deliberately absent, because "no class" is the whole finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndeclaredStatus {
    pub reference: String,
    pub document: String,
    pub row_id: Option<String>,
    /// The authored status value, verbatim.
    pub status: String,
    /// 1-based document line of the matrix row (#210). Two byte-identical
    /// duplicate rows still collapse to ONE record (CR-086); the record
    /// carries the first duplicate's line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

/// A source symbol whose trace tag resolves to no declared target and no
/// declared reference row (FR-050-AC-5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UntrackedSymbol {
    pub path: String,
    pub symbol: String,
    pub trace_id: String,
    /// 1-based declaration line of the tagged symbol (#210).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

/// One `implements` edge as the JSON contract carries it (FR-062).
///
/// Deliberately separate from `untracked_symbols` and from the backed set. This
/// is **scope**, not evidence — which requirement a production symbol is about —
/// and a consumer that folded it into either would let unverified code claim
/// coverage, the backdoor CR-061 closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplementsRecord {
    pub path: String,
    pub symbol: String,
    /// The requirement this code implements.
    pub trace_id: String,
    /// Name of the declared marker form that bound it.
    pub form: String,
}

/// One binding symbol of a shared trace id (FR-050-AC-23).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedTraceSymbol {
    pub path: String,
    pub symbol: String,
}

/// A trace id bound by more than one distinct source symbol (FR-050-AC-23,
/// CR-087).
///
/// The policy this reports on: **one test-case id names one source symbol.**
/// A row backed by N symbols is satisfied by any one of them, so the row can
/// stay green while N−1 of its tests rot or are deleted — the id has stopped
/// naming which evidence backs the row. v0.41.0 shipped two instances
/// (TC-943 ×2, TC-944 ×2) and no surface reported either.
///
/// **Scoped to ids that are row ids of status-carrying rows** — the rows whose
/// green can rot. An id whose rows carry no status (an acceptance criterion
/// verified by several tests, e.g. FR-050-AC-21 by TC-941 *and* TC-942) is
/// legitimately N:1 and never reported; measured unscoped over this repository
/// the list held 100+ such records, which is a rule misreading correct data,
/// not a corpus defect.
///
/// Like `undeclared_statuses` (CR-083), this is a **report list, not a gate**:
/// it does not affect `totals` and `--strict` does not gate on it in this
/// revision — advisory-first, promotion is a separate measured decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedTraceId {
    /// The trace id, exactly as the binding forms yield it.
    pub trace_id: String,
    /// The distinct binding symbols, ordered by `(path, symbol)`. Always at
    /// least two — a uniquely bound id mints no record.
    pub symbols: Vec<SharedTraceSymbol>,
}

/// Backed/total trace-target counts for one minting document (FR-050-AC-6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupCounts {
    /// The minting document, relative to the scope root.
    pub document: String,
    /// Declared target kind the ids belong to.
    pub target: String,
    pub backed: usize,
    pub total: usize,
}

/// One target row the active model minted, with the state needed to open and
/// partition it (FR-050-AC-38, #361).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MintedTargetRecord {
    pub id: String,
    pub target: String,
    pub document: String,
    pub line: usize,
    pub backed: bool,
}

/// Property-shape counts for one document's binding criteria (FR-050-AC-13,
/// CR-028), summarizing what [`crate::grammar::property`] classified.
///
/// A count is data, not a verdict (FR-050-CON-1): a low `property_shaped`
/// share is a description of a corpus, never a failing one — CR-020 already
/// recorded that criteria validated by demonstration legitimately score low.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriteriaCounts {
    /// The document the criteria live in, relative to the scope root.
    pub document: String,
    /// The archetype the document resolved to.
    pub archetype: String,
    /// Binding criteria seen in the document.
    pub criteria: usize,
    /// Criteria a downstream generator can extract a property from.
    pub property_shaped: usize,
    /// Extractable criteria carrying a shape that names what property to write
    /// — everything but the `universal` catch-all (FR-050-AC-28, CR-095).
    ///
    /// Zero — and so absent — for a document whose every extractable criterion
    /// is `universal`, which keeps FR-050-AC-7 byte-identity for a report from
    /// an engine predating the field.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub specific_shaped: usize,
    /// Criteria per property-shape label. `BTreeMap` keeps the histogram
    /// deterministic (NFR-006).
    pub by_property: BTreeMap<String, usize>,
    /// Span-grounding per shape (FR-050-AC-28, CR-095): of each shape's
    /// records, how many carry `domain` / `precondition` / `oracle`.
    ///
    /// The shapes that say the most carried the least: 65 of 67 specific-shape
    /// non-`example` records had zero spans, so `spec-correctness` could not be
    /// driven from the very records that named a property worth writing.
    /// Empty — and so absent — for a document binding no criteria.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub grounding: BTreeMap<String, GroundingCounts>,
    /// One extractable criterion the classifier gave the `universal` catch-all
    /// — a place to look (#261).
    ///
    /// Present only when this document has extractable criteria and a specific
    /// shape for NONE of them, which is also the only time the
    /// `catch-all-universal` diagnostic reads it. Absent otherwise, so a
    /// document with any specifically-shaped criterion keeps the bytes an
    /// engine predating the field emitted (FR-050-AC-7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catch_all_example: Option<CatchAllCriterion>,
}

/// Where one `universal`-shaped criterion is (#261).
///
/// `coverage.specific_shaped` is a ratio over a corpus and names no criterion,
/// so a reader watching it fall has every spec file to search. This is the
/// lowest-lined criterion of the document that has no specifically-shaped one,
/// which makes the number openable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatchAllCriterion {
    /// The criterion's own id, when the table declares one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_id: Option<String>,
    /// 1-based document line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

/// Bundle-wide totals; the backed/total pair equals the sum over
/// [`CoverageReport::groups`], the criteria pair the sum over
/// [`CoverageReport::criteria`].
///
/// The two CR-028 counts are an **all-or-nothing pair**: both are `Some` for a
/// corpus binding any criteria and both are `None` for a corpus binding none.
/// `None` is what keeps FR-050-AC-13's byte-identity — the keys are absent, so
/// a no-criteria report is byte-for-byte what an engine predating the fields
/// would have written. They are `Option` rather than a skipped zero because a
/// corpus with criteria and no property-shaped ones must still emit
/// `property_shaped: 0`: the primary consumer reads this payload as JSON, and
/// an absent key there makes the extraction ratio `NaN` in exactly the case
/// most worth reporting on.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageTotals {
    pub backed: usize,
    pub total: usize,
    /// Binding criteria across the corpus (CR-028), or `None` when the corpus
    /// binds none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria: Option<usize>,
    /// Criteria a generator can extract a property from (CR-028), or `None`
    /// when the corpus binds no criteria at all. Zero is a real value here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_shaped: Option<usize>,
    /// Extractable criteria carrying a shape that names what property to write
    /// (FR-050-AC-28, CR-095). `None` alongside the other two, for the same
    /// reason.
    ///
    /// **The honest half of the headline.** `515/951 extractable (54%)` reads
    /// as "half this specification is property-testable"; 440 of those 515 were
    /// the `universal` catch-all, and the figure for "the classifier said what
    /// property to write" was 78/951 — 8%. Both true, one misleading, and a
    /// summary line carrying only the first is the one a reader repeats.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specific_shaped: Option<usize>,
}

/// The machine-readable coverage report. Every collection is deterministically
/// ordered, so repeated runs over identical inputs serialize byte-identically
/// (FR-050-AC-7).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageReport {
    pub unbacked_rows: Vec<UnbackedRow>,
    pub status_lies: Vec<StatusLie>,
    /// Unbacked rows exempted from `status_lies` by their declared method
    /// (CR-041). Empty — and so absent from the JSON — for a module declaring
    /// no `no_source_symbol` vocabulary, which keeps FR-050-AC-7 byte-identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub no_symbol_rows: Vec<NoSymbolRow>,
    /// Rows whose status value the model classes as nothing (CR-083). Empty —
    /// and so absent from the JSON — for a corpus whose every status value is
    /// declared, which keeps FR-050-AC-7 byte-identity for every module and
    /// repository already conformant.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub undeclared_statuses: Vec<UndeclaredStatus>,
    pub untracked_symbols: Vec<UntrackedSymbol>,
    /// Trace ids bound by more than one distinct source symbol (FR-050-AC-23,
    /// CR-087). Empty — and so absent from the JSON — for a corpus whose every
    /// id is uniquely bound, which keeps FR-050-AC-7 byte-identity for every
    /// repository already conformant. Does not affect `totals`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_trace_ids: Vec<SharedTraceId>,
    pub groups: Vec<GroupCounts>,
    /// Row-level identity behind `groups` and `totals` (#361). Omitted only
    /// when the model minted no rows, preserving the additive v1 contract.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub minted_targets: Vec<MintedTargetRecord>,
    /// Per-document property-shape counts (CR-028). Empty for a corpus whose
    /// documents bind no criteria, so such a report serializes exactly as it
    /// did before the field existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<CriteriaCounts>,
    /// Declarations that scanned nothing, and why (CR-054). Empty — and so
    /// absent from the JSON — for a model whose every declaration selected
    /// something, which keeps FR-050-AC-7 byte-identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<CoverageDiagnostic>,
    /// Every stable reason token this engine can emit in [`Self::diagnostics`].
    /// Published with the payload so fixture inventories can compare against
    /// the engine contract instead of scanning Rust source (#300).
    #[serde(default)]
    pub diagnostic_reason_registry: Vec<String>,
    /// Derived obligation records (FR-053), ordered by source then document
    /// then row. Empty — and so absent from the JSON — for a model declaring no
    /// `obligations:` sources, which keeps FR-050-AC-7 byte-identity for every
    /// module that has not adopted them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligations: Vec<Obligation>,
    /// Requirement → production code (FR-062). Empty — and so absent from the
    /// JSON — for a module declaring no `implements` marker forms, which keeps
    /// FR-050-AC-7 byte-identity for every module that has not adopted them.
    ///
    /// **Carries no weight in `totals`.** Scope is not evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implements: Vec<ImplementsRecord>,
    /// Per-value classification of every declared coverage vocabulary
    /// (FR-059-AC-9, CR-091): owned / excused / unowned, with the deciding
    /// documents. Empty — and so absent from the JSON — for a module declaring
    /// no `vocabulary_coverage`, which keeps FR-050-AC-7 byte-identity for
    /// every module that has not adopted the declaration.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vocabulary_coverage: Vec<VocabularyValueRecord>,
    /// Source files a declared `source_exclude` glob removed from the symbol
    /// walk (FR-050-AC-24, #215). Zero — and so absent from the JSON — for a
    /// model declaring no `source_exclude`, or one whose globs match nothing,
    /// which keeps FR-050-AC-7 byte-identity for every repository already
    /// conformant. Without it an over-broad glob silently drops legitimate
    /// backing and the report reads as a coverage regression, indistinguishable
    /// from tests that were never written.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub excluded_source_files: usize,
    /// What the trace binder looked at and what bound, per language
    /// (FR-050-AC-27, CR-093).
    ///
    /// **Reported unconditionally**, unlike every other list on this report,
    /// and the difference is the point. The rest are defect lists: empty means
    /// nothing to say. This one is the premise the whole report rests on, and a
    /// premise that only appears when it fails is one a reader cannot rely on
    /// when it holds. `1,292 candidates, 0 bound` needs no threshold to be
    /// alarming; `1,292 candidates, 1,290 bound` is a reassurance no previous
    /// version of this payload could give.
    ///
    /// Empty — and so absent — only when the code walk found no evidence symbol
    /// in any language, which is the one case where there is genuinely nothing
    /// to report.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub binding_census: Vec<BindingCensus>,
    /// Row-addressable generic trace ids in evidence-symbol annotation blocks
    /// that no declared form bound on that symbol (FR-050-AC-39, #362).
    /// Omitted when every authored annotation id bound or no such id exists.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched_tags: Vec<UnmatchedTag>,
    /// Every headline number this report emits, with its unit, population,
    /// method and match count (FR-063-AC-3).
    ///
    /// The generalization of `binding_census`: that field names the one premise
    /// the backed/total ratio happens to rest on, and this makes stating the
    /// premise a property of **every** ratio the payload carries. A percentage
    /// without its match counts cannot be constructed.
    ///
    /// Always present — a report emits at least `coverage.backed` — so a
    /// consumer never has to decide whether absence means "no metrics" or "an
    /// engine that predates them".
    #[serde(default)]
    pub metrics: Vec<Metric>,
    /// Things that look wrong, with the measurement that made them look wrong
    /// (FR-064). **Advisory always** — a suspicion is a claim about a shape,
    /// not a verdict, and it affects no total and no exit code.
    ///
    /// Empty — and so absent — for a corpus whose suites all check every
    /// sample, which keeps FR-050-AC-7 byte-identity for every repository
    /// already clean.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suspicions: Vec<crate::skeptic::Suspicion>,
    pub totals: CoverageTotals,
}

/// `skip_serializing_if` predicate for the FR-050-AC-24 count: zero is the
/// no-op case every pre-existing report was in, so it stays off the wire.
fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// A declaration that produced no rows for a reason the operator can act on
/// (CR-054): no scope at all, an archetype no document has, or a declared
/// auxiliary document that could not be read.
///
/// Non-fatal and never an exit code on its own — but since CR-049 made body
/// selection load-bearing on the declaration, a silent one of these is an
/// engine that parsed nothing and reported full coverage of nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageDiagnostic {
    /// The declaration's name, as the model authored it.
    pub declaration: String,
    /// Stable machine token, shared with `quire validate`'s bundle warnings.
    pub reason: String,
    pub message: String,
    /// The unreadable document, when the diagnostic is about one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// One-based source line at the smallest repair locus, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// The vocabulary or catalog value the diagnostic is about, verbatim, when
    /// it is about exactly one (FR-054-AC-12, CR-091).
    /// `uncatalogued-verification-method` carries the authored method here —
    /// the same string the obligation records carry in `method` — so a
    /// consumer joins the two by equality instead of regexing a human
    /// sentence, which was the only recovery path before (#179; quoin#168's
    /// mismatch/uncatalogued split is the filed reader).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Stable structured repair guidance. Optional on input so retained
    /// pre-#364 payloads still deserialize; every current producer populates
    /// it before serialization.
    #[serde(default, flatten)]
    pub guidance: Option<crate::finding::FindingGuidance>,
}

/// Stable reason tokens emitted by coverage diagnostics.
///
/// This is the engine-side registry. Keep reason construction behind these
/// values so consumers can validate fixture vocabularies by equality.
pub const COVERAGE_DIAGNOSTIC_REASONS: &[&str] = &[
    "archetype-matches-nothing",
    "catch-all-universal",
    "hollow-denominator",
    "id-column-matches-nothing",
    "low-symbol-binding",
    "marker-form-mismatch",
    "model-mints-nothing",
    "no-symbol-bound",
    "obligation-row-states-nothing",
    "section-holds-no-table",
    "section-matches-nothing",
    "status-column-matches-nothing",
    "tag-on-non-binding-symbol",
    "untracked-id-near-miss",
    "untracked-id-has-minted-children",
    "uncatalogued-verification-method",
    "undeclared-coverage-vocabulary",
];

/// How one declared vocabulary value stands in the bundle (FR-059-AC-9,
/// CR-091).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VocabularyValueState {
    /// A document of the projected archetype claims it: a requirement exists.
    /// Wins over `excused` when a value is both claimed and recorded absent —
    /// the stale absence record is then the consumer's finding to make.
    Owned,
    /// The declared justified-absence field records it — somebody wrote the
    /// value down as deliberately not addressed. Whether the excuse was earned
    /// is verdict policy and stays out of the engine (FR-059-CON-2).
    Excused,
    /// Nothing claims it and nothing excuses it — the residue the warning
    /// stream reports.
    Unowned,
}

/// One declared vocabulary value, classified (FR-059-AC-9, CR-091).
///
/// The `quire validate` warning stream reports only the unowned residue, so a
/// consumer could not tell an owned value from an excused one — very different
/// facts: one means a requirement exists, the other means the check went quiet
/// because somebody wrote the value into the justified-absence field. Before
/// this record the only recovery was to open every document in the bundle and
/// parse its frontmatter (`agent-ix/quire-rs#179`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabularyValueRecord {
    /// The `vocabulary_coverage` declaration's name.
    pub vocabulary: String,
    /// The projected archetype (the declaration's `from`).
    pub archetype: String,
    /// The frontmatter field whose schema `enum` is the vocabulary.
    pub field: String,
    /// The declared `<check>` severity token, joining the record to the
    /// warning stream's findings.
    pub check: String,
    /// The vocabulary value, verbatim from the schema enum.
    pub value: String,
    /// The classification.
    pub state: VocabularyValueState,
    /// Scope-relative documents that decide the state: the claimants for
    /// `owned`, the justified-absence recorders for `excused`, empty for
    /// `unowned`.
    pub documents: Vec<String>,
}

impl CoverageReport {
    /// The report as JSON — the `quire coverage` stdout payload.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("coverage report serializes")
    }
}

/// Reconcile declared targets, declared reference rows, and scanned source
/// trace tags into a coverage report.
///
/// `root` is the scope root every emitted path is relative to; the symbol
/// `graph` comes from [`crate::symbols::trace::bind`].
pub fn compute(
    spec: &Spec,
    registry: &Registry,
    graph: &SymbolGraph,
    root: &Path,
) -> Result<CoverageReport, CoverageError> {
    let model = registry
        .traceability()
        .ok_or(CoverageError::ModelUndeclared)?;
    let (mut report, minting_census) = reconcile(spec, model, graph, root, registry);
    // CR-028: the criteria counts are computed here rather than inside
    // `reconcile`, which reconciles against the declared model alone and takes
    // no `Registry`. The vocabularies the classifier reads hang off the same
    // `registry` this function already holds.
    report.criteria = criteria_counts(spec, registry, model, root);
    // Set as a pair, so a consumer never sees a criteria count without the
    // property-shaped count it is the denominator of.
    if !report.criteria.is_empty() {
        report.totals.criteria = Some(report.criteria.iter().map(|c| c.criteria).sum());
        report.totals.property_shaped =
            Some(report.criteria.iter().map(|c| c.property_shaped).sum());
        report.totals.specific_shaped =
            Some(report.criteria.iter().map(|c| c.specific_shaped).sum());
    }
    // FR-054-AC-11: reported here rather than in `reconcile`, which takes no
    // `Registry` and so cannot see the catalog the methods are checked against.
    report
        .diagnostics
        .extend(uncatalogued_methods(&report.obligations, registry));
    // FR-059-AC-9 (CR-091): also here rather than in `reconcile`, because the
    // vocabulary lives in an archetype's frontmatter schema, which only the
    // `Registry` can read.
    let (vocabulary_records, dead_declarations) =
        crate::corpus::vocabulary_coverage::coverage_records(spec, registry, root);
    report.vocabulary_coverage = vocabulary_records;
    report.diagnostics.extend(dead_declarations);
    // FR-063: last, because a metric describes a number the steps above
    // finished computing — `criteria` in particular is set here rather than in
    // `reconcile`.
    report.metrics = coverage_metrics(&report, model, graph, minting_census);
    report
        .diagnostics
        .extend(hollow_denominators(&report.metrics));
    // #261: after `criteria` is populated, and beside the hollow-denominator
    // check for the same reason — both are about a number the report already
    // publishes, said in a form a reader can open.
    report
        .diagnostics
        .extend(catch_all_documents(&report.criteria));
    for diagnostic in &mut report.diagnostics {
        diagnostic.guidance = Some(diagnostics::guidance_for(diagnostic));
    }
    for suspicion in &mut report.suspicions {
        suspicion.guidance = Some(suspicion.structured_guidance());
    }
    Ok(report)
}

/// Every headline number the coverage payload emits, enveloped (FR-063-AC-3).
///
/// The set is fixed rather than derived: a metric exists because somebody
/// renders it as a ratio, and a list that grew automatically with the struct
/// would fill up with fields nobody divides.
fn coverage_metrics(
    report: &CoverageReport,
    model: &TraceabilityModel,
    graph: &SymbolGraph,
    minting: declared_tables::MintingCensus,
) -> Vec<Metric> {
    // The premise under `backed/total`: source symbols that bound a trace id.
    // Summed across languages, because the ratio is bundle-wide and a per
    // language split already exists in `binding_census`.
    let bound_symbols: usize = report.binding_census.iter().map(|c| c.bound).sum();
    let tagged_symbols: usize = report.binding_census.iter().map(|c| c.tagged).sum();
    let walked_symbols: usize = report.binding_census.iter().map(|c| c.candidates).sum();

    let mut metrics = vec![Metric::measured(
        "coverage.backed",
        "matrix row",
        "rows of the declared reference tables whose trace target is bound by at \
         least one source symbol; `matched` counts the evidence symbols that bound \
         a trace id out of the `examined` evidence symbols walked, so matched 0 of \
         a non-zero examined is a ratio computed over a corpus the binder could not \
         read",
        report.totals.backed,
        report.totals.total,
        walked_symbols,
        bound_symbols,
    )];

    metrics.push(Metric::measured(
        "authoring.tag_rate",
        "evidence symbol",
        "evidence symbols whose attached annotation block carries an id-shaped token, \
         whether or not the declared trace grammar can read it; `value` counts tagged \
         symbols, while `matched` counts candidates classified as tagged or untagged so \
         an honest zero is not mistaken for an unread population",
        tagged_symbols,
        walked_symbols,
        walked_symbols,
        walked_symbols,
    ));

    // #367: a language-level binder can look healthy because comments or
    // attributes bind while the repository's dominant test-name convention
    // binds nothing. Retain that subpopulation as its own ratio; a zero match
    // over non-zero examined input is then caught by the existing generic
    // hollow-denominator invariant.
    for census in &report.binding_census {
        if census.self_named == 0 {
            continue;
        }
        metrics.push(Metric::measured(
            &format!("coverage.self_named_binding.{}", census.language),
            "self-named evidence symbol",
            "evidence symbols carrying a separator-delimited id in their own declaration name whose id was read by a declared name form; comment and attribute bindings do not satisfy this premise",
            census.self_named_bound,
            census.self_named,
            census.self_named,
            census.self_named_bound,
        ));
    }

    metrics.push(
        match (report.totals.property_shaped, report.totals.criteria) {
            (Some(shaped), Some(criteria)) => Metric::measured(
                "coverage.property_shaped",
                "acceptance criterion",
                "criteria the FR-052 classifier reads a property shape from; `matched` \
             counts criteria that reached the classifier at all",
                shaped,
                criteria,
                criteria,
                criteria,
            ),
            _ => Metric::not_computed(
                "coverage.property_shaped",
                "acceptance criterion",
                "criteria the FR-052 classifier reads a property shape from",
                MetricShape::Ratio,
                "no corpus document in scope binds an acceptance criterion, so nothing \
             was classified",
            ),
        },
    );

    // CR-095: the honest companion to `coverage.property_shaped`. Emitted as
    // its own metric rather than as a field on that one, because a reader who
    // sees only the first number must be able to find the second by name — the
    // whole failure being that 54% travels and 8% does not.
    metrics.push(
        match (report.totals.specific_shaped, report.totals.criteria) {
            (Some(specific), Some(criteria)) => Metric::measured(
                "coverage.specific_shaped",
                "acceptance criterion",
                "extractable criteria whose shape names what property to write — every \
             shape but the `universal` catch-all, `example` and `unclassified`; \
             `matched` counts criteria that reached the classifier. NOT a quality \
             ranking: a `universal` criterion is often the right thing to write",
                specific,
                criteria,
                criteria,
                criteria,
            ),
            _ => Metric::not_computed(
                "coverage.specific_shaped",
                "acceptance criterion",
                "extractable criteria whose shape names what property to write",
                MetricShape::Ratio,
                "no corpus document in scope binds an acceptance criterion, so nothing \
             was classified",
            ),
        },
    );

    // #226, folded: an absent list and an uncomputed one are the same bytes.
    // Here they are different states.
    metrics.push(if model.vocabularies.no_source_symbol.is_empty() {
        Metric::not_computed(
            "coverage.no_symbol_rows",
            "matrix row",
            "unbacked rows exempted by a verification method that mints no source \
             symbol",
            MetricShape::Count,
            "the module declares no `vocabularies.no_source_symbol`, so no row can \
             be exempt by its method and every unbacked row reads as a missing test",
        )
    } else {
        Metric::counted(
            "coverage.no_symbol_rows",
            "matrix row",
            "unbacked rows whose declared test type is in the module's \
             `no_source_symbol` vocabulary, counted over every row reconciled",
            report.no_symbol_rows.len(),
            report.totals.total,
            report.totals.total,
        )
    });

    metrics.push(if model.trace_tags.implements.is_empty() {
        Metric::not_computed(
            "coverage.implements",
            "production symbol",
            "production symbols carrying a declared `implements` marker",
            MetricShape::Count,
            "the module declares no `trace_tags.implements` forms, so the \
             requirement-to-code relation was never derived — this is unasked, not \
             none",
        )
    } else {
        Metric::counted(
            "coverage.implements",
            "production symbol",
            "production symbols carrying at least one declared `implements` marker, \
             counted over the production symbols examined; a zero means none is \
             annotated, not that none was read",
            graph.implements_bound,
            graph.implements_candidates,
            graph.implements_candidates,
        )
    });

    // FR-063-AC-7 (CR-117, #270): the premise under every minting number.
    //
    // `coverage.backed` answers "how many declared rows are backed by a test".
    // It cannot answer "were the rows found at all", and that is the question
    // 88 of 239 repositories fail: the archetype matches, the declared section
    // does not, and those TC ids never reach the denominator. A CANDIDATE
    // census counted 3,514 across 88 repositories, and that is not the causal
    // size of the section defect: CR-118 measured the fix at +83 rows, the
    // population being confounded with id-column mismatch (#318). A repository in
    // that state reports a *smaller* `total` and a plausible percentage, so
    // nothing in the payload distinguished it from a repository with fewer
    // tests.
    //
    // A RATIO, and the shape matters (CR-102). `matched` and `examined` are
    // the same pair of counts as `value` and `population` here, because the
    // measurement's input IS its population — one document offered to one
    // minting declaration. That makes `is_hollow` fire exactly when no
    // declaration found any of its sections, which is a ratio published over a
    // corpus none of whose minting tables were read. It stays quiet on the
    // honest zeroes: a bundle with no documents of any minting archetype
    // examines nothing, and one whose sections are all found matches
    // everything.
    metrics.push(if model.trace_targets.is_empty() {
        Metric::not_computed(
            "minting.section_hit_rate",
            "declared minting document",
            "documents whose declared minting section was found, over the documents \
             the declaration's archetype selected",
            MetricShape::Ratio,
            "the model declares no `trace_targets`, so no document was selected for \
             minting and there is no section to look for",
        )
    } else {
        Metric::measured(
            "minting.section_hit_rate",
            "declared minting document",
            "documents whose declared `section:` heading was found, over the documents \
             a required trace target's `archetype:` selected and its `exclude:` kept, \
             plus documents where an optional target's section was present; counted per \
             (trace target, document) pair. An optional absent section is outside the \
             population. A required miss reads the archetype right and the heading wrong, \
             so the whole table is stranded and its ids never enter `coverage.backed`'s \
             denominator at all",
            minting.section_found,
            minting.selected,
            minting.selected,
            minting.section_found,
        )
    });

    metrics
}
/// Return a stable scope-relative path using the declaration scanner's path
/// normalization (CR-038).
fn relative(root: &Path, path: &Path) -> String {
    declared_tables::relative_path(root, path)
}
