---
id: SR-20261115183664
title: "Gap analysis of PLAT-1441 implementation PR #690"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-058-upward-trace-completeness.md, spec/functional/FR-069-semantic-module-contract-at-load.md, spec/functional/FR-081-record-sections-extraction.md, spec/functional/FR-084-two-way-relation-coverage-report.md; tests/it/frontmatter_owned_keys.rs, tests/it/record_carry.rs (PR #690, head 63152cd)"
review_set: all
---

## Summary

Ticket: PLAT-1441. Planless; plan completion was not assessed. The computed matrix
(`quire matrix --scope . --json`) and `quoin --version` were both blocked by the permission
layer, so the AC-to-test map was checked by hand. The method was a `#[trace]` grep over
`tests/` and `src/`, followed by reading each tagged test against its AC text.

Every new or amended AC has a tagged test or is Inspection, except FR-084-AC-23 (FND-001).
Inspection results:

- FR-069-AC-46 and FR-084-AC-29 pass. `src/` names none of `regulatedCategories`, `health`,
  `personal` or `financial`; the only `health` hit is a grammar test string in
  `src/grammar/property.rs`.
- No production code is unowned. Each new function cites its FR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-084-AC-23 (backing spec-objects-architecture FR-013) has no tagged test. No fixture carries `record_keys: [regulatedCategories]` over a record stating `level` and asserts that `level` is absent at every depth. | tests/it/record_carry.rs:1 |
| FND-002 | low | FR-084-AC-28 "lists its connections by repo then id" is never exercised. Each `cls-used` row in the fixture has at most one linked connection, so the order is not asserted. | tests/it/record_carry.rs:221 |
| FND-003 | low | FR-058-AC-37 requires identical `trace:<check>` findings with and without `record_keys`, but the test compares only report row states and never the bundle trace findings. | tests/it/record_carry.rs:255 |
| FND-004 | low | FR-084-AC-21's second clause, a classification with an empty record carries no `record`, is untested. CLS-002 states `lvl`, and the unlinked CLS-004 is never checked. | tests/it/record_carry.rs:129 |
| FND-005 | low | In the FR-069-AC-40 test, the `[a, a]` and scalar `a` cases use the needle `cats`, which every finding already contains. The test therefore cannot tell a `uniqueItems` or not-a-list failure from any other failure. | tests/it/frontmatter_owned_keys.rs:124 |

## Verdict

Changes requested (medium: FND-001). Bindings verified correct:

- FR-069-AC-39..45 and FR-081-AC-18..21 in `frontmatter_owned_keys.rs`;
- FR-084-AC-20/22/24/25/26/27 and FR-058-AC-36 in `record_carry.rs`;
- the amended FR-069-AC-38 (`located_findings.rs:888`) and FR-081-AC-6
  (`semantic_record_sections.rs:337`).

Related, recorded under code-review SR-20261115183663 FND-001: no test exercises the
`object:` branch of `selected_record`.
