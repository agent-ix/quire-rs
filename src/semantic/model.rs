//! Model feature extraction (FR-075): identity, generalization, and abstract
//! types from frontmatter, and the model tables (populations and the
//! object-type sections), each gated by what the module manifest declares.
//!
//! Field features (`Presence`, `Subsets`, `Redefines`) are read by the
//! Properties extractor and operation frames by the Operations extractor;
//! this module owns their declaration shapes, the feature vocabulary, the
//! one refusal every extractor emits for an undeclared feature, and the
//! availability of the feature set as a whole.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::clauses::SourceLocus;
use super::context::SemanticContext;
use super::contract::SemanticSeverity;
use super::decl::{is_identifier, Multiplicity, TypeRef};
use super::properties::{
    feature_columns, is_typed_prefix, map_multiplicity, map_type, strip_ticks, RowInput,
};
use super::scan::{blocks_in, comma_list, level2_headings, lines, Block, Table};
use super::{AvailabilityState, KindAvailability, SemanticDiagnostic};
use crate::extract::assert_eval::headers_conform;
use crate::extract::dsl::ExtractionDsl;
use crate::extract::locator::{Locator, LocatorPrimitive};

/// Every model feature FR-075 extracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelFeature {
    Generalization,
    AbstractTypes,
    Presence,
    Subsetting,
    Redefinition,
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
    /// The feature name used as a diagnostic `reason`; for a mapping
    /// feature it is also the `semantic.mappings` token that declares it.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Generalization => "generalization",
            Self::AbstractTypes => "abstract-types",
            Self::Presence => "presence",
            Self::Subsetting => "subsetting",
            Self::Redefinition => "redefinition",
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

    /// Is this mapping feature named by the module's `semantic.mappings`?
    /// A table feature is declared by a locator instead, never by a token.
    pub(crate) fn declared_by_mappings(self, ctx: &SemanticContext) -> bool {
        let is_mapping = match self {
            Self::Generalization
            | Self::AbstractTypes
            | Self::Presence
            | Self::Subsetting
            | Self::Redefinition
            | Self::EffectFrames => true,
            Self::Population
            | Self::Values
            | Self::States
            | Self::Transitions
            | Self::Steps
            | Self::Members
            | Self::Vocabulary => false,
        };
        is_mapping && ctx.module.mappings.iter().any(|m| m == self.name())
    }
}

/// One model table the engine can read: the columns it reads and how.
struct TableSpec {
    feature: ModelFeature,
    /// Every column the reader reads, in order; the first is the row key.
    columns: &'static [&'static str],
    /// Columns a declaring locator may not mark optional.
    required: &'static [&'static str],
    read: fn(&mut TableRead<'_>),
}

/// The single source of truth for the model tables (FR-075 Outputs).
const TABLE_SPECS: [TableSpec; 7] = [
    TableSpec {
        feature: ModelFeature::Population,
        columns: &["Type", "Extent"],
        required: &["Type", "Extent"],
        read: read_population,
    },
    TableSpec {
        feature: ModelFeature::Values,
        columns: &["Value", "Description"],
        required: &["Value"],
        read: read_values,
    },
    TableSpec {
        feature: ModelFeature::States,
        columns: &["State", "Description"],
        required: &["State"],
        read: read_states,
    },
    TableSpec {
        feature: ModelFeature::Transitions,
        columns: &["From", "To", "Trigger", "Guard", "Emits"],
        required: &["From", "To", "Trigger"],
        read: read_transitions,
    },
    TableSpec {
        feature: ModelFeature::Steps,
        columns: &["Step", "Kind", "Consumes", "Emits", "Description"],
        required: &["Step", "Kind"],
        read: read_steps,
    },
    TableSpec {
        feature: ModelFeature::Members,
        columns: &["Member", "Multiplicity"],
        required: &["Member", "Multiplicity"],
        read: read_members,
    },
    TableSpec {
        feature: ModelFeature::Vocabulary,
        columns: &["Term", "Description"],
        required: &["Term"],
        read: read_vocabulary,
    },
];

