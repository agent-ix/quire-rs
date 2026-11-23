//! Resolution of traceability declarations across the selected modules.
//!
//! A declaration that cannot load drops itself. Nothing here empties the
//! merged model: each conflict or unresolved reference removes the declarations
//! involved from their modules and records one failure per declaration, so
//! every other declaration and every other module still loads.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::error::{ArchetypeLoadFailure, DroppedDeclaration};
use crate::loader::compile::CompiledArchetype;
use crate::loader::{flow_rules, merge_traceability, LoadOutcome, LoadedModule};
use crate::traceability::{DeclarationKind, DroppedDeclarationRef};
use crate::vocab::EdgeTypeDef;

/// One declaration to drop from one module.
pub(super) struct Drop {
    module: usize,
    kind: DeclarationKind,
    item: String,
    reason: String,
}

impl Drop {
    pub(super) fn new(
        module: usize,
        kind: DeclarationKind,
        item: &str,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            module,
            kind,
            item: item.to_string(),
            reason: reason.into(),
        }
    }
}

/// The failure for a declaration drop: it carries the kind and the item, so a
/// consumer never reads the reason text to tell it from another load failure.
pub(super) fn declaration_failure(
    module: &str,
    path: std::path::PathBuf,
    kind: DeclarationKind,
    item: &str,
    reason: String,
) -> ArchetypeLoadFailure {
    ArchetypeLoadFailure {
        module: module.to_string(),
        archetype: "<manifest>".to_string(),
        path,
        reason,
        declaration: Some(Box::new(DroppedDeclaration {
            kind,
            item: item.to_string(),
        })),
    }
}

fn apply(outcome: &mut LoadOutcome, drops: Vec<Drop>) {
    for drop in drops {
        let module = &mut outcome.modules[drop.module];
        module
            .traceability
            .remove_declaration(drop.kind, &drop.item);
        outcome.failures.push(declaration_failure(
            &module.name,
            module.root.join("manifest.yaml"),
            drop.kind,
            &drop.item,
            drop.reason,
        ));
    }
}

/// Names already dropped for `kind`, from every recorded failure.
fn dropped_names(failures: &[ArchetypeLoadFailure], kind: DeclarationKind) -> BTreeSet<String> {
    failures
        .iter()
        .filter_map(|failure| failure.declaration.as_ref())
        .filter(|declaration| declaration.kind == kind)
        .map(|declaration| declaration.item.clone())
        .collect()
}

/// Drop every archetype whose `frontmatter_schema_ref` cannot be resolved; the
/// module's other archetypes load.
pub(super) fn drop_unresolvable_archetypes(outcome: &mut LoadOutcome) {
    for module in &mut outcome.modules {
        let (kept, dropped): (Vec<_>, Vec<_>) = module
            .archetypes
            .iter()
            .cloned()
            .partition(|archetype| archetype.frontmatter_schema_ref_error.is_none());
        module.archetypes = kept;
        for archetype in dropped {
            let error = archetype
                .frontmatter_schema_ref_error
                .as_deref()
                .unwrap_or_default();
            outcome.failures.push(declaration_failure(
                &module.name,
                module.root.join("manifest.yaml"),
                DeclarationKind::Archetype,
                &archetype.name,
                format!(
                    "frontmatter schema '{}' has an unresolved local reference: {error}",
                    archetype
                        .frontmatter_schema_path
                        .as_deref()
                        .unwrap_or("<unknown>")
                ),
            ));
        }
    }
}

