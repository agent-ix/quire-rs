//! Versioned, source-grounded assurance export (FR-067 / FR-068).
//!
//! This module is a pure projection over records the engine already owns. It
//! does not walk a repository, parse Markdown/frontmatter, harvest source tags,
//! invoke Git, access the network, persist state, or assign assurance verdicts.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use jsonschema::{Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::corpus::declared_tables::ExcludeSet;
use crate::corpus::{Resolution, Spec};
use crate::diagnostic::Diagnostic;
use crate::registry::Registry;
use crate::symbols::trace::SymbolGraph;
use crate::symbols::SymbolExtraction;
use crate::traceability::RelationDirection;

/// Published, hand-authored v1 contract.
pub const ASSURANCE_V1_SCHEMA: &str = include_str!("../schemas/output/assurance-v1.schema.json");

/// Caller-selected immutable repository identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssuranceSource {
    pub repository: String,
    pub revision: String,
}

/// One active archetype's semantic schema premise.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssuranceSchemaPremise {
    pub archetype: String,
    pub schema_digest: String,
}

/// One loaded module and its first-wins active archetypes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssuranceModulePremise {
    pub name: String,
    pub version: String,
    pub schemas: Vec<AssuranceSchemaPremise>,
}

/// Exact source locus for a projected record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssuranceLocator {
    pub path: String,
    pub line: usize,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssuranceArtifact {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    pub artifact_type: String,
    pub locator: AssuranceLocator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssuranceObligation {
    pub source: String,
    pub id: String,
    pub document: String,
    pub statement: String,
    pub statement_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criticality: Option<String>,
    pub target_ids: Vec<String>,
    pub locator: AssuranceLocator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssuranceSymbol {
    pub id: String,
    pub language: String,
    pub kind: String,
    pub qualified_name: String,
    pub container: Option<String>,
    pub capabilities: Vec<String>,
    pub locator: AssuranceLocator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKindAvailability {
    Available,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKindSource {
    ModuleVocabulary,
    RequiredRelation,
    TraceBinding,
    Observed,
}

/// One relation capability available in this bounded export.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssuranceRelationKind {
    pub kind: String,
    pub availability: RelationKindAvailability,
    pub sources: Vec<RelationKindSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceFreshness {
    Current,
    Suspect,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssuranceRelation {
    Corpus {
        source: String,
        target: String,
        edge_type: String,
        resolution: AssuranceResolution,
        locator: AssuranceLocator,
        freshness: AssuranceFreshness,
    },
    Verifies {
        source: String,
        target: String,
        form: String,
        provenance: String,
        locator: AssuranceLocator,
        freshness: AssuranceFreshness,
    },
    Implements {
        source: String,
        target: String,
        form: String,
        locator: AssuranceLocator,
        freshness: AssuranceFreshness,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceResolution {
    Resolved,
    Dangling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationAvailability {
    Available,
    Missing,
    NotApplicable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssuranceRelationObservation {
    pub declaration: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    pub availability: RelationAvailability,
    pub freshness: AssuranceFreshness,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Complete v1 interchange payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssuranceExport {
    pub format: String,
    pub format_version: u32,
    pub source: AssuranceSource,
    pub modules: Vec<AssuranceModulePremise>,
    pub artifacts: Vec<AssuranceArtifact>,
    pub obligations: Vec<AssuranceObligation>,
    pub symbols: Vec<AssuranceSymbol>,
    pub relation_kinds: Vec<AssuranceRelationKind>,
    pub relations: Vec<AssuranceRelation>,
    pub relation_observations: Vec<AssuranceRelationObservation>,
}

impl AssuranceExport {
    /// Serialize with deterministic field and collection order.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, AssuranceError> {
        serde_json::to_vec(self).map_err(|error| AssuranceError::Serialization {
            reason: error.to_string(),
        })
    }
}

/// Premises a caller is prepared to consume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedAssurancePremises {
    pub format_version: u32,
    pub modules: Vec<AssuranceModulePremise>,
}

impl AcceptedAssurancePremises {
    pub fn from_export(export: &AssuranceExport) -> Self {
        Self {
            format_version: export.format_version,
            modules: export.modules.clone(),
        }
    }
}

/// Authoritative inputs to export construction.
pub struct AssuranceInput<'a> {
    pub spec: &'a Spec,
    pub registry: &'a Registry,
    pub corpus_root: &'a Path,
    pub symbols: &'a SymbolExtraction,
    pub symbol_graph: &'a SymbolGraph,
    pub source: AssuranceSource,
}

/// Fail-closed error from export construction or import.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AssuranceError {
    #[error("assurance source repository is empty")]
    EmptyRepository,
    #[error("assurance source revision is not a full lowercase Git object id: {revision}")]
    InvalidRevision { revision: String },
    #[error("loaded module has no authored name: {path}")]
    MissingModuleName { path: String },
    #[error("loaded module '{module}' has no declared version")]
    MissingModuleVersion { module: String },
    #[error("archetype '{archetype}' in module '{module}' did not load: {reason}")]
    ArchetypeLoadFailure {
        module: String,
        archetype: String,
        reason: String,
    },
    #[error("artifact '{artifact}' has no authored id")]
    MissingArtifactId { artifact: String },
    #[error("artifact '{artifact}' has no authored type")]
    MissingArtifactType { artifact: String },
    #[error("source path '{}' is outside corpus root '{}'", path.display(), root.display())]
    PathOutsideRoot { path: PathBuf, root: PathBuf },
    #[error("source bytes unavailable for '{path}'")]
    MissingSourceBytes { path: String },
    #[error("source statement for obligation '{obligation}' was not found in '{path}'")]
    MissingStatement { obligation: String, path: String },
    #[error("invalid assurance JSON: {reason}")]
    InvalidJson { reason: String },
    #[error("unsupported assurance format '{format}'")]
    UnsupportedFormat { format: String },
    #[error("unsupported assurance format_version {version}")]
    UnsupportedFormatVersion { version: u64 },
    #[error("assurance-v1 schema violation: {reason}")]
    SchemaViolation { reason: String },
    #[error("module '{module}' is not in the accepted premise set")]
    UnacceptedModule { module: String },
    #[error("module '{module}' version '{version}' is not accepted")]
    UnacceptedModuleVersion { module: String, version: String },
    #[error("module '{module}' archetype '{archetype}' schema digest '{digest}' is not accepted")]
    UnacceptedSchemaDigest {
        module: String,
        archetype: String,
        digest: String,
    },
    #[error("assurance premise set contains duplicate module '{module}'")]
    DuplicateModulePremise { module: String },
    #[error("assurance module '{module}' contains duplicate archetype '{archetype}'")]
    DuplicateSchemaPremise { module: String, archetype: String },
    #[error("assurance serialization failed: {reason}")]
    Serialization { reason: String },
}

/// Build a complete v1 export without external I/O.
pub fn build_assurance_export(
    input: AssuranceInput<'_>,
) -> Result<AssuranceExport, AssuranceError> {
    validate_source(&input.source)?;
    validate_registry(input.registry)?;

    let modules = module_premises(input.registry)?;
    let artifacts = project_artifacts(input.spec, input.corpus_root)?;
    let obligations = project_obligations(input.spec, input.registry, input.corpus_root)?;
    let symbols = project_symbols(input.symbols)?;
    let mut relations = project_relations(
        input.spec,
        input.corpus_root,
        input.symbols,
        input.symbol_graph,
    )?;
    relations.sort_by(|left, right| relation_key(left).cmp(&relation_key(right)));
    let relation_kinds = project_relation_kinds(input.registry, &relations);
    let relation_observations =
        project_relation_observations(input.spec, input.registry, input.corpus_root)?;

    let export = AssuranceExport {
        format: "quire-assurance".to_string(),
        format_version: 1,
        source: input.source,
        modules,
        artifacts,
        obligations,
        symbols,
        relation_kinds,
        relations,
        relation_observations,
    };
    validate_v1_value(&serde_json::to_value(&export).map_err(|error| {
        AssuranceError::Serialization {
            reason: error.to_string(),
        }
    })?)?;
    Ok(export)
}

/// Validate and import a v1 export, returning no typed record until the full
/// document and every caller premise have passed.
pub fn read_assurance_export(
    bytes: &[u8],
    accepted: &AcceptedAssurancePremises,
) -> Result<AssuranceExport, AssuranceError> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|error| AssuranceError::InvalidJson {
            reason: error.to_string(),
        })?;
    let format = value
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if format != "quire-assurance" {
        return Err(AssuranceError::UnsupportedFormat {
            format: format.to_string(),
        });
    }
    let version = value
        .get("format_version")
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX);
    if version != 1 || accepted.format_version != 1 {
        return Err(AssuranceError::UnsupportedFormatVersion { version });
    }
    validate_v1_value(&value)?;
    let export: AssuranceExport =
        serde_json::from_value(value).map_err(|error| AssuranceError::InvalidJson {
            reason: error.to_string(),
        })?;
    validate_accepted_premises(&export, accepted)?;
    Ok(export)
}

fn validate_source(source: &AssuranceSource) -> Result<(), AssuranceError> {
    if source.repository.is_empty() {
        return Err(AssuranceError::EmptyRepository);
    }
    let valid_revision = source.revision.len() == 40
        && source
            .revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid_revision {
        return Err(AssuranceError::InvalidRevision {
            revision: source.revision.clone(),
        });
    }
    Ok(())
}

fn validate_registry(registry: &Registry) -> Result<(), AssuranceError> {
    for diagnostic in registry.diagnostics() {
        if let Diagnostic::ManifestMissingName { path, .. } = diagnostic {
            return Err(AssuranceError::MissingModuleName {
                path: path.display().to_string(),
            });
        }
    }
    if let Some(failure) = registry.failures().first() {
        return Err(AssuranceError::ArchetypeLoadFailure {
            module: failure.module.clone(),
            archetype: failure.archetype.clone(),
            reason: failure.reason.clone(),
        });
    }
    Ok(())
}

fn module_premises(registry: &Registry) -> Result<Vec<AssuranceModulePremise>, AssuranceError> {
    let mut modules: BTreeMap<String, AssuranceModulePremise> = BTreeMap::new();
    for name in registry.module_names() {
        let version = registry
            .module_version(name)
            .filter(|version| !version.is_empty())
            .ok_or_else(|| AssuranceError::MissingModuleVersion {
                module: name.to_string(),
            })?;
        modules.insert(
            name.to_string(),
            AssuranceModulePremise {
                name: name.to_string(),
                version: version.to_string(),
                schemas: Vec::new(),
            },
        );
    }
    for archetype in registry.active_archetypes() {
        let Some(module) = modules.get_mut(&archetype.module) else {
            return Err(AssuranceError::MissingModuleVersion {
                module: archetype.module.clone(),
            });
        };
        module.schemas.push(AssuranceSchemaPremise {
            archetype: archetype.name.clone(),
            schema_digest: match &archetype.semantic_schema_digest {
                // FR-069: the digest over the shipped schema bytes is the one
                // tuple; no second digest is computed.
                Some(d) => d.trim_start_matches("sha256:").to_string(),
                None => digest_json(&archetype.raw_schema)?,
            },
        });
    }
    for module in modules.values_mut() {
        module.schemas.sort();
        module.schemas.dedup();
    }
    Ok(modules.into_values().collect())
}

fn project_artifacts(spec: &Spec, root: &Path) -> Result<Vec<AssuranceArtifact>, AssuranceError> {
    let mut artifacts = Vec::with_capacity(spec.len());
    for document in spec.documents() {
        let path = relative_path(root, &document.path)?;
        if document.id.is_empty() {
            return Err(AssuranceError::MissingArtifactId { artifact: path });
        }
        let artifact_type = document
            .concept_type()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| AssuranceError::MissingArtifactType {
                artifact: document.id.clone(),
            })?;
        artifacts.push(AssuranceArtifact {
            id: document.id.clone(),
            uuid: document.uuid.map(|value| value.to_string()),
            artifact_type: artifact_type.to_string(),
            locator: AssuranceLocator {
                path,
                line: 1,
                digest: digest_bytes(document.raw().as_bytes()),
            },
        });
    }
    artifacts.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then(left.locator.cmp(&right.locator))
    });
    Ok(artifacts)
}

