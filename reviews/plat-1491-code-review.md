---
id: SR-20261112205305
title: "Code review — PLAT-1491 every table under a declared section, PLAT-1484 record_validation null (quire-rs PR 660)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@a1203093; src/corpus/declared_tables.rs, src/corpus/trace_refs.rs, src/coverage/reconcile.rs, src/obligation.rs, src/loader/manifest.rs, src/loader/record_validation.rs, tests/it/located_findings.rs, tests/it/coverage_rollup.rs; consumers read: src/coverage/row_binding.rs, src/obligation/inventory.rs, src/query.rs, src/traceability.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-049", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-050", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-069", type: references}
---

## Summary

Tickets: PLAT-1491, PLAT-1484. PR head `a120309` over merge base `51f81dd` (= origin/main). Code review with the rust-review lane folded in. The PR adds `DeclaredScope.every_table`, set `true` only for the two document-reference consumers (`trace_refs::referencing_rows`, `reconcile::scan_reference_rows`), and keeps a raw `record_validation: null` as `Some(Null)` so the loader refuses it before the no-semantic-block refusal.

## Verdict

**FAIL (medium findings; no wrong result on the existing corpus).**

- **Call sites.** Every `declared_tables::scan` caller was checked. `every_table: true`: `trace_refs::referencing_rows` (FR-049) and `reconcile::scan_reference_rows` (FR-050, which also feeds `row_binding::from_scanned_rows`). `every_table: false`: `mint` (trace targets), `obligation::derive`, `reconcile::retired_obligation_flags` (which must and does match `derive`), and the three unit tests. `tables_of`/`rows_of`/`obligation::inventory::scan_document` keep first-table behaviour through `tables_in_sections(.., false)`. No call site was switched by accident.
- **Byte identity.** A scratch harness ran `validate_bundle_at` (Okf) and `coverage::compute` with the spec-artifacts-process + spec-artifacts-iso modules over all 291 `~/dev/*/spec` repositories, once with the PR head and once with `every_table` forced `false` in the two reference consumers (the pre-PR behaviour). All 582 runs exited 0, and every pair of outputs (findings with lines, plus the full coverage JSON) was byte-identical. The pairs carried 7561 `dangling-trace-reference` findings in total. The existing `located_findings::`, `coverage_rollup::`, `corpus_cases::`, `coverage_baseline::`, `required_relations_where::`, `semantic_*`, `corpus_recall::` and dogfood suites pass (276 + 13 tests), as do 68 lib tests.
- **Mutation.** With `every_table` false in both consumers, TC-2319, TC-2320, TC-2321 and TC-2322 all fail. With it false in `reconcile` only, TC-2322 alone fails. TC-2318 fails when the `is_null` refusal is disabled and again when `deserialize_with` is removed.
- **Edges probed.** A second table holding only the kind column adds no rows and no finding, which is correct. Tables with different headers are matched per table, which is correct. A huge document is no regression: `parse_table_with_lines` already parsed every table and took the first. The rest are FND-001..003.
- **Rust.** No new `unwrap` or panic. `flat_map` with a per-table `heading.clone()` is fine. FND-004 and FND-005 are style.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A table inside a fenced code block under a declared section is now read as data. `parse_tables_with_lines` has no fence awareness, so an illustrative ```` ```markdown ```` table placed after the real table now yields findings. Reproduced on the PR head: `## Links` with one real table and then a fenced example row `K-9 / north / L-999` gives `dangling-trace-reference 'L-999'` at line 18, the fenced line. Before this PR that example was reached only when it came first. No current `~/dev` document hits it (sweep above). Fix: skip fenced regions when collecting tables, and add a case with a fenced example after the real table that yields no finding. | src/query.rs:306-333; src/corpus/declared_tables.rs:910-927 |
| FND-002 | medium | A table without the reference column still raises a status-column finding in the FR-050 rollup. FR-049 says a table lacking the `column` header "contributes no rows and no finding". But `scan` runs the `status_column` near-miss check on every table, so a legend grid under the matrix section produces a diagnostic. Reproduced: the iso fixture plus a `\| Status Symbol \| Meaning \|` legend under `## Test Cases` adds `status-column-matches-nothing` (traces-to, tests.md:17), and the report is no longer identical. Fix: when `every_table` is set, apply the per-table header checks only to tables that carry the declaration's `column`, and add the legend case as a no-finding control. | src/corpus/declared_tables.rs:576-597; spec/functional/FR-049-verification-reference-integrity.md:134 |
| FND-003 | medium | Reference rows now come from every table, but targets and obligations still mint from the first table only, so one section is read two ways. Reproduced: an FR with a second `Acceptance Criteria` table (`FR-001-AC-3 \| … \| Test (TC-003)`) adds an `unbacked_rows` entry for `FR-001-AC-3`. That id is not a minted `acceptance-criterion`, so the group total stays 2, and it is not an obligation. In the same way, the Test IDs of a second `Test Case Summary` table enter `row_binding` (`matrix_row_ids`, `rows_by_id`) but are not minted `test-case` targets, so an FR `Verification` cell citing one still dangles. FR-049 records the first-table rule for targets but not these consequences. No current `~/dev` repo is affected. Fix: either read every table for a target or obligation source that shares the declaration's section, or state the asymmetry in FR-049/FR-050 with a test that pins it. | src/coverage/reconcile.rs:152-290; src/coverage/row_binding.rs:97-127; src/corpus/declared_tables.rs:650-660 |
| FND-004 | low | `every_table: bool` is a boolean mode flag on `DeclaredScope`. The repo's CLAUDE.md asks for enums over boolean flags, so a two-variant enum (`First` / `Every`) would name the mode at every call site. | src/corpus/declared_tables.rs:107-111 |
| FND-005 | low | `deserialize_present` is now duplicated in `loader/manifest.rs` and `traceability.rs`, with identical bodies. One shared helper would keep the null-is-present rule in one place. | src/loader/manifest.rs:497-502; src/traceability.rs:564-569 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | fenced_lines does not track the fence length or the info-string rule. Reproduced on 356294a: inside a 4-backtick md fence, a lone 3-backtick rust line closes the fence early, so the example table after it is read (dangling L-996 at line 17). A prose line starting with an inline triple-backtick span opens a fence that never closes, so the real table after it is silently dropped (L-997 is not reported). Fix: record the opening run's char and length, close only on a bare run of the same char at least that long, and do not treat a backtick run whose info string contains a backtick as a fence. Add both cases to TC-2325. | src/corpus/declared_tables.rs:942-966 |
| FND-007 | low | null_key_failure was inserted between retired_field_reason's doc comment and its fn. retired_field_reason's doc now opens null_key_failure's doc, and retired_field_reason has no doc. Move the two retired-field lines back above retired_field_reason. | src/loader/mod.rs:134-160 |

## Dispositions

Round 1, PR head `356294a`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
