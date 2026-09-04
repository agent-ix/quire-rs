//! The module `semantic` block and reference-form `data_schema` (FR-069).
//!
//! Refusals carry a `semantic.*` code and are evaluated in the order
//! FR-069 fixes: contract version, semantic-core version, block shape,
//! exports, package, targets, then each exported type's schema form.

use std::collections::BTreeMap;

use jsonschema::{error::ValidationErrorKind, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::vendored;

/// Severity of a semantic diagnostic. `Advisory` lives inside the semantic
/// record only (FR-072); outside it maps onto the existing `warning` level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SemanticSeverity {
    Advisory,
    Warning,
    Error,
}

/// One semantic diagnostic: a stable code, a message naming the value, and a
/// JSON-pointer-like path into the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticFailure {
    pub code: String,
    pub severity: SemanticSeverity,
    pub path: String,
    pub message: String,
}

impl SemanticFailure {
    pub fn error(code: &str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            severity: SemanticSeverity::Error,
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn warning(code: &str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            severity: SemanticSeverity::Warning,
            path: path.into(),
            message: message.into(),
        }
    }

    /// The `ArchetypeLoadFailure.reason` form: `<code>: <message>`.
    pub fn reason(&self) -> String {
        format!("{}: {}", self.code, self.message)
    }
}

/// The loaded `semantic` block (FR-069 Outputs). `mappings` and
/// `sweep_report` are Quoin install-time keys: accepted, not recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticModule {
    pub contract_version: String,
    pub semantic_core: String,
    pub package: String,
    pub exports: Vec<String>,
    pub imports: BTreeMap<String, String>,
    pub targets: Vec<String>,
    pub compatibility_posture: String,
    pub legacy_forms: String,
}

impl SemanticModule {
    /// `(org, repo)` of the `<org>/<repo>` package identity.
    pub fn package_parts(&self) -> (&str, &str) {
        let (org, repo) = self.package.split_once('/').unwrap_or((&self.package, ""));
        (org, repo)
    }

    /// Base of every `$id` this module's schemas must carry at `module_version`.
    pub fn schema_base(&self, module_version: &str) -> String {
        format!(
            "{}{}/{}/",
            vendored::MODULE_SCHEMA_BASE,
            self.package,
            module_version
        )
    }
}

/// A `data_schema: { schema, digest }` reference (quoin FR-073).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSchemaRef {
    pub schema: String,
    pub digest: String,
}

/// Which `data_schema` form a manifest value takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSchemaForm {
    Inline,
    Reference(DataSchemaRef),
    /// `{ schema, digest, type }` and the like: both forms at once.
    Ambiguous,
}

/// Classify a `data_schema` value. The reference form is exactly the keys
/// `schema` and `digest`, both strings; a value carrying those plus schema
/// keywords is ambiguous; anything else is an inline JSON Schema.
pub fn reference_form(value: &Value) -> DataSchemaForm {
    let Some(map) = value.as_object() else {
        return DataSchemaForm::Inline;
    };
    let has_ref = map.get("schema").and_then(Value::as_str).is_some()
        && map.get("digest").and_then(Value::as_str).is_some();
    if !has_ref {
        return DataSchemaForm::Inline;
    }
    if map.len() == 2 {
        return DataSchemaForm::Reference(DataSchemaRef {
            schema: map["schema"].as_str().unwrap_or_default().to_string(),
            digest: map["digest"].as_str().unwrap_or_default().to_string(),
        });
    }
    DataSchemaForm::Ambiguous
}

/// The vendored target registry: `target` ∪ `representationFormat` values of
/// filament-core-data `common.schema.json`.
pub fn target_registry() -> Vec<String> {
    let common: Value =
        serde_json::from_str(vendored::COMMON_SCHEMA).expect("vendored common.schema.json is JSON");
    let mut out = Vec::new();
    for def in ["target", "representationFormat"] {
        if let Some(values) = common["$defs"][def]["enum"].as_array() {
            out.extend(values.iter().filter_map(Value::as_str).map(str::to_string));
        }
    }
    out
}

fn block_validator() -> JSONSchema {
    let schema: Value = serde_json::from_str(vendored::MODULE_MANIFEST_SCHEMA)
        .expect("vendored module-manifest schema is JSON");
    let block = schema["properties"]["semantic"].clone();
    JSONSchema::options()
        .compile(&block)
        .expect("vendored semantic block schema compiles")
}

