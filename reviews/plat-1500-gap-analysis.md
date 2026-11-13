---
id: SR-20261113091247
title: "Gap analysis — PLAT-1500 every table under a declared section (quire-rs PR 671)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@0191395; spec/functional/FR-049-verification-reference-integrity.md (AC-23..30), src/corpus/declared_tables.rs, src/obligation.rs, src/obligation/inventory.rs, tests/it/coverage_rollup.rs, tests/it/located_findings.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-049", type: references}
---

## Summary

Ticket: PLAT-1500. This is a planless gap analysis of the PR diff. Plan completion was not assessed. `quire matrix --format tsv` (quire 0.44.0, engine 0.59.1) shows FR-049-AC-23 to AC-30 as all `tagged`. AC-26 and AC-30 are tagged by tc2322. AC-28 is tagged by tc2323 and tc2326, and AC-29 by tc2323. Each binding was read against its criterion. A mutation (first table only) confirmed which assertions discriminate.

## Verdict

**PASS with gaps: one medium, two low.** AC-26, AC-28 (via tc2326) and AC-29 are backed by tests that fail under the old behaviour. Two of AC-30's three clauses have no test. The trace tags on the new tests use the repo's `#[trace("FR-…")]` convention, and no stub or coverage inflation was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Two of the three clauses of FR-049-AC-30 have no test. tc2322 tests only "a later table lacking the column raises none", through a legend grid. No test covers "a section whose first table lacks the column while a later table carries it raises none". No test covers "a section where no table carries it raises exactly one" with more than one table under the heading. A review probe showed that both currently hold for the id and status findings, but nothing pins them. | tests/it/coverage_rollup.rs:4214-4265 |
| FND-002 | low | Fenced-table exclusion (FR-049-AC-27, now applied to every reader by the FR-049 body) is asserted for the reference scan (tc2325) and for minting (tc2326 control). It is not asserted for obligation derivation or for the rollup's `unbacked_rows`. The code path is shared (`tables_in`), so the risk is low. | src/corpus/declared_tables.rs:917-925; tests/it/located_findings.rs:445-501 |
| FND-003 | low | A combinatorial obligation (FR-061, `combinatorial_obligation`) now builds its configuration space from every table under the section, because `rows_of` widened. A second table with the dimension and values columns joins the space. FR-061 speaks of "the whole table", and FR-049's new paragraph does not name combinatorial sources. The widening is unspecified and untested. | src/obligation.rs:213 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-049-AC-26 | examined | Backed by tc2322 (discriminating) |
| FR-049-AC-27 | examined | FND-002 |
| FR-049-AC-28 | examined | Backed by tc2326 (discriminating); tc2323's AC-28 half is not discriminating (code-review FND-003) |
| FR-049-AC-29 | examined | Backed by tc2323 (discriminating) |
| FR-049-AC-30 | examined | FND-001 |
| FR-049-AC-23..25 | context_only | Unchanged; tests unchanged and passing |
