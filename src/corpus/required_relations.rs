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
use super::relation_edges::{Match, RelationContext, RelationEdges};
use super::spec::Spec;
use super::validate::{pack, BundleFinding, BundleReport};
use super::vocabulary_coverage::{declares_property, enum_at};
use crate::grammar::GrammarSeverity;
use crate::registry::Registry;
use crate::traceability::{
    fold_selector_value, RelationDirection, RequiredRelation, WhereSelector,
};

/// The `trace:` check token of a resolved edge whose target kind is outside a
/// relation's `to` (FR-085).
const KIND_MISMATCH: &str = "target-kind-mismatch";

/// Implements: FR-058
/// Check every declared required relation, plus any declared acyclic edge sets.
///
/// Findings are advisory under **both** postures (FR-058-AC-7, AC-18): the
/// unconfigured tier is a warning whatever the posture, and only the FR-057
/// registry promotes one.
///
/// A no-op when the active module declares neither (FR-058-AC-8), which is the
/// same early return `validate_trace_references` makes — a module that has not
/// adopted the declaration must see byte-identical output.
pub(crate) fn validate_required_relations(
    spec: &Spec,
    registry: &Registry,
    root: &Path,
    context: &RelationContext,
    report: &mut BundleReport,
) {
    let Some(model) = registry.traceability() else {
        return;
    };
    if model.required_relations.is_empty()
        && model.acyclic_edges.is_empty()
        && model.design_relations.is_empty()
    {
        return;
    }

    let severity = registry.grammar_severity();
    let model_exclude = ExcludeSet::compile_validated(&model.exclude);

    // Collected before pushing so order is a property of the bundle rather than
    // of the walk (the FR-049-AC-8 discipline).
    let mut findings: Vec<(PathBuf, String, String, String)> = Vec::new();

    for relation in &model.required_relations {
        check_declaration_is_live(spec, registry, relation, root, &mut findings);
        if let Some(edges) = context.edges_for(relation) {
            check_relation(spec, edges, relation, root, &model_exclude, &mut findings);
        }
    }
    // FR-086-AC-10: a design relation no loaded module declares names nothing
    // for the join to report.
    for entry in &model.design_relations {
        if !model.required_relations.iter().any(|r| &r.name == entry) {
            findings.push((
                root.to_path_buf(),
                String::new(),
                "undeclared-relation-vocabulary".to_string(),
                format!(
                    "design_relations entry '{entry}' names no required relation declared by any \
                     loaded module, so the design-element join cannot report it"
                ),
            ));
        }
    }
    for verb in &model.acyclic_edges {
        check_acyclic(spec, verb, &mut findings);
    }

    findings.sort();
    findings.dedup();
    for (path, _key, check, message) in findings {
        report.route(
            severity,
            GrammarSeverity::Warning,
            BundleFinding::in_pack(pack::TRACE, check, path, message),
        );
    }
}

