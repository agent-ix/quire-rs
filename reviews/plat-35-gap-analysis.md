---
id: SR-20261103118124
title: "gap-analysis of declaration references and typed relation keys (quire-rs PR 593)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@fbf580d; FR-075-AC-22..27, FR-075-CON-2; FR-076-AC-16..21, FR-076-CON-3..4; FR-069-AC-13; TC-1993..TC-2005; tests/fixtures/semantic/cases*.json"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-075
    type: references
  - target: ix://agent-ix/quire-rs/FR-076
    type: references
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
---

# Gap analysis of PR 593 (PLAT-35, PLAT-55)

## Summary

Tickets: PLAT-35, PLAT-55. This is a planless gap analysis of the PR diff.
Plan completion: not assessed. `quire matrix --format tsv` (quire 0.36.2,
engine 0.50.2) reports every in-scope criterion as `tagged`. The tags are:

- FR-075-AC-22 and CON-2 to TC-1993 (`declaration_references_resolve_to_semantic_ids`)
- AC-23 to TC-1994
- AC-24 to TC-1995
- AC-25 to TC-1996
- AC-26 to TC-1997
- AC-27 to TC-1998 (`declaration_reference_cases`)
- FR-076-AC-16 and CON-3 to TC-1999
- AC-17 to TC-2000
- AC-18 to TC-2001
- AC-19 and CON-4 to TC-2002
- AC-20 to TC-2003
- AC-21 to TC-2004
- FR-069-AC-13 to TC-2005

Each test was read against its AC, sub-case by sub-case. All pass in
`cargo test --test it` (540/540).

`cases.json` and `cases.expected.json` were compared with origin/main.
Neither file loses or changes a record: 18 old cases, 36 new ones, 18 added
(9 `plat35-*`, 9 `plat55-*`), and `$comment` is unchanged. The Python
binding returns JSON equal to `cases.expected.json` for all 36 cases
(`pytest tests/python/test_semantic.py`, 4 passed, on a locally built
abi3 wheel).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [coverage] FR-076-AC-16 requires that "an `aggregate_root` with two `emits` rows yields two elements in row order". The TC-1999 test uses two rows that both name `event-001` (once bare, once as `ix://agent-ix/shop/event-001`), so the expected value is `[event-001, event-001]` and a reversed or reordered lowering would still pass. Add a second event to the bundle and assert `[event-001, event-002]`. | tests/it/semantic_relations.rs:668-680 | correct-requirement-no-evidence |
| FND-002 | low | [coverage] FR-076-AC-20 ("a type whose schema has no typed key yields a record byte-identical to AC-1") is tested only with a schema that declares `relations`, and the comparison is with/without `dataSchema` on an ad-hoc request rather than against the AC-1 fixture. No test covers a schema with neither a typed key nor `relations`, the case that judgement call (1) decides: rows lower into `relations` and are not refused `verb-not-typed-key`. | tests/it/semantic_relations.rs:837-848 | correct-requirement-no-evidence |

## Verdict

Coverage is complete at the tag level, and the tests are real behavioural
tests, not stubs. Two oracles are weaker than their AC: FND-001 cannot
detect a row-order bug, and FND-002 leaves the no-typed-key, no-relations
schema untested. The fixture regeneration only adds records. Python parity
holds.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, commit 9231e53. The bundle gains `event-002`. The rows are `shipped -> event-002` then `placed -> ix://.../event-001`, and the test asserts `[event-002, event-001]`, which is reverse id order, so a reordering would now fail. Test passes. |
| FND-002 | fixed | Round 1, commit 9231e53. TC-2003 now extracts the AC-1 `relationships.md` fixture, relation vocabulary included, with and without `dataSchema: {type: object}` (no typed key, no `relations`). It asserts byte-identical output, and that `relations`, `relationSources` and `diagnostics` equal `relationships.expected.json`. Test passes. |
