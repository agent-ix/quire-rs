---
id: SR-20269
title: "spec-review of the FR-050 strict exemption for planned, blocked and parked rows (quire-rs PR 639)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@fdff4d7d; spec/functional/FR-050-declarative-coverage-computation.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-050
    type: references
---

# Spec review of the FR-050 strict exemption for planned, blocked and parked rows (quire-rs PR 639)

## Summary

Ticket: PLAT-1386. One spec-review pass over `git diff origin/main...fdff4d7d`. The branch is one commit on top of main `f618683`. The lead's decision is taken as given: three fixed words, matched in the engine, with no manifest change. This review does not reopen it.

Examined: the new section (FR-050 lines 139-176), FR-050-AC-64, AC-65 and AC-66, and their consistency with FR-050-AC-14, AC-21, AC-45, AC-53 and AC-54, the retired-row section (lines 128-137), the FR-078 explicit-evidence strict paragraph (lines 190-201), and FR-055 and `schemas/output/coverage-v1.schema.json`. The strict gate as implemented in quire-cli `src/commands/coverage.rs` at 661f2e5 was read for context only.

The ticket's own reproduction was measured. `quire coverage --json` (quire 0.39.0, engine 0.55.0) was run over a `git archive` of tl-syntax `origin/main`. It reports 226 unbacked rows. 60 of them belong to the parked set: FR-015..019, NFR-004/005 and TC-059..073. 20 of those 60 are matrix rows (`traces-to` and `functional-coverage`) whose Status cell reads `🚧 planned`. The other 40 are `verification` and `nfr-verification` rows in the FR and NFR acceptance-criteria tables, whose header is `| ID | Criteria | Verification |` and has no Status column. Under FR-050-AC-64 those 40 rows are not exempt (FND-001).

The matching rule itself holds up when worked by hand. `🚧 planned`, `Planned`, `BLOCKED`, `🚧 Parked` and `Blocked: awaiting PLAT-9` all match. `unplanned`, `plannedly`, `Pending` and a blank cell do not. Retired classification runs first, so it is unaffected. AC-21's undeclared-status records are unaffected, because AC-21 does not gate strict. Nothing contradicts AC-45's per-reference `status_column`.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The amendment does not deliver the ticket's outcome. On tl-syntax `origin/main`, 40 of the 60 parked unbacked rows are FR/NFR acceptance-criterion `verification` rows. Their tables (`ID / Criteria / Verification`) have no Status column, and the FR documents carry no frontmatter status. AC-64 makes "a row with no status column" non-exempt, so `--strict` still exits 1 on the parked set after this lands. The same holds for quire-rs's own AC-64..66 rows, which this PR adds as unbacked `verification` rows. Fix, without reopening the three-word rule: state where a status-less criterion row gets its status. For example, a row is exempt when every matrix row that declares it, through `functional-coverage` or `traces-to`, is exempt. Alternatively, state that authors must add a Status column, and confirm that the process FR archetype accepts one. Add the case as an AC-64 vector. | spec/functional/FR-050-declarative-coverage-computation.md:139-176; spec/functional/FR-050-declarative-coverage-computation.md:348 | missing-requirement |
| FND-002 | medium | The interaction with an `error`-promoted `coverage:unbacked-row` (FR-048, quire-cli FR-017-AC-13) is unspecified. quire-cli fails the run on any promoted check before the strict block, counting every unbacked row, exempt ones included. Line 171 says strict SHALL exit 0 when the only unbacked rows are exempt and no other *strict premise* fails. Promotion is not a strict premise, so an all-exempt fixture with module `grammar_severity` `coverage:unbacked-row: error` has two defensible exit codes. Fix: state whether exempt rows count toward an `error`-promoted unbacked-row check, and add that vector to AC-66. | spec/functional/FR-050-declarative-coverage-computation.md:169-176; spec/functional/FR-050-declarative-coverage-computation.md:350 | missing-requirement |
| FND-003 | medium | The amendment adds `totals.strict_exempt_unbacked` and per-record `strict_exempt`. However, `coverage-v1.schema.json` declares both `CoverageTotals` and `UnbackedRow` with `additionalProperties: false`, and neither the section nor AC-65 says that the published schema gains the two optional keys. An implementation that follows the spec alone emits payloads that FR-055-AC-5 rejects. Fix: state the two additive keys in the schema (an integer `minimum: 1` omitted when zero, matching `retired`; a boolean `const: true`), and have TC-2221 validate an exempt payload against coverage-v1 (FR-055-AC-3). | spec/functional/FR-050-declarative-coverage-computation.md:158-167; spec/functional/FR-050-declarative-coverage-computation.md:349 | missing-requirement |
| FND-004 | medium | The section does not say whether it applies on the FR-078 explicit-evidence path. Lines 196-197 say that on that path 'unbacked rows ... retain their existing strict rejection behavior'. That paragraph predates the exemption and now reads as excluding it. FR-078's per-act reasons (`absent`, `unevaluated`) also reject a criterion that has no record, so an exempt planned criterion may still fail. Fix: state whether the exemption applies in evidence mode, and amend lines 196-197 to agree. | spec/functional/FR-050-declarative-coverage-computation.md:190-201 | wrong-requirement |
| FND-005 | low | FR-050-AC-53 still says that the mixed scope (Withdrawn, live Complete unbacked, Planned unbacked) 'fails `--strict` because of the live rows'. Under AC-64 the Planned row is exempt, so only the Complete row fails it. The verdict is unchanged but the stated cause is now wrong. Fix: 'because of the live Complete row'. | spec/functional/FR-050-declarative-coverage-computation.md:338 | wrong-requirement |
| FND-006 | low | 'Non-alphanumeric' is not pinned to ASCII or Unicode. The two readings disagree on a non-ASCII letter after the word (`plannedé` matches under ASCII and not under Unicode `char::is_alphanumeric`). They also disagree on a non-ASCII letter in leading position. Fix: name the class, for example Unicode alphanumeric. | spec/functional/FR-050-declarative-coverage-computation.md:148-153 | wrong-requirement |

