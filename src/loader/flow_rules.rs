//! Load-time resolution of generic flow declarations against the merged registry (FR-096).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde_json::Value;

use crate::loader::compile::CompiledArchetype;
use crate::loader::declarations::Drop;
use crate::loader::LoadedModule;
use crate::traceability::{
    fold_selector_value, DeclarationKind, FlowCondition, FlowConditionTest, FlowRemediation,
    FlowRule, FlowStep, TraceabilityModel,
};
use crate::vocab::EdgeTypeDef;

type ValidatedPaths<'a> = Vec<(Vec<String>, Vec<&'a FlowStep>)>;

/// The flow declarations that cannot resolve in the merged registry, each
/// dropped alone. `dropped_remediations` holds the remediations already dropped
/// for another reason; a rule naming one depends on a dropped declaration.
pub(super) fn validate_flow_declarations(
    modules: &[LoadedModule],
    model: &TraceabilityModel,
    archetypes: &BTreeMap<String, Arc<CompiledArchetype>>,
    edge_types: &BTreeMap<String, EdgeTypeDef>,
    inverse_edges: &BTreeMap<String, String>,
    dropped_remediations: &BTreeSet<String>,
) -> Vec<Drop> {
    let mut drops = Vec::new();
    let all_remediations: BTreeMap<&str, &FlowRemediation> = model
        .remediations
        .iter()
        .map(|remediation| (remediation.name.as_str(), remediation))
        .collect();
    let reserved_checks = reserved_trace_checks(model);

    let mut dropped_remediations = dropped_remediations.clone();
    for (index, module) in modules.iter().enumerate() {
        for remediation in &module.traceability.remediations {
            if let Err(reason) =
                validate_remediation_declaration(remediation, archetypes, edge_types, inverse_edges)
            {
                dropped_remediations.insert(remediation.name.clone());
                drops.push(Drop::new(
                    index,
                    DeclarationKind::Remediation,
                    &remediation.name,
                    reason,
                ));
            }
        }
    }

    let mut dropped_rules: BTreeSet<(usize, String)> = BTreeSet::new();
    // A `sets` value the schema of a rule's subject does not admit drops the
    // rule that names the remediation; the remediation stays for the rules it
    // fits.
    for module in modules {
        for remediation in &module.traceability.remediations {
            let Some(sets) = &remediation.sets else {
                continue;
            };
            if dropped_remediations.contains(&remediation.name) {
                continue;
            }
            for (index, declaring) in modules.iter().enumerate() {
                for rule in declaring
                    .traceability
                    .flow_rules
                    .iter()
                    .filter(|rule| rule.remediations.contains(&remediation.name))
                {
                    if dropped_rules.contains(&(index, rule.name.clone())) {
                        continue;
                    }
                    let refused = rule.subject.iter().find_map(|subject| {
                        validate_field_value(
                            &remediation.name,
                            subject,
                            &sets.field,
                            &sets.value,
                            archetypes,
                        )
                        .err()
                    });
                    if let Some(reason) = refused {
                        dropped_rules.insert((index, rule.name.clone()));
                        drops.push(Drop::new(
                            index,
                            DeclarationKind::FlowRule,
                            &rule.name,
                            format!(
                                "traceability: flow_rules entry '{}' depends on remediation \
                                 '{}': {reason}",
                                rule.name, remediation.name
                            ),
                        ));
                    }
                }
            }
        }
    }
    for (index, module) in modules.iter().enumerate() {
        for rule in &module.traceability.flow_rules {
            if dropped_rules.contains(&(index, rule.name.clone())) {
                continue;
            }
            let dependency = rule
                .remediations
                .iter()
                .find(|name| dropped_remediations.contains(*name));
            let verdict = match dependency {
                Some(name) => Err(format!(
                    "traceability: flow_rules entry '{}' depends on dropped remediations entry \
                     '{name}'",
                    rule.name
                )),
                None => validate_rule(
                    rule,
                    &all_remediations,
                    archetypes,
                    edge_types,
                    inverse_edges,
                    &reserved_checks,
                ),
            };
            if let Err(reason) = verdict {
                dropped_rules.insert((index, rule.name.clone()));
                drops.push(Drop::new(
                    index,
                    DeclarationKind::FlowRule,
                    &rule.name,
                    reason,
                ));
            }
        }
    }

    drops
}

