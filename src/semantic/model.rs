//! Model feature extraction (FR-075): generalization and abstract types from
//! frontmatter, and the typed model tables (populations and the object-type
//! sections), each gated by what the module manifest declares.
//!
//! Field features (`Presence`, `Subsets`, `Redefines`) are read by the
//! Properties extractor and operation frames by the Operations extractor;
//! this module owns their declaration shapes, the feature vocabulary, and
//! the one refusal every extractor emits for an undeclared feature.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::clauses::SourceLocus;
use super::context::SemanticContext;
use super::contract::SemanticSeverity;
use super::decl::{is_identifier, Multiplicity, TypeRef};
use super::properties::{map_multiplicity, map_type, strip_ticks, RowInput};
use super::scan::{blocks_in, level2_headings, lines, Block, Table};
use super::{KindAvailability, SemanticDiagnostic};

/// Every model feature FR-075 extracts. The manifest declares a feature
/// either by naming its [`ModelFeature::mapping`] token in
/// `semantic.mappings` or, for a table feature, by a `table_row` locator
/// whose columns equal [`ModelFeature::columns`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFeature {
    Generalization,
    AbstractTypes,
    Presence,
    Subsetting,
    Redefinition,
    OperationContracts,
    EffectFrames,
    Population,
    Values,
    States,
    Transitions,
    Steps,
    Members,
    Vocabulary,
}

impl ModelFeature {
    /// The table features, in the order their tables are matched.
    pub const TABLES: [ModelFeature; 7] = [
        ModelFeature::Population,
        ModelFeature::Values,
        ModelFeature::States,
        ModelFeature::Transitions,
        ModelFeature::Steps,
        ModelFeature::Members,
        ModelFeature::Vocabulary,
    ];

    /// The feature name used as a diagnostic `reason`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Generalization => "generalization",
            Self::AbstractTypes => "abstract-types",
            Self::Presence => "presence",
            Self::Subsetting => "subsetting",
            Self::Redefinition => "redefinition",
            Self::OperationContracts => "operation-contracts",
            Self::EffectFrames => "effect-frames",
            Self::Population => "population",
            Self::Values => "values",
            Self::States => "states",
            Self::Transitions => "transitions",
            Self::Steps => "steps",
            Self::Members => "members",
            Self::Vocabulary => "vocabulary",
        }
    }

    /// The `semantic.mappings` token that declares a non-table feature.
    pub fn mapping(self) -> Option<&'static str> {
        match self {
            Self::Generalization
            | Self::AbstractTypes
            | Self::Presence
            | Self::Subsetting
            | Self::Redefinition
            | Self::OperationContracts
            | Self::EffectFrames => Some(self.name()),
            Self::Population
            | Self::Values
            | Self::States
            | Self::Transitions
            | Self::Steps
            | Self::Members
            | Self::Vocabulary => None,
        }
    }

    /// The exact header of a table feature.
    pub fn columns(self) -> Option<&'static [&'static str]> {
        match self {
            Self::Population => Some(&["Type", "Extent"]),
            Self::Values => Some(&["Value", "Description"]),
            Self::States => Some(&["State", "Description"]),
            Self::Transitions => Some(&["From", "To", "Trigger", "Guard", "Emits"]),
            Self::Steps => Some(&["Step", "Kind", "Consumes", "Emits", "Description"]),
            Self::Members => Some(&["Member", "Multiplicity"]),
            Self::Vocabulary => Some(&["Term", "Description"]),
            Self::Generalization
            | Self::AbstractTypes
            | Self::Presence
            | Self::Subsetting
            | Self::Redefinition
            | Self::OperationContracts
            | Self::EffectFrames => None,
        }
    }

    /// Is this non-table feature declared by the context's module?
    pub fn declared_by_mappings(self, ctx: &SemanticContext) -> bool {
        self.mapping()
            .is_some_and(|token| ctx.module.mappings.iter().any(|m| m == token))
    }
}

