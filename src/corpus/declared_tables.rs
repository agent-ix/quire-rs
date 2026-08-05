//! Scanning the tables a traceability model declares (FR-049 / FR-050).
//!
//! Both the reference-integrity check and the coverage rollup read the same
//! shape: "the table under section S of every document of archetype A, or of a
//! declared auxiliary document". This is that one scan, so the two consumers
//! cannot drift apart on which rows they see.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::spec::Spec;
use crate::ast::QuireDocument;
use crate::query::{concept_type, parse_table, section};

/// One scanned table row: the document it lives in and its cells keyed by the
/// table's own column headers (trimmed, as authored).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScannedRow {
    pub path: PathBuf,
    pub cells: BTreeMap<String, String>,
}

impl ScannedRow {
    /// The value of `column`, case-insensitively, when non-empty.
    pub fn cell(&self, column: &str) -> Option<&str> {
        self.cells
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(column))
            .map(|(_, value)| value.as_str())
            .filter(|v| !v.is_empty())
    }
}

/// Scan the table under `heading` for a declaration bound to `archetype`
/// (every bundle document of that type) and/or `document` (one auxiliary file
/// under `root`, harvested off-corpus). Bundle documents come first, in corpus
/// order; the auxiliary rows follow.
pub(crate) fn scan(
    spec: &Spec,
    root: &Path,
    archetype: Option<&str>,
    document: Option<&Path>,
    heading: &str,
) -> Vec<ScannedRow> {
    let mut out = Vec::new();
    if let Some(archetype) = archetype {
        for doc in &spec.inner.documents {
            if concept_type(&doc.doc) != Some(archetype) {
                continue;
            }
            out.extend(rows_of(&doc.doc, &doc.path, heading));
        }
    }
    if let Some(document) = document {
        if let Some(doc) = harvest(root, document) {
            out.extend(rows_of(&doc, &root.join(document), heading));
        }
    }
    out
}

/// The rows of the table under `heading` in one document.
pub(crate) fn rows_of(doc: &QuireDocument, path: &Path, heading: &str) -> Vec<ScannedRow> {
    let Some(sec) = section(doc, heading) else {
        return Vec::new();
    };
    let Some(table) = parse_table(&sec.content) else {
        return Vec::new();
    };
    table
        .rows
        .iter()
        .map(|row| {
            let mut cells = BTreeMap::new();
            for (idx, header) in table.headers.iter().enumerate() {
                cells.insert(
                    header.trim().to_string(),
                    row.get(idx).map(|v| v.trim()).unwrap_or("").to_string(),
                );
            }
            ScannedRow {
                path: path.to_path_buf(),
                cells,
            }
        })
        .collect()
}

/// Targeted scan of a declared auxiliary source — a file the corpus walk
/// excludes as a non-artifact (FR-044 glossary-harvester pattern). An absent or
/// unreadable file yields nothing: the declaration names a source, not a
/// requirement.
pub(crate) fn harvest(root: &Path, document: &Path) -> Option<QuireDocument> {
    let text = std::fs::read_to_string(root.join(document)).ok()?;
    Some(crate::parse_document(&text))
}
