---
id: SR-20261113045751
title: "Gap analysis — PLAT-1418 design-element join engine (quire-rs PR 672)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-086-design-element-join.md, spec/functional/FR-058-upward-trace-completeness.md (AC-23..29), spec/functional/FR-038-okf-bundle-validation.md (AC-12), tests/it/design_join.rs, src/corpus/relation_edges.rs, src/corpus/relation_report.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-086", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-058", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-038", type: references}
---

## Summary

Ticket: PLAT-1418. PR head: `30ac10c`. Plan completion: not assessed (planless). This analysis checks the computed matrix (`quire matrix --scope .`, quire 0.44.0 / engine 0.59.1) for FR-086, FR-058-AC-23..29 and FR-038-AC-12, and compares each tagged test's assertions with the clauses of its criterion.

## Verdict

**PASS with two medium findings and one low finding.**

- **Tagging.** Every FR-086 AC and CON is tagged, except FR-086-AC-13 (an Inspection AC, checked clean by inspection in the code-review artifact). FR-058-AC-23..29 and FR-038-AC-12 are all tagged.
- **Assertions match their criteria.** These clauses are asserted directly:
  - AC-1: rows, state, `archetype` and `check`.
  - AC-2: the dangling inverse is listed as authored, with a `to: []` control.
  - AC-3 and AC-4: order and membership.
  - AC-6: both halves.
  - AC-7: both directions, including the mismatch fields.
  - AC-8: lexical order, `[]` for a whole-document edge, and serialized placement.
  - AC-9 and AC-10: load errors, the finding and the join error.
  - AC-11: both postures and `off`.
  - AC-15: under the AC-15 ruling in the code-review artifact.
  - CON-1 and CON-2.
- **No stubs or inflation.** No stub was found and no coverage is inflated. All new production code is owned by FR-086 / FR-058 / FR-038.
- **FR-058-AC-25 test.** It is weaker than its criterion. That gap is recorded with the code defect it hides, as code-review SR-20261113045750 FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Two clauses of FR-086-AC-14 are not tested. Document-order independence is asserted vacuously: `load_repo` sorts by path (walk.rs:228), so writing the same files in reverse order builds an identical tree and `render(&reversed)` cannot differ. Member-order independence is not exercised at all, because no test builds a two-member corpus in both orders. | tests/it/design_join.rs:842-866 |
| FND-002 | medium | Two clauses of FR-058-AC-24 are asserted by no test. The byte-identity clause has no comparison of serialized output for a relation whose verbs have no counterpart; the test checks row states only. The clause "a label that is a forward key is not read as an inverse" has no test either. | tests/it/design_join.rs:377-435 |
| FND-003 | low | The FR-058-AC-26 outgoing-mismatch clause "carrying that document's member as `repo`" is unasserted. The test corpus has one member and no identity, and the finding message is checked only for ids and verb. | tests/it/design_join.rs:508-545 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-086-AC-1 .. AC-12, AC-15 | examined | Tagged and asserted |
| FR-086-AC-13 | examined | Inspection; clean |
| FR-086-AC-14 | examined | FND-001 |
| FR-086-CON-1, CON-2 | examined | Tagged and asserted |
| FR-058-AC-23, AC-27, AC-28, AC-29 | examined | Tagged and asserted |
| FR-058-AC-24 | examined | FND-002 |
| FR-058-AC-25 | examined | Weak test; see code-review FND-001 |
| FR-058-AC-26 | examined | FND-003 |
| FR-038-AC-12 | examined | Tagged and asserted, Okf and Strict |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 8e969ed: the AC-14 test now reverses `load.documents` after `load_repo`. `Spec::build` keeps the given order, so the reversal is real. It also swaps the members of a two-member identified corpus, and both orders must render byte-identical. |
| FND-002 | fixed | 8e969ed: new test `a_forward_key_is_not_read_as_an_inverse_and_unrelated_verbs_are_unchanged` covers the forward-key clause and asserts the serialized row (state, linked id and verb, empty dangling and kind_mismatch) for a verb with no counterpart. |
| FND-003 | fixed | 8e969ed: new test `an_outgoing_mismatch_carries_the_repo_of_the_obliged_document` asserts that a two-member `validate_corpus` gives exactly one `target-kind-mismatch` with `repo == o/a`, naming `GLY-001` and `kindled_by`. |
