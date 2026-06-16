//! Base "concept" frontmatter schema (OKF) — the contract every authored
//! document satisfies *before* archetype routing.
//!
//! OKF makes `type` the one required frontmatter key. Selecting which
//! archetype validates a document is code (read the discriminator via
//! [`crate::query::concept_type`]); *requiring* it — and typing the two
//! optional OKF fields `description`/`tags` — is schema. This module owns
//! that shared base schema so the requirement is enforced uniformly across
//! every surface and every module, with no per-module schema duplication.
//!
//! - `type` — required, non-empty string.
//! - `description` — optional string.
//! - `tags` — optional array of strings.
//!
//! `additionalProperties` is left open (the archetype-specific schema, run
//! afterwards, owns the rest of the frontmatter). A violation here is a
//! [`ValidationError`] with reason [`ValidationReason::Frontmatter`] — never
//! a soft warning and never a CLI bail.

use std::sync::OnceLock;

use jsonschema::JSONSchema;
use serde_json::{json, Map, Value};

use crate::loader::compile::compile_schema;
use crate::validate_document::{ValidationError, ValidationReason};

/// Frontmatter `properties` shared by both the required and shape schemas.
fn concept_properties() -> Value {
    json!({
        "type": { "type": "string", "minLength": 1 },
        "description": { "type": "string" },
        "tags": { "type": "array", "items": { "type": "string" } }
    })
}

/// The base concept frontmatter schema — `type` **required** + non-empty,
/// optional typed `description`/`tags`. This is the OKF contract enforced
/// where the document is routed *by* its `type` (corpus / bundle).
pub fn base_concept_schema() -> Value {
    json!({
        "type": "object",
        "required": ["type"],
        "properties": concept_properties()
    })
}

/// The concept **shape** schema — the same field typing, but `type` is
/// *not* required. Used on the per-document path ([`crate::validate_document`])
/// where an explicit `--archetype` override may legitimately validate a
/// typeless document (FR-004-AC-5): typing is still checked, presence is not.
fn concept_shape_schema() -> Value {
    json!({
        "type": "object",
        "properties": concept_properties()
    })
}

fn validator_for(
    schema_fn: fn() -> Value,
    cell: &'static OnceLock<JSONSchema>,
) -> &'static JSONSchema {
    cell.get_or_init(|| compile_schema(&schema_fn()).expect("concept schema is valid"))
}

fn base_validator() -> &'static JSONSchema {
    static VALIDATOR: OnceLock<JSONSchema> = OnceLock::new();
    validator_for(base_concept_schema, &VALIDATOR)
}

fn shape_validator() -> &'static JSONSchema {
    static VALIDATOR: OnceLock<JSONSchema> = OnceLock::new();
    validator_for(concept_shape_schema, &VALIDATOR)
}

fn run(validator: &'static JSONSchema, frontmatter: &Map<String, Value>) -> Vec<ValidationError> {
    let value = Value::Object(frontmatter.clone());
    let errors: Vec<ValidationError> = match validator.validate(&value) {
        Ok(()) => Vec::new(),
        Err(violations) => violations
            .map(|v| ValidationError {
                message: format!(
                    "frontmatter: {v} (at {})",
                    dotted_path(&v.instance_path.to_string())
                ),
                line: None,
                reason: ValidationReason::Frontmatter,
            })
            .collect(),
    };
    errors
}

/// Validate frontmatter against the base concept schema (required `type`).
///
/// Returns one [`ValidationError`] per violation (missing/empty `type`,
/// mistyped `description`/`tags`). Used where routing depends on `type`.
pub fn validate_base_concept(frontmatter: &Map<String, Value>) -> Vec<ValidationError> {
    run(base_validator(), frontmatter)
}

/// Validate frontmatter against the concept **shape** schema (typing only,
/// `type` not required). Used on the per-document validation path so a
/// `--archetype`-overridden typeless document still has its `description`/
/// `tags` typed correctly.
pub fn validate_concept_shape(frontmatter: &Map<String, Value>) -> Vec<ValidationError> {
    run(shape_validator(), frontmatter)
}

/// Render a JSON Pointer instance path as a dotted field path, matching the
/// archetype frontmatter-validator surface in
/// [`crate::validate_document`].
fn dotted_path(ptr: &str) -> String {
    if ptr.is_empty() {
        "<frontmatter>".to_string()
    } else {
        ptr.trim_start_matches('/').replace('/', ".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fm(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn accepts_minimal_typed_concept() {
        let errors = validate_base_concept(&fm(json!({ "type": "FR" })));
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn accepts_optional_description_and_tags() {
        let errors = validate_base_concept(&fm(json!({
            "type": "FR",
            "description": "a thing",
            "tags": ["a", "b"]
        })));
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn rejects_missing_type() {
        let errors = validate_base_concept(&fm(json!({ "id": "FR-001" })));
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].reason, ValidationReason::Frontmatter);
        assert!(errors[0].message.contains("type"));
    }

    #[test]
    fn rejects_empty_type() {
        let errors = validate_base_concept(&fm(json!({ "type": "" })));
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].reason, ValidationReason::Frontmatter);
    }

    #[test]
    fn rejects_mistyped_description() {
        let errors = validate_base_concept(&fm(json!({ "type": "FR", "description": 7 })));
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("description"));
    }

    #[test]
    fn rejects_mistyped_tags() {
        let errors = validate_base_concept(&fm(json!({ "type": "FR", "tags": "not-an-array" })));
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("tags"));
    }

    #[test]
    fn rejects_non_string_tag_item() {
        let errors = validate_base_concept(&fm(json!({ "type": "FR", "tags": ["ok", 3] })));
        assert_eq!(errors.len(), 1);
    }
}
