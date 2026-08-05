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

/// Normalize a reference cell before ids are extracted, per the declaration's
/// opt-in flags (CR-015). Both consumers — FR-049's dangling check and FR-050's
/// rollup — call this, so they cannot disagree about what a cell contains.
///
/// `strip_annotations` removes parenthetical spans, so a qualifier never
/// contributes a reference. `expand_ranges` rewrites `FR-001..FR-003` into
/// `FR-001, FR-002, FR-003`; a range whose ends disagree on prefix, or whose
/// bounds are inverted, is left untouched for the pattern to reject.
pub(crate) fn normalize_reference_cell(
    cell: &str,
    strip_annotations: bool,
    expand_ranges: bool,
) -> String {
    let mut out = cell.to_string();
    if strip_annotations {
        out = re_parenthetical().replace_all(&out, " ").to_string();
    }
    if expand_ranges {
        out = re_range()
            .replace_all(&out, |caps: &regex::Captures<'_>| expand(caps))
            .to_string();
    }
    out
}

/// Expand one matched range, or return it unchanged when it is not a coherent
/// same-prefix ascending range.
fn expand(caps: &regex::Captures<'_>) -> String {
    let whole = caps.get(0).map_or("", |m| m.as_str()).to_string();
    let (prefix, from, to) = (&caps[1], &caps[2], &caps[4]);
    if prefix != &caps[3] {
        return whole; // `FR-001..NFR-003` is not a range
    }
    let width = from.len();
    let (Ok(start), Ok(end)) = (from.parse::<u32>(), to.parse::<u32>()) else {
        return whole;
    };
    if end < start || end - start > MAX_RANGE_SPAN {
        return whole;
    }
    (start..=end)
        .map(|n| format!("{prefix}-{n:0width$}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A range wider than this is treated as authoring noise rather than expanded —
/// it would swamp the report with ids nobody enumerated deliberately.
const MAX_RANGE_SPAN: u32 = 200;

fn re_parenthetical() -> &'static regex::Regex {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"\([^)]*\)").expect("parenthetical regex"))
}

fn re_range() -> &'static regex::Regex {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| {
        regex::Regex::new(r"([A-Za-z]+)-(\d+)\s*\.\.\s*([A-Za-z]+)-(\d+)").expect("range regex")
    })
}

/// Targeted scan of a declared auxiliary source — a file the corpus walk
/// excludes as a non-artifact (FR-044 glossary-harvester pattern). An absent or
/// unreadable file yields nothing: the declaration names a source, not a
/// requirement.
pub(crate) fn harvest(root: &Path, document: &Path) -> Option<QuireDocument> {
    let text = std::fs::read_to_string(root.join(document)).ok()?;
    Some(crate::parse_document(&text))
}