fn project_obligations(
    spec: &Spec,
    registry: &Registry,
    root: &Path,
) -> Result<Vec<AssuranceObligation>, AssuranceError> {
    let Some(model) = registry.traceability() else {
        return Ok(Vec::new());
    };
    let (source, _) = crate::obligation::derive(spec, root, model);
    let mut obligations = Vec::with_capacity(source.len());
    for obligation in source {
        let document = spec
            .documents()
            .iter()
            .find(|document| {
                relative_path(root, &document.path).is_ok_and(|path| path == obligation.document)
            })
            .ok_or_else(|| AssuranceError::MissingSourceBytes {
                path: obligation.document.clone(),
            })?;
        let line =
            first_line_containing(document.raw(), &obligation.statement).ok_or_else(|| {
                AssuranceError::MissingStatement {
                    obligation: obligation.id.clone(),
                    path: obligation.document.clone(),
                }
            })?;
        obligations.push(AssuranceObligation {
            source: obligation.source,
            id: obligation.id,
            document: obligation.document.clone(),
            statement: obligation.statement.clone(),
            statement_hash: obligation.statement_hash,
            method: obligation.method,
            parameters: obligation.parameters,
            criticality: obligation.criticality,
            target_ids: obligation.target_ids,
            locator: AssuranceLocator {
                path: obligation.document,
                line,
                digest: digest_bytes(obligation.statement.as_bytes()),
            },
        });
    }
    obligations.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then(left.document.cmp(&right.document))
    });
    Ok(obligations)
}

