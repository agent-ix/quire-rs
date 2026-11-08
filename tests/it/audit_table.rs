//! Audit-table completeness (FR-050-AC-58, CR-196): a declared criterion with
//! no row in the matrix's AC → TC Coverage Audit table is an error.

use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::audit_table::{audit_gaps, AuditGapSource};

const MATRIX: &str = "\
## Requirements Traceability

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-001 Thing | AC-1..3; CON-1 | TC-1 | done |
| FR-004 Listed | AC-1..2, 4 (AC-3 withdrawn) | TC-4 | done |
| ~~FR-002 Gone~~ | AC-1..9 | - | RETIRED |

## AC → TC Coverage Audit

| AC | Covering TC(s) |
|---|---|
| FR-001-AC-1 | TC-1 |
| FR-001-AC-2 | TC-1 |
| FR-001-AC-3 | TC-1 |
| FR-004-AC-1..2 | TC-4 |
| FR-004-AC-4 | TC-4 |
| FR-005-AC-1 | TC-5 |

## Coverage Gaps

| FR-009-AC-7 | not an audit row |
";

const FR_005: &str = "\
| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | parses | Test |
| FR-005-AC-2 | RETIRED — gone | Test |
| FR-005-AC-3 | Withdrawn. | — |
- **FR-005-AC-4**: bold anchor
";

fn without_row(matrix: &str, row: &str) -> String {
    let removed = format!("{row}\n");
    assert!(matrix.contains(&removed), "fixture lost its {row} row");
    matrix.replacen(&removed, "", 1)
}

/// The control: every declared criterion has a row; retired, withdrawn and
/// parenthetically-annotated ids declare nothing; a row outside the audit
/// section does not count.
#[trace("TC-2012", "FR-050-AC-58")]
#[test]
fn a_complete_audit_table_has_no_gaps() {
    let gaps = audit_gaps(MATRIX, &[]);
    assert!(gaps.is_empty(), "{gaps:?}");
}

/// Deleting one audit row is an error naming exactly that criterion and the
/// coverage row that declared it.
#[trace("TC-2012", "FR-050-AC-58")]
#[test]
fn a_deleted_audit_row_is_named() {
    let matrix = without_row(MATRIX, "| FR-001-AC-2 | TC-1 |");
    let gaps = audit_gaps(&matrix, &[]);
    assert_eq!(gaps.len(), 1, "{gaps:?}");
    assert_eq!(gaps[0].ac, "FR-001-AC-2");
    assert_eq!(
        gaps[0].source,
        AuditGapSource::CoverageRange {
            requirement: "FR-001".to_string()
        }
    );
    assert!(gaps[0].to_string().starts_with("FR-001-AC-2:"));
}

/// A list continuation (`AC-1..2, 4`) declares its later items, so dropping
/// the row for `4` is caught.
#[trace("TC-2012", "FR-050-AC-58")]
#[test]
fn a_list_continuation_declares_its_items() {
    let matrix = without_row(MATRIX, "| FR-004-AC-4 | TC-4 |");
    let gaps = audit_gaps(&matrix, &[]);
    assert_eq!(gaps.len(), 1, "{gaps:?}");
    assert_eq!(gaps[0].ac, "FR-004-AC-4");
}

/// A criterion a requirement document defines, with no coverage-row range
/// naming it, still needs its row. Tombstones define nothing.
#[trace("TC-2012", "FR-050-AC-58")]
#[test]
fn a_defined_criterion_without_a_row_is_named() {
    let documents = [("functional/FR-005.md", FR_005)];
    let gaps = audit_gaps(MATRIX, &documents);
    let named: Vec<&str> = gaps.iter().map(|gap| gap.ac.as_str()).collect();
    assert_eq!(named, ["FR-005-AC-4"], "{gaps:?}");
    assert_eq!(
        gaps[0].source,
        AuditGapSource::Definition {
            document: "functional/FR-005.md".to_string()
        }
    );
}

/// This repository's own matrix passes against its own requirement
/// documents: the check is not vacuous on real data and the corpus holds.
#[trace("TC-2012", "FR-050-AC-58")]
#[test]
fn this_repositorys_matrix_has_no_audit_gaps() {
    let spec = Path::new("spec");
    let matrix = fs::read_to_string(spec.join("tests.md")).expect("spec/tests.md");
    let mut documents = Vec::new();
    for directory in ["stakeholder", "usecase", "functional", "non-functional"] {
        for entry in fs::read_dir(spec.join(directory)).expect("requirement directory") {
            let path = entry.expect("directory entry").path();
            if path.extension().is_some_and(|extension| extension == "md") {
                let label = format!(
                    "{directory}/{}",
                    path.file_name().unwrap().to_string_lossy()
                );
                documents.push((
                    label,
                    fs::read_to_string(&path).expect("requirement document"),
                ));
            }
        }
    }
    let documents: Vec<(&str, &str)> = documents
        .iter()
        .map(|(label, text)| (label.as_str(), text.as_str()))
        .collect();
    let gaps = audit_gaps(&matrix, &documents);
    assert!(gaps.is_empty(), "{gaps:#?}");
}
