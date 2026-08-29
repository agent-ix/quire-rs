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

/// The heading, or headings, a declared table may live under (CR-118, #272).
///
/// Authored as a scalar or as a sequence under the **same** `section:` key:
///
/// ```yaml
/// section: Test Case Summary                 # one heading, exactly
/// section: ["Test Case Summary", "Suites"]   # any of these headings
/// section: "*Test Case Summary*"             # the whole family of them
/// ```
///
/// **One key, not a second `sections:` key.** `deny_unknown_fields` means a new
/// key is a new load-time shape, and two keys meaning one thing is a rule that
/// enforces nothing: every reader, every module and every diagnostic would have
/// to handle both, and a module declaring both would need a fourth rule saying
/// which wins. Scalar-or-sequence on one key is the shape YAML consumers
/// already read everywhere else, and the single-string form is unchanged down
/// to the byte — it round-trips back out as a scalar.
///
/// **`*` is the only metacharacter, and it is opt-in.** An entry without one is
/// the heading exactly, matched the way [`crate::query::section`] has always
/// matched it: case-insensitively, with a decorative section number
/// (`## 2.1 Test Case Summary`) normalized away. So a target declaring one
/// section does not start matching others — the widening happens only where a
/// module writes a `*`.
///
/// **[RAN]** before choosing `*` over a literal list of names: of the 434
/// test-case ids across `~/dev` that sit in a `Test ID` table the ecosystem's
/// `test-case` target cannot reach, **306 are under a heading that contains
/// `Test Case Summary`** and is qualified locally — `Test Case Summary (plugin
/// scope)`, `Phase 4 Test Case Summary`, `Test Case Summary —
/// packages/elements`. Those qualifiers are per-repository and per-phase; a
/// literal list would have to enumerate one repository's phase numbers and
/// would go stale the day somebody wrote `Phase 5`. The list form is kept
/// because a genuinely differently-named section (`Integration Test Matrix`) is
/// a different claim and reads better as one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionNames {
    /// As authored: what a diagnostic names and what serializes back out.
    names: Vec<String>,
    /// One matcher per name, normalized once at load rather than per document.
    matchers: Vec<SectionMatcher>,
}

/// One declared heading, compiled (CR-118).
///
/// Normalization is applied to the pattern at construction and to the heading
/// at comparison, so both sides are in the one form [`matches_heading`] has
/// compared since FR-010.
///
/// [`matches_heading`]: crate::query
#[derive(Debug, Clone, PartialEq, Eq)]
enum SectionMatcher {
    /// A name with no `*`: today's exact, normalized, case-insensitive compare.
    Exact(String),
    /// A name with at least one `*`, matched by [`wildcard_match`].
    Wildcard(String),
}

impl SectionMatcher {
    fn new(name: &str) -> Self {
        let normalized = crate::query::normalize_heading(name).to_lowercase();
        if normalized.contains('*') {
            Self::Wildcard(normalized)
        } else {
            Self::Exact(normalized)
        }
    }

    fn matches(&self, normalized_heading: &str) -> bool {
        match self {
            Self::Exact(name) => name == normalized_heading,
            Self::Wildcard(pattern) => wildcard_match(pattern, normalized_heading),
        }
    }
}

/// `*` matches any run of characters, including none. Greedy left-to-right,
/// which is exact for a pattern whose only metacharacter is `*`.
///
/// Deliberately **not** `globset`, although this crate already depends on it
/// for `exclude:`. A glob makes `?`, `[`, `]`, `{` and `}` special too, and a
/// heading is prose. Making four more characters special could change what an
/// existing declaration means.
///
/// **Measured, because an earlier version of this comment cited an invented
/// example.** It named ``Edge Cases [deferred]`` as "a real ecosystem
/// heading"; that heading exists nowhere in the ecosystem and was written to
/// support the conclusion. FR-050 retracted it and this comment did not — the
/// retraction was applied to two documents by hand and did not find this copy,
/// which is why the rule is now a repository-wide search for the exact claim.
///
/// The census that does hold: 21 distinct `section:` values are declared
/// across every `manifest.yaml` under the dev root and **none carries a glob
/// metacharacter**; of the 2,802 headings in 417 `type: TestMatrix`
/// documents, exactly **one** carries `[`/`]` (a markdown link) and none
/// carries `?`, `{`, `}` or `*`. So globset would not change any declaration
/// that exists today — this forecloses a hazard rather than fixing an observed
/// one, and claims no more than that. The bracketed string in this module's
/// tests is a synthetic input, which is fine; calling it observed evidence was
/// not.
///
/// Byte-wise on UTF-8 is safe: `*` is ASCII, and a UTF-8 substring search
/// cannot match across a character boundary.
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let mut segments = pattern.split('*');
    let first = segments.next().unwrap_or_default();
    let Some(mut rest) = text.strip_prefix(first) else {
        return false;
    };
    let segments: Vec<&str> = segments.collect();
    let Some((last, middle)) = segments.split_last() else {
        // No `*` at all: the pattern is one literal and must be the whole text.
        return rest.is_empty();
    };
    for segment in middle {
        let Some(at) = rest.find(segment) else {
            return false;
        };
        rest = &rest[at + segment.len()..];
    }
    // The final segment anchors at the end, and may not overlap what the
    // middle segments already consumed.
    rest.len() >= last.len() && rest.ends_with(last)
}

