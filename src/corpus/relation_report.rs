//! Two-way relation coverage report.
//!
//! [`required_relations`](super::required_relations) reports a document that
//! lacks a declared edge as an advisory finding. This module reads the same
//! declarations and produces a **report**: one row per obliged document, with
//! what is linked to it and how well that is verified.
//!
//! Nothing here is engine knowledge: no archetype, verb, field or value is
//! written in this file. The rows exist because a module declared a
//! `required_relations` entry, the `state` is the declared-relation satisfaction, and the
//! verification is read from the `coverage_matrix` the caller supplies.
//! The report adds no declaration and no graph walk of its own, and writes
//! nothing back to any document.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Serialize;

use super::declared_tables::{relative_path, ExcludeSet};
use super::relation_edges::{CriterionIndex, RelationEdges};
use super::required_relations::{is_satisfied, obliged};
use super::spec::Spec;
use crate::coverage::{CoverageMatrixRequirement, CoverageMatrixStatus};
use crate::registry::Registry;
use crate::traceability::{RelationDirection, RequiredRelation};

/// What the report reads from one corpus member: the root its paths are
/// relative to, and its `coverage_matrix` ([`crate::CoverageReport`]).
#[derive(Debug, Clone, Copy)]
pub struct RelationReportMember<'a> {
    /// The member's repository root. Relation `exclude` globs and the matrix's
    /// `document` paths are relative to it, and the report's `path` fields are
    /// relative to it.
    pub root: &'a Path,
    /// The member's `coverage_matrix`; empty when its module declares no
    /// `obligations:` source.
    pub matrix: &'a [CoverageMatrixRequirement],
}

/// Why a relation report could not be produced.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RelationReportError {
    /// A named relation is declared by no loaded module.
    #[error("no loaded module declares required relation '{name}'")]
    UnknownRelation {
        /// The name as supplied.
        name: String,
    },
    /// A `design_relations` entry names a relation no loaded module declares.
    #[error(
        "design_relations entry '{name}' names no required relation declared by a loaded module"
    )]
    UnknownDesignRelation {
        /// The entry as declared.
        name: String,
    },
    /// One [`RelationReportMember`] is supplied per corpus member.
    #[error("corpus has {members} member(s) but {supplied} member input(s) were supplied")]
    MemberCount {
        /// Members in the corpus.
        members: usize,
        /// Inputs supplied.
        supplied: usize,
    },
}

/// A verification status, derived from the `coverage_matrix`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verification {
    /// At least one measured criterion, and every one is backed.
    Verified,
    /// At least one backed and at least one measured criterion not backed.
    Partial,
    /// At least one measured criterion, none backed.
    Unverified,
    /// No matrix entry, or no non-retired measured criterion.
    NotTracked,
}

/// Whether the relation is satisfied for a row's document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RowState {
    /// The relation is not satisfied.
    Uncovered,
    /// The relation is satisfied.
    Covered,
}

/// A document's criteria counts; `not_measured` is visible so its exclusion
/// from the denominator is too.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Criteria {
    /// Criteria with status `tagged` or `qualified-record`.
    pub backed: usize,
    /// Measured criteria not backed.
    pub unbacked: usize,
    /// Criteria whose method mints no source symbol.
    pub not_measured: usize,
}

impl Criteria {
    fn verification(self) -> Verification {
        let measured = self.backed + self.unbacked;
        if measured == 0 {
            Verification::NotTracked
        } else if self.unbacked == 0 {
            Verification::Verified
        } else if self.backed == 0 {
            Verification::Unverified
        } else {
            Verification::Partial
        }
    }
}

/// A document in a row: the obliged one, or a resolved `linked` counterpart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RowDocument {
    /// Artifact id.
    pub id: String,
    /// Path relative to the member's root.
    pub path: String,
    /// The document's archetype; absent for an untyped document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archetype: Option<String>,
    /// The member's declared identity, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// The document's own verification.
    pub verification: Verification,
    /// The counts the verification is computed from.
    pub criteria: Criteria,
}

/// The `where` value that selected a row's document, as authored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RowSelector {
    /// The selector field.
    pub field: String,
    /// The document's value for it.
    pub value: serde_json::Value,
}