/// One `table_row` locator of the object type's `body_extraction`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeclaredTable {
    pub section: String,
    pub columns: Vec<String>,
}

/// Every `table_row` locator with `under_section` and `assert.columns` the
/// object type declares (FR-075 Inputs).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeclaredTables(pub Vec<DeclaredTable>);

impl DeclaredTables {
    /// Read the locators from an extraction DSL as JSON.
    pub fn from_dsl(dsl: &Value) -> Self {
        fn walk(v: &Value, out: &mut Vec<DeclaredTable>) {
            match v {
                Value::Object(map) => {
                    let is_table = map.get("from").and_then(Value::as_str) == Some("table_row");
                    let section = map.get("under_section").and_then(Value::as_str);
                    let columns = map
                        .get("assert")
                        .and_then(|a| a.get("columns"))
                        .and_then(Value::as_array);
                    if let (true, Some(section), Some(columns)) = (is_table, section, columns) {
                        out.push(DeclaredTable {
                            section: section.to_string(),
                            columns: columns
                                .iter()
                                .filter_map(Value::as_str)
                                .map(str::to_string)
                                .collect(),
                        });
                    }
                    map.values().for_each(|v| walk(v, out));
                }
                Value::Array(items) => items.iter().for_each(|v| walk(v, out)),
                _ => {}
            }
        }
        let mut out = Vec::new();
        walk(dsl, &mut out);
        Self(out)
    }

    /// Does the object type declare `columns` under `section`?
    pub fn declares(&self, section: &str, columns: &[&str]) -> bool {
        self.0.iter().any(|t| {
            t.section == section
                && t.columns
                    .iter()
                    .map(String::as_str)
                    .eq(columns.iter().copied())
        })
    }
}

/// `semantic.feature-not-extractable` (FR-075 Behavior): the artifact
/// declares `feature` in `section` at `line`, and its manifest does not.
pub fn not_extractable(
    ctx: &SemanticContext,
    feature: ModelFeature,
    section: &str,
    line: usize,
) -> SemanticDiagnostic {
    SemanticDiagnostic::new(
        "semantic.feature-not-extractable",
        SemanticSeverity::Error,
        line,
        format!(
            "artifact {} declares {} in {section} at line {line}; the module {} manifest does not declare it",
            ctx.path,
            feature.name(),
            ctx.module.package
        ),
    )
    .with_reason(feature.name())
}

fn err(code: &str, line: usize, message: impl Into<String>) -> SemanticDiagnostic {
    SemanticDiagnostic::new(code, SemanticSeverity::Error, line, message)
}

/// `startLine`..`endLine` of one declaring line (FR-075 Outputs).
pub fn line_span(ctx: &SemanticContext, lines: &[&str], line: usize) -> SourceLocus {
    let (identity, _) = ctx.resolved_source_identity();
    let len = line
        .checked_sub(1)
        .and_then(|i| lines.get(i))
        .map(|l| l.trim_end_matches('\r').len())
        .unwrap_or(0);
    SourceLocus {
        source_identity: identity,
        path: ctx.path.clone(),
        start_line: line,
        start_column: 1,
        end_line: Some(line),
        end_column: Some(len + 1),
    }
}

