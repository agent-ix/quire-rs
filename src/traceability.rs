//! The declarative traceability model (FR-050).
//!
//! Coverage is **not** an engine concept: quire knows nothing of "FR", "AC", or
//! "TC". A Filament module declares, in its `manifest.yaml` under
//! `traceability:`, which documents mint trace ids, which columns reference
//! them, what the status vocabulary means, and how source symbols carry trace
//! tags. Everything downstream — reference integrity (FR-049), symbol trace
//! binding (FR-051), and the coverage rollup (FR-050) — reads this model, so a
//! module with an entirely different vocabulary gets coverage with no engine
//! change.
//!
//! The model is the same spec-semantics-as-module-data pattern as
//! `body_extraction`, `lint_rules`, and `grammar_ref`: an absent section leaves
//! the model **undeclared** (consumers say so rather than inventing defaults),
//! and a malformed section fails module load like any other manifest shape
//! error.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// A source language an adapter and a trace-marker form can bind to (FR-051).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceLanguage {
    Rust,
    Python,
    #[serde(alias = "ts")]
    Typescript,
}

impl SourceLanguage {
    /// Stable machine-readable label, used in symbol identities (FR-051).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Typescript => "typescript",
        }
    }
}

/// The whole declared model. Every field is optional at the YAML layer so a
/// module can declare only the parts it needs (e.g. reference integrity
/// without a status vocabulary), but what it does declare must be coherent —
/// see [`TraceabilityModel::validate`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TraceabilityModel {
    /// Path globs holding **no traceable data** (CR-060): test fixtures,
    /// deliberately malformed samples, vendored examples. A document under one
    /// mints no trace ids, contributes no reference rows, and is not classified
    /// for criteria — so every declaration is scoped by it, and so is the
    /// CR-028 criteria walk, which has no declaration of its own to hang an
    /// exclusion on.
    ///
    /// Which paths hold test data is a property of the repository, not of one
    /// declaration — a `TraceTarget::exclude` says "these documents mint no
    /// ids for *me*", which is a different, still-supported statement.
    ///
    /// It scopes **traceability only**. An excluded document is still a
    /// document: `validate_bundle` schema- and grammar-checks it like any
    /// other, because a fixture being outside the coverage rollup is not a
    /// licence for it to be malformed in ways nobody reports.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Path globs under the **code** root holding no traceable source (CR-085):
    /// fixture trees whose whole purpose is to contain a tag nothing declares.
    ///
    /// Deliberately a separate key from [`exclude`], which is about **documents**
    /// and has never been applied to a source file. The two vocabularies must
    /// not merge, and the reason is measurable: FR-004-AC-9 in
    /// `spec-artifacts-process` *requires* every trace target to exclude
    /// `tests/**`, while 194 of this crate's ~458 `#[trace(` markers live under
    /// `tests/` — and in every Python and TypeScript repository in the ecosystem
    /// that share is near total. A key that meant both would delete the evidence
    /// tree and read as a catastrophic coverage regression.
    ///
    /// **`tests/**` must never appear here.** Anchor at the fixture directory —
    /// `tests/fixtures/**`, not `tests/**`. `globset` anchors a pattern at the
    /// start unless it opens with `**/`, so `tests/fixtures/**` cannot match
    /// `src/tests/fixtures/x.rs`.
    ///
    /// This **subtracts within** the code root and can do nothing else. It does
    /// not relocate either root, so FR-050 CR-045 ("`spec/` is convention, not
    /// configuration: no manifest key and no flag relocates it") is untouched —
    /// the document root's exclusion stays the caller's non-configurable
    /// argument, and these globs are a second filter applied after it. TC-944
    /// asserts that a `source_exclude` of `spec/**` cannot un-exclude `spec/`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_exclude: Vec<String>,
    /// Documents/sections that mint trace ids.
    #[serde(default)]
    pub trace_targets: Vec<TraceTarget>,
    /// Columns whose cells reference minted trace ids.
    #[serde(default)]
    pub document_references: Vec<DocumentReference>,
    /// The status column and how its values class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<StatusVocabulary>,
    /// How source symbols carry trace ids (FR-051).
    #[serde(default)]
    pub trace_tags: TraceTagGrammar,
    /// Column vocabularies the module's matrix contract and this rollup must
    /// agree on (CR-015). Declaring them here makes the model the single
    /// source: a contract and its coverage computation cannot drift.
    #[serde(default)]
    pub vocabularies: ColumnVocabularies,
    /// Which rows state **obligations** — the quire↔quoin contract (FR-053).
    /// Empty means the module declares none, and every obligation surface is
    /// then empty rather than absent.
    #[serde(default)]
    pub obligations: Vec<ObligationSource>,
    /// Edges every document of a kind must have (FR-058). Empty means the
    /// module declares none and the check is a no-op — the same shape every
    /// other declaration in this model uses.
    #[serde(default)]
    pub required_relations: Vec<RequiredRelation>,
    /// Edge verbs that must not form a cycle (FR-058). `refines` and `derives`
    /// are the motivating pair: a requirement that transitively refines itself
    /// states nothing, and no per-document check can see it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acyclic_edges: Vec<String>,
    /// Declared vocabularies whose values documents are expected to claim
    /// (FR-059). Empty means the module declares none and the check is a
    /// no-op, the same shape every other declaration in this model uses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vocabulary_coverage: Vec<VocabularyCoverage>,
}