impl TableSpec {
    /// The table a column list names: its first column is a table's key and
    /// every column is one that table reads.
    fn for_columns<C: AsRef<str>>(columns: &[C]) -> Option<&'static TableSpec> {
        let first = strip_ticks(columns.first()?.as_ref());
        TABLE_SPECS.iter().find(|spec| {
            spec.columns[0] == first
                && columns
                    .iter()
                    .all(|c| spec.columns.contains(&strip_ticks(c.as_ref())))
        })
    }

    fn of(feature: ModelFeature) -> Option<&'static TableSpec> {
        TABLE_SPECS.iter().find(|spec| spec.feature == feature)
    }
}

/// One `table_row` locator of the object type's `body_extraction` that
/// declares a model table (FR-075 Inputs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclaredTable {
    section: String,
    columns: Vec<String>,
    optional: Vec<String>,
    feature: ModelFeature,
}

impl DeclaredTable {
    fn conforms(&self, table: &Table) -> bool {
        let headers: Vec<&str> = table.headers.iter().map(|h| strip_ticks(h)).collect();
        headers_conform(&headers, &self.columns, &self.optional)
    }
}

/// Every model table the object type declares.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct DeclaredTables(Vec<DeclaredTable>);

impl DeclaredTables {
    /// Read the `table_row` locators of a typed extraction DSL. A locator
    /// declares a model table when its `under_section` is set and its
    /// `assert.columns` name one table's key first, only columns that table
    /// reads, and none of its required columns in `optional_columns`.
    pub(crate) fn from_dsl(dsl: &ExtractionDsl) -> Self {
        let locators = dsl
            .yield_pattern
            .r#match
            .iter()
            .chain(dsl.yield_pattern.per_match.iter())
            .flat_map(|map| map.values())
            .flat_map(|locator| match locator {
                Locator::Primitive(p) => std::slice::from_ref(p),
                Locator::Fallback(chain) => chain.as_slice(),
            });
        let mut out = Vec::new();
        for primitive in locators {
            let LocatorPrimitive::TableRow {
                under_section: Some(section),
                assert: Some(assert),
                ..
            } = primitive
            else {
                continue;
            };
            let Some(columns) = &assert.columns else {
                continue;
            };
            let optional = assert.optional_columns.clone().unwrap_or_default();
            let Some(spec) = TableSpec::for_columns(columns) else {
                continue;
            };
            if spec
                .required
                .iter()
                .any(|r| optional.iter().any(|o| o == r))
            {
                continue;
            }
            out.push(DeclaredTable {
                section: section.clone(),
                columns: columns.clone(),
                optional,
                feature: spec.feature,
            });
        }
        Self(out)
    }

    fn for_section<'a>(&'a self, heading: &'a str) -> impl Iterator<Item = &'a DeclaredTable> {
        self.0.iter().filter(move |t| t.section == heading)
    }
}

/// Where a model feature is declared.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Section<'a> {
    Frontmatter,
    /// Before the first `##` heading.
    Preamble,
    /// A `##` heading.
    Heading(&'a str),
    /// An operation under `## Operations`.
    Operation(&'a str),
}

impl Section<'_> {
    /// The structured `section` value of a refusal.
    pub(crate) fn label(self) -> String {
        match self {
            Self::Frontmatter => "frontmatter".to_string(),
            Self::Preamble => "preamble".to_string(),
            Self::Heading(h) => h.to_string(),
            Self::Operation(op) => format!("Operations / {op}"),
        }
    }

    /// The section as a refusal message names it.
    pub(crate) fn display(self) -> String {
        match self {
            Self::Frontmatter => "frontmatter".to_string(),
            Self::Preamble => "the preamble before the first `##` heading".to_string(),
            Self::Heading(h) => format!("`## {h}`"),
            Self::Operation(op) => format!("`## Operations` / `### {op}`"),
        }
    }
}

/// `semantic.feature-not-extractable` (FR-075 Behavior): the artifact
/// declares `feature` in `section` at `line`, and its manifest does not.
pub(crate) fn not_extractable(
    ctx: &SemanticContext,
    lines: &[&str],
    feature: ModelFeature,
    section: Section<'_>,
    line: usize,
) -> SemanticDiagnostic {
    let mut d = SemanticDiagnostic::new(
        "semantic.feature-not-extractable",
        SemanticSeverity::Error,
        line,
        format!(
            "artifact {} declares {} in {} at line {line}; the module {} manifest does not declare it",
            ctx.path,
            feature.name(),
            section.display(),
            ctx.module.package
        ),
    )
    .with_reason(feature.name());
    d.source_span = Some(line_span(ctx, lines, line));
    d.section = Some(section.label());
    d
}

