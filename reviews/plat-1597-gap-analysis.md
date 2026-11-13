---
id: SR-20261113104843
title: "Gap analysis - PLAT-1597 criticality_field on obligation sources (quire-rs PR 676)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@2930079; spec/functional/FR-053-obligation-record.md (AC-24..26), spec/functional/FR-082-coverage-matrix-criticality.md (AC-7), src/obligation.rs, src/obligation/inventory.rs, tests/it/obligation_criticality.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-053", type: references}
---

# Gap analysis - PLAT-1597 criticality_field on obligation sources (quire-rs PR 676)

## Summary

Ticket: PLAT-1597. PR head `2930079`. Plan completion: not assessed (planless). `quire trace` shows
these claims: FR-053-AC-24 has one test (`tc_rating_reaches_every_criterion_and_absence_stays_absent`),
FR-053-AC-25 has one (`tc_column_cell_wins_over_the_document_rating`), and FR-053-AC-26 has two
(`tc_rating_is_not_inferred_and_undeclared_sources_are_unchanged`,
`tc_coverage_json_carries_the_rating_and_validates`). All four pass.

## Verdict

**PASS with findings (2 medium, 1 low).** Every new AC has a tagged, passing test, and no stub or
inflated coverage was found. AC-24's string, null, blank and non-string cases are each asserted on
`derive`. AC-25 is asserted on both `derive` and `for_document`. AC-26's JSON-and-schema clause is
asserted on the real coverage report. Gaps: the selected-inventory production path has no test, and
two AC-26 clauses are asserted more narrowly than they are stated.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The `SelectedInventory` path is changed but has no test. It passes `document_rating` into `row_obligation`, yet no test builds a selected inventory with `criticality_field` declared. Dropping that argument (passing `None`) would still pass every test. Fix: add one selected-inventory case, tagged FR-053-AC-24, asserting a rated row and an unrated row. | src/obligation/inventory.rs:1188-1195, tests/it/obligation_criticality.rs |
| FND-002 | medium | Two FR-053-AC-26 clauses are asserted more narrowly than stated. "By any relationship type" is tested with `traces_to` only, while FR-082-AC-5 lists `traces_to`, `satisfies`, `implements` and `refines`. "Yields records identical to those before this criterion" is checked only as `criticality == None` on one row, not as whole-record equality. Fix: loop over the four relationship verbs, and compare the full undeclared-source output with the output for the same source declaring an absent key. | tests/it/obligation_criticality.rs:138-167 |
| FND-003 | low | AC-25's "the same holds on the `obligation` of a property classification record" is asserted on `for_document`, the function `classify_document_criteria` calls. It is not asserted on a classification record (`AcClassification::obligation`) itself. The binding holds today because the call is direct, but a record-level assertion would bind the stated surface. | tests/it/obligation_criticality.rs:131-135, src/validate_document.rs:270-280 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-053-AC-24 | examined | Bound (derive); selected inventory unbound (FND-001) |
| FR-053-AC-25 | examined | Bound (derive, for_document); FND-003 |
| FR-053-AC-26 | examined | Bound; clauses narrowed (FND-002) |
| FR-082-AC-7 | context_only | Existing tests unchanged; still pass |
| src/obligation.rs row_criticality / document_rating | examined | Traced to AC-24/25 through tests |
| src/obligation/inventory.rs | examined | FND-001 |
