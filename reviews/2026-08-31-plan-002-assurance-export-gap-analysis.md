---
id: SR-067
title: "Gap analysis: Plan-002 source-grounded assurance export"
type: SpecReview
analysis: gap-analysis
scope: "Plan-002, its four tasks, FR-067/FR-068, and TC-1084..TC-1099"
review_set: all
relationships:
  - target: ix://agent-ix/quire-rs/Plan-002
    type: reviews
  - target: ix://agent-ix/quire-rs/TestMatrix
    type: references
---

## Summary

Mechanical gap analysis found the targeted producer plan complete: 4/4 tasks
are `completed`, all plan and task checkboxes are closed, and every planned test
ID TC-1084..TC-1099 is backed by a real tracked test in the assurance suites.
No open plan, traceability, or reverse-coverage gap remains in quire-rs.

## Verdict

**PASS for the #386 Quire producer scope.** IT-001 remains explicitly partial
at the cross-repository level: this plan supplies the schema and fixture, while
Quoin owns its consumer implementation. The matrix records that boundary rather
than claiming downstream completion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No open mechanical gap found: 4/4 tasks and 16/16 planned test IDs are complete and traceable. | `plan/Plan-002-source-grounded-assurance-export/`, `spec/tests.md`, `tests/assurance_export.rs`, `tests/assurance_boundary.rs` |

## Plan Completion

| Task | Status | Evidence |
| --- | --- | --- |
| Task-011 | completed | strict v1 schema, typed reader, golden and mutation tests |
| Task-012 | completed | deterministic active-premise inventory and atomic rejection tests |
| Task-013 | completed | source-grounded artifact, obligation, symbol, relation, capability, and observation projections |
| Task-014 | completed | purity boundary, legacy-output compatibility, and local gate evidence |

## Test Matrix Verification

- Plan references: TC-1084..TC-1099 (16 unique IDs).
- Executable tracking markers found: 16/16.
- Missing test markers: 0.
- Duplicate test ownership: 0; combined tests use multiple trace attributes
  intentionally while each ID has one owning test location.
- Matrix statuses: all producer TCs are `✅`; IT-001 is accurately marked as
  producer-complete with downstream Quoin work outstanding.

## Reverse Traceability

New production surfaces are limited to the assurance module, its public
re-exports, and two immutable accessors. Each is owned by FR-067 or FR-068 and
exercised by TC-1084..TC-1099. The draft-2020-12 feature move makes runtime
schema validation possible without changing the resolved dependency set. No
underspecified implementation was found.

## Validation

- Focused suite: 12 passed, 0 failed.
- Formatting, clippy, Python feature check, license, unsafe, property-purity,
  static audits, locked-stack spec validation, and consumer-engine agreement:
  pass.
- Script tests: 136 passed, 3 skipped.
- Full Rust suite has one pre-existing environmental failure in TC-868 due to
  a Linux-only absolute fixture path; it is outside the changed code and occurs
  after all new assurance tests pass.

## Semantic Expansion

Skipped. The optional semantic intent↔test↔code expansion was not authorized;
this report contains the required mechanical completeness and traceability pass
only.
