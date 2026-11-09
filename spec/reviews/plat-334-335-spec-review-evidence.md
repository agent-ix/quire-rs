---
id: SR-16373
title: "Evidence review of FR-079 and FR-080 acceptance criteria (quire-rs PR 613)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-rs; spec/functional/FR-079-pytest-trace-marker-plugin.md, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md, spec/tests.md TC-2120..2140 (PR 613 diff only)"
review_set: subset
---

## Summary

Tickets: PLAT-334, PLAT-335. This review checks whether each AC's declared method (all `Test`) can produce evidence that can fail, and whether the TC rows match. Most ACs name concrete fixtures and negative controls. One AC requires a check that cannot be made at run time, one leaves its expected output undefined, and the end-to-end ACs fold in a quoin behaviour that the emitter does not own.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-079-AC-4 (and `Failure is loud`) requires that `a marker argument that is not a string literal fails the run`. Being a literal is a property of the source syntax. At run time the plugin sees only values, so `pytest.mark.trace(IDS)` with `IDS = "FR-001-AC-1"` is indistinguishable from a literal and cannot be made to fail. Either restrict the check to what the run can observe (`not a str`) and record that a non-literal str is emitted while the static side does not bind it, or define how the plugin reads the source. | spec/functional/FR-079-pytest-trace-marker-plugin.md:63, spec/functional/FR-079-pytest-trace-marker-plugin.md:82 |
| FND-002 | low | FR-079-AC-3 requires ids `in order`, and the Emission section says `in declaration order`, across function, class and module markers and stacked decorators. The order across those levels is undefined, and pytest's `iter_markers` yields closest-first, which is the reverse of top-down source order. FR-120 treats the value as an accumulating set, so the order carries no meaning. Drop the order requirement, or define it exactly. | spec/functional/FR-079-pytest-trace-marker-plugin.md:43, spec/functional/FR-079-pytest-trace-marker-plugin.md:81 |
| FND-003 | low | The last clause of FR-079-AC-10 and FR-080-AC-11 (`editing the requirement text marks the binding stale`) tests quoin's evidence-store staleness, not the emitter. It is not part of E-10, and a package-repo test can only re-observe quoin. Drop the clause, or mark it as context that this FR does not verify. | spec/functional/FR-079-pytest-trace-marker-plugin.md:88, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:91 |

## Verdict

Examined all 21 ACs and TC-2120..2140. These can fail as written: FR-079-AC-1/2/5/6/7/8/9 and FR-080-AC-1/2/3/4/6/7/8/9/10. Each states a measurable result and a negative control: a byte-equal comparison apart from timing, a 5% median over five runs or more, an exact count of `<testcase>`, and a compile-time error. The AC-level issues in FR-080-AC-5 are recorded in SR-16370 FND-001 and SR-16374.
