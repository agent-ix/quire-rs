---
id: SR-967175026302197768900474181941935084886
title: "Specification integrity review of PLAT-1904"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; PLAT-1904; spec/functional/FR-092-matrix-design-column.md (FR-092-AC-20)"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/FR-092"
    type: references
---

## Summary

Integrity analysis of the added FR-092-AC-20 and its relationship to AC-19, the engine API, and the integration test found no contradictory or duplicated statement. The exception is narrowly bounded by the declaration kind and preserves strict behavior elsewhere.

## Verdict

PASS — no new structural integrity defect found in the changed requirement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Assurance Context

Applicable active profile: `spec/assurance/AP-201-detection-minting.md`, covering coverage collection and row minting. No population or measurement definition changed. The independent QA corpus path was not exercised for this engine-specific exception. Per the owning minder's ticket-specific disposition, no ix-flow run or separate Peter acceptance is required for PLAT-1904.

## Examined scope

- FR-092-AC-20: examined for atomicity and exact exception boundary. It names the allowed dropped kind (`RequiredRelation`) and the fatal remaining kinds and standard coverage path.
- FR-092-AC-19 and FR-092 Errors: context only; retained sibling output and dropped relation error behavior are consistent with the new helper contract.
- `compute` and `compute_with_evidence`: context only; both remain strict, matching “ordinary coverage remains strict.”
- `compute_coverage_for_design_matrix` and its integration case: examined; implementation and test tags map to AC-20.

## Integrity checks

The AC id is sequential and unique. Requirement, implementation marker, and test trace tag use the same full criterion id. The new function name is also the one in the criterion. No duplicate requirement, conflicting acceptance criterion, stale cross-reference, or hash/digest/pin/tracking-record antipattern was introduced.