impl SectionNames {
    /// Build from authored names. Empty names are kept as authored so
    /// [`TraceabilityModel::validate`] can report them against the declaration
    /// they came from rather than silently dropping them here.
    pub fn new(names: Vec<String>) -> Self {
        let matchers = names.iter().map(|n| SectionMatcher::new(n)).collect();
        Self { names, matchers }
    }

    /// The declared headings, as authored — the list a diagnostic names.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// True when a document heading is one of the declared sections.
    pub fn matches(&self, heading: &str) -> bool {
        let normalized = crate::query::normalize_heading(heading).to_lowercase();
        self.matchers.iter().any(|m| m.matches(&normalized))
    }

    /// True when nothing at all was declared — an empty sequence, which
    /// selects no section and is rejected at load.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

impl From<&str> for SectionNames {
    fn from(name: &str) -> Self {
        Self::new(vec![name.to_string()])
    }
}

impl Serialize for SectionNames {
    /// One name serializes as the scalar it was authored as. A model that
    /// round-trips through JSON — every published payload does — must not
    /// rewrite every module's `section:` into a one-element list.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.names.as_slice() {
            [one] => serializer.serialize_str(one),
            many => serializer.collect_seq(many),
        }
    }
}

impl<'de> Deserialize<'de> for SectionNames {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ScalarOrSequence;

        impl<'de> serde::de::Visitor<'de> for ScalarOrSequence {
            type Value = SectionNames;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a section heading, or a sequence of them")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(SectionNames::new(vec![value.to_string()]))
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut names = Vec::new();
                while let Some(name) = seq.next_element::<String>()? {
                    names.push(name);
                }
                Ok(SectionNames::new(names))
            }
        }

        deserializer.deserialize_any(ScalarOrSequence)
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
    /// Heading — or headings (CR-118) — of the section carrying the table.
    /// Required with `archetype`, ignored with `target` (which supplies its
    /// own).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<SectionNames>,
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TraceTargetEvidence {
    /// The existing posture: rows belong to the source-evidence denominator.
    #[default]
    Source,
    /// IDs exist for reference integrity, not for source-symbol coverage.
    ReferenceOnly,
}

impl TraceTargetEvidence {
    fn is_source(&self) -> bool {
        *self == Self::Source
    }
}

