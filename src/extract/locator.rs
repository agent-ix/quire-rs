//! The six body-extraction Locator primitives (FR-011).
//!
//! Each primitive reads from a [`QuireDocument`] via the Query API
//! (Task 003) and returns either a single string, a sequence of
//! strings, or — for the structural primitives like `table_row` —
//! a sequence of `serde_json::Value`s. The single-yield evaluator
//! collapses sequences to their first element when the locator is
//! bound under a `match:` key; the multi-yield evaluator iterates.

use std::collections::BTreeMap;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ast::QuireDocument;
use crate::query::{
    diagrams_from_content_with_lines, parse_bullet_list_with_lines, parse_tables_with_lines,
    section as q_section, sections as q_sections, ListPattern, TableResult,
};

/// A DSL Locator value — either a single [`LocatorPrimitive`] or a
/// fallback chain. YAML accepts both shapes:
///
/// ```yaml
/// # single primitive
/// from: frontmatter_field
/// path: [id]
/// # fallback chain
/// - from: frontmatter_field
///   path: [id]
/// - from: heading
///   level: 1
/// ```
///
/// Untagged deserialization picks the variant by shape (object vs.
/// list).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Locator {
    /// One primitive — the canonical case.
    Primitive(LocatorPrimitive),
    /// Ordered chain — first non-empty result wins (FR-016).
    Fallback(Vec<LocatorPrimitive>),
}

impl Locator {
    /// Returns the canonical primitive (first in a fallback chain) for
    /// error messages.
    pub fn canonical(&self) -> &LocatorPrimitive {
        match self {
            Self::Primitive(p) => p,
            Self::Fallback(v) => v
                .first()
                .expect("fallback chain must have at least one primitive"),
        }
    }

    /// Whether the canonical position declares `required: true`.
    pub fn required(&self) -> bool {
        self.canonical().required()
    }

    /// Whether the primitive at `position` (the one whose values were
    /// used — see [`eval_locator`]) declares `multiple: true`. For a
    /// bare `Primitive`, `position` is ignored.
    pub fn multiple_at(&self, position: usize) -> bool {
        match self {
            Self::Primitive(p) => p.multiple(),
            Self::Fallback(chain) => chain.get(position).map(|p| p.multiple()).unwrap_or(false),
        }
    }
}

/// One Locator from a `body_extraction` DSL. Untagged on the YAML
/// side: the discriminant is the `from:` field, captured here as the
/// enum variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "snake_case")]
pub enum LocatorPrimitive {
    /// `from: frontmatter_field` — `doc.frontmatter[path[0]][path[1]]...`
    FrontmatterField {
        path: Vec<String>,
        #[serde(default = "default_true")]
        required: bool,
        #[serde(default)]
        multiple: bool,
        #[serde(default)]
        regex: Option<String>,
        #[serde(default)]
        assert: Option<Box<LocatorAssert>>,
    },
    /// `from: section_body` — text content of `after_heading` section.
    SectionBody {
        after_heading: String,
        #[serde(default = "default_true")]
        required: bool,
        #[serde(default)]
        multiple: bool,
        #[serde(default)]
        regex: Option<String>,
        #[serde(default)]
        assert: Option<Box<LocatorAssert>>,
    },
    /// `from: code_block` — source of the first fenced code block in
    /// `language`, optionally constrained to a section.
    CodeBlock {
        #[serde(default)]
        language: Option<String>,
        #[serde(default)]
        #[serde(alias = "after_heading")]
        under_section: Option<String>,
        #[serde(default = "default_true")]
        required: bool,
        #[serde(default)]
        multiple: bool,
        #[serde(default)]
        regex: Option<String>,
        #[serde(default)]
        assert: Option<Box<LocatorAssert>>,
    },
    /// `from: table_row` — rows from a table (optionally inside a
    /// section, optionally projecting a single column).
    TableRow {
        #[serde(default)]
        #[serde(alias = "after_heading")]
        under_section: Option<String>,
        #[serde(default)]
        column: Option<ColumnRef>,
        #[serde(default = "default_true")]
        required: bool,
        #[serde(default)]
        multiple: bool,
        #[serde(default)]
        regex: Option<String>,
        #[serde(default)]
        assert: Option<Box<LocatorAssert>>,
    },
    /// `from: list_item` — items from a bullet list.
    ListItem {
        #[serde(default)]
        #[serde(alias = "after_heading")]
        under_section: Option<String>,
        #[serde(default)]
        pattern: Option<ListPattern>,
        #[serde(default = "default_true")]
        required: bool,
        #[serde(default)]
        multiple: bool,
        #[serde(default)]
        regex: Option<String>,
        #[serde(default)]
        assert: Option<Box<LocatorAssert>>,
    },
    /// `from: heading` — heading text of sections at `level`, or under
    /// `path`.
    Heading {
        #[serde(default)]
        #[serde(alias = "depth")]
        level: Option<u8>,
        #[serde(default)]
        path: Option<Vec<String>>,
        #[serde(default = "default_true")]
        required: bool,
        #[serde(default)]
        multiple: bool,
        #[serde(default)]
        regex: Option<String>,
        #[serde(default)]
        assert: Option<Box<LocatorAssert>>,
    },
}

