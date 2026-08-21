---
id: SR-009
title: "Retroactive code review — merged-unreviewed batch #202..#209 (CR-083..CR-085, slash sweep, v0.41.0)"
type: SpecReview
analysis: code-review
scope: "scripts/corpus.py, scripts/ac_corpus_sweep.py, scripts/classify_matrices.py, scripts/sweep_coverage.py, scripts/slash_tag_sweep.py, src/coverage.rs, src/symbols/typescript.rs, src/symbols/mod.rs, src/corpus/declared_tables.rs, src/traceability.rs, src/loader/mod.rs, spec/tests.md, spec/functional/FR-050-declarative-coverage-computation.md, spec/functional/FR-051-source-symbol-extraction.md, spec/reviews/SR-048-wave-b-gap-analysis.md, reports/2026-08-20-slash-trace-sweep.md, reports/2026-08-20-slash-trace-sweep.json, CHANGELOG.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/log", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references }
---

## Summary

Retroactive review of the seven PRs merged on 2026-08-20/21 without `/code-review`:
**#202** (corpus dedupe module), **#203** (CR-083 `undeclared_statuses`), **#204**
(CR-084 curried TS registrations), **#206** (CR-085 `source_exclude`), **#207**
(changelog, the v0.41.0 cut point), **#208** (slash-chain sweep + harness), **#209**
(TC-827 matrix repair). All seven were merged by their author with zero reviews; no
CI ran on any of them, which is by design (workflows are tag/dispatch-only) — the
missing gate is local and is FND-001. Every PR is reviewed **as merged**; every
defect found is cross-referenced to its ticket (#210–#219), and none is fixed here.

**Release exposure.** The `v0.41.0` tag is commit `7278e98` (#207's merge), so it
contains #202–#207 and **not** #208/#209. Verified at the tag: `spec/tests.md:330`
ships TC-827's summary row with `Traces To` = `FR-051-AC-18` and `Status` =
`TC-943`, plus a spurious `| FR-051-AC-17 | ✅ |` fragment on line 331, and the
AC→TC audit table has **no** `FR-051-AC-18` row at all although FR-051's coverage
row declares `AC-1..18`. **Published v0.41.0 carries this corrupted Test Matrix.**
The corruption is repaired on `main` by #209 (`954b315`); tags are never deleted,
so the remedy is the forthcoming **v0.42.0** tag — the first tag that will carry
the repair (WP3).

Engine measurements in this review were taken with the installed `quire` CLI
(reports `quire 0.23.0` — the quire-cli#52 version-stamping defect observed live;
per its own sweep report it embeds quire-rs v0.41.0, which contains every binder
behavior at issue) against `main` `954b315`, module `spec-artifacts-process`.

## Verdict

CONDITIONAL

The batch's engineering substance is real: CR-083 and CR-085 are well-tested
features, CR-084 fixes a genuine silent-loss class, #209 is a correct repair. But
the batch also shipped a corrupted spec matrix inside a release, minted duplicate
test-case ids twice, folded an untracked fix into an unrelated PR, and committed a
sweep harness whose census cannot be re-derived and whose edits are not all
effective. Every finding below is ticketed; the verdict clears when #212–#219 land
(tracked in WP3).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `make ci` never runs `quire validate` against this repo's own `spec/tests.md`. #204 corrupted the matrix, every local gate stayed green, and the corruption shipped in v0.41.0; #209 names the gap and adds no gate. Ticket agent-ix/quire-rs#212 (NR-1) | ix://agent-ix/quire-rs/spec/tests |
| FND-002 | high | v0.41.0 ships the TC-827 corruption: `Traces To` overwritten `FR-051-AC-17` → `FR-051-AC-18`, `Status` overwritten `✅` → `TC-943`, a stray `| FR-051-AC-17 | ✅ |` row fragment, and the intended `FR-051-AC-18` audit-table row never written. Cause verified from the diffs: #204 inserted its audit row by single-shot string replace on `| FR-051-AC-17 |`, which occurs twice in the file, and the replace hit the summary row. Repaired on main by #209; remedy is the v0.42.0 tag | ix://agent-ix/quire-rs/spec/tests |
| FND-003 | medium | #203: `undeclared_statuses` is sorted (`src/coverage.rs:685`) but never deduplicated — `untracked_symbols` gets `.dedup()` ten lines later and this list does not — so duplicate matching rows yield duplicate records, and no test pins either outcome. Ticket #213 (NR-2) | ix://agent-ix/quire-rs/spec/functional/FR-050 |
| FND-004 | medium | #203 folds an unrelated engine-contract fix into CR-083: `implements` added to tc859's optional-key list in `tests/output_contract.rs` (it has carried `skip_serializing_if` since CR-080). Real fix, zero matrix presence — no TC row, no CR note names it. Ticket #213 (NR-2) | ix://agent-ix/quire-rs/spec/tests |
| FND-005 | medium | #204: the widened TS registration grammar (curried modifier chains, `TITLE_LOOKAHEAD_LINES = 3` forward scan) is exercised only through crate-private `parse()` unit tests inside `src/symbols/typescript.rs`; no fixture-tree or integration coverage, so the extractor's walk-to-binder path for these shapes is untested. Ticket #214 (NR-3) | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-006 | medium | #204 tags both of its tests with the **legacy doc-comment form** (`/// TC-943, FR-051-AC-18 (CR-084)`) three weeks after #201 adopted the canonical `#[trace(...)]` marker this file's neighbors use. Ticket #214 (NR-3) | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-007 | medium | Duplicate test-case ids bound to multiple symbols, twice in one batch: `TC-943` on two test fns (`src/symbols/typescript.rs:550,605`, #204) and `TC-944` on two test fns (`src/symbols/mod.rs:432,479`, #206). `quire coverage` on this repo reports no diagnostic for either — the defect class is undetected by the engine. Ticket #216 (NR-5) | ix://agent-ix/quire-rs/spec/tests |
| FND-008 | medium | #206: `ExcludeSet::compile` (`src/corpus/declared_tables.rs:76`) drops a non-compiling glob with `if let Ok` — partial filtering, no diagnostic. Load-time `TraceabilityModel::validate` does reject invalid patterns (tc945), but any path reaching `compile` without it filters silently; pre-existing, now routed to a user-declarable key. Companion defects: `check_excludes` error text says ``invalid `exclude` pattern`` for a `source_exclude` value (`src/traceability.rs:1066`; tc945 only asserts `contains("source_exclude")`, satisfied by the location prefix, so it cannot catch this), and the walk's subtraction is a bare `continue` (`src/symbols/mod.rs:283`) with no excluded-file count in human or JSON output. Ticket #215 (NR-4) | ix://agent-ix/quire-rs/spec/functional/FR-050 |
| FND-009 | low | #206: `tc944_source_globs_cannot_un_exclude_the_document_root` builds its tree under `std::env::temp_dir()` with manual `remove_dir_all` — no `TempDir` guard, so the directory leaks on panic. Ticket #215 (NR-4) | ix://agent-ix/quire-rs/spec/functional/FR-050 |
| FND-010 | medium | #208: 55 comment-line edits are trace-binding behavior changes shipped with no new test; the PR's "+46 backed ids" was verified manually, not by a committed measurement. Re-measured in this review: aggregate holds (838/1163 backed, `status_lies` 0, matching #209's recorded numbers) but **three edits are placebo** — see FND-011. Harness correctness overall is ticket #217 (NR-6) | ix://agent-ix/quire-rs/reports/2026-08-20-slash-trace-sweep |
| FND-011 | high | NEW (not in the exploration seed). Four of #208's 55 GREEN lines carry an ` + <id>` tail (`// TC-473, FR-024-AC-4 + NFR-006: …`, likewise TC-483/FR-025-AC-4, TC-485/FR-025-AC-6, TC-498/FR-027-AC-6). The classifier classifies only the slash-joined span, but the legacy comma-list grammar still stops at the first id on such lines, so the second id never binds. Measured on main: `verification` rows **FR-024-AC-4, FR-025-AC-4, FR-027-AC-6** (all method Test) sit in `unbacked_rows` today; FR-025-AC-6 escapes only because its row is method Inspection. Three of the sweep's claimed conversions delivered nothing. Belongs on #217 (NR-6) | ix://agent-ix/quire-rs/spec/functional/FR-024 |
| FND-012 | medium | #208's committed harness `scripts/slash_tag_sweep.py`: `except (UnicodeDecodeError, OSError): continue` silently skips unreadable files uncounted; `CHAIN.search` sees only the **first** chain per line (a second slash-chain on the same line is neither counted nor edited); `rewrite_line` uses `line.replace(chain, …, 1)` rather than replacing at the match span; `--write` edits working trees across `~/dev` with no git-clean precondition. Ticket #217 (NR-6) | ix://agent-ix/quire-rs/reports/2026-08-20-slash-trace-sweep |
| FND-013 | medium | #208's report `reports/2026-08-20-slash-trace-sweep.md` declares `id: SR-049` — colliding with `spec/reviews/SR-049-fr059-code-review.md` (undetected because `reports/` is outside the document root) — and `type: Review` where this ecosystem's review artifacts are `type: SpecReview`. Arithmetic: the md claims "quire-rs's 56 GREEN lines are swept in this commit" (214−158) but the diff edits exactly 55 lines. Ticket #217 (NR-6) | ix://agent-ix/quire-rs/reports/2026-08-20-slash-trace-sweep |
| FND-014 | medium | NEW. The committed JSON census is only the **post-edit re-run** (`totals.green` 158; quire-rs `green: 0`, `files_edited: 0`, `write: false`). The before-numbers the md and PR quote (214 GREEN, 56 for quire-rs, backed 793→839) are re-derivable from no committed artifact — violating the rule the harness's own docstring states, that a reported number must be the census and not a reconstruction. Belongs on #217 (NR-6) | ix://agent-ix/quire-rs/reports/2026-08-20-slash-trace-sweep |
| FND-015 | low | NEW. The report md misattributes its own count: "**238 repositories** enumerated by `scripts/corpus.py`" — `corpus.py` enumerates 239 (verified by running it); 238 is the count *after* the harness excludes `ecaz` (JSON `repos_scanned: 238`, `excluded_repos: ["ecaz"]`). This is the concrete form of the 238-vs-239 discrepancy; #203/#207 correctly say 239. Cross-listed on #217/#219 | ix://agent-ix/quire-rs/reports/2026-08-20-slash-trace-sweep |
| FND-016 | medium | #209 repairs the corruption and correctly writes the missing `FR-051-AC-18` audit row, but adds none of the gate it identifies (FND-001, #212), and leaves the two pre-existing validate failures it names unticketed: `spec/tests.md:145` `Traces To` cells `FR-118 compatibility reference` and `Filament IDE FR-046 reference` — both reproduced by `quire validate spec/tests.md` in this review. Ticket #218 (NR-7) | ix://agent-ix/quire-rs/spec/tests |
| FND-017 | low | #202: `scripts/corpus.py` (149 lines) ships untested; consumers use bare `from corpus import …` (works only because Python puts the script's directory on `sys.path`); `ac_corpus_sweep.py:130` imports mid-file (E402, only F401 suppressed); `classify_matrices.py:77` keeps a byte-duplicate `is_test_data` — the "fifth divergent copy" the module's own docstring forbids; and the SKIP_DIRS + structural-worktree membership change moves every downstream census number. Ticket #219 (NR-8) | ix://agent-ix/quire-rs/spec/log |
| FND-018 | low | #207 is changelog-only and clean; noted here as the v0.41.0 cut point, which is what turns FND-002 from a working-tree defect into a released one | ix://agent-ix/quire-rs/spec/log |

## Per-PR disposition

| PR | Verdict as merged | Tickets |
| --- | --- | --- |
| #202 | Accept with findings — right consolidation, wrong hygiene | #219 |
| #203 | Accept with findings — feature well-tested (TC-941/TC-942, `#[trace]`-tagged); dedup gap + smuggled fix | #213 |
| #204 | Accept with findings — real fix; its own spec edit corrupted the matrix (shipped, FND-002); grammar unpinned; duplicate TC-943; legacy tags | #212, #214, #216 |
| #206 | Accept with findings — feature sound, one-way subtraction proven by tc944; silent-drop/observability gaps; duplicate TC-944; SR-048 edit **legitimate** (below) | #215, #216 |
| #207 | Accept — cut point recorded | — |
| #208 | Accept with findings — grammar correctly not widened; harness and census have the defects above; 3 of 55 edits placebo | #217 |
| #209 | Accept with findings — correct repair, missing gate not added, two failures left unticketed | #212, #218 |

## The SR-048 edit (#206) — verified legitimate

#206 modified `spec/reviews/SR-048-wave-b-gap-analysis.md`, which this repo's
convention treats as a dated record (SR-008: "SR-007 is not edited — it is the
record of what was true"). Verified from the diff: the edit is **purely additive**
— a dated, attributed supersession blockquote under the TC-999 triage line, which
itself is preserved verbatim ("Recorded rather than deleted: the triage was
correct when written"). Its factual claims cross-check: CR-085/#199 is the PR
itself; CR-078 (#198) did remove three of the fourteen; the re-measured 1
untracked symbol matches the live report (`untracked_symbols: 1`). This is the
correct way to annotate a review record, not a violation of it.

## Coverage

**Measurements backing this review** (installed CLI, module
`spec-artifacts-process`, repo at `954b315`):

| Measure | Value |
| --- | --- |
| `totals.backed / totals.total` | 838 / 1163 — matches #209's recorded post-repair figures |
| `status_lies` | 0 |
| `undeclared_statuses` | absent (empty) — CR-083's check is clean over its own repo after #209 |
| `untracked_symbols` | 1 |
| Edited-line ids from #208 still unbacked | FR-024-AC-4, FR-025-AC-4, FR-027-AC-6 (FND-011); all 52 other edits' ids back their rows |
| `quire validate spec/tests.md` | fails on exactly the two line-145 cells of FND-016 |

**Environment note.** Every `quire validate` run in this environment emits six
module-load diagnostics (`DuplicateArchetype` ×5, `DuplicateInverseEdge` ×1 —
`spec-artifacts-process` registered twice in the local module set; quoin#174
territory). They are diagnostics, not failures: the exit code still reflects
document validity, and this document and SR-010 both validate clean (exit 0, no
failure lines).

**Not fixed here.** Per the batch policy, every defect above is carried by
#212–#219 (NR-1..NR-8) plus the pre-existing #210 (line numbers, grew to include
`UndeclaredStatus`) and #211 (sweep tail, blocked on #217). This review changes
`reviews/` only.
