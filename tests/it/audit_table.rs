//! Audit-table completeness (FR-050-AC-58, CR-200): a declared criterion with
//! no row in the matrix's AC → TC Coverage Audit table is an error, and so is
//! a row or range id that no requirement document defines.

use ix_trace_rs::trace;
use quire_rs::audit_table::{audit_gaps, AuditFinding, AuditSource, PhantomSource};

const MATRIX: &str = "\
## Requirements Traceability

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-001 Thing | AC-1..3; CON-1 | TC-1 | done |
| FR-004 Listed | AC-1..2, 4 (AC-3 withdrawn) | TC-4 (⛔ RETIRED), TC-5 | done |
| ~~FR-002 Gone~~ | AC-1..9 | - | RETIRED |
| FR-006 Struck | AC-1..2 | - | ⛔ RETIRED |

### Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Status |
|---|---|---|---|
| StR-003 Parse parity | US-002 | TC-20 | done |

## AC → TC Coverage Audit

### Stakeholder Requirements

| AC | Covering TC(s) |
|---|---|
| StR-003-AC-1 | TC-20 |
| StR-005-AC-2 | TC-21 |

### Functional Requirements

| AC | Covering TC(s) |
|---|---|
| FR-001-AC-1 | TC-1 |
| FR-001-AC-2 | TC-1 |
| FR-001-AC-3 | TC-1 |
| FR-004-AC-1..2 | TC-4 |
| FR-004-AC-4 | TC-4 |
| FR-005-AC-1 | TC-5 |
| FR-007-AC-1 | RETIRED (superseded) |

### Gaps summary

| Gap | Note |
|---|---|
| FR-009-AC-7 | not an audit row |
";

const DOCUMENTS: [(&str, &str); 3] = [
    (
        "functional/FR-005.md",
        "| ID | Criteria | Verification |\n|----|----|----|\n| FR-005-AC-1 | parses | Test |\n\
         | FR-005-AC-2 | RETIRED — gone | Test |\n| FR-005-AC-3 | Withdrawn. | — |\n",
    ),
    (
        "functional/FR-007.md",
        "- **FR-007-AC-1**: RETIRED (superseded) gone\n| FR-007-AC-2 | Removed: the id is not reused | - |\n",
    ),
    (
        "stakeholder/StR.md",
        "| StR-003-AC-1 | x | Test |\n| StR-005-AC-2 | y | Test |\n",
    ),
];

fn documents() -> Vec<(&'static str, &'static str)> {
    let mut documents = DOCUMENTS.to_vec();
    documents.push((
        "functional/FR-001.md",
        "- **FR-001-AC-1**: a\n- **FR-001-AC-2**: b\n- **FR-001-AC-3**: c\n",
    ));
    documents.push((
        "functional/FR-004.md",
        "| FR-004-AC-1 | a | T |\n| FR-004-AC-2 | b | T |\n| FR-004-AC-4 | c | T |\n",
    ));
    documents
}

fn without_row(matrix: &str, row: &str) -> String {
    let removed = format!("{row}\n");
    assert!(matrix.contains(&removed), "fixture lost its {row} row");
    matrix.replacen(&removed, "", 1)
}

fn subjects(findings: &[AuditFinding]) -> Vec<&str> {
    findings.iter().map(AuditFinding::subject).collect()
}

/// The control: every declared criterion has a row; tombstones, struck-through
/// and RETIRED-status rows, parenthetical notes, a `###` gaps table and a
/// RETIRED word in a live row's test cell declare or map nothing wrongly.
#[trace("FR-050-AC-58")]
#[test]
fn a_complete_audit_table_has_no_findings() {
    let findings = audit_gaps(MATRIX, &documents());
    assert!(findings.is_empty(), "{findings:?}");
}

/// Deleting one audit row is one error naming exactly that criterion, even
/// though a coverage row and a document both declare it.
#[trace("FR-050-AC-58")]
#[test]
fn a_deleted_audit_row_is_one_error_naming_it() {
    let matrix = without_row(MATRIX, "| FR-001-AC-2 | TC-1 |");
    let findings = audit_gaps(&matrix, &documents());
    assert_eq!(subjects(&findings), ["FR-001-AC-2"], "{findings:?}");
    let AuditFinding::Missing { sources, .. } = &findings[0] else {
        panic!("expected Missing: {findings:?}");
    };
    assert!(sources.contains(&AuditSource::CoverageRange {
        requirement: "FR-001".to_string()
    }));
    assert!(sources.contains(&AuditSource::Definition {
        document: "functional/FR-001.md".to_string()
    }));
    assert!(findings[0].to_string().starts_with("FR-001-AC-2:"));
}