fn default_true() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

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
    /// Heading of the section carrying the minting table, or several headings
    /// (CR-118): a matrix whose rows are grouped under `Test Case Summary
    /// (plugin scope)` and `Test Case Summary (discovery scope)` mints from
    /// both, and one heading name minted from neither.
    pub section: SectionNames,
    /// Table column holding the minted id.
    pub id_column: String,
    /// Whether every selected document must carry the minting section.
    /// Omitted preserves the historical required posture. Optional targets
    /// still mint and validate a section when it is present; only its absence
    /// is healthy (FR-050-AC-41, #327).
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub required: bool,
    /// Whether rows require source-symbol evidence. Omitted is the historical
    /// `source` posture; reference-only targets register identifiers without
    /// entering coverage (FR-050-AC-40, #363).
    #[serde(default, skip_serializing_if = "TraceTargetEvidence::is_source")]
    pub evidence: TraceTargetEvidence,
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
    /// Heading of the section carrying the referencing table, or several
    /// (CR-118). A reference reads the SAME table a trace target mints from —
    /// `traces-to` reads `Traces To` off the rows `test-case` mints — so the
    /// two declarations take the same shape here. Widening only the target
    /// would mint ids out of a section whose cells nothing then reads, and the
    /// criteria those rows answer for would report unbacked.
    pub section: SectionNames,
    /// Column whose cells carry the references.
    pub column: String,
    /// Column identifying the referencing row (the row id in report entries).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_id_column: Option<String>,
    /// Status column for this reference table. When omitted, the model-wide
    /// [`StatusVocabulary::column`] remains the default. A per-reference
    /// column is necessary when one archetype contract contains several table
    /// kinds with deliberately different status headers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_column: Option<String>,
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
        check_excludes("model-level", "exclude", &self.exclude)?;
        check_excludes("model-level", "source_exclude", &self.source_exclude)?;

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
                "exclude",
                &target.exclude,
            )?;
            check_sections("trace_targets", &target.name, &target.section)?;
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
                "exclude",
                &reference.exclude,
            )?;
            check_sections("document_references", &reference.name, &reference.section)?;
            check_field(
                "document_references",
                &reference.name,
                "column",
                &reference.column,
            )?;
            if let Some(status_column) = &reference.status_column {
                check_field(
                    "document_references",
                    &reference.name,
                    "status_column",
                    status_column,
                )?;
                if self.status.is_none() {
                    return Err(format!(
                        "traceability: document_references entry '{}' declares `status_column` \
                         but the model has no `status` vocabulary, so the override would \
                         classify nothing",
                        reference.name
                    ));
                }
            }
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
                    check_sections("obligations", &source.name, section)?;
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
                "exclude",
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
                "exclude",
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
                "exclude",
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
        //
        // #259: the exemption now ALSO reads each reference declaration's own
        // column, because that column is the method on an FR criteria table.
        // `test_type_column` stays required anyway — it is the matrix column
        // and the one `test_type` validates — so this check is unchanged; only
        // its reach grew.
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

/// A declared `section:` must name at least one heading, and no blank one
/// (CR-118).
///
/// Both halves are load-time failures for the same reason the empty scalar
/// always was: `section: []` selects no section in any document, and
/// `section: ["Test Case Summary", ""]` selects every section whose normalized
/// heading is empty — which is none, but silently, leaving a reader to wonder
/// which of the two entries did nothing. A declaration that reads nothing is
/// reported against the declaration, not as an empty report.
fn check_sections(location: &str, name: &str, sections: &SectionNames) -> Result<(), String> {
    if sections.is_empty() {
        return Err(format!(
            "traceability: {location} entry '{name}' declares an empty `section` list, so it \
             selects no section in any document"
        ));
    }
    for value in sections.names() {
        check_field(location, name, "section", value)?;
    }
    Ok(())
}

