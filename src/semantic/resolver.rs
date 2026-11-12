//! Offline schema resolution (FR-069 Behavior, FR-069-CON-1).
//!
//! A reference-form `data_schema` is read from the module (filesystem root or
//! the inline `schemas` map) and compiled with every `$ref`
//! pre-registered from an in-memory `$id → document` map built from the
//! module's sibling files and the embedded semantic-core bundle. The schema
//! library's file and HTTP resolvers are never consulted, so the same code
//! runs under the `wasm` feature.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use jsonschema::Validator;
use serde_json::Value;

use super::contract::{DataSchemaRef, SemanticFailure};
use super::embedded;
use crate::loader::compile::{schema_draft, schema_options};

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

/// A resolved reference-form schema: the parsed document and the compiled
/// validator.
pub struct ResolvedSchema {
    pub schema: Value,
    pub validator: Validator,
    /// Every document the schema's `$ref`s reach, keyed by the URL the
    /// `$ref` names (record-sections reads property types through them).
    pub documents: BTreeMap<String, Value>,
    /// The manifest-relative path of the schema file.
    pub path: String,
    /// Every declared-import `$ref` the schema's module files carry.
    pub import_refs: Vec<ImportRef>,
}

/// A `$ref` under the schema base of a declared import (FR-069), admitted
/// without being followed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRef {
    /// The imported package, `<org>/<repo>`.
    pub package: String,
    /// The provider type the `$ref` names (file stem).
    pub type_name: String,
    /// The module's own type file holding the `$ref` (file stem).
    pub from_type: String,
    /// The full `$ref` URL.
    pub url: String,
}

/// The file-name stem of a schema URL: the final path segment with a
/// `#fragment` dropped, then a trailing `.json`, then a trailing `.schema`.
pub fn type_name_of(url: &str) -> String {
    let file = url.split('#').next().unwrap_or("");
    let file = file.rsplit('/').next().unwrap_or("");
    let file = file.strip_suffix(".json").unwrap_or(file);
    file.strip_suffix(".schema").unwrap_or(file).to_string()
}

/// The package a `$ref` URL belongs to, when it is under the schema base of
/// one of `imports`.
fn import_package<'a>(url: &str, imports: &'a [String]) -> Option<&'a str> {
    let rest = url.strip_prefix(embedded::MODULE_SCHEMA_BASE)?;
    imports
        .iter()
        .map(String::as_str)
        .find(|p| rest.strip_prefix(*p).is_some_and(|r| r.starts_with('/')))
}

/// `schema` with every `$ref` under a declared import's schema base removed,
/// so the module's own validator treats it as an opaque subschema that
/// accepts any value.
fn without_import_refs(schema: &Value, imports: &[String]) -> Value {
    match schema {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(k, v)| {
                    !(k.as_str() == "$ref"
                        && v.as_str()
                            .is_some_and(|u| import_package(u, imports).is_some()))
                })
                .map(|(k, v)| (k.clone(), without_import_refs(v, imports)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|v| without_import_refs(v, imports))
                .collect(),
        ),
        other => other.clone(),
    }
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

fn locus(section: &str, object_type: &str, field: &str, key: &str) -> String {
    format!("{section}[{object_type}].{field}.{key}")
}

/// Resolve and compile one reference-form `data_schema` (FR-069 Behavior).
pub fn resolve_reference(
    source: &SchemaSource,
    reference: &DataSchemaRef,
    section: &str,
    object_type: &str,
    imports: &[String],
) -> Result<ResolvedSchema, SemanticFailure> {
    resolve_reference_at(
        source,
        reference,
        "data_schema",
        section,
        object_type,
        imports,
    )
}