fn err(code: &str, line: usize, message: impl Into<String>) -> SemanticDiagnostic {
    SemanticDiagnostic::new(code, SemanticSeverity::Error, line, message)
}

/// `startLine`..`endLine` of one declaring line (FR-075 Outputs).
pub(crate) fn line_span(ctx: &SemanticContext, lines: &[&str], line: usize) -> SourceLocus {
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

/// The artifact `id` (the declaration's identity) or its `title` (the
/// declared class name), at its frontmatter line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityDecl {
    pub value: String,
    pub source_span: SourceLocus,
}

/// A frontmatter `specializes` relationship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupertypeDecl {
    pub target: String,
    pub source_span: SourceLocus,
}

/// Frontmatter `abstract: <bool>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractDecl {
    pub value: bool,
    pub source_span: SourceLocus,
}

/// The `Presence`/`Subsets`/`Redefines` cells of one Properties row.
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

/// The contract and frame lines of one operation.
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

/// One `Type | Extent` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulationMemberDecl {
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    pub extent: Multiplicity,
    pub source_span: SourceLocus,
}

/// A population's member types and extents.
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

/// A `From | To | Trigger | Guard | Emits` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionDecl {
    pub from: String,
    pub to: String,
    pub trigger: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emits: Option<Vec<String>>,
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

/// A `Kind` cell that names no [`StepKind`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("step kind {0:?} is not command, event, decision, compensation, or wait")]
pub struct UnknownStepKind(pub String);

impl std::str::FromStr for StepKind {
    type Err = UnknownStepKind;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "command" => Ok(Self::Command),
            "event" => Ok(Self::Event),
            "decision" => Ok(Self::Decision),
            "compensation" => Ok(Self::Compensation),
            "wait" => Ok(Self::Wait),
            other => Err(UnknownStepKind(other.to_string())),
        }
    }
}

/// A `Step | Kind | Consumes | Emits | Description` row.
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

/// A `Member | Multiplicity` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberDecl {
    pub target: String,
    pub multiplicity: Multiplicity,
    pub source_span: SourceLocus,
}

/// A `Term | Description` row.
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
    pub identity: Option<IdentityDecl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<IdentityDecl>,
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

/// What the frontmatter and table extraction of one artifact produced.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ModelOutcome {
    /// Identity, frontmatter, and table declarations.
    pub(crate) model: ModelDeclarations,
    /// Whether the artifact declared any frontmatter or table feature.
    pub(crate) declared: bool,
    pub(crate) lossy: bool,
    pub(crate) diagnostics: Vec<SemanticDiagnostic>,
}

/// Names a transition is checked against; `None` skips the check because
/// the referenced kind is already `unavailable`.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ModelRefs<'a> {
    pub(crate) clause_ids: Option<&'a [String]>,
    pub(crate) operation_names: Option<&'a [String]>,
}

/// FR-075 identity, frontmatter, and table features of one document.
pub(crate) fn extract_model(raw: &str, ctx: &SemanticContext, refs: ModelRefs<'_>) -> ModelOutcome {
    let lines = lines(raw);
    let mut out = ModelOutcome::default();
    let body_start = frontmatter_features(raw, &lines, ctx, &mut out);
    table_features(&lines, body_start, ctx, refs, &mut out);
    out
}

/// One model feature source for [`model_availability`].
pub(crate) struct ModelSource<'a> {
    /// The artifact declares a feature this source extracts.
    pub(crate) declared: bool,
    /// The source failed: its declarations are not carried.
    pub(crate) failed: bool,
    pub(crate) lossy: bool,
    pub(crate) diagnostics: &'a [SemanticDiagnostic],
}

