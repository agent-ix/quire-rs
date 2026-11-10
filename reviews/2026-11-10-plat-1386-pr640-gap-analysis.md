---
id: SR-20261103118154
title: "Gap analysis — PLAT-1386 strict exemption of planned, blocked and parked rows (quire-rs PR 640)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-050-declarative-coverage-computation.md (FR-050-AC-64..66), spec/tests.md (TC-2219..2226), src/coverage/reconcile.rs, src/coverage.rs, src/evidence.rs, schemas/output/coverage-v1.schema.json, tests/it/coverage_rollup.rs, tests/it/output_contract.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
---

## Summary

Ticket: PLAT-1386. PR head `cb26344c5648d6f66cca8199739bec28ed7e39d2`. A planless gap analysis scoped to the engine half of the change: FR-050-AC-64..66 against the engine tests TC-2219..2224. The CLI cases TC-2225 and TC-2226 belong to quire-cli and remain 🚧. Plan completion: not assessed.

Trace bindings are correct. TC-2219, TC-2220 and TC-2224 are bound to AC-64, TC-2221 to AC-65, and TC-2222 and TC-2223 to AC-66. The spec/tests.md flips (✅ for TC-2219..2224, 🚧 kept on TC-2225/2226, and the summary row "✅ Implemented (engine)") match the tests that exist.

Covered by tests:

- AC-64 positive and negative word vectors, the `status_column` override against the global column, and pending-classed against undeclared values (TC-2219, TC-2220).
- AC-64 declared status: all-planned referencers, mixed referencers, no referencer, and an unexpanded range (TC-2224).
- AC-65: the keys appear on exactly the exempt rows, an exempt payload is validated against the published schema with `additionalProperties: false` actually exercised, identity, locus and target are unchanged and the rows stay in the census, the keys are absent when nothing is exempt, and repeated runs are identical (TC-2221). The TC-824 baseline is unchanged.

## Verdict

