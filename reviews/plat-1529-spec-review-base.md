---
id: SR-20261113045750
title: "Spec review (base) - PLAT-1529 FR-092 design column on the coverage matrix (quire-rs PR 670)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@83ff183; spec/functional/FR-092-matrix-design-column.md, spec/functional/index.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-092", type: references}
---

# Spec review (base) - PLAT-1529 (quire-rs PR 670)

## Summary

Ticket: PLAT-1529. Spec-only review of `git diff origin/main...83ff183` (the head moved from 2d3e033 to 83ff183 during the review; the only change was the downstream reference renumbering to quire-cli FR-043 and quoin FR-165, which now match quire-cli#188 and quoin#858). Tools: quoin 0.30.1, quire 0.44.0 (engine 0.59.1), module spec-artifacts-process@v0.31.0. `quire validate --scope .` on the two changed files passes (one unrelated module warning, `semantic.unused-import` in spec-artifacts-app). Assurance profile spec/assurance/AP-201-detection-minting.md (`review_policy: require [spec-review]`) applies, because FR-092 defines a new coverage-matrix output. The reviewer ran a direct review. The enforced `ix-flow run review` needs human acknowledgement for its final transition, so it was not started here and is left for the lead. Review set: base plus integrity and ears-conformance. failure-domain, dependency, evidence, risk-complexity and scope-boundary were not run separately. For one new FR that has no new frontmatter edge types, their scope is covered by base FND-001/003 and integrity FND-001/002.

Context read on origin/main (fb0a09b): FR-050 (AC-47..51), FR-055, FR-058, FR-082, FR-084, FR-086, FR-090, schemas/output/coverage-v1.schema.json, src/corpus/relation_report.rs.

Units examined:

- `FR-092` (examined)
- `FR-092-AC-1` .. `FR-092-AC-15` (examined, all 15)
- `FR-092-CON-1`, `FR-092-CON-2` (examined)
- `FR-050-AC-48`, `FR-090`, `FR-084`, `FR-086`, `FR-055` (context_only)

## Verdict

**Changes requested: three medium, no high, no low.** The design is right. The engine holds no archetype name or verb (AC-12/13). Scope comes only from FR-058 `from`/`where`/`exclude` of the declared `design_relations` (AC-4). The `coverage` report and its `coverage_matrix` stay unchanged (CON-2, AC-5). `design_matrix` reports no verdict and writes nothing. There is no compatibility layer and no paywalled standard text.

Rulings the lead asked for:

1. **CR note on FR-050: not needed.** FR-092 does not change the FR-050 `coverage_matrix`. CON-2 forbids it, and the `criteria: []` entries exist only in `design_matrix` output. The author's note says FR-092 changes the FR-050 shape, but that misreads its own CON-2. The FR-090 precedent agrees: FR-090 *did* change `coverage_matrix` (it added `carried`) and still added no CR note on FR-050. It stated the amendment in its own body, through its `extends` edge, and in AC-11.
2. **Published coverage-v1 schema: not broken by FR-092, but FR-092 is silent about its own payload contract** (FND-002). FR-090 put its schema amendment in its own text. FR-092 needs the matching statement for its new payload.
3. **FR-087..089 "reserved".** No record of a reservation exists in quire-rs: no file, index entry, branch, PR or Linear issue names quire-rs FR-087..089. The skip is harmless (ids need not be contiguous), so it is not a finding. The claim is unsubstantiated, though. Do not add a reservation record: that would be tracking ceremony.

Clean units: AC-1, AC-2, AC-4, AC-6, AC-8, AC-9, AC-10, AC-11, AC-12, AC-13, CON-1, CON-2 (examined, no defect).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The branch is based on f8a8cdb, which predates FR-090/FR-091 (#669). The PR is CONFLICTING on spec/functional/index.md. FR-090 puts `carried` on every `requirements[]` entry of `coverage_matrix`, but FR-092's fixed key order (`document`, `repo`, `id`, `criticality`, `criteria`, `design`) has no place for it. An implementer must either drop `carried` from design-bearing entries, which loses the value FR-090 places on the entry, or invent a position for it. Two implementers would emit different bytes. Fix: rebase onto main, resolve the index, and name `carried` in the key order (after `criticality`). | spec/functional/FR-092-matrix-design-column.md:60-61 | wrong-requirement |
| FND-002 | medium | FR-092 calls its output "the `coverage_matrix` with a `design` column" and does not say which published contract it conforms to. Under coverage-v1, `CoverageMatrixRequirement` and `CoverageMatrixCriterion` are `additionalProperties: false`, so `repo`, `id`, `design` and criterion `design_elements` would be rejected by any consumer validating with it. FR-055 asks for a schema for each machine-readable payload. FR-090 amended coverage-v1 explicitly when it added a key (AC-11). Fix: state that `design_matrix` output is a distinct payload and not a coverage-v1 `coverage_matrix`. Then either publish its schema under FR-055, or say it is unschematized as the FR-084/FR-086 reports are. Do not widen coverage-v1, which CON-2 keeps unchanged. | spec/functional/FR-092-matrix-design-column.md:27-29 | missing-requirement |
| FND-003 | medium | The rule that a dangling `linked` entry of a `to: []` relation is not an element (it appears only in `dangling`, and the block is `covered`) has no AC. AC-7 covers only the non-empty-`to` case. A direct projection of FR-084 `linked` would emit an element with no `id`/`path` (FR-084's `{reference, verb, resolution, ...}` shape), and no test would fail. Fix: add an AC: under a `to: []` relation, a requirement whose only edge is to an absent target gives a `covered` block with `elements: []` and the reference in `dangling`. | spec/functional/FR-092-matrix-design-column.md:61-62 | correct-requirement-no-evidence |
