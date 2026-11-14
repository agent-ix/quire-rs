//! The one additive `semantic` record (FR-072): FR-070/FR-071 outcomes with
//! per-kind availability and `lossy`, ordered diagnostics, and the published
//! `semantic-v1` shape.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::extract::dsl::{EdgeTarget, ExtractionDsl};
use crate::extract::locator::{Locator, LocatorPrimitive};

use super::clauses::{
    extract_clauses, extract_operations, ClauseRef, OperationDecl, OperationExpression,
    OperationSource, SourceLocus,
};
use super::context::SemanticContext;
use super::decl::FieldDecl;
use super::model::{
    extract_model, failed, model_availability, ModelDeclarations, ModelRefs, ModelSource,
};
use super::properties::{extract_fields, FieldsForm};
use super::record::{copy_model_keys, extract_record, RecordSource};
use super::relations::{extract_relations, ExtractedRelation, RelationDecl, RelationSource};
use super::systems::{AllocationRecord, ConnectionRecord, PartRecord, PortRecord};
use super::{AvailabilityState, KindAvailability, SemanticDiagnostic};

pub const SEMANTIC_FORMAT_VERSION: u64 = 1;
pub const SEMANTIC_V1_SCHEMA: &str = include_str!("../../schemas/output/semantic-v1.schema.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Availability {
    pub fields: KindAvailability,
    pub clauses: KindAvailability,
    pub operations: KindAvailability,
    /// FR-075 frontmatter and table features; present when any is declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<KindAvailability>,
    /// FR-076 relationships; present when the module names `relationships`
    /// or the artifact holds a relationship-shaped table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relations: Option<KindAvailability>,
    /// FR-081 record-key sections, per key; present when the module names
    /// `record-sections`.
    #[serde(
        default,
        rename = "recordKeys",
        skip_serializing_if = "Option::is_none"
    )]
    pub record_keys: Option<BTreeMap<String, KindAvailability>>,
}

/// The FR-072 record. Optional keys are skipped when absent so the shape
/// is the published one byte for byte.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticExtraction {
    pub format_version: u64,
    pub contract_version: String,
    pub package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldDecl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_form: Option<FieldsForm>,
    /// The source locus of each `fields` element (table row or fence line), at
    /// its index. Beside `fields` so `FieldDecl` stays the semantic-core shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_sources: Option<Vec<SourceLocus>>,
    /// The source loci of each operation's parameters and `Returns:` line, at
    /// the index of the matching `operations` element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_sources: Option<Vec<OperationSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clauses: Option<Vec<ClauseRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clause_text: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<Vec<OperationDecl>>,
    /// The opaque inline `Pre:`/`Post:` expressions of FR-071, present only
    /// when `operations` is and at least one operation carries one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_expressions: Option<Vec<OperationExpression>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelDeclarations>,
    /// FR-076 `RelationDecl[]`, present exactly when `availability.relations`
    /// is `available`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<RelationDecl>>,
    /// The `name` and `sourceSpan` of each `relations` element, at its index
    /// (the embedded semantic-core carrier, `agent-ix/filament-core-data#155`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation_sources: Option<Vec<RelationSource>>,
    /// FR-076 typed relation keys (`emits`, `persists`, ...): each key the
    /// object type's data schema declares, with its `SemanticId`s in row
    /// order, present when `availability.relations` is `available`.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub typed_relations: BTreeMap<String, Vec<String>>,
    /// FR-081 record keys, present when at least one key is extracted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record: Option<BTreeMap<String, Value>>,
    /// The section span of each `record` key, in document order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_sources: Option<Vec<RecordSource>>,
    pub availability: Availability,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

/// Which sections the module's `body_extraction` marks `required`: the
/// `missing` state applies to those (FR-072 Behavior).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequiredSections {
    pub properties: bool,
    pub invariants: bool,
    pub operations: bool,
}