/// One vocabulary whose values documents are expected to claim (FR-059).
///
/// The generic question behind "does this corpus have a reliability
/// requirement at all": *given a declared vocabulary and a declared projection
/// from documents onto it, which values does no document claim?* ISO 25010
/// quality characteristics are one instance; test-type coverage over a matrix
/// and STRIDE-category coverage over declared threats are others.
///
/// **The vocabulary is read, never authored here.** `field` names a frontmatter
/// field on `from`'s archetype whose schema already declares an `enum`, and
/// that enum *is* the vocabulary. Restating the values in the manifest would
/// mint a second list free to drift from the first — which is exactly the
/// defect CR-015 closed, and the reason `agent-ix/quire-rs#162` was filed
/// against a scope that proposed walking a hardcoded list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VocabularyCoverage {
    /// Declaration name, reported in the finding.
    pub name: String,
    /// Archetype whose documents claim values, and whose frontmatter schema
    /// declares the vocabulary.
    pub from: String,
    /// Frontmatter field carrying the claim. Its `enum` in `from`'s
    /// frontmatter schema is the vocabulary.
    pub field: String,
    /// The `<check>` half of the `trace:<check>` severity key (FR-057).
    pub check: String,
    /// Frontmatter field on **any** document in the bundle listing values
    /// deliberately not applicable here. A value named there is *covered*, not
    /// unowned: "this product has no safety characteristic, and here is where
    /// we said so" is an answer, and a check that cannot accept one forces
    /// either a false finding or a fabricated requirement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub justified_absence_field: Option<String>,
    /// Scope-relative globs whose matching documents neither claim nor justify.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

/// An edge every document of one kind must have (FR-058) — the declaration
/// behind upward-trace completeness.
///
/// Nothing here is engine knowledge. The engine knows the *shape* — "documents
/// of kind K must have an edge of one of these verbs, in this direction, to a
/// document of one of these kinds" — and never that an FR traces to a StR. That
/// is the same split [`DocumentReference`] draws for table cells, and it is what
/// lets a security module state "every hazard must be mitigated by something"
/// as manifest data instead of a second engine check.
///
/// **Direction is the whole point.** Upward tracing is the only analysis class
/// that finds *missing* requirements: an FR with no upstream need is a feature
/// nobody asked for, and a StR nothing implements is a need nobody built. They
/// are the same declaration read in opposite directions, so they are one type
/// with a [`direction`](Self::direction) rather than two checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredRelation {
    /// Declaration name, used in the finding and in diagnostics.
    pub name: String,
    /// Archetype whose documents carry the obligation to have this edge.
    pub from: String,
    /// Accepted edge verbs. Any one of them satisfies the relation — a module
    /// that accepts `implements` or `refines` says so here rather than
    /// declaring the relation twice.
    pub edges: Vec<String>,
    /// Accepted archetypes at the other end. Empty means "any document in the
    /// bundle", which is the honest reading of a module that constrains the
    /// verb but not the target.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    /// Which way the edge must point from the `from` document.
    pub direction: RelationDirection,
    /// The `<check>` half of the `trace:<check>` severity key (FR-057), so a
    /// module tunes each declared relation independently. Kebab-case, matching
    /// the registry's key pattern.
    pub check: String,
    /// Scope-relative globs whose matching documents are exempt from this
    /// relation — the same opt-out [`TraceTarget`] has (CR-038).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

/// Which way a [`RequiredRelation`]'s edge must point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationDirection {
    /// The `from` document must be the **source**: an FR with no `implements`
    /// edge to any StR is an orphan requirement.
    Outgoing,
    /// The `from` document must be the **target**: a StR with no incoming
    /// `implements` edge is a stated need nothing builds.
    Incoming,
    /// Either end satisfies it.
    ///
    /// Needed because a link between the same two documents is authored from
    /// whichever end the author was writing. A stakeholder requirement is
    /// "implemented" both when an FR declares `implements` **pointing at it**
    /// and when the requirement itself declares `satisfied_by` **pointing at**
    /// the FR — 956 `implements` edges and 328 `satisfied_by` edges exist side
    /// by side in this ecosystem. Declaring two one-way relations would not
    /// express it: two relations are two independent obligations, and a
    /// document satisfying one would still be reported by the other.
    Either,
}

/// One kind of row that states an obligation (FR-053).
///
/// The engine knows the *shape* — id, statement, method, parameters,
/// criticality — and never a column name, a method name or an archetype. A
/// module says which of its rows are obligations and which cell carries what.
///
/// Minting documents resolve in exactly one of two ways, and declaring both or
/// neither is a load-time error:
///
/// * [`target`](Self::target) names a declared [`TraceTarget`], reusing its
///   archetype, section and id column. An acceptance criterion is then not
///   declared twice, and the obligation id is by construction the id the
///   rollup and every trace tag already key on.
/// * [`archetype`](Self::archetype) + [`section`](Self::section) +
///   [`id_format`](Self::id_format) covers a table whose rows mint no id of
///   their own — the NFR `Measurement and Evaluation` table, where every row is
///   a quantified obligation and none has an `ID` column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationSource {
    /// Source name, used in report entries and diagnostics.
    pub name: String,
    /// Names a declared [`TraceTarget`] to inherit archetype/section/id from.
    /// Mutually exclusive with `archetype`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Archetype whose documents carry these rows. Mutually exclusive with
    /// `target`; requires `section` and `id_format`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archetype: Option<String>,
    /// Heading of the section carrying the table. Required with `archetype`,
    /// ignored with `target` (which supplies its own).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Template rendering an id for a row that has none: `{document}` is the
    /// owning document's id, `{row}` the 1-based row ordinal. Required with
    /// `archetype`, ignored with `target`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_format: Option<String>,
    /// Scope-relative globs whose matching documents state no obligations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Column carrying the normative statement — the text that is hashed.
    pub statement_column: String,
    /// Column carrying the declared verification method. A trailing
    /// parenthetical annotation is dropped, so a `Verification` cell reading
    /// `Test (TC-707)` yields `Test` while FR-049 still reads `TC-707` from
    /// the same cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_column: Option<String>,
    /// Column carrying criticality. Genuinely optional: the ISO acceptance
    /// criteria contract is `ID | Criteria | Verification` and carries no
    /// priority column, so declaring one is a module's choice rather than a
    /// precondition for obligations to exist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criticality_column: Option<String>,
    /// Result key → source column. Carries a metric's target and threshold,
    /// a t-way strength, a mutation-score floor — whatever the method needs,
    /// travelling with the obligation instead of being re-parsed downstream.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
    /// Read this source's table as a **configuration space** and mint ONE
    /// obligation for the whole table rather than one per row (FR-061).
    ///
    /// A new source *kind*, not a new mechanism: the statement hash, the
    /// suspect link and the `parameters` carriage are the ones FR-053 already
    /// defines. What differs is arity — a t-way obligation is a statement about
    /// the interaction of every row, so one row cannot carry it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combinatorial: Option<CombinatorialColumns>,
}

