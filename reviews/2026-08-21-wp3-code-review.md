---
id: SR-051
title: "WP3 pre-release code review — the 11 unreviewed QA-phase commits on main (954b315..fa6f0d7, CR-086..CR-092)"
type: SpecReview
analysis: code-review
scope: "examples/spec_validate.rs, Makefile, src/coverage.rs, src/corpus/declared_tables.rs, src/corpus/vocabulary_coverage.rs, src/query.rs, src/symbols/mod.rs, src/symbols/trace.rs, src/symbols/typescript.rs, src/traceability.rs, src/obligation.rs, src/vocab.rs, schemas/output/coverage-v1.schema.json, scripts/slash_tag_sweep.py, scripts/corpus.py, scripts/__init__.py, scripts/tests/, tests/coverage_rollup.rs, tests/output_contract.rs, tests/verification_catalog.rs, tests/vocabulary_coverage.rs, tests/fixtures/, spec/tests.md, spec/log.md, spec/functional/FR-026, spec/functional/FR-050, spec/functional/FR-051, spec/functional/FR-054, spec/functional/FR-057, spec/functional/FR-059, reports/2026-08-20-slash-trace-sweep.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/log", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references }
---

## Summary

Pre-release review gate for **v0.42.0**: the eleven commits this session landed on
`main` without review (`git log 954b315..HEAD`), batches A–D of the QA phase —
3ccdab3 (#218), 39347d6 (#212), 38c8451 (#213 CR-086), f154fc8 (#216 CR-087),
484a5fe (#215 CR-088), 231aaf4 (#210 CR-089), 113269a (#214 CR-090), 5608474
(#217), fa98e0e (#219), 87a1869 (#179 CR-091), fa6f0d7 (#190 CR-092) — plus a
light (non-code) check of e6e82b9, the SR-009/SR-010 retro artifacts themselves.
Reviewed under the `rust-review` discipline against the repo's own idiom docs
(CLAUDE.md, clippy.toml/deny.toml/rustfmt.toml, the CR-083 additive-key pattern).

Every commit was read in full diff. Gates were run, not assumed: `make ci` green
on HEAD before review edits (fmt-check, clippy `-D warnings`, check-python,
check-scripts, test, deny, audit-unsafe, audit-property, audit-static, validate)
and re-run green after the one inline fix below.

**Mutation spot-verification (4/4 killed).** Four red-verifiable claims in the
new `#[trace]` tests were re-verified by mutating the source and running the
claiming test:

| Mutation | Killed by |
| --- | --- |
| `undeclared_statuses.dedup_by` closure forced to `false` (dedup removed, #213) | `tc946_duplicate_undeclared_status_rows_yield_one_record` |
| `status_row_ids.contains` scoping removed from the shared-id walk (#216) | `tc951_uniquely_bound_ids_report_nothing_and_omit_the_key` |
| `ExcludeSet::compile` reverted to silent `if let Ok` partial filtering (#215) | `tc954_an_invalid_glob_is_loud_and_never_partially_filters` |
| `rows_of` line arithmetic `+ 2` → `+ 1` (#210) | `tc955_row_shaped_records_carry_the_matrix_row_line` |

All mutations reverted; tree byte-identical to HEAD afterwards (`git status` clean).

**Schema additivity holds.** Every new contract key is skip-when-empty
(`shared_trace_ids`, `vocabulary_coverage`: `skip_serializing_if = Vec::is_empty`;
`excluded_source_files`: skip-zero; the five `line` keys and
`CoverageDiagnostic.value`: skip-`None`), each with a test pinning both
directions (tc951, tc965, tc953, tc957, tc967/tc969), tc856 exercises every new
key populated, tc859's optional/required split carries all of them, and the
CR-057 byte-golden (tc824/tc855) passes at HEAD — no new field can leak into a
conformant corpus's payload.

**Dedup-vs-line interaction (#213 × #210) is correct and pinned.** CR-089
switched CR-086's `.dedup()` to `dedup_by` comparing without `line`; the stable
sort keeps the first (lowest-line) record. tc946's duplicate rows now sit on
different lines, so a dedup that consulted `line` would fail the test — the
interaction is regression-pinned, not just commented.

**Seam compliance.** The `ExcludeSet::compile` `Result` seam refuses whole
lists; all in-crate callers route through `compile_validated` (post-validation,
debug-asserted, filters *nothing* — never partially — in release); the one `pub`
entry over unvalidated `&[String]` (`extract_tree_scoped`) refuses loudly with a
`SymbolDiagnostic` naming the pattern (tc954). No `#[cfg(test)]` branches in
production paths, no new `#[allow]`, no gate weakening, no new `unsafe`, no bare
`as` casts or `unwrap()` added on library paths. The `line`/count fields are
`usize` serialized as JSON integers with `minimum: 1`/skip-zero — no truncation
at the wire.

## Verdict

**PASS with findings** — one medium finding fixed in this review, two filed,
the rest notes. The engineering substance of the batch is high: every SR-009
defect it claims to close is genuinely closed and red-verified, and the spec
bookkeeping (CR notes, TC rows, audit entries, rollup rows) is complete for all
seven CRs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | **Fixed here.** The #212 gate went green under the corruption class it was built for, one tier up: breaking FR-050's opening frontmatter fence (`---` → `--`) dropped the document from `load_repo`'s corpus entirely, and `make validate` reported `126 document(s), 0 failed`, exit 0 — the population shrank with no trace. Fixed in this review: `spec_validate` now reconciles every on-disk `spec/**/*.md` against the loaded corpus (files under `spec/assets/` exempt as the declared frontmatter-less home) and fails on any unloaded document; untyped in-corpus documents are named in the summary. Re-verified: corrupt → exit 1 naming the file; clean → exit 0, 127 documents | ix://agent-ix/quire-rs/spec/functional/FR-050 |
| FND-002 | medium | **Filed as #223.** `make validate` still cannot catch audit-table drift — the FR-051-AC-18 shape where a coverage row declares `AC-1..N` and the audit table lacks a well-formed row for one of them. Per-document validation sees only cells that exist; the class lives on the bundle-posture surface (208 pre-existing debt errors at 3ccdab3, correctly judged ungateable wholesale by #212). Needs its own measured, narrow gate | ix://agent-ix/quire-rs/spec/tests |
| FND-003 | low | **Filed as #224.** #179 (CR-091) added two new hand-rolled `std::env::temp_dir()` test trees (`tc966` via the file-local `tmpdir` helper, `tc967` in `tests/verification_catalog.rs`) two commits after #215 (CR-088) moved TC-949 onto `tempfile::TempDir` for exactly the leak-on-panic reason and added the dev-dependency. Consistent with each file's pre-existing local pattern, inconsistent with the repo's stated direction; sweep filed | ix://agent-ix/quire-rs/spec/log |
| FND-004 | low | Style note, no failure scenario: `extract_tree_scoped`'s refusal diagnostic derives the offending pattern by re-probing `Glob::new` per pattern; when every pattern compiles individually and only `GlobSetBuilder::build` fails, `path` is the empty string. No known globset input reaches that arm; no scenario, so a note | ix://agent-ix/quire-rs/spec/functional/FR-050 |
| FND-005 | low | Note, accepted seam: `compile_validated` in release filters **nothing**, silently, for a hand-built model that skipped `TraceabilityModel::validate` — deliberate (#215's documented all-or-nothing choice, debug-asserted, and the AC-24 count/diagnostic surfaces make under-filtering visible on the coverage path). The document-side callers (`required_relations`, `trace_refs`, `vocabulary_coverage`, `obligation`) have no equivalent count, so a bypassing caller there under-filters with only the debug assert. Post-validation this is unreachable; recorded, not actionable | ix://agent-ix/quire-rs/spec/functional/FR-050 |
| FND-006 | low | Note, pre-existing: `reconcile`'s `document_references` regex compile keeps the `let Ok … else continue` shape (`src/coverage.rs`) that #215 eliminated for globs — same "patterns are validated at module load" justification, same theoretical bypass. Pre-existing, out of the batch's scope, consistent until someone extends #215's seam decision to regexes | ix://agent-ix/quire-rs/spec/functional/FR-050 |

## Per-commit disposition

| Commit | Ticket | Verdict |
| --- | --- | --- |
| 3ccdab3 | #218 | Accept — spec-only; pattern-legal local traces with foreign pointers preserved in prose; precedented cell form; no verification claim changed |
| 39347d6 | #212 | Accept with FND-001 (fixed) / FND-002 (filed) — gate real and wired into `ci`; the folded FR-026/FR-057 constraint-table conversions are ground-clearing the gate could not land without, verified content-preserving |
| 38c8451 | #213 | Accept — one-line dedup mirroring the sibling, both directions pinned (tc946 mutation-killed); TC-947 pins the smuggled `implements` fix against the engine, not only the schema |
| f154fc8 | #216 | Accept — the CR-087 policy is measured (status-row scoping, 100+ → 51), BTree-deterministic, advisory-first; the shipped TC-943/TC-944 duplicates re-idded with full matrix bookkeeping; the folded matrix-rot repair (TC-925 table split, TC-897 `\|\|`) is recorded in the CR note |
| 484a5fe | #215 | Accept — all three silences closed at the right seams; tc952/953/954 non-tautological (tc954 mutation-killed); tempfile adoption for TC-949 |
| 231aaf4 | #210 | Accept — no new parsing; public `parse_table` shape untouched; `ears::abs_line` off-by-one correctly refused and filed (#220); baseline regen reviewed; tc955 mutation-killed |
| 113269a | #214 | Accept — grammar edges pinned one by one (tc961), fixture + `extract_tree` integration coverage (tc958/tc960), legacy tags migrated to `#[trace]`; TC-959 allocation skip justified |
| 5608474 | #217 | Accept — finditer/span-replace/counted-refusals/R7/dirty-worktree guard all test-covered (18 tests, in `ci` via `check-scripts`); the three #208 placebo edits repaired and re-measured; SR-049→SR-050 collision fixed with an in-place dated correction note |
| fa98e0e | #219 | Accept — membership rules tested including both #202 behavior changes; one `is_test_data` with an identity assertion; import structure stated in all three entry modes; the 238/239 attribution documented at the source |
| 87a1869 | #179 | Accept with FND-003 (filed) — one shared `classify()` walk so warnings and records cannot drift; warning stream byte-unchanged; owned-wins-over-excused stated and pinned (tc964) |
| fa6f0d7 | #190 | Accept — smallest shape shipped; free-string + CON-6 never-interpreted mirrors the `applicability` discipline; additive both directions (tc969) |
| e6e82b9 | (retro) | Light check only — both artifacts `quire validate` exit 0; SR ids and findings cross-check against the tickets; the review's numbers reproduce (see SR-052) |

## Not fixed here

Findings FND-002/FND-003 are structural or sweep-shaped and carry issues
(#223, #224). FND-004..006 are notes without a failure scenario. The single
inline fix (FND-001) lands with this review's commit, referencing #212.
