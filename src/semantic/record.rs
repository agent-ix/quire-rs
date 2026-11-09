//! Record-sections extraction (FR-081, quoin FR-121/FR-122/FR-123): the named
//! sections of an object artifact (a typed table, a bullet list, or one inline
//! value) to the record keys of the same name that the object type's
//! `data_schema` declares, gated by the `record-sections` mapping token.
//!
//! The mapping is schema-directed. [`RecordSchema`] reads each top-level
//! property's JSON type, `$ref` and `anyOf` branches once, when the context is
//! built; extraction then needs no schema access. A key some other mapping
//! owns ([`KeyOwner`]) is never read here, so every key has one authored
//! source.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Number, Value};

use super::clauses::{declared_operation_names, SourceLocus};
use super::context::SemanticContext;
use super::contract::SemanticSeverity;
use super::embedded::{semantic_core_bundle, SEMANTIC_CORE_BASE};
use super::model::{entry_errors, line_span, MemberDecl, ModelDeclarations, ModelFeature};
use super::properties::{map_multiplicity, map_type, strip_ticks, RowInput};
use super::relations::bundle_package;
use super::relations::RelationDecl;
use super::scan::{blocks_in, level2_headings, lines, Block, Fence, Table};
use super::target::{
    is_id_segment, resolve_reference, resolve_target, Admission, Declarations, OwnArtifact,
    OwnDeclaration, ReferenceRefusal, Target,
};
use super::{AvailabilityState, KindAvailability, SemanticDiagnostic};

/// The `semantic.mappings` token that declares record-sections extraction.
pub const RECORD_SECTIONS_TOKEN: &str = "record-sections";

/// The module whose record keys quoin FR-122 gives their own forms.
const ARCHITECTURE_PACKAGE: &str = "agent-ix/spec-objects-architecture";

/// The module whose unmapped keys quoin FR-123 lists.
const BUSINESS_PACKAGE: &str = "agent-ix/spec-objects-business";

/// Keys of the architecture module lowered from the `## Layout` fence.
const FENCE_KEYS: [&str; 2] = ["records", "endianness"];

/// The systems-model members FR-075 lowers beside `fields` and `operations`.
const SYSTEMS_MEMBERS: [&str; 10] = [
    "owner",
    "declaredType",
    "multiplicity",
    "direction",
    "interfaceType",
    "sourceEnd",
    "targetEnd",
    "flowDirection",
    "sourceElement",
    "targetElement",
];

/// The record keys a declared FR-075 table locator fills.
const MODEL_TABLE_KEYS: [ModelFeature; 6] = [
    ModelFeature::Values,
    ModelFeature::States,
    ModelFeature::Transitions,
    ModelFeature::Steps,
    ModelFeature::Members,
    ModelFeature::Vocabulary,
];

/// Deepest `$ref` chain followed when typing a property.
const MAX_REF_DEPTH: u8 = 8;

/// Who authors a top-level record key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyOwner {
    /// quoin FR-071 `## Properties`.
    Fields,
    /// quoin FR-072 `## Invariants`.
    Clauses,
    /// quoin FR-072 `## Operations`.
    Operations,
    /// quoin FR-104 `## Relationships` rows.
    Relations,
    /// quire-rs FR-075 `Features` table.
    FeatureOrder,
    /// quire-rs FR-075 systems-model table.
    SystemsMember,
    /// quire-rs FR-076 typed relation key.
    TypedRelation,
    /// quire-rs FR-075 table locator.
    ModelTable,
    /// quoin FR-123 lists the key as having no authored source.
    Unmapped,
    /// This mapping (quire-rs FR-081; quoin FR-121 and FR-122).
    RecordSections,
}

/// The scalar a cell or list item is typed as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scalar {
    Integer,
    Number,
    Boolean,
    Text,
    IntegerOrText,
    SemanticId,
}

/// One property of an object-valued key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Column {
    name: String,
    /// `None` for a property typed as an object or an array.
    scalar: Option<Scalar>,
    /// The properties of an object-typed property.
    nested: Vec<Column>,
}

/// The reading shape of one record property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Node {
    Scalar(Scalar),
    ScalarArray(Scalar),
    Object(Vec<Column>),
    ObjectArray(Vec<Column>),
    TypeRefArray,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecordProperty {
    name: String,
    node: Node,
}

/// What a schema node types as, before it is shaped into a [`Node`].
enum Typed {
    Scalar(Scalar),
    Object(Vec<Column>),
    Array(Box<Typed>),
    TypeRef,
    Unknown,
}

/// The record schema FR-081 reads: the top-level properties of the object
/// type's `data_schema` with their shapes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RecordSchema {
    properties: Vec<RecordProperty>,
}

/// The name a semantic-core `$ref` url ends in.
fn core_name(url: &str) -> Option<&str> {
    url.strip_prefix(SEMANTIC_CORE_BASE)?
        .split_once('/')
        .map(|(_, name)| name)
}

struct Documents<'a>(&'a BTreeMap<String, Value>);

