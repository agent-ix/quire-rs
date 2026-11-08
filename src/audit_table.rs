//! Audit-table completeness (FR-050-AC-58).
//!
//! A test matrix declares acceptance criteria twice: its Requirements
//! Traceability coverage rows state a range per requirement (`AC-1..28`), and
//! each requirement document defines the criteria themselves. The matrix's
//! `## AC → TC Coverage Audit` table is the authoritative per-criterion
//! mapping, so a criterion that is declared but has no audit row is invisible
//! to every per-row structural check: each cell that exists validates, the
//! absent row never does. This module closes that class. It reads text only;
//! `examples/spec_validate.rs` supplies the matrix and the requirement
//! documents.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use regex::Regex;

/// Where a declared criterion came from.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditGapSource {
    /// A coverage-row range in the matrix, for the named requirement.
    CoverageRange { requirement: String },
    /// A definition in the named requirement document.
    Definition { document: String },
}

/// A declared criterion with no row in the audit table.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AuditGap {
    /// The fully qualified criterion id, e.g. `FR-051-AC-18`.
    pub ac: String,
    pub source: AuditGapSource,
}

impl std::fmt::Display for AuditGap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            AuditGapSource::CoverageRange { requirement } => write!(
                f,
                "{}: declared by the {requirement} coverage row but absent from the \
                 AC → TC Coverage Audit table",
                self.ac
            ),
            AuditGapSource::Definition { document } => write!(
                f,
                "{}: defined in {document} but absent from the AC → TC Coverage Audit table",
                self.ac
            ),
        }
    }
}

fn qualified_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"([A-Z]+-\d+)-AC-(\d+)(?:\.\.(\d+))?").expect("qualified AC pattern")
    })
}

fn bare_range_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(?:^|[^-\w])AC-(\d+)(?:\.\.(\d+))?((?:\s*[,;]\s*\d+(?:\.\.\d+)?)*)")
            .expect("bare AC range pattern")
    })
}

fn parenthetical_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\([^)]*\)").expect("parenthetical pattern"))
}

fn requirement_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"^[A-Z]+-\d+").expect("requirement id pattern"))
}

fn cells(line: &str) -> Option<Vec<&str>> {
    let line = line.trim();
    let inner = line.strip_prefix('|')?;
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    Some(inner.split('|').map(str::trim).collect())
}

fn expand(prefix: &str, first: &str, last: Option<&str>, out: &mut BTreeSet<String>) {
    let (Ok(first), last) = (first.parse::<u32>(), last) else {
        return;
    };
    let last = last.and_then(|n| n.parse::<u32>().ok()).unwrap_or(first);
    for n in first..=last.max(first) {
        out.insert(format!("{prefix}-AC-{n}"));
    }
}

fn is_heading(line: &str, level: usize) -> bool {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    hashes == level && line[hashes..].starts_with(' ')
}

/// Lines of the `##` section whose title contains `title`.
fn section_lines<'a>(matrix: &'a str, title: &str) -> Vec<&'a str> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in matrix.lines() {
        if is_heading(line, 2) {
            inside = line.contains(title);
        } else if inside {
            lines.push(line);
        }
    }
    lines
}

/// Every criterion id the audit table maps, with ranges expanded. A retired
/// range row (`NFR-012-AC-1..5 | RETIRED ...`) maps its ids by being a row.
pub fn audit_table_ids(matrix: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for line in section_lines(matrix, "Coverage Audit") {
        let Some(cells) = cells(line) else { continue };
        let Some(first) = cells.first() else { continue };
        for caps in qualified_pattern().captures_iter(first) {
            expand(
                &caps[1],
                &caps[2],
                caps.get(3).map(|m| m.as_str()),
                &mut ids,
            );
        }
    }
    ids
}

/// Criteria the matrix's Requirements Traceability coverage rows declare by
/// range (`AC-1..28`), keyed by criterion id with the declaring requirement.
/// A struck-through (`~~FR-001~~`) or `RETIRED` row declares nothing.
pub fn coverage_range_ids(matrix: &str) -> BTreeMap<String, String> {
    let mut declared = BTreeMap::new();
    for line in section_lines(matrix, "Requirements Traceability") {
        let Some(cells) = cells(line) else { continue };
        let [first, criteria, rest @ ..] = cells.as_slice() else {
            continue;
        };
        if first.starts_with("~~") || rest.iter().any(|cell| cell.contains("RETIRED")) {
            continue;
        }
        let Some(requirement) = requirement_pattern().find(first) else {
            continue;
        };
        let requirement = requirement.as_str();
        // A parenthesised note ("(AC-6 withdrawn)") annotates; it declares nothing.
        let criteria = parenthetical_pattern().replace_all(criteria, " ");
        let mut ids = BTreeSet::new();
        for caps in bare_range_pattern().captures_iter(&criteria) {
            expand(
                requirement,
                &caps[1],
                caps.get(2).map(|m| m.as_str()),
                &mut ids,
            );
            // `AC-1..8, 13..21`: later items of the list carry no `AC-` prefix.
            for item in caps[3]
                .split([',', ';'])
                .map(str::trim)
                .filter(|i| !i.is_empty())
            {
                let (first, last) = item
                    .split_once("..")
                    .map_or((item, None), |(a, b)| (a, Some(b)));
                expand(requirement, first, last, &mut ids);
            }
        }
        for id in ids {
            declared.insert(id, requirement.to_string());
        }
    }
    declared
}

