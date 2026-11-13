//! The edges a declared relation matches (FR-058 CR-416, CR-417).
//!
//! The edge set of the corpus records every edge as authored (FR-026, FR-041).
//! A relation does not match them as authored. It reads each one in the
//! orientation the relation lists:
//!
//! * an edge authored with the counterpart label of a listed verb is read from
//!   the opposite end, through the registry's inverse index (CR-416);
//! * a dangling edge whose target is an acceptance-criterion id minted by one
//!   document of the target's member is read as an edge to that document, with
//!   the criterion id kept on the reading (CR-417).
//!
//! Nothing here is engine knowledge: the verbs come from the relation, the
//! counterparts from the inverse index, and the criterion ids from the trace
//! targets an `obligations:` source names. The edge set, its resolution and
//! its `Dangling` classification are not touched.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::declared_tables::{mint, ExcludeSet, ScanContext};
use super::resolve::{Edge, Resolution};
use super::spec::Spec;
use crate::registry::Registry;
use crate::traceability::{RelationDirection, RequiredRelation, TraceTarget};

/// Who minted a criterion id within one member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Owner {
    /// Exactly one document of the member.
    One(usize),
    /// Two or more documents: the id names no single document.
    Many,
}

/// The acceptance-criterion ids the loaded modules mint, per corpus member.
///
/// A criterion id is an id minted by a trace target an `obligations:` source
/// names (the source's `target:`). With no such source the index is empty and
/// every edge to such an id stays dangling.
#[derive(Debug, Default)]
pub(crate) struct CriterionIndex {
    by_member: Vec<BTreeMap<String, Owner>>,
}

impl CriterionIndex {
    /// Mint the criterion ids of every member of `spec`. `roots` holds each
    /// member's repository root in member order; a member without one reads
    /// the first.
    pub(crate) fn build(spec: &Spec, registry: &Registry, roots: &[&Path]) -> Self {
        let Some(model) = registry.traceability() else {
            return Self::default();
        };
        let targets: Vec<&TraceTarget> = model
            .obligations
            .iter()
            .filter_map(|source| source.target.as_deref())
            .filter_map(|name| model.target(name))
            .collect();
        // Only a dangling edge is ever read through a criterion id.
        let any_dangling = spec
            .corpus_edges()
            .iter()
            .any(|edge| edge.resolution == Resolution::Dangling);
        if targets.is_empty() || !any_dangling {
            return Self::default();
        }
        let model_exclude = ExcludeSet::compile_validated(&model.exclude);
        let mut by_member = Vec::with_capacity(spec.member_count());
        for member in 0..spec.member_count() {
            let mut ids: BTreeMap<String, Owner> = BTreeMap::new();
            if let Some(root) = roots.get(member).or_else(|| roots.first()) {
                let view = spec.member_view(member);
                let mut slots: BTreeMap<&Path, usize> = BTreeMap::new();
                for slot in spec.member_slots(member) {
                    slots.entry(spec.doc(slot).path.as_path()).or_insert(slot);
                }
                let mut ctx = ScanContext::default();
                for target in &targets {
                    for minted in mint(&view, root, target, &model_exclude, &mut ctx) {
                        let Some(&slot) = slots.get(minted.path.as_path()) else {
                            continue;
                        };
                        ids.entry(minted.id)
                            .and_modify(|owner| {
                                if *owner != Owner::One(slot) {
                                    *owner = Owner::Many;
                                }
                            })
                            .or_insert(Owner::One(slot));
                    }
                }
            }
            by_member.push(ids);
        }
        Self { by_member }
    }

    /// The document that minted `id`, when `reference` (authored in a document
    /// of `source_member`) is looked up in a member where exactly one document
    /// minted it.
    fn owner(&self, spec: &Spec, source_member: usize, reference: &str, id: &str) -> Option<usize> {
        if self.by_member.is_empty() {
            return None;
        }
        let member = spec.reference_member(source_member, reference)?;
        match self.by_member.get(member)?.get(id)? {
            Owner::One(slot) => Some(*slot),
            Owner::Many => None,
        }
    }

    /// Whether every spelling of the dangling `edge` reads as an edge to a
    /// criterion's document, so it is not a dangling reference (FR-038 CR-418).
    pub(crate) fn reads_every_spelling(&self, spec: &Spec, edge: &Edge) -> bool {
        edge.resolution == Resolution::Dangling
            && spellings(edge).all(|reference| {
                self.owner(
                    spec,
                    spec.member_of(edge.source_slot),
                    reference,
                    &edge.target,
                )
                .is_some()
            })
    }
}