/// Which columns of a configuration-dimensions table mean what (FR-061).
///
/// The engine knows no dimension, no value and no strength — a module says
/// which column holds each, exactly as `statement_column` and `method_column`
/// do for criteria rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombinatorialColumns {
    /// Column naming each configuration dimension.
    pub dimension_column: String,
    /// Column listing that dimension's values, comma-separated.
    pub values_column: String,
    /// Column listing forbidden combinations (`dim=value & dim=value`).
    ///
    /// Optional because a space may genuinely have none — but a module that
    /// omits it on a space that does have them mints obligations over
    /// combinations that cannot exist, and the target becomes permanently
    /// unreachable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excludes_column: Option<String>,
    /// Interaction strength. 2 is the usual default; 3 is for spaces whose
    /// failures are known to need three-way interaction.
    pub strength: usize,
}

/// Declared vocabularies for matrix columns (CR-015).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ColumnVocabularies {
    /// Values the test-type column admits. The module declares the whole list;
    /// a consumer that wants the ecosystem core plus local additions declares
    /// both, exactly as `observable_verbs` layers over engine defaults.
    #[serde(default)]
    pub test_type: Vec<String>,
    /// Column carrying the test type on a reference row (CR-041). Needed only
    /// when `no_source_symbol` is declared — that is the one rule that reads a
    /// row's type rather than its status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_type_column: Option<String>,
    /// Test-type values whose verification method mints **no source symbol**
    /// (CR-041): an agent-behaviour eval, an inspection, a demonstration. A row
    /// verified this way can never carry a trace tag, so reporting it as a
    /// status lie asserts something its own declared method makes impossible.
    ///
    /// Module-declared, like every other vocabulary here: the engine has no
    /// opinion about which methods produce code.
    #[serde(default)]
    pub no_source_symbol: Vec<String>,
}

impl ColumnVocabularies {
    /// True when `value` names a declared method that mints no source symbol.
    /// Compared case-insensitively, as the column vocabularies are elsewhere.
    pub fn mints_no_symbol(&self, value: &str) -> bool {
        let v = value.trim();
        self.no_source_symbol
            .iter()
            .any(|declared| declared.trim().eq_ignore_ascii_case(v))
    }
}

/// One kind of trace id and the archetype whose documents mint it.
///
/// Binding is by archetype only (CR-062). The `document:` form — a
/// scope-relative path harvested off-corpus — existed because
/// `corpus/walk.rs` skipped `tests.md`, so archetype binding could not see the
/// file most repositories call their Test Matrix. Type-driven corpus membership
/// removed that premise, and one way to acquire a minting document is enough:
/// path binding enumerated filenames (`spec/tests.md`, `spec/matrix.md`,
/// `spec/evals.md`, one declaration each) and reached nothing nested, so a
/// matrix at `spec/<module>/matrix/tests.md` minted no ids however correctly it
/// was authored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceTarget {
    /// Kind name, referenced by [`DocumentReference::targets`].
    pub name: String,
    /// Archetype whose documents mint these ids.
    pub archetype: String,
    /// Scope-relative globs whose matching documents mint nothing (CR-038).
    /// The engine has no idea which paths hold test data; a module says so.
    ///
    /// **More** load-bearing since CR-062, not less: archetype binding is what
    /// lets a fixture matrix mint phantom ids, and a fixture that exercises the
    /// `TestMatrix` contract legitimately *is* `type: TestMatrix`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Heading of the section carrying the minting table.
    pub section: String,
    /// Table column holding the minted id.
    pub id_column: String,
}

/// One kind of reference from a document cell to minted trace ids. `pattern`
/// is a regex whose **first capture group** is the referenced id, so a cell
/// like `Test (TC-707)` and a bare `TC-707` are both module-declared shapes
/// rather than engine knowledge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentReference {
    /// Reference name, used in report entries and diagnostics.
    pub name: String,
    /// Archetype whose documents carry these references (CR-062: archetype
    /// only, for the reasons on [`TraceTarget`]).
    pub archetype: String,
    /// Scope-relative globs whose matching documents contribute no reference
    /// rows (CR-038).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Heading of the section carrying the referencing table.
    pub section: String,
    /// Column whose cells carry the references.
    pub column: String,
    /// Column identifying the referencing row (the row id in report entries).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_id_column: Option<String>,
    /// Regex over the cell; capture group 1 is the referenced trace id.
    pub pattern: String,
    /// Names of the [`TraceTarget`]s these references resolve against.
    pub targets: Vec<String>,
    /// Expand a same-prefix range (`FR-001..FR-006`) into its concrete ids
    /// before extraction (CR-015). Off unless declared — the engine gains no
    /// behaviour a module has not asked for.
    #[serde(default)]
    pub expand_ranges: bool,
    /// Drop parenthetical spans before extraction, so a qualifier
    /// (`FR-022-AC-5 (superseded by FR-030)`) contributes one reference rather
    /// than two (CR-015). Off unless declared.
    #[serde(default)]
    pub strip_annotations: bool,
}

