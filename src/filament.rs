//! Canonical one-document Filament extraction (FR-045).
//!
//! This module is intentionally a pure boundary: callers provide the markdown
//! document and ObjectType snapshots, and the engine returns core-data-shaped
//! JSON records. It does not load registries, perform I/O, or reach into any
//! Filament runtime service.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::error::QuireError;
use crate::extract::dsl::{validate_dsl, ExtractionDsl};
use crate::loader::compile::compile_schema;
use crate::parser::FrontmatterStatus;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilamentExtractionInput {
    #[serde(rename = "project_id", alias = "projectId")]
    pub project_id: String,
    #[serde(rename = "document_id", alias = "documentId")]
    pub document_id: String,
    #[serde(rename = "artifact_id", alias = "artifactId", default)]
    pub artifact_id: Option<String>,
    #[serde(rename = "rel_path", alias = "relPath")]
    pub rel_path: String,
    pub markdown: String,
    #[serde(rename = "repo_name", alias = "repoName")]
    pub repo_name: String,
    #[serde(rename = "object_types", alias = "objectTypes", default)]
    pub object_types: Vec<FilamentObjectType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilamentObjectType {
    pub name: String,
    #[serde(default)]
    pub schema: Option<Value>,
    #[serde(default)]
    pub data_schema: Option<Value>,
    #[serde(rename = "allowed_links", alias = "allowedLinks", default)]
    pub allowed_links: Option<Value>,
    #[serde(rename = "body_extraction", alias = "bodyExtraction", default)]
    pub body_extraction: Option<Value>,
    #[serde(rename = "has_plugin", alias = "hasPlugin", default)]
    pub has_plugin: bool,
    #[serde(rename = "module_id", alias = "moduleId", default)]
    pub module_id: Option<String>,
}

impl FilamentObjectType {
    fn data_schema(&self) -> Value {
        self.data_schema
            .clone()
            .or_else(|| self.schema.clone())
            .unwrap_or_else(|| json!({"type": "object"}))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreExtractionResult {
    #[serde(rename = "documentId")]
    pub document_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: Option<String>,
    #[serde(rename = "objectTypes")]
    pub object_types: Vec<CoreObjectTypeRecord>,
    pub nodes: Vec<CoreGraphNodeRef>,
    pub edges: Vec<CoreGraphEdgeRef>,
    pub diagnostics: Vec<CoreExtractionDiagnostic>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreObjectTypeRecord {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub name: String,
    #[serde(rename = "schemaJson")]
    pub schema_json: String,
    #[serde(rename = "allowedLinksJson")]
    pub allowed_links_json: String,
    #[serde(rename = "bodyExtractionJson")]
    pub body_extraction_json: Option<String>,
    #[serde(rename = "hasPlugin")]
    pub has_plugin: bool,
    #[serde(rename = "moduleId")]
    pub module_id: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreGraphNodeRef {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "documentId")]
    pub document_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: Option<String>,
    #[serde(rename = "objectType")]
    pub object_type: String,
    pub name: String,
    #[serde(rename = "ref")]
    pub ref_: String,
    #[serde(rename = "dataJson")]
    pub data_json: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreGraphEdgeRef {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "sourceRef")]
    pub source_ref: String,
    #[serde(rename = "targetRef")]
    pub target_ref: String,
    #[serde(rename = "edgeType")]
    pub edge_type: String,
    #[serde(rename = "dataJson")]
    pub data_json: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreExtractionDiagnostic {
    pub code: String,
    pub message: String,
    pub severity: String,
    #[serde(rename = "objectType")]
    pub object_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledObjectType {
    snapshot: FilamentObjectType,
    schema: Value,
    body_extraction: Option<ExtractionDsl>,
}

#[derive(Debug, Clone, PartialEq)]
struct GraphNode {
    object_type: String,
    name: String,
    data: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
struct GraphEdge {
    source_ref: String,
    edge_type: String,
    target_ref: String,
    metadata: Map<String, Value>,
}

pub fn extract_filament_core(input: FilamentExtractionInput) -> CoreExtractionResult {
    let mut diagnostics = Vec::new();
    let mut errors = Vec::new();

    let doc = crate::parse_document(&input.markdown);
    let Some(frontmatter) = doc.frontmatter.clone() else {
        // Frontmatter is absent. Ask the parser authority *why* — a document
        // with no frontmatter block is a clean skip, but a complete fence
        // block that failed to parse is a malformed document that must reach
        // Filament's index_errors via CoreExtractionResult.errors (issue #127,
        // CR-011). This re-parse only runs on the no-frontmatter branch, so
        // the happy path is unaffected.
        return match crate::extract_frontmatter(&input.markdown).status {
            FrontmatterStatus::Malformed => malformed_frontmatter_result(input),
            // Absent (or the unreachable Present, which would have yielded
            // Some above): a genuinely non-Filament document, skipped cleanly.
            FrontmatterStatus::Absent | FrontmatterStatus::Present => {
                absent_frontmatter_result(input)
            }
        };
    };

    let compiled = compile_object_type_snapshots(&input.object_types, &mut diagnostics);
    let object_type_records = object_type_records(&input.project_id, &input.object_types);
    let object_name = frontmatter
        .get("object")
        .and_then(Value::as_str)
        .map(str::to_string);

    let mut nodes = Vec::new();
    let mut tier_edges = Vec::new();

    if let Some(object_name) = object_name {
        match compiled.get(&object_name) {
            None => diagnostics.push(diagnostic(
                "unknown_object_type",
                format!("frontmatter object={object_name:?} not in supplied object_types"),
                "warning",
                Some(object_name),
            )),
            Some(object_type) if object_type.snapshot.has_plugin => {
                if object_type.body_extraction.is_some() {
                    diagnostics.push(diagnostic(
                        "tier3_overrides_tier2",
                        "ObjectType has both body_extraction and has_plugin=true; plugin tier is unsupported in quire-rs core extraction",
                        "warning",
                        Some(object_name.clone()),
                    ));
                }
                errors.push(format!(
                    "missing_plugin: object type {object_name} declares has_plugin=true; quire-rs core extraction does not execute Python plugins"
                ));
            }
            Some(object_type) => {
                if let Some(dsl) = object_type.body_extraction.as_ref() {
                    match extract_tier2(&doc, &frontmatter, object_type, dsl, &input.rel_path) {
                        Ok((mut extracted_nodes, mut extracted_edges, mut ds)) => {
                            nodes.append(&mut extracted_nodes);
                            tier_edges.append(&mut extracted_edges);
                            diagnostics.append(&mut ds);
                        }
                        Err(err) => errors.push(format!("tier2 extraction failed: {err}")),
                    }
                } else {
                    match extract_tier1(&frontmatter, object_type, &input.rel_path) {
                        Ok(node) => nodes.push(node),
                        Err(err) => errors.push(format!("tier1 extraction failed: {err}")),
                    }
                }
            }
        }
    }

    // Tier 0: a document that minted no object-typed node but carries a
    // frontmatter `id` is still an addressable artifact (`type: FR` +
    // `id: FR-020` spec docs). Mint its primary node here so every edge
    // harvested below gets a resolvable `ix://` source ref instead of the
    // `document:` fallback, which downstream stores cannot bind to a node.
    if nodes.is_empty() {
        if let Some(node) = primary_artifact_node(&frontmatter) {
            nodes.push(node);
        }
    }

    let primary_ref = nodes
        .first()
        .map(|n| n.name.clone())
        .unwrap_or_else(|| format!("document:{}", input.document_id));
    let mut raw_edges = Vec::new();
    match frontmatter_edges(
        &frontmatter,
        &primary_ref,
        &input.repo_name,
        &mut diagnostics,
    ) {
        Ok(mut edges) => raw_edges.append(&mut edges),
        Err(err) => errors.push(err),
    }
    raw_edges.append(&mut body_ix_edges(
        &input.markdown,
        &primary_ref,
        &mut diagnostics,
    ));
    raw_edges.append(&mut tier_edges);

    let raw_edges = dedupe_edges(raw_edges, &input.repo_name, &mut diagnostics);
    let nodes = node_records(&input, nodes);
    let edges = edge_records(&input, raw_edges);

    CoreExtractionResult {
        document_id: input.document_id,
        artifact_id: input.artifact_id,
        object_types: object_type_records,
        nodes,
        edges,
        diagnostics,
        errors,
    }
}

/// Result for a document with no frontmatter block: a clean, non-Filament
/// document. Emits only an informational `no_frontmatter` diagnostic; `errors`
/// stays empty so nothing is recorded as a parse failure downstream.
fn absent_frontmatter_result(input: FilamentExtractionInput) -> CoreExtractionResult {
    CoreExtractionResult {
        document_id: input.document_id,
        artifact_id: input.artifact_id,
        object_types: Vec::new(),
        nodes: Vec::new(),
        edges: Vec::new(),
        diagnostics: vec![diagnostic(
            "no_frontmatter",
            "Shared parser skipped markdown without parsable frontmatter",
            "info",
            None,
        )],
        errors: Vec::new(),
    }
}

/// Result for a document with a complete frontmatter fence block that failed
/// to parse into a YAML mapping. Records a `parse_failed` extraction error so
/// the failure reaches Filament's index_errors via the MarkStale→apply_stale
/// path (issue #127, CR-011), alongside a matching error-severity diagnostic.
fn malformed_frontmatter_result(input: FilamentExtractionInput) -> CoreExtractionResult {
    CoreExtractionResult {
        document_id: input.document_id,
        artifact_id: input.artifact_id,
        object_types: Vec::new(),
        nodes: Vec::new(),
        edges: Vec::new(),
        diagnostics: vec![diagnostic(
            "frontmatter_unparsable",
            "Frontmatter block present but could not be parsed as a YAML mapping",
            "error",
            None,
        )],
        errors: vec![
            "parse_failed: frontmatter block present but could not be parsed as a YAML mapping"
                .to_string(),
        ],
    }
}

fn compile_object_type_snapshots(
    object_types: &[FilamentObjectType],
    diagnostics: &mut Vec<CoreExtractionDiagnostic>,
) -> BTreeMap<String, CompiledObjectType> {
    let mut out = BTreeMap::new();
    for snapshot in object_types {
        if out.contains_key(&snapshot.name) {
            diagnostics.push(diagnostic(
                "duplicate_object_type",
                format!(
                    "duplicate object type {}; first definition wins",
                    snapshot.name
                ),
                "warning",
                Some(snapshot.name.clone()),
            ));
            continue;
        }
        let body_extraction = match snapshot.body_extraction.clone() {
            None | Some(Value::Null) => None,
            Some(value) => match serde_json::from_value::<ExtractionDsl>(value) {
                Ok(dsl) => match validate_dsl(&snapshot.name, &dsl) {
                    Ok(()) => Some(dsl),
                    Err(err) => {
                        diagnostics.push(diagnostic(
                            "body_extraction_invalid",
                            err.to_string(),
                            "error",
                            Some(snapshot.name.clone()),
                        ));
                        None
                    }
                },
                Err(err) => {
                    diagnostics.push(diagnostic(
                        "body_extraction_invalid",
                        format!("{}: body_extraction parse failed: {err}", snapshot.name),
                        "error",
                        Some(snapshot.name.clone()),
                    ));
                    None
                }
            },
        };
        out.insert(
            snapshot.name.clone(),
            CompiledObjectType {
                snapshot: snapshot.clone(),
                schema: snapshot.data_schema(),
                body_extraction,
            },
        );
    }
    out
}

fn object_type_records(
    project_id: &str,
    object_types: &[FilamentObjectType],
) -> Vec<CoreObjectTypeRecord> {
    object_types
        .iter()
        .map(|object_type| CoreObjectTypeRecord {
            id: stable_id(&[project_id, "object-type", &object_type.name]),
            project_id: project_id.to_string(),
            name: object_type.name.clone(),
            schema_json: json_string(&object_type.data_schema()),
            allowed_links_json: json_string(
                object_type
                    .allowed_links
                    .as_ref()
                    .unwrap_or(&Value::Object(Map::new())),
            ),
            body_extraction_json: object_type.body_extraction.as_ref().map(json_string),
            has_plugin: object_type.has_plugin,
            module_id: object_type.module_id.clone(),
            updated_at: None,
        })
        .collect()
}

fn extract_tier1(
    frontmatter: &Map<String, Value>,
    object_type: &CompiledObjectType,
    rel_path: &str,
) -> Result<GraphNode, String> {
    let name = frontmatter
        .get("name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| path_stem(rel_path));

    let mut data = Map::new();
    for (key, value) in frontmatter {
        if !is_reserved_frontmatter_key(key) {
            data.insert(key.clone(), value.clone());
        }
    }
    if let Some(explicit) = frontmatter.get("data") {
        let Some(map) = explicit.as_object() else {
            return Err("frontmatter `data` must be a mapping".to_string());
        };
        for (key, value) in map {
            data.insert(key.clone(), value.clone());
        }
    }
    validate_against_schema(
        &object_type.snapshot.name,
        &object_type.schema,
        &Value::Object(data.clone()),
    )?;
    apply_frontmatter_title_code(&mut data, frontmatter);
    Ok(GraphNode {
        object_type: object_type.snapshot.name.clone(),
        name,
        data,
    })
}

/// Nodes, record-derived edges, and diagnostics from one Tier 2 body extraction.
type Tier2Extraction = (
    Vec<GraphNode>,
    Vec<GraphEdge>,
    Vec<CoreExtractionDiagnostic>,
);

fn extract_tier2(
    doc: &crate::QuireDocument,
    frontmatter: &Map<String, Value>,
    object_type: &CompiledObjectType,
    dsl: &ExtractionDsl,
    rel_path: &str,
) -> Result<Tier2Extraction, String> {
    let extraction = crate::extract(doc, dsl).map_err(quire_error_message)?;
    let mut nodes = Vec::new();
    for record in &extraction.records {
        validate_against_schema(
            &object_type.snapshot.name,
            &object_type.schema,
            &Value::Object(record.clone()),
        )?;
        let mut data = record.clone();
        apply_frontmatter_title_code(&mut data, frontmatter);
        nodes.push(GraphNode {
            object_type: object_type.snapshot.name.clone(),
            name: node_name_from_record(record, rel_path),
            data,
        });
    }
    let mut edges = Vec::new();
    for edge in &extraction.edges {
        if let Some(source) = nodes.get(edge.record_index) {
            let mut metadata = Map::new();
            metadata.insert(
                "source".to_string(),
                Value::String(format!("body_extraction.emit_edges[{}]", edge.record_index)),
            );
            edges.push(GraphEdge {
                source_ref: source.name.clone(),
                edge_type: edge.edge_type.clone(),
                target_ref: edge.target.clone(),
                metadata,
            });
        }
    }
    let diagnostics = extraction
        .diagnostics
        .iter()
        .map(|d| {
            diagnostic(
                "quire_extraction",
                d.to_string(),
                "info",
                Some(object_type.snapshot.name.clone()),
            )
        })
        .collect();
    Ok((nodes, edges, diagnostics))
}

fn validate_against_schema(archetype: &str, schema: &Value, data: &Value) -> Result<(), String> {
    let validator = compile_schema(schema)
        .map_err(|err| format!("{archetype}: schema compile failed: {err}"))?;
    if let Err(mut errors) = validator.validate(data) {
        if let Some(err) = errors.next() {
            let detail = crate::validate::schema_validation_detail(&err);
            return Err(format!(
                "{archetype}: schema validation failed at {}: {}",
                detail.path, detail.message
            ));
        }
    }
    Ok(())
}

fn frontmatter_edges(
    frontmatter: &Map<String, Value>,
    primary_ref: &str,
    repo_name: &str,
    diagnostics: &mut Vec<CoreExtractionDiagnostic>,
) -> Result<Vec<GraphEdge>, String> {
    let mut edges = Vec::new();
    edges.extend(relationship_sugar_edges(
        frontmatter,
        primary_ref,
        repo_name,
        diagnostics,
    ));
    edges.extend(explicit_relationship_edges(
        frontmatter,
        primary_ref,
        repo_name,
        diagnostics,
    )?);
    edges.extend(explicit_frontmatter_edges(frontmatter, primary_ref)?);
    Ok(edges)
}

fn relationship_sugar_edges(
    frontmatter: &Map<String, Value>,
    primary_ref: &str,
    repo_name: &str,
    diagnostics: &mut Vec<CoreExtractionDiagnostic>,
) -> Vec<GraphEdge> {
    const SUGAR: &[(&str, &str, bool)] = &[
        ("depends_on", "depends_on", true),
        ("parent", "parent", false),
        ("parent_process", "parent", false),
        ("template_for", "template_for", true),
        ("archetype_for", "archetype_for", true),
        ("replaced_by", "replaced_by", false),
    ];
    let mut edges = Vec::new();
    for (key, edge_type, list) in SUGAR {
        let Some(raw) = frontmatter.get(*key) else {
            continue;
        };
        if *list {
            let items: Vec<(usize, &Value)> = match raw {
                Value::Array(values) => values.iter().enumerate().collect(),
                Value::String(_) => vec![(0, raw)],
                _ => Vec::new(),
            };
            for (index, item) in items {
                if let Some(target) = item.as_str().filter(|s| !s.trim().is_empty()) {
                    edges.push(relationship_edge(
                        primary_ref,
                        edge_type,
                        target,
                        format!("frontmatter.{key}[{index}]"),
                        repo_name,
                        diagnostics,
                    ));
                }
            }
        } else if let Some(target) = raw.as_str().filter(|s| !s.trim().is_empty()) {
            edges.push(relationship_edge(
                primary_ref,
                edge_type,
                target,
                format!("frontmatter.{key}"),
                repo_name,
                diagnostics,
            ));
        }
    }
    edges
}

fn explicit_relationship_edges(
    frontmatter: &Map<String, Value>,
    primary_ref: &str,
    repo_name: &str,
    diagnostics: &mut Vec<CoreExtractionDiagnostic>,
) -> Result<Vec<GraphEdge>, String> {
    let Some(raw) = frontmatter.get("relationships") else {
        return Ok(Vec::new());
    };
    let Some(items) = raw.as_array() else {
        return Ok(Vec::new());
    };
    let mut edges = Vec::new();
    for (index, entry) in items.iter().enumerate() {
        let Some(map) = entry.as_object() else {
            return Err(format!(
                "malformed_relationship: relationships[{index}] must be a mapping"
            ));
        };
        let target = map
            .get("target")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                format!("malformed_relationship: relationships[{index}] missing or empty `target`")
            })?;
        let edge_type = map
            .get("type")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                format!("malformed_relationship: relationships[{index}] missing or empty `type`")
            })?;
        let mut metadata = Map::new();
        metadata.insert(
            "source".to_string(),
            Value::String(format!("frontmatter.relationships[{index}]")),
        );
        metadata.insert(
            "original_target".to_string(),
            Value::String(target.to_string()),
        );
        let target_ref = normalize_ref(repo_name, target);
        if target.starts_with("ix://") && !is_valid_ix_ref(target) {
            metadata.insert("unresolved".to_string(), Value::Bool(true));
            diagnostics.push(diagnostic(
                "ix_uri_malformed",
                format!("Malformed target in frontmatter.relationships[{index}]: {target:?}"),
                "warning",
                None,
            ));
        }
        edges.push(GraphEdge {
            source_ref: primary_ref.to_string(),
            edge_type: edge_type.to_string(),
            target_ref,
            metadata,
        });
    }
    Ok(edges)
}

fn explicit_frontmatter_edges(
    frontmatter: &Map<String, Value>,
    primary_ref: &str,
) -> Result<Vec<GraphEdge>, String> {
    let Some(raw) = frontmatter.get("edges") else {
        return Ok(Vec::new());
    };
    let Some(items) = raw.as_array() else {
        return Err("malformed_edge: frontmatter `edges` must be a list".to_string());
    };
    let mut edges = Vec::new();
    for (index, entry) in items.iter().enumerate() {
        let Some(map) = entry.as_object() else {
            return Err(format!("malformed_edge: edges[{index}] must be a mapping"));
        };
        let edge_type = map
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("malformed_edge: edges[{index}] missing required `type`"))?;
        let target = map
            .get("target")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("malformed_edge: edges[{index}] missing required `target`"))?;
        let metadata = map
            .get("metadata")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        edges.push(GraphEdge {
            source_ref: primary_ref.to_string(),
            edge_type: edge_type.to_string(),
            target_ref: target.to_string(),
            metadata,
        });
    }
    Ok(edges)
}

