---
id: SR-20261113045749
title: "Spec review (ears-conformance) - PLAT-1526 FR-090 carried fields and FR-091 field consistency (quire-rs PR 669)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs@e68cc6c; spec/functional/FR-090-declared-carried-fields.md, spec/functional/FR-091-ordered-field-consistency.md, spec/functional/index.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-090", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-091", type: references}
---

# Spec review (ears-conformance) - PLAT-1526 (quire-rs PR 669)

## Summary

Ticket: PLAT-1526. Spec-only review of `git diff origin/main...e68cc6c`. EARS conformance of the FR-090 and FR-091 requirement statements. Tools: quoin 0.30.1, quire 0.44.0 (engine 0.59.1), module spec-artifacts-process@v0.31.0. Assurance profile spec/assurance/AP-201-detection-minting.md (`review_policy: require [spec-review]`) applies, because FR-090 changes coverage-matrix and published-schema output. The reviewer ran a direct review. The enforced `ix-flow run review` needs human acknowledgement for its final transition, so it was not started here and remains for the lead. The review set is base plus integrity, dependency and ears-conformance. failure-domain, evidence, risk-complexity and scope-boundary were not run separately: their scope for these two FRs is covered by base FND-002/003/005/007.

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

**Changes requested: one low.** FR-090's statements are well formed ('`quire-rs` SHALL read', 'the engine SHALL carry', 'SHALL NOT derive'). FR-091's core statements constrain data rather than the system.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The normative statements bind the authored data, not the system. 'the `from` value SHALL NOT rank lower than the target value' constrains a document, and the system's response (report a finding) is stated without SHALL. The `required_fields` paragraph has no SHALL at all ('yields a finding'). EARS form: 'When a `from` document's value ranks lower than a reached target's, the engine SHALL report a `trace:<check>` finding naming ...'; likewise 'When a `from` document states no `field`, the engine SHALL report ...'. | spec/functional/FR-091-ordered-field-consistency.md:46, 66 |
