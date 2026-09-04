//! Offline schema resolution (FR-069 Behavior, FR-069-CON-1).
//!
//! A reference-form `data_schema` is read from the module (filesystem root or
//! the inline `schemas` map), digest-checked, and compiled with every `$ref`
//! pre-registered from an in-memory `$id → document` map built from the
//! module's sibling files and the embedded semantic-core bundle. The schema
//! library's file and HTTP resolvers are never consulted, so the same code
//! runs under the `wasm` feature.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use jsonschema::JSONSchema;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::contract::{DataSchemaRef, SemanticFailure, SemanticModule};
use super::vendored;

/// Where a module's files come from.
pub enum SchemaSource<'a> {
    Filesystem {
        module_root: &'a Path,
    },
    /// `Registry::from_inline_parts`: manifest-relative path → file text.
    Inline {
        files: &'a BTreeMap<String, String>,
    },
}

/// A resolved reference-form schema: the parsed document, the digest over
/// the shipped bytes, and the compiled validator.
pub struct ResolvedSchema {
    pub schema: Value,
    pub digest: String,
    pub validator: JSONSchema,
    /// The manifest-relative path of the schema file.
    pub path: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn escapes(rel: &str) -> bool {
    let p = Path::new(rel);
    p.is_absolute()
        || p.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
}

fn read_module_file(source: &SchemaSource, rel: &str) -> Result<Option<Vec<u8>>, String> {
    if escapes(rel) {
        return Err("escape".to_string());
    }
    match source {
        SchemaSource::Inline { files } => Ok(files.get(rel).map(|s| s.as_bytes().to_vec())),
        SchemaSource::Filesystem { module_root } => {
            let path = module_root.join(rel);
            if !path.exists() {
                return Ok(None);
            }
            // A symlink that resolves outside the module root is an escape.
            let root = module_root
                .canonicalize()
                .map_err(|e| format!("module root unreadable: {e}"))?;
            let real = path
                .canonicalize()
                .map_err(|e| format!("unreadable: {e}"))?;
            if !real.starts_with(&root) {
                return Err("escape".to_string());
            }
            std::fs::read(&real)
                .map(Some)
                .map_err(|e| format!("unreadable: {e}"))
        }
    }
}

fn locus(object_type: &str, key: &str) -> String {
    format!("object_types[{object_type}].data_schema.{key}")
}

/// Resolve and compile one reference-form `data_schema` (FR-069 Behavior).
pub fn resolve_reference(
    source: &SchemaSource,
    reference: &DataSchemaRef,
    module: &SemanticModule,
    module_version: &str,
    object_type: &str,
) -> Result<ResolvedSchema, SemanticFailure> {
    let rel = reference.schema.as_str();
    let bytes = match read_module_file(source, rel) {
        Err(e) if e == "escape" => {
            return Err(SemanticFailure::error(
                "semantic.data-schema-escape",
                locus(object_type, "schema"),
                format!("schema path {rel} leaves the module root"),
            ))
        }
        Err(e) => {
            return Err(SemanticFailure::error(
                "semantic.data-schema-missing",
                locus(object_type, "schema"),
                format!("schema file {rel} is {e}"),
            ))
        }
        Ok(None) => {
            return Err(SemanticFailure::error(
                "semantic.data-schema-missing",
                locus(object_type, "schema"),
                format!("schema file {rel} is not shipped in the module"),
            ))
        }
        Ok(Some(b)) => b,
    };
    let digest = sha256_hex(&bytes);
    if digest != reference.digest {
        return Err(SemanticFailure::error(
            "semantic.data-schema-digest-mismatch",
            locus(object_type, "digest"),
            format!(
                "schema file {rel} hashes to {digest}, manifest records {}",
                reference.digest
            ),
        ));
    }
    let schema: Value = serde_json::from_slice(&bytes).map_err(|e| {
        SemanticFailure::error(
            "semantic.data-schema-not-json",
            locus(object_type, "schema"),
            format!("schema file {rel} is not JSON: {e}"),
        )
    })?;
    if schema.get("$schema").and_then(Value::as_str)
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        return Err(SemanticFailure::error(
            "semantic.data-schema-not-schema",
            locus(object_type, "schema"),
            format!("schema file {rel} does not declare JSON Schema 2020-12"),
        ));
    }
    let file_name = Path::new(rel)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(rel);
    let expected_id = format!("{}{}", module.schema_base(module_version), file_name);
    let actual_id = schema.get("$id").and_then(Value::as_str).unwrap_or("");
    if actual_id != expected_id {
        return Err(SemanticFailure::error(
            "semantic.data-schema-id",
            locus(object_type, "schema"),
            format!("schema $id is {actual_id:?}, expected {expected_id}"),
        ));
    }
    let dir: PathBuf = Path::new(rel)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let siblings = |name: &str| -> Option<Result<Value, String>> {
        let sibling = dir.join(name).to_string_lossy().to_string();
        match read_module_file(source, &sibling) {
            Ok(Some(bytes)) => Some(serde_json::from_slice(&bytes).map_err(|e| e.to_string())),
            Ok(None) => None,
            Err(_) => None,
        }
    };
    let validator = compile_module_schema(
        &schema,
        &siblings,
        &module.semantic_core,
        &module.schema_base(module_version),
    )
    .map_err(|mut f| {
        f.path = locus(object_type, "schema");
        f
    })?;
    Ok(ResolvedSchema {
        schema,
        digest,
        validator,
        path: rel.to_string(),
    })
}

