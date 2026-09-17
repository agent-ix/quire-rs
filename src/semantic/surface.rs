//! The one additive `semantic` record (FR-072): FR-070/FR-071 outcomes with
//! per-kind availability and `lossy`, ordered diagnostics, and the published
//! `semantic-v1` shape.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::extract::dsl::{EdgeTarget, ExtractionDsl};
use crate::extract::locator::{Locator, LocatorPrimitive};

use super::clauses::{extract_clauses, extract_operations, ClauseRef, OperationDecl};
use super::context::SemanticContext;
use super::decl::FieldDecl;
use super::model::{
    extract_model, failed, model_availability, ModelDeclarations, ModelRefs, ModelSource,
};
use super::properties::{extract_fields, FieldsForm};
use super::relations::{extract_relations, ExtractedRelation, RelationDecl, RelationSource};
use super::{AvailabilityState, KindAvailability, SemanticDiagnostic};

pub const SEMANTIC_FORMAT_VERSION: u64 = 1;
pub const SEMANTIC_V1_SCHEMA: &str = include_str!("../../schemas/output/semantic-v1.schema.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Availability {
    pub fields: KindAvailability,
    pub clauses: KindAvailability,
    pub operations: KindAvailability,
    /// FR-075 frontmatter and table features; present when any is declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<KindAvailability>,
    /// FR-076 relationships; present when the module names `relationships`
    /// or the artifact holds a relationship-shaped table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relations: Option<KindAvailability>,
}

/// The FR-072 record. Optional keys are skipped when absent so the shape
/// is the published one byte for byte.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticExtraction {
    pub format_version: u64,
    pub contract_version: String,
    pub semantic_core: String,
    pub package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldDecl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_form: Option<FieldsForm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clauses: Option<Vec<ClauseRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clause_text: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<Vec<OperationDecl>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelDeclarations>,
    /// FR-076 `RelationDecl[]`, present exactly when `availability.relations`
    /// is `available`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<RelationDecl>>,
    /// The `name` and `sourceSpan` of each `relations` element, at its index
    /// (the semantic-core 0.2.0 carrier, `agent-ix/filament-core-data#155`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation_sources: Option<Vec<RelationSource>>,
    pub availability: Availability,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

/// Which sections the module's `body_extraction` marks `required`: the
/// `missing` state applies to those (FR-072 Behavior).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequiredSections {
    pub properties: bool,
    pub invariants: bool,
    pub operations: bool,
}

impl RequiredSections {
    /// Scan a typed extraction DSL for required locators under the three
    /// headings: every primitive of every locator in `match`, `per_match`,
    /// and `emit_edges`, including each member of a fallback chain.
    pub fn from_extraction(dsl: &ExtractionDsl) -> Self {
        let pattern = &dsl.yield_pattern;
        let edge_locators = dsl
            .emit_edges
            .iter()
            .flatten()
            .filter_map(|edge| match &edge.target {
                EdgeTarget::Locator(locator) => Some(locator),
                EdgeTarget::Static(_) => None,
            });
        let primitives = pattern
            .r#match
            .iter()
            .chain(pattern.per_match.iter())
            .flat_map(|map| map.values())
            .chain(edge_locators)
            .flat_map(|locator| match locator {
                Locator::Primitive(p) => std::slice::from_ref(p),
                Locator::Fallback(chain) => chain.as_slice(),
            });
        let mut out = Self::default();
        for primitive in primitives {
            let (heading, required) = match primitive {
                LocatorPrimitive::SectionBody {
                    after_heading,
                    required,
                    ..
                } => (Some(after_heading.as_str()), *required),
                LocatorPrimitive::CodeBlock {
                    under_section,
                    required,
                    ..
                }
                | LocatorPrimitive::TableRow {
                    under_section,
                    required,
                    ..
                }
                | LocatorPrimitive::ListItem {
                    under_section,
                    required,
                    ..
                } => (under_section.as_deref(), *required),
                LocatorPrimitive::FrontmatterField { .. } | LocatorPrimitive::Heading { .. } => {
                    (None, false)
                }
            };
            match (heading, required) {
                (Some("Properties"), true) => out.properties = true,
                (Some("Invariants"), true) => out.invariants = true,
                (Some("Operations"), true) => out.operations = true,
                _ => {}
            }
        }
        out
    }

    /// [`RequiredSections::from_extraction`] over an extraction DSL given as
    /// JSON. JSON that is not a valid extraction DSL marks no section
    /// required.
    pub fn from_dsl(dsl: &Value) -> Self {
        serde_json::from_value::<ExtractionDsl>(dsl.clone())
            .map(|dsl| Self::from_extraction(&dsl))
            .unwrap_or_default()
    }
}