impl RequiredSections {
    /// Scan a typed extraction DSL for required locators under the three
    /// headings: every primitive of every locator in `match`, `per_match`,
    /// and `emit_edges`, including each member of a fallback chain.
    pub fn from_extraction(dsl: &ExtractionDsl) -> Self {
        let pattern = &dsl.yield_pattern;
        let edge_locators = dsl
            .emit_edges
            .iter()
            .flatten()
            .filter_map(|edge| match &edge.target {
                EdgeTarget::Locator(locator) => Some(locator),
                EdgeTarget::Static(_) => None,
            });
        let primitives = pattern
            .r#match
            .iter()
            .chain(pattern.per_match.iter())
            .flat_map(|map| map.values())
            .chain(edge_locators)
            .flat_map(|locator| match locator {
                Locator::Primitive(p) => std::slice::from_ref(p),
                Locator::Fallback(chain) => chain.as_slice(),
            });
        let mut out = Self::default();
        for primitive in primitives {
            let (heading, required) = match primitive {
                LocatorPrimitive::SectionBody {
                    after_heading,
                    required,
                    ..
                } => (Some(after_heading.as_str()), *required),
                LocatorPrimitive::CodeBlock {
                    under_section,
                    required,
                    ..
                }
                | LocatorPrimitive::TableRow {
                    under_section,
                    required,
                    ..
                }
                | LocatorPrimitive::ListItem {
                    under_section,
                    required,
                    ..
                } => (under_section.as_deref(), *required),
                LocatorPrimitive::FrontmatterField { .. } | LocatorPrimitive::Heading { .. } => {
                    (None, false)
                }
            };
            match (heading, required) {
                (Some("Properties"), true) => out.properties = true,
                (Some("Invariants"), true) => out.invariants = true,
                (Some("Operations"), true) => out.operations = true,
                _ => {}
            }
        }
        out
    }
}

