---
id: SR-20261112221721
title: "PLAT-1501 quire-rs PR662 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs PR662 head 88d6e7b; computed matrix (quire 0.43.0, engine 0.58.0) origin/main vs PR head; TC-only #[trace] and @pytest.mark.trace markers in src/**, tests/**"
review_set: subset
---

## Summary

Ticket: PLAT-1501. Planless gap analysis over the computed matrix. Plan completion: not assessed. `quire matrix --scope . --format tsv` was compared per (document, criterion) on the same 1192 criteria. origin/main has 746 tagged, 335 untagged and 112 method-without-symbol. The PR head has 816, 265 and 112. No criterion moves from tagged to another status, and 70 move from untagged to tagged.

There are 68 Rust `#[trace("TC-n")]` markers, excluding 3 fixture-payload markers and 1 comment. Together with 19 Python `@pytest.mark.trace("TC-n")` decorators, they name only TC ids. TC ids are no longer minted, so these markers bind nothing. They also produce no diagnostic: TC ids appear in no PR-head matrix warning except in `corpus/` fixtures. They are silent dead markers. On origin/main, `spec/tests.md` gave a row for 72 of their 73 distinct TC ids, and each row named its criteria. The exception is TC-882, which has no criterion. Those rows name 81 criteria, and 34 of them are `untagged` on the PR head, for example FR-024-AC-1 (TC-470), FR-077-AC-1 (TC-1887), FR-033-AC-14 (TC-770) and FR-027-AC-1..3 (TC-493..495).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | 87 TC-only trace markers (68 Rust, 19 Python) bind nothing and produce no diagnostic. The deleted `spec/tests.md` was the only record of which criterion each one verifies, and 34 of those criteria now compute untagged. Map each marker to the criteria named in its origin/main `spec/tests.md` row's Requirement column, check each mapping against what the test asserts, and drop the TC id. Delete the TC-882 marker, whose row names no criterion. Keep the 3 `tests/fixtures/**` markers, which are payload. | src/filament.rs:1247, src/symbols/trace_search.rs:362, src/corpus/query.rs:165, src/corpus/walk.rs:499, tests/it/markdown_validation.rs:454, tests/python/test_bindings.py:22, tests/python/test_semantic.py:32 |

## Verdict

The PR loses no tagged criterion and gains 70. But deleting the matrix turns 87 markers into silent dead text while their criterion mapping still exists only in git history. Map them in this PR, which would cover up to 34 more criteria, instead of leaving them as markers that bind nothing.
