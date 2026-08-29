//! Rights-aware, domain-neutral clause sets.
//!
//! A clause set describes obligations without embedding any particular
//! publication or domain in the engine. Modules opt in with file references
//! from `manifest.yaml`; the loader verifies the declared content digest,
//! rights posture, internal references, and applicability vocabulary before a
//! set reaches the immutable registry.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: &str = "clause-set-v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClauseSetKey {
    pub authority: String,
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClauseForce {
    Mandatory,
    Recommended,
    Permitted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureRights {
    Original,
    CitationOnly,
    ExplicitlyCleared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextRights {
    None,
    Original,
    ExplicitlyCleared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClauseSetRights {
    pub structure: StructureRights,
    pub text: TextRights,
    #[serde(default)]
    pub clearance_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClauseSource {
    pub title: String,
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub official_url: Option<String>,
}

/// A dimension's values and optional ranked levels. Values in the same level
/// are equivalent for `at_least`; values absent from the order are
/// incomparable and therefore evaluate as unresolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassificationDimension {
    pub values: Vec<String>,
    #[serde(default)]
    pub order: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedOutput {
    pub kind: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ApplicabilityExpr {
    All {
        terms: Vec<ApplicabilityExpr>,
    },
    Any {
        terms: Vec<ApplicabilityExpr>,
    },
    Not {
        term: Box<ApplicabilityExpr>,
    },
    Eq {
        dimension: String,
        value: String,
    },
    In {
        dimension: String,
        values: Vec<String>,
    },
    AtLeast {
        dimension: String,
        value: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clause {
    pub id: String,
    pub force: ClauseForce,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub subjects: Vec<String>,
    #[serde(default)]
    pub obligated_actors: Vec<String>,
    #[serde(default)]
    pub approval_roles: Vec<String>,
    #[serde(default)]
    pub styles: BTreeMap<String, String>,
    #[serde(default)]
    pub applicability: Option<ApplicabilityExpr>,
    #[serde(default)]
    pub expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrosswalkRelation {
    Equivalent,
    Partial,
    Informative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClauseRef {
    pub set: ClauseSetKey,
    pub clause: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crosswalk {
    pub source_clause: String,
    pub target: ClauseRef,
    pub relation: CrosswalkRelation,
    #[serde(default)]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClauseSet {
    pub schema_version: String,
    pub authority: String,
    pub id: String,
    pub title: String,
    pub version: String,
    pub digest: String,
    pub rights: ClauseSetRights,
    #[serde(default)]
    pub source: Option<ClauseSource>,
    #[serde(default)]
    pub classification_dimensions: BTreeMap<String, ClassificationDimension>,
    #[serde(default)]
    pub output_catalog: BTreeMap<String, ExpectedOutput>,
    pub clauses: Vec<Clause>,
    #[serde(default)]
    pub crosswalks: Vec<Crosswalk>,
}

impl ClauseSet {
    pub fn key(&self) -> ClauseSetKey {
        ClauseSetKey {
            authority: self.authority.clone(),
            id: self.id.clone(),
            version: self.version.clone(),
        }
    }

    /// Digest of canonical JSON with the digest field blanked. This makes the
    /// declaration self-verifying without hashing its own hash.
    pub fn computed_digest(&self) -> String {
        let mut value = serde_json::to_value(self).expect("ClauseSet is serializable");
        value["digest"] = Value::String(String::new());
        let bytes = serde_json::to_vec(&value).expect("canonical ClauseSet JSON serializes");
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    pub fn validate(&self) -> Result<(), ClauseSetError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(ClauseSetError::Invalid(format!(
                "unsupported schemaVersion {:?}",
                self.schema_version
            )));
        }
        for (name, value) in [
            ("authority", &self.authority),
            ("id", &self.id),
            ("title", &self.title),
            ("version", &self.version),
        ] {
            if value.trim().is_empty() {
                return Err(ClauseSetError::Invalid(format!("{name} must not be empty")));
            }
        }
        if self.digest != self.computed_digest() {
            return Err(ClauseSetError::Invalid(
                "digest does not match canonical clause-set content".into(),
            ));
        }
        if (matches!(self.rights.structure, StructureRights::ExplicitlyCleared)
            || matches!(self.rights.text, TextRights::ExplicitlyCleared))
            && self
                .rights
                .clearance_ref
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
        {
            return Err(ClauseSetError::Invalid(
                "explicitly-cleared content requires clearance_ref".into(),
            ));
        }
        if self.rights.text == TextRights::None
            && self.clauses.iter().any(|clause| clause.text.is_some())
        {
            return Err(ClauseSetError::Invalid(
                "clause text is prohibited by the declared text rights".into(),
            ));
        }

        for (name, dimension) in &self.classification_dimensions {
            let values: BTreeSet<&str> = dimension.values.iter().map(String::as_str).collect();
            if name.trim().is_empty() || values.len() != dimension.values.len() || values.is_empty()
            {
                return Err(ClauseSetError::Invalid(format!(
                    "classification dimension {name:?} has empty or duplicate values"
                )));
            }
            let mut ordered = BTreeSet::new();
            for level in &dimension.order {
                if level.is_empty() {
                    return Err(ClauseSetError::Invalid(format!(
                        "classification dimension {name:?} has an empty order level"
                    )));
                }
                for value in level {
                    if !values.contains(value.as_str()) || !ordered.insert(value.as_str()) {
                        return Err(ClauseSetError::Invalid(format!(
                            "classification dimension {name:?} has an unknown or repeated ordered value {value:?}"
                        )));
                    }
                }
            }
        }

        let ids: BTreeSet<&str> = self
            .clauses
            .iter()
            .map(|clause| clause.id.as_str())
            .collect();
        if ids.len() != self.clauses.len() || ids.contains("") {
            return Err(ClauseSetError::Invalid(
                "clause ids must be non-empty and unique".into(),
            ));
        }
        for clause in &self.clauses {
            for output in &clause.expected_outputs {
                if !self.output_catalog.contains_key(output) {
                    return Err(ClauseSetError::Invalid(format!(
                        "clause {:?} references unknown expected output {output:?}",
                        clause.id
                    )));
                }
            }
            if let Some(expr) = &clause.applicability {
                validate_expr(expr, &self.classification_dimensions)?;
            }
        }
        let own_key = self.key();
        for crosswalk in &self.crosswalks {
            if !ids.contains(crosswalk.source_clause.as_str()) {
                return Err(ClauseSetError::Invalid(format!(
                    "crosswalk references unknown source clause {:?}",
                    crosswalk.source_clause
                )));
            }
            if crosswalk.target.set == own_key && !ids.contains(crosswalk.target.clause.as_str()) {
                return Err(ClauseSetError::Invalid(format!(
                    "crosswalk references unknown local target clause {:?}",
                    crosswalk.target.clause
                )));
            }
        }
        Ok(())
    }

    pub fn evaluate(&self, context: &BTreeMap<String, String>) -> ClauseBindingReport {
        let clauses = self
            .clauses
            .iter()
            .map(|clause| {
                let (outcome, reasons) = match &clause.applicability {
                    None => (BindingOutcome::Binding, Vec::new()),
                    Some(expr) => evaluate_expr(expr, context, &self.classification_dimensions),
                };
                ClauseBinding {
                    clause_id: clause.id.clone(),
                    force: clause.force.clone(),
                    outcome,
                    reasons,
                    expected_outputs: clause.expected_outputs.clone(),
                }
            })
            .collect();
        ClauseBindingReport {
            schema_version: "clause-binding-v1".into(),
            clause_set: self.key(),
            clause_set_digest: self.digest.clone(),
            context: context.clone(),
            clauses,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingOutcome {
    Binding,
    NotBinding,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingReason {
    pub code: String,
    #[serde(default)]
    pub dimension: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClauseBinding {
    pub clause_id: String,
    pub force: ClauseForce,
    pub outcome: BindingOutcome,
    pub reasons: Vec<BindingReason>,
    pub expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClauseBindingReport {
    pub schema_version: String,
    pub clause_set: ClauseSetKey,
    pub clause_set_digest: String,
    pub context: BTreeMap<String, String>,
    pub clauses: Vec<ClauseBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedClause {
    pub clause_id: String,
    pub before: Clause,
    pub after: Clause,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClauseSetDiff {
    pub schema_version: String,
    pub before: ClauseSetKey,
    pub before_digest: String,
    pub after: ClauseSetKey,
    pub after_digest: String,
    pub added: Vec<Clause>,
    pub removed: Vec<Clause>,
    pub changed: Vec<ChangedClause>,
}

pub fn diff_clause_sets(
    before: &ClauseSet,
    after: &ClauseSet,
) -> Result<ClauseSetDiff, ClauseSetError> {
    if before.authority != after.authority || before.id != after.id {
        return Err(ClauseSetError::Invalid(
            "clause-set diff requires matching authority and id".into(),
        ));
    }
    let before_by_id: BTreeMap<&str, &Clause> = before
        .clauses
        .iter()
        .map(|clause| (clause.id.as_str(), clause))
        .collect();
    let after_by_id: BTreeMap<&str, &Clause> = after
        .clauses
        .iter()
        .map(|clause| (clause.id.as_str(), clause))
        .collect();
    let added = after_by_id
        .iter()
        .filter(|(id, _)| !before_by_id.contains_key(**id))
        .map(|(_, clause)| (*clause).clone())
        .collect();
    let removed = before_by_id
        .iter()
        .filter(|(id, _)| !after_by_id.contains_key(**id))
        .map(|(_, clause)| (*clause).clone())
        .collect();
    let changed = before_by_id
        .iter()
        .filter_map(|(id, before_clause)| {
            let after_clause = after_by_id.get(id)?;
            (*before_clause != *after_clause).then(|| ChangedClause {
                clause_id: (*id).to_string(),
                before: (*before_clause).clone(),
                after: (*after_clause).clone(),
            })
        })
        .collect();
    Ok(ClauseSetDiff {
        schema_version: "clause-diff-v1".into(),
        before: before.key(),
        before_digest: before.digest.clone(),
        after: after.key(),
        after_digest: after.digest.clone(),
        added,
        removed,
        changed,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ClauseSetError {
    #[error("{0}")]
    Invalid(String),
    #[error("{path}: {reason}")]
    Read { path: String, reason: String },
}

pub(crate) fn load_clause_set(root: &Path, relative: &Path) -> Result<ClauseSet, ClauseSetError> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ClauseSetError::Invalid(format!(
            "clause-set path {:?} must stay inside its module",
            relative
        )));
    }
    let path = root.join(relative);
    let canonical_root = root.canonicalize().map_err(|error| ClauseSetError::Read {
        path: root.display().to_string(),
        reason: error.to_string(),
    })?;
    let canonical = path.canonicalize().map_err(|error| ClauseSetError::Read {
        path: path.display().to_string(),
        reason: error.to_string(),
    })?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ClauseSetError::Invalid(format!(
            "clause-set path {:?} escapes its module",
            relative
        )));
    }
    let bytes = std::fs::read(&canonical).map_err(|error| ClauseSetError::Read {
        path: canonical.display().to_string(),
        reason: error.to_string(),
    })?;
    let extension = canonical.extension().and_then(|value| value.to_str());
    let set: ClauseSet = if extension == Some("json") {
        serde_json::from_slice(&bytes).map_err(|error| ClauseSetError::Read {
            path: canonical.display().to_string(),
            reason: error.to_string(),
        })?
    } else {
        serde_yaml::from_slice(&bytes).map_err(|error| ClauseSetError::Read {
            path: canonical.display().to_string(),
            reason: error.to_string(),
        })?
    };
    set.validate()?;
    Ok(set)
}

fn validate_expr(
    expr: &ApplicabilityExpr,
    dimensions: &BTreeMap<String, ClassificationDimension>,
) -> Result<(), ClauseSetError> {
    match expr {
        ApplicabilityExpr::All { terms } | ApplicabilityExpr::Any { terms } => {
            if terms.is_empty() {
                return Err(ClauseSetError::Invalid(
                    "all/any applicability expressions require at least one term".into(),
                ));
            }
            for term in terms {
                validate_expr(term, dimensions)?;
            }
        }
        ApplicabilityExpr::Not { term } => validate_expr(term, dimensions)?,
        ApplicabilityExpr::Eq { dimension, value }
        | ApplicabilityExpr::AtLeast { dimension, value } => {
            validate_dimension_value(dimensions, dimension, value)?;
            if matches!(expr, ApplicabilityExpr::AtLeast { .. })
                && dimensions[dimension].order.is_empty()
            {
                return Err(ClauseSetError::Invalid(format!(
                    "at_least requires an order for dimension {dimension:?}"
                )));
            }
        }
        ApplicabilityExpr::In { dimension, values } => {
            if values.is_empty() {
                return Err(ClauseSetError::Invalid(
                    "in applicability expression requires at least one value".into(),
                ));
            }
            for value in values {
                validate_dimension_value(dimensions, dimension, value)?;
            }
        }
    }
    Ok(())
}

fn validate_dimension_value(
    dimensions: &BTreeMap<String, ClassificationDimension>,
    dimension: &str,
    value: &str,
) -> Result<(), ClauseSetError> {
    let Some(definition) = dimensions.get(dimension) else {
        return Err(ClauseSetError::Invalid(format!(
            "applicability references unknown dimension {dimension:?}"
        )));
    };
    if !definition.values.iter().any(|candidate| candidate == value) {
        return Err(ClauseSetError::Invalid(format!(
            "applicability references unknown {dimension:?} value {value:?}"
        )));
    }
    Ok(())
}

fn evaluate_expr(
    expr: &ApplicabilityExpr,
    context: &BTreeMap<String, String>,
    dimensions: &BTreeMap<String, ClassificationDimension>,
) -> (BindingOutcome, Vec<BindingReason>) {
    match expr {
        ApplicabilityExpr::All { terms } => combine_all(
            terms
                .iter()
                .map(|term| evaluate_expr(term, context, dimensions)),
        ),
        ApplicabilityExpr::Any { terms } => combine_any(
            terms
                .iter()
                .map(|term| evaluate_expr(term, context, dimensions)),
        ),
        ApplicabilityExpr::Not { term } => {
            let (outcome, reasons) = evaluate_expr(term, context, dimensions);
            let outcome = match outcome {
                BindingOutcome::Binding => BindingOutcome::NotBinding,
                BindingOutcome::NotBinding => BindingOutcome::Binding,
                BindingOutcome::Unresolved => BindingOutcome::Unresolved,
            };
            (outcome, reasons)
        }
        ApplicabilityExpr::Eq { dimension, value } => {
            match context_value(context, dimensions, dimension) {
                Ok(actual) if actual == value => (BindingOutcome::Binding, Vec::new()),
                Ok(_) => (BindingOutcome::NotBinding, Vec::new()),
                Err(reason) => (BindingOutcome::Unresolved, vec![reason]),
            }
        }
        ApplicabilityExpr::In { dimension, values } => {
            match context_value(context, dimensions, dimension) {
                Ok(actual) if values.contains(actual) => (BindingOutcome::Binding, Vec::new()),
                Ok(_) => (BindingOutcome::NotBinding, Vec::new()),
                Err(reason) => (BindingOutcome::Unresolved, vec![reason]),
            }
        }
        ApplicabilityExpr::AtLeast { dimension, value } => {
            let actual = match context_value(context, dimensions, dimension) {
                Ok(actual) => actual,
                Err(reason) => return (BindingOutcome::Unresolved, vec![reason]),
            };
            let order = &dimensions[dimension].order;
            let actual_rank = order.iter().position(|level| level.contains(actual));
            let threshold_rank = order.iter().position(|level| level.contains(value));
            match (actual_rank, threshold_rank) {
                (Some(actual), Some(threshold)) if actual >= threshold => {
                    (BindingOutcome::Binding, Vec::new())
                }
                (Some(_), Some(_)) => (BindingOutcome::NotBinding, Vec::new()),
                _ => (
                    BindingOutcome::Unresolved,
                    vec![BindingReason {
                        code: "incomparable_value".into(),
                        dimension: Some(dimension.clone()),
                        message: format!(
                            "context value and threshold are not comparable in {dimension}"
                        ),
                    }],
                ),
            }
        }
    }
}

fn context_value<'a>(
    context: &'a BTreeMap<String, String>,
    dimensions: &BTreeMap<String, ClassificationDimension>,
    dimension: &str,
) -> Result<&'a String, BindingReason> {
    let Some(actual) = context.get(dimension) else {
        return Err(BindingReason {
            code: "missing_dimension".into(),
            dimension: Some(dimension.into()),
            message: format!("context does not declare {dimension}"),
        });
    };
    if !dimensions[dimension].values.contains(actual) {
        return Err(BindingReason {
            code: "unknown_value".into(),
            dimension: Some(dimension.into()),
            message: format!("context value {actual:?} is not declared for {dimension}"),
        });
    }
    Ok(actual)
}

fn combine_all(
    evaluations: impl Iterator<Item = (BindingOutcome, Vec<BindingReason>)>,
) -> (BindingOutcome, Vec<BindingReason>) {
    let mut unresolved = Vec::new();
    for (outcome, reasons) in evaluations {
        match outcome {
            BindingOutcome::NotBinding => return (BindingOutcome::NotBinding, Vec::new()),
            BindingOutcome::Unresolved => unresolved.extend(reasons),
            BindingOutcome::Binding => {}
        }
    }
    if unresolved.is_empty() {
        (BindingOutcome::Binding, unresolved)
    } else {
        (BindingOutcome::Unresolved, unresolved)
    }
}

fn combine_any(
    evaluations: impl Iterator<Item = (BindingOutcome, Vec<BindingReason>)>,
) -> (BindingOutcome, Vec<BindingReason>) {
    let mut unresolved = Vec::new();
    for (outcome, reasons) in evaluations {
        match outcome {
            BindingOutcome::Binding => return (BindingOutcome::Binding, Vec::new()),
            BindingOutcome::Unresolved => unresolved.extend(reasons),
            BindingOutcome::NotBinding => {}
        }
    }
    if unresolved.is_empty() {
        (BindingOutcome::NotBinding, unresolved)
    } else {
        (BindingOutcome::Unresolved, unresolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    fn set(version: &str) -> ClauseSet {
        let mut set = ClauseSet {
            schema_version: SCHEMA_VERSION.into(),
            authority: "example.test".into(),
            id: "widget-assurance".into(),
            title: "Synthetic widget assurance rules".into(),
            version: version.into(),
            digest: String::new(),
            rights: ClauseSetRights {
                structure: StructureRights::Original,
                text: TextRights::Original,
                clearance_ref: None,
            },
            source: None,
            classification_dimensions: BTreeMap::from([(
                "impact".into(),
                ClassificationDimension {
                    values: vec![
                        "low".into(),
                        "medium".into(),
                        "high".into(),
                        "special".into(),
                    ],
                    order: vec![
                        vec!["low".into()],
                        vec!["medium".into()],
                        vec!["high".into()],
                    ],
                },
            )]),
            output_catalog: BTreeMap::from([(
                "test-result".into(),
                ExpectedOutput {
                    kind: "record".into(),
                    description: "A synthetic test result".into(),
                },
            )]),
            clauses: vec![Clause {
                id: "W-1".into(),
                force: ClauseForce::Mandatory,
                title: Some("Exercise material widgets".into()),
                text: Some("Exercise each material widget before release.".into()),
                subjects: vec!["widget".into()],
                obligated_actors: vec!["release-owner".into()],
                approval_roles: vec!["reviewer".into()],
                styles: BTreeMap::new(),
                applicability: Some(ApplicabilityExpr::AtLeast {
                    dimension: "impact".into(),
                    value: "medium".into(),
                }),
                expected_outputs: vec!["test-result".into()],
            }],
            crosswalks: Vec::new(),
        };
        set.digest = set.computed_digest();
        set
    }

    #[test]
    #[trace("TC-1084", "FR-067-AC-2", "FR-067-AC-3")]
    fn synthetic_set_validates_and_preserves_three_valued_applicability() {
        let set = set("1.0.0");
        set.validate().unwrap();
        let missing = set.evaluate(&BTreeMap::new());
        assert_eq!(missing.clauses[0].outcome, BindingOutcome::Unresolved);
        let low = set.evaluate(&BTreeMap::from([("impact".into(), "low".into())]));
        assert_eq!(low.clauses[0].outcome, BindingOutcome::NotBinding);
        let high = set.evaluate(&BTreeMap::from([("impact".into(), "high".into())]));
        assert_eq!(high.clauses[0].outcome, BindingOutcome::Binding);
        let incomparable = set.evaluate(&BTreeMap::from([("impact".into(), "special".into())]));
        assert_eq!(incomparable.clauses[0].outcome, BindingOutcome::Unresolved);
    }

    #[test]
    #[trace("TC-1085", "FR-067-AC-1", "FR-067-AC-6")]
    fn rights_and_digest_fail_closed() {
        let mut fixture = set("1.0.0");
        fixture.rights.text = TextRights::None;
        fixture.digest = fixture.computed_digest();
        assert!(fixture
            .validate()
            .unwrap_err()
            .to_string()
            .contains("text rights"));
        let mut changed = set("1.0.0");
        changed.title.push_str(" changed");
        assert!(changed
            .validate()
            .unwrap_err()
            .to_string()
            .contains("digest"));
    }

    #[test]
    #[trace("TC-1086", "FR-067-AC-4")]
    fn diff_reports_added_removed_and_changed_by_clause_id() {
        let before = set("1.0.0");
        let mut after = set("2.0.0");
        after.clauses[0].force = ClauseForce::Recommended;
        after.clauses.push(Clause {
            id: "W-2".into(),
            force: ClauseForce::Permitted,
            title: None,
            text: None,
            subjects: Vec::new(),
            obligated_actors: Vec::new(),
            approval_roles: Vec::new(),
            styles: BTreeMap::new(),
            applicability: None,
            expected_outputs: Vec::new(),
        });
        after.digest = after.computed_digest();
        let diff = diff_clause_sets(&before, &after).unwrap();
        assert_eq!(diff.added[0].id, "W-2");
        assert_eq!(diff.changed[0].clause_id, "W-1");
        assert!(diff.removed.is_empty());
    }

    #[test]
    #[trace("TC-1087", "FR-067-AC-1", "FR-067-AC-5")]
    fn module_registry_loads_referenced_set_and_output_schemas_accept_reports() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("clauses")).unwrap();
        std::fs::write(
            root.path().join("manifest.yaml"),
            "name: synthetic-clause-module\nclause_sets:\n  - clauses/widget.json\n",
        )
        .unwrap();
        let fixture = set("1.0.0");
        std::fs::write(
            root.path().join("clauses/widget.json"),
            serde_json::to_vec_pretty(&fixture).unwrap(),
        )
        .unwrap();

        let registry = crate::Registry::load_module(root.path()).unwrap();
        assert!(registry.failures().is_empty());
        let loaded = registry
            .clause_set("example.test", "widget-assurance", "1.0.0")
            .unwrap();
        let report = loaded.evaluate(&BTreeMap::from([("impact".into(), "high".into())]));

        let binding_schema: Value = serde_json::from_str(include_str!(
            "../schemas/output/clause-binding-v1.schema.json"
        ))
        .unwrap();
        let binding_validator = jsonschema::JSONSchema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .compile(&binding_schema)
            .unwrap();
        assert!(binding_validator
            .validate(&serde_json::to_value(report).unwrap())
            .is_ok());

        let diff = diff_clause_sets(loaded, &set("2.0.0")).unwrap();
        let diff_schema: Value =
            serde_json::from_str(include_str!("../schemas/output/clause-diff-v1.schema.json"))
                .unwrap();
        let diff_validator = jsonschema::JSONSchema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .compile(&diff_schema)
            .unwrap();
        assert!(diff_validator
            .validate(&serde_json::to_value(diff).unwrap())
            .is_ok());
    }
}