/// Run FR-070, FR-071, FR-075, and FR-076 over one document and assemble
/// the record.
pub fn extract_semantic(
    raw: &str,
    ctx: &SemanticContext,
    required: &RequiredSections,
) -> SemanticExtraction {
    let declared_lossy = ctx.module.compatibility_posture == "declared-lossy";
    let fields = extract_fields(raw, ctx);
    let clauses = extract_clauses(raw, ctx);
    let operations = extract_operations(raw, ctx, clauses.clauses.as_deref().unwrap_or(&[]));
    let clause_ids: Option<Vec<String>> =
        (clauses.availability.state != AvailabilityState::Unavailable).then(|| {
            clauses
                .clauses
                .iter()
                .flatten()
                .map(|c| c.clause_id.clone())
                .collect()
        });
    let operation_names: Option<Vec<String>> =
        (operations.availability.state != AvailabilityState::Unavailable).then(|| {
            operations
                .operations
                .iter()
                .flatten()
                .map(|o| o.name.clone())
                .collect()
        });
    let field_names: Option<Vec<String>> =
        (fields.availability.state != AvailabilityState::Unavailable).then(|| {
            fields
                .fields
                .iter()
                .flatten()
                .map(|f| f.name.clone())
                .collect()
        });
    let model_outcome = extract_model(
        raw,
        ctx,
        ModelRefs {
            clause_ids: clause_ids.as_deref(),
            operation_names: operation_names.as_deref(),
            field_names: field_names.as_deref(),
        },
    );

    let relations = extract_relations(raw, ctx);
    let record = extract_record(raw, ctx);

    let mut diagnostics: Vec<SemanticDiagnostic> = Vec::new();
    diagnostics.extend(fields.diagnostics.iter().cloned());
    diagnostics.extend(clauses.diagnostics.iter().cloned());
    diagnostics.extend(operations.diagnostics.iter().cloned());
    diagnostics.extend(model_outcome.diagnostics.iter().cloned());
    diagnostics.extend(relations.diagnostics);

    let missing = |mut a: KindAvailability, required: bool, what: &str| {
        if required && a.state == AvailabilityState::NotApplicable {
            a = KindAvailability::missing(format!(
                "`## {what}` is required by the module's body_extraction and absent"
            ));
        }
        if declared_lossy {
            a.lossy = true;
        }
        a
    };
    let mut availability = Availability {
        fields: missing(
            fields.availability.clone(),
            required.properties,
            "Properties",
        ),
        clauses: missing(
            clauses.availability.clone(),
            required.invariants,
            "Invariants",
        ),
        operations: missing(
            operations.availability.clone(),
            required.operations,
            "Operations",
        ),
        model: model_availability(&[
            model_outcome.source(),
            ModelSource {
                declared: fields.model_declared,
                failed: failed(&fields.availability),
                lossy: false,
                unresolved: None,
                diagnostics: &fields.diagnostics,
            },
            ModelSource {
                declared: operations.model_declared,
                failed: failed(&operations.availability),
                lossy: false,
                unresolved: None,
                diagnostics: &operations.diagnostics,
            },
        ])
        .map(|mut a| {
            if declared_lossy {
                a.lossy = true;
            }
            a
        }),
        relations: relations.availability.map(|mut a| {
            if declared_lossy {
                a.lossy = true;
            }
            a
        }),
        record_keys: None,
    };

    let relation_record = relations
        .relations
        .map(|rows| ExtractedRelation::split(rows, relations.typed.as_ref()))
        .unwrap_or_default();
    // `model` is carried exactly when `availability.model` is available.
    let model = availability
        .model
        .as_ref()
        .is_some_and(|a| a.state == AvailabilityState::Available)
        .then(|| {
            let mut model = model_outcome.model;
            model.field_features =
                (!fields.field_features.is_empty()).then_some(fields.field_features);
            model.operation_frames = (!operations.frames.is_empty()).then_some(operations.frames);
            model
        });
    let mut record = record;
    if let Some(outcome) = record.as_mut() {
        copy_model_keys(
            raw,
            outcome,
            ctx,
            model.as_ref(),
            availability.model.as_ref(),
        );
    }
    availability.record_keys = record
        .as_ref()
        .map(|r| r.availability.clone())
        .filter(|keys| !keys.is_empty())
        .map(|mut keys| {
            if declared_lossy {
                keys.values_mut().for_each(|a| a.lossy = true);
            }
            keys
        });
    let record = record.unwrap_or_default();
    diagnostics.extend(record.diagnostics.iter().cloned());
    let mut extraction = SemanticExtraction {
        format_version: SEMANTIC_FORMAT_VERSION,
        contract_version: ctx.module.contract_version.clone(),
        package: ctx.module.package.clone(),
        field_sources: fields.fields.is_some().then_some(fields.field_sources),
        operation_sources: operations
            .operations
            .is_some()
            .then_some(operations.sources),
        fields: fields.fields,
        fields_form: fields.form,
        clauses: clauses.clauses,
        clause_text: if clauses.clause_text.is_empty()
            && availability.clauses.state != AvailabilityState::Available
        {
            None
        } else {
            Some(clauses.clause_text)
        },
        operations: operations.operations,
        operation_expressions: (!operations.expressions.is_empty())
            .then_some(operations.expressions),
        model,
        relations: relation_record.relations,
        relation_sources: relation_record.sources,
        typed_relations: relation_record.typed,
        record: (!record.record.is_empty()).then_some(record.record),
        record_sources: (!record.sources.is_empty()).then_some(record.sources),
        availability,
        diagnostics,
    };
    compose_frontmatter_record(raw, ctx, &mut extraction);
    extraction.diagnostics.sort_by(|a, b| {
        (a.line.unwrap_or(0), a.column.unwrap_or(0), a.code.as_str()).cmp(&(
            b.line.unwrap_or(0),
            b.column.unwrap_or(0),
            b.code.as_str(),
        ))
    });
    extraction
}

const RESERVED_FRONTMATTER_KEYS: [&str; 8] = [
    "id",
    "title",
    "type",
    "object",
    "uuid",
    "relationships",
    "description",
    "tags",
];

#[derive(Clone, PartialEq, Eq)]
struct RecordOccurrence {
    key: String,
    kind: &'static str,
    line: usize,
}

