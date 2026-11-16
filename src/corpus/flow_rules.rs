//! Evaluation and reporting for module-declared flow rules (FR-096).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use super::declared_tables::ExcludeSet;
use super::resolve::Resolution;
use super::spec::{artifact_key, Spec};
use super::validate::{pack, BundleFinding, BundleReport};
use super::walk::LoadedDocument;
use crate::grammar::{GrammarSeverity, GrammarSeverityLevel};
use crate::registry::Registry;
use crate::traceability::{
    fold_selector_value, EdgeSide, FlowCondition, FlowConditionTest, FlowRemediation, FlowRule,
    FlowStep,
};

/// Serialized generic flow report consumed by downstream tools (FR-096).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FlowReport {
    /// Versioned report schema.
    pub schema: &'static str,
    /// Declared rules in declaration order.
    pub rules: Vec<FlowRuleSummary>,
    /// Enum vocabularies read from the loaded archetype schemas.
    pub vocabularies: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    /// Authored facts read by the declared rules, keyed by archetype.
    pub facts: BTreeMap<String, Vec<FlowFact>>,
    /// Denials of any enabled severity, sorted by check, subject, and rule.
    pub denials: Vec<FlowDenial>,
    /// Declared remediation details in declaration order.
    pub remediations: Vec<FlowRemediation>,
}

/// One declared rule in a flow report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FlowRuleSummary {
    /// Rule name.
    pub name: String,
    /// Subject archetypes.
    pub subject: Vec<String>,
    /// Conditions that must all hold.
    #[serde(rename = "match")]
    pub match_conditions: Vec<FlowCondition>,
    /// Conditions that clear the denial; always present, including when empty.
    pub unless_any: Vec<FlowCondition>,
    /// Finding check token.
    pub check: String,
    /// Suggested remediation names.
    pub remediations: Vec<String>,
    /// Why the rule exists.
    pub rationale: String,
}

/// One document's authored fields and resolved edge targets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FlowFact {
    /// Authored artifact id, or its stable corpus key when no id is stated.
    pub id: String,
    /// Declared corpus-member identity, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Corpus-relative source path.
    pub path: String,
    /// Only authored frontmatter fields read by flow rules.
    pub fields: BTreeMap<String, Value>,
    /// Resolved edges followed from this document by flow-rule paths.
    pub edges: Vec<FlowFactEdge>,
}

/// One resolved edge read by a flow rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FlowFactEdge {
    /// Edge verb.
    pub edge: String,
    /// Direction from this fact's document.
    pub direction: &'static str,
    /// Other edge endpoint.
    pub target: FlowDocumentRef,
    /// Archetype of the target endpoint.
    pub archetype: String,
}

/// A document identity in the matched evidence of one denial.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FlowDocumentRef {
    /// Authored artifact id, or stable corpus key when absent.
    pub id: String,
    /// Declared corpus-member identity, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

/// Subject identity in a flow denial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FlowSubjectRef {
    /// Authored id, or stable corpus key when absent.
    pub id: String,
    /// Declared corpus-member identity, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Subject archetype.
    pub archetype: String,
}

/// One flow-rule denial in a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FlowDenial {
    /// Rule name.
    pub rule: String,
    /// Check token.
    pub check: String,
    /// Subject document identity.
    pub subject: FlowSubjectRef,
    /// Authored matched facts, grouped by archetype.
    pub matched: BTreeMap<String, Vec<FlowDocumentRef>>,
    /// Suggested remediation details, in rule declaration order.
    pub remediations: Vec<FlowDenialRemediation>,
    /// Why the rule exists.
    pub rationale: String,
    /// Applied check severity.
    pub severity: &'static str,
}

/// One remediation applicable to a specific denial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FlowDenialRemediation {
    /// Remediation name.
    pub name: String,
    /// Human-readable change description.
    pub description: String,
    /// Authored change this remediation describes.
    pub change: FlowRemediationChange,
}

/// The single model change described by a remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FlowRemediationChange {
    /// Edge addition when declared by the remediation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adds: Option<crate::traceability::FlowAddRemediation>,
    /// Field update when declared by the remediation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sets: Option<crate::traceability::FlowSetRemediation>,
    /// Edge removal when declared by the remediation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removes: Option<crate::traceability::FlowRemoveRemediation>,
}

/// The resolved edge index used by flow-rule paths.
#[derive(Debug, Default)]
pub(crate) struct FlowEdges {
    neighbors: BTreeMap<(usize, String, bool), Vec<usize>>,
}

type FlowEdgeReads = BTreeMap<String, BTreeSet<(String, String, Vec<String>)>>;
type ConditionPathReads<'a> = Vec<(Vec<String>, Vec<(Vec<String>, &'a FlowStep)>)>;