impl Documents<'_> {
    /// The document a `$ref` url names: a supplied module document, else the
    /// embedded semantic-core bundle.
    fn lookup(&self, url: &str) -> Option<Value> {
        if let Some(document) = self.0.get(url) {
            return Some(document.clone());
        }
        let name = core_name(url)?;
        let (_, text) = semantic_core_bundle().iter().find(|(n, _)| *n == name)?;
        serde_json::from_str(text).ok()
    }

    /// Type `node` of the document `doc`. A `$ref` to a fragment of `doc`
    /// (`#/$defs/X`, or `doc`'s own `$id` with a fragment) is followed in
    /// place; any other is followed through the supplied documents.
    fn classify(&self, node: &Value, doc: &Value, depth: u8) -> Typed {
        if depth > MAX_REF_DEPTH {
            return Typed::Unknown;
        }
        if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
            let (url, fragment) = reference.split_once('#').unwrap_or((reference, ""));
            if url.is_empty() || doc.get("$id").and_then(Value::as_str) == Some(url) {
                return doc.pointer(fragment).map_or(Typed::Unknown, |target| {
                    self.classify(target, doc, depth + 1)
                });
            }
            return match core_name(url) {
                Some("SemanticId.json") => Typed::Scalar(Scalar::SemanticId),
                Some("TypeRef.json") => Typed::TypeRef,
                _ => self.lookup(url).map_or(Typed::Unknown, |other| {
                    other.pointer(fragment).map_or(Typed::Unknown, |target| {
                        self.classify(target, &other, depth + 1)
                    })
                }),
            };
        }
        if let Some(branches) = node
            .get("anyOf")
            .or_else(|| node.get("oneOf"))
            .and_then(Value::as_array)
        {
            return self.classify_branches(branches, doc, depth);
        }
        if let Some(values) = node.get("enum").and_then(Value::as_array) {
            return Self::classify_enum(values);
        }
        match node.get("type").and_then(Value::as_str) {
            Some("string") => Typed::Scalar(Scalar::Text),
            Some("integer") => Typed::Scalar(Scalar::Integer),
            Some("number") => Typed::Scalar(Scalar::Number),
            Some("boolean") => Typed::Scalar(Scalar::Boolean),
            Some("object") => Typed::Object(self.columns(node, doc, depth)),
            Some("array") => node.get("items").map_or(Typed::Unknown, |items| {
                Typed::Array(Box::new(self.classify(items, doc, depth + 1)))
            }),
            _ if node.get("properties").is_some() => Typed::Object(self.columns(node, doc, depth)),
            _ => Typed::Unknown,
        }
    }

    /// An `anyOf`: all strings, or an integer and a string.
    fn classify_branches(&self, branches: &[Value], doc: &Value, depth: u8) -> Typed {
        let scalars: Option<Vec<Scalar>> = branches
            .iter()
            .map(|branch| match self.classify(branch, doc, depth + 1) {
                Typed::Scalar(scalar) => Some(scalar),
                _ => None,
            })
            .collect();
        let Some(scalars) = scalars else {
            return Typed::Unknown;
        };
        let has = |wanted: Scalar| scalars.contains(&wanted);
        match scalars.as_slice() {
            [] => Typed::Unknown,
            all if all.iter().all(|s| *s == Scalar::Text) => Typed::Scalar(Scalar::Text),
            all if all
                .iter()
                .all(|s| matches!(s, Scalar::Integer | Scalar::Text))
                && has(Scalar::Integer) =>
            {
                Typed::Scalar(Scalar::IntegerOrText)
            }
            [one, rest @ ..] if rest.iter().all(|s| s == one) => Typed::Scalar(*one),
            _ => Typed::Unknown,
        }
    }

    fn classify_enum(values: &[Value]) -> Typed {
        if values.iter().all(Value::is_string) && !values.is_empty() {
            Typed::Scalar(Scalar::Text)
        } else if values.iter().all(Value::is_i64) && !values.is_empty() {
            Typed::Scalar(Scalar::Integer)
        } else {
            Typed::Unknown
        }
    }

    fn columns(&self, node: &Value, doc: &Value, depth: u8) -> Vec<Column> {
        node.get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(
                |(name, property)| match self.classify(property, doc, depth + 1) {
                    Typed::Scalar(scalar) => Column {
                        name: name.clone(),
                        scalar: Some(scalar),
                        nested: Vec::new(),
                    },
                    Typed::Object(nested) => Column {
                        name: name.clone(),
                        scalar: None,
                        nested,
                    },
                    Typed::Array(_) | Typed::TypeRef | Typed::Unknown => Column {
                        name: name.clone(),
                        scalar: None,
                        nested: Vec::new(),
                    },
                },
            )
            .collect()
    }
}

impl RecordSchema {
    /// The schema of `root`, following `$ref`s through `documents` and the
    /// embedded semantic-core bundle.
    pub(crate) fn new(root: &Value, documents: &BTreeMap<String, Value>) -> Self {
        let documents = Documents(documents);
        let properties = root
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(name, property)| RecordProperty {
                name: name.clone(),
                node: match documents.classify(property, root, 0) {
                    Typed::Scalar(scalar) => Node::Scalar(scalar),
                    Typed::Object(columns) => Node::Object(columns),
                    Typed::Array(items) => match *items {
                        Typed::Scalar(scalar) => Node::ScalarArray(scalar),
                        Typed::Object(columns) => Node::ObjectArray(columns),
                        Typed::TypeRef => Node::TypeRefArray,
                        Typed::Array(_) | Typed::Unknown => Node::Unreadable,
                    },
                    Typed::TypeRef | Typed::Unknown => Node::Unreadable,
                },
            })
            .collect();
        Self { properties }
    }

    fn owner(&self, ctx: &SemanticContext, property: &RecordProperty) -> Option<KeyOwner> {
        let name = property.name.as_str();
        let package = ctx.module.package.as_str();
        match name {
            "fields" => return Some(KeyOwner::Fields),
            "clauses" => return Some(KeyOwner::Clauses),
            "operations" => return Some(KeyOwner::Operations),
            "relations" => return Some(KeyOwner::Relations),
            "featureOrder" => return Some(KeyOwner::FeatureOrder),
            _ => {}
        }
        if MODEL_TABLE_KEYS
            .iter()
            .any(|feature| feature.name() == name && ctx.declared_tables.declares(*feature))
        {
            return Some(KeyOwner::ModelTable);
        }
        // quoin FR-123: no section fills these keys of the business module.
        let unmapped = matches!(
            (ctx.object_type_name(), name),
            (Some("nested_entity"), "owner")
                | (Some("domain"), "ubiquitous_language")
                | (Some("domain" | "population"), "members")
        );
        if package == BUSINESS_PACKAGE && unmapped {
            return Some(KeyOwner::Unmapped);
        }
        if SYSTEMS_MEMBERS.contains(&name) {
            return Some(KeyOwner::SystemsMember);
        }
        if matches!(property.node, Node::ScalarArray(Scalar::SemanticId))
            && ctx
                .relation_vocabulary
                .as_ref()
                .and_then(|v| v.allowed_targets(name))
                .is_some()
        {
            return Some(KeyOwner::TypedRelation);
        }
        let readable = match property.node {
            Node::Unreadable => false,
            Node::TypeRefArray => name == "associated_types",
            Node::Scalar(_) | Node::ScalarArray(_) | Node::Object(_) | Node::ObjectArray(_) => true,
        };
        readable.then_some(KeyOwner::RecordSections)
    }

    fn has(&self, name: &str) -> bool {
        self.properties.iter().any(|p| p.name == name)
    }
}

/// The owner of every top-level property of the record schema in `ctx`, in
/// schema order; `None` for a property no mapping can author. The ownership
/// audit of quoin FR-122-AC-7 fails on a `None`.
pub fn record_key_owners(ctx: &SemanticContext) -> Vec<(String, Option<KeyOwner>)> {
    ctx.record_schema
        .iter()
        .flat_map(|schema| {
            schema
                .properties
                .iter()
                .map(|p| (p.name.clone(), schema.owner(ctx, p)))
        })
        .collect()
}

