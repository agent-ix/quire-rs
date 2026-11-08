---
id: SR-16310
title: "PLAT-321 audit-table completeness code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/audit_table.rs; src/lib.rs; examples/spec_validate.rs; tests/it/audit_table.rs; tests/it/main.rs; PLAT-321; PR598"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-050", type: references}
---

# PLAT-321 audit-table completeness code-review

## Summary

Ticket: PLAT-321. PR agent-ix/quire-rs#598, head 6fe84930, diff `origin/main...HEAD`
(7 files, +604/-26). This review includes the Rust lane (rust-review). Read-only
review in a scratch clone.

Evidence the reviewer ran: `cargo test --test it` (532 passed),
`cargo test --lib audit_table` (3 passed), and `examples/spec_validate` against
the real spec tree. The unmodified tree gives 0 audit-table errors. Deleting the
audit row for FR-074-AC-5, US-003-AC-1, NFR-021-AC-1 or FR-050-AC-58 fails with
an error naming the criterion. Deleting StR-003-AC-1 or StR-005-AC-2 stays green.
A scratch example ran `audit_gaps` over 16 constructed edge cases. The results are
in the findings below.

## Examined Units

- **FR-050-AC-58** (examined), spec/functional/FR-050-declarative-coverage-computation.md:292. The audit table must have a row for every criterion that a coverage-row range declares or a requirement document defines. A missing row is one error.
- **src/audit_table.rs** (examined): `audit_table_ids`, `coverage_range_ids`, `defined_ids`, `is_tombstone`, `audit_gaps`, `expand`, `section_lines`.
- **examples/spec_validate.rs** (examined): `audit_table_gaps` and its wiring into `failed_documents`.
- **tests/it/audit_table.rs** (examined): five TC-2012 tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | [soundness] Stakeholder criteria are invisible to the check. Both the requirement-id pattern `[A-Z]+-\d+` and the bold-anchor pattern reject the mixed-case `StR-` prefix. Because the qualified pattern has no anchor, the audit cell `StR-003-AC-1` is read as `R-003-AC-1`. As a result, `StR-*` coverage rows, `**StR-nnn-AC-n**` definitions and `StR-*` audit rows all contribute nothing. Reproduced: deleting the `StR-003-AC-1` or `StR-005-AC-2` audit row and running `examples/spec_validate` reports no audit-table error. This is the exact missing-row-stays-green defect that AC-58 closes, open for the whole `spec/stakeholder` directory, which the check claims to read. Fix: accept `[A-Za-z]+` (or the module's id shape) in all three patterns, anchor the qualified match at a token boundary, and add a StR case to TC-2012. | src/audit_table.rs:56; src/audit_table.rs:75; src/audit_table.rs:193 |
| FND-002 | medium | [soundness] A live coverage row declares nothing if any later cell contains `RETIRED`. Real instance: the FR-028 row (`AC-1..8`) has `TC-510 (⛔ RETIRED — render removed)` in its Test Cases cell, so its range is skipped. FR-028's definitions mask it today. Constructed `\| FR-001 \| AC-1..3 \| TC-1 (⛔ RETIRED), TC-2 \| ✅ \|` with only AC-1 audited gives 0 gaps. Fix: judge retirement from the struck-through first cell or the status cell, not from any cell. | src/audit_table.rs:143; spec/tests.md:86 |
| FND-003 | medium | [soundness] Criteria-cell forms the parser does not recognise silently declare less, and no error reports it. `AC-1..AC-3` declares only 1 and 3, so a missing AC-2 row stays green. `AC-1–3` (en dash) declares only 1. `ac-1..3` declares nothing. Reversed `AC-5..2` declares only 5. Every case was reproduced with 0 gaps. Fix: refuse, with an error, a criteria cell whose AC tokens do not all parse, and refuse a reversed range. Do not narrow silently. | src/audit_table.rs:63; src/audit_table.rs:85-93 |
| FND-004 | medium | [soundness] The tombstone test (`~~`, `RETIRED`, `(RETIRED`, `Withdrawn` at statement start) misses forms this corpus uses: `**RETIRED (...)**` (NFR-006-AC-1) and `Removed:` (FR-071-AC-9). The gate therefore demanded audit rows for retired criteria, and the PR added them as live mappings (see the spec-review SR). The opposite error also exists: a live statement that begins with the word `Withdrawn` (`Withdrawn tokens are rejected`) is treated as a tombstone and needs no row (reproduced). A bold `**FR-x-AC-n**: RETIRED` anchor is never a tombstone. Fix: match the corpus's tombstone forms exactly, as a marker rather than a leading word, and apply the same rule on the bold path. | src/audit_table.rs:181-185; src/audit_table.rs:210-212 |
| FND-005 | low | [code-bug] AC-58 and the PR body say a missing row is one error. The code emits one gap per source, so a criterion that both a coverage row and a document declare gives two errors and counts as two failed documents. Reproduced with FR-074-AC-5, US-003-AC-1 and FR-050-AC-58. Fix: dedupe by criterion and name both sources in one error, or reword AC-58. | src/audit_table.rs:220-245; examples/spec_validate.rs:335-339 |
| FND-006 | low | [code-bug] Range expansion has no bound. A typo `AC-1..4000000` produced 4,000,000 gaps and took 19 s. Fix: cap a range, or refuse one that extends past the requirement's defined criteria. | src/audit_table.rs:90 |
| FND-007 | low | [soundness] Section tracking is by `##` headings only. A `###` subsection inside the audit section (for example a gaps table) has its rows counted as mapped (reproduced, 0 gaps), and a `## ` line inside a code fence toggles sections. The current file does not trigger either case. Fix: restrict to the audit tables (skip fenced lines), or document the restriction. | src/audit_table.rs:101-112 |
| FND-008 | low | [other] `audit_table_gaps` drops directory-entry errors through `entries.flatten()`, so an unreadable entry shrinks the population silently. The same four-directory walk is also written a second time in the TC-2012 repo test. Fix: propagate entry errors, and share one loader between the example and the test. | examples/spec_validate.rs:170; tests/it/audit_table.rs:110-134 |

## Verdict

Not mergeable. The structure is sound: a pure text function, one wiring point,
real deletion tests, and a repo-level test that keeps the corpus honest. But
FND-001 leaves a whole requirement family outside the completeness gate, which is
the defect class the ticket closes. FND-002 to FND-004 are silent narrowings of
the declared population, where the gate stays green. Clean parts: list
continuations (`AC-1..2, 4`), parenthetical stripping, `CON-` exclusion, audit
range rows, two ACs in one audit cell, struck-through and `RETIRED` status rows,
and the example's error routing all behave as AC-58 states.