/// Report a relation naming a document kind that is **dead**: no loaded module
/// declares it and no document in the bundle is one.
///
/// A typo in `from` selects **zero** documents, so the relation silently checks
/// nothing. Measured: changing `from: FR` to `from: FRR` in the fixture leaves a
/// genuine orphan requirement unreported and the whole run clean — a
/// one-character slip disables the check with no diagnostic anywhere, which from
/// the outside is indistinguishable from a bundle with nothing wrong.
///
/// [`TraceabilityModel::validate`] cannot catch it: it runs per module at
/// manifest-parse time, before the merge, and a relation legitimately names
/// kinds another module contributes (`from: hazard` in `spec-objects-safety`
/// pointing `to: [FR]` from `spec-artifacts-iso`). Here the merged registry and
/// the walked bundle are both available, which is what makes the question
/// answerable at all.
///
/// **Deliberately not checked: `edges`.** A verb is allowed to be absent from
/// `edge_types` — FR-041-AC-2 says so for inverse labels, and a single-module
/// fixture declares no vocabulary of its own while still using `satisfies`
/// correctly. Flagging verbs fired on the fixture's own working declaration,
/// which is a bad rule rather than a bad fixture. A misspelt verb also fails
/// *loudly* — nothing satisfies the relation, so every `from` document is
/// reported — where a misspelt `from` fails silently, and silence is the case
/// that needs a check.
///
/// [`TraceabilityModel::validate`]: crate::traceability::TraceabilityModel::validate
fn check_declaration_is_live(
    spec: &Spec,
    registry: &Registry,
    relation: &RequiredRelation,
    root: &Path,
    out: &mut Vec<(PathBuf, String, String, String)>,
) {
    // A kind is live if a module declares it OR a document in the bundle is
    // one. Either alone is enough: a declared kind with no documents yet is a
    // contract waiting for content, and an undeclared kind that documents do
    // use is a separate defect other checks already report.
    let is_live =
        |kind: &str| registry.archetype_names().any(|n| n == kind) || spec.corpus_has_type(kind);

    let mut dead: Vec<String> = Vec::new();
    if !is_live(&relation.from) {
        dead.push(format!("from '{}'", relation.from));
    }
    // `to: []` means "any document" and is not a defect (FR-058-AC-1).
    for target in &relation.to {
        if !is_live(target) {
            dead.push(format!("to '{target}'"));
        }
    }
    if !dead.is_empty() {
        out.push((
            root.to_path_buf(),
            String::new(),
            "undeclared-relation-vocabulary".to_string(),
            format!(
                "required relation '{}' names {} — no loaded module declares that kind and no \
                 document in this bundle is one, so the relation matches nothing",
                relation.name,
                dead.join(", ")
            ),
        ));
    }
    if let Some(selector) = &relation.selector {
        check_selector_is_live(spec, registry, relation, selector, root, out);
    }
}

/// Report a `where` that selects nothing (FR-058-AC-22): a `field` that is
/// neither a property of the `from` archetype's frontmatter schema nor carried
/// by any `from` document, or a `values` entry outside the field's schema
/// `enum` and held by no `from` document. Both comparisons trim and ASCII fold.
/// A field whose schema declares no `enum` leaves `values` unchecked.
fn check_selector_is_live(
    spec: &Spec,
    registry: &Registry,
    relation: &RequiredRelation,
    selector: &WhereSelector,
    root: &Path,
    out: &mut Vec<(PathBuf, String, String, String)>,
) {
    let field = selector.field();
    let docs = spec.corpus_by_type(&relation.from);
    let schema = registry
        .archetype(&relation.from)
        .and_then(|a| a.frontmatter_schema.as_ref());
    let carried = docs
        .iter()
        .any(|d| d.frontmatter().is_some_and(|fm| fm.contains_key(field)));
    let mut push = |message: String| {
        out.push((
            root.to_path_buf(),
            String::new(),
            "undeclared-relation-vocabulary".to_string(),
            message,
        ));
    };

    if !carried && !schema.is_some_and(|s| declares_property(s, field)) {
        push(format!(
            "required relation '{}' selects on `where` field '{field}', which is not a property \
             of '{}' frontmatter and no '{}' document carries it, so the relation matches nothing",
            relation.name, relation.from, relation.from
        ));
        return;
    }

    let Some(members) = schema
        .and_then(|s| enum_at(s, field))
        .filter(|m| !m.is_empty())
    else {
        return;
    };
    let members: BTreeSet<String> = members.iter().map(|m| fold_selector_value(m)).collect();
    let held: BTreeSet<String> = docs
        .iter()
        .filter_map(|d| {
            d.frontmatter()?
                .get(field)?
                .as_str()
                .map(fold_selector_value)
        })
        .collect();
    for entry in selector.values() {
        let folded = fold_selector_value(entry);
        if !members.contains(&folded) && !held.contains(&folded) {
            push(format!(
                "required relation '{}' selects on `where` value '{entry}' for field '{field}', \
                 which is outside the field's declared enum and held by no '{}' document, so \
                 that value selects nothing",
                relation.name, relation.from
            ));
        }
    }
}

