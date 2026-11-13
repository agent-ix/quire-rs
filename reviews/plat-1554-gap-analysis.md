---
id: SR-20261113045745
title: "Gap analysis — PLAT-1554 FR-084 two-way relation coverage report (quire-rs PR 666)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@8b45bcb; spec/functional/FR-084-two-way-relation-coverage-report.md, spec/functional/FR-058-upward-trace-completeness.md, spec/functional/FR-050-declarative-coverage-computation.md, src/corpus/relation_report.rs, src/corpus/required_relations.rs, tests/it/relation_report.rs, tests/fixtures/traceability/relation-report/manifest.yaml"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-084", type: references}
---

## Summary

Ticket: PLAT-1554. This is a planless gap analysis of PR head `8b45bcb` against FR-084 on main. Matrix: `quire matrix --scope .` (quire 0.44.0, engine 0.59.1; quoin 0.30.1). FR-084-AC-1..16 are all tagged, each bound to a test in `tests/it/relation_report.rs`. AC-17, CON-1 and CON-2 are `method-without-symbol` (Inspection), and the reviewer inspected them:

- AC-17: no hazard, threat, mitigation or safety name appears in the engine source.
- CON-1: verification is read only from `coverage_matrix`.
- CON-2: no manifest key is added.

There are no stubs, `todo!()` or placeholder returns. The 15 focused tests pass (`cargo test --test it relation_report`, run by the reviewer). The optional semantic review was not run, because it is opt-in and was not requested. The reviewer read every test against its AC by hand.

The tests can fail. AC-1 and AC-15 run the real FR-050 `compute` over tagged sources. AC-5, AC-6 and AC-15 cover every `CoverageMatrixStatus`. AC-16 cross-checks the report against `validate_bundle` findings. The exception is the key-order half of AC-12 (FND-001).

Plan completion: not assessed

## Verdict

**Changes requested: one medium, two low.** The behavioural ACs are bound to tests that can fail. The serialization AC is only partly guarded. Its key-order assertions use a forward substring search over the whole report, so several wrong orders still pass, and the document, `linked`, `dangling` and `kind_mismatch` shapes and the cross-repo row order are not asserted. The code-level findings are in SR-20261113045744.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-084-AC-12 key-order checks cannot fail for several wrong orders. `in_order` and the row loop search forward with `find` over the whole serialized report, so a key out of place is matched in a later section or row. If `summary` moved after `rows`, `"summary":` is found at the end of section 0, and `"rows":` in section 1. If `selector` moved after `state`, `"state":` is found in a later section's `summary`. The Document (`id, path, archetype, repo, verification, criteria`), `linked` resolved and dangling entries, `dangling` and `kind_mismatch` key orders are never asserted, and nor is the row order by repo, then path, then id across members. Fix: assert the exact `serde_json::to_string` output of a small fixture holding each shape, including a two-member corpus. | tests/it/relation_report.rs:571-630 |
| FND-002 | low | FR-084-AC-2's "ordered by repo then id" is tested only inside one member: RULE-001 before RULE-002, with no identity. Nothing asserts that `linked` entries from several members sort by repo (absent first), or that dangling `linked` entries follow the resolved ones in (reference, verb) order. Fix: extend `rows_carry_the_member_identity_and_link_across_members` with two linking members, or cover it in the exact-JSON fixture of FND-001. | tests/it/relation_report.rs:231-270, tests/it/relation_report.rs:795-847 |
| FND-003 | low | Reverse gap: the public error `RelationReportError::MemberCount` (and `NoRelationsDeclared`, see SR-20261113045744 FND-001) has no owning AC and no test. The member-count guard protects the `self.members[member]` indexing, so it is real behaviour that nothing exercises. Fix: add a test passing the wrong number of `RelationReportMember`s, and state the error in FR-084 (with a CR note) or keep it as a documented precondition. | src/corpus/relation_report.rs:289-295 |

## Coverage

- FR-084-AC-1..16: 16/16 tagged. Bindings read: AC-1..11 and AC-13..16 are correct, AC-12 is partial (FND-001), and AC-2 does not cover repo ordering (FND-002).
- FR-084-AC-17, CON-1, CON-2: Inspection, inspected clean.
- Reverse gaps: FND-003. Stubs: none.
- Semantic review: skipped (opt-in); the reviewer read each AC against its test by hand.
- Plan completion: not assessed

## Dispositions

Round 1, at head `b2f9b44`. `quire matrix` shows FR-084-AC-1..16, AC-18 and AC-19 tagged, and the new tests pass.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `serialization_is_exactly_the_stated_shape_and_order_across_members` compares the full `serde_json::to_string` output of a two-member corpus with a constant. The constant covers every shape: document, selector, resolved, untyped and dangling `linked` entries, `dangling`, `kind_mismatch`, zero summaries and rows across repos. Any key or order change now fails it. |
| FND-002 | fixed | The same test (also tagged AC-2) asserts `linked` ordered by repo then id across members o/a and o/b, with dangling entries after the resolved ones in reference order (`ghost-a` before `ghost-b`). |
| FND-003 | fixed | The new FR-084-AC-19 states the member-count error, and `a_member_input_count_that_differs_from_the_corpus_is_an_error` asserts `MemberCount { members: 2, supplied: 1 }`. `NoRelationsDeclared` is gone (code-review FND-001). |