impl FlowEdges {
    pub(crate) fn build(spec: &Spec, registry: &Registry) -> Self {
        let mut neighbors: BTreeMap<(usize, String, bool), BTreeSet<usize>> = BTreeMap::new();
        let inverse = registry.inverse_index();
        for edge in spec.corpus_edges() {
            let Some(target) = edge.target_slot else {
                continue;
            };
            if edge.resolution != Resolution::Resolved {
                continue;
            }
            neighbors
                .entry((edge.source_slot, edge.edge_type.clone(), false))
                .or_default()
                .insert(target);
            neighbors
                .entry((target, edge.edge_type.clone(), true))
                .or_default()
                .insert(edge.source_slot);

            let counterpart = inverse.get(&edge.edge_type).or_else(|| {
                inverse
                    .iter()
                    .find(|(_, forward)| forward.as_str() == edge.edge_type)
                    .map(|(inverse_label, _)| inverse_label)
            });
            if let Some(counterpart) = counterpart {
                neighbors
                    .entry((target, counterpart.clone(), false))
                    .or_default()
                    .insert(edge.source_slot);
                neighbors
                    .entry((edge.source_slot, counterpart.clone(), true))
                    .or_default()
                    .insert(target);
            }
        }
        Self {
            neighbors: neighbors
                .into_iter()
                .map(|(key, slots)| (key, slots.into_iter().collect()))
                .collect(),
        }
    }

    fn neighbors(&self, slot: usize, step: &FlowStep, incoming: bool) -> &[usize] {
        self.neighbors
            .get(&(slot, step.edge.clone(), incoming))
            .map_or(&[], Vec::as_slice)
    }
}

/// Evaluate flow rules for one bundle view and route their findings.
pub(crate) fn validate_flow_rules(
    spec: &Spec,
    registry: &Registry,
    root: &Path,
    edges: &FlowEdges,
    report: &mut BundleReport,
) {
    let Some(model) = registry.traceability() else {
        return;
    };
    if model.flow_rules.is_empty() {
        return;
    }
    let severity = registry.grammar_severity();
    let excluded = ExcludeSet::compile_validated(&model.exclude);
    for rule in &model.flow_rules {
        let level = severity
            .get(&crate::grammar::severity_key("trace", &rule.check))
            .copied()
            .unwrap_or(GrammarSeverityLevel::Warning);
        if level == GrammarSeverityLevel::Off {
            continue;
        }
        for subject in rule
            .subject
            .iter()
            .flat_map(|archetype| spec.by_type_slots(archetype))
        {
            let doc = spec.doc(subject);
            if excluded.excludes(root, &doc.path) {
                continue;
            }
            if let Some(matched) = evaluate_rule(spec, edges, rule, subject) {
                let message = denial_message(spec, model, rule, subject, matched);
                report.route(
                    severity,
                    GrammarSeverity::Warning,
                    BundleFinding::in_pack(
                        pack::TRACE,
                        rule.check.clone(),
                        doc.path.clone(),
                        message,
                    ),
                );
            }
        }
    }
}