/// `record-sections` lowercases and drops spaces, hyphens and underscores.
fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(c, ' ' | '-' | '_'))
        .flat_map(char::to_lowercase)
        .collect()
}

/// The span of one extracted key's section (FR-081 Outputs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordSource {
    pub key: String,
    pub source_span: SourceLocus,
}

/// What record-sections extraction produced for one artifact.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct RecordOutcome {
    pub(crate) record: BTreeMap<String, Value>,
    pub(crate) sources: Vec<RecordSource>,
    pub(crate) availability: BTreeMap<String, KindAvailability>,
    pub(crate) diagnostics: Vec<SemanticDiagnostic>,
}

/// One `##` section: its heading text, heading line and exclusive end line.
#[derive(Clone, Copy)]
struct Sec<'a> {
    text: &'a str,
    heading: usize,
    end: usize,
}

/// What reading one section produced.
struct Read {
    values: Vec<(String, Option<Value>)>,
    diagnostics: Vec<SemanticDiagnostic>,
    advisories: Vec<SemanticDiagnostic>,
    no_package: bool,
    span: SourceLocus,
}

/// Where a heading's content goes.
enum Source<'a> {
    Key(&'a RecordProperty),
    Layout,
}

/// A table column a header cell can match.
struct HeaderTarget {
    norm: String,
    path: Vec<String>,
    kind: Kind,
}

#[derive(Clone, Copy)]
enum Kind {
    Scalar(Scalar),
    Nested,
}

struct Reader<'a> {
    ctx: &'a SemanticContext,
    lines: &'a [&'a str],
    package: Option<&'a str>,
    own_id: Option<String>,
    own_object: Option<String>,
    operations: Vec<String>,
}

