//! `apply_patch` — merge then validate (FR-002).
//!
//! Merge-then-validate is load-bearing: a patch that looks valid in
//! isolation can still produce an invalid *merged* shape (e.g. a
//! patch wiping a required field). The pre-compiled validator from
//! the loader (Task 005) catches the merged result.

use jsonschema::error::ValidationErrorKind;
use jsonschema::ValidationError;
use serde_json::Value;

use crate::error::QuireError;
use crate::loader::compile::CompiledArchetype;
use crate::merge::deep_merge;

/// Merge `patch` onto `current`, validate the merged value against
/// the archetype's compiled JSON Schema, and return either the
/// validated merged value or the first `SchemaViolation`.
pub fn apply_patch(
    archetype: &CompiledArchetype,
    current: &Value,
    patch: &Value,
) -> Result<Value, QuireError> {
    let merged = deep_merge(current, patch);
    validate(archetype, &merged)?;
    Ok(merged)
}

/// Validate a fully-merged `data` value against the archetype's
/// compiled schema. Returns the *first* violation; consumers wanting
/// the full list can use [`validate_all`].
pub fn validate(archetype: &CompiledArchetype, data: &Value) -> Result<(), QuireError> {
    if let Err(mut errors) = archetype.validator.validate(data) {
        if let Some(first) = errors.next() {
            return Err(to_schema_violation(&archetype.name, &first));
        }
    }
    Ok(())
}

/// Validate `data` against a block-type's schema (INPUT.md block
/// model). v0.2: block_type maps 1:1 to archetype.
pub fn validate_block(block_type: &CompiledArchetype, data: &Value) -> Result<(), QuireError> {
    validate(block_type, data)
}

/// Validate `data` and collect every violation as a `Vec<QuireError>`.
/// Returns `Ok(())` when valid; `Err(Vec<QuireError>)` otherwise.
pub fn validate_all(archetype: &CompiledArchetype, data: &Value) -> Result<(), Vec<QuireError>> {
    if let Err(errors) = archetype.validator.validate(data) {
        let v: Vec<QuireError> = errors
            .map(|e| to_schema_violation(&archetype.name, &e))
            .collect();
        if !v.is_empty() {
            return Err(v);
        }
    }
    Ok(())
}

/// Map a `jsonschema::ValidationError` into the public
/// `QuireError::SchemaViolation` shape per NFR-005.
fn to_schema_violation(archetype: &str, err: &ValidationError<'_>) -> QuireError {
    let field_path = json_pointer_to_dotted(&err.instance_path.to_string());
    let expected = describe_expected(err);
    let observed = preview_value(&err.instance);
    QuireError::SchemaViolation {
        archetype: archetype.to_string(),
        field_path,
        expected,
        observed,
    }
}

/// Convert a JSON Pointer (`/relationships/0/target`) to dotted form
/// (`relationships[0].target`) per NFR-005 example. An empty pointer
/// (root) yields `"<root>"`.
fn json_pointer_to_dotted(ptr: &str) -> String {
    if ptr.is_empty() {
        return "<root>".to_string();
    }
    let mut out = String::with_capacity(ptr.len());
    let parts: Vec<&str> = ptr.split('/').filter(|s| !s.is_empty()).collect();
    for (i, part) in parts.iter().enumerate() {
        if let Ok(n) = part.parse::<usize>() {
            out.push('[');
            out.push_str(&n.to_string());
            out.push(']');
        } else {
            if i > 0 {
                out.push('.');
            }
            // Per RFC 6901, ~1 → '/' and ~0 → '~'. Decode.
            let decoded = part.replace("~1", "/").replace("~0", "~");
            out.push_str(&decoded);
        }
    }
    if out.is_empty() {
        "<root>".to_string()
    } else {
        out
    }
}

/// Build a stable, neutral "expected" description from a validator
/// error's kind. Avoids leaking the native debug form (NFR-005-AC-2).
fn describe_expected(err: &ValidationError<'_>) -> String {
    match &err.kind {
        ValidationErrorKind::Required { property } => {
            format!("required property {property}")
        }
        ValidationErrorKind::Type { kind } => format!("type {kind:?}"),
        ValidationErrorKind::Pattern { pattern } => format!("pattern {pattern}"),
        ValidationErrorKind::Enum { options } => format!("enum {options}"),
        ValidationErrorKind::MinLength { limit } => format!("min length {limit}"),
        ValidationErrorKind::MaxLength { limit } => format!("max length {limit}"),
        ValidationErrorKind::MinItems { limit } => format!("min items {limit}"),
        ValidationErrorKind::MaxItems { limit } => format!("max items {limit}"),
        ValidationErrorKind::Minimum { limit } => format!("minimum {limit}"),
        ValidationErrorKind::Maximum { limit } => format!("maximum {limit}"),
        ValidationErrorKind::AdditionalProperties { unexpected } => {
            format!("no additional properties (unexpected: {unexpected:?})")
        }
        ValidationErrorKind::OneOfMultipleValid => "exactly one oneOf branch".to_string(),
        ValidationErrorKind::OneOfNotValid => "exactly one oneOf branch".to_string(),
        ValidationErrorKind::AnyOf => "any of the listed schemas".to_string(),
        // Catch-all — every other variant gets a stable label without
        // leaking inner debug forms.
        _ => "schema constraint".to_string(),
    }
}