fn relationship_edge(
    primary_ref: &str,
    edge_type: &str,
    target: &str,
    provenance: String,
    repo_name: &str,
    diagnostics: &mut Vec<CoreExtractionDiagnostic>,
) -> GraphEdge {
    let mut metadata = Map::new();
    metadata.insert("source".to_string(), Value::String(provenance.clone()));
    metadata.insert(
        "original_target".to_string(),
        Value::String(target.to_string()),
    );
    if target.starts_with("ix://") && !is_valid_ix_ref(target) {
        metadata.insert("unresolved".to_string(), Value::Bool(true));
        diagnostics.push(diagnostic(
            "ix_uri_malformed",
            format!("Malformed target in {provenance}: {target:?}"),
            "warning",
            None,
        ));
    }
    GraphEdge {
        source_ref: primary_ref.to_string(),
        edge_type: edge_type.to_string(),
        target_ref: normalize_ref(repo_name, target),
        metadata,
    }
}

fn body_ix_edges(
    markdown: &str,
    primary_ref: &str,
    diagnostics: &mut Vec<CoreExtractionDiagnostic>,
) -> Vec<GraphEdge> {
    let re = Regex::new(r#"\[([^\]]*)\]\((ix://[^)\s]+)\)"#)
        .expect("static body ix markdown-link regex is valid");
    let mut edges = Vec::new();
    for (index, captures) in re.captures_iter(markdown).enumerate() {
        let display = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
        let uri = captures.get(2).map(|m| m.as_str()).unwrap_or_default();
        if !is_valid_ix_ref(uri) {
            diagnostics.push(diagnostic(
                "ix_uri_malformed",
                format!("Skipping malformed ix:// markdown link {uri:?}"),
                "warning",
                None,
            ));
            continue;
        }
        let mut metadata = Map::new();
        metadata.insert(
            "source".to_string(),
            Value::String(format!("body.ix_link[{index}]")),
        );
        metadata.insert("original_uri".to_string(), Value::String(uri.to_string()));
        metadata.insert("display".to_string(), Value::String(display.to_string()));
        metadata.insert("unresolved".to_string(), Value::Bool(true));
        edges.push(GraphEdge {
            source_ref: primary_ref.to_string(),
            edge_type: "references".to_string(),
            target_ref: uri.to_string(),
            metadata,
        });
    }
    edges
}

fn dedupe_edges(
    edges: Vec<GraphEdge>,
    repo_name: &str,
    diagnostics: &mut Vec<CoreExtractionDiagnostic>,
) -> Vec<GraphEdge> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for edge in edges {
        let key = (
            normalize_ref(repo_name, &edge.source_ref),
            edge.edge_type.clone(),
            normalize_ref(repo_name, &edge.target_ref),
        );
        if seen.insert(key) {
            out.push(edge);
        } else {
            let dup_source = edge
                .metadata
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            diagnostics.push(diagnostic(
                "duplicate_edge",
                format!(
                    "duplicate edge {:?} -> {:?}: dropped {dup_source}",
                    edge.edge_type, edge.target_ref
                ),
                "warning",
                None,
            ));
        }
    }
    out
}