/// The locator kind discriminator (the `from:` value), used by
/// load-time assert validation to decide which assert keys make sense.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocatorKind {
    FrontmatterField,
    SectionBody,
    CodeBlock,
    TableRow,
    ListItem,
    Heading,
}

impl LocatorKind {
    /// Stable snake_case name matching the `from:` value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FrontmatterField => "frontmatter_field",
            Self::SectionBody => "section_body",
            Self::CodeBlock => "code_block",
            Self::TableRow => "table_row",
            Self::ListItem => "list_item",
            Self::Heading => "heading",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColumnRef {
    Name(String),
    Index(usize),
}

/// Optional `assert:` facet on a body-extraction locator (FR-033).
///
/// Evaluated by `validate_document` (FR-032) in an asserting posture and
/// **ignored by extraction** (FR-011). All keys are optional;
/// `deny_unknown_fields` rejects typos at load time. Which keys are
/// sensible depends on the locator kind — see
/// [`crate::extract::dsl::validate_assert_for_kind`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocatorAssert {
    /// Required heading level of the located section.
    #[serde(default)]
    pub level: Option<u8>,
    /// Ordered list of exact table column headers (case-sensitive).
    #[serde(default)]
    pub columns: Option<Vec<String>>,
    /// Subset of `columns` a document MAY omit (CR-023).
    ///
    /// Without this, `columns` is an exact match, so a contract cannot ask for
    /// a column that much of an existing corpus never authored without either
    /// failing every such document or forcing an invented value into it. The
    /// declared order still holds: the headers must be an ordered subsequence
    /// of `columns` containing every non-optional one.
    #[serde(default)]
    pub optional_columns: Option<Vec<String>>,
    /// Minimum number of data rows in the located table.
    #[serde(default)]
    pub min_rows: Option<usize>,
    /// Maximum number of data rows in the located table. `table_row` only.
    #[serde(default)]
    pub max_rows: Option<usize>,
    /// Minimum number of items in the located list.
    #[serde(default)]
    pub min_items: Option<usize>,
    /// Table column whose cells are treated as ids.
    #[serde(default)]
    pub id_column: Option<String>,
    /// Regex each id cell (or located id) MUST match; supports `{field}`
    /// interpolation (FR-034).
    #[serde(default)]
    pub id_pattern: Option<String>,
    /// Regex the located content MUST match; supports `{field}`
    /// interpolation (FR-034).
    #[serde(default)]
    pub matches: Option<String>,
    /// Allowed values the located **scalar** content MUST be one of — an
    /// enum constraint (CR-010, FR-033-AC-11). Legal on scalar content
    /// locators (`section_body`/`heading`/`list_item`/`frontmatter_field`),
    /// illegal on `table_row`/`code_block`.
    #[serde(default)]
    pub choices: Option<Vec<String>>,
    /// Per-column allowed value sets for a table (header → values); every
    /// data cell in that column MUST be one of the values. `table_row`
    /// only (CR-010, FR-033-AC-12).
    #[serde(default)]
    pub column_choices: Option<BTreeMap<String, Vec<String>>>,
    /// Per-column regex for a table (header → pattern, `{field}`-interpolated);
    /// every data cell in that column MUST match. `table_row` only
    /// (CR-010, FR-033-AC-13).
    #[serde(default)]
    pub column_patterns: Option<BTreeMap<String, String>>,
    /// Name of a declared vocabulary whose values are the allowed scalar
    /// content — the dereferencing counterpart to [`choices`](Self::choices)
    /// (FR-060).
    ///
    /// A contract that writes the values out has a second copy of a list the
    /// traceability model already declares, and the two are then kept in
    /// agreement only by someone remembering. Naming the vocabulary removes the
    /// copy rather than testing it.
    ///
    /// Resolved at registry construction, **after** the cross-module merge: the
    /// vocabulary a contract names may be declared by a different module than
    /// the archetype naming it.
    #[serde(default)]
    pub from_vocabulary: Option<String>,
    /// Per-column named vocabularies for a table (header → vocabulary name) —
    /// the dereferencing counterpart to
    /// [`column_choices`](Self::column_choices) (FR-060). `table_row` only.
    #[serde(default)]
    pub column_vocabularies: Option<BTreeMap<String, String>>,
}

