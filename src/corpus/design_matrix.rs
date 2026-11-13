//! The design column on the coverage matrix.
//!
//! [`design_join`] answers requirement to design element; the FR-050
//! `coverage_matrix` answers requirement to criterion to test. This module
//! runs the join once, over the matrix the caller supplies, and attaches each
//! row to the matrix entry of the same member and document, so one payload
//! shows the whole chain.
//!
//! Nothing here is engine knowledge: no archetype, verb or field name is
//! written in this file. Which documents carry a `design` block is decided by
//! the loaded modules' `design_relations`; a block is the FR-086 row, with its
//! `archetype` named `kind`. The payload has no schema of its own and is not a
//! coverage-v1 `coverage_matrix`. The matrix and the `coverage` report are not
//! changed, and nothing is written back to any document.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::relation_report::{
    design_join, DanglingEntry, KindMismatchEntry, LinkedEntry, RelationReportError,
    RelationReportMember, RelationRow, RowState,
};
use super::spec::Spec;
use crate::coverage::{CoverageMatrixCriterion, CoverageMatrixRequirement};
use crate::registry::Registry;

/// A design element that realizes a requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesignElement {
    /// The element's artifact id.
    pub id: String,
    /// The element's archetype (FR-086 `archetype`); absent for an untyped document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Path relative to the element's member root.
    pub path: String,
    /// The element's member identity, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// The accepted verb of the edge.
    pub verb: String,
    /// Criterion ids the element names; empty when it realizes the requirement as a whole.
    pub realized_criteria: Vec<String>,
}

/// One design relation's block for an entry: the FR-086 row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesignBlock {
    /// The relation's name.
    pub relation: String,
    /// The relation's `trace:<check>` token.
    pub check: String,
    /// `covered` or `uncovered`.
    pub state: RowState,
    /// The row's resolved `linked` entries, in the row's order.
    pub elements: Vec<DesignElement>,
    /// Accepted-verb edges whose target is absent.
    pub dangling: Vec<DanglingEntry>,
    /// Accepted-verb edges whose target is outside the relation's `to`.
    pub kind_mismatch: Vec<KindMismatchEntry>,
}

impl DesignBlock {
    fn from_row(row: RelationRow) -> Self {
        let elements = row
            .linked
            .into_iter()
            .filter_map(|entry| match entry {
                LinkedEntry::Resolved {
                    document,
                    verb,
                    realized_criteria,
                } => Some(DesignElement {
                    id: document.id,
                    kind: document.archetype,
                    path: document.path,
                    repo: document.repo,
                    verb,
                    realized_criteria: realized_criteria.unwrap_or_default(),
                }),
                LinkedEntry::Dangling { .. } => None,
            })
            .collect();
        Self {
            relation: row.relation,
            check: row.check,
            state: row.state,
            elements,
            dangling: row.dangling,
            kind_mismatch: row.kind_mismatch,
        }
    }
}

/// A reference to a design element from a criterion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesignElementRef {
    /// The element's artifact id.
    pub id: String,
    /// The element's member identity, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

/// A matrix criterion, with the elements that realize it when its entry has a `design` key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesignMatrixCriterion {
    /// The criterion exactly as the `coverage_matrix` carries it.
    #[serde(flatten)]
    pub criterion: CoverageMatrixCriterion,
    /// Present exactly when the entry has `design`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub design_elements: Option<Vec<DesignElementRef>>,
}

/// One requirement entry of the `design_matrix`.
///
/// Fields serialize in the order `document`, `repo`, `id`, `criticality`,
/// `carried`, `criteria`, `design`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesignMatrixEntry {
    /// The stating document, relative to its member's root.
    pub document: String,
    /// The member's identity, when it declares one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// The document's artifact id; present exactly when the entry has `design`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The document's frontmatter `criticality`, as FR-050 and FR-082 state it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criticality: Option<serde_json::Value>,
    /// The FR-090 `carried` object, when the member's matrix entry has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carried: Option<serde_json::Map<String, serde_json::Value>>,
    /// The entry's criteria; empty only for a document a design relation obliges
    /// and no `obligations:` source yields a criterion for.
    pub criteria: Vec<DesignMatrixCriterion>,
    /// One block per design relation that obliges the document, in
    /// `design_relations` order. A missing key means no design relation obliges it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub design: Option<Vec<DesignBlock>>,
}

