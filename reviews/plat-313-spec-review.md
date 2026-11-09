---
id: SR-16340
title: "Spec review — PLAT-313 FR-050-AC-61 id_column as one name or a list (quire-rs PR 609)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-050-declarative-coverage-computation.md, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
---

## Summary

Ticket: PLAT-313. PR head: `3630737`. Spec-only. Reviewed FR-050-AC-61, the CR-207 note, TC-2083..TC-2098, the FR-050 traceability row and the AC-to-TC audit row against the code on `main`: `src/traceability.rs` (`SectionNames`, `SectionMatcher`, `wildcard_match`, `TraceTargetSource::TableRow`, `check_sections`), `src/corpus/declared_tables.rs` (`ScannedRow::cell`, `scan`, `mint`, `IdColumnMatchesNothing`), `src/coverage/reconcile.rs`, `src/coverage/row_binding.rs` and `src/obligation.rs`. The ids CR-207, TC-2083..TC-2098 and FR-050-AC-61 are not on `main` and not on open PRs #606 (CR-205, TC-2081), #607 (CR-206, TC-2082), #608 or #521. `quire validate` on both files gives no new warning; the only warning (line 302, AC-58) predates this PR. `make validate` was not rerun (it needs a cargo build); the author reported 0 failed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-61 says an entry with no `*` "is the header text exactly". Today a column matches ASCII case-insensitively (`ScannedRow::cell`, the `IdColumnMatchesNothing` check), and AC-34, which AC-61 says it reuses, qualifies "exactly" with "matched as `query::section` has always matched it". Read literally, AC-61 makes column matching case-sensitive and breaks "a scalar as before". Fix: say an exact entry matches the header case-insensitively, as `id_column` always has, and that a `*` entry matches case-insensitively too. State whether `SectionNames`' heading normalization (decorative section numbers) applies to column headers. | spec/functional/FR-050-declarative-coverage-computation.md:305 |
| FND-002 | medium | AC-61 specifies minting only. Other consumers read `TraceTarget::id_column()` as one column: an `obligations:` source bound by `target:` (`obligation.rs` `resolve`, `inventory.rs`), retirement reconciliation (`reconcile.rs` ~881-921), and the row-binding join (`row_binding.rs` 43-44, which compares `row_id_column` to the target's column for equality and cannot compare to a `*` entry). The AC does not say how they behave when the target declares a list or a wildcard. Two implementers would differ: one obligation per matched column, first match only, or refuse at load. Fix: state the rule for each consumer, or that a list or wildcard `id_column` is rejected at load on a target that an obligation source or a `row_id_column` reference binds to. Add a TC vector. | spec/functional/FR-050-declarative-coverage-computation.md:305 |
| FND-003 | low | "Every header that matches … in column order within each row" means one row can mint two ids when two matched columns are both filled. The AC does not say what happens when both cells hold the same id: one id, or two, which then trips the CR-087 shared-trace-id defect (TC-950/951). Fix: state that a row contributes each distinct id once, or that the duplicate is reported. Add the vector to TC-2097. | spec/functional/FR-050-declarative-coverage-computation.md:305 |

## Verdict

Not mergeable as is. FND-001 and FND-002 (medium) and FND-003 (low) should be fixed in one round. Each is a sentence plus a TC vector.

The rest holds. AC-61 reuses AC-34's `section:` grammar faithfully: scalar or sequence on one key, `*` as the only metacharacter, empty sequence or blank entry fails load, and a scalar round-trips. `SectionNames` already provides this and can be reused. The narrowed `id-column-matches-nothing` (fires only when no entry matches, names every entry) fits the per-matched-table check in `scan`. The two choices the PR makes are reasonable and testable: a header matching two entries is read once (TC-2097), and ids come in column order. Column order needs the table's header order, which `scan` holds; `ScannedRow.cells` is a `BTreeMap` and loses it. That is an implementation note, not a spec defect. AC-61 can fail: each clause has an observable outcome, and the one-column control (TC-2097) is the regression a widening would break. The audit row and the traceability row (`AC-1..61`, the five TCs) agree with the TC table. The CR-207 note adds no census, pin, SHA or count, and states as much under its non-goals. The bare-`ID` caution is guidance with no validation, which is appropriate.

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-050-AC-61 | examined | FND-001, FND-002, FND-003 |
| FR-050 CR-207 note | examined | Clean; no ceremony |
| TC-2083..TC-2098 | examined | Clean; each traces to AC-61 |
| spec/tests.md FR-050 traceability row | examined | Consistent |
| spec/tests.md FR-050-AC-61 audit row | examined | Consistent |
| FR-050-AC-33 | context_only | Diagnostic narrowed consistently |
| FR-050-AC-34 | context_only | Grammar reused; matching qualifier dropped (FND-001) |
| FR-050-AC-60 | context_only | table_row-only scope consistent |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | FR-050-AC-61 is also defined by open PR #610 (PLAT-1170, legacy `Implements:` warning, head `9581b22`), with its own audit row `FR-050-AC-61 → TC-2083`. Once both land, one id names two criteria and the audit-table check (AC-58) sees two definitions. The fix round renumbered CR-207 to CR-210 and TC-2083..2087 to TC-2095..2089 to yield to #610, but kept AC-61. Fix: renumber this PR's criterion to FR-050-AC-62 (statement row, test-case Requirement cells, audit row, `AC-1..62` coverage range, and the CR-210 note's 'AC-61'). | spec/functional/FR-050-declarative-coverage-computation.md:305 |
| FND-005 | low | TC-2095 still says "an entry with none matches the header exactly". AC-61 now says exact entries match ignoring ASCII case. Fix: say "matches the header ignoring ASCII case" in TC-2095. | spec/tests.md:1131 |
| FND-006 | low | `reviews/plat-313-spec-review.md`, `-ears.md` and `-criterion-strength.md` carry SR-16340, SR-16341 and SR-16342. Open PR #607 adds `reviews/plat-877-*.md` with the same three ids. Whichever PR lands second has duplicate SpecReview ids. Fix: if #607 lands first, renumber these files to the next free ids; the Linear review comments are keyed by path and need no change. | reviews/plat-313-spec-review.md:2 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-61 now says both exact and `*` entries match ignoring ASCII case. It says heading normalization applies to section headings only, and a column header is compared as written, trimmed. Both statements match `tables_of` (`h.trim()`) and `ScannedRow::cell` (`eq_ignore_ascii_case`). TC-2098 adds the vector. |
| FND-002 | fixed | A list or `*` `id_column` now fails load when an `obligations:` source binds the target by `target:`. That check fits the existing `table_targets` gate in `TraceabilityModel::validate` (traceability.rs ~1341). It covers `obligation::resolve`, `inventory.rs` (via `super::resolve`) and `reconcile.rs` (resolves via `source.target`). Row-binding joins only a single-name, `*`-free target, which matches the `row_binding.rs:43-44` equality join. TC-2100 can fail in four ways: the load is accepted, a wildcard target joins, the single-name control fails to join, or an unbound sequence is rejected. |
| FND-003 | fixed | AC-61 now says a row whose matched columns hold the same id contributes it once and distinct ids each mint. The vector is in TC-2098. |
