//! Quire AST types (FR-005).
//!
//! Public, serializable, thread-safe (`Send + Sync`) shape of a parsed
//! markdown document. Mirrors the TS `QuireDocument` /
//! `QuireSection` types from `~/dev/quire/src/core/types.ts` so the
//! Task 020 parity sweep can compare structurally.
//!
//! `Eq` is *not* derived: `frontmatter` carries `serde_json::Value`,
//! whose `Number(f64)` variant is `PartialEq` but not `Eq` (NaN). All
//! consumer comparisons use `PartialEq`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// One heading-bounded section of a parsed markdown document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuireSection {
    /// Stable `<slug>-L<line>` identifier (see `parser::slug_line_id`).
    pub id: String,
    /// Heading text, leading/trailing whitespace stripped.
    pub heading: String,
    /// ATX heading level, `1..=6`.
    pub level: u8,
    /// Byte-exact content between this heading and the next heading
    /// (any level), or end of body. NOT trimmed (diverges from TS/Py
    /// reference — see FR-008).
    pub content: String,
    /// Direct child sections (strictly deeper level).
    pub children: Vec<QuireSection>,
    /// 0-based line index of the heading line (within the body, after
    /// frontmatter is stripped).
    pub start_line: usize,
    /// 0-based line index of the last content line in this section
    /// (line before the next heading, or last line of body).
    pub end_line: usize,
}

/// A parsed markdown document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuireDocument {
    /// Text before the first heading, trimmed. `None` if absent or
    /// whitespace-only (matches TS/Py reference).
    pub preamble: Option<String>,
    /// Top-level sections in document order.
    pub sections: Vec<QuireSection>,
    /// Verbatim original input (including any frontmatter and BOM).
    pub raw: String,
    /// Parsed frontmatter map, or `None` if absent or malformed
    /// (FR-006 fallback semantics).
    pub frontmatter: Option<Map<String, Value>>,
}

impl QuireDocument {
    /// The empty document — what `parse_document("")` returns.
    pub fn empty() -> Self {
        Self {
            preamble: None,
            sections: Vec::new(),
            raw: String::new(),
            frontmatter: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<QuireDocument>();
        assert_send_sync::<QuireSection>();
    }

    #[test]
    fn empty_document_value() {
        let d = QuireDocument::empty();
        assert!(d.preamble.is_none());
        assert!(d.sections.is_empty());
        assert_eq!(d.raw, "");
        assert!(d.frontmatter.is_none());
    }

    #[test]
    fn round_trip_serde_json() {
        let s = QuireSection {
            id: "x-L0".into(),
            heading: "X".into(),
            level: 2,
            content: "body".into(),
            children: vec![],
            start_line: 0,
            end_line: 1,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: QuireSection = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