/// Mixed-case `StR-` ids are read whole: deleting a stakeholder row fails.
#[trace("FR-050-AC-58")]
#[test]
fn a_deleted_stakeholder_row_is_named() {
    for row in ["| StR-003-AC-1 | TC-20 |", "| StR-005-AC-2 | TC-21 |"] {
        let findings = audit_gaps(&without_row(MATRIX, row), &documents());
        let expected = row.split(' ').nth(1).unwrap();
        assert_eq!(subjects(&findings), [expected], "{findings:?}");
    }
}

/// A list continuation declares its later items; a document-defined criterion
/// from a leading table cell is named; a tombstone never needs a row.
#[trace("FR-050-AC-58")]
#[test]
fn continuations_and_table_cell_definitions_are_read() {
    let matrix = without_row(MATRIX, "| FR-004-AC-4 | TC-4 |");
    let findings = audit_gaps(&matrix, &documents());
    assert_eq!(subjects(&findings), ["FR-004-AC-4"], "{findings:?}");

    let matrix = without_row(MATRIX, "| FR-005-AC-1 | TC-5 |");
    let findings = audit_gaps(&matrix, &documents());
    assert_eq!(subjects(&findings), ["FR-005-AC-1"], "{findings:?}");
}

/// A live statement that merely begins `Withdrawn` is a live criterion.
#[trace("FR-050-AC-58")]
#[test]
fn a_live_withdrawn_sentence_needs_a_row() {
    let mut documents = documents();
    documents.push((
        "functional/FR-009.md",
        "| FR-009-AC-1 | Withdrawn tokens are rejected | Test |\n",
    ));
    let findings = audit_gaps(MATRIX, &documents);
    assert_eq!(subjects(&findings), ["FR-009-AC-1"], "{findings:?}");
}

/// An id that no document defines is an error from either side.
#[trace("FR-050-AC-58")]
#[test]
fn a_phantom_id_is_an_error() {
    let matrix = MATRIX.replace(
        "| FR-001-AC-3 | TC-1 |",
        "| FR-001-AC-3 | TC-1 |\n| FR-001-AC-99 | TC-1 |",
    );
    let findings = audit_gaps(&matrix, &documents());
    assert_eq!(
        findings,
        [AuditFinding::Phantom {
            ac: "FR-001-AC-99".to_string(),
            source: PhantomSource::AuditRow
        }]
    );

    let matrix = MATRIX.replace("| FR-001 Thing | AC-1..3;", "| FR-001 Thing | AC-1..4;");
    let findings = audit_gaps(&matrix, &documents());
    assert_eq!(subjects(&findings), ["FR-001-AC-4"], "{findings:?}");
    assert!(matches!(
        findings[0],
        AuditFinding::Phantom {
            source: PhantomSource::CoverageRange { .. },
            ..
        }
    ));
}

/// A criteria cell the check cannot read in full is an error, never a smaller
/// declaration; a runaway range errors once and quickly.
#[trace("FR-050-AC-58")]
#[test]
fn an_unreadable_criteria_cell_is_an_error() {
    for cell in ["AC-1...3", "ac-1..3", "AC-5..2", "AC-1..4000000", "AC-1-3"] {
        let matrix = MATRIX.replace(
            "| FR-001 Thing | AC-1..3;",
            &format!("| FR-001 Thing | {cell};"),
        );
        let findings = audit_gaps(&matrix, &documents());
        assert!(
            findings
                .iter()
                .any(|finding| matches!(finding, AuditFinding::Unreadable { .. })),
            "{cell}: {findings:?}"
        );
    }
    // The long form `AC-1..AC-3` is read whole: its middle row is still required.
    let long = MATRIX.replace("| FR-001 Thing | AC-1..3;", "| FR-001 Thing | AC-1..AC-3;");
    assert!(audit_gaps(&long, &documents()).is_empty());
    let findings = audit_gaps(&without_row(&long, "| FR-001-AC-2 | TC-1 |"), &documents());
    assert_eq!(subjects(&findings), ["FR-001-AC-2"], "{findings:?}");
}
