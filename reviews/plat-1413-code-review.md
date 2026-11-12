---
id: SR-20261103118159
title: "Code review (with rust-review lane) — PLAT-1413 coverage_matrix requirement criticality (quire-rs PR 648)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@c9711bd; schemas/output/coverage-v1.schema.json, src/coverage.rs, src/coverage/reconcile.rs, tests/it/coverage_matrix.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-082", type: references}
---

## Summary

Ticket: PLAT-1413. PR head `c9711bd`, scoped to `git diff origin/main...HEAD` (4 files). Code-review with the rust-review lane folded in (rust-style loaded; repo CLAUDE.md conventions take precedence). Focused checks run at head through locked-build: `cargo test --test it fr082` (4 passed) and `cargo test --test it output_contract` (12 passed). The implementation reads `criticality` from each document's frontmatter, drops a null, clones the value verbatim as `serde_json::Value` (justified: FR-082-AC-6 and FR-082-CON-1 require carrying any type as written with no engine enumeration), and joins on the same `declared_tables::relative_path` string the obligation `document` field uses, so legacy-path keys match. Obligation-level criticality (FR-053) is untouched. No unwrap, panic path, unsafe, HashMap, pin, digest or tracking record is introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The schema gains `CoverageMatrixRequirement.criticality` under `additionalProperties: false`, but no test validates a criticality-bearing `coverage_matrix` entry against `coverage-v1`. The FR-055-AC-3 "full" payload in output_contract.rs (the payload that exists to carry every optional key) still has a `coverage_matrix` entry with no `criticality`, and its completeness guard counts root keys only, so deleting the schema property fails no test. Fix: add `"criticality": "high"` to that payload's `coverage_matrix[0]` (or validate the fr082 test report against the schema). | schemas/output/coverage-v1.schema.json:1082-1084, tests/it/output_contract.rs:314-330 |
| FND-002 | low | `document_criticality` filters a present-but-null value (`criticality: ~` or `criticality:` with no value) to absent. That branch is untested, and FR-082 only states the no-field case. The choice is defensible (one absent convention) but unpinned; add a null case to the FR-082-AC-2 test. | src/coverage.rs:648-654 |
| FND-003 | low | The inserted `criticality_by_document` map now sits between the FR-050-AC-48 ordering comment and the `by_document` mapping that comment describes. Move the map above the comment. | src/coverage/reconcile.rs:917-933 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/coverage.rs (CoverageMatrixRequirement, document_criticality) | examined | Correct; FND-002 |
| src/coverage/reconcile.rs (coverage_matrix join) | examined | Correct join key; FND-003 |
| schemas/output/coverage-v1.schema.json | examined | Property correct and untyped by design; FND-001 |
| tests/it/coverage_matrix.rs fr082_* | examined | Assert real values; pass |
| FR-082-CON-1 | examined | No enumeration in engine |
| src/obligation/inventory.rs matches_spec | context_only | Evidence-path document keys are root-relative too |

## Verdict

PASS with fixes requested: one medium (schema change untested) and two low. The code itself is correct and idiomatic. Mergeability also depends on the gap-analysis artifact's high finding (missing Test Matrix rows).
