---
id: SR-20279
title: "Gap analysis of PLAT-1831 outputCatalog producers and catalog diff (quire-rs PR 714)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; FR-073-AC-7..AC-12 against PR 714 diff (head 1b095e7f714ffbcb0827bff82c84f4c7a987b2e3): src/clauses.rs, schemas/output/clause-diff-v1.schema.json"
review_set: subset
date: 2026-11-23
relationships:
  - target: "ix://agent-ix/quire-rs/FR-073"
    type: "references"
---

## Summary

Ticket: PLAT-1831. `quire matrix --scope . --format json` reports FR-073-AC-1..AC-12 all `tagged`; AC-7..AC-12 each bind one new unit test in `src/clauses.rs`. Each test was read for whether a wrong implementation makes it fail. AC-12 was checked by mutation: removing `outputCatalog` from the schema's top-level `required` makes `diff_schema_requires_output_catalog` fail (panic at src/clauses.rs:1129), so the test is real. AC-10 is a "must not reject" criterion: disabling producers validation cannot fail it by design; adding a vocabulary check would, which is the intended failure. No reverse gaps: every new item (`CatalogEntry`, `ChangedCatalogEntry`, `OutputCatalogDiff`, `diff_output_catalogs`, `validate_producers`) is owned by FR-073. No stubs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-9's clause "the error names the `outputCatalog` key" is not tested. The test asserts only that the message contains the literal `outputCatalog`; it never checks the entry key (`test-result`), so dropping `{name:?}` from the message would stay green. Fix: also assert `error.contains("\"test-result\"")`. | src/clauses.rs:1059-1075 |
| FND-002 | low | AC-12's conforming document has only a `changed` entry; the `added`/`removed` `CatalogEntry` shape (serde `flatten` of `ExpectedOutput` against `$defs.catalogEntry` with `additionalProperties: false`) is never validated against the schema, and AC-5's test diffs identical catalogs. Fix: in the AC-12 test also validate the diffs with an added and a removed entry (with and without `producers`). | src/clauses.rs:1115-1130; schemas/output/clause-diff-v1.schema.json:61-71 |

## Verdict

CONDITIONAL. All criteria are tagged and the tests for AC-7, AC-8, AC-10, AC-11 and AC-12 can fail for the intended reason; AC-9 misses one clause (FND-001).

## Coverage

- Criteria examined: FR-073-AC-7..AC-12 (6/6 tagged, 0 untagged); AC-1..AC-6 context only.
- Mutation evidence: AC-12 schema `required` mutation fails the test.
- Semantic review: skipped (not requested).
- Plan completion: not assessed

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (head 2fdddf29): the AC-9 test adds `assert!(error.contains("\"test-result\""), "{producers:?}: {error}");` for every rejected case. |
| FND-002 | fixed | Round 1: `diff_schema_requires_output_catalog` validates diffs base→with, base→empty, empty→base, empty→with, with→empty and with→without, which covers added and removed entries with and without `producers` and a changed entry whose `producers` goes from `["run"]` to absent. |
