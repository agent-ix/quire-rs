---
id: SR-20261102118111
title: "PLAT-1149 spec-review-base review"
type: SpecReview
analysis: base
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

The edited own-obligation clause defines a falsifiable boundary, preserves aggregates and explicitly defers retained evidence identity to FR-078. Native identities and declared vocabulary drive behavior without baking RT or ISO names into the engine. The existing TC-734 verification spelling was preserved; this change does not author a parallel matrix. CR-PLAT-1149 records reproduction and limits fixture qualification claims. Historical measured revisions in that report are context, not a new pin or instruction to retain one. No FR/NFR/StR statement, domain object, relationship edge or system responsibility was edited, so EARS/integrity/object/dependency/scope-boundary analyses are not applicable to this delta.

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

## Follow-up disposition: corpus grader controls

This follow-up examines PR 515's change to `tc1024_unbacked_rows_and_groups_are_exact` in `tests/corpus_cases.rs`. No production coverage predicate, corpus grader, expectation file, workflow or CI gate was changed by this test repair. The original findings and review prose above remain historical.

The real bank case is still executed first and its full authored contract must pass. The test then uses a cloned typed report and field-specific `CaseExpect` values through the existing `grade_with` entry point. Empty payload/empty expectation and one-row payload/matching expectation pass. Missing-row and extra-row adversaries both fail at `L2Localised`. The nonempty witness is required with `expect`, so these assertions cannot silently skip. The same unchanged production grader compares the entire row vector. There is no fake passing bank case, bank-expectation relaxation or separate grader implementation. Other fields are intentionally unasserted only in the local unit controls, while the real-case contract and `corpus_cases_hold` continue to grade the bank normally.

QA-corpus PR 31 is confirmed merged. The former named QA ownership/dependency condition is resolved. Inspected existing logs show the focused TC-1024 test passing, the complete `corpus_cases` target passing 21/21 (including `corpus_cases_hold` and determinism), and clippy/formatter passing. This reviewer ran no Cargo gate. The full hosted rerun remains the lead's final gate; these results do not claim hosted aggregate green or a repository-wide assurance audit.

### Current disposition

No new finding. The prior bank/testing condition is resolved; changed-scope review is PASS. Historical repository-wide audit limitations remain limitations of this review and are not a newly identified PR defect. No remaining code-review blocker was found in this fix.

### Additional examined unit

```yaml
scope:
- id: FR-065-AC-28
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: context_only
  excerpt: 'A case asserting `unbacked_rows` or `groups` fails on a payload carrying an extra entry, a missing entry, a changed field or a different order; a case asserting an empty list fails on a payload carrying any entry.'
- id: tc1024_unbacked_rows_and_groups_are_exact
  path: tests/corpus_cases.rs
  role: examined
  excerpt: 'let outcome = grade_with(case, &empty_report, &nonempty_expect);'
```