fn compose_frontmatter_record(
    raw: &str,
    ctx: &SemanticContext,
    extraction: &mut SemanticExtraction,
) {
    if ctx.record_schema.is_none() {
        return;
    }
    let Some(frontmatter) = crate::parser::frontmatter::extract_frontmatter_ref(raw).frontmatter
    else {
        return;
    };

    let mut occurrences = declaration_occurrences(raw, ctx, extraction);
    let mut projected = BTreeMap::new();
    for (key, value) in frontmatter {
        if RESERVED_FRONTMATTER_KEYS.contains(&key.as_str()) {
            continue;
        }
        let line =
            crate::extract::frontmatter_key_line(raw, std::slice::from_ref(&key)).unwrap_or(1);
        projected.insert(key.clone(), value);
        occurrences.push(RecordOccurrence {
            key,
            kind: "frontmatter",
            line,
        });
    }

    let mut by_key: BTreeMap<String, Vec<RecordOccurrence>> = BTreeMap::new();
    for occurrence in occurrences {
        let group = by_key.entry(occurrence.key.clone()).or_default();
        if !group.contains(&occurrence) {
            group.push(occurrence);
        }
    }

    let mut duplicate_keys = BTreeSet::new();
    for (key, group) in &mut by_key {
        group.sort_by_key(|source| source.line);
        let Some(first) = group.first() else {
            continue;
        };
        for later in group.iter().skip(1) {
            duplicate_keys.insert(key.clone());
            extraction.diagnostics.push(
                SemanticDiagnostic::new(
                    "semantic.duplicate-record-key",
                    super::SemanticSeverity::Error,
                    later.line,
                    format!(
                        "record key `{key}` is provided by {} at line {} and later by {} at line {}",
                        first.kind, first.line, later.kind, later.line
                    ),
                )
                .with_reason(key),
            );
        }
    }

    for key in &duplicate_keys {
        projected.remove(key);
        if let Some(record) = extraction.record.as_mut() {
            record.remove(key);
        }
        if let Some(sources) = extraction.record_sources.as_mut() {
            sources.retain(|source| source.key != *key);
        }
        if let Some(keys) = extraction.availability.record_keys.as_mut() {
            keys.insert(
                key.clone(),
                KindAvailability::unavailable("duplicate-record-key"),
            );
        }
    }
    if !projected.is_empty() {
        let record = extraction.record.get_or_insert_with(BTreeMap::new);
        record.extend(projected);
    }
}

fn declaration_occurrences(
    raw: &str,
    ctx: &SemanticContext,
    extraction: &SemanticExtraction,
) -> Vec<RecordOccurrence> {
    let mut out = Vec::new();
    let line_for = |sources: &[SourceLocus], heading: &str| {
        sources
            .first()
            .map_or_else(|| section_line(raw, heading), |source| source.start_line)
    };
    if extraction.fields.is_some() {
        out.push(RecordOccurrence {
            key: "fields".to_string(),
            kind: "semantic declaration",
            line: line_for(
                extraction.field_sources.as_deref().unwrap_or_default(),
                "Properties",
            ),
        });
    }
    if extraction.clauses.is_some() {
        let sources: Vec<SourceLocus> = extraction
            .clauses
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter_map(|clause| clause.source_span.clone())
            .collect();
        out.push(RecordOccurrence {
            key: "clauses".to_string(),
            kind: "semantic declaration",
            line: line_for(&sources, "Invariants"),
        });
    }
    if extraction.operations.is_some() {
        let sources: Vec<SourceLocus> = extraction
            .operation_sources
            .as_deref()
            .unwrap_or_default()
            .iter()
            .flat_map(|source| source.params.iter().chain(source.returns.iter()))
            .cloned()
            .collect();
        out.push(RecordOccurrence {
            key: "operations".to_string(),
            kind: "semantic declaration",
            line: line_for(&sources, "Operations"),
        });
    }
    if extraction.relations.is_some() {
        let sources: Vec<SourceLocus> = extraction
            .relation_sources
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|source| source.source_span.clone())
            .collect();
        out.push(RecordOccurrence {
            key: "relations".to_string(),
            kind: "semantic declaration",
            line: line_for(&sources, "Relationships"),
        });
    }
    if let Some(order) = extraction
        .model
        .as_ref()
        .and_then(|model| model.feature_order.as_deref())
    {
        out.push(RecordOccurrence {
            key: "featureOrder".to_string(),
            kind: "semantic declaration",
            line: order.first().map_or_else(
                || section_line(raw, "Features"),
                |entry| entry.source_span.start_line,
            ),
        });
    }
    for key in extraction.typed_relations.keys() {
        let line = extraction
            .relation_sources
            .as_deref()
            .unwrap_or_default()
            .iter()
            .find(|source| source.verb.as_deref() == Some(key.as_str()))
            .map_or_else(
                || section_line(raw, "Relationships"),
                |source| source.source_span.start_line,
            );
        out.push(RecordOccurrence {
            key: key.clone(),
            kind: "semantic declaration",
            line,
        });
    }
    if let Some(model) = &extraction.model {
        if let Some(declaration) = &model.part {
            add_system_occurrences(&mut out, &declaration.record, &declaration.source_span);
        }
        if let Some(declaration) = &model.port {
            add_system_occurrences(&mut out, &declaration.record, &declaration.source_span);
        }
        if let Some(declaration) = &model.connection {
            add_system_occurrences(&mut out, &declaration.record, &declaration.source_span);
        }
        if let Some(declaration) = &model.allocation {
            add_system_occurrences(&mut out, &declaration.record, &declaration.source_span);
        }
    }
    if let Some(record) = &extraction.record {
        for key in record.keys() {
            let Some(source) = extraction
                .record_sources
                .as_deref()
                .unwrap_or_default()
                .iter()
                .find(|source| source.key == *key)
            else {
                continue;
            };
            let kind = if ctx.declared_tables.section_of(key).is_some() {
                "semantic declaration"
            } else {
                "FR-081 section"
            };
            out.push(RecordOccurrence {
                key: key.clone(),
                kind,
                line: source.source_span.start_line,
            });
        }
    }
    out
}