fn collect_refs(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if k == "$ref" {
                    if let Some(s) = v.as_str() {
                        out.push(s.to_string());
                    }
                } else {
                    collect_refs(v, out);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|v| collect_refs(v, out)),
        _ => {}
    }
}

/// Compile `schema` with every `$ref` resolved offline: module siblings come
/// from `siblings(name)`, semantic-core documents from the embedded bundle at
/// `semantic_core`. Version drift, unshipped targets, and cycles are refused
/// with their FR-069 codes; a `$ref` to a document's own `$id` is a fragment.
pub fn compile_module_schema(
    schema: &Value,
    siblings: &dyn Fn(&str) -> Option<Result<Value, String>>,
    semantic_core: &str,
    module_base: &str,
) -> Result<JSONSchema, SemanticFailure> {
    let bundle = vendored::semantic_core_bundle(semantic_core).ok_or_else(|| {
        SemanticFailure::error(
            "semantic.unsupported-semantic-core",
            "semantic.semantic_core",
            format!("no vendored bundle for {semantic_core}"),
        )
    })?;
    let core_base = format!("{}{}/", vendored::SEMANTIC_CORE_BASE, semantic_core);
    let mut documents: BTreeMap<String, Value> = BTreeMap::new();
    let mut stack: Vec<String> = Vec::new();
    let root_id = schema
        .get("$id")
        .and_then(Value::as_str)
        .unwrap_or("<root>")
        .to_string();
    walk(
        schema,
        &root_id,
        siblings,
        bundle,
        &core_base,
        module_base,
        semantic_core,
        &mut documents,
        &mut stack,
    )?;
    let mut options = JSONSchema::options();
    for (id, doc) in &documents {
        options.with_document(id.clone(), doc.clone());
    }
    options.compile(schema).map_err(|e| {
        SemanticFailure::error(
            "semantic.data-schema-not-schema",
            "",
            format!("schema compile error: {e}"),
        )
    })
}

#[allow(clippy::too_many_arguments)]
fn walk(
    doc: &Value,
    id: &str,
    siblings: &dyn Fn(&str) -> Option<Result<Value, String>>,
    bundle: &[(&str, &str)],
    core_base: &str,
    module_base: &str,
    semantic_core: &str,
    documents: &mut BTreeMap<String, Value>,
    stack: &mut Vec<String>,
) -> Result<(), SemanticFailure> {
    if stack.iter().any(|s| s == id) {
        let mut cycle = stack.clone();
        cycle.push(id.to_string());
        return Err(SemanticFailure::error(
            "semantic.schema-ref-cycle",
            "",
            format!("$ref cycle: {}", cycle.join(" -> ")),
        ));
    }
    if documents.contains_key(id) {
        return Ok(());
    }
    stack.push(id.to_string());
    let mut refs = Vec::new();
    collect_refs(doc, &mut refs);
    for target in refs {
        let url = target.split('#').next().unwrap_or("");
        if url.is_empty() || url == id {
            continue; // fragment within this document
        }
        if let Some(name) = url.strip_prefix(core_base) {
            let Some((_, text)) = bundle.iter().find(|(n, _)| *n == name) else {
                return Err(SemanticFailure::error(
                    "semantic.schema-ref-unshipped",
                    "",
                    format!("$ref {url} names no file in the vendored semantic-core {semantic_core} bundle"),
                ));
            };
            let child: Value = serde_json::from_str(text).expect("vendored schema is JSON");
            walk(
                &child,
                url,
                siblings,
                bundle,
                core_base,
                module_base,
                semantic_core,
                documents,
                stack,
            )?;
        } else if let Some(rest) = url.strip_prefix(vendored::SEMANTIC_CORE_BASE) {
            let version = rest.split('/').next().unwrap_or("");
            return Err(SemanticFailure::error(
                "semantic.schema-ref-version",
                "",
                format!(
                    "$ref {url} names semantic-core {version}, manifest records {semantic_core}"
                ),
            ));
        } else if let Some(name) = url.strip_prefix(module_base) {
            match siblings(name) {
                Some(Ok(child)) => walk(
                    &child,
                    url,
                    siblings,
                    bundle,
                    core_base,
                    module_base,
                    semantic_core,
                    documents,
                    stack,
                )?,
                Some(Err(e)) => {
                    return Err(SemanticFailure::error(
                        "semantic.data-schema-not-json",
                        "",
                        format!("referenced file {name} is not JSON: {e}"),
                    ))
                }
                None => {
                    return Err(SemanticFailure::error(
                        "semantic.schema-ref-unshipped",
                        "",
                        format!("$ref {url} names no shipped file ({name})"),
                    ))
                }
            }
        } else {
            return Err(SemanticFailure::error(
                "semantic.schema-ref-unshipped",
                "",
                format!("$ref {url} is outside the module bundle and the semantic-core bundle"),
            ));
        }
    }
    stack.pop();
    documents.insert(id.to_string(), doc.clone());
    Ok(())
}