fn validate_rule(
    rule: &FlowRule,
    remediations: &BTreeMap<&str, &FlowRemediation>,
    archetypes: &BTreeMap<String, Arc<CompiledArchetype>>,
    edge_types: &BTreeMap<String, EdgeTypeDef>,
    inverse_edges: &BTreeMap<String, String>,
    reserved_checks: &BTreeSet<String>,
) -> Result<(), String> {
    for subject in &rule.subject {
        require_archetype("subject", &rule.name, subject, archetypes)?;
    }
    for condition in rule.match_conditions.iter().chain(&rule.unless_any) {
        validate_condition(
            &rule.name,
            condition,
            &rule.subject,
            archetypes,
            edge_types,
            inverse_edges,
        )?;
    }
    if reserved_checks.contains(&rule.check) {
        return Err(format!(
            "traceability: flow_rules entry '{}' uses check '{}' already reported by another trace declaration or engine check",
            rule.name, rule.check
        ));
    }
    for name in &rule.remediations {
        let Some(remediation) = remediations.get(name.as_str()) else {
            return Err(format!(
                "traceability: flow_rules entry '{}' names undeclared remediation '{name}'",
                rule.name
            ));
        };
        if !remediation_matches(rule, remediation) {
            return Err(format!(
                "traceability: flow_rules entry '{}' names remediation '{}' whose change cannot clear this rule",
                rule.name, remediation.name
            ));
        }
    }
    Ok(())
}

