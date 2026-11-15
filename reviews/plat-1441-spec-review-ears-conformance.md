---
id: SR-20261115183662
title: "EARS conformance review of PLAT-1441 FR-069, FR-081, FR-084 amendments"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/functional/FR-081-record-sections-extraction.md, spec/functional/FR-084-two-way-relation-coverage-report.md (PR #688)"
review_set: all
---

## Summary

Ticket: PLAT-1441. The FR-069 and FR-081 additions use SHALL statements and EARS "If ...
then" unwanted-behaviour forms correctly. The FR-084 addition has no SHALL statement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-084 "Carried record values and the reverse view" section is descriptive prose ("carries", "is omitted", "The engine selects no key") with no SHALL, so the new report obligation lives only in AC-20..AC-22. | spec/functional/FR-084-two-way-relation-coverage-report.md:79-103 |

## Verdict

Changes requested (medium). Restate as, for example, "When a document entry's record is
non-empty, the engine SHALL serialize it as `record` ...". Restate it after the FR-084
selected-field redesign (base review FND-001). Also examined, with no defect found: FR-069
bullets 1–5, the FR-081 frontmatter-owned bullets, and FR-081-AC-18.
