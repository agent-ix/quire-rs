---
id: SR-16372
title: "Integrity review of FR-079, FR-080 and spec/tests.md rows (quire-rs PR 613)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-079-pytest-trace-marker-plugin.md, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md, spec/tests.md (PR 613 diff only)"
review_set: subset
---

## Summary

Tickets: PLAT-334, PLAT-335. This review checks consistency with the upstream Emitter contract (Quoin FR-120 E-1..E-10 and FR-120-AC-5), internal consistency of each FR, and completeness of the `spec/tests.md` rows. FR-080's runtime-only limit breaks E-4 as well as FR-120-AC-5, and the spec does not record either consequence. The `spec/tests.md` additions also leave the coverage summary and the constraint audit inconsistent.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FR-080 Limit (`A test that never reaches its trace(...) call (skipped by a modifier, or throwing before the call) has no declared ids`), together with FR-080-AC-6 (`a skipped test carries no ids`), contradicts the upstream contract in two places. (1) E-4 says a test that throws is recorded `fail` `with its ids, never absent`, and a test that throws before `trace()` has no ids. (2) FR-120-AC-5 says a skipped tagged test produces an entry `with that outcome and its ids`. The FR mentions neither E-4 nor FR-120-AC-5; only the PR body names AC-5, and it says nothing about E-4. FR-120 is normative for this binding, so a known shortfall needs an FR-120 amendment first, as E-1 requires for carriers. Otherwise declare ids at registration: vitest 4.1.9 `createTest` already merges a registration-time `options.meta`. | spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:64-68, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:86 |
| FND-002 | low | `### Functional Requirement Coverage` in `spec/tests.md` ends at the FR-078 row. FR-079 and FR-080 get no summary row (`AC-1..10; CON-1` / `AC-1..11; CON-1`, TC range, `🚧 Planned`), unlike every other FR. | spec/tests.md:117 |
| FND-003 | low | The constraint audit rows `FR-079-CON-1 -> TC-2127` and `FR-080-CON-1 -> TC-2138` are inserted between FR-078-CON-3 and FR-078-CON-4. Both constraints declare Validation `Inspection`, yet the rows map them to tests. The repo's form for an Inspection constraint is `Inspection — ...` (see FR-078-CON-1). TC-2127 and TC-2138 also list only their ACs, not the constraints. | spec/tests.md:2245-2247 |

## Verdict

Examined: E-1..E-10 against both FRs, FR-120-AC-5, the FR-079/FR-080 relationships (`implements` Quoin FR-120, `references` FR-051, `traces_to` StR-007), TC-2120..2140 and their audit rows, and the id ranges. All 21 ACs map one-to-one to a TC. The ids are unique. `QUIRE_TRACE_EMIT=1` is the only enable name used across both FRs and their ACs. Its claim about the Rust emitter is wrong, as recorded in SR-16370 FND-002. FND-001 blocks merge.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The FR-120 amendment is tracked on PLAT-334 and has not merged in quoin. Commit eaf5dbd now records the shortfall in the spec itself, as an explicit `Deviation from E-4 and FR-120-AC-5 (FR-120 amendment pending, PLAT-334)` entry in FR-080 Limits (FR-080:72-79), so the binding no longer claims conformance it cannot meet. |
| FND-002 | fixed | Commit eaf5dbd adds the FR-079 (`AC-1..11; CON-1`, TC-2120..TC-2130) and FR-080 (`AC-1..11; CON-1`, TC-2131..TC-2141) summary rows (spec/tests.md:118-119). |
| FND-003 | still-open | Commit eaf5dbd adds correct `Inspection — ...` audit rows for FR-079-CON-1 and FR-080-CON-1 (spec/tests.md:2282-2283), but leaves the old rows `FR-079-CON-1 -> TC-2127` and `FR-080-CON-1 -> TC-2138` at spec/tests.md:2257-2258, still between FR-078-CON-3 and FR-078-CON-4. After the renumber those rows point at unrelated tests: TC-2127 is FR-079-AC-8 and TC-2138 is FR-080-AC-8 (parallel execution). Each constraint is now declared twice with contradicting evidence. Fix: delete the two stale rows. |
| FND-003 | fixed | Commit bc15b44 deletes the stale rows `FR-079-CON-1 -> TC-2127` and `FR-080-CON-1 -> TC-2138`. Only the `Inspection — ...` rows remain (spec/tests.md:2280-2281). |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The round-1 fix commit eaf5dbd added a renumbered AC audit block (spec/tests.md:2258-2279) but did not delete the original pre-renumber block. It is still at spec/tests.md:2236-2256, between FR-078-CON-3 and FR-078-CON-4. Every FR-079-AC-1..10 and FR-080-AC-1..11 row is therefore declared twice. The 11 old FR-080 rows map each AC to the wrong test, off by one: `FR-080-AC-1 -> TC-2130` (TC-2130 is FR-079-AC-11), and so on through `FR-080-AC-11 -> TC-2140`. `quire validate` does not catch this. Fix: delete spec/tests.md:2236-2256. | spec/tests.md:2236-2256, spec/tests.md:2258-2279 |
| FND-005 | medium | quoin PR 776 (merged, `69716707`) amended FR-120. E-4 now says that when ids come from a call inside the test body and the test never reaches it, the emitter SHALL record the case with the runner's outcome and no `trace` property. FR-120-AC-5 now covers an entry with no `traceIds` that binds and discharges nothing. FR-080's Limits still says `Deviation from E-4 and FR-120-AC-5 (FR-120 amendment pending, PLAT-334)` and still says E-4 and AC-5 require ids. That is now false, and this text is what keeps FND-001 deferred instead of closed. Fix: reword the bullet to state that this is the amended E-4 exception (quoin PR 776) and that FR-080 conforms. | spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:72-79 |
