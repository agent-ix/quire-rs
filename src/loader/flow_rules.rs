//! Load-time resolution of generic flow declarations against the merged registry (FR-096).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use serde_json::Value;

use crate::error::ArchetypeLoadFailure;
use crate::loader::compile::CompiledArchetype;
use crate::loader::LoadedModule;
use crate::traceability::{
    fold_selector_value, FlowCondition, FlowConditionTest, FlowRemediation, FlowRule, FlowStep,
    TraceabilityModel,
};
use crate::vocab::EdgeTypeDef;

type ValidatedPaths<'a> = Vec<(Vec<String>, Vec<&'a FlowStep>)>;

/// Add manifest failures for flow declarations that cannot resolve in the merged registry.
pub(super) fn validate_flow_declarations(
    modules: &[LoadedModule],
    model: &TraceabilityModel,
    archetypes: &BTreeMap<String, Arc<CompiledArchetype>>,
    edge_types: &BTreeMap<String, EdgeTypeDef>,
    inverse_edges: &BTreeMap<String, String>,
    failures: &mut Vec<ArchetypeLoadFailure>,
) -> bool {
    let initial_failures = failures.len();
    report_cross_module_duplicates(modules, failures);

    let all_remediations: BTreeMap<&str, &FlowRemediation> = model
        .remediations
        .iter()
        .map(|remediation| (remediation.name.as_str(), remediation))
        .collect();
    let reserved_checks = reserved_trace_checks(model);

    for module in modules {
        for rule in &module.traceability.flow_rules {
            if let Err(reason) = validate_rule(
                rule,
                &all_remediations,
                archetypes,
                edge_types,
                inverse_edges,
                &reserved_checks,
            ) {
                fail(module, failures, reason);
            }
        }
        for remediation in &module.traceability.remediations {
            if let Err(reason) = validate_remediation_declaration(
                remediation,
                archetypes,
                edge_types,
                inverse_edges,
            ) {
                fail(module, failures, reason);
            }
            if let Some(sets) = &remediation.sets {
                for rule in modules
                    .iter()
                    .flat_map(|declaring| declaring.traceability.flow_rules.iter())
                    .filter(|rule| rule.remediations.contains(&remediation.name))
                {
                    for subject in &rule.subject {
                        if let Err(reason) = validate_field_value(
                            &remediation.name,
                            subject,
                            &sets.field,
                            &sets.value,
                            archetypes,
                        ) {
                            fail(module, failures, reason);
                        }
                    }
                }
            }
        }
    }
    failures.len() != initial_failures
}

fn report_cross_module_duplicates(
    modules: &[LoadedModule],
    failures: &mut Vec<ArchetypeLoadFailure>,
) {
    for (section, names) in [
        (
            "flow_rules",
            modules
                .iter()
                .flat_map(|module| {
                    module
                        .traceability
                        .flow_rules
                        .iter()
                        .map(move |rule| (rule.name.as_str(), module))
                })
                .collect::<Vec<_>>(),
        ),
        (
            "remediations",
            modules
                .iter()
                .flat_map(|module| {
                    module
                        .traceability
                        .remediations
                        .iter()
                        .map(move |remediation| (remediation.name.as_str(), module))
                })
                .collect::<Vec<_>>(),
        ),
    ] {
        let mut by_name: BTreeMap<&str, Vec<&LoadedModule>> = BTreeMap::new();
        for (name, module) in names {
            let modules_for_name = by_name.entry(name).or_default();
            if !modules_for_name.iter().any(|existing| existing.name == module.name) {
                modules_for_name.push(module);
            }
        }
        for (name, declarations) in by_name {
            if declarations.len() < 2 {
                continue;
            }
            let declared_by = declarations
                .iter()
                .map(|module| module.name.as_str())
                .collect::<Vec<_>>()
                .join("', '");
            for module in declarations {
                fail(
                    module,
                    failures,
                    format!(
                        "merged selected traceability model: duplicate {section} entry '{name}' is declared by modules '{declared_by}'"
                    ),
                );
            }
        }
    }
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
        validate_condition(&rule.name, condition, &rule.subject, archetypes, edge_types, inverse_edges)?;
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
            let Some(schema) = archetypes
                .get(endpoint)
                .and_then(|archetype| archetype.frontmatter_schema.as_deref())
            else {
                continue;
            };
            if schema_declares(schema, &condition.field) {
                field_declared = true;
                if let Some(values) = schema_enum(schema, &condition.field) {
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
                    enum_values
                        .iter()
                        .any(|candidate| fold_selector_value(candidate) == fold_selector_value(value))
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

fn step_matches_add(
    step: &FlowStep,
    adds: &crate::traceability::FlowAddRemediation,
) -> bool {
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

fn schema_walk(schema: &Value, field: &str, enum_only: bool) -> Option<Vec<String>> {
    let mut queue = VecDeque::from([schema]);
    while let Some(node) = queue.pop_front() {
        if let Some(property) = node
            .get("properties")
            .and_then(Value::as_object)
            .and_then(|properties| properties.get(field))
        {
            if !enum_only {
                return Some(Vec::new());
            }
            let values = property
                .get("enum")
                .or_else(|| property.get("items").and_then(|items| items.get("enum")))
                .and_then(Value::as_array);
            if let Some(values) = values {
                return Some(values.iter().filter_map(Value::as_str).map(str::to_string).collect());
            }
        }
        match node {
            Value::Object(map) => queue.extend(map.values()),
            Value::Array(items) => queue.extend(items.iter()),
            _ => {}
        }
    }
    None
}

fn condition_paths<'a>(
    subjects: &[String],
    condition: &'a FlowCondition,
) -> ValidatedPaths<'a> {
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

fn fail(module: &LoadedModule, failures: &mut Vec<ArchetypeLoadFailure>, reason: String) {
    failures.push(ArchetypeLoadFailure {
        module: module.name.clone(),
        archetype: "<manifest>".to_string(),
        path: module.root.join("manifest.yaml"),
        reason,
    });
}