fn project_symbols(extraction: &SymbolExtraction) -> Result<Vec<AssuranceSymbol>, AssuranceError> {
    let mut symbols = Vec::with_capacity(extraction.symbols.len());
    for symbol in &extraction.symbols {
        let path = source_relative_path(&symbol.path)?;
        let source = extraction
            .source_of(&symbol.path)
            .ok_or_else(|| AssuranceError::MissingSourceBytes { path: path.clone() })?;
        let mut capabilities = Vec::with_capacity(1);
        if symbol.kind.binds_trace_ids() {
            capabilities.push("verifies".to_string());
        }
        if symbol.kind.carries_implements() {
            capabilities.push("implements".to_string());
        }
        symbols.push(AssuranceSymbol {
            id: symbol.id.clone(),
            language: symbol.language.as_str().to_string(),
            kind: symbol.kind.as_str().to_string(),
            qualified_name: symbol.qualified_name.clone(),
            container: symbol.container.clone(),
            capabilities,
            locator: AssuranceLocator {
                path,
                line: symbol.line,
                digest: digest_bytes(source.as_bytes()),
            },
        });
    }
    symbols.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(symbols)
}

fn project_relations(
    spec: &Spec,
    root: &Path,
    extraction: &SymbolExtraction,
    graph: &SymbolGraph,
) -> Result<Vec<AssuranceRelation>, AssuranceError> {
    let mut relations =
        Vec::with_capacity(spec.edges().len() + graph.verifies.len() + graph.implements.len());
    for edge in spec.edges() {
        let document =
            spec.by_id(&edge.source)
                .ok_or_else(|| AssuranceError::MissingSourceBytes {
                    path: edge.source.clone(),
                })?;
        let path = relative_path(root, &document.path)?;
        relations.push(AssuranceRelation::Corpus {
            source: edge.source.clone(),
            target: edge.target.clone(),
            edge_type: edge.edge_type.clone(),
            resolution: match edge.resolution {
                Resolution::Resolved => AssuranceResolution::Resolved,
                Resolution::Dangling => AssuranceResolution::Dangling,
            },
            locator: AssuranceLocator {
                path,
                line: first_line_containing(document.raw(), &edge.target).unwrap_or(1),
                digest: digest_bytes(document.raw().as_bytes()),
            },
            freshness: AssuranceFreshness::NotApplicable,
        });
    }
    for relation in &graph.verifies {
        relations.push(AssuranceRelation::Verifies {
            source: relation.symbol_id.clone(),
            target: relation.trace_id.clone(),
            form: relation.form.clone(),
            provenance: relation.provenance.as_str().to_string(),
            locator: symbol_locator(extraction, &relation.path, relation.line)?,
            freshness: AssuranceFreshness::Unknown,
        });
    }
    for relation in &graph.implements {
        let line = extraction
            .symbols
            .iter()
            .find(|symbol| symbol.id == relation.symbol_id)
            .map(|symbol| symbol.line)
            .ok_or_else(|| AssuranceError::MissingSourceBytes {
                path: relation.path.clone(),
            })?;
        relations.push(AssuranceRelation::Implements {
            source: relation.symbol_id.clone(),
            target: relation.trace_id.clone(),
            form: relation.form.clone(),
            locator: symbol_locator(extraction, &relation.path, line)?,
            freshness: AssuranceFreshness::NotApplicable,
        });
    }
    Ok(relations)
}

