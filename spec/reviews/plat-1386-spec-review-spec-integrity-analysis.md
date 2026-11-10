---
id: SR-20270
title: "spec-integrity-analysis of the FR-050 strict exemption ACs and TC-2219..TC-2223 (quire-rs PR 639)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@fdff4d7d; spec/functional/FR-050-declarative-coverage-computation.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-050
    type: references
---

# Spec integrity analysis of the FR-050 strict exemption ACs and TC-2219..TC-2223 (quire-rs PR 639)

## Summary

Ticket: PLAT-1386. This pass checks the traceability structure the PR adds: FR-050-AC-64..66, TC-2219..TC-2223, three audit rows, and the FR-050 coverage row's range `62..66`.

- **Mapping.** AC-64 maps to TC-2219 and TC-2220, AC-65 to TC-2221, and AC-66 to TC-2222 and TC-2223. Every AC has a TC, every TC traces to exactly one of the three ACs, and every audit row agrees with its TC's `Traces To` cell.
- **Uniqueness.** Each of TC-2219..TC-2223 is defined once in `spec/tests.md`. No other open quire-rs PR exists, so no concurrent branch can claim the same ids. TC-2218 was the previous maximum.
- **Section shape.** The planned section copies the PLAT-1181 section shape: a "(planned)" heading, the "not runtime or coverage evidence" preamble, and `🚧 Planned` status cells.
- **Validation.** `quire validate --scope .` on both files exits 0. The single warning (`ac:non-canonical-shape`, FR-050-AC-58) appears identically on `origin/main` at line 304, so it pre-exists.
- **Not run.** `make validate` (`examples/spec_validate.rs`, `audit_table::audit_gaps`) could not run in the worktree, because the `corpus` submodule is not checked out (`failed to read corpus/Cargo.toml`). The audit-gap rule was checked by hand instead: AC-64..66 are defined by a leading table cell, they have audit rows, and the coverage range declares them.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | TC-2219, TC-2222 and TC-2223 assert process exit codes, and TC-2221 asserts a text-output line. Both belong to `quire coverage` in agent-ix/quire-cli, not to this crate. quire-cli FR-017-AC-7 still says '`--strict` exits 1 when a row is unbacked'. These TCs are bare `🚧 Planned`, with no host noted. Precedent TC-797 records 'awaiting EXT-3 `quire-cli`'. Fix: name where each CLI-surface TC runs. Split each into its engine half (the `strict_exempt` flag and the totals in `CoverageReport`, testable here) and its CLI half (exit code and text line). Name the companion quire-cli FR-017-AC-7 amendment, so that the two specs do not contradict each other once this merges. | spec/tests.md:1317-1321; spec/functional/FR-050-declarative-coverage-computation.md:169-176 | missing-requirement |
| FND-002 | low | The new ACs use a bare `Test` in the Verification cell. The PLAT-1181 pattern this PR follows writes `Test (TC-nnnn)`, as FR-062-AC-12..19 do. A bare cell gives FR-049 verification-reference integrity nothing to check from the AC side. FR-050-AC-53..57 have the same gap. Fix: `Test (TC-2219, TC-2220)`, `Test (TC-2221)` and `Test (TC-2222, TC-2223)`. | spec/functional/FR-050-declarative-coverage-computation.md:348-350 | wrong-requirement |
| FND-003 | low | AC-64..66 and the planned TC-2219..TC-2223 were added to the FR-050 coverage row, whose status reads `✅ Implemented`. That row now claims as implemented three ACs that are planned. The PLAT-1165 precedent gives a planned AC range its own `🚧 Planned` coverage row: FR-053 AC-15..23. Fix: move `AC-64..66` and its TCs to a separate `FR-050 ... (PLAT-1386)` row marked `🚧 Planned`. | spec/tests.md:148 | wrong-requirement |
| FND-004 | low | Two clauses (TC-2222, and AC-66's 'zero-target model ... even when every unbacked row is exempt') combine a zero-target model with exempt unbacked rows. That combination is ill-posed: a zero-target model is held vacuously 'all exempt' and needs no exempt rows. Fix: phrase it as AC-54 does, 'the same scope with no declared targets still fails the empty-population premise'. | spec/tests.md:1320; spec/functional/FR-050-declarative-coverage-computation.md:350 | wrong-requirement |

## Verdict

Mergeable on structure once FND-001 (medium) is resolved in the same round as SR-20269. FND-002, FND-003 and FND-004 (low) are wording and placement fixes. The mapping is complete in both directions, and the ids are unique.

## Dispositions

Round 1 at 74a4dde9. TC-2224, TC-2225 and TC-2226 are each defined exactly once in `spec/tests.md`, and no other quire-rs PR is open. Audit rows: AC-64 maps to TC-2219, TC-2220 and TC-2224; AC-65 to TC-2221 and TC-2225; AC-66 to TC-2222, TC-2223 and TC-2226. Each matches its TC's `Traces To` cell and the AC's Verification cell.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | TCs split by surface. The engine cases (TC-2219..2224) are prefixed `Engine:`. The CLI cases TC-2225 and TC-2226 are `🚧 awaiting quire-cli (FR-017-AC-7 amendment)`. The Surfaces paragraph names the companion quire-cli FR-017-AC-7 amendment and the change to its error-promotion count. |
| FND-002 | fixed | AC-64..66 Verification cells now read `Test (TC-2219, TC-2220, TC-2224)`, `Test (TC-2221, TC-2225)` and `Test (TC-2222, TC-2223, TC-2226)`. |
| FND-003 | fixed | The FR-050 row is back to `AC-1..60, 62..63`. A separate row, `FR-050 Strict exemption ... (PLAT-1386) / AC-64..66 / 🚧 Planned (engine); CLI cases await quire-cli`, carries the new ACs. |
| FND-004 | fixed | TC-2226 and AC-66 now say "the same scope with no declared targets (the empty-population premise)", AC-54's wording. |
