---
id: SR-20261113091246
title: "Code review — PLAT-1500 every table under a declared section for minting, obligations and rollup (quire-rs PR 671)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@0191395; src/corpus/declared_tables.rs, src/corpus/trace_refs.rs, src/coverage/reconcile.rs, src/obligation.rs, tests/it/coverage_rollup.rs, tests/it/located_findings.rs, CHANGELOG.md; implicit callers read: src/obligation/inventory.rs, src/obligation.rs (for_document, combinatorial_obligation)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-049", type: references}
---

## Summary

Ticket: PLAT-1500. PR head `0191395`, scope `git diff origin/main...HEAD`. Code review with the rust-review lane folded in. The PR deletes `TableScan` and makes `tables_in` read every non-fenced table for every `declared_tables::scan` / `tables_of` / `rows_of` caller. `id-column-matches-nothing` and `status-column-matches-nothing` move to a per-section rule: they fire only on the first table of a heading, and only when no table under that heading has the column.

## Verdict

**FAIL: one high and one medium finding.** The changed `tables_of` also feeds the obligation inventory (`src/obligation/inventory.rs:1089`). The inventory still raises `MissingColumn` for each table, so after this PR a legend grid under an obligation section makes the inventory `Incomplete` (FND-001, reproduced).

- **Dead code.** None. `TableScan`, `tables_in_sections` and the `tables:` field are gone, and no reference is left in `src/` or `tests/`.
- **Fenced tables.** They are excluded on every path, because all callers share the one `tables_in` filter. Minting was confirmed by tc2326's fenced control.
- **AC-30 rule.** Probes confirmed all three clauses. With the first table lacking the column and a later table carrying it, nothing is raised. With no table carrying it, exactly one is raised (for both id and status). With a later table lacking it, nothing is raised.
- **FR-058, relation_report and required_relations.** No regression. These paths do not call `declared_tables::scan` / `tables_of` / `rows_of`, and the focused relation_report and required_relations tests pass.
- **Mutation.** With `tables_in` truncated back to the first table, tc2322, tc2323 and tc2326 all fail.
- **Checks run.** 206 focused `it` tests pass (located_findings, coverage_rollup, obligations, relation_report, required_relations, trace_*, id_column_lists, combinatorial, evidence), plus 21 lib tests. `clippy --lib --test it -D warnings` and `cargo fmt --check` are clean.
- **Ceremony.** None found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Regression: the obligation inventory now reads every table through `tables_of`, but still raises `MissingColumn` for each table that lacks the statement or id column. An FR whose `## Acceptance Criteria` holds the AC table plus a legend grid (`\| Symbol \| Meaning \|`) now yields causes `[MissingColumn, MissingColumn]` with terminal `Incomplete`. This was reproduced against the PR head. On main the legend was never read and the inventory was `Complete`. `derive()` on the same document correctly returns only `FR-001-AC-1`, so the two obligation surfaces now disagree. This contradicts the PR's own rule that a legend grid raises none. Fix: apply the section rule in `scan_document` (a column cause only on the section's first table, and only when no table under the section has the column; later tables without the columns are skipped with no cause), and add an inventory test with a trailing legend. | src/obligation/inventory.rs:1089-1156 |
| FND-002 | medium | The section-level findings are grouped by heading text, not by section. `first` and `any_table_in` compare `h == heading`, so two separate sections with the same heading text in one document merge into one group. Probe: `## Test Cases` holds a good matrix, then `## Other`, then a second `## Test Cases` whose table lacks `ID`. This raises no `id-column-matches-nothing`. On main each matched section's first table was checked, per the CR-118 per-matched-section comment kept at :578-582. Fix: carry a section index out of `tables_of`, for example `(section_ordinal, heading, table)`, and group on that. | src/corpus/declared_tables.rs:537-557 |
| FND-003 | low | The AC-28 half of tc2323 cannot fail. With `tables_in` truncated to the first table, the no-`dangling-trace-reference` assert at :4285-4293 still passes, and only the obligation half fails (:4301). The `FR-049-AC-28` tag therefore claims coverage this test does not give. tc2326 covers AC-28 properly. Fix: assert on something minting controls (for example that the matrix row's reference is resolved in the coverage report), or drop the AC-28 tag from tc2323. | tests/it/coverage_rollup.rs:4268-4293 |
| FND-004 | low | `let exact = !lacks_status;` keeps a name that no longer says what it holds. For every non-first table it is `true` whatever that table's headers are. Inline it as `if lacks_status && status_shaped`. The tc2322 comment "and adds no row" also has no matching assertion. | src/corpus/declared_tables.rs:596-599; tests/it/coverage_rollup.rs:4246-4247 |
| FND-005 | low | The CHANGELOG edit left a trailing space and a dangling line, `...are not read. ` followed by `  (FR-049-AC-23..28).`. The PLAT-1491 entry also still cites AC-26 and AC-28 for behaviour this PR reverses. | CHANGELOG.md:66-69 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/corpus/declared_tables.rs scan, tables_of, tables_in | examined | FND-002, FND-004; dead code fully removed |
| src/corpus/trace_refs.rs referencing_rows | examined | Behaviour unchanged (was Every) |
| src/coverage/reconcile.rs scan_reference_rows, retired_obligation_flags | examined | Clean |
| src/obligation.rs derive, for_document, combinatorial_obligation | examined | Clean (widening of combinatorial noted in gap-analysis) |
| src/obligation/inventory.rs scan_document | examined | FND-001 |
| tests/it/coverage_rollup.rs tc2322, tc2323 | examined | FND-003, FND-004 |
| tests/it/located_findings.rs tc2326 | examined | Clean; discriminating, fenced control present |
| CHANGELOG.md | examined | FND-005 |
| src/corpus/relation_report.rs, required_relations.rs | context_only | Do not use the scan; no regression |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | Missed in the review pass and still present at fe3f6bb: legend rows still enter `derive()`. The scan keeps every row of a table that lacks the statement and id columns, so `collect` turns each legend row into a `SkippedRow`. The coverage report then raises one `obligation-row-states-nothing` per legend row ("the source's statement column is empty"). Reproduced: an AC table plus a 2-row `\| Symbol \| Meaning \|` legend gives two such diagnostics, while the inventory is `Complete` with no cause. This contradicts the FR-049 body ("A table lacking the declared id column contributes no rows") and AC-30's legend intent. The same rows also advance the per-document `{row}` ordinal in both `collect` and the inventory. A legend between two `id_format` tables therefore shifts every later id by the legend's row count. Fix: drop the rows of a table that lacks a required column before ordinals and skipped rows are counted, and add a coverage test asserting no `obligation-row-states-nothing` for a legend. | src/obligation.rs:552-579; src/obligation/inventory.rs:1173-1180; src/coverage/reconcile.rs:565-577 |
| FND-007 | low | The new tc2322 assertion `all(\|u\| u.row_id.as_deref() != Some("✅"))` cannot fail. The legend has no `ID` or `Traces To` column, so it never yields a reference row and no unbacked row can carry `row_id == "✅"`. Assert that `unbacked_rows` is unchanged from the single-table baseline instead. | tests/it/coverage_rollup.rs:4262-4270 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (fe3f6bb): `scan_document` applies `section_lacks` per required column. Probe: AC table plus legend gives causes `[]`, terminal `Complete`. Pinned by `every_table_under_an_obligation_section_derives_and_a_legend_raises_no_cause`, which fails when the per-table cause is restored. |
| FND-002 | fixed | Round 1: `tables_of_sections` carries a matched-section ordinal, and `section_lacks` groups on it. The two-sections-same-heading case is pinned in `id_column_finding_is_per_section_on_its_first_table`, which fails when grouping reverts to heading text. |
| FND-003 | fixed | Round 1: tc2323 asserts `minted_targets` contains FR-001-AC-3. With `tables_in` truncated to the first table, it fails at coverage_rollup.rs:4303. |
| FND-004 | fixed | Round 1: `exact` removed, condition is `if lacks_status && status_shaped`. The added "adds no row" assertion is vacuous; see FND-007. |
| FND-005 | fixed | Round 1: trailing space and dangling line removed; the PLAT-1491 entry now cites AC-23..25 and AC-27. |
