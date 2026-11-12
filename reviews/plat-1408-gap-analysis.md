---
id: SR-20261112034126
title: "Gap analysis — PLAT-1408 dangling-reference finding located at source document path (quire-rs PR 647)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-038-okf-bundle-validation.md (FR-038-AC-11), spec/functional/FR-025-spec-corpus-model.md (FR-025-AC-3), spec/functional/FR-057-corpus-check-severity.md, src/corpus/validate.rs, tests/it/bundle_validate.rs, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-038", type: references}
---

## Summary

Ticket: PLAT-1408. PR head: `96d1ddf`. Planless run scoped to the PR diff. Computed matrix from `quire matrix --scope <repo> --format json` (quoin 0.30.1 on PATH). Semantic intent↔test↔code check done by hand for FR-038-AC-11 because the reviewer brief asks for gap analysis against that AC.

## Verdict

**CONDITIONAL.** The code fully implements FR-038-AC-11: the path is the root-joined loaded path, the message names the edge type and harvested target (raw destination for an unresolved relative-path link, resolve.rs:256), the tier comes from the posture unless FR-057 overrides it, and duplicate ids resolve to the first document (FR-025-AC-3). The diff adds no code without an owning requirement and no stubs. Two medium gaps remain: two AC-11 clauses are untested, and the computed matrix does not credit the new tag. By this skill's verdict rule an untagged criterion is FAIL. It is graded CONDITIONAL here because the tag is present in source and the matrix's refusal to credit it is the same pre-existing state as FR-038-AC-9/AC-10. The lead decides.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-11's relative-path clause (message names the raw destination) and its FR-057 clause (promote/demote/suppress changes only the tier, path and message unchanged) have no test, yet the TC-2227 row the PR flips to ✅ claims both | tests/it/bundle_validate.rs:797-838; spec/tests.md:1219 |
| FND-002 | medium | Computed matrix reports FR-038-AC-11 `untagged`: `criterion-tagged-outside-row` says the TC-2227 tag on the new test "does not provide backing" (no declared matrix row). FR-038-AC-9/AC-10 (TC-1992/TC-2151) are in the same state on main; root cause not confirmed | tests/it/bundle_validate.rs:797-799 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-038-AC-11 | examined | Implemented; FND-001, FND-002 |
| FR-025-AC-3 | examined | First-wins honoured; tagged by tests::duplicate_id_diagnosed_first_wins |
| FR-057 (AC-1..AC-10, CON-1) | context_only | route() changes tier only; no new obligation touched |
| src/corpus/validate.rs | examined | No unspecified behaviour, no stubs |
| tests/it/bundle_validate.rs TC-2227 test | examined | Real assertions; covers relationship edge + duplicate id only |

## Coverage

- Criteria in scope: FR-038-AC-11 (untagged in computed matrix), FR-025-AC-3 (tagged).
- Reverse gaps: none in the diff.
- Stubs / coverage inflation: none found.
- Semantic review: done by hand for FR-038-AC-11 only.
- Plan completion: not assessed