impl LocatorAssert {
    /// `true` if no assert key is set (an empty facet is a no-op).
    pub fn is_empty(&self) -> bool {
        self.level.is_none()
            && self.columns.is_none()
            && self.min_rows.is_none()
            && self.max_rows.is_none()
            && self.min_items.is_none()
            && self.id_column.is_none()
            && self.id_pattern.is_none()
            && self.matches.is_none()
            && self.choices.is_none()
            && self.column_choices.is_none()
            && self.from_vocabulary.is_none()
            && self.column_vocabularies.is_none()
            && self.column_patterns.is_none()
    }
}

fn default_true() -> bool {
    true
}

impl LocatorPrimitive {
    /// Short human-readable description used in error messages.
    pub fn describe(&self) -> String {
        match self {
            Self::FrontmatterField { path, .. } => {
                format!("frontmatter_field({})", path.join("."))
            }
            Self::SectionBody { after_heading, .. } => {
                format!("section_body({after_heading})")
            }
            Self::CodeBlock {
                language,
                under_section,
                ..
            } => match under_section {
                Some(s) => match language {
                    Some(language) => format!("code_block({language} under {s})"),
                    None => format!("code_block(any under {s})"),
                },
                None => match language {
                    Some(language) => format!("code_block({language})"),
                    None => "code_block(any)".to_string(),
                },
            },
            Self::TableRow { under_section, .. } => match under_section {
                Some(s) => format!("table_row(under {s})"),
                None => "table_row".to_string(),
            },
            Self::ListItem { under_section, .. } => match under_section {
                Some(s) => format!("list_item(under {s})"),
                None => "list_item".to_string(),
            },
            Self::Heading { level, path, .. } => {
                format!("heading(level={level:?}, path={path:?})")
            }
        }
    }

    /// Whether the locator declared `required: true`.
    pub fn required(&self) -> bool {
        match self {
            Self::FrontmatterField { required, .. }
            | Self::SectionBody { required, .. }
            | Self::CodeBlock { required, .. }
            | Self::TableRow { required, .. }
            | Self::ListItem { required, .. }
            | Self::Heading { required, .. } => *required,
        }
    }

    /// Whether the locator declared `multiple: true` — the yielded
    /// record keeps EVERY located value as a JSON array instead of
    /// collapsing to the first (FR-011-AC-21).
    pub fn multiple(&self) -> bool {
        match self {
            Self::FrontmatterField { multiple, .. }
            | Self::SectionBody { multiple, .. }
            | Self::CodeBlock { multiple, .. }
            | Self::TableRow { multiple, .. }
            | Self::ListItem { multiple, .. }
            | Self::Heading { multiple, .. } => *multiple,
        }
    }

    /// The locator's kind discriminator.
    pub fn kind(&self) -> LocatorKind {
        match self {
            Self::FrontmatterField { .. } => LocatorKind::FrontmatterField,
            Self::SectionBody { .. } => LocatorKind::SectionBody,
            Self::CodeBlock { .. } => LocatorKind::CodeBlock,
            Self::TableRow { .. } => LocatorKind::TableRow,
            Self::ListItem { .. } => LocatorKind::ListItem,
            Self::Heading { .. } => LocatorKind::Heading,
        }
    }

