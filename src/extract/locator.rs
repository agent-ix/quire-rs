//! The six body-extraction Locator primitives (FR-011).
//!
//! Each primitive reads from a [`QuireDocument`] via the Query API
//! (Task 003) and returns either a single string, a sequence of
//! strings, or — for the structural primitives like `table_row` —
//! a sequence of `serde_json::Value`s. The single-yield evaluator
//! collapses sequences to their first element when the locator is
//! bound under a `match:` key; the multi-yield evaluator iterates.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ast::QuireDocument;
use crate::query::{
    extract_diagrams, parse_bullet_list, parse_table, parse_tables, section as q_section,
    sections as q_sections, table_from_section, ListPattern,
};

/// One Locator from a `body_extraction` DSL. Untagged on the YAML
/// side: the discriminant is the `from:` field, captured here as the
/// enum variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "snake_case")]
pub enum LocatorPrimitive {
    /// `from: frontmatter_field` — `doc.frontmatter[path[0]][path[1]]...`
    FrontmatterField {
        path: Vec<String>,
        #[serde(default)]
        required: bool,
    },
    /// `from: section_body` — text content of `after_heading` section.
    SectionBody {
        after_heading: String,
        #[serde(default)]
        required: bool,
    },
    /// `from: code_block` — source of the first fenced code block in
    /// `language`, optionally constrained to a section.
    CodeBlock {
        language: String,
        #[serde(default)]
        under_section: Option<String>,
        #[serde(default)]
        required: bool,
    },
    /// `from: table_row` — rows from a table (optionally inside a
    /// section, optionally projecting a single column).
    TableRow {
        #[serde(default)]
        under_section: Option<String>,
        #[serde(default)]
        column: Option<String>,
        #[serde(default)]
        required: bool,
    },
    /// `from: list_item` — items from a bullet list.
    ListItem {
        #[serde(default)]
        under_section: Option<String>,
        #[serde(default)]
        pattern: Option<ListPattern>,
        #[serde(default)]
        required: bool,
    },
    /// `from: heading` — heading text of sections at `level`, or under
    /// `path`.
    Heading {
        #[serde(default)]
        level: Option<u8>,
        #[serde(default)]
        path: Option<Vec<String>>,
        #[serde(default)]
        required: bool,
    },
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
                Some(s) => format!("code_block({language} under {s})"),
                None => format!("code_block({language})"),
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
}

/// Evaluate a `LocatorPrimitive` against `doc`. Returns an ordered
/// list of values; the caller picks first-or-all depending on its
/// yield pattern.
pub fn eval(doc: &QuireDocument, loc: &LocatorPrimitive) -> Vec<Value> {
    match loc {
        LocatorPrimitive::FrontmatterField { path, .. } => eval_frontmatter_field(doc, path),
        LocatorPrimitive::SectionBody { after_heading, .. } => {
            eval_section_body(doc, after_heading)
        }
        LocatorPrimitive::CodeBlock {
            language,
            under_section,
            ..
        } => eval_code_block(doc, language, under_section.as_deref()),
        LocatorPrimitive::TableRow {
            under_section,
            column,
            ..
        } => eval_table_row(doc, under_section.as_deref(), column.as_deref()),
        LocatorPrimitive::ListItem {
            under_section,
            pattern,
            ..
        } => eval_list_item(doc, under_section.as_deref(), *pattern),
        LocatorPrimitive::Heading { level, path, .. } => eval_heading(doc, *level, path.as_deref()),
    }
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

fn eval_code_block(doc: &QuireDocument, language: &str, under_section: Option<&str>) -> Vec<Value> {
    let blocks = extract_diagrams(doc, Some(language));
    let filtered: Vec<&_> = match under_section {
        Some(s) => blocks
            .iter()
            .filter(|b| b.section.as_deref() == Some(s))
            .collect(),
        None => blocks.iter().collect(),
    };
    filtered
        .into_iter()
        .map(|b| Value::String(b.source.clone()))
        .collect()
}

fn eval_table_row(
    doc: &QuireDocument,
    under_section: Option<&str>,
    column: Option<&str>,
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
        let idx = match table.headers.iter().position(|h| h == col) {
            Some(i) => i,
            None => return Vec::new(),
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
            .map(|r| {
                let mut obj = serde_json::Map::new();
                for (i, cell) in r.into_iter().enumerate() {
                    let key = table
                        .headers
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| i.to_string());
                    obj.insert(key, Value::String(cell));
                }
                Value::Object(obj)
            })
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
        .map(|item| {
            let mut obj = serde_json::Map::new();
            obj.insert("raw".to_string(), Value::String(item.raw));
            obj.insert("title".to_string(), Value::String(item.title));
            obj.insert("description".to_string(), Value::String(item.description));
            Value::Object(obj)
        })
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
            .map(|s| vec![Value::String(s.heading.clone())])
            .unwrap_or_default()
    } else {
        q_sections(doc, level)
            .into_iter()
            .map(|s| Value::String(s.heading.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;
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
                language: "mermaid".into(),
                under_section: None,
                required: false,
            },
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].as_str().unwrap().contains("A-->B"));
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
            },
        );
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], json!({"Method": "GET", "Path": "/a"}));
    }

    #[test]
    fn table_row_column_projection() {
        let d = doc();
        let v = eval(
            &d,
            &LocatorPrimitive::TableRow {
                under_section: Some("API".into()),
                column: Some("Path".into()),
                required: false,
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
            },
        );
        assert_eq!(v.len(), 2);
        assert_eq!(v[0]["title"], json!("Auth"));
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
            },
        );
        assert_eq!(v, vec![json!("B")]);
    }
}
