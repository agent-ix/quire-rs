---
id: SR-2026112016223475846350
title: "Gap analysis - PLAT-1873 cross-module semantic references"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@9a2ad6dec9c066781053502c8d98a531b20bb368; repository matrix and reverse-gap audit; spec/functional/FR-075-model-feature-extraction.md (FR-075-AC-12, FR-075-AC-25); src/semantic/context.rs, src/semantic/target.rs, src/semantic/model.rs, src/semantic/systems.rs; tests/it/semantic_model.rs, tests/it/semantic_systems.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-075", type: references}
---

## Summary

Planless repository gap analysis at PR head `9a2ad6dec9c066781053502c8d98a531b20bb368`. The computed matrix contains 1,490 criteria; 352 have no binders. Repository coverage reports 1,140 backed of 1,444 rows. This is a repository-wide existing coverage deficit; the aggregate was not attributed to this PR. For changed behavior, FR-075-AC-12 has the new tagged integration binder and exercises explicit inventory success, wrong kind, absent ID, and both source loci. FR-075-AC-25 has one existing binder, but it exercises only legacy unchecked import behavior; the new inventory behavior for `Transitions.Emits` has no binder/test. Reverse-gap inspection found no clearly new production behavior outside FR-075-AC-12/25. No implementation stubs were found in the changed resolution paths. Plan completion: not assessed.

## Verdict

**FAIL.** FR-075-AC-25 is not covered for its newly added explicit-inventory path. The repository-wide matrix also reports 352 unbound criteria; the changed-feature assessment is limited to FR-075-AC-12 and AC-25. No plan was supplied. Semantic review was not requested and was skipped.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-075-AC-25's new imported-package inventory branch is not tested: its sole binder `reference_cell_rules` covers unchecked legacy import with no artifact inventory. Add tagged tests for explicit inventory success, unknown id, wrong kind, and untyped target behavior. | tests/it/semantic_model.rs:1343 |

## Coverage

- Matrix: `quoin matrix --repo . --json`; 1,490 criteria, 352 without binders. FR-075-AC-12 has 10 binders including the new `explicitly_indexed_cross_module_references_carry_both_locations_and_refuse_bad_targets`; FR-075-AC-25 has one binder, `reference_cell_rules`.
- Reverse gap and stub scan: changed production code paths examined in `src/semantic/{context,target,model,systems,record,surface,mod}.rs`; no separate unowned behavior or unfinished stub identified.
- Coverage report: `quire coverage --scope . --json`; totals 1,140 backed / 1,444 rows, 1,308 criteria. Diagnostics were present across the repository; no claim is made that this PR resolves those repository-wide items.
- Semantic review: skipped (not opted in).
- Plan completion: not assessed.