fn add_system_occurrences<T: Serialize>(
    out: &mut Vec<RecordOccurrence>,
    record: &T,
    source: &SourceLocus,
) {
    if let Ok(Value::Object(values)) = serde_json::to_value(record) {
        out.extend(values.into_iter().map(|(key, _)| RecordOccurrence {
            key,
            kind: "semantic declaration",
            line: source.start_line,
        }));
    }
}

fn section_line(raw: &str, heading: &str) -> usize {
    let lines = super::scan::lines(raw);
    super::scan::level2_sections(&lines, heading)
        .first()
        .map_or(1, |(line, _)| *line)
}

/// The value a data schema validates: the declaration arrays and the one
/// systems-model record's keys, flat.
#[derive(Debug, Clone, Serialize)]
pub struct DeclarationRecord<'a> {
    /// FR-070 `fields`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<&'a [FieldDecl]>,
    /// FR-071 `clauses`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clauses: Option<&'a [ClauseRef]>,
    /// FR-076 `relations`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<&'a [RelationDecl]>,
    /// FR-071 `operations`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<&'a [OperationDecl]>,
    /// FR-075 `featureOrder`: the `Features` table's names in row order.
    #[serde(rename = "featureOrder", skip_serializing_if = "Option::is_none")]
    pub feature_order: Option<Vec<&'a str>>,
    /// The systems-model record; FR-075 admits at most one per artifact.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub systems: Option<SystemsRecord<'a>>,
    /// FR-076 typed relation keys, each a `SemanticId[]`.
    #[serde(flatten)]
    pub typed_relations: &'a BTreeMap<String, Vec<String>>,
    /// FR-081 record keys, each the value its section produced.
    #[serde(flatten)]
    pub record: Option<&'a BTreeMap<String, Value>>,
}

