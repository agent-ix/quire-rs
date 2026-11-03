---
id: SR-20261103118114
title: "Gap analysis — quire-rs relationship metadata"
type: SpecReview
analysis: gap-analysis
scope: "spec/, src/, tests/, crates/quire-fixtures/"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/FR-040"
    type: references
  - target: "ix://agent-ix/quire-rs/FR-076"
    type: references
---

## Summary

Repository-driven audit of quire-rs criteria, computed matrix, tests, and changed source. Plan completion was not assessed. The relationship metadata change is traced to FR-040 and FR-076, while the repository-wide matrix retains existing untagged criteria.

## Verdict

**FAIL** — the repository matrix still has 271 untagged criteria; this is a pre-existing repository-wide gap, not a regression introduced by this change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | 271 acceptance criteria have no test bound by the computed matrix; the changed FR-040 and FR-076 paths are tagged, but repository-wide coverage remains incomplete | computed quire-rs matrix; spec/ |

## Coverage

- Reconciliation: quoin matrix --repo /private/tmp/quire-rs-part-of --json (Quoin 0.29.3, Quire 0.38.1 / engine 0.51.0); no run evidence was read.
- Criteria: 642 tagged, 271 untagged, 0 tagged-by-ignored-test, 107 method-without-symbol; 1020 total.
- Evidence: no run evidence.
- Plan completion: not assessed
- Reverse gap and stubs: rechecked the changed production and test surface against the previous repository audit. The new composite field and extraction behavior are owned by FR-040/FR-076; no new CLI route, unowned behavior, source stub, or test stub was introduced.
- Semantic review: skipped.