/// `availability.model` (FR-075 General): absent when no source declares a
/// feature; `unavailable` with the error loci when any declaring source
/// failed; otherwise `available`.
pub(crate) fn model_availability(sources: &[ModelSource<'_>]) -> Option<KindAvailability> {
    let declaring: Vec<&ModelSource<'_>> = sources.iter().filter(|s| s.declared).collect();
    if declaring.is_empty() {
        return None;
    }
    if !declaring.iter().any(|s| s.failed) {
        return Some(KindAvailability::available(
            declaring.iter().any(|s| s.lossy),
        ));
    }
    let failed: Vec<SemanticDiagnostic> = declaring
        .iter()
        .filter(|s| s.failed)
        .flat_map(|s| s.diagnostics.iter().cloned())
        .collect();
    Some(entry_errors(&failed))
}

/// `unavailable` with reason `entry-errors: lines <lines>`: the error lines
/// of `diagnostics`, ascending, deduplicated, joined with `, `.
pub(crate) fn entry_errors(diagnostics: &[SemanticDiagnostic]) -> KindAvailability {
    let mut loci: Vec<usize> = diagnostics
        .iter()
        .filter(|d| d.is_error())
        .filter_map(|d| d.line)
        .collect();
    loci.sort_unstable();
    loci.dedup();
    let loci: Vec<String> = loci.iter().map(usize::to_string).collect();
    KindAvailability::unavailable(format!("entry-errors: lines {}", loci.join(", ")))
}

impl ModelOutcome {
    /// The frontmatter and table source for [`model_availability`].
    pub(crate) fn source(&self) -> ModelSource<'_> {
        ModelSource {
            declared: self.declared,
            failed: self.diagnostics.iter().any(SemanticDiagnostic::is_error),
            lossy: self.lossy,
            diagnostics: &self.diagnostics,
        }
    }
}

/// 1-based lines of the top-level frontmatter keys this module reads and
/// of each item under `relationships:`.
#[derive(Default)]
struct FrontmatterLines {
    id: Option<usize>,
    title: Option<usize>,
    abstract_type: Option<usize>,
    relationships: Option<usize>,
    items: Vec<usize>,
}

fn frontmatter_lines(lines: &[&str], header_lines: usize) -> FrontmatterLines {
    let mut out = FrontmatterLines::default();
    let mut in_relationships = false;
    for (i, raw) in lines.iter().enumerate().take(header_lines) {
        let line = i + 1;
        let text = raw.trim_end_matches('\r');
        let top_level = !text.starts_with(' ') && !text.starts_with('-') && !text.is_empty();
        if top_level {
            let key = text.split(':').next().unwrap_or("");
            in_relationships = key == "relationships";
            let slot = match key {
                "id" => &mut out.id,
                "title" => &mut out.title,
                "abstract" => &mut out.abstract_type,
                "relationships" => &mut out.relationships,
                _ => continue,
            };
            slot.get_or_insert(line);
            continue;
        }
        if in_relationships && text.trim_start().starts_with("- ") {
            out.items.push(line);
        }
    }
    out
}