    /// Mutable access to the optional `assert:` facet (FR-060).
    ///
    /// Exists so `Registry::from_shape` can dereference a named vocabulary
    /// into the literal `choices` the evaluator already understands, once,
    /// after the cross-module merge. The evaluator itself never sees a
    /// vocabulary name — it is resolved before any document is validated, so
    /// the hot path is unchanged.
    pub(crate) fn assert_mut(&mut self) -> Option<&mut LocatorAssert> {
        match self {
            Self::FrontmatterField { assert, .. }
            | Self::SectionBody { assert, .. }
            | Self::CodeBlock { assert, .. }
            | Self::TableRow { assert, .. }
            | Self::ListItem { assert, .. }
            | Self::Heading { assert, .. } => assert.as_deref_mut(),
        }
    }

    /// The optional `assert:` facet, if declared (FR-033).
    pub fn assert(&self) -> Option<&LocatorAssert> {
        match self {
            Self::FrontmatterField { assert, .. }
            | Self::SectionBody { assert, .. }
            | Self::CodeBlock { assert, .. }
            | Self::TableRow { assert, .. }
            | Self::ListItem { assert, .. }
            | Self::Heading { assert, .. } => assert.as_deref(),
        }
    }
}

/// Where a located value sits in its document (FR-033-AC-18), before the
/// document-line conversion that needs the frontmatter's extent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    /// 0-based line of the value's source within the document body.
    Body(usize),
    /// The value is a frontmatter field: its line is the last path key's.
    Frontmatter,
}

/// A located value with its position. The one internal currency of
/// evaluation: [`eval`] drops the position, [`eval_located`] resolves it.
#[derive(Debug, Clone, PartialEq)]
struct Located {
    value: Value,
    at: At,
}

/// A value a locator located, with the 1-based document line (frontmatter
/// counted) it was read from (FR-033-AC-18). The line travels with the value
/// through `regex` filtering, a missing `column` cell and a fallback chain.
#[derive(Debug, Clone, PartialEq)]
pub struct LocatedValue {
    pub value: Value,
    /// `None` only when the value has no line to point at.
    pub line: Option<usize>,
}

/// Implements: FR-016
/// Evaluate a [`Locator`] (primitive or fallback chain) against
/// `doc`. For a fallback chain, returns the first primitive that
/// produces a non-empty result and records its position so the caller
/// can emit a `FallbackLocatorUsed` diagnostic when `position > 0`.
///
/// Returns `(values, position_used)` where `position_used == 0` for a
/// `Primitive` or a fallback that hit at the head.
pub fn eval_locator(doc: &QuireDocument, locator: &Locator) -> (Vec<Value>, usize) {
    let (located, position) = eval_locator_at(doc, locator);
    (located.into_iter().map(|l| l.value).collect(), position)
}

/// [`eval_locator`] with each value's 1-based document line (FR-033-AC-18).
/// For a fallback chain the lines are those of the primitive whose values are
/// returned.
pub fn eval_locator_located(doc: &QuireDocument, locator: &Locator) -> (Vec<LocatedValue>, usize) {
    eval_locator_located_at(doc, locator, offset_of(doc))
}

/// The number of document lines before the body (frontmatter, any BOM).
fn offset_of(doc: &QuireDocument) -> usize {
    crate::validate_document::body_line_offset(&doc.raw)
}

/// [`eval_locator_located`] given the document's body-line `offset`, so a
/// caller locating many values parses the frontmatter's extent once.
pub(crate) fn eval_locator_located_at(
    doc: &QuireDocument,
    locator: &Locator,
    offset: usize,
) -> (Vec<LocatedValue>, usize) {
    let (located, position) = eval_locator_at(doc, locator);
    let primitive = match locator {
        Locator::Primitive(p) => p,
        Locator::Fallback(chain) => chain.get(position).unwrap_or_else(|| locator.canonical()),
    };
    (resolve_lines(doc, primitive, located, offset), position)
}

fn eval_locator_at(doc: &QuireDocument, locator: &Locator) -> (Vec<Located>, usize) {
    match locator {
        Locator::Primitive(p) => (eval_at(doc, p), 0),
        Locator::Fallback(chain) => {
            for (i, p) in chain.iter().enumerate() {
                let v = eval_at(doc, p);
                if !v.is_empty() {
                    return (v, i);
                }
            }
            (Vec::new(), 0)
        }
    }
}

