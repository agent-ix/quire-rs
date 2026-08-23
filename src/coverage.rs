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

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::corpus::declared_tables;
use crate::corpus::spec::Spec;
use crate::grammar::{AcPropertyCounts, GrammarVocabularies, GroundingCounts};
use crate::metric::{Metric, MetricShape};
use crate::obligation::Obligation;
use crate::registry::Registry;
use crate::symbols::trace::{BindingCensus, SymbolGraph};
use crate::traceability::{StatusClass, TraceabilityModel};

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
/// [`StatusClass::Unknown`] was computed and discarded from v0.1 until this
/// existed: the only consumer asked `== Complete`, `Unknown` compared false, and
/// the row fell out of every report. A value the contract admits and the model
/// does not class is therefore exempt from the status-lie check *by
/// construction* — not because it is honest, but because the engine had no
/// opinion about it and said nothing.
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
    /// The vocabulary or catalog value the diagnostic is about, verbatim, when
    /// it is about exactly one (FR-054-AC-12, CR-091).
    /// `uncatalogued-verification-method` carries the authored method here —
    /// the same string the obligation records carry in `method` — so a
    /// consumer joins the two by equality instead of regexing a human
    /// sentence, which was the only recovery path before (#179; quoin#168's
    /// mismatch/uncatalogued split is the filed reader).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

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

/// One minted trace id and where it came from.
struct MintedTarget {
    id: String,
    target: String,
    document: String,
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
    let mut report = reconcile(spec, model, graph, root);
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
    report.metrics = coverage_metrics(&report, model, graph);
    report
        .diagnostics
        .extend(hollow_denominators(&report.metrics));
    // #261: after `criteria` is populated, and beside the hollow-denominator
    // check for the same reason — both are about a number the report already
    // publishes, said in a form a reader can open.
    report
        .diagnostics
        .extend(catch_all_documents(&report.criteria));
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
) -> Vec<Metric> {
    // The premise under `backed/total`: source symbols that bound a trace id.
    // Summed across languages, because the ratio is bundle-wide and a per
    // language split already exists in `binding_census`.
    let bound_symbols: usize = report.binding_census.iter().map(|c| c.bound).sum();
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

    metrics
}

/// Documents whose every extractable criterion is the `universal` catch-all
/// (FR-050-AC-28, #261).
///
/// `coverage.specific_shaped` already reports the split — that was CR-095, and
/// it is what made "54% extractable" and "8% the tool told me what to write"
/// two visible numbers instead of one. But a ratio over a corpus names no
/// criterion, so a reader watching it fall has every spec file to search.
///
/// ONE diagnostic for the corpus, not one per document. Measured across five
/// repositories before choosing: 23/51, 41/85, 69/139, 6/9 and 8/28 documents
/// are all-`universal`, so per-document would add 147 findings and read as a
/// flood rather than a fact. The count carries the scale and the example
/// carries the locus, the same shape `no-symbol-bound` uses.
///
/// A corpus-level `specific_shaped == 0` was the other candidate and is
/// rejected on measurement: it fires on NONE of those five repositories, so it
/// would be a check that cannot fire — the very family this programme is
/// building a detector for.
///
/// **Data, not a verdict** (FR-050-CON-1). A `universal` criterion is often the
/// right thing to write, and CR-020 recorded that criteria validated by
/// demonstration legitimately score low. This says where the catch-alls are; it
/// does not say they are wrong.
fn catch_all_documents(criteria: &[CriteriaCounts]) -> Vec<CoverageDiagnostic> {
    let binding: Vec<&CriteriaCounts> = criteria.iter().filter(|c| c.property_shaped > 0).collect();
    let all_universal: Vec<&CriteriaCounts> = binding
        .iter()
        .copied()
        .filter(|c| c.specific_shaped == 0)
        .collect();
    let Some(first) = all_universal.first() else {
        return Vec::new();
    };
    // `criteria` is sorted by document, so `first` is deterministic (NFR-006).
    let at = match &first.catch_all_example {
        Some(example) => {
            let id = example.row_id.as_deref().unwrap_or("its first criterion");
            match example.line {
                Some(line) => format!(" — for example `{id}` at {}:{line}", first.document),
                None => format!(" — for example `{id}` in {}", first.document),
            }
        }
        None => String::new(),
    };
    vec![CoverageDiagnostic {
        declaration: "criteria".to_string(),
        reason: "catch-all-universal".to_string(),
        message: format!(
            "{} of {} documents binding extractable criteria named a specific property \
             shape for none of them{at}; the extractable headline counts what a generator \
             could quantify over, and this is the part it could not tell you what to write",
            all_universal.len(),
            binding.len()
        ),
        path: Some(first.document.clone()),
        value: Some("coverage.specific_shaped".to_string()),
    }]
}

