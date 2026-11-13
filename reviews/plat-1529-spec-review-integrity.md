---
id: SR-20261113045751
title: "Spec review (integrity) - PLAT-1529 FR-092 design column on the coverage matrix (quire-rs PR 670)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@83ff183; spec/functional/FR-092-matrix-design-column.md, spec/functional/index.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-092", type: references}
---

# Spec review (integrity) - PLAT-1529 (quire-rs PR 670)

## Summary

Ticket: PLAT-1529. Integrity analysis of FR-092 at 83ff183: internal consistency, completeness and atomicity, and consistency with FR-050/FR-084/FR-086 on origin/main (fb0a09b). Two medium contradictions and two low defects.

Units examined:

- `FR-092` (examined)
- `FR-092-AC-1` .. `FR-092-AC-15` (examined)
- `FR-092-CON-1`, `FR-092-CON-2` (examined)
- `FR-084` (context_only: row and document shapes, per-member join in src/corpus/relation_report.rs)

## Verdict

**Changes requested: two medium, two low.** The ordering, gap-versus-out-of-scope and error rules are consistent with FR-084/FR-086. The defects are two statements that contradict each other, an AC table out of order, and a join key that is not stated.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The criteria-less entry is given as `{"document", "criteria": [], "design": [...]}`. The rule above it says an entry that has `design` also carries `id`, carries `criticality` as FR-082 states, and carries `repo` for an identified member, in a fixed key order. The literal shape omits all three, so the two readings give different bytes, and AC-3 does not decide between them. Fix: write the criteria-less entry as the general shape with `criteria: []`, for example "an entry with the keys above and `criteria: []`". | spec/functional/FR-092-matrix-design-column.md:74 | wrong-requirement |
| FND-002 | medium | AC-5 and the description say that with no `design_relations` the output is byte-identical to the FR-050 `coverage_matrix`. But `design_matrix` returns every member's entries and adds `repo` for an identified member, while FR-050 computes one member's matrix with no `repo`. For a multi-member or identified corpus the claim cannot hold. Fix: restrict the identity to a single member with no identity, or state the multi-member case as one concatenation of each member's FR-050 entries plus `repo`, ordered by `repo` then `document`. | spec/functional/FR-092-matrix-design-column.md:69; FR-092-AC-5 | wrong-requirement |
| FND-003 | low | AC-15 sits before AC-14 in the table. AC-15 also joins two unrelated checks: same-path entries in two members, and an entry array present with no obligations. Fix: reorder, and split AC-15 into two ACs. | spec/functional/FR-092-matrix-design-column.md:119-120 | wrong-requirement |
| FND-004 | low | "Attaches each row to the entry of the document the row obliges" does not name the join key. FR-084 rows carry `document.{id,path,repo}`, while a matrix entry carries the scope-relative `document` path. The engine joins by member and member-relative path (src/corpus/relation_report.rs:366-370, 445). Fix: state that a row attaches to the entry with the same member (`repo`) and `document` equal to the row's `document.path`. | spec/functional/FR-092-matrix-design-column.md:31 | missing-requirement |
