---
id: SR-008
title: "Gap analysis — SR-007's three blockers (CR-059..CR-061), review findings resolved"
type: SpecReview
analysis: gap-analysis
scope: "spec/tests.md, spec/log.md, spec/functional/FR-050-declarative-coverage-computation.md, spec/functional/FR-051-source-symbol-extraction.md, src/corpus/declared_tables.rs, src/corpus/trace_refs.rs, src/coverage.rs, src/traceability.rs, src/loader/mod.rs, src/symbols/, benches/parse.rs, fuzz/fuzz_targets/, tests/fixtures/coverage_baseline/, tests/trace_dogfood.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/log", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references }
---

## Summary

Successor to **SR-007**, which returned **CONDITIONAL** on the SR-006 follow-up
program. All twelve original findings were resolved there; the verdict was held
back by three *new* mediums the fixes themselves surfaced — quire-rs #129, #124
and #126. This reviews the work closing those three: CR-059, CR-060 and CR-061,
opened as the stacked PRs #131 → #132 → #137.

SR-007 is not edited — it is the record of what was true on 2026-08-16.

**Plan completion.** N/A — ticket-driven, no plan bundle. Plan-001's staleness
(SR-006 FND-014) is untouched and remains a pre-existing finding.

**Matrix verification.** Measured with the **engine under test** — the branch
binder against the real `spec-artifacts-process` module, not the installed
`quire` 0.17.0, which pins quire-rs v0.26.0 and predates all three changes:

| Measure | SR-007 (v0.26.0) | This branch |
| --- | --- | --- |
| `diagnostics` | 6 × `unreadable-declared-document` | **0** |
| `status_lies` | 0 | **0** |
| `TC-577` backed | no | **yes** |
| `TC-579` backed | no | **yes** |
| `TC-502` backed | no | no — deliberate, see FND-006 |
| backed / total | — | 394 / 967 |

The six diagnostics were the finding that made SR-007 CONDITIONAL. The module
still declares all six `spec/evals.md` / `spec/matrix.md` sources and this
repository still has neither, so their absence from the report is CR-059
working, not the declarations having moved.

**Review pass.** A code review of the full stacked diff found five issues, all
**fixed in this pass** and each now pinned by a test rather than by prose. Two
are recorded below at the severity they would have carried unfixed, because one
of them reproduced the exact failure mode the program exists to eliminate.

## Verdict

PASS

Every AC in scope has a backing tagged test; the engine measurement over this
repository is clean; the five review findings are resolved and gated. The one
row that stays 🚧 is carved out deliberately, with its blocker named and filed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A trailing `//` comment on a `criterion_group!` line silently un-registered every bench it named: the argument offset was derived from the trimmed line and applied to the untrimmed one, which retains the whitespace the stripped comment left. Fixed by passing the slice instead of an offset; pinned by `tc827_a_trailing_comment_does_not_hide_the_registration` | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-002 | medium | `targets` was matched as a bare substring, so a bench or group *named* `targets_*` was parsed as the long-form clause and its siblings dropped. Now matched as a whole word followed by `=`; pinned by `tc827_a_name_beginning_with_targets_is_not_the_targets_clause` | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-003 | medium | FR-050-AC-15 and the field doc described the model-level `exclude:` as "not corpus data for any purpose", which over-claims: `validate_bundle` still schema- and grammar-checks an excluded document. AC, CR note and doc comments now state the three surfaces it actually scopes, and the boundary is asserted in `tc826` | ix://agent-ix/quire-rs/spec/functional/FR-050 |
| FND-004 | low | A second `fuzz_target!` in one file would have minted a second symbol with an identical `(language, path, qualified_name, kind)` identity, violating FR-051-AC-2. Guarded to one per file | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-005 | low | `benchmark` and `fuzz_target` are new values in the FR-045 record `kind` field, which no AC owned. FR-051-AC-17 now states the labels are part of both the symbol identity and the record | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-006 | low | TC-502 remains unbacked and 🚧. The blocker is `language_of`, which reads `.rs`/`.py`/`.ts` — a `.sh` audit is never opened, so no widening of the binder reaches it. Its matrix note is corrected to name that, and the shell-language question is filed as agent-ix/quire-rs#138 | ix://agent-ix/quire-rs/spec/tests |
| FND-007 | low | Pre-existing, out of scope: the AC→TC index carries 506 rows against 493 AC/CON ids declared across `spec/`. The difference predates this branch (CR-058 moved performance criteria out of the audit and retired rows remain indexed) and is untouched here | ix://agent-ix/quire-rs/spec/tests |

## Coverage

**Requirements in scope.** FR-050-AC-13, FR-050-AC-15, FR-050-AC-19 (amended),
FR-051-AC-17 (new). Every one carries at least one backing tagged test.

**Matrix rows in scope.**

| Row | State | Backing |
| --- | --- | --- |
| TC-825 | ✅ | 4 tests across `declared_tables.rs`, `coverage_rollup.rs`, `trace_references.rs` — including the first coverage of a *present but unreadable* document, which had none |
| TC-826 | ✅ | `coverage_rollup.rs`, `coverage_baseline.rs`, plus validation and cross-module merge cases |
| TC-827 | ✅ | 4 unit cases in `symbols/rust.rs` |
| TC-828 | ✅ | `symbols/trace.rs` unit case + `tests/trace_dogfood.rs` over this repo's own bench and fuzz files |
| TC-577 | 🚧 → ✅ | backed by `bench_validate_document`, confirmed against the real module |
| TC-579 | 🚧 → ✅ | backed by `fuzz_validate_extract_query`; `Type` corrected `Integration` → `Fuzz` |
| TC-502 | 🚧 | unchanged by design — FND-006 |

**Baseline.** The FR-050-AC-7 byte-identity baseline moved once, as intended:
`spec/fixtures/FR-900.md` leaves `criteria`, `totals.criteria` 4 → 3,
`totals.property_shaped` 3 → 2. CR-057's companion test pinned that leak on
purpose so it could not be absorbed; the assertion is inverted, not deleted.

**Underspecified code.** One reverse gap found (FND-005) and closed. Every other
symbol added by the three changes — `ExcludeSet`, `HarvestError`,
`SymbolKind::binds_trace_ids`, the criterion and fuzz-target recognisers — has an
owning AC.

**Semantic review.** Not run; not offered for this pass. The intent↔test↔code
question is partly answered mechanically here instead: `tests/trace_dogfood.rs`
asserts the claim over this repository's *own* files rather than over fixtures,
so a tag reverted to the `/`-separated form or moved into a `//!` header fails
the suite.

**Follow-ups filed, not done here.** agent-ix/quire-rs#138 (shell audits mint no
symbol) and agent-ix/spec-artifacts-process#37 (`/// A / B` binds only `A`).