/// Evaluate a `LocatorPrimitive` against `doc`. Returns an ordered
/// list of values; the caller picks first-or-all depending on its
/// yield pattern.
pub fn eval(doc: &QuireDocument, loc: &LocatorPrimitive) -> Vec<Value> {
    eval_at(doc, loc).into_iter().map(|l| l.value).collect()
}

/// Every value `loc` locates in `doc`, each with the 1-based document line it
/// was read from (FR-033-AC-18), independent of any `assert`. A locator that
/// located nothing returns an empty list and so exposes no line.
///
/// | locator | line |
/// |---|---|
/// | `frontmatter_field` | the last key of `path`; a list value has the key's line |
/// | `heading` | the heading |
/// | `section_body` | the heading that owns the body |
/// | `code_block` | the opening fence |
/// | `list_item` | the item |
/// | `table_row` | the row's own line |
pub fn eval_located(doc: &QuireDocument, loc: &LocatorPrimitive) -> Vec<LocatedValue> {
    resolve_lines(doc, loc, eval_at(doc, loc), offset_of(doc))
}

fn eval_at(doc: &QuireDocument, loc: &LocatorPrimitive) -> Vec<Located> {
    let (values, regex) = eval_unfiltered(doc, loc);
    apply_regex(values, regex)
}

/// Convert body-relative positions to 1-based document lines.
fn resolve_lines(
    doc: &QuireDocument,
    loc: &LocatorPrimitive,
    located: Vec<Located>,
    offset: usize,
) -> Vec<LocatedValue> {
    if located.is_empty() {
        return Vec::new();
    }
    let frontmatter_line = match loc {
        LocatorPrimitive::FrontmatterField { path, .. } => {
            super::frontmatter_lines::key_line(&doc.raw, path)
        }
        _ => None,
    };
    located
        .into_iter()
        .map(|l| LocatedValue {
            line: match l.at {
                At::Body(line) => Some(offset + line + 1),
                At::Frontmatter => frontmatter_line,
            },
            value: l.value,
        })
        .collect()
}

/// The text of the first value `loc` locates **before** its `regex` filter,
/// when that filter is what leaves the locator with nothing: the value is
/// present and does not match. `None` when the locator has no `regex`, when
/// the regex keeps a value, or when there is no located text.
///
/// Lets `validate_document` tell "the value is absent" from "the value is
/// there and does not match the pattern". Only `frontmatter_field` and
/// `table_row` locators answer: for a heading or a section, a different text
/// is the named heading or section being absent, which is what `missing`
/// says.
pub fn regex_mismatch<'a>(
    doc: &QuireDocument,
    loc: &'a LocatorPrimitive,
) -> Option<(String, &'a str)> {
    if !matches!(
        loc,
        LocatorPrimitive::FrontmatterField { .. } | LocatorPrimitive::TableRow { .. }
    ) {
        return None;
    }
    let (values, regex) = eval_unfiltered(doc, loc);
    let pattern = regex?;
    let located = values.iter().find_map(|l| match &l.value {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    })?;
    apply_regex(values, Some(pattern))
        .is_empty()
        .then_some((located, pattern))
}

fn eval_unfiltered<'a>(
    doc: &QuireDocument,
    loc: &'a LocatorPrimitive,
) -> (Vec<Located>, Option<&'a str>) {
    match loc {
        LocatorPrimitive::FrontmatterField { path, regex, .. } => {
            (eval_frontmatter_field(doc, path), regex.as_deref())
        }
        LocatorPrimitive::SectionBody {
            after_heading,
            regex,
            ..
        } => (eval_section_body(doc, after_heading), regex.as_deref()),
        LocatorPrimitive::CodeBlock {
            language,
            under_section,
            regex,
            ..
        } => (
            eval_code_block(doc, language.as_deref(), under_section.as_deref()),
            regex.as_deref(),
        ),
        LocatorPrimitive::TableRow {
            under_section,
            column,
            regex,
            ..
        } => (
            eval_table_row(doc, under_section.as_deref(), column.as_ref()),
            regex.as_deref(),
        ),
        LocatorPrimitive::ListItem {
            under_section,
            pattern,
            regex,
            ..
        } => (
            eval_list_item(doc, under_section.as_deref(), *pattern),
            regex.as_deref(),
        ),
        LocatorPrimitive::Heading {
            level, path, regex, ..
        } => (eval_heading(doc, *level, path.as_deref()), regex.as_deref()),
    }
}