/// [`resolve_reference`] for the schema file named by the manifest key `field`
/// of the archetype (`data_schema`, or `record_validation` for FR-069's located
/// record validation): the refusal path is `<section>[<type>].<field>.schema`.
pub(crate) fn resolve_reference_at(
    source: &SchemaSource,
    reference: &DataSchemaRef,
    field: &str,
    section: &str,
    object_type: &str,
    imports: &[String],
) -> Result<ResolvedSchema, SemanticFailure> {
    let rel = reference.schema.as_str();
    let bytes = match read_module_file(source, rel) {
        Err(e) if e == "escape" => {
            return Err(SemanticFailure::error(
                "semantic.data-schema-escape",
                locus(section, object_type, field, "schema"),
                format!("schema path {rel} leaves the module root"),
            ))
        }
        Err(e) => {
            return Err(SemanticFailure::error(
                "semantic.data-schema-missing",
                locus(section, object_type, field, "schema"),
                format!("schema file {rel} is {e}"),
            ))
        }
        Ok(None) => {
            return Err(SemanticFailure::error(
                "semantic.data-schema-missing",
                locus(section, object_type, field, "schema"),
                format!("schema file {rel} is not shipped in the module"),
            ))
        }
        Ok(Some(b)) => b,
    };
    let schema: Value = serde_json::from_slice(&bytes).map_err(|e| {
        SemanticFailure::error(
            "semantic.data-schema-not-json",
            locus(section, object_type, field, "schema"),
            format!("schema file {rel} is not JSON: {e}"),
        )
    })?;
    if schema.get("$schema").and_then(Value::as_str)
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        return Err(SemanticFailure::error(
            "semantic.data-schema-not-schema",
            locus(section, object_type, field, "schema"),
            format!("schema file {rel} does not declare JSON Schema 2020-12"),
        ));
    }
    let module_base = schema
        .get("$id")
        .and_then(Value::as_str)
        .and_then(|id| id.rfind('/').map(|i| id[..=i].to_string()))
        .unwrap_or_default();
    // A module's own package is never an import: its sibling `$ref`s are
    // read and validated, even when `semantic.imports` names the package at
    // another version.
    let imports: Vec<String> = imports
        .iter()
        .filter(|p| {
            !module_base
                .strip_prefix(embedded::MODULE_SCHEMA_BASE)
                .is_some_and(|rest| {
                    rest.strip_prefix(p.as_str())
                        .is_some_and(|r| r.starts_with('/'))
                })
        })
        .cloned()
        .collect();
    let imports = imports.as_slice();
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
    let relocate = |mut f: SemanticFailure| {
        f.path = locus(section, object_type, field, "schema");
        f
    };
    let (documents, mut import_refs) =
        walk_documents(&schema, &siblings, &module_base, imports).map_err(relocate)?;
    let validator = compile_with_imports(&schema, &documents, imports).map_err(relocate)?;
    // The root document may carry no `$id`; its file name is the type then.
    let root_name = type_name_of(rel);
    for r in &mut import_refs {
        if r.from_type.is_empty() {
            r.from_type = root_name.clone();
        }
    }
    Ok(ResolvedSchema {
        schema,
        validator,
        documents,
        path: rel.to_string(),
        import_refs,
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

/// Every document `schema` reaches through its `$ref`s, keyed by the URL the
/// `$ref` names (the schema itself under its `$id`): module siblings from
/// `siblings(name)`, semantic-core documents from the embedded bundle.
/// Unshipped targets and cycles are refused with their FR-069 codes; a `$ref`
/// to a document's own `$id` is a fragment.
pub fn module_schema_documents(
    schema: &Value,
    siblings: &dyn Fn(&str) -> Option<Result<Value, String>>,
    module_base: &str,
) -> Result<BTreeMap<String, Value>, SemanticFailure> {
    walk_documents(schema, siblings, module_base, &[]).map(|(documents, _)| documents)
}

/// [`module_schema_documents`] that also admits `$ref`s under the schema
/// base of a package in `imports`, returning them without following them.
fn walk_documents(
    schema: &Value,
    siblings: &dyn Fn(&str) -> Option<Result<Value, String>>,
    module_base: &str,
    imports: &[String],
) -> Result<(BTreeMap<String, Value>, Vec<ImportRef>), SemanticFailure> {
    let bundle = embedded::semantic_core_bundle();
    let semantic_core = embedded::embedded_semantic_core_version();
    let core_base = format!("{}{}/", embedded::SEMANTIC_CORE_BASE, semantic_core);
    let mut documents: BTreeMap<String, Value> = BTreeMap::new();
    let mut stack: Vec<String> = Vec::new();
    let root_id = schema
        .get("$id")
        .and_then(Value::as_str)
        .unwrap_or("<root>")
        .to_string();
    let mut import_refs = Vec::new();
    let ctx = Walk {
        siblings,
        bundle,
        core_base: &core_base,
        module_base,
        semantic_core,
        imports,
    };
    walk(
        &ctx,
        schema,
        &root_id,
        &mut documents,
        &mut stack,
        &mut import_refs,
    )?;
    Ok((documents, import_refs))
}

/// Compile `schema` with every `$ref` resolved offline
/// ([`module_schema_documents`]).
pub fn compile_module_schema(
    schema: &Value,
    siblings: &dyn Fn(&str) -> Option<Result<Value, String>>,
    module_base: &str,
) -> Result<Validator, SemanticFailure> {
    let documents = module_schema_documents(schema, siblings, module_base)?;
    compile_documents(schema, &documents)
}

/// Compile `schema` against its already-resolved `documents`.
pub fn compile_documents(
    schema: &Value,
    documents: &BTreeMap<String, Value>,
) -> Result<Validator, SemanticFailure> {
    compile_with_imports(schema, documents, &[])
}

/// [`compile_documents`] where a `$ref` under the schema base of a package in
/// `imports` accepts any value.
fn compile_with_imports(
    schema: &Value,
    documents: &BTreeMap<String, Value>,
    imports: &[String],
) -> Result<Validator, SemanticFailure> {
    let strip = |v: &Value| {
        if imports.is_empty() {
            v.clone()
        } else {
            without_import_refs(v, imports)
        }
    };
    let schema = &strip(schema);
    let not_schema = |e: &dyn std::fmt::Display| {
        SemanticFailure::error(
            "semantic.data-schema-not-schema",
            "",
            format!("schema compile error: {e}"),
        )
    };
    // Each resolved document is registered under its own id; one that
    // declares no `$schema` is read as draft-07, as `schema_options` does for
    // the root.
    let mut registry = jsonschema::Registry::new();
    for (id, doc) in documents {
        // A root that declares no `$id` is walked under a placeholder name
        // and is compiled directly below, so it has no URI to register.
        if id == "<root>" {
            continue;
        }
        let resource = schema_draft(doc).create_resource(strip(doc));
        registry = registry.add(id, resource).map_err(|e| not_schema(&e))?;
    }
    let registry = registry.prepare().map_err(|e| not_schema(&e))?;
    schema_options(schema)
        .with_registry(&registry)
        .build(schema)
        .map_err(|e| not_schema(&e))
}

/// What `walk` needs besides the document it is on.
struct Walk<'a> {
    siblings: &'a dyn Fn(&str) -> Option<Result<Value, String>>,
    bundle: &'a [(&'a str, &'a str)],
    core_base: &'a str,
    module_base: &'a str,
    semantic_core: &'a str,
    imports: &'a [String],
}

fn walk(
    ctx: &Walk,
    doc: &Value,
    id: &str,
    documents: &mut BTreeMap<String, Value>,
    stack: &mut Vec<String>,
    import_refs: &mut Vec<ImportRef>,
) -> Result<(), SemanticFailure> {
    let Walk {
        siblings,
        bundle,
        core_base,
        module_base,
        semantic_core,
        imports,
    } = *ctx;
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
        if let Some(package) = import_package(url, imports) {
            import_refs.push(ImportRef {
                package: package.to_string(),
                type_name: type_name_of(url),
                // The root document may carry no `$id`: filled by the caller.
                from_type: if id == "<root>" {
                    String::new()
                } else {
                    type_name_of(id)
                },
                url: url.to_string(),
            });
        } else if let Some(rest) = url.strip_prefix(embedded::SEMANTIC_CORE_BASE) {
            // A semantic-core `$ref` resolves to the one embedded bundle by
            // file name; the version segment of the URL is not consulted.
            let name = rest.split_once('/').map_or("", |(_, name)| name);
            let Some((_, text)) = bundle.iter().find(|(n, _)| *n == name) else {
                return Err(SemanticFailure::error(
                    "semantic.schema-ref-unshipped",
                    "",
                    format!("$ref {url} names no file in the embedded semantic-core {semantic_core} bundle"),
                ));
            };
            let canonical = format!("{core_base}{name}");
            let child: Value = serde_json::from_str(text).expect("embedded schema is JSON");
            walk(ctx, &child, &canonical, documents, stack, import_refs)?;
            if url != canonical {
                if let Some(doc) = documents.get(&canonical).cloned() {
                    documents.insert(url.to_string(), doc);
                }
            }
        } else if let Some(name) = url.strip_prefix(module_base) {
            match siblings(name) {
                Some(Ok(child)) => walk(ctx, &child, url, documents, stack, import_refs)?,
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
