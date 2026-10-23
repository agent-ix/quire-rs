---
id: SR-16289
title: "spec-review of FR-070 Clock and Quantity<unit> resolution and FR-075 dimension and unit tables (quire-rs PR 546)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@5714a267; spec/functional/FR-070-typed-properties-extraction.md; spec/functional/FR-075-model-feature-extraction.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-070
    type: references
  - target: ix://agent-ix/quire-rs/FR-075
    type: references
---

# Spec review of FR-070 Clock and Quantity<unit> resolution and FR-075 dimension and unit tables (quire-rs PR 546)

## Summary

Ticket: STD-422. One spec-review pass over `git diff origin/main...5714a267` (main is an ancestor). Gate: `make validate` with the sibling process and iso roots gives 175 documents, 46 failed, 41 warnings, identical to the baseline (delta 0); FR-070 and FR-075 raise no error. The 46 failures include `spec/tests.md` (unknown type TestMatrix), so the new TC rows and ledger edits are not validated by this gate. Ids FR-070-AC-11..13, FR-075-AC-16..20 and TC-1945..TC-1951 are on no other open PR head (#515, #518, #519, #521) and not on main. Cross-PR check against spec-objects-foundation PR 7 (same ticket): the tables (`Base`; `Dimension | Exponent`; `Dimension | Target | Scale | Offset`), the canonical integer and rational forms, the `true`/`false` cells, the optional `Target`, the unresolved placeholder with `semantic.unresolved-type`, the one-row rule and the match keys `dimension`, `terms`, `unit` agree. They disagree on how a token is matched (FND-005) and PR 7's record-schema claim about `2/4` has no counterpart here (see SR-16290 FND-002). The AC vectors of FR-075-AC-16..18 were checked by hand against the stated cell forms (`1/1000`, `-3/2`, `0`, `-3` accepted; `2/4`, `1/1`, `-0`, `0/5`, `01/3`, `1.5`, `1/0`, empty refused) and hold. The PR body flags the placeholder token, Imports and unit-graph choices as lead defaults, not owner answers.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Scope narrowing: unit and dimension tokens never consult imports, so a Quantity<u> whose unit is exported by a loaded package module resolves to the unresolved placeholder. Ordinary Type cells resolve step 5 from `imports`; the unit token drops that source. The ticket asks for resolution to the bundle identity of the unit artifact, with no exclusion. Fix: resolve a unit or dimension token from imported package exports as step 5 does (the engine needs the exported artifact's object type, so name how the BundleIndex carries it), with an AC vector, or ask Peter for the cut. | spec/functional/FR-070-typed-properties-extraction.md | missing-requirement |
| FND-002 | medium | The rule 'an artifact SHALL declare at most one `dimension` table and at most one of `dimension` and `unit`' has no diagnostic code and no AC vector. Two implementers would refuse it differently, and no test can fail. Fix: name the error and add a vector (two Base tables; a Base table with a unit table) to AC-16 or AC-18. | spec/functional/FR-075-model-feature-extraction.md | wrong-requirement |
| FND-003 | medium | AC-13 gives seven malformed inputs and states the verdict as 'semantic.invalid-type-token or agent-ix.semantic-core.UNIT_NOT_ALLOWED ... as FR-070 states'. It never maps each input to its code, so an engine that returns the other code for `Quantity<ms>(1,2)` or `Quantity<ms> [s]` passes. Fix: pair each input with its code. | spec/functional/FR-070-typed-properties-extraction.md | wrong-requirement |
| FND-004 | medium | The placeholder for an unresolved `Quantity<u>` is `unresolved/<u>`, but the ticket and FCD FR-092 state `unresolved/<Token>` with Token the cell's token, which is `Quantity<hour>`. The PR flags this as a lead default. A consumer reading FR-092 expects `unresolved/Quantity<hour>`. Fix: take Peter's ruling and make FR-092 and this AC agree. | spec/functional/FR-070-typed-properties-extraction.md | wrong-requirement |
| FND-005 | medium | Tokens match by `id` and then by `names` (the title), and AC-12 pins `Quantity<Millisecond>` by title. spec-objects-foundation PR 7 FR-002 and FR-003 say the artifact id is the token and a Dimension or Target cell 'SHALL be the id'. The two PRs of one ticket disagree on what spelling is accepted. Fix: state in both that id and title both resolve, or restrict to id. | spec/functional/FR-070-typed-properties-extraction.md | missing-requirement |

## Verdict

Not mergeable: FND-001 (high) block the PR; FND-002, FND-003, FND-004, FND-005 (medium) in the same round.

Merge: yes against origin/main (ancestor). git merge-tree conflicts with stale open #515 and #519 (both append a correction note at the end of FR-075 and carry a pre-main base), #518 (spec/log.md, corpus submodule) and clean with #521; those PRs are not main-based and conflict on their own base.
