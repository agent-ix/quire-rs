---
id: SR-16433
title: "Evidence review of FR-051-AC-47/48 and TC-2154/2155 (quire-rs PR 622)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md (PR 622 diff only, head 21408e0)"
review_set: subset
---

## Summary

Tickets: PLAT-334, PLAT-335. This review checks whether the verification authored for FR-051-AC-47 and AC-48 can fail on the defect each one guards against. It compares them with the Rust companion pair FR-051-AC-32/33 and with the amended E-4 in quoin FR-120.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-051-AC-48's second run requires only that the failing test is recorded `fail` and `binds no obligation`. It does not require the failing entry to carry the test's ids. The amended quoin FR-120 E-4, which ix-trace-ts FR-001 Limits adopts, records a vitest test that throws before its `trace()` call with the runner's outcome and no `trace` property. That run binds nothing whatever the evidence gate does with a `fail` outcome. So the vitest half of TC-2155 can pass without testing that a failed result refuses to bind. The Rust companion AC-33 does not have this gap, because its ids come from an attribute. Fix: the failing test calls `trace` (or carries its marker) before it fails, and its recorded entry carries its ids. | spec/functional/FR-051-source-symbol-extraction.md:236, spec/tests.md:1154 |
| FND-002 | low | FR-051-AC-48 is less precise than AC-33, the pattern it follows. It drops AC-33's `No other test of the fixture carries the panicking test's ids`, which is what makes `binds no obligation` attributable to the failed test. It names no failure trigger (AC-33 names `IX_TRACE_E2E_FAIL`). It also leaves open whether the failing second run is required per language: the AC says `one emitter chain per language` but `A second run of the same suite`, and TC-2155 says `a second run`. Fix: one failing run per language, a named trigger, and AC-33's exclusivity clause. | spec/functional/FR-051-source-symbol-extraction.md:236, spec/tests.md:1154 |

## Verdict

Examined: FR-051-AC-47 with TC-2154 (Integration, `quire coverage --json` pairs), and FR-051-AC-48 with TC-2155 (E2E). FR-051-AC-32/33 with TC-2091/2092 were read as context. Test is the right method for both ACs, and the Integration/E2E split matches the Rust pair. AC-47's five named pairs are concrete and each can fail against the extractor (the nested-class gap is SR-16430 FND-002). AC-48's first run can fail on `name`, `classname`, `file` and the `trace` property. Its weak point is the second run, as described in FND-001 and FND-002.