/// Implements: FR-092
/// Produce the `coverage_matrix` with a `design` column, for `corpus`.
///
/// `members` holds one input per corpus member, in member order, with the
/// member's `coverage_matrix`. The join ([`design_join`]) runs once; each of
/// its rows attaches to the entry of the same member whose `document` is the
/// row's path, and a document a relation obliges that the matrix has no entry
/// for gets an entry with `criteria: []`. Entries are ordered by `repo`
/// (absent first), then `document`. An entry no design relation obliges has no
/// `design` key. A `design_relations` entry naming no declared relation is an
/// error and no matrix.
pub fn design_matrix(
    corpus: &Spec,
    registry: &Registry,
    members: &[RelationReportMember<'_>],
) -> Result<Vec<DesignMatrixEntry>, RelationReportError> {
    let join = design_join(corpus, registry, members)?;

    // The obliged document's id and blocks per (repo, document path), the
    // blocks in section (design_relations) order.
    let mut obliged: BTreeMap<(Option<String>, String), (String, Vec<DesignBlock>)> =
        BTreeMap::new();
    for section in join.relations {
        for row in section.rows {
            let key = (row.document.repo.clone(), row.document.path.clone());
            let id = row.document.id.clone();
            obliged
                .entry(key)
                .or_insert_with(|| (id, Vec::new()))
                .1
                .push(DesignBlock::from_row(row));
        }
    }

    let mut entries: Vec<DesignMatrixEntry> = Vec::new();
    for (index, member) in members.iter().enumerate() {
        let repo = corpus.member_identity(index).map(str::to_string);
        for requirement in member.matrix {
            let key = (repo.clone(), requirement.document.clone());
            let (id, design) = match obliged.remove(&key) {
                Some((id, blocks)) => (Some(id), Some(blocks)),
                None => (None, None),
            };
            entries.push(entry_of(requirement, repo.clone(), id, design));
        }
    }
    // Obliged documents the matrix holds no entry for: no criterion yields one.
    for ((repo, document), (id, design)) in obliged {
        // A criteria-less entry has no matrix entry to read `criticality` or
        // `carried` from: both are read from this document.
        let doc = match repo.as_deref() {
            Some(identity) => corpus.by_qualified_id(identity, &id),
            None => corpus.by_id(&id),
        };
        let criticality = doc.and_then(crate::coverage::document_criticality);
        entries.push(DesignMatrixEntry {
            document,
            repo,
            id: Some(id),
            criticality,
            carried: doc.and_then(|doc| {
                registry
                    .traceability()
                    .and_then(|model| crate::coverage::carried_for(doc, model))
            }),
            criteria: Vec::new(),
            design: Some(design),
        });
    }
    entries.sort_by(|a, b| (&a.repo, &a.document).cmp(&(&b.repo, &b.document)));
    Ok(entries)
}

fn entry_of(
    requirement: &CoverageMatrixRequirement,
    repo: Option<String>,
    id: Option<String>,
    design: Option<Vec<DesignBlock>>,
) -> DesignMatrixEntry {
    // Exhaustive: a field added to the matrix entry (FR-090 `carried`) is a
    // compile error here until this entry decides what to do with it.
    let CoverageMatrixRequirement {
        document,
        criticality,
        carried,
        criteria,
    } = requirement;
    let criteria = criteria
        .iter()
        .map(|criterion| DesignMatrixCriterion {
            design_elements: design
                .as_deref()
                .map(|blocks| elements_of(blocks, &criterion.id)),
            criterion: criterion.clone(),
        })
        .collect();
    DesignMatrixEntry {
        document: document.clone(),
        repo,
        id,
        criticality: criticality.clone(),
        carried: carried.clone(),
        criteria,
        design,
    }
}

/// The elements of `blocks` that realize criterion `id`: those naming no
/// criterion (the requirement as a whole) or naming `id`, in block then element
/// order, once each by (`repo`, `id`).
fn elements_of(blocks: &[DesignBlock], id: &str) -> Vec<DesignElementRef> {
    let mut seen: BTreeSet<(&Option<String>, &str)> = BTreeSet::new();
    blocks
        .iter()
        .flat_map(|block| &block.elements)
        .filter(|element| {
            element.realized_criteria.is_empty()
                || element.realized_criteria.iter().any(|c| c == id)
        })
        .filter(|element| seen.insert((&element.repo, element.id.as_str())))
        .map(|element| DesignElementRef {
            id: element.id.clone(),
            repo: element.repo.clone(),
        })
        .collect()
}
