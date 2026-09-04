//! JSON request adapter shared by the Python and WASM bindings (FR-072):
//! input/output conversion only, no extraction policy (FR-046-AC-3).

use serde::Deserialize;
use serde_json::Value;

use super::context::{BundleIndex, SemanticContext};
use super::contract::SemanticModule;
use super::surface::{extract_semantic, RequiredSections, SemanticExtraction};

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
}

/// Run FR-072 for a JSON request; the error is a deserialization message.
pub fn extract_semantic_json(request: &Value) -> Result<SemanticExtraction, String> {
    let req: Request = serde_json::from_value(request.clone()).map_err(|e| e.to_string())?;
    let module = SemanticModule {
        contract_version: req.module.contract_version,
        semantic_core: req.module.semantic_core,
        package: req.module.package,
        exports: req.module.exports,
        imports: req.module.imports,
        targets: req.module.targets,
        compatibility_posture: req.module.compatibility_posture,
        legacy_forms: req.module.legacy_forms,
    };
    let mut ctx = SemanticContext::new(
        module,
        req.path.unwrap_or_else(|| "<document>".to_string()),
        req.bundle.unwrap_or_default(),
    );
    ctx.source_identity = req.source_identity;
    ctx.scope = req.scope;
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