/// Construct the serialized report for a full corpus, or `None` when no rules
/// were declared so existing relation reports remain byte-identical.
pub(crate) fn flow_report(
    corpus: &Spec,
    registry: &Registry,
    edges: &FlowEdges,
    roots: &[&Path],
) -> Option<FlowReport> {
    let model = registry.traceability()?;
    if model.flow_rules.is_empty() {
        return None;
    }

    let rules: Vec<&FlowRule> = model.flow_rules.iter().collect();
    let rule_summaries = rules
        .iter()
        .map(|rule| FlowRuleSummary {
            name: rule.name.clone(),
            check: rule.check.clone(),
            subject: rule.subject.clone(),
            match_conditions: rule.match_conditions.clone(),
            unless_any: rule.unless_any.clone(),
            remediations: rule.remediations.clone(),
            rationale: rule.rationale.clone(),
        })
        .collect();

    let mut field_reads: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut edge_reads: FlowEdgeReads = BTreeMap::new();
    for rule in &rules {
        for condition in rule.match_conditions.iter().chain(&rule.unless_any) {
            for (archetypes, steps) in condition_paths(rule, condition) {
                for archetype in &archetypes {
                    field_reads
                        .entry(archetype.clone())
                        .or_default()
                        .insert(condition.field.clone());
                }
                for step in steps {
                    for archetype in &step.0 {
                        edge_reads.entry(archetype.clone()).or_default().insert((
                            step.1.edge.clone(),
                            match step.1.direction {
                                EdgeSide::Outgoing => "outgoing".to_string(),
                                EdgeSide::Incoming => "incoming".to_string(),
                            },
                            step.1.to.clone(),
                        ));
                    }
                }
            }
        }
    }

    let archetypes: BTreeSet<String> = field_reads
        .keys()
        .chain(edge_reads.keys())
        .cloned()
        .collect();
    let mut facts = BTreeMap::new();
    let mut vocabularies = BTreeMap::new();
    for archetype in archetypes {
        if let Some(fields) = field_reads.get(&archetype) {
            if let Some(compiled) = registry.archetype(&archetype) {
                if let Some(schema) = compiled.frontmatter_schema.as_deref() {
                    let enums: BTreeMap<String, Vec<String>> = fields
                        .iter()
                        .filter_map(|field| {
                            super::vocabulary_coverage::enum_at(schema, field)
                                .map(|values| (field.clone(), values))
                        })
                        .collect();
                    if !enums.is_empty() {
                        vocabularies.insert(archetype.clone(), enums);
                    }
                }
            }
        }
        let slots = corpus.corpus_type_slots(&archetype);
        let mut rows: Vec<(String, Option<String>, FlowFact)> = slots
            .into_iter()
            .map(|slot| {
                let doc = corpus.doc(slot);
                let mut fields = BTreeMap::new();
                if let Some(reads) = field_reads.get(&archetype) {
                    for field in reads {
                        if let Some(value) = doc.frontmatter().and_then(|fm| fm.get(field)) {
                            fields.insert(field.clone(), value.clone());
                        }
                    }
                }
                let fact_edges = edge_reads
                    .get(&archetype)
                    .map(|reads| fact_edges_for(corpus, edges, slot, reads))
                    .unwrap_or_default();
                let repo = corpus
                    .member_identity(corpus.member_of(slot))
                    .map(str::to_string);
                let id = artifact_key(doc);
                (
                    repo.clone().unwrap_or_default(),
                    Some(id.clone()),
                    FlowFact {
                        id,
                        repo,
                        path: super::declared_tables::relative_path(
                            roots
                                .get(corpus.member_of(slot))
                                .copied()
                                .unwrap_or(Path::new("")),
                            &doc.path,
                        ),
                        fields,
                        edges: fact_edges,
                    },
                )
            })
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        facts.insert(
            archetype,
            rows.into_iter().map(|(_, _, fact)| fact).collect(),
        );
    }

    let mut denials = Vec::new();
    let excluded = ExcludeSet::compile_validated(&model.exclude);
    for rule in &rules {
        let level = registry
            .grammar_severity()
            .get(&crate::grammar::severity_key("trace", &rule.check))
            .copied()
            .unwrap_or(GrammarSeverityLevel::Warning);
        let severity = match level {
            GrammarSeverityLevel::Off => continue,
            GrammarSeverityLevel::Warning => GrammarSeverity::Warning,
            GrammarSeverityLevel::Error => GrammarSeverity::Error,
        };
        for subject in rule
            .subject
            .iter()
            .flat_map(|archetype| corpus.by_type_slots(archetype))
        {
            let subject_doc = corpus.doc(subject);
            let Some(root) = roots.get(corpus.member_of(subject)) else {
                continue;
            };
            if excluded.excludes(root, &subject_doc.path) {
                continue;
            }
            let Some(matched) = evaluate_rule(corpus, edges, rule, subject) else {
                continue;
            };
            let groups = group_matched(corpus, matched);
            denials.push(FlowDenial {
                rule: rule.name.clone(),
                check: rule.check.clone(),
                subject: subject_ref(corpus, subject),
                matched: groups,
                remediations: denial_remediations(model, rule),
                rationale: rule.rationale.clone(),
                severity: severity.as_str(),
            });
        }
    }
    denials.sort_by(|left, right| {
        left.check
            .cmp(&right.check)
            .then(left.subject.repo.cmp(&right.subject.repo))
            .then(left.subject.id.cmp(&right.subject.id))
            .then(left.rule.cmp(&right.rule))
    });

    Some(FlowReport {
        schema: "quire.flow-report/v1",
        rules: rule_summaries,
        vocabularies,
        facts,
        denials,
        remediations: model.remediations.clone(),
    })
}