## Verdict

Not mergeable. FND-001 (high) blocks: the strict exemption is consistent and testable as written, but over the ticket's own reproduction it leaves 40 of the 60 parked rows failing. FND-002, FND-003 and FND-004 (medium) should be fixed in the same round. FND-005 and FND-006 (low) are wording fixes.

What holds: the word-boundary rule, the leading-marker strip, the blank-cell and no-column cases, and the `status_column` versus global column precedence are each unambiguous and testable. The byte-identity clause ('omitted when zero') follows the `retired` and AC-21 precedent. The exemption never touches status lies, the empty-population premise, retired rows or the census. `quire validate` on both files exits 0. Its only output is the `ac:non-canonical-shape` warning on FR-050-AC-58, which appears at line 304 on main and at line 343 on this branch, so it pre-exists.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-007 | medium | The Declared status rule names module data in the engine. It names the reference declarations `functional-coverage` and `traces-to`, and it uses "matrix row" and "criterion or verification row". FR-050 line 23 says the engine SHALL NOT hardcode FR/AC/TC semantics, and those names are spec-artifacts-process module declarations. "Declares" is also undefined. It could mean the row's `target_ids` after the declaration's normalizations, or the FR-050-AC-58 range reading (`AC-1..6`, `AC-1 through AC-6`). The two readings differ when a coverage row's range is the only declarer. Fix: state it generically. For example: a reference row that has no status cell of its own is exempt when it is unbacked and every reference row that has its own status cell, and whose `target_ids` contain this row's own id, has an exempt own status. Drop the declaration names, and name the `target_ids` reading. | spec/functional/FR-050-declarative-coverage-computation.md:159-164; spec/functional/FR-050-declarative-coverage-computation.md:374 | wrong-requirement |
| FND-008 | medium | The `plannedé` vector contradicts the rule it tests. The rule says a match is `planned` followed by the end of the text or by "a character that is not an ASCII letter or digit". `é` is not an ASCII letter, so `plannedé` matches. The section, AC-64 and TC-2219 all list it as not exempt, so no implementation can satisfy both the rule and the vector. Fix: either move `plannedé` to the exempt vectors, or make the trailing boundary "not a Unicode alphanumeric", keeping the leading strip ASCII if wanted. | spec/functional/FR-050-declarative-coverage-computation.md:148-153; spec/functional/FR-050-declarative-coverage-computation.md:374; spec/tests.md:1321 | wrong-requirement |

