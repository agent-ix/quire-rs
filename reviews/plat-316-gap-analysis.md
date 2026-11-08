---
id: SR-16322
title: "Gap analysis of trace targets from list items and headings (quire-rs PR 603)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@ab57de1; spec/functional/FR-050-declarative-coverage-computation.md (FR-050-AC-60, CR-203), spec/tests.md (TC-2080), src/traceability.rs, src/corpus/declared_tables.rs, src/query.rs, tests/it/trace_target_sources.rs, tests/fixtures/corpus_cases/trace-target-sources.json"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-050
    type: references
---

# Gap analysis of trace targets from list items and headings (quire-rs PR 603)

## Summary

Ticket: PLAT-316. A planless gap analysis, scoped to the PR diff. It checks
FR-050-AC-60 against TC-2080's tagged tests and against the code that
implements the AC. Plan completion: not assessed. `quoin matrix --json` (quoin
0.29.1) reports `FR-050-AC-60` with method `Test`, `static_status: tagged`.

Four tests are tagged `#[trace("TC-2080", "FR-050-AC-60")]`. Three are
integration tests in `tests/it/trace_target_sources.rs`, driven by seven
fixture cases. The fourth is the lib unit test
`trace_target_source_round_trips_its_authored_shape`.

The AC clauses map to the tests as follows:

- **Omitted `from` behaves as `table_row`.** Backed by
  `omitted_from_is_table_row_and_reports_identically`, which byte-compares the
  two reports.
- **Archetype and section limits.** The fixture excludes a `Notes` section, an
  FR document's `Test Procedure`, and a US `Other` section.
- **Capture group 1 supplies the id.** The fixture checks this, using a prefix
  outside the group.
- **List-item and heading text.** Covered, including a nested list item and a
  `####` heading under a `###`.
- **Load-time rejection of a pattern that does not compile or has no group 1.**
  Covered by `malformed_non_table_sources_fail_module_load` over six cases.
- **Fenced code.** The basic case is covered. A nested fence is not (SR-16321
  FND-002).

TC-2080 was changed to ✅, and the evidence supports that.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | [coverage] The ordered-list marker branch (`\d+[.)]`) of `list_item_lines` never mints in any test. The fixture's only ordered item (`1. Run the search.`) carries no id, so a regression that drops ordered items from minting would pass. Add an id-bearing `1.` or `1)` item to the fixture. | src/query.rs:396-399; tests/fixtures/corpus_cases/trace-target-sources.json |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-050-AC-60: "A trace target's `from` selects `table_row`, `list_item`, or `heading`; when omitted it behaves as `table_row` … capture group 1 supplies the id. A non-table source declaration with a pattern that does not compile or has no capture group 1 is rejected at module load rather than yielding partial ids." | examined | Every clause has an assertion. The ordered-list branch is unexercised (FND-001). |
| TC-2080 → FR-050-AC-60 bindings (4 tests) | examined | Trace tags are correct. |
| CR-203 | context_only | Motivation (ISO IT scenarios, US examples) |
| Production code with no owning requirement | examined | None. All new code traces to AC-60. |

## Verdict

PASS with one low coverage gap. The acceptance criteria are backed by
behavioural tests. The two code defects found in SR-16321 should land together
with their own regression tests.

## Dispositions

Round 1, at head d7534d5 (fix commit d7534d5 checked against ab57de1).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The fixture adds `2. IT-002-SC-04` and `3) IT-002-SC-05`. Both are expected in `minted` at lines 16 and 17, and `trace_target_sources` passes at d7534d5, so both ordered-marker forms are now exercised. |
