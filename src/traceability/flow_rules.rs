//! Generic flow-rule declarations (FR-096).

use serde::{Deserialize, Serialize};

use super::EdgeSide;

/// One generic rule evaluated over authored documents and resolved edges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowRule {
    /// Unique declaration name.
    pub name: String,
    /// Document archetypes evaluated by the rule.
    pub subject: Vec<String>,
    /// All conditions that must hold for a denial.
    #[serde(rename = "match")]
    pub match_conditions: Vec<FlowCondition>,
    /// Any condition that clears a denial.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unless_any: Vec<FlowCondition>,
    /// The `<check>` portion of `trace:<check>`.
    pub check: String,
    /// Remediation names suggested by a denial.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediations: Vec<String>,
    /// Human-readable reason for the rule.
    pub rationale: String,
}

/// A field predicate over the documents reached by a path.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawFlowCondition")]
pub struct FlowCondition {
    /// Exactly one of `path` and `any_path` is present.
    pub paths: Vec<Vec<FlowStep>>,
    /// Whether the declaration used `any_path`, including when it has one path.
    any_path: bool,
    /// Frontmatter key read on path endpoints.
    pub field: String,
    /// Values accepted by this predicate.
    pub test: FlowConditionTest,
}

impl Serialize for FlowCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(3))?;
        if self.any_path {
            map.serialize_entry("any_path", &self.paths)?;
        } else {
            map.serialize_entry("path", &self.paths[0])?;
        }
        map.serialize_entry("field", &self.field)?;
        match &self.test {
            FlowConditionTest::AnyOf(values) => map.serialize_entry("any_of", values)?,
            FlowConditionTest::Absent => map.serialize_entry("absent", &true)?,
            FlowConditionTest::Present => map.serialize_entry("present", &true)?,
        }
        map.end()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFlowCondition {
    #[serde(default)]
    path: Option<Vec<FlowStep>>,
    #[serde(default)]
    any_path: Option<Vec<Vec<FlowStep>>>,
    field: String,
    #[serde(default)]
    any_of: Option<Vec<String>>,
    #[serde(default)]
    absent: Option<bool>,
    #[serde(default)]
    present: Option<bool>,
}

impl TryFrom<RawFlowCondition> for FlowCondition {
    type Error = String;

    fn try_from(raw: RawFlowCondition) -> Result<Self, Self::Error> {
        let (paths, any_path) = match (raw.path, raw.any_path) {
            (Some(path), None) => (vec![path], false),
            (None, Some(paths)) if !paths.is_empty() => (paths, true),
            (None, Some(_)) => return Err("`any_path` is empty".into()),
            (Some(_), Some(_)) => return Err("both `path` and `any_path` are present".into()),
            (None, None) => return Err("neither `path` nor `any_path` is present".into()),
        };
        let test = match (raw.any_of, raw.absent, raw.present) {
            (Some(values), None, None) if !values.is_empty() => FlowConditionTest::AnyOf(values),
            (Some(_), None, None) => return Err("`any_of` is empty".into()),
            (None, Some(true), None) => FlowConditionTest::Absent,
            (None, None, Some(true)) => FlowConditionTest::Present,
            (None, Some(false), None) => return Err("`absent` must be true".into()),
            (None, None, Some(false)) => return Err("`present` must be true".into()),
            _ => return Err("exactly one of `any_of`, `absent` and `present` is required".into()),
        };
        Ok(Self {
            paths,
            any_path,
            field: raw.field,
            test,
        })
    }
}

/// The test applied to a condition's path endpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowConditionTest {
    /// At least one endpoint has a matching scalar or list member.
    AnyOf(Vec<String>),
    /// No endpoint states the field.
    Absent,
    /// At least one endpoint states the field.
    Present,
}

/// One resolved edge hop in a flow condition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowStep {
    /// Declared edge verb.
    pub edge: String,
    /// Which end of the relation is the current document.
    pub direction: EdgeSide,
    /// Accepted archetypes at the other end.
    pub to: Vec<String>,
}

/// A module-declared corrective model change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowRemediation {
    /// Unique remediation name.
    pub name: String,
    /// Human-readable change description.
    pub description: String,
    /// Add an authored edge from documents with the declared field value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adds: Option<FlowAddRemediation>,
    /// Set a subject document's frontmatter field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sets: Option<FlowSetRemediation>,
    /// Remove an edge from the subject's paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removes: Option<FlowRemoveRemediation>,
}

/// Edge addition described by a remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowAddRemediation {
    /// Declared edge verb.
    pub edge: String,
    /// Direction of the edge to add.
    pub direction: EdgeSide,
    /// Archetype declaring the new edge.
    pub from: String,
    /// Frontmatter field whose value accompanies the edge.
    pub field: String,
    /// Value that accompanies the edge.
    pub value: String,
}

/// Frontmatter update described by a remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowSetRemediation {
    /// Frontmatter field to set.
    pub field: String,
    /// Value to set.
    pub value: String,
}

/// Edge removal described by a remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowRemoveRemediation {
    /// Declared edge verb.
    pub edge: String,
    /// Direction of the edge to remove.
    pub direction: EdgeSide,
}

