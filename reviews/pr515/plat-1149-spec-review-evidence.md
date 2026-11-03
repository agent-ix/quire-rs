---
id: SR-20261102118112
title: "PLAT-1149 spec-review-evidence review"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-rs; PLAT-1149; five-file diff and adjacent identity/evidence paths"
review_set: subset
---

## Summary

Candidate: 204d70ed841d6a0872ca239dd36e4d4639c51a1c; base main: 9adea425. Ticket: PLAT-1149; PR: quire-rs#515. Review-only; frozen code was not edited.

Main-red log reproduces the missing five RT criterion gaps; candidate focused logs show 53 rollup, 17 matrix, and 2 baseline tests passing. Candidate formatter and clippy logs pass. These were inspected existing logs, not rerun by this reviewer. No aggregate Cargo gate was run in this reviewer as directed. `corpus_cases_hold` is red with 63 forward-expectation differences; corpus ref is unchanged and QA-corpus #31 is an explicit unresolved dependency. No full-bank or merge-ready claim.

## Verdict

**PASS** — No defect found in the dispatched changed behavior; merge gates remain independently outstanding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Assurance Context

AP-201 (`spec/assurance/AP-201-detection-minting.md`) applies: wrong evidence interpretation has material impact across coverage consumers. Reviewed the existing selected-inventory path, exact native retained row lookup and retirement/status handling. AP-201 still contains legacy `review_selection`; it has no current `review_policy.operations` and is not interpreted as an enforced workflow instruction. No active exception was supplied or inferred. The focused candidate tests share engine implementation and do not establish independent producer qualification. Whole `make ci`, candidate-wide matrix, and a compatible green independent corpus are unavailable in this review; QA-corpus #31 remains open/unowned in this dispatch. No measurement target or gate is waived.

## Coverage

Deterministic `quoin advise --json` ran: FR-050-AC-3 is authored Test; advisor recommends property-based-testing (Test), matched by universal property shape, mismatch=false, inconclusive=false. A native behavioral integration suite exercises the predicate and controls. Review judgement: Test is appropriate for this deterministic evidence-reconciliation boundary; the integration regression does not claim to implement a quantified property harness or to qualify RT source behavior. Retained exact-row evidence remains FR-078's separate contract.

## Examined Units

```yaml
scope:
- id: FR-050-AC-3
  path: spec/functional/FR-050-declarative-coverage-computation.md
  role: examined
  excerpt: A live reference row whose trace target has no backing `verifies` relation
    appears in the report's unbacked rows with the row id and its target id. When
    its row ID names a module-derived obligation, source backing requires a `verifies`
    relation for that obligation's own ID; a referenced TC or sibling obligation cannot
    supply it. Other reference rows retain the own-ID-or-referenced-ID rule. Explicit
    retained evidence continues to require FR-078's accepted exact row binding.
```