/// Authored presence of a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Presence {
    Required,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupertypeDecl {
    pub target: String,
    pub source_span: SourceLocus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractDecl {
    pub value: bool,
    pub source_span: SourceLocus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldFeatureDecl {
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Presence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsets: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redefines: Option<String>,
    pub source_span: SourceLocus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationFrameDecl {
    pub operation: String,
    pub requires: Vec<String>,
    pub ensures: Vec<String>,
    pub modifies: Vec<String>,
    pub creates: Vec<String>,
    pub deletes: Vec<String>,
    pub source_span: SourceLocus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulationMemberDecl {
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    pub extent: Multiplicity,
    pub source_span: SourceLocus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopulationDecl {
    pub members: Vec<PopulationMemberDecl>,
}

/// A `Value` or `State` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumValueDecl {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    pub source_span: SourceLocus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionDecl {
    pub from: String,
    pub to: String,
    pub trigger: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emits: Option<String>,
    pub source_span: SourceLocus,
}

/// The closed set of process step kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepKind {
    Command,
    Event,
    Decision,
    Compensation,
    Wait,
}

impl std::str::FromStr for StepKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "command" => Ok(Self::Command),
            "event" => Ok(Self::Event),
            "decision" => Ok(Self::Decision),
            "compensation" => Ok(Self::Compensation),
            "wait" => Ok(Self::Wait),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepDecl {
    pub name: String,
    pub kind: StepKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emits: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    pub source_span: SourceLocus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberDecl {
    pub target: String,
    pub multiplicity: Multiplicity,
    pub source_span: SourceLocus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TermDecl {
    pub term: String,
    pub doc: String,
    pub source_span: SourceLocus,
}

/// The FR-075 `model` record: every member is absent unless declared.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDeclarations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supertypes: Option<Vec<SupertypeDecl>>,
    #[serde(rename = "abstract", skip_serializing_if = "Option::is_none")]
    pub abstract_type: Option<AbstractDecl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_features: Option<Vec<FieldFeatureDecl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_frames: Option<Vec<OperationFrameDecl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population: Option<PopulationDecl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<EnumValueDecl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub states: Option<Vec<EnumValueDecl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transitions: Option<Vec<TransitionDecl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<StepDecl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<MemberDecl>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary: Option<Vec<TermDecl>>,
}

impl ModelDeclarations {
    /// True when no member is declared.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// What the frontmatter and table extraction of one artifact produced.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModelOutcome {
    pub model: ModelDeclarations,
    /// Whether the artifact declared any frontmatter or table feature.
    pub declared: bool,
    pub lossy: bool,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

/// Names an operation or state reference is checked against; `None` skips
/// the check because the referenced kind is already `unavailable`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModelRefs<'a> {
    pub clause_ids: Option<&'a [String]>,
    pub operation_names: Option<&'a [String]>,
}

/// FR-075 frontmatter and table features of one document.
pub fn extract_model(raw: &str, ctx: &SemanticContext, refs: ModelRefs<'_>) -> ModelOutcome {
    let lines = lines(raw);
    let mut out = ModelOutcome::default();
    frontmatter_features(raw, &lines, ctx, &mut out);
    table_features(&lines, ctx, refs, &mut out);
    out
}

/// 1-based line of each top-level frontmatter key and of each item under
/// `relationships:`.
struct FrontmatterLines {
    abstract_line: Option<usize>,
    relationships_line: Option<usize>,
    items: Vec<usize>,
}

fn frontmatter_lines(lines: &[&str], header_lines: usize) -> FrontmatterLines {
    let mut out = FrontmatterLines {
        abstract_line: None,
        relationships_line: None,
        items: Vec::new(),
    };
    let mut in_relationships = false;
    for (i, raw) in lines.iter().enumerate().take(header_lines) {
        let line = i + 1;
        let text = raw.trim_end_matches('\r');
        let top_level = !text.starts_with(' ') && !text.starts_with('-') && !text.is_empty();
        if top_level {
            in_relationships = text.starts_with("relationships:");
            if in_relationships {
                out.relationships_line = Some(line);
            }
            if text.starts_with("abstract:") {
                out.abstract_line = Some(line);
            }
            continue;
        }
        if in_relationships && text.trim_start().starts_with("- ") {
            out.items.push(line);
        }
    }
    out
}

fn frontmatter_features(raw: &str, lines: &[&str], ctx: &SemanticContext, out: &mut ModelOutcome) {
    let fm = crate::parser::frontmatter::extract_frontmatter_ref(raw);
    let Some(map) = fm.frontmatter else {
        return;
    };
    let header = &raw[..raw.len().saturating_sub(fm.body.len())];
    let header_lines = header.split('\n').count();
    let at = frontmatter_lines(lines, header_lines);

    if let Some(value) = map.get("abstract") {
        out.declared = true;
        let line = at.abstract_line.unwrap_or(1);
        if !ModelFeature::AbstractTypes.declared_by_mappings(ctx) {
            out.diagnostics.push(not_extractable(
                ctx,
                ModelFeature::AbstractTypes,
                "frontmatter",
                line,
            ));
        } else if let Some(flag) = value.as_bool() {
            out.model.abstract_type = Some(AbstractDecl {
                value: flag,
                source_span: line_span(ctx, lines, line),
            });
        } else {
            out.diagnostics.push(err(
                "semantic.invalid-model-cell",
                line,
                format!("abstract {value} is not a boolean"),
            ));
        }
    }

    let Some(relationships) = map.get("relationships").and_then(Value::as_array) else {
        return;
    };
    let mut supertypes: Vec<SupertypeDecl> = Vec::new();
    for (i, rel) in relationships.iter().enumerate() {
        if rel.get("type").and_then(Value::as_str) != Some("specializes") {
            continue;
        }
        out.declared = true;
        let line = if at.items.len() == relationships.len() {
            at.items[i]
        } else {
            at.relationships_line.unwrap_or(1)
        };
        if !ModelFeature::Generalization.declared_by_mappings(ctx) {
            out.diagnostics.push(not_extractable(
                ctx,
                ModelFeature::Generalization,
                "frontmatter",
                line,
            ));
            continue;
        }
        let Some(target) = rel.get("target").and_then(Value::as_str) else {
            out.diagnostics.push(err(
                "semantic.invalid-model-cell",
                line,
                "a specializes relationship carries no string target",
            ));
            continue;
        };
        if supertypes.iter().any(|s| s.target == target) {
            out.diagnostics.push(err(
                "semantic.duplicate-model-entry",
                line,
                format!("specializes {target} is declared twice"),
            ));
            continue;
        }
        supertypes.push(SupertypeDecl {
            target: target.to_string(),
            source_span: line_span(ctx, lines, line),
        });
    }
    if !supertypes.is_empty() {
        out.model.supertypes = Some(supertypes);
    }
}

fn table_feature(table: &Table) -> Option<ModelFeature> {
    ModelFeature::TABLES.into_iter().find(|f| {
        f.columns().is_some_and(|cols| {
            table
                .headers
                .iter()
                .map(|h| strip_ticks(h))
                .eq(cols.iter().copied())
        })
    })
}

fn cell(cells: &[String], i: usize) -> &str {
    strip_ticks(cells.get(i).map(String::as_str).unwrap_or(""))
}

fn optional(text: &str) -> Option<String> {
    (!text.is_empty()).then(|| text.to_string())
}

fn name_list(text: &str) -> Option<Vec<String>> {
    let items: Vec<String> = text
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    (!items.is_empty()).then_some(items)
}

fn table_features(
    lines: &[&str],
    ctx: &SemanticContext,
    refs: ModelRefs<'_>,
    out: &mut ModelOutcome,
) {
    let mut seen: Vec<ModelFeature> = Vec::new();
    for (heading, start, end) in level2_headings(lines) {
        for block in blocks_in(lines, start + 1, end) {
            let Block::Table(table) = block else {
                continue;
            };
            let Some(feature) = table_feature(&table) else {
                continue;
            };
            out.declared = true;
            let columns = feature.columns().unwrap_or(&[]);
            if !ctx.declared_tables.declares(&heading, columns) {
                out.diagnostics.push(not_extractable(
                    ctx,
                    feature,
                    &format!("`## {heading}`"),
                    table.line,
                ));
                continue;
            }
            if seen.contains(&feature) {
                out.diagnostics.push(err(
                    "semantic.duplicate-section",
                    table.line,
                    format!("a second {} table", feature.name()),
                ));
                continue;
            }
            seen.push(feature);
            read_table(feature, &table, lines, ctx, out);
        }
    }
    check_transitions(refs, out);
}

fn read_table(
    feature: ModelFeature,
    table: &Table,
    lines: &[&str],
    ctx: &SemanticContext,
    out: &mut ModelOutcome,
) {
    let diagnostics = &mut out.diagnostics;
    let mut keys: Vec<String> = Vec::new();
    let mut duplicate = |key: &str, line: usize, diagnostics: &mut Vec<SemanticDiagnostic>| {
        if keys.iter().any(|k| k == key) {
            diagnostics.push(err(
                "semantic.duplicate-model-entry",
                line,
                format!("{} {key} is declared twice", feature.name()),
            ));
            return true;
        }
        keys.push(key.to_string());
        false
    };
    let identifier =
        |text: &str, what: &str, line: usize, diagnostics: &mut Vec<SemanticDiagnostic>| {
            if is_identifier(text) {
                true
            } else {
                diagnostics.push(err(
                    "semantic.invalid-model-cell",
                    line,
                    format!("{what} {text:?} is not an Identifier"),
                ));
                false
            }
        };
    match feature {
        ModelFeature::Values | ModelFeature::States => {
            let what = if feature == ModelFeature::Values {
                "value"
            } else {
                "state"
            };
            let mut entries = Vec::new();
            for (line, cells) in &table.rows {
                let value = cell(cells, 0);
                if !identifier(value, what, *line, diagnostics)
                    || duplicate(value, *line, diagnostics)
                {
                    continue;
                }
                entries.push(EnumValueDecl {
                    value: value.to_string(),
                    doc: optional(cell(cells, 1)),
                    source_span: line_span(ctx, lines, *line),
                });
            }
            if feature == ModelFeature::Values {
                out.model.values = Some(entries);
            } else {
                out.model.states = Some(entries);
            }
        }
        ModelFeature::Transitions => {
            let mut entries = Vec::new();
            for (line, cells) in &table.rows {
                let (from, to, trigger) = (cell(cells, 0), cell(cells, 1), cell(cells, 2));
                let ok = identifier(from, "from state", *line, diagnostics)
                    & identifier(to, "to state", *line, diagnostics)
                    & identifier(trigger, "trigger", *line, diagnostics);
                if !ok {
                    continue;
                }
                entries.push(TransitionDecl {
                    from: from.to_string(),
                    to: to.to_string(),
                    trigger: trigger.to_string(),
                    guard: optional(cell(cells, 3)),
                    emits: optional(cell(cells, 4)),
                    source_span: line_span(ctx, lines, *line),
                });
            }
            out.model.transitions = Some(entries);
        }
        ModelFeature::Steps => {
            let mut entries = Vec::new();
            for (line, cells) in &table.rows {
                let name = cell(cells, 0);
                let kind_text = cell(cells, 1);
                let kind = kind_text.parse::<StepKind>().ok();
                if kind.is_none() {
                    diagnostics.push(err(
                        "semantic.invalid-model-cell",
                        *line,
                        format!("step kind {kind_text:?} is not command, event, decision, compensation, or wait"),
                    ));
                }
                let named = identifier(name, "step", *line, diagnostics);
                let (Some(kind), true) = (kind, named) else {
                    continue;
                };
                if duplicate(name, *line, diagnostics) {
                    continue;
                }
                entries.push(StepDecl {
                    name: name.to_string(),
                    kind,
                    consumes: name_list(cell(cells, 2)),
                    emits: name_list(cell(cells, 3)),
                    doc: optional(cell(cells, 4)),
                    source_span: line_span(ctx, lines, *line),
                });
            }
            out.model.steps = Some(entries);
        }
        ModelFeature::Members => {
            let mut entries = Vec::new();
            for (line, cells) in &table.rows {
                let target = cell(cells, 0);
                if target.is_empty() {
                    diagnostics.push(err(
                        "semantic.invalid-model-cell",
                        *line,
                        "a member row names no member",
                    ));
                    continue;
                }
                let Some(multiplicity) = map_multiplicity(cell(cells, 1), *line, diagnostics)
                else {
                    continue;
                };
                if duplicate(target, *line, diagnostics) {
                    continue;
                }
                entries.push(MemberDecl {
                    target: target.to_string(),
                    multiplicity,
                    source_span: line_span(ctx, lines, *line),
                });
            }
            out.model.members = Some(entries);
        }
        ModelFeature::Vocabulary => {
            let mut entries = Vec::new();
            for (line, cells) in &table.rows {
                let term = cell(cells, 0);
                if term.is_empty() {
                    diagnostics.push(err(
                        "semantic.invalid-model-cell",
                        *line,
                        "a vocabulary row names no term",
                    ));
                    continue;
                }
                if duplicate(term, *line, diagnostics) {
                    continue;
                }
                entries.push(TermDecl {
                    term: term.to_string(),
                    doc: cell(cells, 1).to_string(),
                    source_span: line_span(ctx, lines, *line),
                });
            }
            out.model.vocabulary = Some(entries);
        }
        ModelFeature::Population => {
            let mut members = Vec::new();
            for (line, cells) in &table.rows {
                let type_cell = cell(cells, 0);
                let row = RowInput {
                    line: *line,
                    name: type_cell.to_string(),
                    type_cell: type_cell.to_string(),
                    mult_cell: cell(cells, 1).to_string(),
                    constraints_cell: String::new(),
                    reference_only: false,
                };
                let (type_ref, lossy) = map_type(type_cell, &row, ctx, diagnostics);
                let extent = map_multiplicity(&row.mult_cell, *line, diagnostics);
                let (Some(type_ref), Some(extent)) = (type_ref, extent) else {
                    continue;
                };
                if duplicate(type_cell, *line, diagnostics) {
                    continue;
                }
                out.lossy |= lossy;
                members.push(PopulationMemberDecl {
                    type_ref,
                    extent,
                    source_span: line_span(ctx, lines, *line),
                });
            }
            out.model.population = Some(PopulationDecl { members });
        }
        ModelFeature::Generalization
        | ModelFeature::AbstractTypes
        | ModelFeature::Presence
        | ModelFeature::Subsetting
        | ModelFeature::Redefinition
        | ModelFeature::OperationContracts
        | ModelFeature::EffectFrames => {}
    }
}

/// Transition reader rules: `from`/`to` name a state, `trigger` an
/// operation, `guard` an invariant clause of the same artifact.
fn check_transitions(refs: ModelRefs<'_>, out: &mut ModelOutcome) {
    let Some(transitions) = &out.model.transitions else {
        return;
    };
    let states: Vec<&str> = out
        .model
        .states
        .iter()
        .flatten()
        .map(|s| s.value.as_str())
        .collect();
    for t in transitions {
        let line = t.source_span.start_line;
        for state in [&t.from, &t.to] {
            if !states.contains(&state.as_str()) {
                out.diagnostics.push(err(
                    "semantic.unknown-state",
                    line,
                    format!("transition state {state} names no declared state"),
                ));
            }
        }
        if let Some(ops) = refs.operation_names {
            if !ops.contains(&t.trigger) {
                out.diagnostics.push(err(
                    "semantic.unknown-trigger",
                    line,
                    format!("transition trigger {} names no operation", t.trigger),
                ));
            }
        }
        if let (Some(guard), Some(clauses)) = (&t.guard, refs.clause_ids) {
            if !clauses.contains(guard) {
                out.diagnostics.push(err(
                    "semantic.dangling-clause-ref",
                    line,
                    format!("guard {guard} is declared by no invariant of this artifact"),
                ));
            }
        }
    }
}

/// The availability of the frontmatter and table features.
pub fn model_availability(outcome: &ModelOutcome) -> Option<KindAvailability> {
    if !outcome.declared {
        return None;
    }
    let loci: Vec<String> = outcome
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .filter_map(|d| d.line.map(|l| l.to_string()))
        .collect();
    if loci.is_empty() {
        Some(KindAvailability::available(outcome.lossy))
    } else {
        Some(KindAvailability::unavailable(format!(
            "entry-errors: lines {}",
            loci.join(", ")
        )))
    }
}