/// One systems-model record, serialized as its own keys.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(untagged)]
pub enum SystemsRecord<'a> {
    /// `model.part`.
    Part(&'a PartRecord),
    /// `model.port`.
    Port(&'a PortRecord),
    /// `model.connection`.
    Connection(&'a ConnectionRecord),
    /// `model.allocation`.
    Allocation(&'a AllocationRecord),
}

/// Why the declaration record could not be built.
#[derive(Debug, thiserror::Error)]
pub enum DeclarationError {
    /// FR-075 extracts at most one systems-model table per artifact.
    #[error("the declaration carries more than one systems-model record")]
    MultipleSystemsRecords,
    /// The typed record did not serialize to JSON.
    #[error("the record does not serialize: {0}")]
    Serialize(#[from] serde_json::Error),
}

impl SemanticExtraction {
    /// The typed declaration record (what `Entity.json`, `Part.json`,
    /// `Interface.json`, and their kin describe).
    ///
    /// # Errors
    ///
    /// More than one systems-model record: FR-075 extracts at most one
    /// systems-model table per artifact.
    pub fn declaration(&self) -> Result<DeclarationRecord<'_>, DeclarationError> {
        let systems = self.model.as_ref().map(|model| {
            let records = [
                model.part.as_ref().map(|d| SystemsRecord::Part(&d.record)),
                model.port.as_ref().map(|d| SystemsRecord::Port(&d.record)),
                model
                    .connection
                    .as_ref()
                    .map(|d| SystemsRecord::Connection(&d.record)),
                model
                    .allocation
                    .as_ref()
                    .map(|d| SystemsRecord::Allocation(&d.record)),
            ];
            let mut present = records.into_iter().flatten();
            let first = present.next();
            (first, present.next().is_some())
        });
        let (systems, extra_systems) = systems.unwrap_or((None, false));
        if extra_systems {
            return Err(DeclarationError::MultipleSystemsRecords);
        }
        Ok(DeclarationRecord {
            fields: self.fields.as_deref(),
            clauses: self.clauses.as_deref(),
            relations: self.relations.as_deref(),
            operations: self.operations.as_deref(),
            feature_order: self
                .model
                .as_ref()
                .and_then(|m| m.feature_order.as_ref())
                .map(|order| order.iter().map(|e| e.name.as_str()).collect()),
            systems,
            typed_relations: &self.typed_relations,
            record: self.record.as_ref(),
        })
    }

    /// The declaration record as the JSON value a data schema validates.
    ///
    /// # Errors
    ///
    /// As [`Self::declaration`], or the record does not serialize.
    pub fn declaration_record(&self) -> Result<Value, DeclarationError> {
        self.declaration_record_omitting(&self.duplicate_record_keys())
    }

    /// The declaration record with keys omitted after duplicate source errors.
    pub(crate) fn declaration_record_omitting(
        &self,
        omitted: &BTreeSet<String>,
    ) -> Result<Value, DeclarationError> {
        let declaration = self.declaration()?;
        let mut value = serde_json::to_value(&declaration)?;
        if let Some(object) = value.as_object_mut() {
            for key in omitted {
                object.remove(key);
            }
        }
        #[cfg(debug_assertions)]
        {
            let mut expected_keys = BTreeSet::new();
            for (key, present) in [
                ("fields", declaration.fields.is_some()),
                ("clauses", declaration.clauses.is_some()),
                ("relations", declaration.relations.is_some()),
                ("operations", declaration.operations.is_some()),
                ("featureOrder", declaration.feature_order.is_some()),
            ] {
                if present {
                    expected_keys.insert(key.to_string());
                }
            }
            if let Some(Ok(Value::Object(keys))) = declaration.systems.map(serde_json::to_value) {
                expected_keys.extend(keys.into_iter().map(|(key, _)| key));
            }
            expected_keys.extend(declaration.typed_relations.keys().cloned());
            if let Some(record) = declaration.record {
                expected_keys.extend(record.keys().cloned());
            }
            for key in omitted {
                expected_keys.remove(key);
            }
            debug_assert_eq!(
                value.as_object().map_or(0, serde_json::Map::len),
                expected_keys.len(),
                "declaration record serialization does not preserve its distinct keys"
            );
        }
        Ok(value)
    }

    /// Keys whose multiple authored sources make their composed value
    /// ambiguous. The diagnostic reason carries the source key for the
    /// validation layer's filtered declaration record.
    pub(crate) fn duplicate_record_keys(&self) -> BTreeSet<String> {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "semantic.duplicate-record-key")
            .filter_map(|diagnostic| diagnostic.reason.clone())
            .collect()
    }

    /// Whether any declaration kind could not be read (`unavailable`). The
    /// declaration record omits such a kind's key, which is the shape of "nothing
    /// was declared"; a consumer must not validate that omission as a record.
    pub fn any_kind_unavailable(&self) -> bool {
        self.any_kind_unavailable_except(&BTreeSet::new())
    }

    /// Whether any declaration kind other than `omitted_record_keys` could not
    /// be read. Duplicate record keys are deliberately omitted from the
    /// declaration, but do not prevent schema validation of the remaining
    /// unambiguous values.
    pub(crate) fn any_kind_unavailable_except(
        &self,
        omitted_record_keys: &BTreeSet<String>,
    ) -> bool {
        let a = &self.availability;
        [Some(&a.fields), Some(&a.clauses), Some(&a.operations)]
            .into_iter()
            .chain([a.model.as_ref(), a.relations.as_ref()])
            .flatten()
            .any(|k| k.state == AvailabilityState::Unavailable)
            || a.record_keys.as_ref().is_some_and(|keys| {
                keys.iter().any(|(key, availability)| {
                    !omitted_record_keys.contains(key)
                        && availability.state == AvailabilityState::Unavailable
                })
            })
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(SemanticDiagnostic::is_error)
    }
}