fn project_relation_kinds(
    registry: &Registry,
    relations: &[AssuranceRelation],
) -> Vec<AssuranceRelationKind> {
    let mut kinds: BTreeMap<String, BTreeSet<RelationKindSource>> = BTreeMap::new();
    for kind in registry.edge_types().keys() {
        kinds
            .entry(kind.clone())
            .or_default()
            .insert(RelationKindSource::ModuleVocabulary);
    }
    if let Some(model) = registry.traceability() {
        for relation in &model.required_relations {
            for kind in &relation.edges {
                kinds
                    .entry(kind.clone())
                    .or_default()
                    .insert(RelationKindSource::RequiredRelation);
            }
        }
        if !model.trace_tags.markers.is_empty() || !model.trace_tags.legacy.is_empty() {
            kinds
                .entry("verifies".to_string())
                .or_default()
                .insert(RelationKindSource::TraceBinding);
        }
        if !model.trace_tags.implements.is_empty() {
            kinds
                .entry("implements".to_string())
                .or_default()
                .insert(RelationKindSource::TraceBinding);
        }
    }
    for relation in relations {
        let kind = match relation {
            AssuranceRelation::Corpus { edge_type, .. } => edge_type,
            AssuranceRelation::Verifies { .. } => "verifies",
            AssuranceRelation::Implements { .. } => "implements",
        };
        kinds
            .entry(kind.to_string())
            .or_default()
            .insert(RelationKindSource::Observed);
    }
    kinds
        .into_iter()
        .map(|(kind, sources)| AssuranceRelationKind {
            kind,
            availability: RelationKindAvailability::Available,
            sources: sources.into_iter().collect(),
        })
        .collect()
}

