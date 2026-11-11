---
id: SR-20272
title: "Gap analysis — PLAT-1392 row binding constrains only when matrix rows exist (quire-rs PR 642)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-050-declarative-coverage-computation.md (AC-3), spec/tests.md (TC-734), src/coverage/row_binding.rs, tests/it/coverage_rollup.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-050", type: references}
---

## Summary

Ticket: PLAT-1392. PR head: `412a0ab`. Plan completion: not assessed. The changed code is reached through `coverage::compute`. The only test that exercises the new fixture case is TC-734 (`a_criterion_tag_only_backs_its_own_matrix_row`), which is tagged FR-050-AC-3.

## Verdict

**FAIL.** The regression case is not executed (see SR-20271 FND-001). The no-matrix and Inspections-only behaviours are stated in no acceptance criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No acceptance criterion states the row-binding rule: that a criterion tag backs only with a same-test tag of a matrix row citing it, and that a repo with no matrix rows binds directly. FR-050-AC-3, the AC TC-734 is tagged with, covers unbacked reference rows and obligation own-ID backing. It says nothing about `criterion-tagged-outside-row`. So PLAT-2 and this PR's carve-out are behaviour with no owning AC, and the Inspections case (SR-20271 FND-002) has no AC to judge it against. Add an FR-050 AC that states when row binding applies (matrix rows present), when it does not (none), and which declarations count as a matrix. Tag TC-734 with it. | spec/functional/FR-050-declarative-coverage-computation.md:395; src/coverage/row_binding.rs:66-69 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-050-AC-3 | examined | Does not cover row binding; FND-001 |
| TC-734 a_criterion_tag_only_backs_its_own_matrix_row | examined | Fails on PR head (SR-20271 FND-001) |
| src/coverage/row_binding.rs | examined | No owning AC for the row-binding rule |
