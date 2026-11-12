//! Audit-table completeness (FR-050-AC-58).
//!
//! A test matrix declares acceptance criteria twice: its Requirements
//! Traceability coverage rows state a range per requirement (`AC-1..28`), and
//! each requirement document defines the criteria themselves. The matrix's
//! `## AC → TC Coverage Audit` table is the authoritative per-criterion
//! mapping, so a criterion that is declared but has no audit row is invisible
//! to every per-row structural check: each cell that exists validates, the
//! absent row never does. This module closes that class in both directions:
//! a declared criterion with no row, and a row (or coverage-range id) naming a
//! criterion no document defines. It reads text only; `examples/spec_validate.rs`
//! supplies the matrix and the requirement documents.
//!
//! The vocabulary is the quire spec corpus's own and is fixed here rather than
//! read from a module: the matrix archetype declares no audit-table section or
//! tombstone marker, so there is no declaration to take it from. It is
//! deliberately minimal: the audit section title, the id shape
//! `<letters>-<digits>-AC-<n>`, and the three tombstone markers below.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use regex::Regex;

/// Title fragment of the `##` section holding the AC → TC audit tables.
const AUDIT_SECTION: &str = "Coverage Audit";
/// Title fragment of the `##` section holding the coverage rows.
const COVERAGE_SECTION: &str = "Requirements Traceability";
/// The largest `AC-a..b` span accepted; a typo such as `AC-1..4000000`
/// is an error, not four million findings.
const MAX_RANGE_SPAN: u32 = 1000;

/// Where a declared criterion came from.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditSource {
    /// A coverage-row range in the matrix, for the named requirement.
    CoverageRange { requirement: String },
    /// A definition in the named requirement document.
    Definition { document: String },
}

/// One audit-table finding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditFinding {
    /// A declared criterion with no audit row. One finding per criterion;
    /// `sources` lists every declaration of it.
    Missing {
        ac: String,
        sources: Vec<AuditSource>,
    },
    /// An audit row or coverage-range id naming a criterion no requirement
    /// document defines.
    Phantom { ac: String, source: PhantomSource },
    /// A criteria or id cell the check cannot read in full: reading less than
    /// the cell says would silently shrink the declared population.
    Unreadable { location: String, cell: String },
}

/// Where a phantom id was written.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhantomSource {
    AuditRow,
    CoverageRange { requirement: String },
}

impl AuditFinding {
    /// The criterion (or location) the finding names.
    pub fn subject(&self) -> &str {
        match self {
            Self::Missing { ac, .. } | Self::Phantom { ac, .. } => ac,
            Self::Unreadable { location, .. } => location,
        }
    }
}

impl std::fmt::Display for AuditSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CoverageRange { requirement } => write!(f, "the {requirement} coverage row"),
            Self::Definition { document } => write!(f, "{document}"),
        }
    }
}

impl std::fmt::Display for AuditFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { ac, sources } => {
                let sources: Vec<String> = sources.iter().map(ToString::to_string).collect();
                write!(
                    f,
                    "{ac}: declared by {} but absent from the AC → TC Coverage Audit table",
                    sources.join(" and ")
                )
            }
            Self::Phantom { ac, source } => match source {
                PhantomSource::AuditRow => write!(
                    f,
                    "{ac}: has an AC → TC Coverage Audit row but no requirement document defines it"
                ),
                PhantomSource::CoverageRange { requirement } => write!(
                    f,
                    "{ac}: declared by the {requirement} coverage row but no requirement \
                     document defines it"
                ),
            },
            Self::Unreadable { location, cell } => {
                write!(
                    f,
                    "{location}: cannot read the criteria cell `{cell}` in full"
                )
            }
        }
    }
}

struct Patterns {
    qualified: Regex,
    exact: Regex,
    bare: Regex,
    bare_mention: Regex,
    parenthetical: Regex,
    requirement: Regex,
    bold: Regex,
}

fn patterns() -> &'static Patterns {
    static PATTERNS: OnceLock<Patterns> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let compile = |pattern: &str| Regex::new(pattern).expect("audit table pattern");
        Patterns {
            // `StR-003-AC-1`, `FR-051-AC-18`, `NFR-012-AC-1..5`, anchored at a
            // token boundary so `StR-` is never read as `R-`.
            qualified: compile(r"(?:^|[^A-Za-z0-9-])([A-Za-z]+-\d+)-AC-(\d+)(?:\.\.(\d+))?"),
            exact: compile(r"^([A-Za-z]+-\d+)-AC-(\d+)$"),
            // `AC-1`, `AC-1..28`, `AC-1..AC-3`, `AC-1–3`, then a `, 4..8` list.
            bare: compile(
                r"(?:^|[^-\w])AC-(\d+)(?:(?:\.\.|–|—)(?:AC-)?(\d+))?((?:\s*[,;]\s*\d+(?:\.\.\d+)?)*)",
            ),
            bare_mention: compile(r"(?i)(?:^|[^-\w.–—])ac-\d"),
            parenthetical: compile(r"\([^)]*\)"),
            requirement: compile(r"^[A-Za-z]+-\d+"),
            bold: compile(r"\*\*([A-Za-z]+-\d+-AC-\d+)\*\*"),
        }
    })
}

