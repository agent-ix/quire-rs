---
id: SR-20261103118118
title: "PLAT-217 and PLAT-1056 row-width assert and foreign ix authority gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs PR590 head 677dbd60; FR-033-AC-17; FR-038-AC-9; FR-038-AC-3; TC-1991; TC-1992; TC-602"
review_set: subset
---

## Summary

Tickets: PLAT-217, PLAT-1056. Gap analysis scoped to the PR diff, planless. FR-033-AC-17 is tagged by `tc1991_row_width_must_match_the_header` (`#[trace("TC-1991", "FR-033-AC-17")]`) and FR-038-AC-9 by `foreign_authority_target_is_not_dangling` (`#[trace("TC-1992", "FR-038-AC-9")]`); both pass. Both bindings point at the right criterion. Every implemented path has an owning AC (FR-033-AC-17 for `AssertReason::RowWidth` and `ValidationReason::RowWidth`; FR-038-AC-9 for `Edge::authority` and the skip). No stubs. The gaps are in what the tagged tests leave unexercised, plus one rebound fixture. Plan completion: not assessed.

## Verdict

Gaps found. TC-1991 covers too-few and too-many rows, row lines, row ids and a uniform control. It does not cover the escaped-pipe and empty-trailing-cell cases that PLAT-217 names. TC-1992 never runs the AC-9 main clause (a declared package that differs from the authority) or Strict. TC-602 still claims an `ix://` dangling reference but no longer contains one.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | TC-1991 has no row with an escaped `\|` inside a cell and no row with an empty trailing cell (`\| A-4 \| \|`). These are the main false-positive risks of re-reading widths from the source line, and PLAT-217 requires that `\|` count as one cell (the process module's `Verdict` column authors `Pass\|Fail\|Waived`). Fix: add both rows to the fixture and assert that neither raises `row-width`. | src/extract/assert_eval.rs:722-753 | correct-requirement-no-evidence |
| FND-002 | medium | TC-1992 never runs FR-038-AC-9's main clause. The first half declares no package, and the second half declares a package equal to the link's authority. The case "package declared, authority different" is untested, and so is Strict, where a dangling ref is an error. Fix: declare package `agent-ix/own`, link `ix://other-org/other-repo/X`, and assert no `dangling-reference` under Strict and Okf, plus an own-authority miss under Strict reported as an error. | tests/it/bundle_validate.rs:684-707 | correct-requirement-no-evidence |
| FND-003 | medium | TC-602 (`okf_tolerates_unknown_type_and_broken_links`, tagged FR-038-AC-3) keeps its doc "a dangling `ix://` reference", but its fixture was changed from `ix://o/r/MISSING` to a bare-id relationship. It no longer tests what FR-038-AC-3 states. The rebind (and the same switch in TC-884/886/887/889) hides that a package-less `ix://` miss is no longer reported. Fix: once the code-review FND-001 fix lands, restore an `ix://` dangling fixture in TC-602. | tests/it/bundle_validate.rs:90-101 | correct-requirement-no-evidence |