/// The status column and the values that class as complete / pending / failed.
/// The engine never assumes `✅` means anything — the module says so.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusVocabulary {
    /// Column holding the status value.
    pub column: String,
    #[serde(default)]
    pub complete: Vec<String>,
    #[serde(default)]
    pub pending: Vec<String>,
    #[serde(default)]
    pub failed: Vec<String>,
    /// Values marking a row as withdrawn rather than outstanding (CR-015).
    #[serde(default)]
    pub retired: Vec<String>,
}

/// How a status value classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Complete,
    Pending,
    Failed,
    /// Withdrawn rather than outstanding (CR-015).
    Retired,
    /// Declared nowhere in the vocabulary.
    Unknown,
}

impl StatusVocabulary {
    /// Class `value` per the declared vocabulary.
    ///
    /// Matching is on the **leading marker** (CR-015): the ecosystem authors
    /// `✅ Complete` and `⚠️ scale evidence deferred`, where the marker carries
    /// the class and the remainder carries why. An exact match wins first, so a
    /// vocabulary of whole words (`done`, `planned`) behaves as before.
    pub fn class_of(&self, value: &str) -> StatusClass {
        let v = value.trim();
        let exact = |set: &[String]| set.iter().any(|s| s.trim() == v);
        let leading = |set: &[String]| {
            set.iter().any(|s| {
                let s = s.trim();
                !s.is_empty()
                    && v.starts_with(s)
                    // The marker ends where the note begins: any non-alphanumeric
                    // boundary counts (`✅ Complete`, `done, verified`), so
                    // `doneish` is still not `done`.
                    && v[s.len()..]
                        .chars()
                        .next()
                        .map_or(true, |c| !c.is_alphanumeric())
            })
        };
        for (set, class) in [
            (&self.complete, StatusClass::Complete),
            (&self.pending, StatusClass::Pending),
            (&self.failed, StatusClass::Failed),
            (&self.retired, StatusClass::Retired),
        ] {
            if exact(set) {
                return class;
            }
        }
        for (set, class) in [
            (&self.complete, StatusClass::Complete),
            (&self.pending, StatusClass::Pending),
            (&self.failed, StatusClass::Failed),
            (&self.retired, StatusClass::Retired),
        ] {
            if leading(set) {
                return class;
            }
        }
        StatusClass::Unknown
    }
}

/// The trace-tag grammar (FR-051): framework-native **markers** are the
/// canonical form; the textual **legacy** forms are read only during migration
/// and their bindings carry `legacy` provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TraceTagGrammar {
    #[serde(default)]
    pub markers: Vec<TraceMarkerForm>,
    #[serde(default)]
    pub legacy: Vec<TraceLegacyForm>,
    /// Marker forms binding a **production** symbol to the requirement it
    /// implements (FR-062).
    ///
    /// A separate list, not a flag on `markers`, and that is the whole design.
    /// `verifies` means *"this test would fail if the behaviour broke"* —
    /// evidence, and it can back an acceptance criterion. `implements` means
    /// *"this code is what the requirement is about"* — scope, and it must
    /// never back one.
    ///
    /// CR-061 stopped `verifies` binding production symbols precisely because a
    /// doc comment citing `FR-053-AC-1` would otherwise count as evidence for
    /// it. Widening `verifies` was the wrong fix; so is a shared list with a
    /// discriminator, because the two would then differ by a field a typo can
    /// flip. They differ by which list declares them and which symbol kinds
    /// they may attach to.
    #[serde(default)]
    pub implements: Vec<TraceMarkerForm>,
}

/// One canonical marker form — a statically parseable construct attached to a
/// test symbol (a pytest marker, a Rust attribute, a TS helper call). `pattern`
/// matches the construct; capture group 1 is the id-bearing argument list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceMarkerForm {
    pub name: String,
    pub language: SourceLanguage,
    pub pattern: String,
    /// Authoring template for the marker, e.g. `#[trace("{ids}")]`. Present
    /// only when the module wants mechanical rewrite suggestions from legacy
    /// forms: FR-051 emits a suggestion "where the equivalent marker is
    /// derivable", and this template is what makes it derivable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

/// One legacy textual form (bare id in a docstring, a `Trace:` line, a
/// line-comment id, a trace-embedding test name). `rewrite_to` names the
/// marker form a mechanical rewrite suggestion should target, when derivable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceLegacyForm {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<SourceLanguage>,
    pub pattern: String,
    /// Template rebuilding the trace id from the captures, e.g. `TC-{1}` for a
    /// test name like `tc741_extracts`. Absent means capture group 1 already
    /// *is* the id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewrite_to: Option<String>,
}

impl TraceabilityModel {
    /// True when the module declared nothing at all — consumers report the
    /// model as undeclared rather than computing an empty rollup
    /// (FR-050-AC-2/AC-9).
    pub fn is_empty(&self) -> bool {
        self.trace_targets.is_empty()
            && self.document_references.is_empty()
            && self.status.is_none()
            && self.trace_tags.markers.is_empty()
            && self.trace_tags.legacy.is_empty()
            // FR-062 (CR-081): the third marker list, listed for the same
            // reason as the two above. Both hand-maintained per-field
            // functions — this one and `merge_traceability` — missed it when
            // the field was added, which is the failure TC-905 was written to
            // catch and did not, because its assertions are hand-listed too.
            && self.trace_tags.implements.is_empty()
            && self.vocabularies.test_type.is_empty()
            // FR-053: obligations mint records, so a model declaring only them
            // has declared something. (Unlike the model-level `exclude`, which
            // states what is *not* corpus data and reconciles nothing — CR-060.)
            && self.obligations.is_empty()
            // FR-058: same reasoning. A module whose whole model is "every FR
            // must trace to a StR" has declared something, and omitting these
            // dropped that model on the floor — `traceability()` returned
            // `None` and the check never ran. Every field that makes the model
            // *do* something has to be listed here; the list is the definition
            // of "declared nothing", not a summary of it.
            && self.required_relations.is_empty()
            && self.acyclic_edges.is_empty()
            && self.vocabulary_coverage.is_empty()
        // CR-085: `source_exclude` is deliberately **absent** from this list,
        // mirroring the model-level `exclude` two fields above it. Both state
        // what is *not* traceable data; neither reconciles anything, so a module
        // whose whole model is "these paths are not source" has declared
        // nothing. Written down because the surrounding comment says every field
        // that makes the model *do* something must be listed, and the next
        // reader will otherwise correct the omission.
    }