/// A ratio computed over a population the measurement could not read
/// (FR-063-AC-5).
///
/// The generalization of `no-symbol-bound`: that diagnostic knows it is about
/// the trace binder and can name the declared forms to check, and it stays,
/// because a finding that says what to look at beats one that says a number is
/// wrong. This one fires for any metric, including ones added later that have
/// no bespoke check of their own — which is the point of a schema invariant.
fn hollow_denominators(metrics: &[Metric]) -> Vec<CoverageDiagnostic> {
    metrics
        .iter()
        .filter(|metric| metric.is_hollow())
        .filter_map(|metric| {
            // `is_hollow` is false for every `NotComputed`, so this destructure
            // always succeeds — `filter_map` states that rather than rendering
            // a half-sentence for a branch that cannot happen (CR-102).
            let crate::metric::Measurement::Measured {
                population,
                examined,
                ..
            } = metric.measurement
            else {
                return None;
            };
            Some(CoverageDiagnostic {
                declaration: "metrics".to_string(),
                reason: "hollow-denominator".to_string(),
                message: format!(
                    "`{}` published a ratio over {} {}{} but read none of the {} \
                     it walked, so the number is arithmetic over nothing; {}",
                    metric.name,
                    population,
                    metric.unit,
                    if population == 1 { "" } else { "s" },
                    examined,
                    metric.method
                ),
                path: None,
                value: Some(metric.name.clone()),
            })
        })
        .collect()
}

/// Declared methods that are in no catalog (FR-054-AC-11).
///
/// The catalog exists so a verification method is a **declared thing** rather
/// than a habit, and until this the two were indistinguishable: the derived
/// `verification_method` / `verification_class` vocabularies had no reader, so
/// a cell saying `CI Gate` sat in the report looking exactly like one saying
/// `Test`. Measured across the ecosystem at the time this landed, 55 of 577
/// obligations declared a method matching no catalog entry, and downstream
/// quoin's conformance check *skipped* each of them — so the requirements whose
/// verification is least well defined were the ones nothing questioned.
///
/// **Skipped entirely when no module declares a catalog.** An absent catalog
/// means the question cannot be asked, which is a different answer from "yes"
/// — and it keeps FR-050-AC-7 byte-identity for every module that has not
/// adopted one.
///
/// One diagnostic per distinct (source, method) pair, not per row: 14 rows
/// reading `Criterion Benchmark` are one decision to make, and 14 identical
/// diagnostics is how a report stops being read.
fn uncatalogued_methods(
    obligations: &[crate::obligation::Obligation],
    registry: &Registry,
) -> Vec<CoverageDiagnostic> {
    if registry.verification_catalog().is_none() {
        return Vec::new();
    }
    let methods = registry.column_vocabulary("verification_method");
    let classes = registry.column_vocabulary("verification_class");
    let known = |declared: &str| {
        let declared = declared.trim();
        methods.iter().any(|m| m.eq_ignore_ascii_case(declared))
            || classes.iter().any(|c| c.eq_ignore_ascii_case(declared))
    };

    // (source, method) -> (row count, first document). BTreeMap so the order is
    // a property of the data rather than of the walk (NFR-006).
    let mut unknown: BTreeMap<(&str, &str), (usize, &str)> = BTreeMap::new();
    for obligation in obligations {
        let Some(method) = obligation.method.as_deref() else {
            continue;
        };
        if known(method) {
            continue;
        }
        let entry = unknown
            .entry((&obligation.source, method))
            .or_insert((0, &obligation.document));
        entry.0 += 1;
    }

    unknown
        .into_iter()
        .map(|((source, method), (rows, document))| CoverageDiagnostic {
            declaration: source.to_string(),
            reason: "uncatalogued-verification-method".to_string(),
            message: format!(
                "'{method}' is neither a declared verification_catalog method id nor a \
                 declared class, so nothing can say what discharging it means ({rows} \
                 row(s), first in '{document}'). Add a catalog entry, or write a \
                 declared method in the cell"
            ),
            path: Some(document.to_string()),
            // Verbatim — the same string the obligation records carry in
            // `method` — so the join is equality, not prose parsing
            // (FR-054-AC-12).
            value: Some(method.to_string()),
        })
        .collect()
}