/// Read the record-key sections of `raw` (FR-081); `None` when the module
/// does not name the token or no record schema is supplied.
pub(crate) fn extract_record(raw: &str, ctx: &SemanticContext) -> Option<RecordOutcome> {
    if !ctx
        .module
        .mappings
        .iter()
        .any(|m| m == RECORD_SECTIONS_TOKEN)
    {
        return None;
    }
    let schema = ctx.record_schema.as_ref()?;
    let all_lines = lines(raw);
    let frontmatter = crate::parser::frontmatter::extract_frontmatter_ref(raw).frontmatter;
    let own = |key: &str| {
        frontmatter
            .as_ref()
            .and_then(|m| m.get(key))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let reader = Reader {
        ctx,
        lines: &all_lines,
        package: bundle_package(ctx),
        own_id: own("id"),
        own_object: own("object"),
        operations: declared_operation_names(raw),
    };
    let architecture = ctx.module.package == ARCHITECTURE_PACKAGE;
    let mut groups: BTreeMap<String, Vec<&RecordProperty>> = BTreeMap::new();
    for property in &schema.properties {
        groups
            .entry(normalize(&property.name))
            .or_default()
            .push(property);
    }
    let layout_declared = architecture && FENCE_KEYS.iter().any(|k| schema.has(k));
    let mut outcome = RecordOutcome::default();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut ambiguous: BTreeSet<String> = BTreeSet::new();
    let mut unreadable: BTreeSet<String> = BTreeSet::new();
    let mut reads: Vec<(String, Read)> = Vec::new();
    let mut duplicates: BTreeMap<String, Vec<SemanticDiagnostic>> = BTreeMap::new();
    for (text, line, end) in level2_headings(&all_lines) {
        let sec = Sec {
            text: &text,
            heading: line,
            end,
        };
        let norm = normalize(&text);
        let source = if layout_declared && norm == "layout" {
            Source::Layout
        } else {
            let Some(group) = groups.get(&norm) else {
                continue;
            };
            let readable: Vec<&RecordProperty> = group
                .iter()
                .copied()
                .filter(|p| {
                    !(architecture && FENCE_KEYS.contains(&p.name.as_str()))
                        && schema.owner(ctx, p) == Some(KeyOwner::RecordSections)
                })
                .collect();
            if group.len() > 1 && !readable.is_empty() {
                ambiguous.insert(norm.clone());
                let names: Vec<String> = group.iter().map(|p| format!("`{}`", p.name)).collect();
                outcome.diagnostics.push(reader.finding(
                    "semantic.feature-not-extractable",
                    "key-ambiguous",
                    sec,
                    line,
                    format!(
                        "properties {} normalize to the same name",
                        names.join(" and ")
                    ),
                ));
                continue;
            }
            let [property] = readable.as_slice() else {
                // A key no mapping can read is refused, never silently prose.
                for p in group.iter().filter(|p| schema.owner(ctx, p).is_none()) {
                    if unreadable.insert(p.name.clone()) {
                        outcome.diagnostics.push(reader.finding(
                            "semantic.feature-not-extractable",
                            "schema-unreadable",
                            sec,
                            line,
                            format!(
                                "record key `{}` has a schema shape record-sections cannot read: it is not a scalar, an object, an array of scalars or of objects, or a `$ref` resolves to no supplied document",
                                p.name
                            ),
                        ));
                    }
                }
                continue;
            };
            Source::Key(property)
        };
        let id = match &source {
            Source::Key(property) => property.name.clone(),
            Source::Layout => "## Layout".to_string(),
        };
        if !seen.insert(id.clone()) {
            duplicates.entry(id).or_default().push(reader.finding(
                "semantic.duplicate-section",
                "second-section",
                sec,
                line,
                "a second section names the same record key".to_string(),
            ));
            continue;
        }
        let read = match source {
            Source::Key(property) => reader.read_key(property, sec),
            Source::Layout => reader.read_layout(sec),
        };
        reads.push((id, read));
    }
    for (id, read) in reads {
        let mut diagnostics = read.diagnostics;
        diagnostics.extend(duplicates.remove(&id).into_iter().flatten());
        let failed = diagnostics.iter().any(SemanticDiagnostic::is_error);
        if !failed {
            diagnostics.extend(read.advisories);
        }
        for (key, value) in read.values {
            if !schema.has(&key) {
                continue;
            }
            let state = if failed {
                entry_errors(&diagnostics)
            } else if read.no_package {
                KindAvailability::unavailable("no-bundle-package")
            } else if let Some(value) = value {
                outcome.record.insert(key.clone(), value);
                outcome.sources.push(RecordSource {
                    key: key.clone(),
                    source_span: read.span.clone(),
                });
                KindAvailability::available(false)
            } else {
                KindAvailability::not_applicable()
            };
            outcome.availability.insert(key, state);
        }
        outcome.diagnostics.extend(diagnostics);
    }
    for key in &unreadable {
        outcome.availability.insert(
            key.clone(),
            KindAvailability::unavailable("schema-unreadable"),
        );
    }
    for property in &schema.properties {
        let readable = schema.owner(ctx, property) == Some(KeyOwner::RecordSections)
            && !ambiguous.contains(&normalize(&property.name));
        if readable && !outcome.availability.contains_key(&property.name) {
            outcome
                .availability
                .insert(property.name.clone(), KindAvailability::not_applicable());
        }
    }
    Some(outcome)
}

/// The mutable result of reading one section.
#[derive(Default)]
struct State {
    diagnostics: Vec<SemanticDiagnostic>,
    advisories: Vec<SemanticDiagnostic>,
    no_package: bool,
}

impl Reader<'_> {
    fn finding(
        &self,
        code: &str,
        reason: &str,
        sec: Sec<'_>,
        line: usize,
        detail: String,
    ) -> SemanticDiagnostic {
        let mut d = SemanticDiagnostic::new(
            code,
            SemanticSeverity::Error,
            line,
            format!(
                "artifact {} declares {RECORD_SECTIONS_TOKEN} in `## {}` at line {line}: {detail}",
                self.ctx.path, sec.text
            ),
        )
        .with_reason(reason);
        d.source_span = Some(line_span(self.ctx, self.lines, line));
        d.section = Some(sec.text.to_string());
        d
    }

    fn severity(mut d: SemanticDiagnostic, severity: SemanticSeverity) -> SemanticDiagnostic {
        d.severity = severity;
        d
    }

    fn span(&self, sec: Sec<'_>) -> SourceLocus {
        section_span(self.ctx, self.lines, sec)
    }

    fn read(&self, sec: Sec<'_>, keys: &[&str], state: State, values: Vec<Option<Value>>) -> Read {
        Read {
            values: keys
                .iter()
                .zip(values)
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
            diagnostics: state.diagnostics,
            advisories: state.advisories,
            no_package: state.no_package,
            span: self.span(sec),
        }
    }

    fn no_block(&self, sec: Sec<'_>, keys: &[&str], mut state: State) -> Read {
        state.diagnostics.push(Self::severity(
            self.finding(
                "semantic.record-key-no-block",
                "no-block",
                sec,
                sec.heading,
                "the section holds no block; nothing is extracted".to_string(),
            ),
            SemanticSeverity::Warning,
        ));
        self.read(sec, keys, state, vec![None; keys.len()])
    }

    /// The non-blank lines of the section (1-based line, trimmed text).
    fn text_lines(&self, sec: Sec<'_>) -> Vec<(usize, &str)> {
        (sec.heading + 1..sec.end.min(self.lines.len() + 1))
            .map(|line| (line, self.lines[line - 1].trim()))
            .filter(|(_, text)| !text.is_empty())
            .collect()
    }

    fn not_of_form(&self, sec: Sec<'_>, reason: &str, line: usize, state: &mut State) {
        state.diagnostics.push(self.finding(
            "semantic.feature-not-extractable",
            reason,
            sec,
            line,
            format!("the section is not of the form of record key `{reason}`"),
        ));
    }

    fn read_key(&self, property: &RecordProperty, sec: Sec<'_>) -> Read {
        let name = property.name.as_str();
        let keys = [name];
        let mut state = State::default();
        let blocks = blocks_in(self.lines, sec.heading + 1, sec.end);
        let line_of = |block: &Block| match block {
            Block::Table(table) => table.line,
            Block::Fence(fence) => fence.open_line,
            Block::List { line } => *line,
        };
        match &property.node {
            Node::Scalar(scalar) => {
                for block in &blocks {
                    self.not_of_form(sec, name, line_of(block), &mut state);
                }
                if !blocks.is_empty() {
                    return self.read(sec, &keys, state, vec![None]);
                }
                let text = self.text_lines(sec);
                let Some(((line, first), rest)) = text.split_first() else {
                    return self.no_block(sec, &keys, state);
                };
                if let Some((line, _)) = rest.first() {
                    self.not_of_form(sec, name, *line, &mut state);
                    return self.read(sec, &keys, state, vec![None]);
                }
                let inline = first
                    .strip_prefix('`')
                    .and_then(|s| s.strip_suffix('`'))
                    .filter(|s| !s.is_empty() && !s.contains('`'));
                let Some(inline) = inline else {
                    self.not_of_form(sec, name, *line, &mut state);
                    return self.read(sec, &keys, state, vec![None]);
                };
                let value = self.type_scalar(*scalar, inline.trim(), sec, *line, &mut state);
                self.read(sec, &keys, state, vec![value])
            }
            Node::ScalarArray(scalar) => {
                if blocks.is_empty() {
                    return self.no_block(sec, &keys, state);
                }
                for (i, block) in blocks.iter().enumerate() {
                    if i > 0 || !matches!(block, Block::List { .. }) {
                        self.not_of_form(sec, name, line_of(block), &mut state);
                    }
                }
                let Some(Block::List { line: start }) = blocks.first() else {
                    return self.read(sec, &keys, state, vec![None]);
                };
                let stop = blocks.get(1).map_or(sec.end, line_of);
                let mut items = Vec::new();
                for line in *start..stop.min(self.lines.len() + 1) {
                    let text = self.lines[line - 1].trim_end_matches('\r');
                    let Some(item) = text.strip_prefix("- ").or_else(|| text.strip_prefix("* "))
                    else {
                        continue;
                    };
                    let item = strip_ticks(item);
                    if item.is_empty() {
                        continue;
                    }
                    if let Some(value) = self.type_scalar(*scalar, item, sec, line, &mut state) {
                        items.push(value);
                    }
                }
                self.read(sec, &keys, state, vec![Some(Value::Array(items))])
            }
            Node::Object(columns) | Node::ObjectArray(columns) => {
                let object = matches!(property.node, Node::Object(_));
                let targets = self.targets(name, columns);
                self.read_tables(sec, name, &blocks, &targets, object, state)
            }
            Node::TypeRefArray => self.read_associated_types(sec, name, &blocks, state),
            Node::Unreadable => self.read(sec, &keys, state, vec![None]),
        }
    }

    /// The columns a header cell can match for key `name`.
    fn targets(&self, name: &str, columns: &[Column]) -> Vec<HeaderTarget> {
        let mut targets: Vec<HeaderTarget> = columns
            .iter()
            .map(|c| HeaderTarget {
                norm: normalize(&c.name),
                path: vec![c.name.clone()],
                kind: c.scalar.map_or(Kind::Nested, Kind::Scalar),
            })
            .collect();
        if self.ctx.module.package == ARCHITECTURE_PACKAGE && name == "thresholds" {
            for column in columns.iter().filter(|c| c.name == "on_exceeded") {
                for nested in &column.nested {
                    if targets.iter().any(|t| t.norm == normalize(&nested.name)) {
                        continue;
                    }
                    targets.push(HeaderTarget {
                        norm: normalize(&nested.name),
                        path: vec![column.name.clone(), nested.name.clone()],
                        kind: nested.scalar.map_or(Kind::Nested, Kind::Scalar),
                    });
                }
            }
        }
        targets
    }

    /// The target index of each header cell; `None` after a header error.
    fn match_headers(
        &self,
        sec: Sec<'_>,
        table: &Table,
        targets: &[HeaderTarget],
        state: &mut State,
    ) -> Option<Vec<usize>> {
        let mut matched: Vec<usize> = Vec::new();
        let mut failed = false;
        for header in &table.headers {
            let norm = normalize(strip_ticks(header));
            match targets.iter().position(|t| t.norm == norm) {
                None => {
                    failed = true;
                    state.diagnostics.push(self.finding(
                        "semantic.invalid-model-cell",
                        "column-unknown",
                        sec,
                        table.line,
                        format!("header `{header}` matches no property of the record key"),
                    ));
                }
                Some(index) if matched.contains(&index) => {
                    failed = true;
                    state.diagnostics.push(self.finding(
                        "semantic.invalid-model-cell",
                        "column-duplicate",
                        sec,
                        table.line,
                        format!("header `{header}` matches a property an earlier header matched"),
                    ));
                }
                Some(index) => matched.push(index),
            }
        }
        (!failed).then_some(matched)
    }

    fn read_tables(
        &self,
        sec: Sec<'_>,
        name: &str,
        blocks: &[Block],
        targets: &[HeaderTarget],
        object: bool,
        mut state: State,
    ) -> Read {
        let keys = [name];
        if blocks.is_empty() {
            return self.no_block(sec, &keys, state);
        }
        let mut first: Option<&Table> = None;
        for (i, block) in blocks.iter().enumerate() {
            match block {
                Block::Table(table) if i == 0 => first = Some(table),
                Block::Table(Table { line, .. })
                | Block::Fence(Fence {
                    open_line: line, ..
                })
                | Block::List { line } => self.not_of_form(sec, name, *line, &mut state),
            }
        }
        let Some(table) = first else {
            return self.read(sec, &keys, state, vec![None]);
        };
        let Some(matched) = self.match_headers(sec, table, targets, &mut state) else {
            return self.read(sec, &keys, state, vec![None]);
        };
        if object && table.rows.is_empty() {
            state.diagnostics.push(self.finding(
                "semantic.invalid-model-cell",
                "no-row",
                sec,
                table.line,
                "an object-valued key needs one data row".to_string(),
            ));
        }
        if object {
            if let Some((line, _)) = table.rows.get(1) {
                state.diagnostics.push(self.finding(
                    "semantic.invalid-model-cell",
                    "second-row",
                    sec,
                    *line,
                    "an object-valued key takes one data row".to_string(),
                ));
            }
        }
        let rows = if object {
            &table.rows[..table.rows.len().min(1)]
        } else {
            &table.rows[..]
        };
        let mut elements = Vec::new();
        for (line, cells) in rows {
            elements.push(Value::Object(
                self.read_row(sec, name, *line, cells, targets, &matched, &mut state),
            ));
        }
        let value = if object {
            elements.into_iter().next()
        } else {
            Some(Value::Array(elements))
        };
        self.read(sec, &keys, state, vec![value])
    }

    #[allow(clippy::too_many_arguments)]
    fn read_row(
        &self,
        sec: Sec<'_>,
        name: &str,
        line: usize,
        cells: &[String],
        targets: &[HeaderTarget],
        matched: &[usize],
        state: &mut State,
    ) -> Map<String, Value> {
        let mut row = Map::new();
        let before = state.diagnostics.len();
        for (position, index) in matched.iter().enumerate() {
            let target = &targets[*index];
            let cell = strip_ticks(cells.get(position).map_or("", String::as_str));
            if cell.is_empty() {
                continue;
            }
            let value = match target.kind {
                Kind::Nested => {
                    state.diagnostics.push(self.finding(
                        "semantic.invalid-model-cell",
                        "nested-object-cell",
                        sec,
                        line,
                        format!(
                            "`{}` is an object or an array of objects",
                            target.path.join(".")
                        ),
                    ));
                    continue;
                }
                Kind::Scalar(scalar) => match self.type_scalar(scalar, cell, sec, line, state) {
                    Some(value) => value,
                    None => continue,
                },
            };
            match target.path.as_slice() {
                [key] => {
                    row.insert(key.clone(), value);
                }
                [parent, key] => {
                    if let Value::Object(nested) = row
                        .entry(parent.clone())
                        .or_insert_with(|| Value::Object(Map::new()))
                    {
                        nested.insert(key.clone(), value);
                    }
                }
                _ => {}
            }
        }
        if self.ctx.module.package == ARCHITECTURE_PACKAGE {
            let clean = state.diagnostics.len() == before;
            self.row_rules(sec, name, line, &row, clean, state);
        }
        row
    }

    /// The reader rules of quoin FR-122 over one row.
    fn row_rules(
        &self,
        sec: Sec<'_>,
        name: &str,
        line: usize,
        row: &Map<String, Value>,
        clean: bool,
        state: &mut State,
    ) {
        match name {
            "routes" => {
                let operation = row.get("operation").and_then(Value::as_str);
                if operation.is_some_and(|op| !self.operations.iter().any(|o| o == op)) {
                    state.diagnostics.push(self.finding(
                        "semantic.invalid-model-cell",
                        "route-operation-unknown",
                        sec,
                        line,
                        format!(
                            "operation `{}` is declared by no `### <name>` under `## Operations`",
                            operation.unwrap_or_default()
                        ),
                    ));
                }
            }
            "thresholds" => {
                let status = row.get("on_exceeded").and_then(|o| o.get("status"));
                // A cell that already failed is refused once (first failing check).
                if status.is_none() && clean {
                    state.diagnostics.push(
                        self.finding(
                            "semantic.invalid-model-cell",
                            "status-required",
                            sec,
                            line,
                            "a threshold needs the response `Status` when its limit is exceeded"
                                .to_string(),
                        ),
                    );
                }
            }
            _ => {}
        }
    }

    /// Type one cell or list item as `scalar`; `None` after a diagnostic.
    fn type_scalar(
        &self,
        scalar: Scalar,
        text: &str,
        sec: Sec<'_>,
        line: usize,
        state: &mut State,
    ) -> Option<Value> {
        let cell_type = |state: &mut State, expected: &str| {
            state.diagnostics.push(self.finding(
                "semantic.invalid-model-cell",
                "cell-type",
                sec,
                line,
                format!("`{text}` is not {expected}"),
            ));
            None
        };
        match scalar {
            Scalar::Text => Some(Value::String(text.to_string())),
            Scalar::Integer => match decimal_integer(text) {
                Some(n) => Some(Value::from(n)),
                None => cell_type(state, "an integer literal"),
            },
            Scalar::IntegerOrText => Some(
                decimal_integer(text).map_or_else(|| Value::String(text.to_string()), Value::from),
            ),
            Scalar::Number => match decimal_integer(text) {
                Some(n) => Some(Value::from(n)),
                None => match text.parse::<f64>().ok().and_then(Number::from_f64) {
                    Some(n) if text.chars().all(|c| "+-.eE0123456789".contains(c)) => {
                        Some(Value::Number(n))
                    }
                    _ => cell_type(state, "a number literal"),
                },
            },
            Scalar::Boolean => match text {
                "true" => Some(Value::Bool(true)),
                "false" => Some(Value::Bool(false)),
                _ => cell_type(state, "`true` or `false`"),
            },
            Scalar::SemanticId => self.resolve_id(text, sec, line, state),
        }
    }

    /// A `SemanticId` cell by the FR-076 `Target` rules (quoin FR-104), with
    /// no `allowed_links` check: no verb names one.
    fn resolve_id(
        &self,
        text: &str,
        sec: Sec<'_>,
        line: usize,
        state: &mut State,
    ) -> Option<Value> {
        let Some(package) = self.package else {
            if !state.no_package {
                state.no_package = true;
                state.advisories.push(Self::severity(
                    self.finding(
                        "semantic.unresolved-target",
                        "no-bundle-package",
                        sec,
                        line,
                        format!(
                            "target `{text}` is not resolved: this surface supplies no bundle package"
                        ),
                    ),
                    SemanticSeverity::Advisory,
                ));
            }
            return None;
        };
        let own = OwnArtifact {
            id: self.own_id.as_deref(),
            object: self.own_object.as_deref(),
            operations: &[],
        };
        match resolve_target(self.ctx, package, &own, text, is_id_segment) {
            Ok(target) => {
                if matches!(target, Target::Unchecked { .. }) {
                    state.advisories.push(Self::severity(
                        self.finding(
                            "semantic.unresolved-target",
                            "no-bundle-index",
                            sec,
                            line,
                            format!(
                                "target `{text}` is not checked against the bundle: this surface supplies no bundle index"
                            ),
                        ),
                        SemanticSeverity::Advisory,
                    ));
                }
                Some(Value::String(target.identity().to_string()))
            }
            Err(_) => {
                state.diagnostics.push(self.finding(
                    "semantic.invalid-model-cell",
                    "target-not-id",
                    sec,
                    line,
                    format!(
                        "target `{text}` is not a bundle artifact id, an `ix://<org>/<repo>/<id>` identity of a bundle artifact, or an identity in an imported package"
                    ),
                ));
                None
            }
        }
    }

    /// `associated_types`: a `Type | Multiplicity` table, one `TypeRef` per
    /// row by the FR-070 cell grammars (quoin FR-122).
    fn read_associated_types(
        &self,
        sec: Sec<'_>,
        name: &str,
        blocks: &[Block],
        mut state: State,
    ) -> Read {
        let keys = [name];
        let targets = [("Type", 0), ("Multiplicity", 1)].map(|(header, i)| HeaderTarget {
            norm: normalize(header),
            path: vec![i.to_string()],
            kind: Kind::Scalar(Scalar::Text),
        });
        if blocks.is_empty() {
            return self.no_block(sec, &keys, state);
        }
        let mut first: Option<&Table> = None;
        for (i, block) in blocks.iter().enumerate() {
            match block {
                Block::Table(table) if i == 0 => first = Some(table),
                Block::Table(Table { line, .. })
                | Block::Fence(Fence {
                    open_line: line, ..
                })
                | Block::List { line } => self.not_of_form(sec, name, *line, &mut state),
            }
        }
        let Some(table) = first else {
            return self.read(sec, &keys, state, vec![None]);
        };
        let Some(matched) = self.match_headers(sec, table, &targets, &mut state) else {
            return self.read(sec, &keys, state, vec![None]);
        };
        let mut elements = Vec::new();
        for (line, cells) in &table.rows {
            let cell = |column: usize| {
                matched
                    .iter()
                    .position(|index| *index == column)
                    .and_then(|position| cells.get(position))
                    .map_or("", |c| strip_ticks(c))
            };
            let row = RowInput {
                line: *line,
                name: String::new(),
                type_cell: cell(0).to_string(),
                mult_cell: cell(1).to_string(),
                constraints_cell: String::new(),
                reference_only: false,
            };
            let mut found = Vec::new();
            let (type_ref, _) = map_type(&row.type_cell, &row, self.ctx, &mut found);
            let multiplicity = map_multiplicity(&row.mult_cell, *line, &mut found);
            state.diagnostics.extend(found);
            if let (Some(mut type_ref), Some(multiplicity)) = (type_ref, multiplicity) {
                type_ref.multiplicity = Some(multiplicity);
                if let Ok(value) = serde_json::to_value(&type_ref) {
                    elements.push(value);
                }
            }
        }
        self.read(sec, &keys, state, vec![Some(Value::Array(elements))])
    }

    /// `records` and `endianness` from the YAML fence under `## Layout`
    /// (quoin FR-122).
    fn read_layout(&self, sec: Sec<'_>) -> Read {
        let mut state = State::default();
        let blocks = blocks_in(self.lines, sec.heading + 1, sec.end);
        if blocks.is_empty() {
            return self.no_block(sec, &FENCE_KEYS, state);
        }
        let mut fence: Option<&Fence> = None;
        for (i, block) in blocks.iter().enumerate() {
            match block {
                Block::Fence(f) if i == 0 && matches!(f.language.as_str(), "yaml" | "yml") => {
                    fence = Some(f);
                }
                Block::Table(Table { line, .. })
                | Block::Fence(Fence {
                    open_line: line, ..
                })
                | Block::List { line } => self.not_of_form(sec, "layout", *line, &mut state),
            }
        }
        let none = vec![None, None];
        let Some(fence) = fence else {
            return self.read(sec, &FENCE_KEYS, state, none);
        };
        let (endianness, records) = match parse_layout(&fence.body) {
            Ok(parsed) => parsed,
            Err(why) => {
                state.diagnostics.push(self.finding(
                    "semantic.feature-not-extractable",
                    "layout",
                    sec,
                    fence.open_line,
                    format!("the `## Layout` fence is not a YAML mapping of the layout: {why}"),
                ));
                return self.read(sec, &FENCE_KEYS, state, none);
            }
        };
        if let Some(records) = &records {
            self.check_sizes(sec, fence.open_line, records, &mut state);
        }
        self.read(
            sec,
            &FENCE_KEYS,
            state,
            vec![records.map(Value::Array), endianness],
        )
    }

    /// A string `size` names a field of exactly one record of the fence.
    fn check_sizes(&self, sec: Sec<'_>, line: usize, records: &[Value], state: &mut State) {
        let field_names = |record: &Value| -> BTreeSet<String> {
            record["fields"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|f| f["name"].as_str().map(str::to_string))
                .collect()
        };
        let names: Vec<BTreeSet<String>> = records.iter().map(field_names).collect();
        for record in records {
            for field in record["fields"].as_array().into_iter().flatten() {
                let Some(size) = field["size"].as_str() else {
                    continue;
                };
                let owners = names.iter().filter(|n| n.contains(size)).count();
                let reason = match owners {
                    1 => continue,
                    0 => "size-unknown-field",
                    _ => "size-ambiguous",
                };
                state.diagnostics.push(self.finding(
                    "semantic.invalid-model-cell",
                    reason,
                    sec,
                    line,
                    format!(
                        "size `{size}` of field `{}` names a field of {owners} records; it must name exactly one",
                        field["name"].as_str().unwrap_or_default()
                    ),
                ));
            }
        }
    }
}