    /// Look up a declared target by name.
    pub fn target(&self, name: &str) -> Option<&TraceTarget> {
        self.trace_targets.iter().find(|t| t.name == name)
    }

    /// Shape-validate the declaration. Returns a caller-facing reason on the
    /// first problem, which the manifest loader turns into a module-load
    /// failure (FR-050-AC-2).
    pub fn validate(&self) -> Result<(), String> {
        check_excludes("model-level `exclude`", &self.exclude)?;
        check_excludes("model-level `source_exclude`", &self.source_exclude)?;

        let mut target_names: BTreeSet<&str> = BTreeSet::new();
        for target in &self.trace_targets {
            check_named("trace_targets", &target.name)?;
            if !target_names.insert(target.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate trace_targets entry '{}'",
                    target.name
                ));
            }
            check_field(
                "trace_targets",
                &target.name,
                "archetype",
                &target.archetype,
            )?;
            check_excludes(
                &format!("trace_targets entry '{}'", target.name),
                &target.exclude,
            )?;
            check_field("trace_targets", &target.name, "section", &target.section)?;
            check_field(
                "trace_targets",
                &target.name,
                "id_column",
                &target.id_column,
            )?;
        }

        let mut reference_names: BTreeSet<&str> = BTreeSet::new();
        for reference in &self.document_references {
            check_named("document_references", &reference.name)?;
            if !reference_names.insert(reference.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate document_references entry '{}'",
                    reference.name
                ));
            }
            check_field(
                "document_references",
                &reference.name,
                "archetype",
                &reference.archetype,
            )?;
            check_excludes(
                &format!("document_references entry '{}'", reference.name),
                &reference.exclude,
            )?;
            check_field(
                "document_references",
                &reference.name,
                "section",
                &reference.section,
            )?;
            check_field(
                "document_references",
                &reference.name,
                "column",
                &reference.column,
            )?;
            check_capturing_pattern("document_references", &reference.name, &reference.pattern)?;
            if reference.targets.is_empty() {
                return Err(format!(
                    "traceability: document_references entry '{}' declares no targets",
                    reference.name
                ));
            }
            for target in &reference.targets {
                if !target_names.contains(target.as_str()) {
                    return Err(format!(
                        "traceability: document_references entry '{}' references undeclared \
                         target '{target}'",
                        reference.name
                    ));
                }
            }
        }

        // FR-053: an obligation source resolves its minting documents exactly
        // one way. Both or neither is a declaration that cannot be executed,
        // and failing at load is the only place it can be reported against the
        // declaration rather than against a mysteriously empty report.
        let mut obligation_names: BTreeSet<&str> = BTreeSet::new();
        for source in &self.obligations {
            check_named("obligations", &source.name)?;
            if !obligation_names.insert(source.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate obligations entry '{}'",
                    source.name
                ));
            }
            match (&source.target, &source.archetype) {
                (Some(_), Some(_)) => {
                    return Err(format!(
                        "traceability: obligations entry '{}' declares both 'target' and \
                         'archetype'; a source inherits from a trace target or names its own \
                         archetype, never both",
                        source.name
                    ))
                }
                (None, None) => {
                    return Err(format!(
                        "traceability: obligations entry '{}' declares neither 'target' nor \
                         'archetype', so nothing says which documents state it",
                        source.name
                    ))
                }
                (Some(target), None) => {
                    if !target_names.contains(target.as_str()) {
                        return Err(format!(
                            "traceability: obligations entry '{}' references undeclared target \
                             '{target}'",
                            source.name
                        ));
                    }
                }
                (None, Some(archetype)) => {
                    check_field("obligations", &source.name, "archetype", archetype)?;
                    let Some(section) = &source.section else {
                        return Err(format!(
                            "traceability: obligations entry '{}' declares 'archetype' without \
                             'section'",
                            source.name
                        ));
                    };
                    check_field("obligations", &source.name, "section", section)?;
                    let Some(id_format) = &source.id_format else {
                        return Err(format!(
                            "traceability: obligations entry '{}' declares 'archetype' without \
                             'id_format'; rows under an archetype-bound section mint no id of \
                             their own",
                            source.name
                        ));
                    };
                    // A template naming neither placeholder renders one id for
                    // every row, so every obligation after the first would
                    // silently overwrite the one before.
                    if !id_format.contains("{document}") && !id_format.contains("{row}") {
                        return Err(format!(
                            "traceability: obligations entry '{}' has an id_format naming \
                             neither {{document}} nor {{row}}, so every row would render the \
                             same id",
                            source.name
                        ));
                    }
                }
            }
            check_excludes(
                &format!("obligations entry '{}'", source.name),
                &source.exclude,
            )?;
            check_field(
                "obligations",
                &source.name,
                "statement_column",
                &source.statement_column,
            )?;
            if let Some(combinatorial) = &source.combinatorial {
                check_field(
                    "obligations",
                    &source.name,
                    "combinatorial.dimension_column",
                    &combinatorial.dimension_column,
                )?;
                check_field(
                    "obligations",
                    &source.name,
                    "combinatorial.values_column",
                    &combinatorial.values_column,
                )?;
                if let Some(column) = &combinatorial.excludes_column {
                    check_field(
                        "obligations",
                        &source.name,
                        "combinatorial.excludes_column",
                        column,
                    )?;
                }
                // Strength 0 mints an obligation over nothing, which reads as a
                // declared configuration space that demands no coverage at all
                // — worse than absent, because it looks answered.
                if combinatorial.strength == 0 {
                    return Err(format!(
                        "traceability: obligations entry '{}' declares combinatorial strength 0, \
                         which demands coverage of nothing while reading as declared",
                        source.name
                    ));
                }
            }
            for (key, column) in &source.parameters {
                if key.trim().is_empty() {
                    return Err(format!(
                        "traceability: obligations entry '{}' declares a parameter with an \
                         empty key",
                        source.name
                    ));
                }
                check_field(
                    "obligations",
                    &source.name,
                    &format!("parameters.{key}"),
                    column,
                )?;
            }
        }

        // FR-058: a required relation that cannot be executed must fail at
        // load. Every rule below rejects a declaration whose *runtime* effect
        // is silent and wrong rather than absent — the failure mode this whole
        // model is built to avoid, since a check reporting nothing and a check
        // reporting everything both look like "no bug here" from the outside.
        let mut relation_names: BTreeSet<&str> = BTreeSet::new();
        for relation in &self.required_relations {
            check_named("required_relations", &relation.name)?;
            if !relation_names.insert(relation.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate required_relations entry '{}'",
                    relation.name
                ));
            }
            check_field("required_relations", &relation.name, "from", &relation.from)?;
            // No accepted verb means no edge can satisfy the relation, so
            // EVERY `from` document is reported. That reads as a corpus-wide
            // defect rather than as the empty declaration it is.
            if relation.edges.is_empty() {
                return Err(format!(
                    "traceability: required_relations entry '{}' declares no edges, so no link \
                     could ever satisfy it and every '{}' document would be reported",
                    relation.name, relation.from
                ));
            }
            for edge in &relation.edges {
                check_field("required_relations", &relation.name, "edges", edge)?;
            }
            // An empty entry here is the mirror image: `to` is meaningful when
            // absent (any document satisfies), so a blank string is a typo that
            // would narrow the accepted set to a target nothing can match.
            for target in &relation.to {
                check_field("required_relations", &relation.name, "to", target)?;
            }
            check_field(
                "required_relations",
                &relation.name,
                "check",
                &relation.check,
            )?;
            // The token is the `<check>` half of a `trace:<check>` severity key
            // (FR-057). One that cannot round-trip through the registry is a
            // relation whose severity can never be tuned or switched off —
            // `is_severity_key` is the same predicate the `--severity` CLI
            // parser uses, so both surfaces accept exactly one vocabulary.
            if !crate::grammar::is_severity_key(&crate::grammar::severity_key(
                "trace",
                &relation.check,
            )) {
                return Err(format!(
                    "traceability: required_relations entry '{}' has a `check` token '{}' that \
                     cannot form a `trace:<check>` severity key, so its severity could never be \
                     configured",
                    relation.name, relation.check
                ));
            }
            check_excludes(
                &format!("required_relations entry '{}'", relation.name),
                &relation.exclude,
            )?;
        }

        // An empty verb here would make `check_acyclic` walk a graph keyed on
        // the empty string — no edge matches, so the cycle check silently
        // covers nothing while the declaration says it does.
        for edge in &self.acyclic_edges {
            if edge.trim().is_empty() {
                return Err(
                    "traceability: acyclic_edges contains an empty verb, which would check \
                     nothing while appearing to be declared"
                        .to_string(),
                );
            }
        }

        // FR-059: same discipline as required relations — a declaration whose
        // runtime effect is silent and wrong fails at load.
        let mut coverage_names: BTreeSet<&str> = BTreeSet::new();
        for coverage in &self.vocabulary_coverage {
            check_named("vocabulary_coverage", &coverage.name)?;
            if !coverage_names.insert(coverage.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate vocabulary_coverage entry '{}'",
                    coverage.name
                ));
            }
            check_field(
                "vocabulary_coverage",
                &coverage.name,
                "from",
                &coverage.from,
            )?;
            check_field(
                "vocabulary_coverage",
                &coverage.name,
                "field",
                &coverage.field,
            )?;
            check_field(
                "vocabulary_coverage",
                &coverage.name,
                "check",
                &coverage.check,
            )?;
            if !crate::grammar::is_severity_key(&crate::grammar::severity_key(
                "trace",
                &coverage.check,
            )) {
                return Err(format!(
                    "traceability: vocabulary_coverage entry '{}' has a `check` token '{}' that \
                     cannot form a `trace:<check>` severity key, so its severity could never be \
                     configured",
                    coverage.name, coverage.check
                ));
            }
            if let Some(field) = &coverage.justified_absence_field {
                check_field(
                    "vocabulary_coverage",
                    &coverage.name,
                    "justified_absence_field",
                    field,
                )?;
            }
            check_excludes(
                &format!("vocabulary_coverage entry '{}'", coverage.name),
                &coverage.exclude,
            )?;
        }

        if let Some(status) = &self.status {
            check_field("status", &status.column, "column", &status.column)?;
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for value in status
                .complete
                .iter()
                .chain(&status.pending)
                .chain(&status.failed)
            {
                if !seen.insert(value.as_str()) {
                    return Err(format!(
                        "traceability: status value '{value}' is declared in more than one class"
                    ));
                }
            }
        }

        // CR-041: the exemption reads a column, so the column must be named,
        // and a value outside the declared vocabulary is a typo that would
        // silently exempt nothing.
        if !self.vocabularies.no_source_symbol.is_empty() {
            if self.vocabularies.test_type_column.is_none() {
                return Err("traceability: vocabularies.no_source_symbol needs \
                            vocabularies.test_type_column to say which column carries the value"
                    .to_string());
            }
            for value in &self.vocabularies.no_source_symbol {
                if !self
                    .vocabularies
                    .test_type
                    .iter()
                    .any(|t| t.trim().eq_ignore_ascii_case(value.trim()))
                {
                    return Err(format!(
                        "traceability: vocabularies.no_source_symbol value '{value}' is not in \
                         the declared test_type vocabulary"
                    ));
                }
            }
        }

        let mut marker_names: BTreeSet<&str> = BTreeSet::new();
        for marker in &self.trace_tags.markers {
            check_named("trace_tags.markers", &marker.name)?;
            if !marker_names.insert(marker.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate trace_tags.markers entry '{}'",
                    marker.name
                ));
            }
            check_capturing_pattern("trace_tags.markers", &marker.name, &marker.pattern)?;
        }
        for legacy in &self.trace_tags.legacy {
            check_named("trace_tags.legacy", &legacy.name)?;
            check_capturing_pattern("trace_tags.legacy", &legacy.name, &legacy.pattern)?;
            if let Some(rewrite_to) = &legacy.rewrite_to {
                if !marker_names.contains(rewrite_to.as_str()) {
                    return Err(format!(
                        "traceability: trace_tags.legacy entry '{}' rewrites to undeclared \
                         marker '{rewrite_to}'",
                        legacy.name
                    ));
                }
            }
        }
        Ok(())
    }
}

