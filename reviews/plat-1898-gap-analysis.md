---
id: SR-20261115183673
title: "Gap analysis — quire-rs repository"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/, src/, tests/, corpus/"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs", type: references }
---

## Summary

Ran the evidence-backed matrix for the quire-rs repository. It binds the new FR-099-AC-10 tests, while the repository-wide matrix still reports untagged criteria and the evidence store has no run evidence; plan completion was not assessed.

## Verdict

**FAIL** — the computed matrix has 234 untagged criteria, including 158 declared as `Test` and 4 as `Unit Test`; this is a repository-wide gap result, not a claim that all gaps were introduced by PR #747.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The repository matrix reports 234 untagged criteria (158 Test, 4 Unit Test, 72 other declared or unspecified methods), so these criteria have no source-tagged test evidence. | quire matrix: 234 untagged criteria |
| FND-002 | medium | The coverage report has 61 untracked tags outside corpus fixtures; these name no current criterion and include stale or malformed criterion references. | quire coverage: 61 non-corpus untracked symbols |

## Coverage

- Reconciliation: quoin matrix (quoin 0.36.10, quire 0.52.1)
- Plan completion: not assessed
- Criteria: tagged 1136 · untagged 234 · tagged-by-ignored-test 0 · method-without-symbol 141 · qualified-record 0 · retired 0
- Evidence: no run evidence for all 1511 criteria
- Untracked tags: 269 total; 208 occur in corpus fixture inputs and 61 outside corpus fixtures
- Changed criterion: FR-099-AC-10 is tagged by `tests::dependency_move_links_upstream_items_to_local_tests_and_reports_unmatched_items` and `tests::unavailable_upstream_tree_remains_explicit_and_unresolved`
- Reverse-gap / stub check: the changed `src/impact.rs` behavior and tests were inspected; every new helper and the public API carry `#[implements("FR-099-AC-10")]`, both new tests carry `#[trace("FR-099-AC-10")]`, and no new stub was found. The repository-wide reverse-gap inventory was not exhaustively reviewed; this artifact does not claim that portion as complete.
- Semantic review: skipped (optional)