/// Per-document property-shape counts over the corpus (CR-028).
///
/// The rollup carries **no acceptance-criteria knowledge**: it walks the
/// already path-sorted corpus, resolves each document's frontmatter type to an
/// archetype, and asks the grammar layer what that archetype's binding
/// criteria classify as. A document yielding none — anything the `ac` binding
/// does not cover, and every document under a module declaring no grammar —
/// contributes no entry, so a corpus of non-requirement documents produces an
/// empty list.
///
/// The walk is declaration-free by necessity — criteria classification is not a
/// declared target — but it is not *scope*-free: the model-level `exclude:`
/// says which paths hold no corpus data at all, and this walk reads it (CR-060).
/// Before that, a deliberately malformed fixture under an excluded path
/// contributed to `criteria` and to both totals, inflating the denominator, and
/// its body was parsed despite the declaration saying it is not corpus data.
///
/// This is a pure function of statement text and the merged module
/// vocabularies: no network, no service, no execution (FR-050-CON-2).
fn criteria_counts(
    spec: &Spec,
    registry: &Registry,
    model: &TraceabilityModel,
    root: &Path,
) -> Vec<CriteriaCounts> {
    let vocab = GrammarVocabularies {
        lexicon: registry.lexicon_matcher(),
        observable: registry.observable_verbs_matcher(),
        vacuous: registry.vacuous_predicates_matcher(),
        idioms: registry.property_idioms_matcher(),
        ambiguous: registry.ambiguity_terms_matcher(),
    };
    let excluded = declared_tables::ExcludeSet::compile_validated(&model.exclude);

    let mut out: Vec<CriteriaCounts> = Vec::new();
    for entry in &spec.inner.documents {
        // Path-only, and ahead of every other gate: an excluded document must
        // not be classified *or* body-parsed (CR-060). It matches on the same
        // `relative_path` derivation a report path uses, so a glob and a
        // reported path compare as the same string (CR-038).
        if excluded.excludes(root, &entry.path) {
            continue;
        }
        let Some(archetype) =
            crate::corpus::spec::artifact_type(entry).and_then(|ty| registry.archetype(&ty))
        else {
            continue;
        };
        let Some(grammar_ref) = archetype.grammar_ref() else {
            continue;
        };
        // The line offset is NO LONGER immaterial (#261): the catch-all
        // example carries a document line, so the same conversion every other
        // located finding uses is needed here too. It was `0` while nothing
        // read a record's line.
        //
        // The body touch happens only past the archetype/grammar gates
        // above (CR-047): a document under a module declaring no grammar
        // stays unparsed.
        let records = crate::grammar::classify_document_properties(
            grammar_ref,
            &archetype.name,
            entry.body(),
            crate::validate_document::body_line_offset(&entry.body().raw),
            vocab,
        );
        if records.is_empty() {
            continue;
        }
        let counts = AcPropertyCounts::tally(records.iter());
        // The lowest-lined `universal` criterion, and only when the document
        // named a specific shape for none of them. A document with one
        // specific criterion is not what this finding is about, and carrying
        // an example there would make the field noise rather than a locus.
        let catch_all_example = (counts.property_shaped > 0 && counts.specific_shaped == 0)
            .then(|| {
                records
                    .iter()
                    .filter(|r| r.extractable)
                    .min_by_key(|r| (r.line.unwrap_or(usize::MAX), r.row_id.clone()))
                    .map(|r| CatchAllCriterion {
                        row_id: r.row_id.clone(),
                        line: r.line,
                    })
            })
            .flatten();
        out.push(CriteriaCounts {
            document: relative(root, &entry.path),
            archetype: archetype.name.clone(),
            criteria: counts.criteria,
            property_shaped: counts.property_shaped,
            specific_shaped: counts.specific_shaped,
            by_property: counts.by_property,
            grounding: counts.grounding,
            catch_all_example,
        });
    }
    out.sort_by(|a, b| (&a.document, &a.archetype).cmp(&(&b.document, &b.archetype)));
    out
}

