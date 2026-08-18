//! Upward-trace completeness (FR-058).
//!
//! Downward coverage — FR → AC → TC → code — is what FR-049/FR-050 build, and
//! it answers "is what we wrote verified". It cannot answer "is anything
//! *missing*", because nothing operating over the existing spec text can:
//! a requirement nobody wrote leaves no trace to follow.
//!
//! Upward tracing is the analysis class that can. An FR with no upstream need
//! is a feature nobody asked for; a stakeholder requirement with no downstream
//! FR is a need nobody built. Both are invisible to every check that starts
//! from a document and walks down.
//!
//! Nothing here is engine knowledge — no archetype name, no verb, no direction
//! is written in this file. Each is read from the active module's declared
//! [`RequiredRelation`]s, exactly as `trace_refs` reads its patterns and columns
//! from `document_references`. That is what lets a security module state "every
//! hazard must be mitigated by something" as manifest data rather than as a
//! second engine check (agent-ix/spec-objects-security#5).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::declared_tables::ExcludeSet;
use super::resolve::Resolution;
use super::spec::Spec;
use super::validate::{pack, posture_tier, BundleFinding, BundlePosture, BundleReport};
use crate::registry::Registry;
use crate::traceability::{RelationDirection, RequiredRelation};

/// Check every declared required relation, plus any declared acyclic edge sets.
///
/// A no-op when the active module declares neither (FR-058-AC-8), which is the
/// same early return `validate_trace_references` makes — a module that has not
/// adopted the declaration must see byte-identical output.
pub(crate) fn validate_required_relations(
    spec: &Spec,
    registry: &Registry,
    posture: BundlePosture,
    root: &Path,
    report: &mut BundleReport,
) {
    let Some(model) = registry.traceability() else {
        return;
    };
    if model.required_relations.is_empty() && model.acyclic_edges.is_empty() {
        return;
    }

    let severity = registry.grammar_severity();
    let model_exclude = ExcludeSet::compile(&model.exclude);

    // Collected before pushing so order is a property of the bundle rather than
    // of the walk (the FR-049-AC-8 discipline).
    let mut findings: Vec<(PathBuf, String, String, String)> = Vec::new();

    for relation in &model.required_relations {
        check_relation(spec, relation, root, &model_exclude, &mut findings);
    }
    for verb in &model.acyclic_edges {
        check_acyclic(spec, verb, &mut findings);
    }

    findings.sort();
    findings.dedup();
    for (path, _key, check, message) in findings {
        report.route(
            severity,
            posture_tier(posture),
            BundleFinding::in_pack(pack::TRACE, check, path, message),
        );
    }
}

/// Every document of `relation.from` that lacks the declared edge.
fn check_relation(
    spec: &Spec,
    relation: &RequiredRelation,
    root: &Path,
    model_exclude: &ExcludeSet,
    out: &mut Vec<(PathBuf, String, String, String)>,
) {
    let exclude = ExcludeSet::compile(&relation.exclude);
    // Owned, not leaked. The check token is the `<check>` half of a
    // `trace:<check>` severity key and comes from the manifest, so it cannot be
    // a `&'static str` without leaking one per call — and a corpus sweep calls
    // `validate_bundle` once per repository. `BundleFinding::reason` is a
    // `Cow<'static, str>` for exactly this: literals stay borrowed and cost
    // nothing, a declared token is owned.
    let check = relation.check.clone();

    let accepted_targets: BTreeSet<&str> = relation.to.iter().map(String::as_str).collect();
    let accepted_edges: BTreeSet<&str> = relation.edges.iter().map(String::as_str).collect();

    for doc in spec.by_type(&relation.from) {
        if exclude.excludes(root, &doc.path) || model_exclude.excludes(root, &doc.path) {
            continue;
        }
        if doc.id.is_empty() {
            continue; // nothing to key an edge on
        }

        let satisfied = match relation.direction {
            RelationDirection::Outgoing => spec.outgoing(&doc.id).iter().any(|e| {
                accepted_edges.contains(e.edge_type.as_str())
                    && target_is_accepted(spec, &e.target, e.resolution, &accepted_targets)
            }),
            RelationDirection::Incoming => spec.referencing(&doc.id).iter().any(|e| {
                accepted_edges.contains(e.edge_type.as_str())
                    && target_is_accepted(spec, &e.source, Resolution::Resolved, &accepted_targets)
            }),
        };
        if satisfied {
            continue;
        }

        let verbs = relation.edges.join("/");
        let kinds = if relation.to.is_empty() {
            "any document".to_string()
        } else {
            relation.to.join("/")
        };
        let message = match relation.direction {
            RelationDirection::Outgoing => format!(
                "'{}' has no '{verbs}' edge to any {kinds} — declared by '{}'",
                doc.id, relation.name
            ),
            RelationDirection::Incoming => format!(
                "nothing reaches '{}' by '{verbs}' from any {kinds} — declared by '{}'",
                doc.id, relation.name
            ),
        };
        out.push((doc.path.clone(), doc.id.clone(), check.clone(), message));
    }
}

