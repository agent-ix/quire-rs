//! The one additive `semantic` record (FR-072): FR-070/FR-071 outcomes with
//! per-kind availability and `lossy`, ordered diagnostics, and the published
//! `semantic-v1` shape.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::clauses::{extract_clauses, extract_operations, ClauseRef, OperationDecl};
use super::context::SemanticContext;
use super::decl::FieldDecl;
use super::properties::{extract_fields, FieldsForm};
use super::{AvailabilityState, KindAvailability, SemanticDiagnostic};

pub const SEMANTIC_FORMAT_VERSION: u64 = 1;
pub const SEMANTIC_V1_SCHEMA: &str = include_str!("../../schemas/output/semantic-v1.schema.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Availability {
    pub fields: KindAvailability,
    pub clauses: KindAvailability,
    pub operations: KindAvailability,
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
    /// Scan an extraction DSL (as JSON) for required locators under the
    /// three headings.
    pub fn from_dsl(dsl: &Value) -> Self {
        let mut out = Self::default();
        fn walk(v: &Value, out: &mut RequiredSections) {
            match v {
                Value::Object(map) => {
                    let heading = map
                        .get("after_heading")
                        .or_else(|| map.get("under_section"))
                        .and_then(Value::as_str);
                    let required = map
                        .get("required")
                        .map_or(true, |r| r.as_bool().unwrap_or(true));
                    if let (Some(h), true) = (heading, required) {
                        match h {
                            "Properties" => out.properties = true,
                            "Invariants" => out.invariants = true,
                            "Operations" => out.operations = true,
                            _ => {}
                        }
                    }
                    map.values().for_each(|v| walk(v, out));
                }
                Value::Array(items) => items.iter().for_each(|v| walk(v, out)),
                _ => {}
            }
        }
        walk(dsl, &mut out);
        out
    }
}

/// Run FR-070 and FR-071 over one document and assemble the record.
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

    let mut diagnostics: Vec<SemanticDiagnostic> = Vec::new();
    diagnostics.extend(fields.diagnostics.iter().cloned());
    diagnostics.extend(clauses.diagnostics.iter().cloned());
    diagnostics.extend(operations.diagnostics.iter().cloned());
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
    };
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
