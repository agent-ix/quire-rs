---
id: SR-20261113073339
title: "Gap analysis — PLAT-1674 FR-058 when_any (quire-rs PR 667)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@18c3bb9; spec/functional/FR-058-upward-trace-completeness.md (section 'Narrowing the obligation by an edge', AC-23..28, CON-4), tests/it/required_relations_when_any.rs, src/corpus/required_relations.rs, src/traceability.rs, src/assurance.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-058", type: references}
---

## Summary

Ticket: PLAT-1674. This is a planless gap analysis of PR head `18c3bb9`, scoped to the FR-058 amendment. Matrix: `quire matrix --scope . --format json` (quire 0.44.0, engine 0.59.1; quoin 0.30.1). FR-058-AC-23..28 are each `tagged` to one test in `tests/it/required_relations_when_any.rs`. FR-058-CON-4 (Inspection) is `method-without-symbol`, as expected. Inspection confirms the engine names no verb, direction or kind. No stubs or placeholder returns were found. The optional semantic review was not run; each test was read against its AC by hand, with the mutation each assertion must catch.

Plan completion: not assessed

## Verdict

**FAIL (one medium, one low).** Bindings for AC-23, 25, 26 and 28 are correct and can fail. The direction clause of AC-24 cannot fail: both of its direction cases are confounded with a wrong `other` kind. A mutation that ignores `direction` passes all six tests. Reverse gap: the FR-068 assurance projection does not apply `when_any` (code-review SR FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-058-AC-24 says an outgoing condition is not held by an incoming edge of the same verb, and the reverse. The test cannot fail on that clause. C-2's outgoing `assumes` points at H-1 (`Haz`), but the `assumes` condition requires `other: [Est]`. E-1's incoming `protects` comes from an `Est`, but the `protects` condition requires `other: [Haz]`. Both are rejected by kind whatever the direction. Mutating `in_scope_by_edge` to read edges in both directions passes every test. Fix: give C-2 an outgoing `assumes` to an `Est` document, and give some Ctl an incoming `protects` from a `Haz` document, so only direction excludes them. | tests/it/required_relations_when_any.rs:116-133 |
| FND-002 | low | FR-058-AC-27 requires the finding to be advisory by default. The test's `reported` helper chains errors and warnings, and the test only asserts the Error override. It never asserts that the unoverridden finding is a warning and not an error. Fix: assert `report.warnings` holds `ctl-without-req` and `report.errors` does not, before the override. | tests/it/required_relations_when_any.rs:196-222 |

## Coverage

- FR-058-AC-23: tagged, correct (first only, second only, both, satisfied).
- FR-058-AC-24: tagged; the no-edge, dangling and outside-`other` cases are correct; the direction clause is vacuous (FND-001).
- FR-058-AC-25: tagged, correct (outside `where`, excluded, in scope).
- FR-058-AC-26: tagged, correct; all 14 malformed shapes are listed and rejected with the relation and `when_any` named.
- FR-058-AC-27: tagged; the advisory default is not asserted (FND-002).
- FR-058-AC-28: tagged, correct.
- FR-058-CON-4: Inspection; no verb, direction or archetype literal in src/ for `when_any`.
- Reverse gaps: src/assurance.rs required-relation subjects ignore `when_any`/`where` (code-review SR-20261113073338 FND-001).
- Plan completion: not assessed
