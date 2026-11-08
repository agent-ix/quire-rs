---
id: SR-20261103118118
title: "Gap analysis — PLAT-2 row-aware trace binding (quire-rs PR 587)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-050-declarative-coverage-computation.md, src/coverage.rs, src/coverage/diagnostics.rs, src/coverage/reconcile.rs, src/coverage/row_binding.rs, tests/fixtures/corpus_cases/row-aware-trace-binding.json, tests/it/coverage_rollup.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
---

## Summary

Ticket: PLAT-2. Frozen PR head: `a75246390043717327cb2786214d1806ce611cc8`. This is a planless traceability and reverse-gap audit for the PR's coverage-binding slice. `quoin matrix --repo /home/peter/dev/worktrees/quire-rs-plat-2 --json` ran successfully. The installed Quoin CLI has no `gap-analysis` command, so reverse-gap and stub checks were performed manually against the changed production and test files. No semantic review was opted into. The QA corpus linked PR is outside this analysis scope.

## Verdict

**FAIL** — the computed repository matrix has 269 untagged criteria. The PR's new test is tagged, but the no-matrix-row fallback in production still allows tags without row evidence to back criteria (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed matrix contains 269 `untagged` criteria across the repository, so the criterion-to-test traceability chain is incomplete. All 1,020 criteria report `no run evidence`; static tags do not establish execution evidence. | quoin matrix output |
| FND-002 | high | `RowBindings::eligible_relations` preserves a criterion-tagged relation whenever the symbol has no declared matrix row (`!has_matrix_row`). That lets criterion backing survive a missing or unrecognized row, so the ticket's required row-to-criterion binding is not enforced across this path. | src/coverage/row_binding.rs:133-139 |

## Coverage

| Check | Result |
| --- | --- |
| Requirements in computed matrix | 94 |
| Criteria in computed matrix | 1,020 |
| Static status: tagged | 644 |
| Static status: untagged | 269 |
| Static status: method-without-symbol | 107 |
| Evidence status: no run evidence | 1,020 |
| PLAT-2 regression tag | present on `a_criterion_tag_only_backs_its_own_matrix_row` |
| Reverse-gap scan of changed production paths | coverage implementation remains within the existing FR-050 subsystem; no added standalone behavior outside that ownership was identified |
| Stub/TODO scan of changed production paths | no stub, TODO, FIXME, `todo!`, or `unimplemented!` found |
| Semantic review | skipped — not opted into |
| Plan completion | not assessed |