/// Render a JSON value as a compact preview suitable for the
/// `observed` slot. Truncation is the caller's responsibility (see
/// `error::format_violation`); this just gives a compact string.
fn preview_value(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{s}\""),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(_) | Value::Object(_) => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::compile::{compile_schema, CompiledArchetype};
    use proptest::prelude::*;
    use serde_json::json;
    use std::sync::Arc;

    fn archetype(schema: Value) -> CompiledArchetype {
        let validator = compile_schema(&schema).expect("compile");
        CompiledArchetype {
            name: "fr".into(),
            module: "test".into(),
            raw_schema: Arc::new(schema),
            validator: Arc::new(validator),
            template_path: None,
            template_name: None,
        }
    }

    // FR-002-AC-1
    #[test]
    fn merge_preserves_siblings_through_apply_patch() {
        let arch = archetype(json!({
            "type": "object",
            "required": ["title", "body"],
            "properties": {
                "title": {"type": "string", "minLength": 1},
                "body": {"type": "string"}
            }
        }));
        let current = json!({"title": "old", "body": "content"});
        let patch = json!({"title": "new"});
        let out = apply_patch(&arch, &current, &patch).expect("ok");
        assert_eq!(out, json!({"title": "new", "body": "content"}));
    }

    // FR-002-AC-2: merged-shape validation catches a patch that wipes
    // a required-by-minLength field.
    #[test]
    fn merged_shape_validation_rejects_emptied_required_field() {
        let arch = archetype(json!({
            "type": "object",
            "required": ["title"],
            "properties": {"title": {"type": "string", "minLength": 1}}
        }));
        let current = json!({"title": "valid"});
        let patch = json!({"title": ""});
        let err = apply_patch(&arch, &current, &patch).expect_err("violation");
        let s = err.to_string();
        assert!(s.contains("title"), "{s}");
        assert!(
            s.contains("min length 1") || s.contains("min length"),
            "{s}"
        );
    }

    // FR-002-AC-3
    #[test]
    fn additional_property_is_rejected_when_disallowed() {
        let arch = archetype(json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {"title": {"type": "string"}}
        }));
        let current = json!({"title": "x"});
        let patch = json!({"unknown": "y"});
        let err = apply_patch(&arch, &current, &patch).expect_err("violation");
        let s = err.to_string();
        assert!(s.contains("unknown") || s.contains("additional"), "{s}");
    }

    // FR-002-AC-6: $defs + recursive $ref compiles and validates.
    #[test]
    fn recursive_ref_through_defs_validates_tree() {
        let schema = json!({
            "$id": "https://test/recursive",
            "$defs": {
                "node": {
                    "type": "object",
                    "required": ["name"],
                    "properties": {
                        "name": {"type": "string"},
                        "children": {
                            "type": "array",
                            "items": {"$ref": "#/$defs/node"}
                        }
                    }
                }
            },
            "$ref": "#/$defs/node"
        });
        let arch = archetype(schema);
        let good = json!({"name": "root", "children": [{"name": "kid"}]});
        let bad = json!({"name": "root", "children": [{"missing": true}]});
        assert!(apply_patch(&arch, &json!({}), &good).is_ok());
        assert!(apply_patch(&arch, &json!({}), &bad).is_err());
    }

    #[test]
    fn json_pointer_to_dotted_handles_indices_and_keys() {
        assert_eq!(
            json_pointer_to_dotted("/relationships/0/target"),
            "relationships[0].target"
        );
        assert_eq!(json_pointer_to_dotted(""), "<root>");
        assert_eq!(json_pointer_to_dotted("/title"), "title");
        // RFC 6901 escapes.
        assert_eq!(json_pointer_to_dotted("/a~1b"), "a/b");
    }

    fn flexible_archetype() -> CompiledArchetype {
        // Schema accepts any object but pins one shape constraint
        // (`id` must be a string when present) so the validator
        // actually does work.
        archetype(json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"}
            }
        }))
    }

    /// Generate small arbitrary JSON values. Bounded depth + length
    /// to keep each proptest iteration cheap.
    fn any_json() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            any::<i64>().prop_map(|n| json!(n)),
            "[a-zA-Z0-9 _-]{0,16}".prop_map(Value::String),
        ];
        leaf.prop_recursive(4, 32, 8, |inner| {
            prop_oneof![
                proptest::collection::vec(inner.clone(), 0..8).prop_map(Value::Array),
                proptest::collection::vec(("[a-z]{1,6}", inner), 0..8).prop_map(|kvs| {
                    let mut m = serde_json::Map::new();
                    for (k, v) in kvs {
                        m.insert(k, v);
                    }
                    Value::Object(m)
                }),
            ]
        })
    }

    proptest! {
        // FR-002-AC-4: apply_patch never panics on arbitrary input.
        // Outcome may be Ok(_) or Err(SchemaViolation); both are fine
        // — the AC is "no panic".
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        #[test]
        fn apply_patch_never_panics(
            current in any_json(),
            patch   in any_json(),
        ) {
            let arch = flexible_archetype();
            let _ = apply_patch(&arch, &current, &patch);
        }
    }
}