fn fact_edges_for(
    corpus: &Spec,
    edges: &FlowEdges,
    slot: usize,
    reads: &BTreeSet<(String, String, Vec<String>)>,
) -> Vec<FlowFactEdge> {
    let mut facts = BTreeSet::new();
    for (edge, direction, to) in reads {
        let incoming = direction == "incoming";
        for target in edges
            .neighbors
            .get(&(slot, edge.clone(), incoming))
            .into_iter()
            .flatten()
        {
            let doc = corpus.doc(*target);
            let Some(archetype) = doc.concept_type() else {
                continue;
            };
            if !to.iter().any(|name| name == archetype) {
                continue;
            }
            facts.insert(FlowFactEdge {
                edge: edge.clone(),
                direction: if incoming { "incoming" } else { "outgoing" },
                target: FlowDocumentRef {
                    id: artifact_key(doc),
                    repo: corpus
                        .member_identity(corpus.member_of(*target))
                        .map(str::to_string),
                },
                archetype: archetype.to_string(),
            });
        }
    }
    facts.into_iter().collect()
}

fn evaluate_rule(
    spec: &Spec,
    edges: &FlowEdges,
    rule: &FlowRule,
    subject: usize,
) -> Option<BTreeSet<usize>> {
    let mut matched = BTreeSet::new();
    for condition in &rule.match_conditions {
        let (holds, documents) = evaluate_condition(spec, edges, subject, condition);
        if !holds {
            return None;
        }
        matched.extend(documents);
    }
    if rule
        .unless_any
        .iter()
        .any(|condition| evaluate_condition(spec, edges, subject, condition).0)
    {
        return None;
    }
    Some(matched)
}

fn evaluate_condition(
    spec: &Spec,
    edges: &FlowEdges,
    subject: usize,
    condition: &FlowCondition,
) -> (bool, BTreeSet<usize>) {
    let mut endpoints = BTreeSet::new();
    for path in &condition.paths {
        let mut current = BTreeSet::from([subject]);
        for step in path {
            let mut next = BTreeSet::new();
            let incoming = step.direction == crate::traceability::EdgeSide::Incoming;
            for slot in current {
                next.extend(
                    edges
                        .neighbors(slot, step, incoming)
                        .iter()
                        .copied()
                        .filter(|other| {
                            spec.doc(*other)
                                .concept_type()
                                .is_some_and(|kind| step.to.iter().any(|to| to == kind))
                        }),
                );
            }
            current = next;
        }
        endpoints.extend(current);
    }

    match &condition.test {
        FlowConditionTest::AnyOf(values) => {
            let accepted: BTreeSet<String> =
                values.iter().map(|v| fold_selector_value(v)).collect();
            let matched: BTreeSet<usize> = endpoints
                .into_iter()
                .filter(|slot| field_matches(spec.doc(*slot), &condition.field, &accepted))
                .collect();
            (!matched.is_empty(), matched)
        }
        FlowConditionTest::Present => {
            let matched: BTreeSet<usize> = endpoints
                .into_iter()
                .filter(|slot| {
                    spec.doc(*slot)
                        .frontmatter()
                        .is_some_and(|fm| fm.contains_key(&condition.field))
                })
                .collect();
            (!matched.is_empty(), matched)
        }
        FlowConditionTest::Absent => {
            let present = endpoints.iter().any(|slot| {
                spec.doc(*slot)
                    .frontmatter()
                    .is_some_and(|fm| fm.contains_key(&condition.field))
            });
            (!present, BTreeSet::new())
        }
    }
}

fn field_matches(doc: &LoadedDocument, field: &str, accepted: &BTreeSet<String>) -> bool {
    let Some(value) = doc.frontmatter().and_then(|fm| fm.get(field)) else {
        return false;
    };
    match value {
        Value::String(text) => accepted.contains(&fold_selector_value(text)),
        Value::Array(items) => items.iter().any(|item| {
            item.as_str()
                .is_some_and(|text| accepted.contains(&fold_selector_value(text)))
        }),
        _ => false,
    }
}

fn group_matched(spec: &Spec, matched: BTreeSet<usize>) -> BTreeMap<String, Vec<FlowDocumentRef>> {
    let mut out: BTreeMap<String, Vec<FlowDocumentRef>> = BTreeMap::new();
    for slot in matched {
        let Some(archetype) = spec.doc(slot).concept_type() else {
            continue;
        };
        out.entry(archetype.to_string())
            .or_default()
            .push(document_ref(spec, slot));
    }
    for docs in out.values_mut() {
        docs.sort();
        docs.dedup();
    }
    out
}

fn document_ref(spec: &Spec, slot: usize) -> FlowDocumentRef {
    FlowDocumentRef {
        id: artifact_key(spec.doc(slot)),
        repo: spec
            .member_identity(spec.member_of(slot))
            .map(str::to_string),
    }
}