/// FR-011-AC-17 / FR-032: a resolved value whose trimmed content is a
/// single unresolved `{{ … }}` template marker is a placeholder. On the
/// **extract** path it contributes no value (the key is omitted); at
/// **validate** time it surfaces as reason `placeholder`. A `{{…}}` token
/// embedded inside otherwise substantive content does NOT trigger the
/// rule — only a whole-value marker does. Shared so both postures agree.
pub fn is_whole_value_mustache(s: &str) -> bool {
    let t = s.trim();
    t.starts_with("{{") && t.ends_with("}}") && t.len() >= 4
}

fn apply_regex(values: Vec<Located>, pattern: Option<&str>) -> Vec<Located> {
    let Some(pattern) = pattern else {
        return values;
    };
    let Ok(re) = Regex::new(pattern) else {
        return Vec::new();
    };
    values
        .into_iter()
        .filter_map(|located| match located.value {
            Value::String(s) => re.captures(&s).map(|caps| {
                let matched = caps
                    .get(1)
                    .or_else(|| caps.get(0))
                    .map(|m| m.as_str())
                    .unwrap_or("");
                Located {
                    value: Value::String(matched.to_string()),
                    at: located.at,
                }
            }),
            value => Some(Located {
                value,
                at: located.at,
            }),
        })
        .collect()
}

fn eval_frontmatter_field(doc: &QuireDocument, path: &[String]) -> Vec<Located> {
    let mut cur: Value = match &doc.frontmatter {
        Some(map) => Value::Object(map.clone()),
        None => return Vec::new(),
    };
    for key in path {
        cur = match cur {
            Value::Object(map) => match map.get(key) {
                Some(v) => v.clone(),
                None => return Vec::new(),
            },
            _ => return Vec::new(),
        };
    }
    if cur.is_null() {
        Vec::new()
    } else {
        vec![Located {
            value: cur,
            at: At::Frontmatter,
        }]
    }
}

fn eval_section_body(doc: &QuireDocument, heading: &str) -> Vec<Located> {
    match q_section(doc, heading) {
        Some(s) => vec![Located {
            value: Value::String(s.content.trim().to_string()),
            at: At::Body(s.start_line),
        }],
        None => Vec::new(),
    }
}

/// The content a section-owned locator reads, and where each part of it
/// starts in the document body. A named section is one part; with no section
/// named the top-level sections' contents are joined by `\n`, so a line of the
/// joined text maps back to its own section.
struct Scope {
    content: String,
    /// `(first line of the part within content, body line of the part's first
    /// content line)`, ascending.
    parts: Vec<(usize, usize)>,
}

impl Scope {
    fn of(doc: &QuireDocument, under_section: Option<&str>) -> Option<Self> {
        match under_section {
            Some(h) => {
                let s = q_section(doc, h)?;
                Some(Self {
                    content: s.content.clone(),
                    parts: vec![(0, s.start_line + 1)],
                })
            }
            None => {
                let mut parts = Vec::new();
                let mut next = 0;
                for s in &doc.sections {
                    parts.push((next, s.start_line + 1));
                    next += s.content.matches('\n').count() + 1;
                }
                Some(Self {
                    content: doc
                        .sections
                        .iter()
                        .map(|s| s.content.as_str())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    parts,
                })
            }
        }
    }

    /// The body line of line `rel` of the scope's content.
    fn body_line(&self, rel: usize) -> usize {
        let (first, body) = self
            .parts
            .iter()
            .rev()
            .find(|(first, _)| *first <= rel)
            .copied()
            .unwrap_or((0, 0));
        body + (rel - first)
    }
}