/// Every document of `relation.from` that lacks the declared edge.
fn check_relation(
    spec: &Spec,
    edges: &RelationEdges,
    relation: &RequiredRelation,
    root: &Path,
    model_exclude: &ExcludeSet,
    out: &mut Vec<(PathBuf, String, String, String)>,
) {
    // Owned, not leaked. The check token is the `<check>` half of a
    // `trace:<check>` severity key and comes from the manifest, so it cannot be
    // a `&'static str` without leaking one per call — and a corpus sweep calls
    // `validate_bundle` once per repository. `BundleFinding::reason` is a
    // `Cow<'static, str>` for exactly this: literals stay borrowed and cost
    // nothing, a declared token is owned.
    let check = relation.check.clone();

    let accepted_targets: BTreeSet<&str> = relation.to.iter().map(String::as_str).collect();

    for Obliged { slot, value } in obliged(spec, relation, root, model_exclude) {
        let doc = spec.doc(slot);
        // The authored value is kept for the finding text (AC-20).
        let selected_value = value.and_then(serde_json::Value::as_str);
        let seen = edges.matches(relation.direction, slot);
        // A listed-verb edge from this document to a resolved target of a kind
        // outside a non-empty `to` never satisfies the relation, and is
        // reported whether or not the relation is otherwise satisfied. Only the
        // obliged document's own outgoing reading names a mismatch: under
        // `incoming` a wrong-kind source simply leaves the relation
        // unsatisfied (FR-085-AC-25).
        if !accepted_targets.is_empty() && relation.direction != RelationDirection::Incoming {
            for found in seen.iter().filter(|m| m.outgoing) {
                let Some(target) = found.counterpart else {
                    continue;
                };
                let actual = spec.doc(target).concept_type().unwrap_or("");
                if accepted_targets.contains(actual) {
                    continue;
                }
                let reading = found.reading;
                // Qualified by the member only when the target lives in another.
                let target_repo = spec.member_identity(spec.member_of(target));
                let reference = match target_repo {
                    Some(repo) if target_repo != spec.member_identity(spec.member_of(slot)) => {
                        format!("{repo}:{}", reading.authored)
                    }
                    _ => reading.authored.clone(),
                };
                out.push((
                    doc.path.clone(),
                    doc.id.clone(),
                    KIND_MISMATCH.to_string(),
                    format!(
                        "'{}' has a '{}' edge to '{reference}' of archetype '{}', outside the \
                         accepted {} — declared by '{}'; the edge does not satisfy the relation",
                        doc.id,
                        reading.verb,
                        if actual.is_empty() {
                            "(untyped)"
                        } else {
                            actual
                        },
                        relation.to.join("/"),
                        relation.name
                    ),
                ));
            }
        }
        let satisfied = seen_satisfies(spec, relation, &seen);
        if satisfied {
            continue;
        }

        let verbs = relation.edges.join("/");
        // Renders the whole noun phrase, article included. The templates below
        // must NOT prepend "any": with `to: []` this is already "any document",
        // and a hardcoded article produced "from any any document" in the first
        // end-to-end run against `spec-objects-safety`.
        let kinds = if relation.to.is_empty() {
            "any document".to_string()
        } else {
            format!("any {}", relation.to.join("/"))
        };
        let mut message = match relation.direction {
            RelationDirection::Outgoing => format!(
                "'{}' has no '{verbs}' edge to {kinds} — declared by '{}'",
                doc.id, relation.name
            ),
            RelationDirection::Incoming => format!(
                "nothing reaches '{}' by '{verbs}' from {kinds} — declared by '{}'",
                doc.id, relation.name
            ),
            RelationDirection::Either => format!(
                "'{}' has no '{verbs}' edge to or from {kinds} — declared by '{}'",
                doc.id, relation.name
            ),
        };
        // A dangling reference never satisfies a relation whose `to` is
        // constrained; name each one as authored so the author can see which
        // target is missing (FR-085).
        let dangling: BTreeSet<&str> = seen
            .iter()
            .filter(|m| m.counterpart.is_none())
            .filter_map(|m| m.reading.reference.as_deref())
            .collect();
        if !dangling.is_empty() {
            let named: Vec<String> = dangling.iter().map(|r| format!("'{r}'")).collect();
            message.push_str(&format!(
                ", dangling reference {} not in the corpus",
                named.join(", ")
            ));
        }
        if let (Some(sel), Some(value)) = (relation.selector.as_ref(), selected_value) {
            message.push_str(&format!(
                ", selected because `{}` is '{value}'",
                sel.field()
            ));
        }
        out.push((doc.path.clone(), doc.id.clone(), check.clone(), message));
    }
}

