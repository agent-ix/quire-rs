---
id: SR-20261103118134
title: "Gap analysis — PLAT-1356 Clock, Quantity<u>, unit tokens, importedArtifacts (quire-rs PR 611)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; FR-070-AC-11..14, FR-075-AC-19 and the PR 611 diff (src/semantic/properties.rs, src/semantic/units.rs, src/semantic/context.rs, src/semantic/model.rs, tests/it/semantic_model.rs, tests/it/semantic_properties.rs)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-070", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-075", type: references}
---

## Summary

Ticket: PLAT-1356. Frozen PR head: `e4f73ec`. Planless gap analysis scoped to the PR. `quire matrix --scope <worktree> --format json` reports FR-070-AC-11, AC-12, AC-13, AC-14 and FR-075-AC-19 `tagged`; FR-075-AC-16, 17, 18, 20 and 21 `untagged`, which is expected: they are outside PLAT-1356's scope, stay `🚧` in `spec/tests.md`, and nothing in the PR claims them. The tagged tests run and pass (see the code-review SR-20261103118133 for gate output).

Oracle check per TC: TC-1945 (`clock_is_a_native_type_with_no_unit_or_decimal`, `clock_field_record_validates_against_semantic_v1`) is a real oracle — exact target JSON, table/fence equality, hostile bundle object `Clock`, three refusals, and semantic-v1 validation through the `extract_request` helper; FieldDecl.json validation is covered implicitly by the zero-error assertion. TC-1946 is real: id and title resolution, three placeholder identities with exact reasons, two-way ambiguity, bare `Quantity`. TC-1947 is real for the ten malformed tokens, `row-errors` availability, `UNIT_NOT_ALLOWED` and the `ref item` refusal. TC-1952 is real at the resolver level, every AC-14 clause has an assertion. TC-1950 is real for the refusals (reason, section, span line) and absence of members.

No unowned code: every new function traces to FR-070 or FR-075-AC-19. No stubs on the tagged path; the three `read: None` table entries are the declared refusal-only surface of AC-19 (their declared-locator behaviour is code-review FND-003). Semantic review (intent↔test↔code) was run informally for these five ACs as part of the review; no separate fan-out.

Plan completion: not assessed

## Verdict

**CONDITIONAL** — every in-scope criterion is tagged by a test that can fail; the findings below are weaker-than-AC oracles, all low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-070-AC-14 is tested only through `resolve_token` unit tests; no test drives a qualified `Quantity<pkg::name>` cell through `extract_fields`, so the placeholder identities (`ix://<own>/unresolved/ms` with `unknown-token`/`import-unresolved`/`no-bundle-index`) and "distinct from the imported one in the same record" are not asserted end to end. | src/semantic/units.rs:256-360, tests/it/semantic_properties.rs:690-893 |
| FND-002 | low | TC-1950 asserts that an artifact with none of the tables has no `dimension`/`terms`/`unit` member and no refusal, but FR-075-AC-19 asks for a record byte-identical to the FR-072 record; no byte comparison is made. | tests/it/semantic_model.rs:1537-1572 |
| FND-003 | low | FR-070-AC-13 requires the `ref item` refusals "at the fence line"; the test checks the code only, not the locus. | tests/it/semantic_properties.rs:885-892 |

## Coverage

| Criterion | Matrix status | Tests | Result |
| --- | --- | --- | --- |
| FR-070-AC-11 | tagged | TC-1945 (2 tests) | covered |
| FR-070-AC-12 | tagged | TC-1946 | covered |
| FR-070-AC-13 | tagged | TC-1947 (2 tests) | covered; FND-003 |
| FR-070-AC-14 | tagged | TC-1952 (5 unit tests) | covered at resolver level; FND-001 |
| FR-075-AC-19 | tagged | TC-1950 | covered; FND-002 |
| FR-075-AC-16..18, 20, 21 | untagged | none | out of scope (PLAT-287 later slice), `🚧`, not claimed |

Plan completion: not assessed

## Dispositions

Round 1, fix commit `e20a773`.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `qualified_quantity_cells_resolve_through_imported_artifacts` (tests/it/semantic_properties.rs:920) drives bare, title, and two qualified `Quantity<>` cells through `extract_fields` in one record and asserts the owner-qualified identities, the cross-package ambiguity, and the placeholders with `unknown-token`, `import-unresolved` and `no-bundle-index` |
| FND-002 | fixed | TC-1950 now compares the serialized record of an artifact with none of the tables against the record extracted with no body extraction, byte for byte (tests/it/semantic_model.rs:1562-1567) |
| FND-003 | fixed | The `ref item` refusal test asserts the diagnostic line equals the fence line for both `Quantity<ms>` and `Clock` |