/// The modules named in a conflict message other than `except`, in load order.
fn others(modules: &[LoadedModule], declaring: &BTreeSet<usize>, except: usize) -> String {
    declaring
        .iter()
        .filter(|index| **index != except)
        .map(|index| format!("'{}'", modules[*index].name))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Drop each declaration of a conflict from every module declaring it, each
/// failure naming every other module `involved` in the conflict.
fn conflict(
    modules: &[LoadedModule],
    drops: &mut Vec<Drop>,
    kind: DeclarationKind,
    item: &str,
    declaring: &BTreeSet<usize>,
    involved: &BTreeSet<usize>,
    what: &str,
) {
    for index in declaring {
        let other = others(modules, involved, *index);
        let reason = if other.is_empty() {
            format!("merged selected traceability model: {what}")
        } else {
            format!("merged selected traceability model: {what}; also declared by modules {other}")
        };
        drops.push(Drop::new(*index, kind, item, reason));
    }
}

/// Conflicts between modules where no declaration can be chosen: every
/// conflicting declaration is dropped.
fn cross_module_conflicts(modules: &[LoadedModule]) -> Vec<Drop> {
    let mut drops = Vec::new();

    let mut relations: BTreeMap<&str, Vec<(usize, &crate::traceability::RequiredRelation)>> =
        BTreeMap::new();
    let mut carried: BTreeMap<&str, Vec<(usize, &str)>> = BTreeMap::new();
    let mut flow: BTreeMap<&str, BTreeSet<usize>> = BTreeMap::new();
    let mut remediations: BTreeMap<&str, BTreeSet<usize>> = BTreeMap::new();
    for (index, module) in modules.iter().enumerate() {
        let model = &module.traceability;
        for relation in &model.required_relations {
            relations
                .entry(relation.name.as_str())
                .or_default()
                .push((index, relation));
        }
        for field in &model.carried_fields {
            carried
                .entry(field.name.as_str())
                .or_default()
                .push((index, field.key.as_str()));
        }
        for rule in &model.flow_rules {
            flow.entry(rule.name.as_str()).or_default().insert(index);
        }
        for remediation in &model.remediations {
            remediations
                .entry(remediation.name.as_str())
                .or_default()
                .insert(index);
        }
    }

    for (name, entries) in relations {
        if entries
            .iter()
            .any(|(_, relation)| *relation != entries[0].1)
        {
            let declaring = entries.iter().map(|(index, _)| *index).collect();
            conflict(
                modules,
                &mut drops,
                DeclarationKind::RequiredRelation,
                name,
                &declaring,
                &declaring,
                &format!("required_relations entry '{name}' is declared with different bodies"),
            );
        }
    }
    for (name, entries) in carried {
        if entries.iter().any(|(_, key)| *key != entries[0].1) {
            let declaring = entries.iter().map(|(index, _)| *index).collect();
            conflict(
                modules,
                &mut drops,
                DeclarationKind::CarriedField,
                name,
                &declaring,
                &declaring,
                &format!("carried_fields entry '{name}' is declared with different keys"),
            );
        }
    }
    for (kind, label, declared) in [
        (DeclarationKind::FlowRule, "flow_rules", flow),
        (DeclarationKind::Remediation, "remediations", remediations),
    ] {
        for (name, declaring) in declared {
            if declaring.len() > 1 {
                conflict(
                    modules,
                    &mut drops,
                    kind,
                    name,
                    &declaring,
                    &declaring,
                    &format!("duplicate {label} entry '{name}'"),
                );
            }
        }
    }

    qualifier_conflicts(modules, &mut drops);
    retirement_conflicts(modules, &mut drops);
    drops
}

/// NFR-140: prefixes are unique and none contains another, across modules.
/// Identical declarations merge.
fn qualifier_conflicts(modules: &[LoadedModule], drops: &mut Vec<Drop>) {
    let mut declared: BTreeMap<(&str, &str), BTreeSet<usize>> = BTreeMap::new();
    for (index, module) in modules.iter().enumerate() {
        for qualifier in &module.traceability.trace_id_qualifiers {
            declared
                .entry((qualifier.prefix.as_str(), qualifier.repository.as_str()))
                .or_default()
                .insert(index);
        }
    }
    let entries: Vec<(&(&str, &str), &BTreeSet<usize>)> = declared.iter().collect();
    // Per prefix: the modules declaring it, and every module in a conflict with it.
    let mut conflicting: BTreeMap<&str, (BTreeSet<usize>, BTreeSet<usize>)> = BTreeMap::new();
    for (position, ((prefix, _), declaring)) in entries.iter().enumerate() {
        for ((other, _), other_declaring) in &entries[position + 1..] {
            if !(prefix.starts_with(other) || other.starts_with(prefix)) {
                continue;
            }
            for (name, mine, theirs) in [
                (*prefix, declaring, other_declaring),
                (*other, other_declaring, declaring),
            ] {
                let entry = conflicting.entry(name).or_default();
                entry.0.extend(mine.iter().copied());
                entry.1.extend(mine.iter().chain(theirs.iter()).copied());
            }
        }
    }
    for (prefix, (declaring, involved)) in conflicting {
        conflict(
            modules,
            drops,
            DeclarationKind::TraceIdQualifier,
            prefix,
            &declaring,
            &involved,
            &format!(
                "trace_id_qualifiers prefix '{prefix}' is not unique or overlaps another prefix, \
                 so a tag could name either repository"
            ),
        );
    }
}

/// A retired value another module declares as minting no source symbol
/// contradicts it. Every such declaration drops once, naming every other module
/// in the conflict. A contradiction inside one module dropped at its own load.
fn retirement_conflicts(modules: &[LoadedModule], drops: &mut Vec<Drop>) {
    let fold = |value: &str| value.trim().to_ascii_lowercase();
    let mut retiring: BTreeMap<String, BTreeMap<usize, &str>> = BTreeMap::new();
    let mut minting: BTreeMap<String, BTreeMap<usize, &str>> = BTreeMap::new();
    for (index, module) in modules.iter().enumerate() {
        let vocabularies = &module.traceability.vocabularies;
        for value in &vocabularies.retired {
            retiring
                .entry(fold(value))
                .or_default()
                .insert(index, value);
        }
        for value in &vocabularies.no_source_symbol {
            minting.entry(fold(value)).or_default().insert(index, value);
        }
    }
    for (value, retired_by) in &retiring {
        let Some(minted_by) = minting.get(value) else {
            continue;
        };
        let involved: BTreeSet<usize> =
            retired_by.keys().chain(minted_by.keys()).copied().collect();
        let raw = retired_by.values().next().copied().unwrap_or(value);
        let what =
            format!("vocabularies.retired value '{raw}' is also declared in no_source_symbol");
        for (kind, declared) in [
            (DeclarationKind::RetirementVocabulary, retired_by),
            (DeclarationKind::NoSourceSymbol, minted_by),
        ] {
            for (index, item) in declared {
                let other = others(modules, &involved, *index);
                let reason = if other.is_empty() {
                    format!("merged selected traceability model: {what}")
                } else {
                    format!(
                        "merged selected traceability model: {what}; also declared by modules \
                         {other}"
                    )
                };
                drops.push(Drop::new(*index, kind, item, reason));
            }
        }
    }
}

/// Resolve the declarations of `outcome.modules` against the merged registry,
/// dropping each one that cannot load.
pub(super) fn resolve(
    outcome: &mut LoadOutcome,
    archetypes: &BTreeMap<String, Arc<CompiledArchetype>>,
    edge_types: &BTreeMap<String, EdgeTypeDef>,
    inverse_edges: &BTreeMap<String, String>,
) {
    let drops = cross_module_conflicts(&outcome.modules);
    apply(outcome, drops);

    let merged = merge_traceability(&outcome.modules);
    let mut drops = Vec::new();

    // Targets merge first-wins by name, so an `obligations:` source one module
    // validated against its own target can end up bound to another module's
    // target that reads a list, `*`, or no table column.
    for source in &merged.obligations {
        let Some(target) = &source.target else {
            continue;
        };
        let Err(reason) = merged.check_bound_target(&source.name, target) else {
            continue;
        };
        let winner = outcome
            .modules
            .iter()
            .find(|m| {
                m.traceability
                    .trace_targets
                    .iter()
                    .any(|t| &t.name == target)
            })
            .map_or("?", |m| m.name.as_str());
        for (index, module) in outcome.modules.iter().enumerate() {
            if module
                .traceability
                .obligations
                .iter()
                .any(|o| o.name == source.name)
            {
                drops.push(Drop::new(
                    index,
                    DeclarationKind::Obligation,
                    &source.name,
                    format!(
                        "merged selected traceability model: {reason} (target '{target}' was \
                         declared first by module '{winner}')"
                    ),
                ));
            }
        }
    }

    // A `record_keys` entry no endpoint archetype declares would select nothing
    // in every report, so the relation naming it is dropped.
    for relation in &merged.required_relations {
        let Some(reason) = super::undeclared_record_key(relation, archetypes) else {
            continue;
        };
        for (index, module) in outcome.modules.iter().enumerate() {
            if module
                .traceability
                .required_relations
                .iter()
                .any(|declared| declared.name == relation.name)
            {
                drops.push(Drop::new(
                    index,
                    DeclarationKind::RequiredRelation,
                    &relation.name,
                    format!("merged selected traceability model: {reason}"),
                ));
            }
        }
    }

    let dropped_remediations = dropped_names(&outcome.failures, DeclarationKind::Remediation);
    let dropped_archetypes: BTreeMap<String, String> = outcome
        .failures
        .iter()
        .filter_map(|failure| {
            let declaration = failure.declaration.as_deref()?;
            (declaration.kind == DeclarationKind::Archetype)
                .then(|| (declaration.item.clone(), failure.reason.clone()))
        })
        .collect();
    drops.extend(flow_rules::validate_flow_declarations(
        &outcome.modules,
        &merged,
        archetypes,
        edge_types,
        inverse_edges,
        &dropped_remediations,
        &dropped_archetypes,
    ));
    apply(outcome, drops);
    drop_dependents(outcome);
    order_declaration_failures(outcome);
}

/// Put the declaration failures in module load order, then kind, then item.
/// Every other failure keeps its place ahead of them.
fn order_declaration_failures(outcome: &mut LoadOutcome) {
    let (mut declared, others): (Vec<_>, Vec<_>) = std::mem::take(&mut outcome.failures)
        .into_iter()
        .partition(|failure| failure.declaration.is_some());
    declared.sort_by_cached_key(|failure| {
        let declaration = failure.declaration.as_deref();
        (
            outcome
                .modules
                .iter()
                .position(|m| m.name == failure.module),
            declaration.map(|d| d.kind.as_str()),
            declaration.map(|d| d.item.clone()),
        )
    });
    outcome.failures = others;
    outcome.failures.extend(declared);
}

/// The dropped declarations recorded in `failures`, located by module.
pub(super) fn dropped_refs(failures: &[ArchetypeLoadFailure]) -> Vec<DroppedDeclarationRef> {
    failures
        .iter()
        .filter_map(|failure| {
            failure
                .declaration
                .as_deref()
                .map(|d| DroppedDeclarationRef {
                    module: failure.module.clone(),
                    kind: d.kind,
                    item: d.item.clone(),
                })
        })
        .collect()
}

/// Drop each declaration naming a relation or carried field that was dropped,
/// when no other module still declares it.
fn drop_dependents(outcome: &mut LoadOutcome) {
    let merged = merge_traceability(&outcome.modules);
    let dropped_relations = dropped_names(&outcome.failures, DeclarationKind::RequiredRelation);
    let dropped_carried = dropped_names(&outcome.failures, DeclarationKind::CarriedField);
    let mut drops = Vec::new();
    for (index, module) in outcome.modules.iter().enumerate() {
        for name in &module.traceability.design_relations {
            if dropped_relations.contains(name)
                && !merged.required_relations.iter().any(|r| &r.name == name)
            {
                drops.push(Drop::new(
                    index,
                    DeclarationKind::DesignRelation,
                    name,
                    format!(
                        "traceability: design_relations entry '{name}' depends on dropped \
                         required_relations entry '{name}'"
                    ),
                ));
            }
        }
        for rule in &module.traceability.vocabulary {
            if dropped_carried.contains(&rule.carried)
                && !merged.carried_fields.iter().any(|c| c.name == rule.carried)
            {
                drops.push(Drop::new(
                    index,
                    DeclarationKind::Vocabulary,
                    &rule.name,
                    format!(
                        "traceability: vocabulary entry '{}' depends on dropped carried_fields \
                         entry '{}'",
                        rule.name, rule.carried
                    ),
                ));
            }
        }
    }
    apply(outcome, drops);
}