fn node_records(input: &FilamentExtractionInput, nodes: Vec<GraphNode>) -> Vec<CoreGraphNodeRef> {
    nodes
        .into_iter()
        .map(|node| {
            let ref_ = normalize_ref(&input.repo_name, &node.name);
            CoreGraphNodeRef {
                id: stable_id(&[&input.project_id, "graph-node", &ref_]),
                project_id: input.project_id.clone(),
                document_id: input.document_id.clone(),
                artifact_id: input.artifact_id.clone(),
                object_type: node.object_type,
                name: node.name,
                ref_,
                data_json: json_string(&Value::Object(node.data)),
                updated_at: None,
            }
        })
        .collect()
}

fn edge_records(input: &FilamentExtractionInput, edges: Vec<GraphEdge>) -> Vec<CoreGraphEdgeRef> {
    edges
        .into_iter()
        .map(|edge| {
            let source_ref = normalize_ref(&input.repo_name, &edge.source_ref);
            let target_ref = normalize_ref(&input.repo_name, &edge.target_ref);
            CoreGraphEdgeRef {
                id: stable_id(&[
                    &input.project_id,
                    "graph-edge",
                    &source_ref,
                    &target_ref,
                    &edge.edge_type,
                ]),
                project_id: input.project_id.clone(),
                source_ref,
                target_ref,
                edge_type: edge.edge_type,
                data_json: json_string(&Value::Object(edge.metadata)),
                updated_at: None,
            }
        })
        .collect()
}

