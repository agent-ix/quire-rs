---
id: SR-052
title: "WP3 pre-release gap analysis — does each closed ticket's acceptance hold, and is the Test Matrix truthful (v0.42.0 gate)"
type: SpecReview
analysis: gap-analysis
scope: "spec/tests.md, spec/log.md, spec/functional/FR-050-declarative-coverage-computation.md, spec/functional/FR-051-source-symbol-extraction.md, spec/functional/FR-054-verification-method-catalog.md, spec/functional/FR-059-declared-vocabulary-coverage.md, Makefile, examples/spec_validate.rs, scripts/, tests/, src/"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/log", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: references }
---

## Summary

The verification half of the WP3 pre-release gate, companion to SR-051 (code
review). Question one: does each closed ticket's acceptance actually hold, with
evidence, for #210 #212 #213 #214 #215 #216 #217 #218 #219 #179 #190? Question
two: is the Test Matrix state truthful — `make validate` and repo self-coverage
run and recorded, not quoted?

**Measurement method.** Working-tree engine at HEAD (never the installed
`quire` CLI, which reports 0.23.0 / embeds v0.41.0 and predates every change
under review), module `spec-artifacts-process` from the local checkout
(`~/dev/spec-artifacts-process` @ de8bf25). Self-coverage was computed with
`extract_tree_scoped(root, &[spec], &model.source_exclude)` — the CLI's path.
**Scaffolding trap, recorded for the next reviewer:** the unscoped
`extract_tree` walk (which the older `wave_b_gap` example uses) skips the
declared `source_exclude`, and reports 898/1221 backed, 1 untracked symbol and
52 shared ids — three numbers that look like drift against the CR-085/CR-087
records and are all measurement error. Under the correct walk every recorded
number reproduces exactly.

## Per-ticket acceptance

