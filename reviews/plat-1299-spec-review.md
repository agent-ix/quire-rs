---
id: SR-20261103118132
title: "Spec review — PLAT-1299 FR-069-AC-14 / CR-205 / TC-2081 (quire-rs PR 606)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-1299. Frozen PR head: `baa7cb0`. This review covers the CR-205 note, the revised FR-069 Outputs bullet, the new FR-069-AC-14, the TC-2081 test-case row, the FR-069 coverage-summary row (`AC-10..14`, `TC-2081`) and the audit row `FR-069-AC-14 | TC-2081`. All are present and consistent. `quire validate` passes on both changed files (rc 0). AC-14 is verifiable because it names five concrete loci. The CR note states the change, not a ledger. No ceremony was introduced.

## Verdict

**PASS** with one low finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-069-AC-14 opens "Every `semantic.*` refusal that records a manifest path", which defines its own scope. A refusal kind built without a path is excluded by definition, so the "every" cannot fail for it. The hand-formatted `semantic.duplicate-authority` is one such kind. Meanwhile the Outputs clause "a refusal with no manifest locus omits the path segment" is in no AC. Restate the AC over the refusal kinds (or the `SemanticFailure` reason form) and add the no-locus clause. | spec/functional/FR-069-semantic-module-contract-at-load.md:216 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-069-AC-14 | examined | Verifiable; scope qualifier self-defining (FND-001) |
| FR-069 Outputs `ArchetypeLoadFailure` bullet | examined | Consistent with code |
| CR-205 note | examined | Accurate |
| TC-2081 row, coverage-summary row, audit row | examined | Present and consistent |
| FR-069-AC-13 | context_only | Its `semantic.duplicate-authority` reason shape is unaffected |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | 10a2506 rewords FR-069-AC-14 over every `semantic.*` refusal reason, adds the single-occurrence rule and the no-locus `<code>: <message>` clause, and covers duplicate-authority |