/// Whether a `linked` entry's counterpart is a loaded document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkResolution {
    /// The counterpart resolved to a loaded document.
    Resolved,
    /// The counterpart is absent from the corpus.
    Dangling,
}

/// A document on the other end of an accepted edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkedEntry {
    /// The target or source resolved to a loaded document.
    Resolved {
        /// The counterpart document.
        document: RowDocument,
        /// The accepted verb of the edge.
        verb: String,
        /// The distinct criterion ids of the pair's edges, in lexical order;
        /// empty when the pair is linked as a whole. `None` outside the
        /// design-element join (FR-086), which is the only report that carries
        /// the key.
        realized_criteria: Option<Vec<String>>,
    },
    /// An accepted-verb edge whose target is absent, satisfying a relation
    /// with `to: []`. It is never tracked: serialized `not-tracked` with zero
    /// criteria.
    Dangling {
        /// The target as authored.
        reference: String,
        /// The accepted verb of the edge.
        verb: String,
    },
}

impl LinkedEntry {
    /// Whether the counterpart resolved.
    pub fn resolution(&self) -> LinkResolution {
        match self {
            Self::Resolved { .. } => LinkResolution::Resolved,
            Self::Dangling { .. } => LinkResolution::Dangling,
        }
    }
}

impl Serialize for LinkedEntry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Resolved<'a> {
            #[serde(flatten)]
            document: &'a RowDocument,
            verb: &'a str,
            resolution: LinkResolution,
            #[serde(skip_serializing_if = "Option::is_none")]
            realized_criteria: &'a Option<Vec<String>>,
        }
        #[derive(Serialize)]
        struct Dangling<'a> {
            reference: &'a str,
            verb: &'a str,
            resolution: LinkResolution,
            verification: Verification,
            criteria: Criteria,
        }
        match self {
            Self::Resolved {
                document,
                verb,
                realized_criteria,
            } => Resolved {
                document,
                verb,
                resolution: LinkResolution::Resolved,
                realized_criteria,
            }
            .serialize(serializer),
            Self::Dangling { reference, verb } => Dangling {
                reference,
                verb,
                resolution: LinkResolution::Dangling,
                verification: Verification::NotTracked,
                criteria: Criteria::default(),
            }
            .serialize(serializer),
        }
    }
}

/// An accepted-verb edge whose target is absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DanglingEntry {
    /// The target as authored.
    pub reference: String,
    /// The edge's verb.
    pub verb: String,
}

/// An accepted-verb edge whose target resolved outside the relation's `to`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KindMismatchEntry {
    /// The target's id.
    pub id: String,
    /// The target's member identity, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// The target's archetype; absent for an untyped document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archetype: Option<String>,
    /// The edge's verb.
    pub verb: String,
    /// The relation's `to` as declared.
    pub accepted: Vec<String>,
}

/// One obliged document of one relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationRow {
    /// The declaration's `name`.
    pub relation: String,
    /// The declaration's `check` token.
    pub check: String,
    /// The declaration's `direction`.
    pub direction: RelationDirection,
    /// The obliged document.
    pub document: RowDocument,
    /// Present only when the declaration has `where`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<RowSelector>,
    /// Satisfaction of the declared relation.
    pub state: RowState,
    /// The aggregate over the row's tracked documents.
    pub verification: Verification,
    /// Documents on the other end of an accepted edge.
    pub linked: Vec<LinkedEntry>,
    /// Accepted-verb edges whose target is absent.
    pub dangling: Vec<DanglingEntry>,
    /// Accepted-verb edges whose target is outside `to`.
    pub kind_mismatch: Vec<KindMismatchEntry>,
}

/// Row counts by state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct StateCounts {
    /// Rows not satisfied.
    pub uncovered: usize,
    /// Rows satisfied.
    pub covered: usize,
}

/// Row counts by verification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct VerificationCounts {
    /// Rows `verified`.
    pub verified: usize,
    /// Rows `partial`.
    pub partial: usize,
    /// Rows `unverified`.
    pub unverified: usize,
    /// Rows `not-tracked`.
    #[serde(rename = "not-tracked")]
    pub not_tracked: usize,
}

