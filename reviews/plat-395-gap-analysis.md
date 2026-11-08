---
id: SR-16314
title: "Gap analysis of the imperative-head allowance against FR-047-AC-19 and TC-2030 (quire-rs PR 597)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@66459e84; src/grammar/ac.rs; spec/functional/FR-047-acceptance-criteria-grammar.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-047
    type: reviews
---

# Gap analysis of quire-rs PR 597

## Summary

Ticket: PLAT-395. Planless gap analysis of the change set. Plan completion: not assessed.
`quire matrix --scope .` (quire 0.36.2, engine 0.50.2) was run on the PR head. FR-047-AC-19
is tagged by `tc2030_imperative_head_is_a_predicate` (ac.rs:1503). That test asserts every
example AC-19 names: three imperatives classify `assertion` with 0 findings, and `Structural
evaluation`, `Task ID`, `Type Check`, `Add` and `Attach job` classify `unstructured` with 1
finding each. FR-047-AC-15 regressed to `untagged`. Focused `grammar::ac` tests pass
(20 run), and `cargo test --test it` passes (527 of 527).

## Verdict

**CHANGES REQUIRED.** AC-19 is backed by a real test. AC-15 lost its only binder.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-047-AC-15 went from tagged to `untagged` in the computed matrix, because the TC-775 test's trace was edited to name FR-047-AC-19 (see SR-16313 FND-001). The AC-15 behaviour is still tested, but the matrix no longer shows it. Restore `#[trace("TC-775", "FR-047-AC-15")]`. | src/grammar/ac.rs:1001; spec/tests.md:647 |
| FND-002 | low | TC-2030 has no Title-Case heading among its negative cases (for example `Add Item To Cart`), so the shape SR-16313 FND-003 shows false-passing is untested. Add it once the regex rejects it. | src/grammar/ac.rs:1516-1526 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (head 155b534): `quire matrix` shows FR-047-AC-15 tagged by ac.rs:1011 and FR-047-AC-19 tagged by TC-2070 (ac.rs:1509). |
| FND-002 | fixed | Round 1: TC-2070 negative cases include `Add Item To Cart` (one `unclassifiable` finding, `unstructured`). |