impl super::TraceabilityModel {
    /// Validate the flow declaration's local shape (FR-096-AC-6, AC-7).
    pub(super) fn validate_flow_rules(&self) -> Result<(), String> {
        let mut names = std::collections::BTreeSet::new();
        for rule in &self.flow_rules {
            check_nonblank("flow_rules", &rule.name, "name", &rule.name)?;
            if !names.insert(rule.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate flow_rules entry '{}'",
                    rule.name
                ));
            }
            if rule.subject.is_empty() || rule.subject.iter().any(|name| name.trim().is_empty()) {
                return Err(format!(
                    "traceability: flow_rules entry '{}' has an empty `subject`",
                    rule.name
                ));
            }
            if rule.match_conditions.is_empty() {
                return Err(format!(
                    "traceability: flow_rules entry '{}' has an empty `match`",
                    rule.name
                ));
            }
            check_nonblank("flow_rules", &rule.name, "check", &rule.check)?;
            if !crate::grammar::is_severity_key(&crate::grammar::severity_key("trace", &rule.check))
            {
                return Err(format!(
                    "traceability: flow_rules entry '{}' has a `check` token '{}' that cannot \
                     form a `trace:<check>` severity key",
                    rule.name, rule.check
                ));
            }
            check_nonblank("flow_rules", &rule.name, "rationale", &rule.rationale)?;
            for remediation in &rule.remediations {
                if remediation.trim().is_empty() {
                    return Err(format!(
                        "traceability: flow_rules entry '{}' has a blank remediation name",
                        rule.name
                    ));
                }
            }
            for condition in rule.match_conditions.iter().chain(&rule.unless_any) {
                check_condition(&rule.name, condition)?;
            }
        }

        let mut remediation_names = std::collections::BTreeSet::new();
        for remediation in &self.remediations {
            check_nonblank("remediations", &remediation.name, "name", &remediation.name)?;
            if !remediation_names.insert(remediation.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate remediations entry '{}'",
                    remediation.name
                ));
            }
            check_nonblank(
                "remediations",
                &remediation.name,
                "description",
                &remediation.description,
            )?;
            let changes = [
                remediation.adds.is_some(),
                remediation.sets.is_some(),
                remediation.removes.is_some(),
            ]
            .into_iter()
            .filter(|present| *present)
            .count();
            if changes != 1 {
                return Err(format!(
                    "traceability: remediations entry '{}' must declare exactly one of `adds`, \
                     `sets` and `removes`",
                    remediation.name
                ));
            }
            if let Some(adds) = &remediation.adds {
                check_nonblank("remediations", &remediation.name, "adds.edge", &adds.edge)?;
                check_nonblank("remediations", &remediation.name, "adds.from", &adds.from)?;
                check_nonblank("remediations", &remediation.name, "adds.field", &adds.field)?;
                check_nonblank("remediations", &remediation.name, "adds.value", &adds.value)?;
                let _ = adds.direction;
            }
            if let Some(sets) = &remediation.sets {
                check_nonblank("remediations", &remediation.name, "sets.field", &sets.field)?;
                check_nonblank("remediations", &remediation.name, "sets.value", &sets.value)?;
            }
            if let Some(removes) = &remediation.removes {
                check_nonblank(
                    "remediations",
                    &remediation.name,
                    "removes.edge",
                    &removes.edge,
                )?;
                let _ = removes.direction;
            }
        }
        Ok(())
    }
}

fn check_nonblank(section: &str, entry: &str, field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "traceability: {section} entry '{entry}' has an empty `{field}`"
        ));
    }
    Ok(())
}

fn check_condition(rule: &str, condition: &FlowCondition) -> Result<(), String> {
    let bad = |why: &str| {
        format!("traceability: flow_rules entry '{rule}' has an invalid condition: {why}")
    };
    if condition.field.trim().is_empty() {
        return Err(bad("`field` is blank"));
    }
    for path in &condition.paths {
        if path.len() > 2 {
            return Err(bad("a `path` may contain at most two steps"));
        }
        for step in path {
            if step.edge.trim().is_empty() {
                return Err(bad("a step `edge` must be non-blank"));
            }
            if step.to.is_empty() || step.to.iter().any(|name| name.trim().is_empty()) {
                return Err(bad("a step `to` must contain non-blank archetype names"));
            }
        }
    }
    if let FlowConditionTest::AnyOf(values) = &condition.test {
        if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
            return Err(bad("`any_of` must contain non-blank values"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::FlowCondition;

    #[test]
    fn condition_round_trip_preserves_single_any_path() {
        let yaml = "any_path:\n  - []\nfield: id\npresent: true\n";
        let condition: FlowCondition = serde_yaml::from_str(yaml).expect("condition parses");
        let encoded = serde_yaml::to_string(&condition).expect("condition serializes");

        assert!(encoded.contains("any_path:"));
        assert!(!encoded.lines().any(|line| line.starts_with("path:")));
        assert_eq!(
            serde_yaml::from_str::<FlowCondition>(&encoded).unwrap(),
            condition
        );
    }

    #[test]
    fn condition_rejects_multiple_tests_and_paths() {
        let yaml = "path: []\nany_path: [[]]\nfield: id\npresent: true\n";
        assert!(serde_yaml::from_str::<FlowCondition>(yaml).is_err());

        let yaml = "path: []\nfield: id\nabsent: true\npresent: true\n";
        assert!(serde_yaml::from_str::<FlowCondition>(yaml).is_err());
    }
}