/// The span from the heading to the section's last non-blank line.
fn section_span(ctx: &SemanticContext, lines: &[&str], sec: Sec<'_>) -> SourceLocus {
    let last = (sec.heading..sec.end.min(lines.len() + 1))
        .rev()
        .find(|line| !lines[line - 1].trim().is_empty())
        .unwrap_or(sec.heading);
    let mut span = line_span(ctx, lines, sec.heading);
    let tail = line_span(ctx, lines, last);
    span.end_line = tail.end_line;
    span.end_column = tail.end_column;
    span
}

/// A decimal integer literal: an optional `-`, then digits.
fn decimal_integer(text: &str) -> Option<i64> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// The `## Layout` fence as authored. A `tag` deserializes as a string, so a
/// plain scalar keeps its authored text in block and flow style alike
/// (`0x21` is not the integer 33).
#[derive(Deserialize)]
struct LayoutDoc {
    endianness: Option<Value>,
    record_types: Option<Vec<RecordDoc>>,
}

#[derive(Deserialize)]
struct RecordDoc {
    name: Option<Value>,
    tag: Option<String>,
    size: Option<Value>,
    fields: Option<Vec<Map<String, Value>>>,
}

/// `endianness` and `record_types[]` lowered to `RecordLayout[]`, each
/// `fields[]` entry to a `LayoutField`; keys with no counterpart (`format`,
/// `magic`) stay out.
fn parse_layout(body: &str) -> Result<(Option<Value>, Option<Vec<Value>>), String> {
    let doc: LayoutDoc = serde_yaml::from_str(body).map_err(|e| e.to_string())?;
    let records = doc.record_types.map(|types| {
        types
            .into_iter()
            .map(|record| {
                let mut out = Map::new();
                let scalars = [
                    ("name", record.name),
                    ("tag", record.tag.map(Value::String)),
                    ("size", record.size),
                ];
                out.extend(
                    scalars
                        .into_iter()
                        .filter_map(|(key, value)| Some((key.to_string(), value?))),
                );
                if let Some(fields) = record.fields {
                    let fields = fields.into_iter().map(|field| {
                        Value::Object(
                            ["name", "offset", "size", "type"]
                                .iter()
                                .filter_map(|k| field.get(*k).map(|v| (k.to_string(), v.clone())))
                                .collect(),
                        )
                    });
                    out.insert("fields".to_string(), Value::Array(fields.collect()));
                }
                Value::Object(out)
            })
            .collect()
    });
    Ok((doc.endianness, records))
}

