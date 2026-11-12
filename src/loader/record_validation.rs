//! Located record validation, load half (FR-069 "Located record validation").
//!
//! An archetype of a module with a `semantic` block may declare
//! `record_validation: { schema, locate_absence_at }`. This resolves the
//! declaration at load: the schema file under the reference-form rules of
//! `data_schema`, and the key under which an absence is located. Everything is
//! a module declaration; the engine knows no archetype, field or key name.

use std::path::PathBuf;
use std::sync::Arc;

use super::compile::{failure, CompiledRecordValidation};
use super::manifest::Archetype;
use crate::error::ArchetypeLoadFailure;
use crate::semantic::contract::DataSchemaRef;
use crate::semantic::resolver::resolve_reference_at;
use crate::semantic::{SchemaSource, SemanticFailure, SemanticModule};

/// The only keys a `record_validation` declaration carries.
const KEYS: [&str; 2] = ["schema", "locate_absence_at"];

/// Resolve `a.record_validation`, or `Ok(None)` when the archetype declares
/// none. Each refusal is a `semantic.invalid-value` (or the `data_schema`
/// resolution code) at the declaration's manifest path, in the
/// `<code>: <path>: <message>` reason form.
pub(super) fn resolve(
    module: &str,
    manifest_path: PathBuf,
    a: &Archetype,
    section: &str,
    semantic: Option<&SemanticModule>,
    source: &SchemaSource<'_>,
) -> Result<Option<CompiledRecordValidation>, ArchetypeLoadFailure> {
    let Some(declaration) = &a.record_validation else {
        return Ok(None);
    };
    let at = format!("{section}[{}].record_validation", a.name);
    let refuse = |path: String, message: String| {
        failure(
            module,
            &a.name,
            manifest_path.clone(),
            SemanticFailure::error("semantic.invalid-value", path, message).reason(),
        )
    };
    if declaration.is_null() {
        return Err(refuse(
            at,
            format!(
                "archetype `{}` declares record_validation as null; give a mapping of schema and locate_absence_at, or omit the key",
                a.name
            ),
        ));
    }
    let Some(semantic) = semantic else {
        return Err(refuse(
            at,
            "record_validation requires a semantic block".to_string(),
        ));
    };
    let Some(map) = declaration.as_object() else {
        return Err(refuse(
            at,
            "record_validation must be a mapping of schema and locate_absence_at".to_string(),
        ));
    };
    if let Some(unknown) = map.keys().find(|k| !KEYS.contains(&k.as_str())) {
        return Err(refuse(
            at,
            format!("record_validation has unknown key `{unknown}`; it admits only schema and locate_absence_at"),
        ));
    }
    let keys: Vec<&String> = a
        .body_extraction
        .as_ref()
        .and_then(|dsl| dsl.yield_pattern.r#match.as_ref())
        .map(|m| m.keys().collect())
        .unwrap_or_default();
    if keys.is_empty() {
        return Err(refuse(
            at,
            "record_validation needs a body_extraction.yield_pattern.match to build the record from"
                .to_string(),
        ));
    }
    let locate_at = format!("{at}.locate_absence_at");
    let locate_absence_at = match map.get("locate_absence_at").map(serde_json::Value::as_str) {
        None => {
            return Err(refuse(
                locate_at,
                "locate_absence_at is required".to_string(),
            ))
        }
        Some(None) => {
            return Err(refuse(
                locate_at,
                "locate_absence_at must be a string".to_string(),
            ))
        }
        Some(Some("")) => return Err(refuse(locate_at, "locate_absence_at is empty".to_string())),
        Some(Some(key)) if !keys.iter().any(|k| k.as_str() == key) => {
            return Err(refuse(
                locate_at,
                format!("locate_absence_at `{key}` is not a key of yield_pattern.match"),
            ))
        }
        Some(Some(key)) => key.to_string(),
    };
    let Some(schema) = map.get("schema").and_then(serde_json::Value::as_str) else {
        return Err(refuse(
            format!("{at}.schema"),
            "schema is required and must be a string".to_string(),
        ));
    };
    let imports: Vec<String> = semantic.imports.keys().cloned().collect();
    let resolved = resolve_reference_at(
        source,
        &DataSchemaRef {
            schema: schema.to_string(),
        },
        "record_validation",
        section,
        &a.name,
        &imports,
    )
    .map_err(|f| failure(module, &a.name, manifest_path, f.reason()))?;
    Ok(Some(CompiledRecordValidation {
        schema: Arc::new(resolved.schema),
        validator: Arc::new(resolved.validator),
        locate_absence_at,
    }))
}
