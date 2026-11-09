---
id: SR-16391
title: "Gap analysis — PLAT-313 id_column as one name or a list (quire-rs PR 616)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; src/traceability.rs, src/corpus/declared_tables.rs, tests/it/id_column_lists.rs, tests/fixtures/corpus_cases/id-column-lists.json"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
---

## Summary

Ticket: PLAT-313. PR: agent-ix/quire-rs#616, head `6166414`. Plan completion: not assessed (planless). This analysis maps FR-050-AC-63 and TC-2095..TC-2100 to the tagged tests in `tests/it/id_column_lists.rs` (found with `quire trace --id`) and to the production code in the diff. For each clause it asks whether the test would fail on a wrong implementation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The column-order clause of AC-63 ("ids of every header that matches … in column order within each row") has no test that can fail. Every assertion reads `CoverageReport.minted_targets`, which `reconcile.rs:106-108` sorts by `(target, document, id, line)`. Ids from one row therefore always come out in lexical order. In the corpus list case (`[TC *, test id]` over `Test ID | TC Ref`), declaration order gives `T-2, T-1` for row X-1, and the test still passes because the output is sorted. An implementation that emits ids in declaration order passes every test. Fix: add a test against `declared_tables::mint` or `ScannedRow::ids`, which keep row order. Use a row whose matched columns hold ids in reverse lexical order, declared in the opposite order (e.g. `[B, A]` over `\| A \| B \|` holding `Z-1 \| A-1`), and assert `["Z-1", "A-1"]`. | tests/it/id_column_lists.rs:154 |
| FND-002 | low | TC-2100 says "a row-binding declaration never joins a sequence or wildcard target". `row_binding_never_joins_a_list_target_while_a_single_name_target_joins` widens `test-case` only to `[ID, Other]`. The wildcard half, a `'I*'` entry, is not exercised. Today both shapes go through `single_name()`, so the gap is latent. Fix: run the same assertion with `id_column: 'I*'`. | tests/it/id_column_lists.rs:269 |
| FND-003 | low | Load validation is per manifest only (`loader/manifest.rs:550`). `merge_traceability` merges trace targets first-wins by name (`loader/mod.rs:1416-1420`). Module A can declare target `X` with a list `id_column` while module B declares a single-name `X` plus an `obligations:` source bound to it. B passes its own validation, and the merged model then binds B's obligation to A's list target. `obligation::resolve` gets `None` from `target.id_column()?` (`obligation.rs:435`), so B's obligations disappear silently, and `reconcile.rs:883` skips retirement. AC-63 says that binding fails load. The same gap already exists for a `from: list_item` target (CR-203), so this PR extends an existing class rather than creating it. Fix: re-run the obligations/target shape check on the merged model, or have `resolve` report a declaration whose target is not single-column instead of dropping it. | src/loader/mod.rs:1416 |

## Verdict

Mergeable after one fix round. FND-001 (medium) needs one discriminating test. FND-002 is one more loop value. FND-003 is low. It can land in this PR as a merged-model check, or be deferred with a stated reason, because the same gap exists for `list_item` targets.

Clean, with tests that can fail. Each of these has a test that a wrong implementation would break:

- One declared column reads no other: `mint("ID", WIDE)`.
- A repeated id in one row mints once and distinct ids each mint: `[Test ID, Alt ID]` expects `T-1, T-2, T-9`. Without dedup the sorted list would hold two `T-1`.
- Case-insensitive exact and `*` entries: `'test note'` and `'*test id'`.
- No section-number normalization on column headers: scalar `Test ID` against `2. Test ID` mints nothing.
- `IdColumnMatchesNothing` fires only when no entry matches and names every entry: one entry matching, a wildcard matching, and none matching.
- An empty list, a blank entry and a whitespace scalar each fail load naming `test-case` and `id_column`.
- A one-name declaration serializes as a scalar and a list as a list.
- A list or `*` target bound by `obligations:` fails load. An unbound list target loads, and the single-name row-binding control joins.

A header matching two entries cannot be told apart from within-row dedup by any output, so the dedup test covers it. Corpus cases exist for Python, TypeScript and Rust, each with a list case, a scalar control and a one-column control. In all three, the `T-2` binding through the `*` column flips `backed`, so the binding path is exercised too. All production code in the diff traces to AC-63.

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-050-AC-63 | examined | FND-001, FND-002, FND-003 |
| TC-2095 | examined | Clean |
| TC-2096 | examined | Clean |
| TC-2097 | examined | Clean |
| TC-2098 | examined | FND-001 (column order) |
| TC-2099 | examined | Clean |
| TC-2100 | examined | FND-002 |
| FR-050-AC-60 | context_only | table_row-only scope consistent |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `ac63_id_column_order::ids_run_in_column_order_not_declaration_or_lexical_order` (declared_tables.rs, TC-2095) builds a row with columns `[Zeta, Alpha]` holding `T-9`, `T-1` and declares `[Alpha, Zeta]`. It expects `["T-9", "T-1"]`. Declaration order, BTreeMap key order and lexical id order would each give `["T-1", "T-9"]`, so the test fails for all three wrong orders. |
| FND-002 | fixed | `row_binding_never_joins_a_list_target_while_a_single_name_target_joins` now loops over `[ID, Other]` and `'I*'` and asserts no row-binding diagnostic for each. |
| FND-003 | fixed | `TraceabilityModel::check_bound_target` (an undeclared target, a non-table target, or a list or `*` target) is called from `validate` and, in `flatten_into_registry`, on the merged model. On failure it records an `ArchetypeLoadFailure` against every module declaring that obligations source and resets the merged model to default. This mirrors the existing merged-model `validate_retirement` failure directly above it ("must never expose a contradictory coverage model"), and other capabilities stay loaded. `loader::tests::an_obligation_bound_to_another_modules_unreadable_target_fails_load` loads two real modules through `load_modules` (sorted, so mod-a deterministically wins) and covers list, `*` and `list_item`. It asserts the failure on mod-b and an empty model. A single-module load is unchanged because per-module `validate` already refused these. |