| Ticket | Commit | Acceptance | Holds? | Evidence |
| --- | --- | --- | --- | --- |
| #218 | 3ccdab3 | The two pre-existing `quire validate` failures on spec/tests.md (TC-768/TC-769 foreign-id Traces To cells) are gone without fabricating ids | **Yes** | `make validate` at HEAD: spec/tests.md among 127 documents, 0 failed. Foreign pointers preserved in row titles/status prose; FR-045/FR-046 rollup rows carry the TCs marked as downstream references; both rows stay 🚧 external (CR-058) |
| #212 | 39347d6 | (a) `make ci` green on clean main; (b) the #204 corruption class fails `make validate` and therefore `ci` | **Yes** | (a) `make ci` exit 0 at HEAD, `validate` in the chain. (b) Re-verified in this review: a corrupted matrix row id → 1 error, exit 1; and after SR-051 FND-001's fix, a corrupted frontmatter fence → exit 1 naming the file (before the fix that class silently shrank the population to 126 documents, green — the residual audit-table-drift class is filed as #223) |
| #213 | 38c8451 | `undeclared_statuses` dedups like its siblings, both directions pinned; the smuggled `implements` fix gets a matrix record | **Yes** | tc946 red-verified by mutation in SR-051 (dedup removed → test fails); distinct drifted values both survive (second half of tc946). TC-947 pins empty-key omission + populated round-trip against the engine. CR-086 log entry, TC rows, audit rows FR-050-AC-21/FR-055-AC-6 all present |
| #216 | f154fc8 | `shared_trace_ids` reports a status-row id bound by N distinct symbols; scoped, deterministic, advisory, additive; the two shipped duplicates resolved | **Yes** | tc950 (positive) + tc951 (scoping + byte-identity) — tc951 red-verified by mutation (scoping removed → fails). Repo measurement: 51 shared ids, matching the CR-087 record exactly; binder-set diff between f154fc8 and HEAD is empty, so the later commits in this batch minted **no new duplicate** — the check now guards the defect class that produced TC-943×2/TC-944×2. TC-948/TC-949 re-ids landed with full matrix bookkeeping; neither appears in the list |
| #215 | 484a5fe | `source_exclude` subtraction counted end to end; invalid glob refuses the whole list loudly; error names the key as the noun | **Yes** | tc952 (walk counts), tc953 (extraction → graph → JSON, absent-never-0), tc954 red-verified by mutation (partial filter restored → fails). tc945 asserts ``invalid `source_exclude` pattern`` as the message noun. Live on this repo: `excluded_source_files: 6` with the module's declared `tests/fixtures/**` family |
| #210 | 231aaf4 | Row-shaped records and `untracked_symbols` carry 1-based document/declaration lines; contract additive both directions; baseline regen reviewed | **Yes** | tc955 red-verified by mutation (`+2`→`+1` → fails against hand-counted fixture lines), tc956 (symbol declaration line), tc957 (omitted-never-null both directions). Baseline diff carries every recovered line; tc824/tc855 green at HEAD. The `ears::abs_line` off-by-one was correctly refused and filed (#220) rather than inherited |
| #214 | 113269a | The CR-084 grammar pinned edge by edge and exercised through `extract_tree`; legacy tags migrated | **Yes** | tc961 pins 4 admitted + 4 refused edges; tc958/tc960 read the new `registration.test.ts` fixture through `extract_tree` (8 positive titles in order, 5 negative shapes absent); TC-943/TC-948/TC-798 now `#[trace]`-tagged. TC-959 allocation skip documented in the CR-090 note |
| #217 | 5608474 | Harness counts all chains, span-replaces, counts refusals, guards R7, refuses dirty worktrees; census re-derivable; the three placebo edits repaired; report re-identified | **Yes** | 18 tests in `scripts/tests/test_slash_tag_sweep.py`, run by `make check-scripts` inside `ci` — multi-chain census, span-vs-str.replace, unreadable-file refusal, R7 verbatim placebo shape + repaired form, dirty/clean/non-git preconditions. Repair measured at HEAD: FR-024-AC-4, FR-025-AC-4, FR-027-AC-6 all **backed = true**; `unbacked_rows` 279 matches the report's after-number. Report now `id: SR-050`, `type: SpecReview`, validates exit 0, with the in-place dated correction note (55 not 56; 239 enumerated vs 238 scanned) |
| #219 | fa98e0e | corpus.py membership rules tested (incl. both #202 behavior changes), one `is_test_data`, import structure stated, count documented | **Yes** | 9 tests in `scripts/tests/test_corpus.py`: spec-dir membership, SKIP_DIRS/hidden/SUPERSEDED at top level, structural worktree check (`.git`-file skipped, name-only kept), sorted order, any-depth pruning, `is_test_data` boundaries, `classify_matrices.is_test_data is corpus.is_test_data` identity, and a subprocess pin of `python -m scripts.slash_tag_sweep --help`. Docstring states what 239 counts and forbids the 238 mislabel |
| #179 | 87a1869 | Coverage payload classifies every declared vocabulary value owned/excused/unowned from one shared walk; dead declaration is a coverage diagnostic; uncatalogued-method diagnostic carries `value` | **Yes** | tc962/tc963/tc964 (one record per value in enum order, deciding documents, owned-wins-over-excused), tc965 (absent-key byte-identity), tc966 (dead declaration diagnostic, same token as the bundle warning), tc967 (`value` byte-equal to the obligation's `method`, omitted elsewhere). Warning stream byte-unchanged — `check_coverage` rebuilt on the same `classify()` the records read |
| #190 | fa6f0d7 | Catalog entry can declare `cost`; stored and surfaced, never interpreted; additive both directions | **Yes** | tc968 (verbatim through the accessor, absent reads `None`), tc969 (serialized entry omits undeclared cost, carries declared). FR-054-AC-13 + CON-6 authored; fixture manifest declares `cost: assurance-only` on one entry and none on another |

