---
id: SR-20261113045748
title: "Spec review (dependency) - PLAT-1526 FR-090 carried fields and FR-091 field consistency (quire-rs PR 669)"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-rs@e68cc6c; spec/functional/FR-090-declared-carried-fields.md, spec/functional/FR-091-ordered-field-consistency.md, spec/functional/index.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-090", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-091", type: references}
---

# Spec review (dependency) - PLAT-1526 (quire-rs PR 669)

## Summary

Ticket: PLAT-1526. Spec-only review of `git diff origin/main...e68cc6c`. Dependency: the new `relationships:` edges and the `## Dependencies` sections. Tools: quoin 0.30.1, quire 0.44.0 (engine 0.59.1), module spec-artifacts-process@v0.31.0. Assurance profile spec/assurance/AP-201-detection-minting.md (`review_policy: require [spec-review]`) applies, because FR-090 changes coverage-matrix and published-schema output. The reviewer ran a direct review. The enforced `ix-flow run review` needs human acknowledgement for its final transition, so it was not started here and remains for the lead. The review set is base plus integrity, dependency and ears-conformance. failure-domain, evidence, risk-complexity and scope-boundary were not run separately: their scope for these two FRs is covered by base FND-002/003/005/007.

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

## Verdict

**Changes requested: one low.** The declared edges (FR-090 extends FR-050 and FR-053, references FR-082 and iso FR-009; FR-091 extends FR-058, requires FR-057, references FR-090 and architecture FR-012; both trace to StR-006) resolve and point the right way. The frontmatter and the Dependencies prose disagree on FR-059. The dependency on FR-058 CR-416 (#668) is a content gap and is filed as base FND-007.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Both FRs list FR-059 under Upstream in `## Dependencies` and rely on it for the enum read, but neither has a FR-059 `relationships:` edge. FR-091 also takes its rank vocabulary from spec-artifacts-iso FR-009 and its target value from an extracted record, with no frontmatter edge to either. The relationship graph is therefore missing the edges that impact analysis would follow. Fix: add `requires` edges to FR-059 in both FRs, and to spec-artifacts-iso FR-009 (and the record-extraction FR, see SR base FND-008) in FR-091. | spec/functional/FR-091-ordered-field-consistency.md:1-16, 104 |