fn apply_frontmatter_title_code(data: &mut Map<String, Value>, frontmatter: &Map<String, Value>) {
    if let Some(id) = frontmatter.get("id") {
        data.insert("code".to_string(), Value::String(value_to_string(id)));
    }
    if let Some(title) = frontmatter.get("title") {
        data.insert("title".to_string(), Value::String(value_to_string(title)));
    }
}

/// Tier-0 primary node for an identified artifact document: `name` is the
/// frontmatter `id`, typed `artifact`, carrying the frontmatter `type`
/// alongside the usual code/title projection. `None` when the document has
/// no usable `id` — only scalar string/number ids qualify; `null`, booleans,
/// lists, and maps would stringify into colliding or junk refs (every
/// blank-`id:` stub would share one `.../null` node), so those keep the
/// `document:` primary-ref fallback.
fn primary_artifact_node(frontmatter: &Map<String, Value>) -> Option<GraphNode> {
    let raw = frontmatter.get("id")?;
    if !matches!(raw, Value::String(_) | Value::Number(_)) {
        return None;
    }
    let id = value_to_string(raw).trim().to_string();
    if id.is_empty() {
        return None;
    }
    let mut data = Map::new();
    apply_frontmatter_title_code(&mut data, frontmatter);
    data.insert("code".to_string(), Value::String(id.clone()));
    if let Some(artifact_type) = frontmatter.get("type") {
        data.insert(
            "artifact_type".to_string(),
            Value::String(value_to_string(artifact_type)),
        );
    }
    Some(GraphNode {
        object_type: "artifact".to_string(),
        name: id,
        data,
    })
}

