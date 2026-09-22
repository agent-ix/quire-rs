//! JSON request adapter shared by the Python and WASM bindings (FR-072):
//! input/output conversion only, no extraction policy (FR-046-AC-3).

use serde::Deserialize;
use serde_json::Value;

use super::context::{BundleIndex, SemanticContext};
use super::contract::SemanticModule;
use super::relations::RelationVocabulary;
use super::surface::{extract_semantic, RequiredSections, SemanticExtraction};
use crate::extract::dsl::ExtractionDsl;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModuleRequest {
    contract_version: String,
    semantic_core: String,
    package: String,
    #[serde(default)]
    exports: Vec<String>,
    #[serde(default)]
    imports: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    targets: Vec<String>,
    #[serde(default = "additive")]
    compatibility_posture: String,
    #[serde(default = "warning")]
    legacy_forms: String,
    #[serde(default)]
    mappings: Vec<String>,
}

fn additive() -> String {
    "additive".to_string()
}
fn warning() -> String {
    "warning".to_string()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    markdown: String,
    module: ModuleRequest,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    source_identity: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    bundle: Option<BundleIndex>,
    #[serde(default)]
    schema_digest: Option<String>,
    #[serde(default)]
    required: Option<Value>,
    /// The object type's `body_extraction` DSL; its `table_row` locators
    /// gate FR-075 tables.
    #[serde(default)]
    body_extraction: Option<Value>,
    /// The edge registry and object-type facts relationship rows are
    /// checked against (FR-076).
    #[serde(default)]
    relation_vocabulary: Option<RelationVocabulary>,
}

/// Run FR-072 for a JSON request; the error is a deserialization message.
pub fn extract_semantic_json(request: &Value) -> Result<SemanticExtraction, String> {
    let req: Request = serde_json::from_value(request.clone()).map_err(|e| e.to_string())?;
    if req.module.contract_version != super::embedded::CONTRACT_VERSION {
        return Err(format!(
            "semantic.unsupported-contract-version: {:?} is not {}",
            req.module.contract_version,
            super::embedded::CONTRACT_VERSION
        ));
    }
    if super::embedded::semantic_core_bundle(&req.module.semantic_core).is_none() {
        return Err(format!(
            "semantic.unsupported-semantic-core: {:?} has no embedded bundle (embedded: {})",
            req.module.semantic_core,
            super::embedded::SEMANTIC_CORE_VERSIONS.join(", ")
        ));
    }
    let module = SemanticModule {
        contract_version: req.module.contract_version,
        semantic_core: req.module.semantic_core,
        package: req.module.package,
        exports: req.module.exports,
        imports: req.module.imports,
        targets: req.module.targets,
        compatibility_posture: req.module.compatibility_posture,
        legacy_forms: req.module.legacy_forms,
        mappings: req.module.mappings,
    };
    let mut ctx = SemanticContext::new(
        module,
        req.path.unwrap_or_else(|| "<document>".to_string()),
        req.bundle.unwrap_or_default(),
    );
    ctx.source_identity = req.source_identity;
    ctx.scope = req.scope;
    if let Some(dsl) = req.body_extraction {
        let dsl: ExtractionDsl = serde_json::from_value(dsl)
            .map_err(|e| format!("semantic.body-extraction-invalid: {e}"))?;
        ctx = ctx.with_body_extraction(&dsl);
    }
    ctx.relation_vocabulary = req.relation_vocabulary;
    let required = req
        .required
        .map(|v| RequiredSections {
            properties: v["properties"].as_bool().unwrap_or(false),
            invariants: v["invariants"].as_bool().unwrap_or(false),
            operations: v["operations"].as_bool().unwrap_or(false),
        })
        .unwrap_or_default();
    Ok(extract_semantic(
        &req.markdown,
        &ctx,
        req.schema_digest.as_deref(),
        &required,
    ))
}
