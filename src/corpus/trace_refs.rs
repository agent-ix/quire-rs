//! Verification-reference integrity (FR-049).
//!
//! Table cells reference trace ids — the ISO AC `Verification` cell's
//! `Test (TC-035)` annotation is the motivating case — and those references
//! deserve the same dangling check `ix://` edges already get. Nothing about
//! the pattern, the column, or the target archetype is engine knowledge: every
//! part comes from the active module's declared traceability model (FR-050).
//!
//! The resolution set for a target kind is the union of bundle documents whose
//! archetype matches a declared target and the trace ids minted by declared
//! **auxiliary sources** — files the corpus walk excludes as non-artifacts
//! (`tests.md`), harvested here with a targeted scan following the FR-044
//! glossary-harvester pattern.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use regex::Regex;

use super::spec::Spec;
use super::validate::{BundleFinding, BundlePosture, BundleReport};
use crate::query::{concept_type, parse_table, section};
use crate::registry::Registry;
use crate::traceability::{DocumentReference, TraceTarget};

/// One referencing row found in a declared reference column.
struct ReferencingRow {
    /// Path of the document the reference was authored in.
    path: PathBuf,
    /// The row's own id, when the declaration names a `row_id_column`.
    row_id: Option<String>,
    /// The raw cell text.
    cell: String,
}

/// Check every declared document reference against the bundle's resolution
/// set, appending posture-degradable `dangling-trace-reference` findings.
///
/// A no-op when no active module declares a traceability model (FR-049-AC-6).
pub(crate) fn validate_trace_references(
    spec: &Spec,
    registry: &Registry,
    posture: BundlePosture,
    root: &Path,
    report: &mut BundleReport,
) {
    let Some(model) = registry.traceability() else {
        return;
    };
    if model.document_references.is_empty() {
        return;
    }

    // Resolution sets, one per declared target kind.
    let mut resolution: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for target in &model.trace_targets {
        resolution.insert(target.name.as_str(), minted_ids(spec, root, target));
    }

    // Collect findings before pushing so their order is a property of the
    // bundle, not of the walk (FR-049-AC-8).
    let mut findings: Vec<(PathBuf, String, String)> = Vec::new();
    for declaration in &model.document_references {
        let Ok(pattern) = Regex::new(&declaration.pattern) else {
            continue; // patterns are validated at module load
        };
        let mut resolvable: BTreeSet<&String> = BTreeSet::new();
        for target in &declaration.targets {
            if let Some(ids) = resolution.get(target.as_str()) {
                resolvable.extend(ids.iter());
            }
        }
        for row in referencing_rows(spec, root, declaration) {
            for caps in pattern.captures_iter(&row.cell) {
                let Some(id) = caps.get(1).map(|m| m.as_str().trim()) else {
                    continue;
                };
                if id.is_empty() || resolvable.iter().any(|known| known.as_str() == id) {
                    continue;
                }
                let owner = row
                    .row_id
                    .as_deref()
                    .map(|r| format!(" (row '{r}')"))
                    .unwrap_or_default();
                findings.push((
                    row.path.clone(),
                    id.to_string(),
                    format!(
                        "trace reference '{id}'{owner} declared by '{}' resolves to no {} target",
                        declaration.name,
                        declaration.targets.join("/")
                    ),
                ));
            }
        }
    }

    findings.sort();
    findings.dedup();
    for (path, _, message) in findings {
        report.degradable(
            posture,
            BundleFinding {
                path,
                message,
                reason: "dangling-trace-reference",
            },
        );
    }
}

/// The ids a declared target mints: from bundle documents of the declared
/// archetype, or from a declared auxiliary document harvested off-corpus.
fn minted_ids(spec: &Spec, root: &Path, target: &TraceTarget) -> BTreeSet<String> {
    let mut ids: BTreeSet<String> = BTreeSet::new();
    if let Some(archetype) = &target.archetype {
        for doc in &spec.inner.documents {
            if concept_type(&doc.doc) != Some(archetype.as_str()) {
                continue;
            }
            // A document of the target archetype mints its own id…
            if !doc.id.is_empty() {
                ids.insert(doc.id.clone());
            }
            // …plus every id in its declared minting column, when it has one
            // (the FR `Acceptance Criteria` `ID` column mints AC ids).
            ids.extend(column_values(&doc.doc, &target.section, &target.id_column));
        }
    }
    if let Some(document) = &target.document {
        if let Some(doc) = harvest(root, document) {
            ids.extend(column_values(&doc, &target.section, &target.id_column));
        }
    }
    ids
}

/// Every referencing row a declaration covers, in bundle order.
fn referencing_rows(
    spec: &Spec,
    root: &Path,
    declaration: &DocumentReference,
) -> Vec<ReferencingRow> {
    let mut out = Vec::new();
    if let Some(archetype) = &declaration.archetype {
        for doc in &spec.inner.documents {
            if concept_type(&doc.doc) != Some(archetype.as_str()) {
                continue;
            }
            out.extend(rows_of(
                &doc.doc,
                &doc.path,
                &declaration.section,
                &declaration.column,
                declaration.row_id_column.as_deref(),
            ));
        }
    }
    if let Some(document) = &declaration.document {
        if let Some(doc) = harvest(root, document) {
            out.extend(rows_of(
                &doc,
                &root.join(document),
                &declaration.section,
                &declaration.column,
                declaration.row_id_column.as_deref(),
            ));
        }
    }
    out
}

fn rows_of(
    doc: &crate::ast::QuireDocument,
    path: &Path,
    heading: &str,
    column: &str,
    row_id_column: Option<&str>,
) -> Vec<ReferencingRow> {
    let Some(sec) = section(doc, heading) else {
        return Vec::new();
    };
    let Some(table) = parse_table(&sec.content) else {
        return Vec::new();
    };
    let Some(col_idx) = column_index(&table.headers, column) else {
        return Vec::new();
    };
    let id_idx = row_id_column.and_then(|c| column_index(&table.headers, c));
    table
        .rows
        .iter()
        .filter_map(|row| {
            let cell = row.get(col_idx)?.trim();
            (!cell.is_empty()).then(|| ReferencingRow {
                path: path.to_path_buf(),
                row_id: id_idx
                    .and_then(|i| row.get(i))
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty()),
                cell: cell.to_string(),
            })
        })
        .collect()
}

/// Non-empty values of `column` in the table under `heading`.
fn column_values(doc: &crate::ast::QuireDocument, heading: &str, column: &str) -> Vec<String> {
    let Some(sec) = section(doc, heading) else {
        return Vec::new();
    };
    let Some(table) = parse_table(&sec.content) else {
        return Vec::new();
    };
    let Some(idx) = column_index(&table.headers, column) else {
        return Vec::new();
    };
    table
        .rows
        .iter()
        .filter_map(|row| row.get(idx))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

fn column_index(headers: &[String], column: &str) -> Option<usize> {
    headers
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case(column))
}

/// Targeted scan of a declared auxiliary source — a file the corpus walk
/// excludes as a non-artifact. Unreadable or absent → no ids, no finding: the
/// declaration names an optional source, not a required one.
fn harvest(root: &Path, document: &Path) -> Option<crate::ast::QuireDocument> {
    let path = root.join(document);
    let text = std::fs::read_to_string(path).ok()?;
    Some(crate::parse_document(&text))
}
