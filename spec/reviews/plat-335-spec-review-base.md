---
id: SR-16430
title: "Base review of the pytest/vitest emitter relocation and FR-051-AC-47/48 (quire-rs PR 622)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md, deleted spec/functional/FR-079-pytest-trace-marker-plugin.md and spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md (PR 622 diff only, head 21408e0)"
review_set: subset
---

## Summary

Tickets: PLAT-334, PLAT-335. The PR deletes FR-079 and FR-080 with TC-2120..TC-2141, their summary, AC-audit and CON-audit rows, and adds FR-051-AC-47 (Python and TypeScript extractor names, TC-2154), FR-051-AC-48 (per-language emit chain, TC-2155), a PLAT-334 / PLAT-335 note, and an FR-051 summary row of AC-1..48. This review checked ID formats and uniqueness, AC-to-TC coverage, and dangling references. It also compared the removed text with ix-trace-py PR 2 and ix-trace-ts PR 2 FR-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-051 summary row's status still reads `✅ Implemented except AC-32..33 (🚧, TC-2091/TC-2092 unimplemented)`. That status omits AC-47..48, which are 🚧 and whose tests TC-2154/TC-2155 are unimplemented. The row therefore claims AC-47 and AC-48 are implemented. Fix: `except AC-32..33, 47..48 (🚧, TC-2091/TC-2092/TC-2154/TC-2155 unimplemented)`. | spec/tests.md:149 |
| FND-002 | medium | FR-051-AC-47 states that a Python method is `Class.method` with `nested classes dot-joined`, but none of its five fixture pairs, and nothing in TC-2154, has a nested class. No test of AC-47 can fail on the nested rule. ix-trace-py PR 2 FR-001-AC-5 requires `Outer.Inner.method` to equal what `quire coverage --json` reports, so that pair needs a quire-rs anchor. The extractor does produce it: `qualify` in src/symbols/python.rs:526-534 prefixes the enclosing scope's qualified name. A method only counts as a test when its class is `Test`-prefixed (python.rs:397-403, 435-447), which matches pytest collection. Fix: add a pair such as `(tests/test_x.py, TestOuter.TestInner.test_n)` to AC-47 and TC-2154. | spec/functional/FR-051-source-symbol-extraction.md:235, spec/tests.md:1153 |

## Verdict

Examined: FR-051-AC-47, FR-051-AC-48, the PLAT-334 / PLAT-335 note, the FR-051 summary row, TC-2154, TC-2155, the audit rows `FR-051-AC-47 | TC-2154` and `FR-051-AC-48 | TC-2155`, and every deleted FR-079/FR-080 row.

Relocation completeness: nothing is lost. ix-trace-py FR-001-AC-1..11 and CON-1 restate FR-079-AC-1..11 and CON-1. AC-5 adds `Outer.Inner.method` and AC-9 adds the `legacy` family alias. ix-trace-ts FR-001-AC-1..11 and CON-1 restate FR-080-AC-1..11 and CON-1. AC-8 adds the `describe.concurrent` case, AC-12 the unrecoverable `it.each` template, and AC-13 plain JUnit when emission is off. Both new files are supersets. Both are still open PRs, so this PR must not land before them (see the merge notes on the ticket).

Dangling references: a grep of `spec/`, `src/`, `tests/`, `crates/` and `docs/` for `FR-079`, `FR-080` and `TC-2120..TC-2141` finds none. The exceptions are the untouched `spec/reviews/plat-334-335-*`, unrelated fixture JSON in `tests/fixtures/filament_core`, and old bench measurements. The new ids are unique on main and in every open PR. `quire validate` over all requirement documents and `spec/tests.md` exits 0, and `quoin matrix` lists FR-051-AC-47 and AC-48. The diff adds no hash, pin or tracking record. Two medium findings, no high: mergeable once FND-001 and FND-002 are fixed.

FR-051-AC-47's other clauses match the real extractors. Python: bare name at module level, `Class.method`, the module never a prefix, and a parametrized test named by its `def`. TypeScript: `describe` never qualifies (typescript.rs:57-65), and `it.each` gives the template title as written (typescript.rs tc943 asserts `TC-119 each case %i`). AC-47 correctly gives no class form for vitest.