**e6e82b9 (light check).** Both retro artifacts (`SR-009`, `SR-010`) `quire
validate` exit 0; their finding→ticket map matches the tickets closed above;
their coverage numbers (838/1163 at 954b315) belong to the pre-batch tree and
are superseded by the measurements below.

## Test Matrix truthfulness — the numbers

`make validate` (working-tree engine, full module set), at HEAD + SR-051 fix:

| Measure | Value |
| --- | --- |
| documents validated | **127** |
| failed | **0** |
| warnings (advisory grammar/quality) | **39** |
| untyped skipped | **0** (the two frontmatter-less notes live under `spec/assets/`, outside the corpus) |

Repo self-coverage (working-tree engine, scoped walk with the module's
`source_exclude`):

| Measure | Value |
| --- | --- |
| `totals.backed / totals.total` | **897 / 1221** (up from 838/1163 at 954b315: the CR-087 matrix-rot repair un-split 33 rows, the batch added TC-946..TC-969 minus the skipped TC-959, and #217's three repaired ids left `unbacked_rows`) |
| `status_lies` | **0** ✔ (gate requirement) |
| `undeclared_statuses` | **0** (CR-083's check clean over its own repo) |
| `untracked_symbols` | **0** ✔ (the deliberate TC-999 fixture symbol is subtracted by the module's declared `source_exclude`, exactly as CR-085 recorded) |
| `shared_trace_ids` | **51** — advisory corpus debt, the pre-existing several-fns-per-row convention (TC-609 ×6, TC-528 ×3, …); set identical between f154fc8 and HEAD, so this batch added none |
| `excluded_source_files` | **6** (the AC-24 count, live) |
| `unbacked_rows` | **279** (matches the SR-050 report's post-repair number) |
| `no_symbol_rows` | **1** |
| `diagnostics` | **0** |
| #217 repaired ids backed | FR-024-AC-4 ✔, FR-025-AC-4 ✔, FR-027-AC-6 ✔ |

Matrix bookkeeping cross-checks: every new TC id (TC-946..TC-958, TC-960..TC-969)
has a Test Case Summary row, an AC→TC audit row, a rollup-row mention and a
`#[trace]`-tagged test; TC-959 is a documented allocation skip; TC-970+ unused
(frontier claim holds, verified by grep); every new AC (FR-050-AC-23..26,
FR-054-AC-12/13, FR-059-AC-9/10) is enumerated in its FR, its rollup range, and
the audit table; every CR note (CR-086..CR-092) resolves to its log entry and
ticket. `spec/reviews/` SR ids and `reports/` SR-050 no longer collide; SR-051
and SR-052 (this document) take the next free ids across `reviews/`,
`spec/reviews/` and `reports/`.

## Findings

Gaps and residuals — none blocking the tag.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Audit-table drift stays ungated (SR-051 FND-002, filed #223): a well-formed matrix missing a declared AC's audit row passes `make validate`; the class lives on the ungateable-wholesale bundle surface. Known, filed, not a regression of this batch | ix://agent-ix/quire-rs/spec/tests |
| FND-002 | low | Test-hygiene sweep (SR-051 FND-003, filed #224): two new manual temp-dir trees in #179 against the #215 `TempDir` direction. Leak-on-panic only | ix://agent-ix/quire-rs/spec/log |
| FND-003 | low | Release prerequisite outside this review's scope: `CHANGELOG.md` has no `[0.42.0]` section yet — the v0.41.0 cut was its own commit (#207). The coordinator's tag flow should include the changelog cut; nothing in the reviewed batch blocks it | ix://agent-ix/quire-rs/spec/log |

## Verdict

Every closed ticket's acceptance holds with evidence, the Test Matrix state is
truthful under the engine under test (`status_lies` 0, `untracked_symbols` 0,
`undeclared_statuses` 0, gate green with the SR-051 hardening), all findings
are fixed inline or filed (#223, #224), and `make ci` is green end to end.

**v0.42.0: GO**