fn eval_code_block(
    doc: &QuireDocument,
    language: Option<&str>,
    under_section: Option<&str>,
) -> Vec<Located> {
    // Section-owned: resolve from the section's content slice (and thus,
    // under multi-yield, the iteration unit's slice), mirroring
    // `eval_list_item`. The document-wide `extract_diagrams` harvest is a
    // separate query, not the locator substrate (FR-011).
    let Some(scope) = Scope::of(doc, under_section) else {
        return Vec::new();
    };
    diagrams_from_content_with_lines(&scope.content, language)
        .into_iter()
        .map(|(fence, block)| Located {
            value: Value::String(block.source),
            at: At::Body(scope.body_line(fence)),
        })
        .collect()
}

/// The first table a table locator reads, with the body line of each data row:
/// the named section's first table, or the first table of the first section
/// that has one.
fn find_table(
    doc: &QuireDocument,
    under_section: Option<&str>,
) -> Option<(TableResult, Vec<usize>)> {
    let pick = |s: &crate::ast::QuireSection| {
        parse_tables_with_lines(&s.content)
            .into_iter()
            .next()
            .map(|(table, row_lines, _)| {
                let body = row_lines.iter().map(|rel| s.start_line + 1 + rel).collect();
                (table, body)
            })
    };
    match under_section {
        Some(h) => q_section(doc, h).and_then(pick),
        None => doc.sections.iter().find_map(pick),
    }
}

fn eval_table_row(
    doc: &QuireDocument,
    under_section: Option<&str>,
    column: Option<&ColumnRef>,
) -> Vec<Located> {
    let Some((table, row_lines)) = find_table(doc, under_section) else {
        return Vec::new();
    };
    let rows = table.rows.into_iter().zip(row_lines);
    if let Some(col) = column {
        let idx = match col {
            ColumnRef::Name(name) => match table.headers.iter().position(|h| h == name) {
                Some(i) => i,
                None => return Vec::new(),
            },
            ColumnRef::Index(n) => match n.checked_sub(1) {
                Some(i) if i < table.headers.len() => i,
                _ => return Vec::new(),
            },
        };
        rows.filter_map(|(r, line)| {
            r.get(idx).cloned().map(|cell| Located {
                value: Value::String(cell),
                at: At::Body(line),
            })
        })
        .collect()
    } else {
        rows.map(|(r, line)| Located {
            value: Value::String(r.join("\t")),
            at: At::Body(line),
        })
        .collect()
    }
}

fn eval_list_item(
    doc: &QuireDocument,
    under_section: Option<&str>,
    pattern: Option<ListPattern>,
) -> Vec<Located> {
    let Some(scope) = Scope::of(doc, under_section) else {
        return Vec::new();
    };
    parse_bullet_list_with_lines(&scope.content, pattern)
        .into_iter()
        .map(|(line, item)| Located {
            value: Value::String(item.raw),
            at: At::Body(scope.body_line(line)),
        })
        .collect()
}

