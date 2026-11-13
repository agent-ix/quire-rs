---
id: SR-20261113045752
title: "Spec review (ears-conformance) - PLAT-1529 FR-092 design column on the coverage matrix (quire-rs PR 670)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs@83ff183; spec/functional/FR-092-matrix-design-column.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-092", type: references}
---

# Spec review (ears-conformance) - PLAT-1529 (quire-rs PR 670)

## Summary

Ticket: PLAT-1529. EARS grammar check of FR-092's SHALL statements at 83ff183: `SHALL provide design_matrix` (ubiquitous), `SHALL add an entry` for each obliged criteria-less document (ubiquitous with a scoping clause), `SHALL fail ... when ...` (event-driven), and CON-1/CON-2 (ubiquitous and unwanted-behaviour negatives). One low finding.

Units examined:

- `FR-092` (examined)
- `FR-092-CON-1`, `FR-092-CON-2` (examined)

## Verdict

**Approve with one low.** Each statement has a single subject, `quire-rs` / `design_matrix`, and a single response. The Errors statement puts its trigger after the response.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The Errors statement is event-driven but puts its trigger last ("SHALL fail with ... when a `design_relations` entry names no declared relation, and SHALL return no matrix"), so the trigger reads as part of the error text. EARS form: "When a `design_relations` entry names no declared relation, `design_matrix` SHALL fail with the FR-086 error naming the entry and SHALL return no matrix." | spec/functional/FR-092-matrix-design-column.md:99-100 |
