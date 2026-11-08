---
id: SR-16308
title: "gap-analysis of the FR-075 single population extent (quire-rs PR 591)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@62c592a9; spec/functional/FR-075-model-feature-extraction.md; spec/tests.md; src/semantic/model.rs; tests/it/semantic_model.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-075
    type: references
---

# Gap analysis of the FR-075 single population extent (quire-rs PR 591)

## Summary

Ticket: PLAT-8. Scoped to the PR diff. Plan completion: not assessed.
Checked FR-075-AC-4 (edited) against TC-1843 and its two tagged tests,
`population_members_with_one_extent` and
`population_extent_open_and_required`. `cargo test --test it` passes at the
head (524).

Examined, clean:

- AC-4's closed, open, missing and invalid `extent` cases are each asserted.
  The member `TypeRef` and the member row spans are asserted.
- Both tests are tagged `TC-1843`/`FR-075-AC-4`. The tags are correct.
- AC-7's schema validation runs on every record through the `extract`
  helper.

The ticket's main-vs-head quoin non-regression diff (advise, assurance and
validate over the quoin, FCD and SOA corpora) has not been run. The PR body
says so.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [coverage] `extentSpan` is required output, but no test asserts it. The tests check only the `extent` value. A wrong line, such as the `unwrap_or(1)` fallback in `frontmatter_features`, would still pass. Fix: `assert_span(&record["model"]["population"]["extentSpan"], ...)` at the `extent:` line. | tests/it/semantic_model.rs:415-485; src/semantic/model.rs:933, 1403-1407 | correct-requirement-no-evidence |
| FND-002 | medium | [coverage] Two cases are untested. The first is the transitional case every consumer is in today: a manifest that still declares `columns: [Type, Extent]`. Under that manifest the old table is dropped silently (SR-16307 FND-001), and a test would have shown it. The second is `extent:` on an artifact whose type declares no population table (the AC-6 analogue, SR-16307 FND-002). Fix: add both cases to TC-1843 or TC-1845, with the refusal they should produce. | tests/it/semantic_model.rs:437-485 | correct-requirement-no-evidence |

## Verdict

AC-4's main path is covered. The gaps are the two cases where the
implementation is weak, so they block together with SR-16307.

## Dispositions

Round 1, at head 2337e140. `cargo test --test it`: 526 passed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `population_members_with_one_extent` asserts `extentSpan` at the `extent:` line. |
| FND-002 | fixed | `legacy_population_table_is_refused_under_either_manifest` (TC-1843) covers the `[Type, Extent]` manifest. `extent_without_a_declared_population_is_refused` (TC-1845) covers `extent` with no declared population and with no `Type` table. |