fn eval_heading(doc: &QuireDocument, level: Option<u8>, path: Option<&[String]>) -> Vec<Located> {
    let located = |s: &crate::ast::QuireSection| Located {
        value: Value::String(crate::query::normalize_heading(&s.heading)),
        at: At::Body(s.start_line),
    };
    if let Some(p) = path {
        // Walk by exact heading path.
        let mut current: Option<&crate::ast::QuireSection> = None;
        for (i, name) in p.iter().enumerate() {
            current = if i == 0 {
                doc.sections
                    .iter()
                    .find(|s| s.heading.eq_ignore_ascii_case(name))
            } else {
                current.and_then(|s| {
                    s.children
                        .iter()
                        .find(|c| c.heading.eq_ignore_ascii_case(name))
                })
            };
            if current.is_none() {
                return Vec::new();
            }
        }
        current.map(|s| vec![located(s)]).unwrap_or_default()
    } else {
        q_sections(doc, level).into_iter().map(located).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn doc() -> QuireDocument {
        parse_document(
            "---\nid: FR-001\ntags:\n- a\n- b\n---\n\
             ## Purpose\nthe purpose\n\
             ## API\n| Method | Path |\n| - | - |\n| GET | /a |\n| POST | /b |\n\
             ## Notes\n- **Auth** — token-based\n- plain note\n\
             ## Diagrams\n```mermaid\ngraph TD; A-->B\n```\n",
        )
    }

    #[test]
    fn frontmatter_field_walks_path() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::FrontmatterField {
                path: vec!["id".into()],
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v, vec![json!("FR-001")]);
    }

    #[test]
    fn frontmatter_field_array_value_returned_whole() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::FrontmatterField {
                path: vec!["tags".into()],
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v, vec![json!(["a", "b"])]);
    }

    #[test]
    fn frontmatter_field_missing_path_returns_empty() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::FrontmatterField {
                path: vec!["nope".into()],
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert!(v.is_empty());
    }

    #[test]
    fn section_body_returns_trimmed_content() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::SectionBody {
                after_heading: "Purpose".into(),
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v, vec![json!("the purpose")]);
    }

    #[test]
    fn code_block_returns_source_filtered_by_language() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::CodeBlock {
                language: Some("mermaid".into()),
                under_section: None,
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].as_str().unwrap().contains("A-->B"));
    }

    #[test]
    #[trace("FR-011-AC-13")]
    // `code_block` is section-owned — a single-yield
    // `under:X` never reaches another section's block.
    fn code_block_under_section_is_section_owned_excludes_other_sections() {
        // Two mermaid blocks of the same language in DIFFERENT sections.
        // A `code_block under: Workflow` locator must return ONLY the
        // block contained in `## Workflow` — the `## Other` block (the
        // bug: document-wide harvest filtered by heading) must not leak.
        let d = parse_document(
            "## Workflow\n```mermaid\ngraph TD; WANT-->ME\n```\n\
             ## Other\n```mermaid\ngraph TD; LEAK-->NO\n```\n",
        );
        let v = eval(
            &d,
            &LocatorPrimitive::CodeBlock {
                language: Some("mermaid".into()),
                under_section: Some("Workflow".into()),
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].as_str().unwrap().contains("WANT-->ME"));
        assert!(!v[0].as_str().unwrap().contains("LEAK"));
    }

    #[test]
    #[trace("FR-011-AC-14")]
    // a `~~~` fence closes on its own character, and a
    // section-owned `code_block` resolves it.
    fn code_block_resolves_tilde_fenced_block_under_section() {
        // FR-011-AC-14: a section-owned `code_block` locator resolves a
        // `~~~`-fenced block under its heading, like a ``` block.
        let d = parse_document(
            "## Workflow\n~~~mermaid\ngraph TD; WANT-->ME\n~~~\n\
             ## Other\nprose\n",
        );
        let v = eval(
            &d,
            &LocatorPrimitive::CodeBlock {
                language: Some("mermaid".into()),
                under_section: Some("Workflow".into()),
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].as_str().unwrap().contains("WANT-->ME"));
    }

    #[test]
    fn code_block_under_missing_section_returns_empty() {
        let d = parse_document("## A\n```mermaid\ng\n```\n");
        let v = eval(
            &d,
            &LocatorPrimitive::CodeBlock {
                language: Some("mermaid".into()),
                under_section: Some("Nope".into()),
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert!(v.is_empty());
    }

    #[test]
    fn table_row_returns_objects_with_header_keys() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::TableRow {
                under_section: Some("API".into()),
                column: None,
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], json!("GET\t/a"));
    }

    #[test]
    fn table_row_column_projection() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::TableRow {
                under_section: Some("API".into()),
                column: Some(ColumnRef::Name("Path".into())),
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v, vec![json!("/a"), json!("/b")]);
    }

    #[test]
    fn list_item_returns_parsed_items() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::ListItem {
                under_section: Some("Notes".into()),
                pattern: None,
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], json!("**Auth** — token-based"));
    }

    #[test]
    fn heading_returns_all_at_level() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::Heading {
                level: Some(2),
                path: None,
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert!(v.iter().any(|h| h == &json!("Purpose")));
        assert!(v.iter().any(|h| h == &json!("API")));
    }

    #[test]
    fn heading_path_walks_named_chain() {
        let d = parse_document("## A\n### B\n## C");
        let v = eval(
            &d,
            &LocatorPrimitive::Heading {
                level: None,
                path: Some(vec!["A".into(), "B".into()]),
                required: false,
                multiple: false,
                regex: None,
                assert: None,
            },
        );
        assert_eq!(v, vec![json!("B")]);
    }
}