fn reconcile(
    spec: &Spec,
    model: &TraceabilityModel,
    graph: &SymbolGraph,
    root: &Path,
) -> CoverageReport {
    let backed: BTreeSet<&str> = graph.backed_trace_ids();
    // CR-060: compiled once for the whole reconciliation — every declaration
    // is scoped by it.
    let model_exclude = declared_tables::ExcludeSet::compile_validated(&model.exclude);

    // ── Minted targets, grouped by their minting document ──
    let mut ctx = declared_tables::ScanContext::default();
    let mut minted: Vec<MintedTarget> = Vec::new();
    for target in &model.trace_targets {
        let exclude = declared_tables::ExcludeSet::compile_validated(&target.exclude);
        for row in declared_tables::scan(
            spec,
            root,
            declared_tables::DeclaredScope {
                name: &target.name,
                archetype: &target.archetype,
                exclude: &exclude,
                model_exclude: &model_exclude,
            },
            &target.section,
            &mut ctx,
        ) {
            let Some(id) = row.cell(&target.id_column) else {
                continue;
            };
            minted.push(MintedTarget {
                id: id.to_string(),
                target: target.name.clone(),
                document: relative(root, &row.path),
            });
        }
    }

    let mut counts: BTreeMap<(String, String), (usize, usize)> = BTreeMap::new();
    let mut declared_ids: BTreeSet<String> = BTreeSet::new();
    for entry in &minted {
        declared_ids.insert(entry.id.clone());
        let slot = counts
            .entry((entry.document.clone(), entry.target.clone()))
            .or_insert((0, 0));
        slot.1 += 1;
        if backed.contains(entry.id.as_str()) {
            slot.0 += 1;
        }
    }

    // ── Reference rows: unbacked rows and status lies ──
    let mut unbacked_rows: Vec<UnbackedRow> = Vec::new();
    let mut status_lies: Vec<StatusLie> = Vec::new();
    let mut no_symbol_rows: Vec<NoSymbolRow> = Vec::new();
    let mut undeclared_statuses: Vec<UndeclaredStatus> = Vec::new();
    let mut referenced_ids: BTreeSet<String> = BTreeSet::new();
    let mut row_ids: BTreeSet<String> = BTreeSet::new();
    // Row ids of rows that carry a status cell (FR-050-AC-23, CR-087). The
    // one-id-one-symbol policy is scoped to these: a status-classed row backed
    // by any one of N binders stays green while the other N−1 rot, which is
    // the defect. An id whose rows carry no status (an acceptance criterion
    // verified by several tests) is legitimately N:1 and never reported.
    let mut status_row_ids: BTreeSet<String> = BTreeSet::new();

    for declaration in &model.document_references {
        let Ok(pattern) = regex::Regex::new(&declaration.pattern) else {
            continue; // patterns are validated at module load
        };
        let exclude = declared_tables::ExcludeSet::compile_validated(&declaration.exclude);
        for row in declared_tables::scan(
            spec,
            root,
            declared_tables::DeclaredScope {
                name: &declaration.name,
                archetype: &declaration.archetype,
                exclude: &exclude,
                model_exclude: &model_exclude,
            },
            &declaration.section,
            &mut ctx,
        ) {
            let Some(raw_cell) = row.cell(&declaration.column) else {
                continue;
            };
            // CR-015: same normalization as FR-049, from the shared helper.
            let cell = &declared_tables::normalize_reference_cell(
                raw_cell,
                declaration.strip_annotations,
                declaration.expand_ranges,
            );
            let row_id = declaration
                .row_id_column
                .as_deref()
                .and_then(|c| row.cell(c))
                .map(str::to_string);
            if let Some(id) = &row_id {
                row_ids.insert(id.clone());
            }
            let mut ids: Vec<String> = pattern
                .captures_iter(cell)
                .filter_map(|c| c.get(1))
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            referenced_ids.extend(ids.iter().cloned());

            // A row is answerable for its own trace id *and* the ids it
            // references: a matrix row is backed when a test binds the row's
            // own id, an AC row when a test binds the TC it names.
            let mut answerable = ids.clone();
            if let Some(id) = &row_id {
                answerable.push(id.clone());
            }
            answerable.sort();
            answerable.dedup();
            ids.sort();
            ids.dedup();
            let is_backed = answerable.iter().any(|id| backed.contains(id.as_str()));
            let document = relative(root, &row.path);

            // CR-083: an undeclared status is classified **above** the backed
            // early-continue, and that placement is the whole point. Vocabulary
            // drift is a property of the declaration, not of the row's evidence:
            // a backstop that only ever sees unbacked rows is a by-product, not
            // a backstop. `class_of` has always returned `Unknown` here; until
            // now nothing asked.
            if let Some(status) = &model.status {
                if let Some(value) = row.cell(&status.column) {
                    if let Some(id) = &row_id {
                        status_row_ids.insert(id.clone());
                    }
                    if status.class_of(value) == StatusClass::Unknown {
                        undeclared_statuses.push(UndeclaredStatus {
                            reference: declaration.name.clone(),
                            document: document.clone(),
                            row_id: row_id.clone(),
                            status: value.to_string(),
                            line: Some(row.line),
                        });
                    }
                }
            }

            if is_backed {
                continue;
            }

            unbacked_rows.push(UnbackedRow {
                reference: declaration.name.clone(),
                document: document.clone(),
                row_id: row_id.clone(),
                target_ids: answerable.clone(),
                line: Some(row.line),
            });

            // CR-041: a row verified by a method that mints no source symbol
            // cannot carry a trace tag, so calling it a lie asserts something
            // its own declared method makes impossible. It stays in
            // `unbacked_rows` — that is a fact — and is explained here.
            // Two columns carry a verification method, and the exemption read
            // one (#259). A matrix `Type` cell saying `Inspection` was
            // exempted; the identical word in an FR Acceptance Criteria
            // `Verification` cell was not — one module vocabulary, consulted
            // for one of the two places it governs. Measured on
            // filament-ide-rs @ 6f87a7e: 21 exempt, 15 identical rows not.
            //
            // The second column is the DECLARATION'S OWN, not a new module
            // key: a reference declaration already names the column it reads,
            // and for `verification` that column IS the method. So every
            // existing module is fixed now rather than when each adopts a key.
            // For `traces-to` the declaration's column carries ids, no declared
            // method matches, and the `test_type_column` path still handles it.
            //
            // Parentheticals are stripped before comparing, because
            // `Inspection (TC-085)` names the method `Inspection` annotated
            // with its evidence — the same normalization `strip_annotations`
            // applies to ids, applied to the method. Eleven of the fifteen are
            // written that way, so an exact compare misses them even reading
            // the right column.
            let mints_no_symbol = |value: &str| {
                let bare = declared_tables::normalize_reference_cell(value, true, false);
                model.vocabularies.mints_no_symbol(bare.trim())
            };
            let exempting_type = model
                .vocabularies
                .test_type_column
                .as_deref()
                .and_then(|column| row.cell(column))
                .filter(|value| mints_no_symbol(value))
                .or_else(|| {
                    row.cell(&declaration.column)
                        .filter(|value| mints_no_symbol(value))
                });
            if let Some(test_type) = exempting_type {
                no_symbol_rows.push(NoSymbolRow {
                    reference: declaration.name.clone(),
                    document,
                    row_id,
                    test_type: test_type.to_string(),
                    target_ids: answerable,
                    line: Some(row.line),
                });
                continue;
            }

            // A status that classes `complete` over an unbacked row is a lie.
            if let Some(status) = &model.status {
                if let Some(value) = row.cell(&status.column) {
                    if status.class_of(value) == StatusClass::Complete {
                        status_lies.push(StatusLie {
                            reference: declaration.name.clone(),
                            document,
                            row_id,
                            status: value.to_string(),
                            target_ids: answerable,
                            line: Some(row.line),
                        });
                    }
                }
            }
        }
    }

    // ── Untracked symbols: a trace tag pointing at nothing declared ──
    let mut untracked_symbols: Vec<UntrackedSymbol> = graph
        .verifies
        .iter()
        .filter(|relation| {
            !declared_ids.contains(&relation.trace_id)
                && !referenced_ids.contains(&relation.trace_id)
                && !row_ids.contains(&relation.trace_id)
        })
        .map(|relation| UntrackedSymbol {
            path: relation.path.clone(),
            symbol: relation.symbol.clone(),
            trace_id: relation.trace_id.clone(),
            line: Some(relation.line),
        })
        .collect();

    let groups: Vec<GroupCounts> = counts
        .into_iter()
        .map(|((document, target), (backed, total))| GroupCounts {
            document,
            target,
            backed,
            total,
        })
        .collect();
    let totals = CoverageTotals {
        backed: groups.iter().map(|g| g.backed).sum(),
        total: groups.iter().map(|g| g.total).sum(),
        ..CoverageTotals::default()
    };

    unbacked_rows.sort_by(|a, b| {
        (&a.reference, &a.document, &a.row_id, &a.target_ids).cmp(&(
            &b.reference,
            &b.document,
            &b.row_id,
            &b.target_ids,
        ))
    });
    status_lies.sort_by(|a, b| {
        (&a.reference, &a.document, &a.row_id).cmp(&(&b.reference, &b.document, &b.row_id))
    });
    no_symbol_rows.sort_by(|a, b| {
        (&a.reference, &a.document, &a.row_id).cmp(&(&b.reference, &b.document, &b.row_id))
    });
    undeclared_statuses.sort_by(|a, b| {
        (&a.reference, &a.document, &a.row_id, &a.status).cmp(&(
            &b.reference,
            &b.document,
            &b.row_id,
            &b.status,
        ))
    });
    // Two identical matching rows in one document are one defect, not two
    // records — mirrors `untracked_symbols` below (#213). Compared without
    // `line` (#210): the duplicates sit on different lines by definition, and
    // letting the line distinguish them would quietly reopen CR-086. The
    // sort above is stable, so the surviving record is the first — lowest —
    // line.
    undeclared_statuses.dedup_by(|a, b| {
        a.reference == b.reference
            && a.document == b.document
            && a.row_id == b.row_id
            && a.status == b.status
    });
    untracked_symbols
        .sort_by(|a, b| (&a.path, &a.symbol, &a.trace_id).cmp(&(&b.path, &b.symbol, &b.trace_id)));
    untracked_symbols.dedup();

    // CR-054: declarations that selected nothing, rendered from the one
    // shared vocabulary `quire validate` also reports them under. Already
    // sorted by `into_diagnostics`, so the order is a property of the model.
    let mut diagnostics: Vec<CoverageDiagnostic> = ctx
        .into_diagnostics(totals.total > 0)
        .into_iter()
        .map(|(declaration, diagnostic)| {
            let (_, message) = declared_tables::scan_finding(&declaration, &diagnostic, root);
            CoverageDiagnostic {
                declaration,
                reason: declared_tables::scan_reason(&diagnostic).to_string(),
                message,
                // Since CR-062 the only scan diagnostic is declaration-level,
                // and a declaration-level fault has no document to point at.
                // The field stays on the report shape: a future diagnostic that
                // does name a document must not require a payload change.
                path: None,
                value: None,
            }
        })
        .collect();

    // A model declared without a single trace target mints nothing at all, so
    // every ratio it reports is over an empty denominator. `is_empty()` reads
    // it as *declared* (status or trace-tag entries alone are enough), which
    // is why `ModelUndeclared` never fires for it (CR-054).
    if model.trace_targets.is_empty() {
        diagnostics.insert(
            0,
            CoverageDiagnostic {
                declaration: "traceability".to_string(),
                reason: "model-mints-nothing".to_string(),
                message: "the declared traceability model has no trace_targets, so it \
                          mints no ids and every count is over an empty denominator"
                    .to_string(),
                path: None,
                value: None,
            },
        );
    }

    // CR-093: the binder could not read a language it was pointed at. Ordered
    // ahead of the obligation diagnostics and after the declaration-scan ones
    // because it invalidates more than either: a language that bound nothing
    // makes every unbacked row in it unreadable, not just the rows one
    // declaration selects.
    diagnostics.extend(binding_diagnostics(&graph.binding_census));

    // FR-053: derived here rather than in `compute` because obligations read
    // the same declared tables this reconciliation already walks, and need no
    // `Registry`.
    let (obligations, skipped) = crate::obligation::derive(spec, root, model);

    // FR-053-AC-8: a row whose statement cell is empty contributes no record,
    // and the AC says it is skipped **with a diagnostic**. It was not — the
    // second half of `derive`'s return was dropped here, so the only surface
    // that ever named a skipped row was a unit test calling `derive` directly
    // (CR-063). An authoring error nobody is told about is an authoring error
    // nobody fixes.
    diagnostics.extend(skipped.iter().map(|row| CoverageDiagnostic {
        declaration: row.source.clone(),
        reason: "obligation-row-states-nothing".to_string(),
        message: format!(
            "row {} of '{}' states no obligation: the source's statement column \
             is empty, so the row mints no record",
            row.row, row.document
        ),
        path: Some(row.document.clone()),
        value: None,
    }));

    // ── Shared trace ids: one id bound by several distinct symbols ──
    // (FR-050-AC-23, CR-087). Scoped to ids that are row ids of
    // status-carrying rows — the population where N binders let a green row
    // rot N−1 tests deep. Grouped through BTree collections so the record
    // order — and inside each record the symbol order — is a property of the
    // data, not of the walk (NFR-006). Distinctness is `(path, symbol)`: the
    // same symbol binding one id through two declared forms (or a duplicated
    // in-symbol id, FR-051's dedup) is one binder, not a shared id.
    let mut binders: BTreeMap<&str, BTreeSet<(&str, &str)>> = BTreeMap::new();
    for relation in &graph.verifies {
        if !status_row_ids.contains(&relation.trace_id) {
            continue;
        }
        binders
            .entry(relation.trace_id.as_str())
            .or_default()
            .insert((relation.path.as_str(), relation.symbol.as_str()));
    }
    let shared_trace_ids: Vec<SharedTraceId> = binders
        .into_iter()
        .filter(|(_, symbols)| symbols.len() > 1)
        .map(|(trace_id, symbols)| SharedTraceId {
            trace_id: trace_id.to_string(),
            symbols: symbols
                .into_iter()
                .map(|(path, symbol)| SharedTraceSymbol {
                    path: path.to_string(),
                    symbol: symbol.to_string(),
                })
                .collect(),
        })
        .collect();

    CoverageReport {
        unbacked_rows,
        status_lies,
        no_symbol_rows,
        undeclared_statuses,
        untracked_symbols,
        shared_trace_ids,
        groups,
        // CR-028: filled by `compute`, which holds the `Registry` this
        // reconciliation deliberately does not take.
        criteria: Vec::new(),
        // FR-059-AC-9 (CR-091): likewise filled by `compute` — the vocabulary
        // lives in an archetype's frontmatter schema, which only the
        // `Registry` can read.
        vocabulary_coverage: Vec::new(),
        diagnostics,
        obligations,
        // FR-062. Carried through to the JSON so a consumer can scope work by
        // requirement; deliberately NOT folded into `totals`, `backed` or
        // `untracked_symbols` — scope is not evidence.
        implements: graph
            .implements
            .iter()
            .map(|relation| ImplementsRecord {
                path: relation.path.clone(),
                symbol: relation.symbol.clone(),
                trace_id: relation.trace_id.clone(),
                form: relation.form.clone(),
            })
            .collect(),
        // FR-050-AC-24 (#215): what the declared `source_exclude` subtracted,
        // carried by the graph because this reconciliation never sees the
        // extraction itself.
        excluded_source_files: graph.excluded_source_files,
        // FR-050-AC-27 (CR-093): the premise, carried whether or not it holds.
        binding_census: graph.binding_census.clone(),
        // FR-063: filled by `compute`, which is where the criteria totals one
        // of these metrics describes are set.
        metrics: Vec::new(),
        // FR-064: carried by the graph, because the binder is where the
        // extraction and the symbol kinds are both in hand.
        suspicions: graph.suspicions.clone(),
        totals,
    }
}