fn check_named(section: &str, name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err(format!("traceability: {section} entry has an empty name"));
    }
    Ok(())
}

fn check_field(section: &str, name: &str, field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "traceability: {section} entry '{name}' has an empty `{field}`"
        ));
    }
    Ok(())
}

/// Exclusion globs must compile; a typo that silently matched nothing would
/// quietly readmit the documents the module meant to keep out (CR-038).
///
/// `location` reads as the place the patterns were authored, because the same
/// rule now guards a model-level `exclude:` that belongs to no entry (CR-060).
fn check_excludes(location: &str, exclude: &[String]) -> Result<(), String> {
    for pattern in exclude {
        if pattern.trim().is_empty() {
            return Err(format!(
                "traceability: {location} has an empty `exclude` pattern"
            ));
        }
        globset::Glob::new(pattern).map_err(|e| {
            format!("traceability: {location} has an invalid `exclude` pattern: {e}")
        })?;
    }
    Ok(())
}

/// Patterns must compile and must capture the id they extract.
fn check_capturing_pattern(section: &str, name: &str, pattern: &str) -> Result<(), String> {
    let compiled = regex::Regex::new(pattern).map_err(|e| {
        format!("traceability: {section} entry '{name}' has an invalid pattern: {e}")
    })?;
    if compiled.captures_len() < 2 {
        return Err(format!(
            "traceability: {section} entry '{name}' pattern captures nothing; the first capture \
             group must be the trace id"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    fn model(yaml: &str) -> TraceabilityModel {
        serde_yaml::from_str(yaml).expect("parse")
    }

    #[trace("TC-945", "FR-050-AC-22")]
    // `source_exclude` is validated at load like every other
    // glob list, and a model declaring only it has still declared nothing
    // (CR-085).
    #[test]
    fn tc945_source_exclude_is_validated_and_declares_nothing() {
        model("source_exclude: ['tests/fixtures/**']")
            .validate()
            .expect("a well-formed glob list is valid");

        // A pattern that does not compile is rejected at load, where it names
        // the module, rather than silently matching nothing forever.
        let err = model("source_exclude: ['tests/fixtures/[**']")
            .validate()
            .expect_err("an invalid glob must be rejected");
        assert!(
            err.contains("source_exclude"),
            "the error must name the key that is wrong: {err}"
        );

        let err = model("source_exclude: ['  ']")
            .validate()
            .expect_err("an empty pattern must be rejected");
        assert!(err.contains("source_exclude"), "{err}");

        // Declaring only what is *not* source reconciles nothing, so the model
        // still reads as undeclared — the same rule the model-level `exclude`
        // follows, and the reason neither is listed in `is_empty`.
        assert!(
            model("source_exclude: ['tests/fixtures/**']").is_empty(),
            "a model that declares only non-source paths has declared nothing"
        );
    }

    const FULL: &str = r#"
trace_targets:
- name: acceptance-criterion
  archetype: FR
  section: Acceptance Criteria
  id_column: ID
- name: test-case
  archetype: TestMatrix
  section: Test Cases
  id_column: TC ID
document_references:
- name: verification
  archetype: FR
  section: Acceptance Criteria
  column: Verification
  row_id_column: ID
  pattern: '\((TC-\d+)\)'
  targets: [test-case]
status:
  column: Status
  complete: ["done"]
  pending: ["planned"]
  failed: ["broken"]
trace_tags:
  markers:
  - name: rust-trace-attribute
    language: rust
    pattern: '#\[trace\(([^)]*)\)\]'
  legacy:
  - name: rust-test-name
    language: rust
    pattern: 'fn (tc\d+)_'
    rewrite_to: rust-trace-attribute
"#;

    #[test]
    fn full_model_parses_and_validates() {
        let m = model(FULL);
        assert!(!m.is_empty());
        m.validate().expect("valid");
        assert_eq!(m.trace_targets.len(), 2);
        assert_eq!(m.target("test-case").unwrap().archetype, "TestMatrix");
        assert_eq!(m.document_references[0].targets, vec!["test-case"]);
        assert_eq!(
            m.status.as_ref().unwrap().class_of(" done "),
            StatusClass::Complete
        );
        assert_eq!(
            m.status.as_ref().unwrap().class_of("???"),
            StatusClass::Unknown
        );
        assert_eq!(m.trace_tags.markers[0].language, SourceLanguage::Rust);
    }

    #[test]
    fn default_model_is_undeclared() {
        assert!(TraceabilityModel::default().is_empty());
    }

    #[test]
    fn validation_rejects_incoherent_declarations() {
        let cases = [
            // reference to an undeclared target
            (
                r#"
document_references:
- name: r
  archetype: FR
  section: S
  column: C
  pattern: '(TC-\d+)'
  targets: [nope]
"#,
                "undeclared target",
            ),
            // an archetype declared as an empty string names nothing
            (
                "trace_targets:\n- name: t\n  archetype: '  '\n  section: S\n  id_column: ID\n",
                "empty `archetype`",
            ),
            // an exclude pattern that cannot compile
            (
                "trace_targets:\n- name: t\n  archetype: FR\n  exclude: ['[bad']\n  section: S\n  id_column: ID\n",
                "invalid `exclude` pattern",
            ),
            // non-capturing pattern
            (
                r#"
trace_targets:
- name: t
  archetype: FR
  section: S
  id_column: ID
document_references:
- name: r
  archetype: FR
  section: S
  column: C
  pattern: 'TC-\d+'
  targets: [t]
"#,
                "captures nothing",
            ),
            // duplicate target names
            (
                "trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n- name: t\n  archetype: NFR\n  section: S\n  id_column: ID\n",
                "duplicate trace_targets",
            ),
            // a status value in two classes
            (
                "status:\n  column: Status\n  complete: [\"x\"]\n  pending: [\"x\"]\n",
                "more than one class",
            ),
        ];
        for (yaml, expected) in cases {
            let err = model(yaml).validate().expect_err(yaml);
            assert!(err.contains(expected), "{err} != {expected}");
        }
    }

    /// TC-829, FR-050-AC-7 (CR-062): `archetype` is the only origin, and it is
    /// required. A `document:` key — the deleted form — is now an unknown field,
    /// which `deny_unknown_fields` rejects rather than silently ignores: a module
    /// carrying the old declaration fails load instead of minting nothing.
    #[test]
    fn tc829_archetype_is_the_only_origin() {
        let m = model(
            "trace_targets:\n- name: test-case\n  archetype: TestMatrix\n\
             \n  exclude: ['tests/fixtures/**']\n  section: Test Case Summary\n  id_column: Test ID\n",
        );
        m.validate().expect("archetype alone is a valid origin");
        let target = m.target("test-case").expect("declared");
        assert_eq!(target.archetype, "TestMatrix");
        assert_eq!(target.exclude, vec!["tests/fixtures/**".to_string()]);

        let stale = "trace_targets:\n- name: test-case\n  document: spec/tests.md\n\
                     \n  section: Test Case Summary\n  id_column: Test ID\n";
        let error = serde_yaml::from_str::<TraceabilityModel>(stale)
            .expect_err("the deleted form must not load");
        assert!(
            error.to_string().contains("document"),
            "the error must name the retired key: {error}"
        );

        let empty = "trace_targets:\n- name: test-case\n  archetype: ''\n\
                     \n  section: Test Case Summary\n  id_column: Test ID\n";
        let error = model(empty).validate().expect_err("empty archetype");
        assert!(error.contains("archetype"), "{error}");

        // A target with no origin at all was a `validate()` error while
        // `archetype` was optional. Now it is a *deserialization* error, which
        // is strictly earlier: the module fails to load rather than loading and
        // then failing its own coherence check.
        let none = "trace_targets:\n- name: test-case\n  section: S\n  id_column: ID\n";
        let error = serde_yaml::from_str::<TraceabilityModel>(none)
            .expect_err("a target with no origin must not load");
        assert!(
            error.to_string().contains("archetype"),
            "the error must name the missing origin: {error}"
        );
    }

    #[test]
    fn exclude_defaults_to_empty_and_is_omitted_when_unset() {
        let m = model(FULL);
        assert!(m.target("test-case").unwrap().exclude.is_empty());
        // FR-050-AC-7: a model declaring no exclusions must serialize exactly
        // as it did before the field existed.
        let yaml = serde_yaml::to_string(&m).expect("serialize");
        assert!(!yaml.contains("exclude"), "{yaml}");
    }

    #[trace("TC-826")]
    // the model-level `exclude:` is a repository fact, so it (CR-060)
    // is not what makes a model *declared* — a module that says only "these
    // paths are not corpus data" has declared no model to reconcile against.
    #[test]
    fn model_level_exclude_alone_is_still_an_undeclared_model() {
        let m = model("exclude: ['spec/fixtures/**']\n");
        assert_eq!(m.exclude, vec!["spec/fixtures/**".to_string()]);
        assert!(m.is_empty());
        m.validate().expect("valid");

        // FR-050-AC-7: unset, it serializes exactly as before the field.
        let yaml = serde_yaml::to_string(&model(FULL)).expect("serialize");
        assert!(!yaml.contains("exclude"), "{yaml}");
    }

    #[trace("TC-826")]
    // the same compile check `exclude:` has carried since (CR-060)
    // CR-038 — a pattern that cannot compile would quietly readmit everything
    // the module meant to keep out.
    #[test]
    fn model_level_exclude_patterns_must_compile() {
        let bad = model("exclude: ['[bad']\n");
        let err = bad.validate().expect_err("must not validate");
        assert!(err.contains("invalid `exclude` pattern"), "{err}");
        assert!(
            err.contains("model-level"),
            "names where it was authored: {err}"
        );

        let empty = model("exclude: ['  ']\n");
        let err = empty.validate().expect_err("must not validate");
        assert!(err.contains("empty `exclude` pattern"), "{err}");
    }

    #[test]
    fn unknown_field_is_a_parse_error() {
        let bad = "trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n  typo: x\n";
        assert!(serde_yaml::from_str::<TraceabilityModel>(bad).is_err());
    }
}
