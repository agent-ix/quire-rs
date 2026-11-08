---
id: SR-20261103118117
title: "Code review — PLAT-2 row-aware trace binding (quire-rs PR 587)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/coverage.rs, src/coverage/diagnostics.rs, src/coverage/reconcile.rs, src/coverage/row_binding.rs, tests/fixtures/corpus_cases/, tests/it/coverage_rollup.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
---

## Summary

Ticket: PLAT-2. Frozen PR head: `a75246390043717327cb2786214d1806ce611cc8`. The review covered the changed Rust implementation and regression fixtures, including the integrated Rust lane. No duplication or vendoring was introduced. The new declarative cases exercise the production coverage path and assert both backing and diagnostic behavior. The repository's active AP-201 applies and is recorded below.

## Assurance Context

AP-201, “Quire detection and minting assurance profile,” applies to coverage collection and trace binding. It calls for a failure fixture and healthy control, deterministic reporting, and a complete source-to-test evidence chain. The PR adds declarative failure/control cases and a shared-row case. The fresh matrix reports 269 untagged criteria and no run evidence across 1,020 criteria; those repository-wide gaps are recorded separately in the gap-analysis artifact. The controlled QA corpus is intentionally outside this review's PR scope.

## Verdict

**FAIL** — one high-severity defect leaves criterion evidence eligible when its symbol has no declared matrix row. The focused Rust checks pass, but this does not satisfy the ticket's row-binding contract.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Criterion tags still count as backing when the tagged test has no declared matrix row, because `!has_matrix_row` admits the relation. This preserves the false-backing path whenever the test case row is absent or unrecognized, contrary to PLAT-2's requirement that criterion backing come only from a matrix row that cites it. | src/coverage/row_binding.rs:133-139 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| PLAT-2 acceptance: mismatched-only test row leaves criterion unbacked and emits a diagnostic; matching row remains backed | examined | FND-001: the no-row fallback still admits the tag |
| PLAT-2 additional acceptance: a row citing multiple criteria is backed only when each criterion has evidence | examined | Implemented in reconciliation and covered by the shared-row case |
| FR-050-AC-3: reference row backing and its own obligation identity | examined | New all-criteria row proxy behavior is exercised; no separate finding beyond FND-001 |
| FR-050-AC-48: computed coverage matrix entries and binders | examined | Eligible relations feed the matrix |
| FR-050-AC-49: per-criterion computed status | examined | Eligible relations determine static status |
| Changed Rust diagnostics and fixture assertions | examined | Stable reason and guidance are present; failure and control fixtures assert outcomes |

## Verification

`cargo fmt --all -- --check`, `git diff --check`, and `cargo clippy --locked --all-targets -- -D warnings` passed. `cargo test --locked --test it a_criterion_tag_only_backs_its_own_matrix_row` and `cargo test --locked --test it coverage_rollup::obligation_reference_rows_require_their_own_evidence` passed. The build warning reports that the optional `@agent-ix/spec-objects-architecture@0.10.1` package could not be fetched; its two dependent tests skip by design. The aggregate gate was not run by this reviewer.