/// A relation's row counts; every count is present even at zero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct RelationSummary {
    /// Rows in the relation.
    pub rows: usize,
    /// Rows by state.
    pub state: StateCounts,
    /// Rows by verification.
    pub verification: VerificationCounts,
}

/// One declared relation and its rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationSection {
    /// The declaration's `name`.
    pub name: String,
    /// The declaration's `check` token.
    pub check: String,
    /// The declaration's `direction`.
    pub direction: RelationDirection,
    /// Row counts.
    pub summary: RelationSummary,
    /// Rows ordered by repo, document path and id.
    pub rows: Vec<RelationRow>,
}

/// The two-way relation coverage report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationReport {
    /// One section per reported relation, in declaration order.
    pub relations: Vec<RelationSection>,
}

/// Implements: FR-084
/// Produce the relation coverage report for `corpus`.
///
/// `members` holds one input per corpus member, in member order. `names`
/// restricts the report to the named relations; empty reports every declared
/// relation. Sections are in declaration order whatever order `names` is in.
pub fn relation_report(
    corpus: &Spec,
    registry: &Registry,
    members: &[RelationReportMember<'_>],
    names: &[String],
) -> Result<RelationReport, RelationReportError> {
    check_members(corpus, members)?;
    let model = registry.traceability();
    let declared = model.map_or(&[][..], |m| m.required_relations.as_slice());
    if let Some(name) = names
        .iter()
        .find(|n| !declared.iter().any(|r| &r.name == *n))
    {
        return Err(RelationReportError::UnknownRelation { name: name.clone() });
    }
    let relations: Vec<&RequiredRelation> = declared
        .iter()
        .filter(|r| names.is_empty() || names.contains(&r.name))
        .collect();
    Ok(report_of(corpus, registry, members, &relations, false))
}

/// Implements: FR-086
/// Produce the design-element join for `corpus`: the [`relation_report`] over
/// the relations the loaded modules name in `design_relations`, in that order.
///
/// A requirement with no design element is a row with state `uncovered`, never
/// an absence, and each resolved `linked` entry carries `realized_criteria`.
/// A corpus whose modules name no design relation yields no sections. An entry
/// that names no declared relation is an error naming it, and no report.
pub fn design_join(
    corpus: &Spec,
    registry: &Registry,
    members: &[RelationReportMember<'_>],
) -> Result<RelationReport, RelationReportError> {
    check_members(corpus, members)?;
    let Some(model) = registry.traceability() else {
        return Ok(RelationReport { relations: vec![] });
    };
    let relations = model
        .design_relations
        .iter()
        .map(|name| {
            model
                .required_relations
                .iter()
                .find(|r| &r.name == name)
                .ok_or_else(|| RelationReportError::UnknownDesignRelation { name: name.clone() })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(report_of(corpus, registry, members, &relations, true))
}

fn check_members(
    corpus: &Spec,
    members: &[RelationReportMember<'_>],
) -> Result<(), RelationReportError> {
    if members.len() == corpus.member_count() {
        return Ok(());
    }
    Err(RelationReportError::MemberCount {
        members: corpus.member_count(),
        supplied: members.len(),
    })
}

fn report_of(
    corpus: &Spec,
    registry: &Registry,
    members: &[RelationReportMember<'_>],
    relations: &[&RequiredRelation],
    realized: bool,
) -> RelationReport {
    let model_exclude = ExcludeSet::compile_validated(
        registry
            .traceability()
            .map_or(&[][..], |model| model.exclude.as_slice()),
    );
    let standing: Vec<BTreeMap<String, Criteria>> =
        members.iter().map(|m| standing_of(m)).collect();
    let roots: Vec<&Path> = members.iter().map(|m| m.root).collect();
    let criteria = CriterionIndex::build(corpus, registry, &roots);
    let ctx = Context {
        corpus,
        registry,
        criteria: &criteria,
        members,
        standing: &standing,
        realized,
    };
    RelationReport {
        relations: relations
            .iter()
            .map(|relation| ctx.section(relation, &model_exclude))
            .collect(),
    }
}

/// Criteria counts per document path, from one member's matrix.
fn standing_of(member: &RelationReportMember<'_>) -> BTreeMap<String, Criteria> {
    let mut out: BTreeMap<String, Criteria> = BTreeMap::new();
    for requirement in member.matrix {
        let counts = out.entry(requirement.document.clone()).or_default();
        for criterion in &requirement.criteria {
            match criterion.status {
                CoverageMatrixStatus::Retired => {}
                CoverageMatrixStatus::MethodWithoutSymbol => counts.not_measured += 1,
                CoverageMatrixStatus::Tagged | CoverageMatrixStatus::QualifiedRecord => {
                    counts.backed += 1;
                }
                CoverageMatrixStatus::Untagged | CoverageMatrixStatus::TaggedByIgnoredTest => {
                    counts.unbacked += 1;
                }
            }
        }
    }
    out
}

struct Context<'a> {
    corpus: &'a Spec,
    registry: &'a Registry,
    criteria: &'a CriterionIndex,
    /// Whether `linked` entries carry `realized_criteria` (the design join).
    realized: bool,
    members: &'a [RelationReportMember<'a>],
    standing: &'a [BTreeMap<String, Criteria>],
}

/// Sort key of a counterpart: repo (absent first), then id, then verb.
type LinkKey = (Option<String>, String, String);

impl Context<'_> {
    fn section(&self, relation: &RequiredRelation, model_exclude: &ExcludeSet) -> RelationSection {
        let mut rows: Vec<RelationRow> = Vec::new();
        let edges = RelationEdges::new(self.corpus, self.registry, self.criteria, relation);
        for (member, input) in self.members.iter().enumerate() {
            let view = self.corpus.member_view(member);
            for obliged in obliged(&view, relation, input.root, model_exclude) {
                rows.push(self.row(relation, &edges, obliged.slot, obliged.value));
            }
        }
        rows.sort_by(|a, b| {
            (&a.document.repo, &a.document.path, &a.document.id).cmp(&(
                &b.document.repo,
                &b.document.path,
                &b.document.id,
            ))
        });

        let mut summary = RelationSummary {
            rows: rows.len(),
            ..RelationSummary::default()
        };
        for row in &rows {
            match row.state {
                RowState::Uncovered => summary.state.uncovered += 1,
                RowState::Covered => summary.state.covered += 1,
            }
            let counts = &mut summary.verification;
            match row.verification {
                Verification::Verified => counts.verified += 1,
                Verification::Partial => counts.partial += 1,
                Verification::Unverified => counts.unverified += 1,
                Verification::NotTracked => counts.not_tracked += 1,
            }
        }
        RelationSection {
            name: relation.name.clone(),
            check: relation.check.clone(),
            direction: relation.direction,
            summary,
            rows,
        }
    }

    fn document(&self, slot: usize) -> RowDocument {
        let doc = self.corpus.doc(slot);
        let member = self.corpus.member_of(slot);
        let criteria = self
            .standing
            .get(member)
            .and_then(|by_path| by_path.get(&relative_path(self.members[member].root, &doc.path)))
            .copied()
            .unwrap_or_default();
        RowDocument {
            id: doc.id.clone(),
            path: relative_path(self.members[member].root, &doc.path),
            archetype: doc.concept_type().map(str::to_string),
            repo: self.corpus.member_identity(member).map(str::to_string),
            verification: criteria.verification(),
            criteria,
        }
    }

    fn row(
        &self,
        relation: &RequiredRelation,
        edges: &RelationEdges,
        slot: usize,
        value: Option<&serde_json::Value>,
    ) -> RelationRow {
        let corpus = self.corpus;
        let accepted_kinds: BTreeSet<&str> = relation.to.iter().map(String::as_str).collect();
        let kind_of = |slot: usize| corpus.doc(slot).concept_type().unwrap_or("");
        let accepts =
            |slot: usize| accepted_kinds.is_empty() || accepted_kinds.contains(kind_of(slot));

        let mut resolved: BTreeMap<LinkKey, (usize, BTreeSet<String>)> = BTreeMap::new();
        let mut dangling: BTreeSet<(String, String)> = BTreeSet::new();
        let mut mismatched: BTreeMap<LinkKey, KindMismatchEntry> = BTreeMap::new();
        let key_of = |slot: usize, verb: &str| -> LinkKey {
            let member = corpus.member_of(slot);
            (
                corpus.member_identity(member).map(str::to_string),
                corpus.doc(slot).id.clone(),
                verb.to_string(),
            )
        };

        for found in edges.matches(relation.direction, slot) {
            let reading = found.reading;
            match found.counterpart {
                Some(other) if accepts(other) => {
                    resolved
                        .entry(key_of(other, &reading.verb))
                        .or_insert_with(|| (other, BTreeSet::new()))
                        .1
                        .extend(reading.criteria.iter().cloned());
                }
                // Only the obliged document's own outgoing edge names a wrong
                // kind; under `incoming` a wrong-kind source is not a link and
                // not a mismatch (FR-085-AC-25).
                Some(other) if found.outgoing => {
                    let member = corpus.member_of(other);
                    mismatched.insert(
                        key_of(other, &reading.verb),
                        KindMismatchEntry {
                            id: corpus.doc(other).id.clone(),
                            repo: corpus.member_identity(member).map(str::to_string),
                            archetype: corpus.doc(other).concept_type().map(str::to_string),
                            verb: reading.verb.clone(),
                            accepted: relation.to.clone(),
                        },
                    );
                }
                Some(_) => {}
                None => {
                    if let Some(reference) = &reading.reference {
                        dangling.insert((reference.clone(), reading.verb.clone()));
                    }
                }
            }
        }

        // `to: []` makes the verb alone the contract, so a dangling edge is
        // also listed as a link, never as a tracked document.
        let mut linked: Vec<LinkedEntry> = resolved
            .into_iter()
            .map(|((_, _, verb), (other, ids))| LinkedEntry::Resolved {
                document: self.document(other),
                verb,
                realized_criteria: self.realized.then(|| ids.into_iter().collect()),
            })
            .collect();
        if relation.to.is_empty() {
            linked.extend(
                dangling
                    .iter()
                    .map(|(reference, verb)| LinkedEntry::Dangling {
                        reference: reference.clone(),
                        verb: verb.clone(),
                    }),
            );
        }

        let document = self.document(slot);
        let mut tracked: BTreeMap<(Option<String>, String), Verification> = BTreeMap::new();
        let mut track = |doc: &RowDocument| {
            if doc.verification != Verification::NotTracked {
                tracked.insert((doc.repo.clone(), doc.id.clone()), doc.verification);
            }
        };
        track(&document);
        for entry in &linked {
            if let LinkedEntry::Resolved { document, .. } = entry {
                track(document);
            }
        }
        let verification = aggregate(tracked.values().copied());

        RelationRow {
            relation: relation.name.clone(),
            check: relation.check.clone(),
            direction: relation.direction,
            selector: relation
                .selector
                .as_ref()
                .zip(value)
                .map(|(selector, value)| RowSelector {
                    field: selector.field().to_string(),
                    value: value.clone(),
                }),
            document,
            state: if is_satisfied(corpus, edges, relation, slot) {
                RowState::Covered
            } else {
                RowState::Uncovered
            },
            verification,
            linked,
            dangling: dangling
                .into_iter()
                .map(|(reference, verb)| DanglingEntry { reference, verb })
                .collect(),
            kind_mismatch: mismatched.into_values().collect(),
        }
    }
}

/// `verified` when all are, `unverified` when none is `verified` or `partial`,
/// `partial` otherwise, `not-tracked` for no tracked document.
fn aggregate(tracked: impl Iterator<Item = Verification>) -> Verification {
    let statuses: Vec<Verification> = tracked.collect();
    if statuses.is_empty() {
        Verification::NotTracked
    } else if statuses.iter().all(|s| *s == Verification::Verified) {
        Verification::Verified
    } else if statuses.iter().all(|s| *s == Verification::Unverified) {
        Verification::Unverified
    } else {
        Verification::Partial
    }
}