/// The fraction of a language's candidates that must bind before the census
/// stops being a finding (FR-050-AC-27, CR-093).
///
/// Deliberately low. This is not a coverage target — an unbound candidate is
/// usually a real untagged test, and a repository mid-migration will sit well
/// under any number worth calling healthy. It is the floor below which the most
/// likely explanation stops being "tests are untagged" and starts being "the
/// binder cannot read this repository's convention", which is a different
/// finding with a different fix.
const BINDING_FLOOR: f64 = 0.05;

/// Diagnostics for a language the binder examined and could not read
/// (FR-050-AC-27, CR-093).
///
/// Two reasons, and the split is the actionable part: `no-symbol-bound` is
/// unambiguous — every candidate was walked and every declared pattern missed —
/// while `low-symbol-binding` reports both counts and lets the reader judge,
/// because at 3% the tail of untagged tests and a near-miss pattern look alike
/// from here.
fn binding_diagnostics(census: &[BindingCensus]) -> Vec<CoverageDiagnostic> {
    census
        .iter()
        .filter(|entry| entry.candidates > 0)
        .filter_map(|entry| {
            let forms = if entry.forms.is_empty() {
                "none declared".to_string()
            } else {
                entry.forms.join(", ")
            };
            // One unbound symbol, named. A census is a count and a count
            // cannot be opened: this diagnostic named the LANGUAGE and nothing
            // else, so a reader holding 1,292 unbound Rust symbols was told to
            // search Rust (#256). `at` is the sentence a reader can act on;
            // `path` is the same fact as a field, so `path:line` output and the
            // benchmark's positional scoring both work.
            let at = entry.unbound_example.as_ref().map(|e| {
                format!(
                    " — for example `{}` at {}:{}, whose annotation carries no \
                     matching form",
                    e.symbol, e.path, e.line
                )
            });
            let at = at.unwrap_or_default();
            let (reason, message) = if entry.bound == 0 {
                (
                    "no-symbol-bound",
                    format!(
                        "{} {} evidence symbols were examined and none carried a tag any \
                         declared form matched (forms: {forms}){at}; every row those symbols \
                         verify is reported unbacked, which is indistinguishable from a \
                         missing test",
                        entry.candidates, entry.language
                    ),
                )
            } else if (entry.bound as f64) < (entry.candidates as f64) * BINDING_FLOOR {
                (
                    "low-symbol-binding",
                    format!(
                        "{} of {} {} evidence symbols bound a trace id (forms: {forms}){at}; \
                         below {}% the likeliest reading is a marker-form mismatch rather \
                         than untagged tests",
                        entry.bound,
                        entry.candidates,
                        entry.language,
                        (BINDING_FLOOR * 100.0) as usize
                    ),
                )
            } else {
                return None;
            };
            Some(CoverageDiagnostic {
                declaration: "traceability.trace_tags".to_string(),
                reason: reason.to_string(),
                message,
                path: entry.unbound_example.as_ref().map(|e| e.path.clone()),
                value: Some(entry.language.clone()),
            })
        })
        .collect()
}

/// A path relative to the scope root, `/`-separated so reports are stable
/// across platforms. Shared with the scan layer so a report path and an
/// `exclude` glob are matched against the same string (CR-038).
fn relative(root: &Path, path: &Path) -> String {
    declared_tables::relative_path(root, path)
}
