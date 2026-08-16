---
id: SR-007
title: "Gap analysis — SR-006 follow-up program (CR-050..CR-058, review findings resolved)"
type: SpecReview
analysis: gap-analysis
scope: "spec/tests.md, spec/log.md, spec/functional/FR-005-parse-document-api.md, spec/functional/FR-024-parallel-repo-walk.md, spec/functional/FR-044-project-glossary-lexicon.md, spec/functional/FR-050-declarative-coverage-computation.md, src/parser/, src/corpus/, src/symbols/, scripts/audits/check_no_shared_mutable.sh, tests/fixtures/parser_golden/, tests/fixtures/coverage_baseline/"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/log", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-005", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-024", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-044", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references }
---

## Summary

Successor to **SR-006**, which returned **FAIL** on the #90 program and produced
13 tickets (umbrella #106). This reviews the follow-up program that closed them:
CR-050..CR-058 in quire-rs (released v0.26.0), quire-cli #32/#33, quoin #86.

SR-006 is not edited — it is the record of what was true on 2026-08-15.

**Plan completion.** N/A — ticket-driven again, no plan bundle. Plan-001's
staleness (SR-006 FND-014) is untouched and remains a pre-existing finding.

**Matrix verification.** Measured with the engine under test (quire-cli built
against quire-rs v0.26.0, not the installed 0.17.0): `quire coverage --scope .`
reports **zero status lies**, down from 10 at SR-006. Backed trace ids 376 →
380. Every TC this program added (TC-819..TC-824) and TC-502 carry a real tag.
The headline is now stated as **mapping** completeness — 505/505 ACs mapped, of
496 TC rows 192 ✅ / 279 🚧 / 25 retired — which was SR-006 FND-010's last part.

**Underspecified code.** Two behaviors that SR-006 found enforced by prose now
have criteria and tests: `parse_body`'s totality (FR-005-AC-7) and the
walk→bundle bridge (FR-024-AC-12). Two claims that rested on assertion now rest
on gates: parser byte-identity against a **pre-refactor capture** (FR-005-AC-8,
measured reproducing exactly) and the coverage report against a **checked-in
baseline** (FR-050-AC-20).

Three findings below are **new**, all surfaced by the fixes themselves rather
than by re-reading the same code.

## Verdict

**CONDITIONAL** — every SR-006 finding is resolved and no high finding stands.
The three new findings are all mediums, all filed, and two of them are
consequences of gates that did not exist to fire before. SR-006 predicted
"the residual mediums support CONDITIONAL at that point"; that is where this
lands, on different mediums than it expected.

Not PASS, for one reason worth stating plainly: **FND-101** means a
fleet-wide module declaration now emits six diagnostics on this repo's own
spec, and shipping that to 200+ repositories before splitting absent from
unreadable would be the noise-that-nobody-reads failure this program spent
CR-053 and CR-058 removing.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-101 | medium | **New.** CR-054 conflates an **absent** declared auxiliary document with an **unreadable** one. `spec-artifacts-process` names `spec/evals.md` and `spec/matrix.md` as optional auxiliary sources; quire-rs has neither, so the v0.26.0 engine emits six `unreadable-declared-document` diagnostics on its own spec — true, and noise. Split the reason: non-`NotFound` always reported, `NotFound` only when the model minted nothing, the rule `archetype-matches-nothing` already uses. | #129, FR-050-AC-19, CR-054 |
| FND-102 | medium | **New.** `exclude:` globs scope trace targets and document references but **not** the CR-028 criteria walk, which has no declaration to hang an exclusion on. Deliberately malformed fixture data therefore still inflates the criteria denominator and is body-parsed during coverage — the class CR-038 fixed for trace targets. Pinned deliberately in the #114 baseline so closing it produces a reviewed diff. | #124, FR-050-AC-13, CR-028, CR-038 |
| FND-103 | medium | **New.** `trace::bind` binds trace ids on **test functions only**, so a criterion bench, a `fuzz_target!` body and a shell audit can never back a matrix row however they are tagged (verified by tagging them and re-running). TC-502/577/579 are consequently 🚧 while the work behind them runs on every `make ci`. CR-041's `no_source_symbol` vocabulary is the designed answer and lives in the module, not here. Related: a `///` doc-comment tag binds only the first id on the line, so `/// NFR-002-AC-4 / TC-577` silently drops the second. | #126, FR-050-AC-16, FR-051, CR-041 |
| FND-104 | low | Carried from SR-006 FND-013, unchanged: CR-050..CR-058 exist only as prose bullets in `spec/log.md` — no frontmatter, not engine-validatable. This program added nine more of them, so the convention is now load-bearing for 58 entries. | CR-050..CR-058 |
| FND-105 | low | Carried from SR-006 FND-014, unchanged and still not attributable: Plan-001 is `status: active` with Task-010 `not_started`. | Plan-001, Task-010 |

## Resolution of SR-006

| SR-006 | Resolved by | Evidence |
| --- | --- | --- |
| FND-001 high — `parse_body` panics | CR-050, #107 | FR-005-AC-7 + TC-819; offset re-derived, proptest over arbitrary `(a, b)` pairs |
| FND-002 high — TC-813 pins nothing | CR-052, #108 | FR-005-AC-8 + TC-821; golden corpus captured from `7b1db82`, the commit **before** CR-046; current engine reproduces it byte-for-byte |
| FND-003 high — mitigations unenforced | CR-053, #109 | audit in ci.yml + `sanitize` in `make hardening`; exemptions path-scoped, exact-line, stale-checked, `why` printed; TC-816 widened to 8×16 + the rayon-forcing shape; TC-502 given a real identity |
| FND-004 high — CR-048 machine surface | CR-051 + quire-cli#33 | FR-024-AC-12 + TC-820 (distinct `malformed-frontmatter` reason); IT-088 asserts `severity: "warning"` end to end through the binary |
| FND-005 high — quire-cli spec contradicts behavior | quire-cli#32 | FR-014-AC-6 / FR-015-AC-5 corrected with CR notes; `--help`, README, CHANGELOG 0.16.0 + 0.17.0 |
| FND-006 medium — selection fails open | CR-054, #111 | FR-050-AC-19 + TC-822; `CoverageReport.diagnostics`, absent when empty so AC-7 byte-identity holds. **Surfaced FND-101.** |
| FND-007 medium — glossary narrowed | CR-055, #112 | FR-044-AC-8 + TC-823; pre-filter normalizes exactly as the lookup does; `glossary_terms_from_path_with_diagnostics` |
| FND-008 medium — two-root residuals | CR-056 + quire-cli#33 | exclusion compared by canonicalized identity; `spec_root_of` canonicalizes; typed `DocumentRootError` with a stable `kind`; TC-814's `exclude:` half covered |
| FND-009 medium — byte-identity gate | CR-057, #114 | FR-050-AC-20 + TC-824; checked-in baseline + `make coverage-baseline-update`, plus a companion test that fails if the corpus stops exercising the surface. **Surfaced FND-102.** |
| FND-010 medium — matrix hygiene | CR-058, #115 | 10 status lies → **0**; the four filed rows re-derived against the tool (three had tests, untagged); PC treatment unified; headline reworded. **Surfaced FND-103.** |
| FND-011 medium — quire-cli test gaps | quire-cli#33 | FR-017/FR-018 authored; IT-080, IT-086, IT-087, IT-089, IT-092..097 added |
| FND-012 medium — quoin docs | quoin#86 | false exit-0 claim corrected against the code; notes backported to all 7 sites; `no_symbol_rows`/`criteria`/`diagnostics` added to the field table; version probe stated |
| FND-013 low — CRs are prose | — | carried as FND-104 |
| FND-014 low — Plan-001 stale | — | carried as FND-105 |

## Coverage

| Check | Result |
| --- | --- |
| Plan completion gate | N/A — ticket-driven program, no plan bundle |
| SR-006 findings resolved | 12 / 14 (2 carried as low, unchanged and pre-existing) |
| SR-006 high findings outstanding | 0 / 5 |
| Defined non-retired ACs with a mapped TC | 505 / 505 |
| Program TCs (TC-819..TC-824 + TC-502) backed by a tagged test | 7 / 7 |
| `quire coverage` status lies | **0** (was 10) |
| Backed trace ids | 380 / 962 (was 376) |
| All-matrix TC rows ✅ | 192 / 496 (279 🚧, 25 retired — pre-existing backlog) |
| CR→PR mappings verified | 9 / 9 (CR-050..CR-058 → #117..#123, #125, #127) |
| Engine used for measurement | quire-cli built against quire-rs **v0.26.0** — not the installed 0.17.0 |
| Semantic review (step 4) | skipped — not opted into |

## Notes

Every number above was re-derived from the tool or the file, not quoted from a
summary. Two of SR-006's own numbers did not survive that: it reported four
status-lie rows where the engine reports ten, and three of the four it named
(TC-563, TC-564, TC-583) had real tests that were merely untagged. The
correction is recorded in CR-058 rather than left in the ticket.
