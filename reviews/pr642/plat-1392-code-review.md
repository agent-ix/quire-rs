---
id: SR-20271
title: "Code review — PLAT-1392 row binding constrains only when matrix rows exist (quire-rs PR 642)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/coverage/row_binding.rs (RowBindings::from_scanned_rows), tests/fixtures/corpus_cases/row-aware-trace-binding.json; consumers: src/coverage/reconcile.rs, tests/it/coverage_rollup.rs, tests/it/id_column_lists.rs (read only)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-050", type: references}
---

## Summary

Ticket: PLAT-1392. PR head: `412a0ab`. Code review with the rust-review lane folded in. The change moves the "no rows" early return in `RowBindings::from_scanned_rows` ahead of the `constrained_ids` extension, and widens it from `None` to `None or empty`, so a declaration that scanned no rows no longer constrains its targets' minted ids.

## Verdict

**FAIL.** The source change is correct for the stated case, but the PR's own test fails, and the fix does not cover a repo that has an Inspections document and no TestMatrix.

- **Trigger.** `rows` is `None` only when the declaration's pattern fails to compile. A missing TestMatrix gives `Some(vec![])`, so before this PR every such repo constrained every criterion and rejected every tag. The new `None or empty` trigger is the right one for that case.
- **Empty or misparsed matrix.** A TestMatrix whose section is missing or holds no table now leaves binding off without a row-binding diagnostic. That is acceptable. The `test-case` mint already reports `SectionMatchesNothing` or `SectionHoldsNoTable`, and a table with zero rows is a matrix with no tests yet, where PLAT-2 would reject every tag anyway. A table with rows but wrong headers still constrains and fails loudly. This is not a finding.
- **Other declarations in the shipped module.** `verification`, `nfr-verification` and `interface-verification` never constrain, because `test-case` is not an obligation target. `functional-coverage` never constrains, because no trace target matches its section. `traces-to` is the intended constrainer. `inspection-obligation` also constrains (FND-002).
- **PLAT-2 with a matrix present.** The rule is unchanged: the loop body for a declaration with rows is the same as before.
- **Fixture discrimination.** I fixed the harness locally (not pushed) by relaxing the `issue_ref` assert and adding `backed: 2, total: 2`. The new case then passes on the PR head. With `src/coverage/row_binding.rs` reverted to `origin/main`, it fails with `backed count left: 0 right: 2`. So the case discriminates once FND-001 is fixed.
- **Rust.** The code is idiomatic (`let-else` with `Option::filter`) and adds no clones, panics or allocation. Ceremony: none found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new fixture case breaks `coverage_rollup::a_criterion_tag_only_backs_its_own_matrix_row` (TC-734), which loops every case in the file. It asserts `case["issue_ref"] == "agent-ix/quire-rs#467"`, but the new case carries `"PLAT-1392"`, so the test panics at tests/it/coverage_rollup.rs:311. The case also has no `expect.backed` or `expect.total`, which the same loop reads with `.expect(...)` at lines 341-348, so it would panic there next. The PR's own regression case therefore never runs its expectations. Fix: replace the hardcoded `issue_ref` assert with a non-blank check (or accept either ref), and add `"backed": 2, "total": 2` to the case. Both were verified locally. | tests/it/coverage_rollup.rs:311; tests/fixtures/corpus_cases/row-aware-trace-binding.json:174-201 |
| FND-002 | medium | The trigger is per declaration, so a repo with an `Inspections` document and no TestMatrix still has every criterion tag rejected. In the shipped module, `inspection-obligation` (archetype Inspections, row target `inspection`, targets acceptance-criterion, nfr-acceptance-criterion, interface-acceptance-criterion and constraint) passes every filter in `from_scanned_rows`. When it has rows it constrains every minted criterion, and the only allowed rows are inspection ids, which tests never tag. A local probe case (Inspections doc with one row and no TestMatrix) gave backed 0 of 2 on the PR head. No current repo hits this: QSL, quire-rs, quoin, engineering-assurance, quire-cli and spec-artifacts-process on main all have a TestMatrix or no Inspections doc. It is still the same defect this PR fixes, reached through a second declaration. Fix: let an inspection-style declaration (whose row ids are not tagged by tests) add allowed rows without creating the constraint, and add an Inspections-only fixture case. | src/coverage/row_binding.rs:66-82 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/coverage/row_binding.rs from_scanned_rows | examined | Correct for the TestMatrix case; FND-002 |
| src/coverage/row_binding.rs eligible_relations | context_only | Unchanged; an id is constrained only via constrained_ids |
| src/coverage/reconcile.rs scan_reference_rows | context_only | `None` only for a bad pattern; a missing archetype gives `Some(vec![])` |
| tests/fixtures/corpus_cases/row-aware-trace-binding.json new case | examined | FND-001 |
| tests/it/coverage_rollup.rs a_criterion_tag_only_backs_its_own_matrix_row | examined | Fails on PR head (FND-001) |
| tests/it/id_column_lists.rs row-binding tests | context_only | Read cases[0] only; pass |
| spec-artifacts-process manifest document_references | context_only | inspection-obligation constrains (FND-002) |

## Verification

Focused runs on PR head `412a0ab`. `cargo test --test it a_criterion_tag_only_backs_its_own_matrix_row` fails with the `issue_ref` assert. `row_binding_never_joins_a_list_target_while_a_single_name_target_joins` and `corpus_cases_hold` pass. The discrimination and Inspections probes were local, uncommitted edits, reverted afterwards.