/// Run FR-070, FR-071, FR-075, and FR-076 over one document and assemble
/// the record.
pub fn extract_semantic(
    raw: &str,
    ctx: &SemanticContext,
    schema_digest: Option<&str>,
    required: &RequiredSections,
) -> SemanticExtraction {
    let declared_lossy = ctx.module.compatibility_posture == "declared-lossy";
    let fields = extract_fields(raw, ctx);
    let clauses = extract_clauses(raw, ctx);
    let operations = extract_operations(raw, ctx, clauses.clauses.as_deref().unwrap_or(&[]));
    let clause_ids: Option<Vec<String>> =
        (clauses.availability.state != AvailabilityState::Unavailable).then(|| {
            clauses
                .clauses
                .iter()
                .flatten()
                .map(|c| c.clause_id.clone())
                .collect()
        });
    let operation_names: Option<Vec<String>> =
        (operations.availability.state != AvailabilityState::Unavailable).then(|| {
            operations
                .operations
                .iter()
                .flatten()
                .map(|o| o.name.clone())
                .collect()
        });
    let model_outcome = extract_model(
        raw,
        ctx,
        ModelRefs {
            clause_ids: clause_ids.as_deref(),
            operation_names: operation_names.as_deref(),
        },
    );

    let relations = extract_relations(raw, ctx);

    let mut diagnostics: Vec<SemanticDiagnostic> = Vec::new();
    diagnostics.extend(fields.diagnostics.iter().cloned());
    diagnostics.extend(clauses.diagnostics.iter().cloned());
    diagnostics.extend(operations.diagnostics.iter().cloned());
    diagnostics.extend(model_outcome.diagnostics.iter().cloned());
    diagnostics.extend(relations.diagnostics);
    diagnostics.sort_by(|a, b| {
        (a.line.unwrap_or(0), a.column.unwrap_or(0), a.code.as_str()).cmp(&(
            b.line.unwrap_or(0),
            b.column.unwrap_or(0),
            b.code.as_str(),
        ))
    });

    let missing = |mut a: KindAvailability, required: bool, what: &str| {
        if required && a.state == AvailabilityState::NotApplicable {
            a = KindAvailability::missing(format!(
                "`## {what}` is required by the module's body_extraction and absent"
            ));
        }
        if declared_lossy {
            a.lossy = true;
        }
        a
    };
    let availability = Availability {
        fields: missing(
            fields.availability.clone(),
            required.properties,
            "Properties",
        ),
        clauses: missing(
            clauses.availability.clone(),
            required.invariants,
            "Invariants",
        ),
        operations: missing(
            operations.availability.clone(),
            required.operations,
            "Operations",
        ),
        model: model_availability(&[
            model_outcome.source(),
            ModelSource {
                declared: fields.model_declared,
                failed: failed(&fields.availability),
                lossy: false,
                diagnostics: &fields.diagnostics,
            },
            ModelSource {
                declared: operations.model_declared,
                failed: failed(&operations.availability),
                lossy: false,
                diagnostics: &operations.diagnostics,
            },
        ])
        .map(|mut a| {
            if declared_lossy {
                a.lossy = true;
            }
            a
        }),
        relations: relations.availability.map(|mut a| {
            if declared_lossy {
                a.lossy = true;
            }
            a
        }),
    };
    let (relation_decls, relation_sources) =
        relations.relations.map(ExtractedRelation::split).unzip();
    // `model` is carried exactly when `availability.model` is available.
    let model = availability
        .model
        .as_ref()
        .is_some_and(|a| a.state == AvailabilityState::Available)
        .then(|| {
            let mut model = model_outcome.model;
            model.field_features =
                (!fields.field_features.is_empty()).then_some(fields.field_features);
            model.operation_frames = (!operations.frames.is_empty()).then_some(operations.frames);
            model
        });
    SemanticExtraction {
        format_version: SEMANTIC_FORMAT_VERSION,
        contract_version: ctx.module.contract_version.clone(),
        semantic_core: ctx.module.semantic_core.clone(),
        package: ctx.module.package.clone(),
        schema_digest: schema_digest.map(str::to_string),
        fields: fields.fields,
        fields_form: fields.form,
        clauses: clauses.clauses,
        clause_text: if clauses.clause_text.is_empty()
            && availability.clauses.state != AvailabilityState::Available
        {
            None
        } else {
            Some(clauses.clause_text)
        },
        operations: operations.operations,
        model,
        relations: relation_decls,
        relation_sources,
        availability,
        diagnostics,
    }
}

impl SemanticExtraction {
    /// The record as the value a data schema validates: the declaration
    /// arrays only (what `Entity.json` and its kin describe).
    pub fn declaration_record(&self) -> Value {
        let mut map = serde_json::Map::new();
        if let Some(f) = &self.fields {
            map.insert(
                "fields".into(),
                serde_json::to_value(f).unwrap_or(Value::Null),
            );
        }
        if let Some(c) = &self.clauses {
            map.insert(
                "clauses".into(),
                serde_json::to_value(c).unwrap_or(Value::Null),
            );
        }
        if let Some(r) = &self.relations {
            map.insert(
                "relations".into(),
                serde_json::to_value(r).unwrap_or(Value::Null),
            );
        }
        if let Some(o) = &self.operations {
            map.insert(
                "operations".into(),
                serde_json::to_value(o).unwrap_or(Value::Null),
            );
        }
        Value::Object(map)
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(SemanticDiagnostic::is_error)
    }
}
