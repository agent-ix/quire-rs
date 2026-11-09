---
id: SR-16331
title: "Gap analysis — PLAT-182 linear-time skeptic oracle detection (quire-rs PR 608)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; src/skeptic.rs; spec/functional/FR-064-skeptic-layer.md (FR-064-AC-3)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-064", type: references}
---

## Summary

Ticket: PLAT-182. PR head: `70781bc`. Plan completion: not assessed. The change adds or alters no requirement. The ticket's acceptance criteria are not spec ACs, so the reviewer checked them by hand against the diff. `quire trace --id FR-064-AC-3` (quoin 0.29.1) was run to check the trace binding next to the changed test.

Ticket criteria, checked against the code:

- No regex re-run inside a loop over matches from the same span: met in effect. The assertion scan still sits lexically inside the binding loop, but it is memoized per binding name and runs at most twice per span.
- No `matches('\n').count()` remains: met.
- A regression test over several `expected`/`oracle` bindings asserts candidate and `line_offset`: met for one shape (4000 unasserted `expected` bindings, then an asserted `oracle`).
- Reverting the change keeps the functional assertions passing: met. The timing guard does not discriminate either; see SR-16330 FND-002.

## Verdict

**One medium gap.** The diff moves an existing requirement's trace binding: FR-064-AC-3 is now claimed only by the PLAT-182 performance test, which does not exercise token similarity. Its real verifier, `tc1000_similarity_reads_tokens_not_layout`, lost the tag. No stubs and no untested production branches were added, and no production code is left without an owning requirement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-064-AC-3 (TC-1000) is now credited to `plat182_oracle_candidate_scales_and_picks_first_asserted_binding`, which does not verify it. `tc1000_similarity_reads_tokens_not_layout` carries no trace, so the matrix reports the AC as covered by the wrong test | src/skeptic.rs:1400-1423 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-064-AC-3 | examined | Trace displaced (FND-001) |
| PLAT-182 ticket criteria (4) | examined | Met; timing guard weakness recorded in SR-16330 |
| src/skeptic.rs | context_only | Production change covered by SR-16330 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At 9b3103c `quire trace --id FR-064-AC-3` lists only `tests::tc1000_similarity_reads_tokens_not_layout` (src/skeptic.rs:1430) |