/// Read the frontmatter features; returns the first body line.
fn frontmatter_features(
    raw: &str,
    lines: &[&str],
    ctx: &SemanticContext,
    out: &mut ModelOutcome,
) -> usize {
    let fm = crate::parser::frontmatter::extract_frontmatter_ref(raw);
    let Some(map) = fm.frontmatter else {
        return 1;
    };
    let header = &raw[..raw.len().saturating_sub(fm.body.len())];
    let header_lines = header.split('\n').count();
    let at = frontmatter_lines(lines, header_lines);
    let identity = |key: &str, line: Option<usize>| {
        let value = map.get(key)?.as_str()?;
        let line = line?;
        Some(IdentityDecl {
            value: value.to_string(),
            source_span: line_span(ctx, lines, line),
        })
    };
    out.model.identity = identity("id", at.id);
    out.model.display_name = identity("title", at.title);

    if let Some(value) = map.get("abstract") {
        out.declared = true;
        let line = at.abstract_type.unwrap_or(1);
        if !ModelFeature::AbstractTypes.declared_by_mappings(ctx) {
            out.diagnostics.push(not_extractable(
                ctx,
                lines,
                ModelFeature::AbstractTypes,
                Section::Frontmatter,
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

    if let Some(relationships) = map.get("relationships").and_then(Value::as_array) {
        let mut supertypes: Vec<SupertypeDecl> = Vec::new();
        for (i, rel) in relationships.iter().enumerate() {
            if rel.get("type").and_then(Value::as_str) != Some("specializes") {
                continue;
            }
            out.declared = true;
            let line = match at.items.get(i) {
                Some(line) if at.items.len() == relationships.len() => *line,
                _ => at.relationships.unwrap_or(1),
            };
            if !ModelFeature::Generalization.declared_by_mappings(ctx) {
                out.diagnostics.push(not_extractable(
                    ctx,
                    lines,
                    ModelFeature::Generalization,
                    Section::Frontmatter,
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
    header_lines.max(1)
}

fn block_line(block: &Block) -> usize {
    match block {
        Block::Table(t) => t.line,
        Block::Fence(f) => f.open_line,
        Block::List { line } => *line,
    }
}

fn table_features(
    lines: &[&str],
    body_start: usize,
    ctx: &SemanticContext,
    refs: ModelRefs<'_>,
    out: &mut ModelOutcome,
) {
    let headings = level2_headings(lines);
    let preamble_end = headings.first().map_or(lines.len() + 1, |h| h.1);
    let refuse = |out: &mut ModelOutcome, feature, section, line| {
        out.declared = true;
        out.diagnostics
            .push(not_extractable(ctx, lines, feature, section, line));
    };
    let mut read: Vec<ModelFeature> = Vec::new();
    let mut failed: Vec<ModelFeature> = Vec::new();
    for block in blocks_in(lines, body_start, preamble_end) {
        if let Block::Table(table) = &block {
            if let Some(spec) = TableSpec::for_columns(&table.headers) {
                refuse(out, spec.feature, Section::Preamble, table.line);
                failed.push(spec.feature);
            } else if is_typed_prefix(&table.headers) {
                // A typed Properties table with feature columns outside
                // `## Properties` declares features no extractor reads.
                for column in feature_columns(&table.headers).unwrap_or_default() {
                    refuse(out, column.feature(), Section::Preamble, table.line);
                }
            }
        }
    }

    for (heading, start, end) in &headings {
        let owners: Vec<&DeclaredTable> = ctx.declared_tables.for_section(heading).collect();
        let blocks = blocks_in(lines, start + 1, *end);
        let Some(first_owner) = owners.first() else {
            for block in &blocks {
                if let Block::Table(table) = block {
                    if let Some(spec) = TableSpec::for_columns(&table.headers) {
                        refuse(out, spec.feature, Section::Heading(heading), table.line);
                        failed.push(spec.feature);
                    }
                }
            }
            continue;
        };
        out.declared |= !blocks.is_empty();
        for block in &blocks {
            let owner = match block {
                Block::Table(table) => owners.iter().find(|o| o.conforms(table)),
                Block::Fence(_) | Block::List { .. } => None,
            };
            let (Some(owner), Block::Table(table)) = (owner, block) else {
                refuse(
                    out,
                    first_owner.feature,
                    Section::Heading(heading),
                    block_line(block),
                );
                failed.push(first_owner.feature);
                continue;
            };
            if read.contains(&owner.feature) {
                out.diagnostics.push(err(
                    "semantic.duplicate-section",
                    table.line,
                    format!("a second {} table; one per artifact", owner.feature.name()),
                ));
                failed.push(owner.feature);
                continue;
            }
            read.push(owner.feature);
            let Some(spec) = TableSpec::of(owner.feature) else {
                continue;
            };
            let before = out.diagnostics.len();
            (spec.read)(&mut TableRead {
                feature: spec.feature,
                table,
                lines,
                ctx,
                out,
                keys: Vec::new(),
            });
            if out.diagnostics[before..].iter().any(|d| d.is_error()) {
                failed.push(spec.feature);
            }
        }
    }
    check_transitions(refs, failed.contains(&ModelFeature::States), out);
}

/// The reader state for one declared model table.
struct TableRead<'a> {
    feature: ModelFeature,
    table: &'a Table,
    lines: &'a [&'a str],
    ctx: &'a SemanticContext,
    out: &'a mut ModelOutcome,
    /// Row keys seen so far, for `semantic.duplicate-model-entry`.
    keys: Vec<String>,
}

impl TableRead<'_> {
    /// The cell under `column`, empty when the column is absent (declared
    /// optional) or the row is short.
    fn cell<'c>(&self, cells: &'c [String], column: &str) -> &'c str {
        self.table
            .headers
            .iter()
            .position(|h| strip_ticks(h) == column)
            .and_then(|i| cells.get(i))
            .map_or("", |c| strip_ticks(c))
    }

    fn span(&self, line: usize) -> SourceLocus {
        line_span(self.ctx, self.lines, line)
    }

    fn error(&mut self, code: &str, line: usize, message: impl Into<String>) {
        self.out.diagnostics.push(err(code, line, message));
    }

    fn identifier(&mut self, text: &str, what: &str, line: usize) -> bool {
        let ok = is_identifier(text);
        if !ok {
            self.error(
                "semantic.invalid-model-cell",
                line,
                format!("{what} {text:?} is not an Identifier"),
            );
        }
        ok
    }

    fn non_empty(&mut self, text: &str, what: &str, line: usize) -> bool {
        let ok = !text.is_empty();
        if !ok {
            self.error(
                "semantic.invalid-model-cell",
                line,
                format!("a {} row names no {what}", self.feature.name()),
            );
        }
        ok
    }

    /// Record `key`; false (with the diagnostic) when already seen.
    fn fresh(&mut self, key: &str, line: usize) -> bool {
        if self.keys.iter().any(|k| k == key) {
            let message = format!("{} {key} is declared twice", self.feature.name());
            self.error("semantic.duplicate-model-entry", line, message);
            return false;
        }
        self.keys.push(key.to_string());
        true
    }
}

fn optional(text: &str) -> Option<String> {
    (!text.is_empty()).then(|| text.to_string())
}

fn name_list(text: &str) -> Option<Vec<String>> {
    let items: Vec<String> = comma_list(text).map(str::to_string).collect();
    (!items.is_empty()).then_some(items)
}

fn read_enum(r: &mut TableRead<'_>, key: &str) -> Vec<EnumValueDecl> {
    let table = r.table;
    let what = key.to_ascii_lowercase();
    let mut entries = Vec::new();
    for (line, cells) in &table.rows {
        let value = r.cell(cells, key);
        if !r.identifier(value, &what, *line) || !r.fresh(value, *line) {
            continue;
        }
        entries.push(EnumValueDecl {
            value: value.to_string(),
            doc: optional(r.cell(cells, "Description")),
            source_span: r.span(*line),
        });
    }
    entries
}

fn read_values(r: &mut TableRead<'_>) {
    let entries = read_enum(r, "Value");
    r.out.model.values = Some(entries);
}

fn read_states(r: &mut TableRead<'_>) {
    let entries = read_enum(r, "State");
    r.out.model.states = Some(entries);
}

fn read_transitions(r: &mut TableRead<'_>) {
    let table = r.table;
    let mut entries = Vec::new();
    for (line, cells) in &table.rows {
        let (from, to, trigger) = (
            r.cell(cells, "From"),
            r.cell(cells, "To"),
            r.cell(cells, "Trigger"),
        );
        let ok = r.identifier(from, "from state", *line)
            & r.identifier(to, "to state", *line)
            & r.identifier(trigger, "trigger", *line);
        if !ok {
            continue;
        }
        entries.push(TransitionDecl {
            from: from.to_string(),
            to: to.to_string(),
            trigger: trigger.to_string(),
            guard: optional(r.cell(cells, "Guard")),
            emits: name_list(r.cell(cells, "Emits")),
            source_span: r.span(*line),
        });
    }
    r.out.model.transitions = Some(entries);
}

fn read_steps(r: &mut TableRead<'_>) {
    let table = r.table;
    let mut entries = Vec::new();
    for (line, cells) in &table.rows {
        let name = r.cell(cells, "Step");
        let kind = match r.cell(cells, "Kind").parse::<StepKind>() {
            Ok(kind) => Some(kind),
            Err(unknown) => {
                r.error("semantic.invalid-model-cell", *line, unknown.to_string());
                None
            }
        };
        let named = r.identifier(name, "step", *line);
        let (Some(kind), true) = (kind, named) else {
            continue;
        };
        if !r.fresh(name, *line) {
            continue;
        }
        entries.push(StepDecl {
            name: name.to_string(),
            kind,
            consumes: name_list(r.cell(cells, "Consumes")),
            emits: name_list(r.cell(cells, "Emits")),
            doc: optional(r.cell(cells, "Description")),
            source_span: r.span(*line),
        });
    }
    r.out.model.steps = Some(entries);
}

fn read_members(r: &mut TableRead<'_>) {
    let table = r.table;
    let mut entries = Vec::new();
    for (line, cells) in &table.rows {
        let target = r.cell(cells, "Member");
        if !r.non_empty(target, "member", *line) {
            continue;
        }
        let Some(multiplicity) =
            map_multiplicity(r.cell(cells, "Multiplicity"), *line, &mut r.out.diagnostics)
        else {
            continue;
        };
        if !r.fresh(target, *line) {
            continue;
        }
        entries.push(MemberDecl {
            target: target.to_string(),
            multiplicity,
            source_span: r.span(*line),
        });
    }
    r.out.model.members = Some(entries);
}

fn read_vocabulary(r: &mut TableRead<'_>) {
    let table = r.table;
    let mut entries = Vec::new();
    for (line, cells) in &table.rows {
        let term = r.cell(cells, "Term");
        if !r.non_empty(term, "term", *line) || !r.fresh(term, *line) {
            continue;
        }
        entries.push(TermDecl {
            term: term.to_string(),
            doc: r.cell(cells, "Description").to_string(),
            source_span: r.span(*line),
        });
    }
    r.out.model.vocabulary = Some(entries);
}

fn read_population(r: &mut TableRead<'_>) {
    let table = r.table;
    let mut members = Vec::new();
    for (line, cells) in &table.rows {
        let type_cell = r.cell(cells, "Type");
        let row = RowInput {
            line: *line,
            name: type_cell.to_string(),
            type_cell: type_cell.to_string(),
            mult_cell: r.cell(cells, "Extent").to_string(),
            constraints_cell: String::new(),
            reference_only: false,
        };
        let (type_ref, lossy) = map_type(type_cell, &row, r.ctx, &mut r.out.diagnostics);
        let extent = map_multiplicity(&row.mult_cell, *line, &mut r.out.diagnostics);
        let (Some(type_ref), Some(extent)) = (type_ref, extent) else {
            continue;
        };
        if !r.fresh(type_cell, *line) {
            continue;
        }
        r.out.lossy |= lossy;
        members.push(PopulationMemberDecl {
            type_ref,
            extent,
            source_span: r.span(*line),
        });
    }
    r.out.model.population = Some(PopulationDecl { members });
}

/// Transition reader rules: `from`/`to` name a state, `trigger` an
/// operation, `guard` an invariant clause of the same artifact. The state
/// check is skipped when the states table failed.
fn check_transitions(refs: ModelRefs<'_>, states_failed: bool, out: &mut ModelOutcome) {
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
    let mut found = Vec::new();
    for t in transitions {
        let line = t.source_span.start_line;
        if !states_failed {
            for state in [&t.from, &t.to] {
                if !states.contains(&state.as_str()) {
                    found.push(err(
                        "semantic.unknown-state",
                        line,
                        format!("transition state {state} names no declared state"),
                    ));
                }
            }
        }
        if let Some(ops) = refs.operation_names {
            if !ops.contains(&t.trigger) {
                found.push(err(
                    "semantic.unknown-trigger",
                    line,
                    format!("transition trigger {} names no operation", t.trigger),
                ));
            }
        }
        if let (Some(guard), Some(clauses)) = (&t.guard, refs.clause_ids) {
            if !clauses.contains(guard) {
                found.push(err(
                    "semantic.dangling-clause-ref",
                    line,
                    format!("guard {guard} is declared by no invariant of this artifact"),
                ));
            }
        }
    }
    out.diagnostics.extend(found);
}

/// Whether `availability` makes the kind's declarations absent.
pub(crate) fn failed(availability: &KindAvailability) -> bool {
    availability.state == AvailabilityState::Unavailable
}
