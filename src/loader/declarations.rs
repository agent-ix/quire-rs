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
use crate::traceability::DeclarationKind;
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

/// Drop each declaration of a conflict from every module declaring it.
fn conflict(
    modules: &[LoadedModule],
    drops: &mut Vec<Drop>,
    kind: DeclarationKind,
    item: &str,
    declaring: &BTreeSet<usize>,
    what: &str,
) {
    for index in declaring {
        let other = others(modules, declaring, *index);
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
    let mut conflicting: BTreeMap<&str, BTreeSet<usize>> = BTreeMap::new();
    for (position, ((prefix, _), declaring)) in entries.iter().enumerate() {
        for ((other, _), other_declaring) in &entries[position + 1..] {
            if prefix.starts_with(other) || other.starts_with(prefix) {
                conflicting
                    .entry(prefix)
                    .or_default()
                    .extend(declaring.iter().copied());
                conflicting
                    .entry(other)
                    .or_default()
                    .extend(other_declaring.iter().copied());
            }
        }
    }
    for (prefix, declaring) in conflicting {
        conflict(
            modules,
            drops,
            DeclarationKind::TraceIdQualifier,
            prefix,
            &declaring,
            &format!(
                "trace_id_qualifiers prefix '{prefix}' is not unique or overlaps another prefix, \
                 so a tag could name either repository"
            ),
        );
    }
}

/// A retired value another vocabulary declares as minting no source symbol
/// contradicts it; both declarations are dropped.
fn retirement_conflicts(modules: &[LoadedModule], drops: &mut Vec<Drop>) {
    for (index, module) in modules.iter().enumerate() {
        for value in &module.traceability.vocabularies.retired {
            let minting: BTreeSet<usize> = modules
                .iter()
                .enumerate()
                .filter(|(_, other)| other.traceability.vocabularies.mints_no_symbol(value))
                .map(|(other, _)| other)
                .collect();
            if minting.is_empty() {
                continue;
            }
            let mut declaring = minting.clone();
            declaring.insert(index);
            let what = format!(
                "vocabularies.retired value '{value}' is also declared in no_source_symbol"
            );
            drops.push(Drop::new(
                index,
                DeclarationKind::RetirementVocabulary,
                value,
                format!(
                    "merged selected traceability model: {what}; declared by modules {}",
                    others(modules, &minting, index)
                ),
            ));
            for other in minting {
                let symbol = modules[other]
                    .traceability
                    .vocabularies
                    .no_source_symbol
                    .iter()
                    .find(|declared| declared.trim().eq_ignore_ascii_case(value.trim()))
                    .map_or(value.as_str(), String::as_str);
                drops.push(Drop::new(
                    other,
                    DeclarationKind::NoSourceSymbol,
                    symbol,
                    format!(
                        "merged selected traceability model: {what}; retired by module '{}'",
                        module.name
                    ),
                ));
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
    drops.extend(flow_rules::validate_flow_declarations(
        &outcome.modules,
        &merged,
        archetypes,
        edge_types,
        inverse_edges,
        &dropped_remediations,
    ));
    apply(outcome, drops);
    drop_dependents(outcome);
    order_declaration_failures(outcome);
}

/// Put the declaration failures in module load order, then kind, then item,
/// leaving every other failure where it is.
fn order_declaration_failures(outcome: &mut LoadOutcome) {
    let module_position = |name: &str| outcome.modules.iter().position(|m| m.name == name);
    let slots: Vec<usize> = outcome
        .failures
        .iter()
        .enumerate()
        .filter(|(_, failure)| failure.declaration.is_some())
        .map(|(slot, _)| slot)
        .collect();
    let mut ordered: Vec<ArchetypeLoadFailure> = slots
        .iter()
        .map(|slot| {
            std::mem::replace(
                &mut outcome.failures[*slot],
                declaration_failure(
                    "",
                    Default::default(),
                    DeclarationKind::Exclude,
                    "",
                    String::new(),
                ),
            )
        })
        .collect();
    ordered.sort_by(|a, b| {
        let key = |f: &ArchetypeLoadFailure| {
            let declaration = f.declaration.as_ref();
            (
                module_position(&f.module),
                declaration.map(|d| d.kind.as_str()),
                declaration.map(|d| d.item.clone()),
            )
        };
        key(a).cmp(&key(b))
    });
    for (slot, failure) in slots.into_iter().zip(ordered) {
        outcome.failures[slot] = failure;
    }
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