fn validate_condition(
    rule: &str,
    condition: &FlowCondition,
    subjects: &[String],
    archetypes: &BTreeMap<String, Arc<CompiledArchetype>>,
    edge_types: &BTreeMap<String, EdgeTypeDef>,
    inverse_edges: &BTreeMap<String, String>,
) -> Result<(), String> {
    let mut declared_enums = Vec::new();
    for (endpoints, path) in condition_paths(subjects, condition) {
        let mut current = endpoints;
        for step in path {
            require_edge(rule, &step.edge, edge_types, inverse_edges)?;
            for target in &step.to {
                require_archetype("step to", rule, target, archetypes)?;
            }
            current = step.to.clone();
        }
        let mut field_declared = false;
        for endpoint in &current {
            if condition.field == "id" {
                field_declared = true;
                continue;
            }
            let Some(archetype) = archetypes.get(endpoint) else {
                continue;
            };
            let Some(schema) = archetype.frontmatter_schema.as_deref() else {
                continue;
            };
            if schema_declares(schema, &condition.field) {
                field_declared = true;
                let values = flow_schema_enum(archetype, &condition.field).map_err(|error| {
                    format!(
                        "traceability: flow_rules entry '{rule}' condition field '{}' {error}",
                        condition.field
                    )
                })?;
                if let Some(values) = values {
                    declared_enums.push((endpoint.clone(), values));
                }
            }
        }
        if !field_declared {
            return Err(format!(
                "traceability: flow_rules entry '{rule}' condition field '{}' is not declared by any endpoint archetype ({})",
                condition.field,
                current.join(", ")
            ));
        }
    }
    if let FlowConditionTest::AnyOf(values) = &condition.test {
        if !declared_enums.is_empty() {
            for value in values {
                let accepted = declared_enums.iter().any(|(_, enum_values)| {
                    enum_values.iter().any(|candidate| {
                        fold_selector_value(candidate) == fold_selector_value(value)
                    })
                });
                if !accepted {
                    return Err(format!(
                        "traceability: flow_rules entry '{rule}' value '{value}' is outside the enum of field '{}' on endpoint archetypes ({})",
                        condition.field,
                        declared_enums
                            .iter()
                            .map(|(name, _)| name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_remediation_declaration(
    remediation: &FlowRemediation,
    archetypes: &BTreeMap<String, Arc<CompiledArchetype>>,
    edge_types: &BTreeMap<String, EdgeTypeDef>,
    inverse_edges: &BTreeMap<String, String>,
) -> Result<(), String> {
    if let Some(adds) = &remediation.adds {
        require_edge(&remediation.name, &adds.edge, edge_types, inverse_edges)?;
        require_archetype("adds from", &remediation.name, &adds.from, archetypes)?;
        validate_field_value(
            &remediation.name,
            &adds.from,
            &adds.field,
            &adds.value,
            archetypes,
        )?;
    }
    if let Some(removes) = &remediation.removes {
        require_edge(&remediation.name, &removes.edge, edge_types, inverse_edges)?;
    }
    Ok(())
}

fn validate_field_value(
    name: &str,
    archetype: &str,
    field: &str,
    value: &str,
    archetypes: &BTreeMap<String, Arc<CompiledArchetype>>,
) -> Result<(), String> {
    if field == "id" {
        return Ok(());
    }
    let Some(schema) = archetypes
        .get(archetype)
        .and_then(|entry| entry.frontmatter_schema.as_deref())
    else {
        return Err(format!(
            "traceability: remediations entry '{name}' field '{field}' is not declared by archetype '{archetype}'"
        ));
    };
    if !schema_declares(schema, field) {
        return Err(format!(
            "traceability: remediations entry '{name}' field '{field}' is not declared by archetype '{archetype}'"
        ));
    }
    if let Some(values) = schema_enum(schema, field) {
        if !values
            .iter()
            .any(|candidate| fold_selector_value(candidate) == fold_selector_value(value))
        {
            return Err(format!(
                "traceability: remediations entry '{name}' value '{value}' is outside field '{field}' enum for archetype '{archetype}'"
            ));
        }
    }
    Ok(())
}

fn remediation_matches(rule: &FlowRule, remediation: &FlowRemediation) -> bool {
    if let Some(adds) = &remediation.adds {
        return rule.unless_any.iter().any(|condition| {
            condition.paths.iter().any(|path| {
                path.len() == 1
                    && step_matches_add(&path[0], adds)
                    && matches_value(condition, &adds.value)
            }) && condition.field == adds.field
        });
    }
    if let Some(sets) = &remediation.sets {
        return rule.subject.iter().any(|_| {
            rule.unless_any.iter().any(|condition| {
                condition.paths.iter().any(Vec::is_empty)
                    && condition.field == sets.field
                    && matches_value(condition, &sets.value)
            })
        });
    }
    if let Some(removes) = &remediation.removes {
        return rule.match_conditions.iter().any(|condition| {
            !matches!(condition.test, FlowConditionTest::Absent)
                && condition.paths.iter().all(|path| {
                    path.first().is_some_and(|step| {
                        step.edge == removes.edge && step.direction == removes.direction
                    })
                })
        });
    }
    false
}

fn step_matches_add(step: &FlowStep, adds: &crate::traceability::FlowAddRemediation) -> bool {
    step.edge == adds.edge
        && step.direction == adds.direction
        && step.to.iter().any(|to| to == &adds.from)
}

fn matches_value(condition: &FlowCondition, value: &str) -> bool {
    matches!(&condition.test, FlowConditionTest::AnyOf(values) if values.iter().any(|candidate| fold_selector_value(candidate) == fold_selector_value(value)))
}

fn require_archetype(
    section: &str,
    name: &str,
    archetype: &str,
    archetypes: &BTreeMap<String, Arc<CompiledArchetype>>,
) -> Result<(), String> {
    if archetypes.contains_key(archetype) {
        Ok(())
    } else {
        Err(format!(
            "traceability: flow_rules entry '{name}' names unknown {section} archetype '{archetype}'"
        ))
    }
}

fn require_edge(
    name: &str,
    edge: &str,
    edge_types: &BTreeMap<String, EdgeTypeDef>,
    inverse_edges: &BTreeMap<String, String>,
) -> Result<(), String> {
    if edge_types.contains_key(edge) || inverse_edges.contains_key(edge) {
        Ok(())
    } else {
        Err(format!(
            "traceability: flow declaration '{name}' names unknown edge verb '{edge}'"
        ))
    }
}

fn schema_declares(schema: &Value, field: &str) -> bool {
    schema_walk(schema, field, false).is_some()
}

fn schema_enum(schema: &Value, field: &str) -> Option<Vec<String>> {
    schema_walk(schema, field, true)
}

pub(crate) fn flow_schema_enum(
    archetype: &CompiledArchetype,
    field: &str,
) -> Result<Option<Vec<String>>, String> {
    let Some(schema) = archetype.frontmatter_schema.as_deref() else {
        return Ok(None);
    };
    let Some(property) = schema
        .get("properties")
        .and_then(Value::as_object)
        .and_then(|properties| properties.get(field))
    else {
        return Ok(None);
    };
    let has_ref = property.get("$ref").is_some()
        || property
            .get("items")
            .and_then(|items| items.get("$ref"))
            .is_some();
    if !has_ref {
        return schema_enum(schema, field).map_or(Ok(None), |values| Ok(Some(values)));
    }
    if let Some(error) = &archetype.frontmatter_schema_ref_error {
        return Err(format!(
            "schema reference in '{}' is unresolved: {error}",
            archetype
                .frontmatter_schema_path
                .as_deref()
                .unwrap_or("frontmatter schema")
        ));
    }

    let mut documents = archetype
        .frontmatter_schema_documents
        .as_deref()
        .cloned()
        .unwrap_or_default();
    let root_id = schema
        .get("$id")
        .and_then(Value::as_str)
        .unwrap_or("<root>")
        .to_string();
    documents
        .entry(root_id.clone())
        .or_insert_with(|| schema.clone());
    let location = format!(
        "{}#/properties/{field}",
        archetype
            .frontmatter_schema_path
            .as_deref()
            .unwrap_or("frontmatter schema")
    );
    let mut references = BTreeSet::new();
    match enum_at_node(
        property,
        schema,
        &root_id,
        &documents,
        &location,
        &mut references,
    ) {
        Ok(values) => Ok(values),
        Err(error) => {
            let detail = archetype
                .frontmatter_schema_ref_error
                .as_deref()
                .map(|stored| format!("; module reference scan: {stored}"))
                .unwrap_or_default();
            Err(format!("schema reference {error}{detail}"))
        }
    }
}

fn enum_at_node(
    node: &Value,
    current_document: &Value,
    current_id: &str,
    documents: &BTreeMap<String, Value>,
    location: &str,
    references: &mut BTreeSet<String>,
) -> Result<Option<Vec<String>>, String> {
    if let Some(values) = node
        .get("enum")
        .and_then(Value::as_array)
        .map(|values| enum_strings(values))
    {
        return Ok(Some(values));
    }
    if let Some(items) = node.get("items") {
        let item_location = format!("{location}/items");
        if let Some(values) = enum_at_node(
            items,
            current_document,
            current_id,
            documents,
            &item_location,
            references,
        )? {
            return Ok(Some(values));
        }
    }
    let Some(reference) = node.get("$ref").and_then(Value::as_str) else {
        return Ok(None);
    };
    if !references.insert(reference.to_string()) {
        return Err(format!(
            "at {location}/$ref '{reference}' forms a reference cycle"
        ));
    }
    let (document_id, fragment) = match reference.split_once('#') {
        Some((id, fragment)) => (if id.is_empty() { current_id } else { id }, fragment),
        None => (reference, ""),
    };
    let target_document = if document_id == current_id {
        current_document
    } else {
        documents.get(document_id).ok_or_else(|| {
            format!("at {location}/$ref '{reference}' names no local schema document")
        })?
    };
    let target = if fragment.is_empty() {
        target_document
    } else {
        target_document
            .pointer(fragment)
            .ok_or_else(|| format!("at {location}/$ref '{reference}' has no target"))?
    };
    let target_id = target_document
        .get("$id")
        .and_then(Value::as_str)
        .unwrap_or(document_id);
    let result = enum_at_node(
        target,
        target_document,
        target_id,
        documents,
        &format!("{location}/$ref[{reference}]"),
        references,
    );
    references.remove(reference);
    result
}

fn enum_strings(values: &[Value]) -> Vec<String> {
    values
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn schema_walk(schema: &Value, field: &str, enum_only: bool) -> Option<Vec<String>> {
    let property = schema
        .get("properties")
        .and_then(Value::as_object)
        .and_then(|properties| properties.get(field))?;
    if !enum_only {
        return Some(Vec::new());
    }
    property
        .get("enum")
        .or_else(|| property.get("items").and_then(|items| items.get("enum")))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
}

fn condition_paths<'a>(subjects: &[String], condition: &'a FlowCondition) -> ValidatedPaths<'a> {
    condition
        .paths
        .iter()
        .map(|path| {
            let mut current = subjects.to_vec();
            let mut steps = Vec::new();
            for step in path {
                steps.push(step);
                current = step.to.clone();
            }
            (current, steps)
        })
        .collect()
}

fn reserved_trace_checks(model: &TraceabilityModel) -> BTreeSet<String> {
    let mut checks = BTreeSet::from([
        "dangling-trace-reference".to_string(),
        "reference-kind-mismatch".to_string(),
        "undeclared-relation-vocabulary".to_string(),
        "undeclared-field-value".to_string(),
        "carried-vocabulary-mismatch".to_string(),
        "undeclared-vocabulary-value".to_string(),
    ]);
    for check in model
        .required_relations
        .iter()
        .map(|entry| &entry.check)
        .chain(model.vocabulary_coverage.iter().map(|entry| &entry.check))
        .chain(model.field_consistency.iter().map(|entry| &entry.check))
        .chain(model.required_fields.iter().map(|entry| &entry.check))
        .chain(model.vocabulary.iter().map(|entry| &entry.check))
    {
        checks.insert(check.clone());
    }
    checks
}