/// Whether the document at the other end of an edge is an accepted kind.
///
/// A **dangling** edge cannot satisfy a relation whose targets are constrained:
/// its target is not in the bundle, so nothing can say what archetype it is, and
/// accepting it would let a typo satisfy the requirement it broke. With no
/// declared targets the verb alone is the contract, and a cross-repo edge counts
/// — which is the reading a module gets by declaring `to: []` deliberately.
fn target_is_accepted(
    spec: &Spec,
    id: &str,
    resolution: Resolution,
    accepted: &BTreeSet<&str>,
) -> bool {
    if accepted.is_empty() {
        return true;
    }
    if resolution == Resolution::Dangling {
        return false;
    }
    spec.by_id(id)
        .and_then(|d| d.concept_type())
        .is_some_and(|ty| accepted.contains(ty))
}

/// Report every cycle reachable over one declared verb.
///
/// A requirement that transitively refines itself states nothing, and no
/// per-document check can see it — the defect is a property of the graph. Each
/// cycle is reported **once**, keyed on its lexicographically smallest member,
/// so a three-node cycle is one finding rather than three.
fn check_acyclic(spec: &Spec, verb: &str, out: &mut Vec<(PathBuf, String, String, String)>) {
    let check = format!("cyclic-{verb}");

    // Adjacency over the declared verb only, resolved edges only — a dangling
    // edge leaves the bundle and cannot close a cycle inside it.
    let mut adjacency: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for edge in spec.edges() {
        if edge.edge_type == verb && edge.resolution == Resolution::Resolved {
            adjacency
                .entry(edge.source.as_str())
                .or_default()
                .insert(edge.target.as_str());
        }
    }

    let mut reported: BTreeSet<String> = BTreeSet::new();
    for start in adjacency.keys().copied() {
        let Some(cycle) = shortest_cycle_through(start, &adjacency) else {
            continue;
        };
        // Key on the smallest member so every rotation of one cycle collapses.
        let Some(key) = cycle.iter().min() else {
            continue;
        };
        if !reported.insert((*key).to_string()) {
            continue;
        }
        let Some(doc) = spec.by_id(key) else { continue };
        let mut path = cycle.clone();
        path.push(cycle[0]);
        out.push((
            doc.path.clone(),
            (*key).to_string(),
            check.clone(),
            format!(
                "'{verb}' edges form a cycle: {} — a requirement that transitively \
                 {verb}s itself states nothing",
                path.join(" → ")
            ),
        ));
    }
}

/// The shortest cycle containing `start`, as ids in traversal order, or `None`.
///
/// Breadth-first from `start` back to `start`, so the reported path is the
/// smallest one a reader has to check rather than whichever a depth-first walk
/// happened to find. Bounded by the node count, so a dense graph cannot spin.
fn shortest_cycle_through<'a>(
    start: &'a str,
    adjacency: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> Option<Vec<&'a str>> {
    let mut previous: BTreeMap<&str, &str> = BTreeMap::new();
    let mut queue: std::collections::VecDeque<&str> = std::collections::VecDeque::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    queue.push_back(start);
    seen.insert(start);

    while let Some(node) = queue.pop_front() {
        for next in adjacency.get(node).into_iter().flatten().copied() {
            if next == start {
                // Walk `previous` back from `node` to `start`.
                let mut path = vec![node];
                let mut cursor = node;
                while let Some(prev) = previous.get(cursor).copied() {
                    path.push(prev);
                    cursor = prev;
                }
                path.reverse();
                return Some(path);
            }
            if seen.insert(next) {
                previous.insert(next, node);
                queue.push_back(next);
            }
        }
    }
    None
}