/// A retired criterion keeps its row but its statement opens `RETIRED`,
/// `(RETIRED)`, `Withdrawn` or is struck through.
fn is_tombstone(statement: &str) -> bool {
    statement.starts_with("~~")
        || statement.trim_start_matches('(').starts_with("RETIRED")
        || statement.starts_with("Withdrawn")
}

/// Criteria a requirement document defines: a bold `**FR-050-AC-1**` anchor
/// or an `| FR-050-AC-1 |` leading table cell. A row whose statement begins
/// `RETIRED` is a tombstone, not a live criterion.
pub fn defined_ids(document: &str) -> BTreeSet<String> {
    static BOLD: OnceLock<Regex> = OnceLock::new();
    let bold = BOLD.get_or_init(|| {
        Regex::new(r"\*\*([A-Z]+-\d+-AC-\d+)\*\*").expect("bold AC anchor pattern")
    });
    let mut ids = BTreeSet::new();
    for line in document.lines() {
        if let Some(cells) = cells(line) {
            if let [first, statement, ..] = cells.as_slice() {
                let exact = qualified_pattern().captures(first).filter(|c| {
                    c.get(0).is_some_and(|m| m.as_str() == *first) && c.get(3).is_none()
                });
                if let Some(caps) = exact {
                    if !is_tombstone(statement) {
                        ids.insert(format!("{}-AC-{}", &caps[1], &caps[2]));
                    }
                    continue;
                }
            }
        }
        for caps in bold.captures_iter(line) {
            ids.insert(caps[1].to_string());
        }
    }
    ids
}

/// Every declared criterion (coverage-row range or `documents` definition)
/// that the matrix's audit table does not map, sorted by id. `documents` is
/// `(label, text)` pairs of the requirement documents.
pub fn audit_gaps(matrix: &str, documents: &[(&str, &str)]) -> Vec<AuditGap> {
    let mapped = audit_table_ids(matrix);
    let mut gaps = Vec::new();
    for (ac, requirement) in coverage_range_ids(matrix) {
        if !mapped.contains(&ac) {
            gaps.push(AuditGap {
                ac,
                source: AuditGapSource::CoverageRange { requirement },
            });
        }
    }
    for (label, text) in documents {
        for ac in defined_ids(text) {
            if !mapped.contains(&ac) {
                gaps.push(AuditGap {
                    ac,
                    source: AuditGapSource::Definition {
                        document: (*label).to_string(),
                    },
                });
            }
        }
    }
    gaps.sort();
    gaps
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATRIX: &str = "\
## Requirements Traceability

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-001 Thing | AC-1..3; CON-1..2 | TC-1 | done |
| ~~FR-002 Gone~~ | AC-1..9 | TC-2 | RETIRED |

## AC → TC Coverage Audit

| AC | Covering TC(s) |
|---|---|
| FR-001-AC-1 | TC-1 |
| FR-001-AC-3 | TC-1 |
| FR-003-AC-1..2 | RETIRED |
";

    #[test]
    fn a_range_expands_and_a_retired_row_declares_nothing() {
        let declared = coverage_range_ids(MATRIX);
        let ids: Vec<&str> = declared.keys().map(String::as_str).collect();
        assert_eq!(ids, ["FR-001-AC-1", "FR-001-AC-2", "FR-001-AC-3"]);
    }

    #[test]
    fn audit_ids_expand_ranges() {
        let ids = audit_table_ids(MATRIX);
        assert!(ids.contains("FR-003-AC-2"));
        assert!(!ids.contains("FR-001-AC-2"));
    }

    #[test]
    fn definitions_skip_tombstones() {
        let doc = "| FR-009-AC-1 | live | T |\n| FR-009-AC-2 | RETIRED — gone | T |\n- **FR-009-AC-3**: x\n";
        let ids: Vec<String> = defined_ids(doc).into_iter().collect();
        assert_eq!(ids, ["FR-009-AC-1", "FR-009-AC-3"]);
    }
}