/// Read and check a `semantic` block (FR-069 Behavior, refusals in order).
///
/// `object_types` are the declared object-type names; `has_reference_schema`
/// answers whether a named type declares the reference-form `data_schema`.
pub fn read_semantic_block(
    block: &Value,
    object_types: &[String],
    has_reference_schema: &dyn Fn(&str) -> bool,
) -> Result<SemanticModule, Vec<SemanticFailure>> {
    let Some(map) = block.as_object() else {
        return Err(vec![SemanticFailure::error(
            "semantic.invalid-value",
            "semantic",
            "semantic must be a mapping",
        )]);
    };
    // 1. contract version, before any other key is read.
    let contract_version = map.get("contract_version").and_then(Value::as_str);
    if contract_version != Some(vendored::CONTRACT_VERSION) {
        return Err(vec![SemanticFailure::error(
            "semantic.unsupported-contract-version",
            "semantic.contract_version",
            format!(
                "semantic.contract_version {} is not {}",
                contract_version
                    .map(|v| format!("{v:?}"))
                    .unwrap_or_else(|| "absent".to_string()),
                vendored::CONTRACT_VERSION
            ),
        )]);
    }
    // 2. semantic-core version must have an embedded bundle.
    let semantic_core = map.get("semantic_core").and_then(Value::as_str);
    match semantic_core {
        Some(v) if vendored::semantic_core_bundle(v).is_some() => {}
        other => {
            return Err(vec![SemanticFailure::error(
                "semantic.unsupported-semantic-core",
                "semantic.semantic_core",
                format!(
                    "semantic.semantic_core {} has no vendored bundle (vendored: {})",
                    other
                        .map(|v| format!("{v:?}"))
                        .unwrap_or_else(|| "absent".to_string()),
                    vendored::SEMANTIC_CORE_VERSIONS.join(", ")
                ),
            )]);
        }
    }
    // 3. shape against the vendored module-manifest schema.
    let mut failures = Vec::new();
    let validator = block_validator();
    if let Err(errors) = validator.validate(block) {
        for error in errors {
            let at = format!(
                "semantic{}",
                error.instance_path.to_string().replace('/', ".")
            );
            let failure = match &error.kind {
                ValidationErrorKind::AdditionalProperties { unexpected } => SemanticFailure::error(
                    "semantic.unknown-key",
                    at.clone(),
                    format!("unknown key(s) {}", unexpected.join(", ")),
                ),
                ValidationErrorKind::Enum { .. } if at.starts_with("semantic.targets") => {
                    SemanticFailure::error(
                        "semantic.unknown-target",
                        at.clone(),
                        format!(
                            "target {} is outside the vendored target registry",
                            error.instance
                        ),
                    )
                }
                ValidationErrorKind::Pattern { .. } if at == "semantic.package" => {
                    SemanticFailure::error(
                        "semantic.invalid-package",
                        at.clone(),
                        format!("package {} is not <org>/<repo>", error.instance),
                    )
                }
                ValidationErrorKind::Required { property } => SemanticFailure::error(
                    "semantic.missing-key",
                    at.clone(),
                    format!("semantic block requires {property}"),
                ),
                _ => {
                    SemanticFailure::error("semantic.invalid-value", at.clone(), error.to_string())
                }
            };
            failures.push(failure);
        }
    }
    if !failures.is_empty() {
        return Err(failures);
    }
    let string_list = |key: &str| -> Vec<String> {
        map.get(key)
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    let exports = string_list("exports");
    let targets = string_list("targets");
    let package = map["package"].as_str().unwrap_or_default().to_string();
    // 4. exports name declared object types.
    for name in &exports {
        if !object_types.iter().any(|t| t == name) {
            failures.push(SemanticFailure::error(
                "semantic.export-undeclared",
                format!("semantic.exports.{name}"),
                format!("semantic.exports names {name}, which object_types does not declare"),
            ));
        }
    }
    // 5. package shape (belt and braces beside the schema pattern).
    if package.starts_with("ix://") || package.contains("://") || package.matches('/').count() != 1
    {
        failures.push(SemanticFailure::error(
            "semantic.invalid-package",
            "semantic.package",
            format!("package {package:?} is not <org>/<repo>"),
        ));
    }
    // 6. targets against the vendored registry.
    let registry = target_registry();
    for (i, target) in targets.iter().enumerate() {
        if !registry.iter().any(|t| t == target) {
            failures.push(SemanticFailure::error(
                "semantic.unknown-target",
                format!("semantic.targets.{i}"),
                format!("target {target:?} is outside the vendored target registry"),
            ));
        }
    }
    // 7. every export carries the reference-form data_schema.
    for name in &exports {
        if object_types.iter().any(|t| t == name) && !has_reference_schema(name) {
            failures.push(SemanticFailure::error(
                "semantic.export-without-schema",
                format!("semantic.exports.{name}"),
                format!(
                    "semantic.exports names {name}, whose data_schema is not a {{ schema, digest }} reference; nothing can be pinned for it"
                ),
            ));
        }
    }
    if !failures.is_empty() {
        return Err(failures);
    }
    let imports = map
        .get("imports")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(SemanticModule {
        contract_version: vendored::CONTRACT_VERSION.to_string(),
        semantic_core: semantic_core.unwrap_or_default().to_string(),
        package,
        exports,
        imports,
        targets,
        compatibility_posture: map
            .get("compatibility_posture")
            .and_then(Value::as_str)
            .unwrap_or("additive")
            .to_string(),
        legacy_forms: map
            .get("legacy_forms")
            .and_then(Value::as_str)
            .unwrap_or("warning")
            .to_string(),
    })
}