/// A model entry as a record element: the entry without its `sourceSpan`
/// (each key's span rides in `recordSources`) and without the carriers the
/// record schema has no field for.
fn record_element(entry: &impl Serialize, drop: &[&str]) -> Value {
    let mut value = serde_json::to_value(entry).unwrap_or(Value::Null);
    if let Value::Object(map) = &mut value {
        map.remove("sourceSpan");
        for key in drop {
            map.remove(*key);
        }
    }
    value
}

/// What copying one model key produced.
struct Copied {
    elements: Vec<Value>,
    /// The span of the entries, used when the key's section is not found.
    entries: Vec<SourceLocus>,
    diagnostics: Vec<SemanticDiagnostic>,
    /// Why the bundle cannot resolve the key's references at all.
    unresolved: Option<&'static str>,
}

impl Copied {
    fn of<T: Serialize>(
        entries: &[T],
        span: impl Fn(&T) -> SourceLocus,
        drop: &[&str],
        lower: impl Fn(Value, &T) -> Value,
    ) -> Self {
        Self {
            elements: entries
                .iter()
                .map(|e| lower(record_element(e, drop), e))
                .collect(),
            entries: entries.iter().map(span).collect(),
            diagnostics: Vec::new(),
            unresolved: None,
        }
    }
}