/// The spellings of an edge's target as authored.
fn spellings(edge: &Edge) -> impl Iterator<Item = &str> {
    let authored: &[String] = &edge.references;
    let fallback = authored.is_empty().then_some(edge.target.as_str());
    authored.iter().map(String::as_str).chain(fallback)
}

/// One end of an authored edge, as a relation reads it.
enum End {
    /// A document, with the criterion ids the edge named to reach it.
    Doc(usize, BTreeSet<String>),
    /// Nothing loaded, with the target as authored.
    Absent(String),
}

/// The ends an authored edge reaches.
fn ends_of(spec: &Spec, criteria: &CriterionIndex, edge: &Edge) -> Vec<End> {
    if let Some(target) = edge.target_slot {
        return vec![End::Doc(target, BTreeSet::new())];
    }
    let source_member = spec.member_of(edge.source_slot);
    let mut owned: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    let mut ends = Vec::new();
    for reference in spellings(edge) {
        match criteria.owner(spec, source_member, reference, &edge.target) {
            Some(owner) => {
                owned.entry(owner).or_default().insert(edge.target.clone());
            }
            None => ends.push(End::Absent(reference.to_string())),
        }
    }
    ends.extend(owned.into_iter().map(|(slot, ids)| End::Doc(slot, ids)));
    ends
}

/// One edge as a relation reads it: `source` has `verb` toward `target`.
#[derive(Debug, Clone)]
pub(crate) struct Reading {
    /// The source end; `None` when an inverse-authored edge names an absent
    /// document.
    pub(crate) source: Option<usize>,
    /// The target end; `None` when the edge names an absent document.
    pub(crate) target: Option<usize>,
    /// The verb the relation lists, not the one authored.
    pub(crate) verb: String,
    /// The criterion ids the authored edges named to reach the document.
    pub(crate) criteria: BTreeSet<String>,
    /// The absent end as authored.
    pub(crate) reference: Option<String>,
    /// The target as a finding names it.
    pub(crate) authored: String,
}

/// A reading seen from one obliged document.
pub(crate) struct Match<'a> {
    pub(crate) reading: &'a Reading,
    /// The document at the other end; `None` when it is absent.
    pub(crate) counterpart: Option<usize>,
    /// Whether the obliged document is the reading's source.
    pub(crate) outgoing: bool,
}

/// The counterpart label of `verb` in the inverse index: the forward verb when
/// `verb` is an inverse label, else the label that inverse index maps to
/// `verb`. A verb the index relates to nothing has none.
fn counterpart<'a>(inverse: &'a BTreeMap<String, String>, verb: &'a str) -> Option<&'a str> {
    if let Some(forward) = inverse.get(verb) {
        return Some(forward.as_str());
    }
    inverse
        .iter()
        .find(|(_, forward)| forward.as_str() == verb)
        .map(|(label, _)| label.as_str())
}

/// Which way a relation reads an authored edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    /// Source to target, as authored.
    AsAuthored,
    /// From the opposite end: the authored label is the counterpart of the
    /// listed verb.
    Reversed,
}

/// Every edge a relation matches, read in the orientation it lists.
#[derive(Debug)]
pub(crate) struct RelationEdges {
    readings: Vec<Reading>,
    by_source: BTreeMap<usize, Vec<usize>>,
    by_target: BTreeMap<usize, Vec<usize>>,
}