**FAIL (one high).** AC-66's single engine behaviour change, the `strict_decision` exclusion, has no test that can fail (FND-001). Further AC-64 vectors are untested (FND-002), and the range assertion in TC-2224 contradicts the TC-2224 text (FND-003).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | No test exercises AC-66's verdict exclusion. TC-2222 asserts `causes.contains(UnbackedRows) == (evidence.non_exempt_unbacked() > 0)`, but its "all-exempt" fixture is not all-exempt: AC-2 is unreferenced and so non-exempt, a fact the test's own comment notes. Both sides of the check are therefore `true`, and reverting src/evidence.rs:685 to `!report.unbacked_rows.is_empty()` keeps every test green. The `mixed` count assertion is a restatement of `non_exempt_unbacked()`'s own body. No engine test shows `strict_decision` returning without `UnbackedRows` when only exempt rows are unbacked. No engine test shows that ZeroTargets, ContradictedStatus (a Complete row without evidence) or an incomplete inventory still fail alongside exempt-only rows. Add a truly all-exempt fixture (every criterion referenced by an exempt row) and assert `strict_decision(..)` is `Ok` on both the source and evidence paths. Add one fixture per remaining premise and assert the matching cause is still present. | tests/it/coverage_rollup.rs:3934-4001, src/evidence.rs:685 |
| FND-002 | medium | Further AC-64 declared-status vectors are untested. No test covers a blank-cell referencing row, a retired referencing row (counted as a non-exempt referencer because accumulation runs before the retirement `continue`), or a backed referencing row (also counted). Each changes whether a criterion is exempt. The blank-cell case is currently wrong (code review SR-20261103118153 FND-001). Add a TC-2224 vector for each, with the expected outcome stated in the spec. | tests/it/coverage_rollup.rs:3817-3853, src/coverage/reconcile.rs:253-262 |
| FND-003 | medium | TC-2224 says a range that is not expanded "contributes nothing, so a row it would have referenced is not exempt". The test asserts `FR-001-AC-1` (the range's own left endpoint, which the reference pattern captures into `target_ids`) **is** exempt. The behaviour follows the spec's governing definition ("Reference means only the record's own computed `target_ids` field"), but the TC text and the FR sentence "an unexpanded … range contributes nothing" say otherwise for the endpoint. Reconcile them: either state in FR-050 and TC-2224 that a captured endpoint token is an ordinary reference, or make the engine drop it. | tests/it/coverage_rollup.rs:3842-3852, spec/tests.md (TC-2224 row) |
| FND-004 | low | TC-2223 runs `compute` twice with identical input. Because `compute` takes no severity input it can only show determinism, not "the same exempt count under default severities and under `--severity coverage:unbacked-row=off`". The structural argument is sound, but the TC title promises a comparison the engine test cannot make. Note that the severity half lives with the quire-cli case (TC-2226), or reword TC-2223 to the structural claim. | tests/it/coverage_rollup.rs:4004-4016 |
| FND-005 | low | TC-856's "payload carrying every optional key" was not extended with `unbacked_rows[].strict_exempt` and `totals.strict_exempt_unbacked`. Schema acceptance of both is exercised by TC-2221, so nothing is unverified, but the FR-055-AC-3 test no longer covers what its name claims. | tests/it/output_contract.rs:184-330 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-050-AC-64: "Rows `Planned`, `BLOCKED`, `🚧 Parked`, `Blocked: awaiting PLAT-9` and `plannedé` are exempt; `unplanned`, `plannedly`, `In Progress`, `Pending`, a blank status and a row with no status column and no referencing row are not. … A reference row with no status cell takes its exemption from the rows that reference it" | examined | Word vectors and column selection covered; declared-status edge vectors missing (FND-002, FND-003) |
| FR-050-AC-65: "an exempt payload validates against it. Exempt rows stay in `unbacked_rows` … A report with no exempt rows serializes byte-identically …" | examined | Covered by TC-2221 and TC-824; CLI line is TC-2225 (quire-cli) |
| FR-050-AC-66: "The strict verdict and an `error`-promoted `coverage:unbacked-row` count only non-exempt unbacked rows … A live Complete row without evidence, an incomplete explicit evidence inventory, and the same scope with no declared targets … each still fail" | examined | Verdict exclusion untested (FND-001); severity half structural only (FND-004) |
| TC-2219..2224 trace bindings and spec/tests.md status flips | examined | Correct |
| Production code with no owning requirement | examined | None; every new line traces to AC-64..66 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head `250577b`. New `tc2222_strict_decision_passes_exempt_only_rows_and_keeps_every_other_premise` uses a truly all-exempt fixture (4 unbacked rows, `non_exempt_unbacked() == 0`). It asserts `strict_decision` is `Ok` on the source and evidence paths, which fails if src/evidence.rs:685 reverts to `!unbacked_rows.is_empty()`. With exempt-only rows it asserts that ZeroTargets, ContradictedStatus and an incomplete inventory (`CauseCode::Absent`) each still fail, and that a mixed fixture yields UnbackedRows. ZeroTargets and the status lie are injected by mutating the report rather than by fixture, which is adequate for the engine's `strict_decision` unit. |
| FND-002 | fixed | Round 1, head `250577b`. `tc2224_blank_backed_and_retired_referencing_rows_block_the_exemption` covers a blank referencing row (veto), a blank row referenced only by an exempt row (not exempt), a backed `✅` referencing row (veto) and a `⛔` retired referencing row (veto). FR-050 now states "Referencing rows count whether they are backed, retired or unbacked." |
| FND-003 | fixed | Round 1, head `250577b`. Ruling: the engine's computed `target_ids` is truth. FR-050 now reads "an unexpanded range contributes only the ids the declaration's pattern itself captures from it (for example the range's endpoint), nothing is expanded". TC-2224 is reworded to match, and the existing assertions (AC-1 exempt; AC-2 and AC-3 not) now agree with both. |
| FND-004 | fixed | Round 1, head `250577b`. TC-2223 is reworded to the structural claim ("`compute` takes no severity input … the severity override itself is exercised by the CLI case TC-2226"), which is what the test shows. |
| FND-005 | fixed | Round 1, head `250577b`. The TC-856 payload now carries `"strict_exempt": true` on its unbacked row and `"strict_exempt_unbacked": 1` in `totals`, and it validates. |