/// The copy of model key `key`, when the model holds it.
fn model_elements(
    raw: &str,
    ctx: &SemanticContext,
    model: &ModelDeclarations,
    key: &str,
) -> Option<Copied> {
    match key {
        "values" => Some(Copied::of(
            model.values.as_deref()?,
            |e| e.source_span.clone(),
            &[],
            |v, _| v,
        )),
        "states" => Some(Copied::of(
            model.states.as_deref()?,
            |e| e.source_span.clone(),
            &[],
            |v, _| v,
        )),
        "steps" => Some(Copied::of(
            model.steps.as_deref()?,
            |e| e.source_span.clone(),
            &[],
            |v, _| v,
        )),
        "vocabulary" => Some(Copied::of(
            model.vocabulary.as_deref()?,
            |e| e.source_span.clone(),
            &[],
            |v, _| v,
        )),
        "transitions" => Some(Copied::of(
            model.transitions.as_deref()?,
            |e| e.source_span.clone(),
            &["guardExpression"],
            |mut v, e| {
                // A clause-id `Guard` is the semantic-core `ClauseRef`, with
                // no span; `quire` is the only checked clause language.
                if let (Some(id), Value::Object(map)) = (&e.guard, &mut v) {
                    map.insert(
                        "guard".to_string(),
                        json!({ "language": "quire", "clauseId": id }),
                    );
                }
                v
            },
        )),
        "members" => Some(copy_members(raw, ctx, model.members.as_deref()?)),
        _ => None,
    }
}

