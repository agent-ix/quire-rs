---
id: SR-20261113045747
title: "Spec review (integrity) - PLAT-1526 FR-090 carried fields and FR-091 field consistency (quire-rs PR 669)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@e68cc6c; spec/functional/FR-090-declared-carried-fields.md, spec/functional/FR-091-ordered-field-consistency.md, spec/functional/index.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-090", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-091", type: references}
---

# Spec review (integrity) - PLAT-1526 (quire-rs PR 669)

## Summary

Ticket: PLAT-1526. Spec-only review of `git diff origin/main...e68cc6c`. Integrity: id formats, duplicates, gaps, index entries, link targets and merge collisions with concurrent PRs. Tools: quoin 0.30.1, quire 0.44.0 (engine 0.59.1), module spec-artifacts-process@v0.31.0. Assurance profile spec/assurance/AP-201-detection-minting.md (`review_policy: require [spec-review]`) applies, because FR-090 changes coverage-matrix and published-schema output. The reviewer ran a direct review. The enforced `ix-flow run review` needs human acknowledgement for its final transition, so it was not started here and remains for the lead. The review set is base plus integrity, dependency and ears-conformance. failure-domain, evidence, risk-complexity and scope-boundary were not run separately: their scope for these two FRs is covered by base FND-002/003/005/007.

Units examined:

- `FR-090` (examined)
- `FR-090-AC-1` (examined)
- `FR-090-AC-2` (examined)
- `FR-090-AC-3` (examined)
- `FR-090-AC-4` (examined)
- `FR-090-AC-5` (examined)
- `FR-090-AC-6` (examined)
- `FR-090-AC-7` (examined)
- `FR-090-AC-8` (examined)
- `FR-090-AC-9` (examined)
- `FR-090-CON-1` (examined)
- `FR-090-CON-2` (examined)
- `FR-091` (examined)
- `FR-091-RF` (examined)
- `FR-091-AC-1` (examined)
- `FR-091-AC-2` (examined)
- `FR-091-AC-3` (examined)
- `FR-091-AC-4` (examined)
- `FR-091-AC-5` (examined)
- `FR-091-AC-6` (examined)
- `FR-091-AC-7` (examined)
- `FR-091-AC-8` (examined)
- `FR-091-AC-9` (examined)
- `FR-091-AC-10` (examined)
- `FR-091-AC-11` (examined)
- `FR-091-AC-12` (examined)
- `FR-091-CON-1` (examined)
- `FR-091-CON-2` (examined)
- `index.md` (examined)

## Verdict

**Clean.** FR-090 and FR-091 are unused on main and on every open quire-rs PR (#667 touches FR-058 only; #668 adds FR-086 and FR-058 CR-416). No CR number is used by this PR, so CR-416 does not collide. AC and CON ids are sequential with no duplicates. The index gets two entries in id order. FR-087 to FR-089 stay unallocated, which is not a defect. Merge note: #668 and #669 both append to spec/functional/index.md directly after the FR-085 line. Whichever merges second gets a trivial textual conflict, resolved by keeping both lines in id order (FR-086, FR-090, FR-091). `quire validate --scope . spec/functional/FR-09*.md spec/functional/index.md` reported no error for these files, only one unrelated semantic.unused-import notice for module spec-artifacts-app.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