fn project_relation_observations(
    spec: &Spec,
    registry: &Registry,
    root: &Path,
) -> Result<Vec<AssuranceRelationObservation>, AssuranceError> {
    let Some(model) = registry.traceability() else {
        return Ok(Vec::new());
    };
    let model_exclude = ExcludeSet::compile_validated(&model.exclude);
    let mut observations = Vec::new();
    for relation in &model.required_relations {
        let relation_exclude = ExcludeSet::compile_validated(&relation.exclude);
        let subjects: Vec<_> = spec
            .by_type(&relation.from)
            .into_iter()
            .filter(|document| {
                !model_exclude.excludes(root, &document.path)
                    && !relation_exclude.excludes(root, &document.path)
                    && !document.id.is_empty()
            })
            .collect();
        if subjects.is_empty() {
            observations.push(AssuranceRelationObservation {
                declaration: relation.name.clone(),
                subject: None,
                availability: RelationAvailability::NotApplicable,
                freshness: AssuranceFreshness::NotApplicable,
                reason: None,
            });
        } else {
            for subject in subjects {
                observations.push(AssuranceRelationObservation {
                    declaration: relation.name.clone(),
                    subject: Some(subject.id.clone()),
                    availability: if required_relation_satisfied(spec, relation, &subject.id) {
                        RelationAvailability::Available
                    } else {
                        RelationAvailability::Missing
                    },
                    freshness: AssuranceFreshness::NotApplicable,
                    reason: None,
                });
            }
        }
        for diagnostic in spec.diagnostics() {
            if let Diagnostic::DocumentUnreadable { path, reason } = diagnostic {
                observations.push(AssuranceRelationObservation {
                    declaration: relation.name.clone(),
                    subject: Some(relative_path(root, path)?),
                    availability: RelationAvailability::Unknown,
                    freshness: AssuranceFreshness::Unknown,
                    reason: Some(reason.clone()),
                });
            }
        }
    }
    observations.sort_by(|left, right| {
        left.declaration
            .cmp(&right.declaration)
            .then(left.subject.cmp(&right.subject))
            .then(left.availability.cmp(&right.availability))
    });
    observations.dedup();
    Ok(observations)
}

fn required_relation_satisfied(
    spec: &Spec,
    relation: &crate::traceability::RequiredRelation,
    subject: &str,
) -> bool {
    let accepts = |other: &str, resolution: Resolution| {
        relation.to.is_empty()
            || (resolution == Resolution::Resolved
                && spec
                    .by_id(other)
                    .and_then(|document| document.concept_type())
                    .is_some_and(|kind| relation.to.iter().any(|target| target == kind)))
    };
    let outgoing = || {
        spec.outgoing(subject).iter().any(|edge| {
            relation.edges.contains(&edge.edge_type) && accepts(&edge.target, edge.resolution)
        })
    };
    let incoming = || {
        spec.referencing(subject).iter().any(|edge| {
            relation.edges.contains(&edge.edge_type) && accepts(&edge.source, Resolution::Resolved)
        })
    };
    match relation.direction {
        RelationDirection::Outgoing => outgoing(),
        RelationDirection::Incoming => incoming(),
        RelationDirection::Either => outgoing() || incoming(),
    }
}

