---
id: SR-20261103118144
title: "Spec review — FR-069-AC-15 / TC-2190 duplicate semantic export (quire-rs PR 629)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-214. PR head `accd773`. This review covers the new FR-069 cross-module rule bullet (lines 161-164), FR-069-AC-15 (line 221), the TC-2190 row (tests.md:933) and the audit row (tests.md:2191). `quire validate --scope .` on both files exits 0.

## Verdict

**Pass with findings.** The rule uses the EARS unwanted-behaviour form ("If ..., then the loader SHALL ..."), matching the duplicate-package bullet above it. AC-15 is testable, has a control clause, and maps 1:1 to TC-2190.

"naming the type and both modules" is met in the same way as the AC-10 precedent: the reason names the first module, and the refused module is the failure's `module`. This review does not raise that as a finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The rule and AC-15 define the two-module case only. They do not say whether a refused module's other exports still count as exported. With three modules, two implementers would differ: one refuses a third module that conflicts only with a refused module, and the other loads it. The current code does the first (code-review FND-001). State that a refused module exports nothing, and extend AC-15 with the three-module clause. | spec/functional/FR-069-semantic-module-contract-at-load.md:161-164 |
| FND-002 | low | The names are compared as exact strings (`semantic.exports` entries are declared type names). The pair that motivated the ticket is `master-requirements` in iso and `MasterRequirements` in app, and that pair would not be refused. This is acceptable if the canonical spelling is enforced elsewhere (spec-artifacts-app#28). The spec should say that equality is exact, so the gap is deliberate. | spec/functional/FR-069-semantic-module-contract-at-load.md:221 |

## Dispositions