## Dispositions

Round 1 at 74a4dde9. The parked rows were measured again over tl-syntax `origin/main` with the Declared status rule simulated: every `traces-to` matrix row that names each of the 40 status-less `verification`/`nfr-verification` rows of FR-015..019 and NFR-004/005 has `🚧 planned`, so all 40 are exempt and the remaining 20 rows are exempt by Own status. `quire validate` on the four changed spec files exits 0; the only warning is the pre-existing FR-050-AC-58 `ac:non-canonical-shape` (now line 369).

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Declared status paragraph added; AC-64 and TC-2224 carry the all-planned, mixed and undeclared vectors; tl-syntax 40/40 status-less parked rows exempt by simulation. |
| FND-002 | fixed | Verdict paragraph: exempt rows are excluded from the error-promoted `coverage:unbacked-row` count; AC-66 and TC-2222/TC-2226 carry the promoted vector. |
| FND-003 | fixed | Section names both additive keys of `coverage-v1.schema.json` (`minimum: 1`, `const: true`); AC-65 and TC-2221 require an exempt payload to validate. |
| FND-004 | fixed | Evidence-mode paragraph now says "non-exempt unbacked rows" and that the exemption applies identically there while per-act reasons still reject; AC-66 and TC-2222 carry it. |
| FND-005 | fixed | AC-53 now reads "fails `--strict` because of the live Complete row; the Planned row is strict-exempt". |
| FND-006 | fixed | Boundary pinned to ASCII letters and digits. The pinned rule's own `plannedé` vector is wrong (FND-008). |

Round 2 at e7d96549. `quire validate` on FR-050, tests.md and both committed review files exits 0. The only warning is the pre-existing FR-050-AC-58 `ac:non-canonical-shape`, now at line 371. The committed `spec/reviews/` copies matched the reviewer's round-1 files byte for byte.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-007 | fixed | The declaration names `functional-coverage`/`traces-to` and the terms "matrix row" and "criterion or verification row" are gone from the section; the rule is stated over reference rows, status cells and `target_ids`. The new range clause in that wording is a separate defect (FND-009). |
| FND-008 | fixed | `plannedé` moved to the exempt vectors in the section, AC-64 and TC-2219, consistent with the ASCII boundary rule. |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-009 | medium | The Declared status rule gives `target_ids` a second meaning. In the report, `target_ids` is the record's computed field: the ids the row is answerable for after its declaration's normalizations, where range expansion happens only if the declaration opts into `expand_ranges` (FR-050-AC-12). The rule instead says a row's `target_ids` "contain" every id inside a range "as FR-050-AC-58 reads them". AC-58 is the matrix audit check run by `examples/spec_validate.rs`. It resolves short ranges (`AC-1..28`) against the row's requirement id, which is the id-prefix semantics that the next sentence ("The engine hardcodes no ... id-prefix semantics") and FR-050 line 23 rule out. AC-58 also makes an unreadable cell an error, and the engine rule does not say what an unreadable range (for example tl-syntax's `FR-015-AC-1 through FR-015-AC-6`) does to exemption. Two implementers would build different joins. Fix: define "references" as the record's own computed `target_ids`, with no second range reading; a module that wants ranges declares `expand_ranges`. Drop the AC-58 sentence and the "an id inside a range counts as referenced" vector from TC-2224, or tie that vector to `expand_ranges`. On tl-syntax `origin/main`, all 40 status-less parked rows are already exempt through explicit `traces-to` ids, so the simpler rule still meets the ticket. | spec/functional/FR-050-declarative-coverage-computation.md:161-166; spec/tests.md:1323 | wrong-requirement |
| FND-010 | low | AC-64 still says "a row with no status column and no declaring matrix row", after the section dropped the "matrix row" term. Its vector list also reads "`Blocked: awaiting PLAT-9` and `plannedé`" after "and" (a double "and"). Fix: "no referencing row with its own status cell", and a single list conjunction. | spec/functional/FR-050-declarative-coverage-computation.md:376 | wrong-requirement |