/// Exclusion globs must compile; a typo that silently matched nothing would
/// quietly readmit the documents the module meant to keep out (CR-038).
///
/// `location` reads as the place the patterns were authored, because the same
/// rule now guards a model-level `exclude:` that belongs to no entry (CR-060).
/// `key` is the key the patterns sit under — `exclude` or `source_exclude` —
/// so the error names the thing to fix rather than hardcoding `exclude` for
/// both (#215: the `source_exclude` message read "invalid `exclude` pattern",
/// and tc945's `contains("source_exclude")` was satisfied by the location
/// prefix alone).
fn check_excludes(location: &str, key: &str, exclude: &[String]) -> Result<(), String> {
    for pattern in exclude {
        if pattern.trim().is_empty() {
            return Err(format!(
                "traceability: {location} has an empty `{key}` pattern"
            ));
        }
        globset::Glob::new(pattern)
            .map_err(|e| format!("traceability: {location} has an invalid `{key}` pattern: {e}"))?;
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
mod cr118_props {
    use proptest::prelude::*;

    use super::SectionNames;

    /// Heading-ish text: the punctuation real ecosystem headings carry, plus a
    /// leading section number, and never a `*` — the claim under test is about
    /// names that declare no wildcard.
    fn headingish() -> impl Strategy<Value = String> {
        let token = prop_oneof![
            Just(" ".to_string()),
            Just(" — ".to_string()),
            Just("(".to_string()),
            Just(")".to_string()),
            Just("[".to_string()),
            Just("]".to_string()),
            Just("?".to_string()),
            Just("/".to_string()),
            Just("2.1 ".to_string()),
            "[A-Za-z]{1,6}".prop_map(|s| s),
            "[0-9]{1,3}".prop_map(|s| s),
        ];
        proptest::collection::vec(token, 1..8).prop_map(|v| v.concat())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        /// TC-1038 (FR-050-AC-34, CR-118, Property): a declared name carrying no
        /// `*` selects a section exactly when [`crate::query::section`] — the
        /// lookup every `section:` has gone through since FR-010 — finds it.
        ///
        /// The claim `#272` may not break: the single-string form is unchanged.
        /// Asserted against the real parser and the real query rather than
        /// against a restatement of the matcher.
        #[test]
        fn tc1038_a_star_free_name_agrees_with_the_engines_own_section_lookup(
            declared in headingish(),
            authored in headingish(),
        ) {
            let doc = crate::parser::parse_document(&format!("## {authored}\n\nProse.\n"));
            // The heading as the PARSER read it. Comparing against the
            // generated string instead would fail on inputs the parser does not
            // make a section of at all (`## ` alone), which is a fact about the
            // parser and not about this matcher.
            let sections = crate::query::sections(&doc, None);
            let Some(section) = sections.first() else {
                return Ok(());
            };
            let found = crate::query::section(&doc, &declared).is_some();
            prop_assert_eq!(
                SectionNames::from(declared.as_str()).matches(&section.heading),
                found,
                "declared={:?} authored={:?} parsed={:?}", declared, authored, section.heading
            );
        }
    }
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
        // The **noun**, not merely the location prefix (#215): until then the
        // message read "invalid `exclude` pattern" for a `source_exclude`
        // list, and a bare `contains("source_exclude")` was satisfied by the
        // location alone.
        assert!(
            err.contains("invalid `source_exclude` pattern"),
            "the error must name the key that is wrong, as the noun: {err}"
        );

        let err = model("source_exclude: ['  ']")
            .validate()
            .expect_err("an empty pattern must be rejected");
        assert!(err.contains("empty `source_exclude` pattern"), "{err}");

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
  status_column: Coverage Status
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
            m.document_references[0].status_column.as_deref(),
            Some("Coverage Status")
        );
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
            // an explicit per-reference status column must name a real column
            (
                "trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\ndocument_references:\n- name: r\n  archetype: TestMatrix\n  section: S\n  column: C\n  status_column: '  '\n  pattern: '(TC-\\d+)'\n  targets: [t]\n",
                "empty `status_column`",
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

    #[trace("TC-1075", "FR-050-AC-40", "FR-066-AC-2")]
    #[test]
    fn tc1075_trace_target_evidence_posture_is_typed_and_defaults_to_source() {
        let defaulted =
            model("trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n");
        assert_eq!(
            defaulted.trace_targets[0].evidence,
            TraceTargetEvidence::Source
        );
        let serialized = serde_yaml::to_string(&defaulted).expect("serialize default posture");
        assert!(
            !serialized.contains("evidence:"),
            "the historical declaration stays byte-shaped: {serialized}"
        );

        let reference_only = model(
            "trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n  evidence: reference-only\n",
        );
        assert_eq!(
            reference_only.trace_targets[0].evidence,
            TraceTargetEvidence::ReferenceOnly
        );
        assert!(
            serde_yaml::from_str::<TraceabilityModel>(
                "trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n  evidence: guessed\n",
            )
            .is_err(),
            "an unknown posture must fail at the module boundary"
        );
    }

    #[trace("TC-1076", "FR-050-AC-41")]
    #[test]
    fn tc1076_trace_target_required_defaults_true_and_false_round_trips() {
        let defaulted =
            model("trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n");
        assert!(defaulted.trace_targets[0].required);
        let serialized = serde_yaml::to_string(&defaulted).expect("serialize default");
        assert!(
            !serialized.contains("required:"),
            "existing manifests retain their serialized shape: {serialized}"
        );

        let optional = model(
            "trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n  required: false\n",
        );
        assert!(!optional.trace_targets[0].required);
        assert!(serde_yaml::to_string(&optional)
            .expect("serialize optional")
            .contains("required: false"));
        assert!(
            serde_yaml::from_str::<TraceabilityModel>(
                "trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n  required: sometimes\n",
            )
            .is_err(),
            "the posture is a boolean, not a truthy string"
        );
    }

    #[trace("TC-1038", "FR-050-AC-34")]
    // `section:` reads a scalar or a sequence under the ONE (CR-118)
    // key, a one-name declaration round-trips back out as the scalar it was
    // authored as, and a declaration that selects nothing is a load failure
    // rather than an empty report.
    #[test]
    fn tc1038_section_takes_a_scalar_or_a_sequence_on_one_key() {
        let scalar = model(
            "trace_targets:\n- name: t\n  archetype: TestMatrix\n  \
             section: Test Case Summary\n  id_column: Test ID\n",
        );
        scalar.validate().expect("the single-string form is valid");
        let section = &scalar.trace_targets[0].section;
        assert_eq!(section.names(), ["Test Case Summary"]);

        // Unchanged down to the byte: a model that round-trips — every
        // published payload does — must not rewrite every module's `section:`
        // into a one-element list.
        let json = serde_json::to_value(section).expect("serialize");
        assert_eq!(json, serde_json::json!("Test Case Summary"));

        let sequence = model(
            "trace_targets:\n- name: t\n  archetype: TestMatrix\n  \
             section: ['Test Case Summary', 'Integration Test Matrix']\n  id_column: Test ID\n",
        );
        sequence.validate().expect("the sequence form is valid");
        let section = &sequence.trace_targets[0].section;
        assert_eq!(
            section.names(),
            ["Test Case Summary", "Integration Test Matrix"]
        );
        assert_eq!(
            serde_json::to_value(section).expect("serialize"),
            serde_json::json!(["Test Case Summary", "Integration Test Matrix"]),
            "and a sequence stays a sequence"
        );

        // A declaration that names nothing selects nothing in any document,
        // and says so at load rather than reporting an empty rollup.
        let err = model(
            "trace_targets:\n- name: t\n  archetype: TestMatrix\n  section: []\n  \
             id_column: Test ID\n",
        )
        .validate()
        .expect_err("an empty list must be rejected");
        assert!(err.contains("empty `section` list"), "{err}");

        let err = model(
            "trace_targets:\n- name: t\n  archetype: TestMatrix\n  \
             section: ['Test Case Summary', '  ']\n  id_column: Test ID\n",
        )
        .validate()
        .expect_err("a blank entry must be rejected");
        assert!(err.contains("empty `section`"), "{err}");
    }

    #[trace("TC-1038", "FR-050-AC-34")]
    // matching is unchanged for a name without `*`: (CR-118)
    // case-insensitive, decorative section numbering normalized away, and
    // nothing else. `*` is the only metacharacter, and it is opt-in.
    #[test]
    fn tc1038_a_name_without_a_star_matches_exactly_what_it_always_did() {
        let exact = SectionNames::from("Test Case Summary");
        assert!(exact.matches("Test Case Summary"));
        assert!(exact.matches("test case summary"), "case-insensitive");
        assert!(
            exact.matches("2.1 Test Case Summary"),
            "a decorative section number is normalized away, as FR-010 does"
        );
        // The widening a module did not ask for, which the control fixture in
        // TC-1037 asserts end to end.
        assert!(!exact.matches("Test Case Summary (plugin scope)"));
        assert!(!exact.matches("Phase 4 Test Case Summary"));
        assert!(!exact.matches("Test Cases"));

        // Prose punctuation stays literal. A glob would make four more
        // characters special and silently change what these declarations mean.
        let bracketed = SectionNames::from("Edge Cases [deferred]");
        assert!(bracketed.matches("Edge Cases [deferred]"));
        assert!(!bracketed.matches("Edge Cases d"));
        let brace = SectionNames::from("Coverage {2026}");
        assert!(brace.matches("Coverage {2026}"));
        let question = SectionNames::from("Open Questions?");
        assert!(question.matches("Open Questions?"));
        assert!(!question.matches("Open Questionsx"));

        // `*` runs anywhere, matches any run including none, and anchors at
        // both ends of the heading.
        let prefix = SectionNames::from("Test Case Summary*");
        assert!(prefix.matches("Test Case Summary"), "including none");
        assert!(prefix.matches("Test Case Summary (plugin scope)"));
        assert!(!prefix.matches("Phase 4 Test Case Summary"), "anchored");

        let family = SectionNames::from("*Test Case Summary*");
        assert!(family.matches("Phase 4 Test Case Summary"));
        assert!(family.matches("Test Case Summary — packages/elements"));
        assert!(!family.matches("Test Cases"));

        let inner = SectionNames::from("Test*Summary");
        assert!(inner.matches("Test Case Summary"));
        assert!(!inner.matches("Test Case Summary (plugin scope)"));

        // Several names: any of them.
        let several = SectionNames::new(vec![
            "Test Cases".to_string(),
            "Integration Test Matrix".to_string(),
        ]);
        assert!(several.matches("Test Cases"));
        assert!(several.matches("Integration Test Matrix"));
        assert!(!several.matches("Edge Cases"));
    }
}