impl RelationEdges {
    pub(crate) fn new(
        spec: &Spec,
        registry: &Registry,
        criteria: &CriterionIndex,
        relation: &RequiredRelation,
    ) -> Self {
        let inverse = registry.inverse_index();
        // A pair listed by both labels is read one way only, as its forward
        // verb: reading it both ways would count one edge between one pair
        // twice and accept edges listing only one label does not.
        let listed: Vec<&String> = relation
            .edges
            .iter()
            .filter(|verb| {
                inverse
                    .get(verb.as_str())
                    .is_none_or(|forward| !relation.edges.contains(forward))
            })
            .collect();
        // Authored label -> (listed verb, orientation).
        let mut forms: BTreeMap<&str, Vec<(&str, Orientation)>> = BTreeMap::new();
        for verb in listed {
            forms
                .entry(verb)
                .or_default()
                .push((verb, Orientation::AsAuthored));
            if let Some(label) = counterpart(inverse, verb) {
                forms
                    .entry(label)
                    .or_default()
                    .push((verb, Orientation::Reversed));
            }
        }

        type Key = (Option<usize>, Option<usize>, String, Option<String>);
        let mut merged: BTreeMap<Key, Reading> = BTreeMap::new();
        for edge in spec.corpus_edges() {
            let Some(forms) = forms.get(edge.edge_type.as_str()) else {
                continue;
            };
            for end in ends_of(spec, criteria, edge) {
                for &(verb, orientation) in forms {
                    let (source, target, ids, reference, authored) = match (&end, orientation) {
                        (End::Doc(slot, ids), Orientation::AsAuthored) => (
                            Some(edge.source_slot),
                            Some(*slot),
                            ids.clone(),
                            None,
                            edge.target.clone(),
                        ),
                        (End::Doc(slot, ids), Orientation::Reversed) => (
                            Some(*slot),
                            Some(edge.source_slot),
                            ids.clone(),
                            None,
                            spec.doc(edge.source_slot).id.clone(),
                        ),
                        (End::Absent(reference), Orientation::AsAuthored) => (
                            Some(edge.source_slot),
                            None,
                            BTreeSet::new(),
                            Some(reference.clone()),
                            edge.target.clone(),
                        ),
                        (End::Absent(reference), Orientation::Reversed) => (
                            None,
                            Some(edge.source_slot),
                            BTreeSet::new(),
                            Some(reference.clone()),
                            edge.target.clone(),
                        ),
                    };
                    merged
                        .entry((source, target, verb.to_string(), reference.clone()))
                        .and_modify(|reading| reading.criteria.extend(ids.iter().cloned()))
                        .or_insert(Reading {
                            source,
                            target,
                            verb: verb.to_string(),
                            criteria: ids,
                            reference,
                            authored,
                        });
                }
            }
        }

        let readings: Vec<Reading> = merged.into_values().collect();
        let mut by_source: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        let mut by_target: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (index, reading) in readings.iter().enumerate() {
            if let Some(source) = reading.source {
                by_source.entry(source).or_default().push(index);
            }
            if let Some(target) = reading.target {
                by_target.entry(target).or_default().push(index);
            }
        }
        Self {
            readings,
            by_source,
            by_target,
        }
    }

    /// The readings the relation matches at `slot` in `direction`.
    pub(crate) fn matches(&self, direction: RelationDirection, slot: usize) -> Vec<Match<'_>> {
        let indices = |index: &BTreeMap<usize, Vec<usize>>| -> Vec<usize> {
            index.get(&slot).cloned().unwrap_or_default()
        };
        let mut out = Vec::new();
        if direction != RelationDirection::Incoming {
            out.extend(indices(&self.by_source).into_iter().map(|i| {
                let reading = &self.readings[i];
                Match {
                    reading,
                    counterpart: reading.target,
                    outgoing: true,
                }
            }));
        }
        if direction != RelationDirection::Outgoing {
            out.extend(indices(&self.by_target).into_iter().map(|i| {
                let reading = &self.readings[i];
                Match {
                    reading,
                    counterpart: reading.source,
                    outgoing: false,
                }
            }));
        }
        out
    }
}

/// Everything a relation check reads beyond the corpus, built once per corpus
/// rather than per relation per member view.
#[derive(Debug, Default)]
pub(crate) struct RelationContext {
    criteria: CriterionIndex,
    edges: BTreeMap<String, RelationEdges>,
}

impl RelationContext {
    /// Read every declared relation's edges. `roots` is each member's
    /// repository root, in member order.
    pub(crate) fn build(spec: &Spec, registry: &Registry, roots: &[&Path]) -> Self {
        let criteria = CriterionIndex::build(spec, registry, roots);
        let edges = registry
            .traceability()
            .map(|model| {
                model
                    .required_relations
                    .iter()
                    .map(|relation| {
                        (
                            relation.name.clone(),
                            RelationEdges::new(spec, registry, &criteria, relation),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self { criteria, edges }
    }

    /// Whether every spelling of the dangling `edge` reads as an edge to a
    /// criterion's document (FR-038 CR-418).
    pub(crate) fn reads_every_spelling(&self, spec: &Spec, edge: &Edge) -> bool {
        self.criteria.reads_every_spelling(spec, edge)
    }

    /// The edges `relation` matches.
    pub(crate) fn edges_for(&self, relation: &RequiredRelation) -> Option<&RelationEdges> {
        self.edges.get(&relation.name)
    }
}
