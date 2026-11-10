//! The module `semantic` block and reference-form `data_schema` (FR-069).
//!
//! Refusals carry a `semantic.*` code and are evaluated in the order
//! FR-069 fixes: contract version, semantic-core version, block shape
//! (which includes an unknown `targets` value, via the module-manifest
//! schema's own enum), exports, then package.

use std::collections::BTreeMap;

use jsonschema::{error::ValidationErrorKind, Validator};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::embedded;
use crate::loader::compile::schema_options;

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

    /// The `ArchetypeLoadFailure.reason` form: `<code>: <path>: <message>`,
    /// or `<code>: <message>` when the refusal has no manifest locus.
    pub fn reason(&self) -> String {
        if self.path.is_empty() {
            format!("{}: {}", self.code, self.message)
        } else {
            format!("{}: {}: {}", self.code, self.path, self.message)
        }
    }
}

/// The loaded `semantic` block (FR-069 Outputs). `mappings` names the
/// representation mappings and gates FR-075 features; `sweep_report` is a
/// Quoin install-time key: accepted, not recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticModule {
    pub contract_version: String,
    pub package: String,
    pub exports: Vec<String>,
    pub imports: BTreeMap<String, String>,
    /// For an imported package, the names of its types this module uses.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub import_types: BTreeMap<String, Vec<String>>,
    pub targets: Vec<String>,
    pub compatibility_posture: String,
    pub legacy_forms: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<String>,
}

impl SemanticModule {
    /// `(org, repo)` of the `<org>/<repo>` package identity.
    pub fn package_parts(&self) -> (&str, &str) {
        let (org, repo) = self.package.split_once('/').unwrap_or((&self.package, ""));
        (org, repo)
    }
}

/// A `data_schema: { schema }` reference (quoin FR-073).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSchemaRef {
    pub schema: String,
}

/// Which `data_schema` form a manifest value takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSchemaForm {
    Inline,
    Reference(DataSchemaRef),
    /// `{ schema, type }` and the like: both forms at once.
    Ambiguous,
}

/// Classify a `data_schema` value. The reference form is exactly the key
/// `schema`, a string; a value carrying it plus other keys is ambiguous;
/// anything else is an inline JSON Schema.
pub fn reference_form(value: &Value) -> DataSchemaForm {
    let Some(map) = value.as_object() else {
        return DataSchemaForm::Inline;
    };
    let Some(schema) = map.get("schema").and_then(Value::as_str) else {
        return DataSchemaForm::Inline;
    };
    if map.len() == 1 {
        return DataSchemaForm::Reference(DataSchemaRef {
            schema: schema.to_string(),
        });
    }
    DataSchemaForm::Ambiguous
}

fn block_validator() -> Validator {
    let schema: Value = serde_json::from_str(embedded::MODULE_MANIFEST_SCHEMA)
        .expect("embedded module-manifest schema is JSON");
    let block = schema["properties"]["semantic"].clone();
    schema_options(&block)
        .build(&block)
        .expect("embedded semantic block schema compiles")
}

/// Shape of `semantic.import_types` (FR-069-AC-16). The locus is the
/// unescaped package key with no index, not the schema-error pointer form.
fn check_import_types(map: &serde_json::Map<String, Value>) -> Vec<SemanticFailure> {
    let Some(value) = map.get("import_types") else {
        return Vec::new();
    };
    let invalid =
        |at: String, why: String| SemanticFailure::error("semantic.invalid-value", at, why);
    let Some(entries) = value.as_object() else {
        return vec![invalid(
            "semantic.import_types".to_string(),
            "import_types must be a mapping".to_string(),
        )];
    };
    let imports = map.get("imports").and_then(Value::as_object);
    let mut failures = Vec::new();
    for (package, types) in entries {
        let at = format!("semantic.import_types.{package}");
        let why = if !imports.is_some_and(|i| i.contains_key(package)) {
            Some(format!("{package} is not a key of semantic.imports"))
        } else if let Some(list) = types.as_array() {
            let mut seen = std::collections::BTreeSet::new();
            list.iter()
                .find_map(|t| match t.as_str() {
                    None => Some(format!("type name {t} is not a string")),
                    Some("") => Some("type name is empty".to_string()),
                    Some(name) if !seen.insert(name) => {
                        Some(format!("type name {name} is listed twice"))
                    }
                    Some(_) => None,
                })
                .or_else(|| list.is_empty().then(|| "type list is empty".to_string()))
        } else {
            Some("import_types value must be an array of type names".to_string())
        };
        if let Some(why) = why {
            failures.push(invalid(at, why));
        }
    }
    failures
}

fn read_import_types(map: &serde_json::Map<String, Value>) -> BTreeMap<String, Vec<String>> {
    map.get("import_types")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(package, types)| {
                    let names = types
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default();
                    (package.clone(), names)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Read and check a `semantic` block (FR-069 Behavior, refusals in order).
///
/// `declared_types` are all names declared by the manifest's type sections.
pub fn read_semantic_block(
    block: &Value,
    declared_types: &[String],
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
    if contract_version != Some(embedded::CONTRACT_VERSION) {
        return Err(vec![SemanticFailure::error(
            "semantic.unsupported-contract-version",
            "semantic.contract_version",
            format!(
                "contract_version {} is not {}",
                contract_version
                    .map(|v| format!("{v:?}"))
                    .unwrap_or_else(|| "absent".to_string()),
                embedded::CONTRACT_VERSION
            ),
        )]);
    }
    // 2. shape against the embedded module-manifest schema.
    let mut failures = Vec::new();
    let validator = block_validator();
    failures.extend(check_import_types(map));
    for error in validator.iter_errors(block) {
        // `import_types` is checked above, at the unescaped package locus.
        if error
            .instance_path()
            .to_string()
            .starts_with("/import_types")
        {
            continue;
        }
        let at = format!(
            "semantic{}",
            error.instance_path().to_string().replace('/', ".")
        );
        let failure = match error.kind() {
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
                        "target {} is outside the embedded target registry",
                        error.instance()
                    ),
                )
            }
            ValidationErrorKind::Pattern { .. } if at == "semantic.package" => {
                SemanticFailure::error(
                    "semantic.invalid-package",
                    at.clone(),
                    format!("package {} is not <org>/<repo>", error.instance()),
                )
            }
            ValidationErrorKind::Required { property } => SemanticFailure::error(
                "semantic.missing-key",
                at.clone(),
                format!("semantic block requires {property}"),
            ),
            _ => SemanticFailure::error("semantic.invalid-value", at.clone(), error.to_string()),
        };
        failures.push(failure);
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
    let mappings = string_list("mappings");
    let package = map["package"].as_str().unwrap_or_default().to_string();
    // 3. exports name declared module types.
    for name in &exports {
        if !declared_types.iter().any(|t| t == name) {
            failures.push(SemanticFailure::error(
                "semantic.export-undeclared",
                format!("semantic.exports.{name}"),
                format!("exports names {name}, which the manifest does not declare"),
            ));
        }
    }
    // 4. package shape (belt and braces beside the schema pattern).
    if package.starts_with("ix://") || package.contains("://") || package.matches('/').count() != 1
    {
        failures.push(SemanticFailure::error(
            "semantic.invalid-package",
            "semantic.package",
            format!("package {package:?} is not <org>/<repo>"),
        ));
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
        contract_version: embedded::CONTRACT_VERSION.to_string(),
        package,
        exports,
        imports,
        import_types: read_import_types(map),
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
        mappings,
    })
}
