---
id: SR-16313
title: "Code review (with Rust lane) of the has_predicate doc fix and imperative-head allowance (quire-rs PR 597)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@66459e84; src/grammar/ac.rs"
review_set: subset
---

# Code review of quire-rs PR 597

## Summary

Ticket: PLAT-395. Code-review plus rust-review lane over `git diff origin/main...HEAD`
for `src/grammar/ac.rs`: the corrected `has_predicate` doc comment, the new
`re_imperative_head` disjunct, the TC-2030 unit test, and a changed `#[trace]` on the
TC-775 test.

The imperative rule's effect was measured, which the PR had not done. The frame was built
with `scripts/ac_corpus_sweep.py`'s `iter_docs` (FR/NFR/StR, its `SKIP_DIRS`/worktree rules),
excluding nested `corpus/`, `.cargo-*`, `qa-corpus*` and `-task`/`-pr`/`-wave` copies. That gave
5,920 documents and 27,214 Criteria cells, close to the ticket's 26,709. Each document was run
through `check_document_grammar` with the `spec-artifacts-iso` module vocabularies, once on
`origin/main`'s `ac.rs` and once on the PR head. Before the change, `ac:unclassifiable`
fired 49 times. After it, it fired 46 times. The 3 removed cells are `Add service to scenario
version`, `Attach dataset reference to scenario` and `Attach job to scenario version`. All 3 are
genuine predicates, so the removals are 3 of 3 correct. No true positive stopped firing:
`Structural evaluation`, `Task ID`, `Type Check` and `Withdrawn.` all still fire. Because the new
disjunct only changes cells that would otherwise fail, the corpus has 0 newly passing cells
that are not predicates. Precision went from 38/49 (77.6%) to 38/46 (82.6%). The rule removed 3
of about 7 imperative false positives, not "most of ~12" as the doc comment and the PR claim.

## Verdict

**CHANGES REQUIRED.** The regex is linear-time in Rust's `regex` crate, it follows the
repo's `OnceLock` idiom, and 3 of 3 corpus removals are correct. But the PR moves FR-047-AC-15's
only trace tag away from that AC, it states the rule's effect without having measured it,
and the rule accepts Title-Case headings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The TC-775 test's trace was changed from `FR-047-AC-15` to `FR-047-AC-19`. TC-775 asserts the pair idiom, which is AC-15, and tests.md still maps TC-775 to FR-047-AC-15. `quire matrix` on the PR head now reports FR-047-AC-15 as `untagged` with binder `(none)`, and credits AC-19 with an unrelated binder (ac.rs:1005). Revert the line to `FR-047-AC-15`. | src/grammar/ac.rs:1001 |
| FND-002 | medium | The doc comment claims the remaining non-true findings "were mostly imperatives, which the imperative-head branch now covers". Measured on the same frame, the branch removes 3 of 49 findings. These imperative false positives still fire: `Attach valid config overlay reference`, `Attach secret reference ID` (no preposition) and `walk the chain in topological order.` (`walk` is not in the list). The comment's 50-finding figure is also the before-rule count. State the measured after-count (46 on a 27,214-cell frame, 38 true, 82.6% precision) and that the branch removed 3. | src/grammar/ac.rs:378-389 |
| FND-003 | medium | `(?i)` makes the whole pattern case-insensitive, so a Title-Case section heading that opens with a listed verb passes: `Add Item To Cart`, `Remove User From Group`, `Assign Role To Team`. The corpus has this exact heading shape. `Link Repo to Subsystem` (catalog-service FR-005-AC-3, whose real outcome sits in the Verification column) still fires only because `link` is not listed. All 3 corpus wins are sentence case, so requiring a lower-case object word after the verb (or rejecting a cell whose content words are all capitalised) keeps every measured win and closes this. Add `Add Item To Cart` to TC-2030's negative cases. | src/grammar/ac.rs:798-806 |
| FND-004 | low | The doc comment says the list "omits words that are as often nouns", but the list includes `copy`, `move`, `merge`, `split`, `upload`, `download`, `install` and `cancel`. Combined with the preposition `of`, noun-phrase headings such as `Download of artifacts from registry` or `Copy of record in cache` pass. Drop those verbs, or drop `of` from the preposition set (no corpus win uses `of`). | src/grammar/ac.rs:792-794,802 |
