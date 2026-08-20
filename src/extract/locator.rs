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
    diagrams_from_content, parse_bullet_list, parse_table, parse_tables, section as q_section,
    sections as q_sections, table_from_section, ListPattern,
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

/// Implements: FR-016
/// Evaluate a [`Locator`] (primitive or fallback chain) against
/// `doc`. For a fallback chain, returns the first primitive that
/// produces a non-empty result and records its position so the caller
/// can emit a `FallbackLocatorUsed` diagnostic when `position > 0`.
///
/// Returns `(values, position_used)` where `position_used == 0` for a
/// `Primitive` or a fallback that hit at the head.
pub fn eval_locator(doc: &QuireDocument, locator: &Locator) -> (Vec<Value>, usize) {
    match locator {
        Locator::Primitive(p) => (eval(doc, p), 0),
        Locator::Fallback(chain) => {
            for (i, p) in chain.iter().enumerate() {
                let v = eval(doc, p);
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
    let (values, regex) = match loc {
        LocatorPrimitive::FrontmatterField { path, regex, .. } => {
            (eval_frontmatter_field(doc, path), regex.as_deref())
        }
        LocatorPrimitive::SectionBody { after_heading, .. } => {
            let regex = match loc {
                LocatorPrimitive::SectionBody { regex, .. } => regex.as_deref(),
                _ => None,
            };
            (eval_section_body(doc, after_heading), regex)
        }
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
    };
    apply_regex(values, regex)
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

fn apply_regex(values: Vec<Value>, pattern: Option<&str>) -> Vec<Value> {
    let Some(pattern) = pattern else {
        return values;
    };
    let Ok(re) = Regex::new(pattern) else {
        return Vec::new();
    };
    values
        .into_iter()
        .filter_map(|value| match value {
            Value::String(s) => re.captures(&s).map(|caps| {
                let matched = caps
                    .get(1)
                    .or_else(|| caps.get(0))
                    .map(|m| m.as_str())
                    .unwrap_or("");
                Value::String(matched.to_string())
            }),
            other => Some(other),
        })
        .collect()
}

fn eval_frontmatter_field(doc: &QuireDocument, path: &[String]) -> Vec<Value> {
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
        vec![cur]
    }
}

fn eval_section_body(doc: &QuireDocument, heading: &str) -> Vec<Value> {
    match q_section(doc, heading) {
        Some(s) => vec![Value::String(s.content.trim().to_string())],
        None => Vec::new(),
    }
}

fn eval_code_block(
    doc: &QuireDocument,
    language: Option<&str>,
    under_section: Option<&str>,
) -> Vec<Value> {
    // Section-owned: resolve from the section's content slice (and thus,
    // under multi-yield, the iteration unit's slice), mirroring
    // `eval_list_item`. The document-wide `extract_diagrams` harvest is a
    // separate query, not the locator substrate (FR-011).
    let content: String = match under_section {
        Some(h) => match q_section(doc, h) {
            Some(s) => s.content.clone(),
            None => return Vec::new(),
        },
        None => doc
            .sections
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    };
    diagrams_from_content(&content, language)
        .into_iter()
        .map(|b| Value::String(b.source))
        .collect()
}

fn eval_table_row(
    doc: &QuireDocument,
    under_section: Option<&str>,
    column: Option<&ColumnRef>,
) -> Vec<Value> {
    let table = match under_section {
        Some(h) => table_from_section(doc, h),
        None => doc
            .sections
            .first()
            .and_then(|s| parse_table(&s.content))
            .or_else(|| {
                doc.sections
                    .iter()
                    .flat_map(|s| parse_tables(&s.content))
                    .next()
            }),
    };
    let table = match table {
        Some(t) => t,
        None => return Vec::new(),
    };
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
        table
            .rows
            .into_iter()
            .filter_map(|r| r.get(idx).cloned().map(Value::String))
            .collect()
    } else {
        table
            .rows
            .into_iter()
            .map(|r| Value::String(r.join("\t")))
            .collect()
    }
}

fn eval_list_item(
    doc: &QuireDocument,
    under_section: Option<&str>,
    pattern: Option<ListPattern>,
) -> Vec<Value> {
    let content: String = match under_section {
        Some(h) => match q_section(doc, h) {
            Some(s) => s.content.clone(),
            None => return Vec::new(),
        },
        None => doc
            .sections
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    };
    parse_bullet_list(&content, pattern)
        .into_iter()
        .map(|item| Value::String(item.raw))
        .collect()
}

fn eval_heading(doc: &QuireDocument, level: Option<u8>, path: Option<&[String]>) -> Vec<Value> {
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
        current
            .map(|s| vec![Value::String(crate::query::normalize_heading(&s.heading))])
            .unwrap_or_default()
    } else {
        q_sections(doc, level)
            .into_iter()
            .map(|s| Value::String(crate::query::normalize_heading(&s.heading)))
            .collect()
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
    #[trace("TC-563", "FR-011-AC-13")]
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
    #[trace("TC-564", "FR-011-AC-14")]
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