fn symbol_locator(
    extraction: &SymbolExtraction,
    path: &str,
    line: usize,
) -> Result<AssuranceLocator, AssuranceError> {
    let path = source_relative_path(path)?;
    let source = extraction
        .source_of(&path)
        .ok_or_else(|| AssuranceError::MissingSourceBytes { path: path.clone() })?;
    Ok(AssuranceLocator {
        path,
        line,
        digest: digest_bytes(source.as_bytes()),
    })
}

fn relation_key(relation: &AssuranceRelation) -> (u8, &str, &str, &str) {
    match relation {
        AssuranceRelation::Corpus {
            source,
            target,
            edge_type,
            ..
        } => (0, source, target, edge_type),
        AssuranceRelation::Verifies {
            source,
            target,
            form,
            ..
        } => (1, source, target, form),
        AssuranceRelation::Implements {
            source,
            target,
            form,
            ..
        } => (2, source, target, form),
    }
}

fn validate_v1_value(value: &Value) -> Result<(), AssuranceError> {
    let schema: Value = serde_json::from_str(ASSURANCE_V1_SCHEMA).map_err(|error| {
        AssuranceError::SchemaViolation {
            reason: format!("published schema is invalid JSON: {error}"),
        }
    })?;
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .map_err(|error| AssuranceError::SchemaViolation {
            reason: format!("published schema does not compile: {error}"),
        })?;
    if let Err(mut errors) = compiled.validate(value) {
        if let Some(error) = errors.next() {
            return Err(AssuranceError::SchemaViolation {
                reason: error.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_accepted_premises(
    export: &AssuranceExport,
    accepted: &AcceptedAssurancePremises,
) -> Result<(), AssuranceError> {
    validate_premise_uniqueness(&export.modules)?;
    validate_premise_uniqueness(&accepted.modules)?;
    for module in &export.modules {
        let Some(expected) = accepted
            .modules
            .iter()
            .find(|item| item.name == module.name)
        else {
            return Err(AssuranceError::UnacceptedModule {
                module: module.name.clone(),
            });
        };
        if expected.version != module.version {
            return Err(AssuranceError::UnacceptedModuleVersion {
                module: module.name.clone(),
                version: module.version.clone(),
            });
        }
        let accepted_schemas: BTreeSet<_> = expected.schemas.iter().collect();
        for schema in &module.schemas {
            if !accepted_schemas.contains(schema) {
                return Err(AssuranceError::UnacceptedSchemaDigest {
                    module: module.name.clone(),
                    archetype: schema.archetype.clone(),
                    digest: schema.schema_digest.clone(),
                });
            }
        }
    }
    Ok(())
}

fn validate_premise_uniqueness(modules: &[AssuranceModulePremise]) -> Result<(), AssuranceError> {
    let mut module_names = BTreeSet::new();
    for module in modules {
        if !module_names.insert(&module.name) {
            return Err(AssuranceError::DuplicateModulePremise {
                module: module.name.clone(),
            });
        }
        let mut archetypes = BTreeSet::new();
        for schema in &module.schemas {
            if !archetypes.insert(&schema.archetype) {
                return Err(AssuranceError::DuplicateSchemaPremise {
                    module: module.name.clone(),
                    archetype: schema.archetype.clone(),
                });
            }
        }
    }
    Ok(())
}

fn digest_json(value: &Value) -> Result<String, AssuranceError> {
    let bytes = serde_json::to_vec(value).map_err(|error| AssuranceError::Serialization {
        reason: error.to_string(),
    })?;
    Ok(digest_bytes(&bytes))
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn relative_path(root: &Path, path: &Path) -> Result<String, AssuranceError> {
    let relative = if path.is_absolute() {
        path.strip_prefix(root)
            .map_err(|_| AssuranceError::PathOutsideRoot {
                path: path.to_path_buf(),
                root: root.to_path_buf(),
            })?
    } else {
        path
    };
    path_components(relative).ok_or_else(|| AssuranceError::PathOutsideRoot {
        path: path.to_path_buf(),
        root: root.to_path_buf(),
    })
}

fn source_relative_path(path: &str) -> Result<String, AssuranceError> {
    path_components(Path::new(path)).ok_or_else(|| AssuranceError::PathOutsideRoot {
        path: PathBuf::from(path),
        root: PathBuf::from("."),
    })
}

fn path_components(path: &Path) -> Option<String> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => components.push(value.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    (!components.is_empty()).then(|| components.join("/"))
}

fn first_line_containing(source: &str, needle: &str) -> Option<usize> {
    source
        .lines()
        .position(|line| line.contains(needle))
        .map(|line| line + 1)
}
