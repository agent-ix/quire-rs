---
id: SR-967175026302197768900474181935084798
title: "Gap analysis — PLAT-1889 quire-rs repository audit"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/, src/, tests/, corpus/"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs", type: references}
---

## Summary

Ran the planless repository audit using the computed Quoin matrix, coverage report, reverse-gap inventory, and stub scan. The matrix reports 237 untagged criteria, including FR-099-AC-9, FR-099-AC-10, and FR-099-CON-4; plan completion was not assessed.

## Verdict

**FAIL** — the computed matrix has untagged criteria, including the remaining staged FR-099 criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The repository matrix reports 237 untagged criteria, including FR-099-AC-9, FR-099-AC-10, and FR-099-CON-4 | FR-099-AC-9, FR-099-AC-10, FR-099-CON-4 |

## Coverage

- Reconciliation: `quoin matrix` (`quoin 0.36.10`, `quire 0.52.1 (engine 0.62.2)`) and `quire coverage`; `quire matrix --strict --format json` also returned exit 1.
- Static matrix: 1,510 criteria; 1,132 tagged, 237 untagged, 141 method-without-symbol, 0 retired, and 0 tagged-by-ignored-test.
- Evidence: all 1,510 criteria report no run evidence.
- FR-099: AC-1..8 and AC-11 tagged; AC-9, AC-10, and CON-4 untagged.
- Coverage report: 1,154/1,464 backed; 1,324 criteria; 767 property-shaped and 150 specific-shaped; 269 untracked symbols and 1,402 diagnostics.
- Reverse-gap and stub scan: no source stubs or test stubs identified; untracked symbols were grouped as corpus 208, src 13, tests 39, tools 9, across 45 trace ids.
- Plan completion: not assessed
- Semantic review: skipped (not opted in).

## Scope

- `FR-099-AC-1` examined: Given readable base and head revisions, the report maps changed Rust lines on both sides to the containing symbols, includes enclosing containers, and reports deleted base-side and added head-side symbols. An unavailable revision is an explicit error, not an empty report.
- `FR-099-AC-2` examined: Every changed line outside an extracted symbol is returned as an unmapped-change record with repository-relative path, revision side, and line range.
- `FR-099-AC-3` examined: Every test symbol added or edited by the diff is selected with a Rust test-runner filter and a direct-change reason; a deleted test is not selected.
- `FR-099-AC-4` examined: For a changed symbol implementing an exact id, the report selects every test verifying that id and lists every other symbol implementing it. A test or symbol with only a different id is not included by this rule.
- `FR-099-AC-5` examined: Every selected test and related symbol carries a complete reason chain from a changed path and line range through the relevant symbol and exact relation id; a directly edited test carries its direct-change chain.
- `FR-099-AC-6` examined: Requirement-level ids match only the same normalized requirement-level id and do not expand to criterion ids; criterion ids match only the same normalized criterion id; repository-prefixed ids remain distinct from local ids.
- `FR-099-AC-7` examined: A changed production symbol with no bound relation is reported as unlinked. An id that is absent from supplied specs, a spec that is unavailable, and a source marker that fails to bind produce distinct findings; a path-qualified non-binding Rust attribute is not reported as a successful implementation link.
- `FR-099-AC-8` examined: When extraction, traceability forms, or a referenced specification are unavailable, the affected files or ids are identified as unavailable or unverifiable, and the report does not claim an empty selection or unaffected tests.
- `FR-099-AC-9` examined: Caller following resolves only statically named paths that exactly match one extracted Rust symbol path; dynamic, ambiguous, aliased, or unmatched calls are unresolved. Following stops at its finite depth, returns only nearest linked callers once even in a cycle, adds their verified tests without removing earlier selections, reports unresolved call sites, and marks depth truncation when more caller edges remain.
- `FR-099-AC-10` examined: A dependency revision change reports changed upstream files and extracted items, local references, selected tests with upstream-to-local-to-test reason chains, and every unmatched upstream change as uncovered. An unavailable upstream tree is not reported as having no impact.
- `FR-099-AC-11` examined: Identical analysis inputs produce byte-identical report output and stable result ordering, without timestamps, temporary paths, or host-specific values.
- `FR-099-CON-4` examined: The engine SHALL NOT define or match another trace-marker grammar.

## Audit Notes

The whole-repository matrix includes many pre-existing gaps beyond the PR's scope; the FR-099 untagged criteria are called out because the PR body and PLAT-1889 ticket explicitly record them as staged debt. The audit used the engine's matrix rather than handwritten matrix rows. No plan was supplied, so no plan bundle was selected or inferred.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The 234 pre-existing non-FR-099 gaps belong to PLAT-1297 (Triage Quire matrix gaps and fixture-boundary noise). FR-099-AC-9/10 remain staged coverage under PLAT-1889, with implementation owned by PLAT-1892/PLAT-1898. The repository-wide gap is not fixed by PR #734. |

Disposition recheck: the current matrix has 236 untagged criteria after FR-099-CON-4 gained a test tag; the residual 236 are the 234 pre-existing non-FR-099 gaps plus FR-099-AC-9 and FR-099-AC-10.