/// The `Member | Multiplicity` rows as `RelationDecl[]` (quoin FR-123): each
/// `Member` resolves by declaration references admitted by the construct
/// `references.members`, and its `verb` is the one composite verb the object
/// type's `allowed_links` admit for the target's object type or role.
fn copy_members(raw: &str, ctx: &SemanticContext, members: &[MemberDecl]) -> Copied {
    let mut out = Copied::of(members, |m| m.source_span.clone(), &[], |v, _| v);
    out.elements.clear();
    let section = ctx
        .declared_tables
        .section_of("members")
        .unwrap_or("Members");
    let at = |code: &str, reason: Option<&str>, member: &MemberDecl, detail: String| {
        let mut d = SemanticDiagnostic::new(
            code,
            SemanticSeverity::Error,
            member.source_span.start_line,
            format!(
                "artifact {} declares members in `## {section}` at line {}: {detail}",
                ctx.path, member.source_span.start_line
            ),
        );
        d.reason = reason.map(str::to_string);
        d.source_span = Some(member.source_span.clone());
        d.section = Some(section.to_string());
        d
    };
    let Some(package) = bundle_package(ctx) else {
        out.unresolved = Some("no-bundle-package");
        return out;
    };
    if ctx.bundle.artifacts.is_empty() {
        out.unresolved = Some("no-bundle-index");
        return out;
    }
    let Some(vocabulary) = ctx.relation_vocabulary.as_ref() else {
        out.unresolved = Some("no-relation-vocabulary");
        return out;
    };
    let own = OwnDeclaration::from_raw(raw);
    let declarations = Declarations::new(ctx, &own);
    let admission = match ctx
        .construct_references
        .as_ref()
        .and_then(|c| c.roles("members"))
    {
        Some(roles) => Admission::Construct { roles, vocabulary },
        None => Admission::Any,
    };
    for member in members {
        let item = member.target.as_str();
        let identity = match resolve_reference(ctx, package, &own, &declarations, item, admission) {
            Ok(identity) => identity,
            Err(refusal) => {
                let (code, reason, detail) = match refusal {
                    ReferenceRefusal::UnknownName => (
                        "semantic.unknown-reference",
                        Some("unknown-name"),
                        format!("Member item {item:?} names no declaration of the bundle"),
                    ),
                    ReferenceRefusal::KindMismatch => (
                        "semantic.reference-kind-mismatch",
                        None,
                        format!("Member item {item:?} names no declaration the members reference admits"),
                    ),
                    ReferenceRefusal::Ambiguous(ids) => (
                        "semantic.ambiguous-reference",
                        None,
                        format!("Member item {item:?} names more than one declaration: {}", ids.join(", ")),
                    ),
                };
                out.diagnostics.push(at(code, reason, member, detail));
                continue;
            }
        };
        let id = identity.rsplit('/').next().unwrap_or_default();
        let object = if own.id.as_deref() == Some(id) {
            own.object.as_deref()
        } else {
            ctx.bundle
                .artifacts
                .iter()
                .find(|a| a.id == id)
                .and_then(|a| a.object.as_deref())
        };
        let verbs = vocabulary.composite_verbs_admitting(object);
        let [(verb, entry)] = verbs.as_slice() else {
            out.diagnostics.push(at(
                "semantic.invalid-model-cell",
                Some("member-verb"),
                member,
                format!(
                    "{} composite verbs admit {item:?}; exactly one must",
                    verbs.len()
                ),
            ));
            continue;
        };
        let decl = RelationDecl {
            verb: (*verb).to_string(),
            category: entry.category,
            composite: entry.composite,
            target: identity,
            multiplicity: member.multiplicity.clone(),
        };
        out.elements
            .push(serde_json::to_value(decl).unwrap_or(Value::Null));
    }
    out
}

/// Copy the FR-075 `model` entries to the record keys of the same name, only
/// where the record schema declares the key and a table locator fills it
/// (quoin FR-123). Each key's span is its whole section; `recordSources`
/// stays in document order.
pub(crate) fn copy_model_keys(
    raw: &str,
    outcome: &mut RecordOutcome,
    ctx: &SemanticContext,
    model: Option<&ModelDeclarations>,
    model_state: Option<&KindAvailability>,
) {
    let Some(schema) = ctx.record_schema.as_ref() else {
        return;
    };
    let all_lines = lines(raw);
    let headings = level2_headings(&all_lines);
    for property in &schema.properties {
        let name = property.name.as_str();
        if schema.owner(ctx, property) != Some(KeyOwner::ModelTable) {
            continue;
        }
        let state = match model.and_then(|m| model_elements(raw, ctx, m, name)) {
            Some(copied) if copied.unresolved.is_some() => {
                KindAvailability::unavailable(copied.unresolved.unwrap_or_default())
            }
            Some(copied) if copied.diagnostics.iter().any(SemanticDiagnostic::is_error) => {
                let state = entry_errors(&copied.diagnostics);
                outcome.diagnostics.extend(copied.diagnostics);
                state
            }
            Some(copied) => {
                let section = ctx.declared_tables.section_of(name).and_then(|heading| {
                    headings
                        .iter()
                        .find(|(text, _, _)| text == heading)
                        .map(|(text, line, end)| {
                            let sec = Sec {
                                text,
                                heading: *line,
                                end: *end,
                            };
                            section_span(ctx, &all_lines, sec)
                        })
                });
                let entries = match (copied.entries.first(), copied.entries.last()) {
                    (Some(first), Some(last)) => {
                        let mut span = first.clone();
                        span.end_line = last.end_line;
                        span.end_column = last.end_column;
                        Some(span)
                    }
                    _ => None,
                };
                outcome
                    .sources
                    .extend(section.or(entries).map(|source_span| RecordSource {
                        key: name.to_string(),
                        source_span,
                    }));
                outcome
                    .record
                    .insert(name.to_string(), Value::Array(copied.elements));
                KindAvailability::available(false)
            }
            None => match model_state {
                Some(state) if state.state == AvailabilityState::Unavailable => state.clone(),
                _ => KindAvailability::not_applicable(),
            },
        };
        outcome.availability.insert(name.to_string(), state);
    }
    outcome.sources.sort_by_key(|s| s.source_span.start_line);
}