fn cells(line: &str) -> Option<Vec<&str>> {
    let inner = line.trim().strip_prefix('|')?;
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    Some(inner.split('|').map(str::trim).collect())
}

/// Add `prefix-AC-first..=last`; `false` when the span is reversed or wider
/// than [`MAX_RANGE_SPAN`], or a number does not parse.
fn expand(prefix: &str, first: &str, last: Option<&str>, out: &mut BTreeSet<String>) -> bool {
    let Ok(first) = first.parse::<u32>() else {
        return false;
    };
    let last = match last {
        Some(last) => match last.parse::<u32>() {
            Ok(last) => last,
            Err(_) => return false,
        },
        None => first,
    };
    if last < first || last - first > MAX_RANGE_SPAN {
        return false;
    }
    out.extend((first..=last).map(|n| format!("{prefix}-AC-{n}")));
    true
}

/// The lines of the `##` section whose title contains `title`, outside code
/// fences.
fn section_lines<'a>(matrix: &'a str, title: &str) -> Vec<&'a str> {
    let mut inside = false;
    let mut fenced = false;
    let mut lines = Vec::new();
    for line in matrix.lines() {
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if line.starts_with("## ") {
            inside = line.contains(title);
        } else if inside {
            lines.push(line);
        }
    }
    lines
}

/// Every criterion the audit tables map, with ranges expanded, and the subset
/// whose row states it is retired. Only a table whose header cell is `AC`
/// counts, so a gaps or summary table in the section maps nothing.
fn audit_ids(
    matrix: &str,
    findings: &mut Vec<AuditFinding>,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut mapped = BTreeSet::new();
    let mut retired = BTreeSet::new();
    let mut in_audit_table = false;
    for line in section_lines(matrix, AUDIT_SECTION) {
        let Some(cells) = cells(line) else {
            in_audit_table = false;
            continue;
        };
        let Some(first) = cells.first() else { continue };
        if first.eq_ignore_ascii_case("AC") {
            in_audit_table = true;
            continue;
        }
        if !in_audit_table || first.chars().all(|c| c == '-' || c == ':' || c == ' ') {
            continue;
        }
        let mut row_ids = BTreeSet::new();
        for caps in patterns().qualified.captures_iter(first) {
            let last = caps.get(3).map(|m| m.as_str());
            if !expand(&caps[1], &caps[2], last, &mut row_ids) {
                findings.push(AuditFinding::Unreadable {
                    location: "AC → TC Coverage Audit".to_string(),
                    cell: (*first).to_string(),
                });
            }
        }
        if cells.get(1).is_some_and(|cell| is_tombstone(cell)) {
            retired.extend(row_ids.iter().cloned());
        }
        mapped.extend(row_ids);
    }
    (mapped, retired)
}

/// Criteria the matrix's coverage rows declare by range (`AC-1..28`), keyed by
/// criterion id with the declaring requirement. A row declares nothing when
/// its first cell is struck through or its last (status) cell begins
/// `RETIRED`. A criteria cell that does not read in full is an `Unreadable`
/// finding, never a smaller declaration.
fn coverage_ranges(matrix: &str, findings: &mut Vec<AuditFinding>) -> BTreeMap<String, String> {
    let patterns = patterns();
    let mut declared = BTreeMap::new();
    let mut in_table = false;
    for line in section_lines(matrix, COVERAGE_SECTION) {
        let Some(cells) = cells(line) else {
            in_table = false;
            continue;
        };
        if !in_table {
            // The header row opens a table; its delimiter row follows.
            in_table = true;
            continue;
        }
        let [first, criteria, .., status] = cells.as_slice() else {
            continue;
        };
        if first.starts_with("~~") || first.chars().all(|c| c == '-' || c == ':' || c == ' ') {
            continue;
        }
        if status
            .trim_start_matches(|c: char| !c.is_ascii_alphabetic())
            .starts_with("RETIRED")
        {
            continue;
        }
        let Some(requirement) = patterns.requirement.find(first) else {
            continue;
        };
        let requirement = requirement.as_str();
        // A parenthesised note ("(AC-6 withdrawn)") annotates; it declares nothing.
        let criteria = patterns.parenthetical.replace_all(criteria, " ");
        let mut ids = BTreeSet::new();
        let mut parsed = 0usize;
        let mut readable = true;
        for caps in patterns.bare.captures_iter(&criteria) {
            parsed += 1;
            let end = caps.get(0).map_or(0, |m| m.end());
            // `AC-1...3` or `AC-1-3`: a range marker the grammar did not take.
            if criteria[end..].starts_with(['.', '–', '—', '-']) {
                readable = false;
            }
            let last = caps.get(2).map(|m| m.as_str());
            readable &= expand(requirement, &caps[1], last, &mut ids);
            for item in caps[3]
                .split([',', ';'])
                .map(str::trim)
                .filter(|i| !i.is_empty())
            {
                let (first, last) = item
                    .split_once("..")
                    .map_or((item, None), |(a, b)| (a, Some(b)));
                readable &= expand(requirement, first, last, &mut ids);
            }
        }
        // Every `AC-n` mention must have been read (`ac-1..3` reads as none).
        if parsed != patterns.bare_mention.find_iter(&criteria).count() {
            readable = false;
        }
        if !readable {
            findings.push(AuditFinding::Unreadable {
                location: format!("{requirement} coverage row"),
                cell: (*criteria).trim().to_string(),
            });
            continue;
        }
        for id in ids {
            declared.insert(id, requirement.to_string());
        }
    }
    declared
}