fn node_name_from_record(record: &Map<String, Value>, rel_path: &str) -> String {
    record
        .get("name")
        .or_else(|| record.get("heading"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| path_stem(rel_path))
}

fn path_stem(rel_path: &str) -> String {
    Path::new(rel_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(rel_path)
        .to_string()
}

fn is_reserved_frontmatter_key(key: &str) -> bool {
    matches!(
        key,
        "object" | "id" | "name" | "tags" | "edges" | "relationships" | "parent" | "data"
    )
}

fn normalize_ref(repo_name: &str, value: &str) -> String {
    if value.starts_with("ix://") {
        value.to_string()
    } else {
        format!("ix://agent-ix/{repo_name}/{value}")
    }
}

fn is_valid_ix_ref(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("ix://") else {
        return false;
    };
    let mut parts = rest.split('/');
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(org), Some(repo), Some(name))
            if !org.is_empty() && !repo.is_empty() && !name.is_empty()
    )
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn json_string(value: &Value) -> String {
    serde_json::to_string(value).expect("serializing serde_json::Value cannot fail")
}

fn quire_error_message(error: QuireError) -> String {
    error.to_string()
}

fn diagnostic(
    code: impl Into<String>,
    message: impl Into<String>,
    severity: impl Into<String>,
    object_type: Option<String>,
) -> CoreExtractionDiagnostic {
    CoreExtractionDiagnostic {
        code: code.into(),
        message: message.into(),
        severity: severity.into(),
        object_type,
    }
}

fn stable_id(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for (idx, part) in parts.iter().enumerate() {
        if idx > 0 {
            hasher.update([0]);
        }
        hasher.update(part.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input(markdown: &str) -> FilamentExtractionInput {
        FilamentExtractionInput {
            project_id: "project".to_string(),
            document_id: "doc-1".to_string(),
            artifact_id: Some("artifact-1".to_string()),
            rel_path: "spec/FR-001.md".to_string(),
            markdown: markdown.to_string(),
            repo_name: "example".to_string(),
            object_types: vec![FilamentObjectType {
                name: "capability".to_string(),
                schema: Some(json!({"type": "object", "additionalProperties": true})),
                data_schema: None,
                allowed_links: Some(json!({})),
                body_extraction: None,
                has_plugin: false,
                module_id: None,
            }],
        }
    }

    #[test]
    fn tc681_tier1_frontmatter_extracts_core_node() {
        let result = extract_filament_core(base_input(
            "---\nid: FR-001\ntitle: Pay vendors\nobject: capability\npriority: P0\n---\n# Body\n",
        ));
        assert_eq!(result.errors, Vec::<String>::new());
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].object_type, "capability");
        assert_eq!(result.nodes[0].name, "FR-001");
        assert_eq!(result.nodes[0].ref_, "ix://agent-ix/example/FR-001");
        let data: Value = serde_json::from_str(&result.nodes[0].data_json).unwrap();
        assert_eq!(data["code"], "FR-001");
        assert_eq!(data["title"], "Pay vendors");
        assert_eq!(data["priority"], "P0");
    }

    #[test]
    fn tc682_tier2_uses_rust_dsl_records_and_edges() {
        let mut input = base_input(
            "---\nid: API-001\ntitle: Payments API\nobject: endpoint\n---\n## Endpoint\nGET /payments\n",
        );
        input.object_types[0] = FilamentObjectType {
            name: "endpoint".to_string(),
            schema: Some(json!({
                "type": "object",
                "required": ["method", "target"],
                "properties": {
                    "method": {"type": "string"},
                    "target": {"type": "string"}
                }
            })),
            data_schema: None,
            allowed_links: Some(json!({})),
            body_extraction: Some(json!({
                "yield_pattern": {
                    "match": {
                        "method": {
                            "from": "section_body",
                            "after_heading": "Endpoint",
                            "regex": "^(GET)",
                            "required": true
                        },
                        "target": {
                            "from": "section_body",
                            "after_heading": "Endpoint",
                            "regex": "(/payments)",
                            "required": true
                        }
                    }
                },
                "emit_edges": [{"type": "serves", "target": "ix://agent-ix/example/API"}]
            })),
            has_plugin: false,
            module_id: None,
        };
        let result = extract_filament_core(input);
        assert_eq!(result.errors, Vec::<String>::new());
        assert_eq!(result.nodes.len(), 1);
        let data: Value = serde_json::from_str(&result.nodes[0].data_json).unwrap();
        assert_eq!(data["method"], "GET");
        assert_eq!(data["target"], "/payments");
        assert!(result.edges.iter().any(
            |edge| edge.edge_type == "serves" && edge.target_ref == "ix://agent-ix/example/API"
        ));
    }

    #[test]
    fn tc683_error_fixtures_are_reported_without_panic() {
        let no_frontmatter = extract_filament_core(base_input("# Body\n"));
        assert!(no_frontmatter.nodes.is_empty());
        assert_eq!(no_frontmatter.diagnostics[0].code, "no_frontmatter");

        let unknown = extract_filament_core(base_input(
            "---\nid: FR-001\nobject: missing\n---\n# Body\n",
        ));
        // The unknown object type is still surfaced, but the identified
        // document now falls back to its tier-0 primary artifact node
        // instead of extracting nothing.
        assert_eq!(unknown.nodes.len(), 1);
        assert_eq!(unknown.nodes[0].object_type, "artifact");
        assert!(unknown
            .diagnostics
            .iter()
            .any(|d| d.code == "unknown_object_type"));

        let mut plugin = base_input("---\nid: FR-001\nobject: capability\n---\n# Body\n");
        plugin.object_types[0].has_plugin = true;
        let plugin = extract_filament_core(plugin);
        assert!(plugin
            .errors
            .iter()
            .any(|err| err.contains("missing_plugin")));

        let duplicate = extract_filament_core(base_input(
            "---\nid: FR-001\nobject: capability\nrelationships:\n  - target: ix://agent-ix/example/FR-002\n    type: references\n---\nSee [FR-002](ix://agent-ix/example/FR-002).\n",
        ));
        assert_eq!(duplicate.edges.len(), 1);
        assert!(duplicate
            .diagnostics
            .iter()
            .any(|d| d.code == "duplicate_edge"));

        let malformed = extract_filament_core(base_input(
            "---\nid: FR-001\nobject: capability\nrelationships:\n  - target: ix://agent-ix\n    type: references\n---\n# Body\n",
        ));
        assert!(malformed
            .diagnostics
            .iter()
            .any(|d| d.code == "ix_uri_malformed"));
    }

    #[test]
    fn tc705_malformed_frontmatter_is_a_parse_failure_but_absent_is_clean() {
        // Complete `---` … `---` fence block with unparseable YAML → parse
        // failure: a non-empty `errors` entry (routed to Filament index_errors
        // via MarkStale→apply_stale) plus a `frontmatter_unparsable` diagnostic.
        let malformed = extract_filament_core(base_input("---\nid: : bad\n---\n# Body\n"));
        assert!(malformed.nodes.is_empty());
        assert!(malformed.edges.is_empty());
        assert!(
            malformed
                .errors
                .iter()
                .any(|err| err.contains("parse_failed")),
            "expected parse_failed error; got {:?}",
            malformed.errors
        );
        assert!(malformed
            .diagnostics
            .iter()
            .any(|d| d.code == "frontmatter_unparsable" && d.severity == "error"));

        // No frontmatter block at all → clean skip: empty errors, only the
        // informational `no_frontmatter` diagnostic.
        let absent = extract_filament_core(base_input("# Body\n"));
        assert_eq!(absent.errors, Vec::<String>::new());
        assert!(absent
            .diagnostics
            .iter()
            .any(|d| d.code == "no_frontmatter"));
    }

    #[test]
    fn tc684_relationship_sugar_and_body_ix_links_have_metadata() {
        let result = extract_filament_core(base_input(
            "---\nid: FR-001\nobject: capability\ndepends_on:\n  - FR-002\n---\nSee [US-001](ix://agent-ix/example/US-001).\n",
        ));
        assert_eq!(result.errors, Vec::<String>::new());
        assert!(result
            .edges
            .iter()
            .any(|edge| edge.edge_type == "depends_on"
                && edge.target_ref == "ix://agent-ix/example/FR-002"
                && edge.data_json.contains("original_target")));
        assert!(result
            .edges
            .iter()
            .any(|edge| edge.edge_type == "references"
                && edge.target_ref == "ix://agent-ix/example/US-001"
                && edge.data_json.contains("original_uri")));
    }

    #[test]
    fn tc706_identified_document_without_object_mints_primary_artifact_node() {
        let result = extract_filament_core(base_input(
            "---\nid: FR-020\ntype: FR\ntitle: Persist settings\nrelationships:\n  - type: implements\n    target: ix://agent-ix/example/US-012\n---\nSee [NFR-016](ix://agent-ix/example/NFR-016).\n",
        ));
        assert_eq!(result.errors, Vec::<String>::new());
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].object_type, "artifact");
        assert_eq!(result.nodes[0].name, "FR-020");
        assert_eq!(result.nodes[0].ref_, "ix://agent-ix/example/FR-020");
        let data: Value = serde_json::from_str(&result.nodes[0].data_json).unwrap();
        assert_eq!(data["code"], "FR-020");
        assert_eq!(data["title"], "Persist settings");
        assert_eq!(data["artifact_type"], "FR");
        // Every harvested edge is sourced from the minted primary ref, not a
        // `document:` fallback.
        assert!(!result.edges.is_empty());
        for edge in &result.edges {
            assert_eq!(edge.source_ref, "ix://agent-ix/example/FR-020");
        }
    }

    #[test]
    fn tc707_document_without_id_keeps_document_fallback_source() {
        let result = extract_filament_core(base_input(
            "---\ntitle: Notes\n---\nSee [US-001](ix://agent-ix/example/US-001).\n",
        ));
        assert_eq!(result.errors, Vec::<String>::new());
        assert_eq!(result.nodes.len(), 0);
        assert_eq!(result.edges.len(), 1);
        assert_eq!(
            result.edges[0].source_ref,
            "ix://agent-ix/example/document:doc-1"
        );
    }

    #[test]
    fn tc708_object_typed_document_does_not_mint_extra_artifact_node() {
        let result = extract_filament_core(base_input(
            "---\nid: FR-001\ntitle: Pay vendors\nobject: capability\n---\n# Body\n",
        ));
        assert_eq!(result.errors, Vec::<String>::new());
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].object_type, "capability");
    }

    #[test]
    fn tc709_non_scalar_or_blank_ids_do_not_mint_a_primary_node() {
        for markdown in [
            "---\nid:\n---\nSee [US-001](ix://agent-ix/example/US-001).\n",
            "---\nid: \"\"\n---\nSee [US-001](ix://agent-ix/example/US-001).\n",
            "---\nid: false\n---\nSee [US-001](ix://agent-ix/example/US-001).\n",
            "---\nid: [a, b]\n---\nSee [US-001](ix://agent-ix/example/US-001).\n",
            "---\nid: {a: b}\n---\nSee [US-001](ix://agent-ix/example/US-001).\n",
        ] {
            let result = extract_filament_core(base_input(markdown));
            assert_eq!(result.nodes.len(), 0, "no node for {markdown:?}");
            assert_eq!(
                result.edges[0].source_ref, "ix://agent-ix/example/document:doc-1",
                "fallback source for {markdown:?}"
            );
        }
        // Numeric ids are scalar and usable.
        let numeric = extract_filament_core(base_input("---\nid: 123\n---\n# Body\n"));
        assert_eq!(numeric.nodes.len(), 1);
        assert_eq!(numeric.nodes[0].name, "123");
    }

    #[test]
    fn tc685_repeated_extraction_is_byte_identical() {
        let input = base_input(
            "---\nid: FR-001\ntitle: Pay vendors\nobject: capability\ndepends_on:\n  - FR-002\n---\nSee [US-001](ix://agent-ix/example/US-001).\n",
        );
        let first = extract_filament_core(input.clone());
        let second = extract_filament_core(input);
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
    }
}