fn subject_ref(spec: &Spec, slot: usize) -> FlowSubjectRef {
    FlowSubjectRef {
        id: artifact_key(spec.doc(slot)),
        repo: spec
            .member_identity(spec.member_of(slot))
            .map(str::to_string),
        archetype: spec
            .doc(slot)
            .concept_type()
            .unwrap_or_default()
            .to_string(),
    }
}

fn denial_message(
    spec: &Spec,
    model: &crate::traceability::TraceabilityModel,
    rule: &FlowRule,
    subject: usize,
    matched: BTreeSet<usize>,
) -> String {
    let subject = document_ref(spec, subject);
    let id = match subject.repo {
        Some(repo) => format!("{repo}::{}", subject.id),
        None => subject.id,
    };
    let matched = group_matched(spec, matched)
        .into_iter()
        .map(|(archetype, documents)| {
            let ids = documents
                .into_iter()
                .map(|document| match document.repo {
                    Some(repo) => format!("{repo}::{}", document.id),
                    None => document.id,
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{archetype}: [{ids}]")
        })
        .collect::<Vec<_>>()
        .join("; ");
    if rule.remediations.is_empty() {
        format!(
            "flow rule '{}' denied '{id}': {} [matched: {matched}]",
            rule.name, rule.rationale,
        )
    } else {
        format!(
            "flow rule '{}' denied '{id}': {} [matched: {matched}] [remediations: {}]",
            rule.name,
            rule.rationale,
            serde_json::to_string(&denial_remediations(model, rule))
                .expect("flow remediation report types serialize")
        )
    }
}

fn denial_remediations(
    model: &crate::traceability::TraceabilityModel,
    rule: &FlowRule,
) -> Vec<FlowDenialRemediation> {
    rule.remediations
        .iter()
        .map(|name| {
            let remediation = model
                .remediations
                .iter()
                .find(|remediation| remediation.name == *name)
                .expect("validated flow rule remediation names resolve in the registry");
            FlowDenialRemediation {
                name: remediation.name.clone(),
                description: remediation.description.clone(),
                change: FlowRemediationChange {
                    adds: remediation.adds.clone(),
                    sets: remediation.sets.clone(),
                    removes: remediation.removes.clone(),
                },
            }
        })
        .collect()
}

/// Yield endpoint archetypes and the archetype at each step's source.
fn condition_paths<'a>(rule: &'a FlowRule, condition: &'a FlowCondition) -> ConditionPathReads<'a> {
    condition
        .paths
        .iter()
        .map(|path| {
            let mut current = rule.subject.clone();
            let mut steps = Vec::new();
            for step in path {
                steps.push((current, step));
                current = step.to.clone();
            }
            (current, steps)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::traceability::{FlowCondition, FlowRule};

    use super::{evaluate_rule, FlowEdges};

    #[test]
    #[ix_trace_rs::trace("FR-096-AC-11", "FR-096-AC-21")]
    fn any_of_denial_names_only_endpoint_documents_that_match_authored_values() {
        let root = tempfile::tempdir().expect("tempdir");
        let docs = root.path().join("spec");
        fs::create_dir_all(&docs).expect("spec dir");
        fs::write(
            docs.join("subject.md"),
            "---\nid: A-1\ntype: alpha\n---\n# Subject\n",
        )
        .expect("subject");
        fs::write(
            docs.join("matching.md"),
            "---\nid: B-1\ntype: beta\ncategory: ' Health '\n---\n# Matching\n",
        )
        .expect("matching endpoint");
        fs::write(
            docs.join("other.md"),
            "---\nid: B-2\ntype: beta\ncategory: personal\n---\n# Other\n",
        )
        .expect("other endpoint");

        let spec = crate::corpus::Spec::from_path(&docs);
        let subject = spec.by_type_slots("alpha")[0];
        let endpoints = spec.by_type_slots("beta");
        let rule = FlowRule {
            name: "check-category".into(),
            subject: vec!["alpha".into()],
            match_conditions: vec![
                serde_yaml::from_str::<FlowCondition>(
                    "path:\n  - edge: links\n    direction: outgoing\n    to: [beta]\nfield: category\nany_of: [health]\n",
                )
                .expect("condition"),
            ],
            unless_any: Vec::new(),
            check: "category-crossing".into(),
            remediations: Vec::new(),
            rationale: "The category needs a control.".into(),
        };
        let mut edges = FlowEdges::default();
        edges
            .neighbors
            .insert((subject, "links".into(), false), endpoints);

        let matched = evaluate_rule(&spec, &edges, &rule, subject).expect("denial");
        assert_eq!(matched.len(), 1);
        assert_eq!(
            spec.doc(*matched.iter().next().unwrap())
                .frontmatter()
                .unwrap()["id"],
            "B-1"
        );
    }
}