/// A document a relation obliges, with the `where` value that selected it.
pub(crate) struct Obliged<'a> {
    /// Corpus slot of the obliged document.
    pub(crate) slot: usize,
    /// The selector field's value as authored; `None` when the relation has no
    /// `where`.
    pub(crate) value: Option<&'a serde_json::Value>,
}

/// The documents `relation` obliges in the view `spec` sees: the `from` kind,
/// then `where`, then `exclude`, in id order. Shared by the finding
/// check and the relation report so the two select the same population.
pub(crate) fn obliged<'a>(
    spec: &'a Spec,
    relation: &RequiredRelation,
    root: &Path,
    model_exclude: &ExcludeSet,
) -> Vec<Obliged<'a>> {
    let exclude = ExcludeSet::compile_validated(&relation.exclude);
    let selector = relation.selector.as_ref();
    let mut out = Vec::new();
    for slot in spec.by_type_slots(&relation.from) {
        let doc = spec.doc(slot);
        let value = match selector {
            Some(sel) => match doc.frontmatter().and_then(|fm| fm.get(sel.field())) {
                Some(value) if sel.selects(value) => Some(value),
                _ => continue,
            },
            None => None,
        };
        if exclude.excludes(root, &doc.path) || model_exclude.excludes(root, &doc.path) {
            continue;
        }
        if doc.id.is_empty() {
            continue; // nothing to key an edge on
        }
        out.push(Obliged { slot, value });
    }
    out
}

/// Whether the document at `slot` satisfies `relation`. The one
/// definition of satisfaction: the finding and the report `state` both read it.
pub(crate) fn is_satisfied(
    spec: &Spec,
    edges: &RelationEdges,
    relation: &RequiredRelation,
    slot: usize,
) -> bool {
    seen_satisfies(spec, relation, &edges.matches(relation.direction, slot))
}

/// Whether any of the readings seen at a document is an accepted counterpart.
fn seen_satisfies(spec: &Spec, relation: &RequiredRelation, seen: &[Match<'_>]) -> bool {
    let accepted_targets: BTreeSet<&str> = relation.to.iter().map(String::as_str).collect();
    seen.iter()
        .any(|m| target_is_accepted(spec, m.counterpart, &accepted_targets))
}

/// Whether the document at the other end of an edge is an accepted kind.
///
/// A **dangling** edge (`slot` is `None`) cannot satisfy a relation whose
/// targets are constrained: its target is not in the corpus, so nothing can say
/// what archetype it is, and accepting it would let a typo satisfy the
/// requirement it broke. With no declared targets the verb alone is the
/// contract, and a cross-repo edge counts — which is the reading a module gets
/// by declaring `to: []` deliberately.
fn target_is_accepted(spec: &Spec, slot: Option<usize>, accepted: &BTreeSet<&str>) -> bool {
    if accepted.is_empty() {
        return true;
    }
    slot.and_then(|slot| spec.doc(slot).concept_type())
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
    // edge leaves the corpus and cannot close a cycle inside it. Nodes are
    // labelled by id, qualified by repository once a corpus has several
    // members, and the cycle search reads the corpus-wide edge set so a cycle
    // through two repositories is found.
    let label = |slot: usize| -> String {
        let id = &spec.doc(slot).id;
        match spec.member_identity(spec.member_of(slot)) {
            Some(repo) if spec.member_count() > 1 => format!("{repo}:{id}"),
            _ => id.clone(),
        }
    };
    let mut slots: BTreeMap<String, usize> = BTreeMap::new();
    let mut pairs: Vec<(String, String)> = Vec::new();
    for edge in spec.corpus_edges() {
        let (Some(target), true) = (edge.target_slot, edge.edge_type == verb) else {
            continue;
        };
        let (from, to) = (label(edge.source_slot), label(target));
        slots.insert(from.clone(), edge.source_slot);
        slots.insert(to.clone(), target);
        pairs.push((from, to));
    }
    let mut adjacency: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (from, to) in &pairs {
        adjacency.entry(from).or_default().insert(to);
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
        let Some(&slot) = slots.get(*key) else {
            continue;
        };
        // Every member view sees the whole cycle; the one holding its key
        // reports it.
        if spec
            .scope
            .is_some_and(|member| spec.member_of(slot) != member)
        {
            continue;
        }
        let doc = spec.doc(slot);
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