/// A criterion row that is not a criterion: its statement opens with the
/// marker `RETIRED` (optionally parenthesised or bold), `Removed:`, is struck
/// through, or is the bare `Withdrawn` marker. A live statement that merely
/// begins with the word "Withdrawn" is not a tombstone.
fn is_tombstone(statement: &str) -> bool {
    let marker = statement.trim_start_matches(['(', '*', '~', ' ']);
    marker.starts_with("RETIRED")
        || marker.starts_with("Removed:")
        || statement.starts_with("~~")
        || marker.trim_end_matches('.') == "Withdrawn"
}

/// Criteria a requirement document defines, as `(live, all)`: a bold
/// `**FR-050-AC-1**` anchor or an `| FR-050-AC-1 |` leading table cell is a
/// definition; one whose statement is a tombstone is defined but not live.
fn defined_ids(document: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let patterns = patterns();
    let mut live = BTreeSet::new();
    let mut all = BTreeSet::new();
    let mut fenced = false;
    for line in document.lines() {
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some(cells) = cells(line) {
            if let [first, statement, ..] = cells.as_slice() {
                if let Some(caps) = patterns.exact.captures(first) {
                    let id = format!("{}-AC-{}", &caps[1], &caps[2]);
                    if !is_tombstone(statement) {
                        live.insert(id.clone());
                    }
                    all.insert(id);
                    continue;
                }
            }
        }
        for caps in patterns.bold.captures_iter(line) {
            // `**FR-x-AC-n**: RETIRED ...` is a tombstone on the bold path too.
            let rest = line.split_once(&caps[0]).map_or("", |(_, rest)| rest);
            let statement = rest.trim_start_matches([':', ' ']);
            if !is_tombstone(statement) {
                live.insert(caps[1].to_string());
            }
            all.insert(caps[1].to_string());
        }
    }
    (live, all)
}

/// Every audit-table finding for `matrix` against its requirement
/// `documents` (`(label, text)` pairs), sorted. Empty means the audit table
/// maps exactly the criteria that are declared and defined.
pub fn audit_gaps(matrix: &str, documents: &[(&str, &str)]) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let (mapped, retired_rows) = audit_ids(matrix, &mut findings);
    let ranges = coverage_ranges(matrix, &mut findings);

    let mut live = BTreeMap::<String, Vec<AuditSource>>::new();
    let mut defined = BTreeSet::new();
    for (label, text) in documents {
        let (document_live, document_all) = defined_ids(text);
        for ac in document_live {
            live.entry(ac).or_default().push(AuditSource::Definition {
                document: (*label).to_string(),
            });
        }
        defined.extend(document_all);
    }
    for (ac, requirement) in &ranges {
        if !defined.contains(ac) {
            findings.push(AuditFinding::Phantom {
                ac: ac.clone(),
                source: PhantomSource::CoverageRange {
                    requirement: requirement.clone(),
                },
            });
        } else if let Some(sources) = live.get_mut(ac) {
            // A range over a retired criterion needs no row, so only a live
            // one gains this declaration.
            sources.push(AuditSource::CoverageRange {
                requirement: requirement.clone(),
            });
        }
    }
    for (ac, sources) in live {
        if !mapped.contains(&ac) {
            findings.push(AuditFinding::Missing { ac, sources });
        }
    }
    for ac in &mapped {
        if !defined.contains(ac) && !retired_rows.contains(ac) {
            findings.push(AuditFinding::Phantom {
                ac: ac.clone(),
                source: PhantomSource::AuditRow,
            });
        }
    }
    findings.sort();
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tombstones_are_markers_not_leading_words() {
        assert!(is_tombstone("**RETIRED (render removal)** gone"));
        assert!(is_tombstone("(RETIRED) gone"));
        assert!(is_tombstone("Removed: the id is not reused"));
        assert!(is_tombstone("Withdrawn."));
        assert!(!is_tombstone("Withdrawn tokens are rejected"));
    }

    #[test]
    fn a_range_is_bounded_and_ordered() {
        let mut ids = BTreeSet::new();
        assert!(expand("FR-1", "2", Some("4"), &mut ids));
        assert_eq!(ids.len(), 3);
        assert!(!expand("FR-1", "5", Some("2"), &mut ids));
        assert!(!expand("FR-1", "1", Some("4000000"), &mut ids));
    }
}
