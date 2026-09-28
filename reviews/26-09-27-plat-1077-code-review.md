---
id: SR-120
title: "Code review — coverage_matrix, ignored symbols, trace-tag ranges (CR-187, quire-rs#495)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@271dba170a53f7b0b2e333b20a1845fdc1313d0f; src/coverage.rs, src/coverage/reconcile.rs, src/coverage/diagnostics.rs, src/symbol_table.rs, src/symbols/mod.rs, src/symbols/python.rs, src/symbols/rust.rs, src/symbols/trace.rs, src/symbols/typescript.rs, schemas/output/coverage-v1.schema.json, tests/coverage_matrix.rs, tests/coverage_baseline.rs, tests/output_contract.rs, tests/assurance_export.rs, tests/fixtures/coverage_baseline/**, tests/fixtures/semantic/baseline/published-schemas.json, tests/fixtures/traceability/iso-obligations, tests/fixtures/traceability/obligations-nfr-tagged, spec/functional/FR-050-declarative-coverage-computation.md, spec/tests.md, corpus (submodule pin 2ccc2e5)"
review_set: subset
---

## Summary

Ticket: PLAT-1077 (epic PLAT-1076). PR: quire-rs#495 at `271dba1`, diff
`origin/main...HEAD` (base `af5ec21`), 7 commits, 24 files, +2156/−109. Code
review with the Rust lane (`rust-review`) folded in. Scope: FR-050-AC-5/20/47..51
(CR-187, CR-188), FR-051-AC-3/18/21/23/27/28, TC-1930..TC-1944.

Method: read the full diff, then probed the behaviour in a scratch copy with an
extra integration test driving `extract_tree` and `trace::bind`. The PR
worktree was not touched. Then ran 16 hand-applied mutants against
`coverage_matrix`, `coverage_baseline`, `output_contract` and the lib unit
tests.

The core computation is sound and well pinned. Of the 16 mutants, 13 were killed and 3 survived (one of those is equivalent). Mutants on the status order
(`all`→`any`, method exemption demoted), the binder dedup key (column dropped,
qualified-name key), ignored inheritance removed, the marker range binding its
literal, the legacy left endpoint still binding, the range diagnostic
suppressed, the obligation-id-as-declared filter removed, a blanked
`statement`, the `unmatched_tags` range filter removed, and binder `ignored`
always serialized were all killed.

The defects are at the edges of the ignored and range rules. The worst one
(FND-001) is a false-green path. TypeScript ignored-ness is inherited by
qualified name, and FR-051-AC-21 makes TypeScript titles non-unique.

Regenerated goldens: `tests/fixtures/coverage_baseline/expected.json` is
byte-identical to the engine output (`tc824_coverage_report_matches_the_checked_in_baseline`
passes on the frozen head). Its delta is explained by the new `FR-003.md`, the
`obligations:` source added to the baseline module, the three new test fns
(which shift lines by +8) and the range finding. `published-schemas.json`
matches the committed schema bytes (`output_contract` passes). The schema
change itself was hand-authored (see FND-011).

## Verdict

**CONDITIONAL, not mergeable as-is.** One `high` false-green defect (FND-001).
Five `medium` defects break explicit AC-27/AC-28 clauses (FND-002..FND-006),
and one `medium` test-oracle gap (FND-007). Gates are green.

## Gate results (frozen head 271dba1, own CARGO_TARGET_DIR)

| Gate | Result |
| --- | --- |
| `make fmt-check lint test` | exit 0; 50 test binaries, 1145 passed, 0 failed, 0 ignored |
| `cargo test --test coverage_matrix --test coverage_baseline --test output_contract --test corpus_cases` | 8 + 2 + 11 + 19 passed, 0 failed |
| `quire coverage --scope . --json` (quire 0.33.0, engine 0.47.1) | TC-1930..TC-1944 all backed; no status lie or untracked symbol in scope |

## Mutation results

| Mutant | Result |
| --- | --- |
| status `all(ignored)`→`any(ignored)` | killed (TC-1934) |
| `untagged` checked before method exemption | killed (baseline, TC-1935) |
| dedup key `(path,line,0)` | killed (TC-1932) |
| dedup key on qualified name | killed (TC-1944) |
| `propagate_ignored` removed | killed (TC-1940, TC-1941) |
| marker range binds literal | killed (baseline) |
| legacy left endpoint binds | killed (TC-1937) |
| range diagnostic suppressed | killed (baseline) |
| obligation-id filter removed | killed (TC-1942) |
| `statement` blanked | killed by the baseline golden only; TC-1932 does not assert it |
| `unmatched_tags` range filter removed | killed (baseline) |
| range `dedup_by` removed | **survived** (FND-009) |
| binder `sort_by` removed | survived; an equivalent mutant, since the `BTreeMap` key is already `(path,line,column)` and unique |
| binder `ignored` always serialized | killed (baseline) |
| Rust `#[ignore]` not gated on `#[test]` | **survived**; no negative control for a non-test fn carrying `#[ignore]` (folded into FND-007) |
| TS `.skip` matched by prefix | killed (TC-1940 `skipIf` control) |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `propagate_ignored` resolves inheritance by `qualified_name` within a file (`own_ignored`/`container_of` are `BTreeMap<String,_>`, last-wins). FR-051-AC-21 makes TypeScript titles non-unique, and TC-1944's own premise is duplicate titles in one file. A running suite and a skipped suite (or a skipped test) that share a title therefore swap ignored-ness. Probed: `describe('s',{it('m')}); describe.skip('s',{it('n')})` gives m=true (false-red). The reverse order gives n=false, so a skipped test is not ignored and its criterion computes `tagged` (false-green). `describe('setup',{it('m')}); it.skip('setup')` also gives m=true. Fix: inherit through the adapter's own scope stack or by declaration site, never by title. | src/symbols/mod.rs:520, src/symbols/mod.rs:541 |
| FND-002 | medium | `is_pytestmark_skip_line` uses `starts_with("pytest.mark.skip")`, so `pytestmark = pytest.mark.skipif(sys.platform == 'win32', ...)` marks every test in the module ignored (probed: `test_a` ignored=true). AC-27 lists `skipif` as a control that may still run. Result: a false `tagged-by-ignored-test` for a whole module. Compare the head exactly, as `is_skip_decorator` already does. | src/symbols/python.rs:448, src/symbols/python.rs:456 |
| FND-003 | medium | AC-27 requires "the aliased-import forms AC-3's own unittest binding already resolves" to count identically. `is_skip_decorator` matches only the literal heads `pytest.mark.skip` and `unittest.skip`. Probed: `import unittest as ut; @ut.skip('y')` and `from unittest import skip; @skip('x')` both give ignored=false. That is a false-green, because such a test's criterion computes `tagged`. | src/symbols/python.rs:435, src/symbols/python.rs:441 |
| FND-004 | medium | Test fns inside `proptest! { ... }` are classified `TestFunction` (AC-3), but `scan_token_tree_for_fns` hard-codes `ignored: false`. So `#[test] #[ignore] fn prop_x(..)` inside `proptest!` is not ignored (probed). AC-27 says "on any function AC-3 already classifies as a test". False-green. | src/symbols/rust.rs:647 |
| FND-005 | medium | A range in an `implements` marker still mints an `ImplementsRelation` on the literal range string, and it produces no `range-in-trace-tag` finding. `bind_implements` never consults `range_pattern`. Probed with the `required-relations` module: `#[implements("FR-001..FR-003")]` gives `implements: ["FR-001..FR-003"]` and `ranges: []`. FR-051-AC-28: "mints no `verifies`/`implements` relation for any id it names". | src/symbols/trace.rs:1007, src/symbols/trace.rs:1026 |
| FND-006 | medium | Range endpoints leak into `mentions`. `find_mentions` scans every line with `generic_id_pattern` and suppresses only claimed ids, and a range claims nothing. Probed: `#[trace("FR-001-AC-1..FR-001-AC-2", "FR-001-AC-3..4")]` produces `Mention` records for FR-001-AC-1, FR-001-AC-2 and FR-001-AC-3 on that line. FR-051-AC-28 says the occurrence contributes to neither `unmatched_tags` nor `mentions`, "the range finding is the only record of it". `quire trace search` consumers see phantom citations. | src/symbols/trace.rs:741, src/symbols/trace.rs:786 |
| FND-007 | medium | Test oracles are weaker than their TC rows. TC-1932 promises `statement` (CR-188) and `(path,line,column,qualified_name,kind)` ordering, but asserts neither: the blanked-statement mutant is caught only by the baseline golden, and it uses `any()` rather than order. TC-1936 promises "reported once", naming the line and qualified symbol, but asserts only that the reason and value exist. No test covers the `mentions` or `implements` halves of FR-051-AC-28, which is why FND-005 and FND-006 passed the gate. The Rust `is_test &&` gate on `#[ignore]` survives its removal: no fixture has a non-test fn carrying `#[ignore]`. | tests/coverage_matrix.rs:127, tests/coverage_matrix.rs:323 |
| FND-008 | low | A legacy list that continues after a range loses its explicit ids. `// Trace: FR-001-AC-1..FR-001-AC-3, FR-001-AC-7` binds nothing, and FR-001-AC-7 lands in `unmatched_tags` as an `EvidenceNearMiss` (probed). The capture stops at `..`, which is pre-existing, but the tag now reads as "range reported" while an explicitly named id silently fails to bind. | src/symbols/trace.rs:1515 |
| FND-009 | low | The range `dedup_by` compares `(path,line,range_text)` after sorting by `(path,line,symbol,range_text)`. Equal entries from two symbols are adjacent only when nothing sorts between them. Binding kinds never nest in practice, so this looks like dead code: the removal mutant survived. Either drop it or make the sort key match the dedup key. | src/symbols/trace.rs:710 |
| FND-010 | low | Inheritance also marks `Function`-kind members of a skipped Python class or TS suite `ignored: true` in the symbol table. `Symbol::ignored`'s doc says it is `false` for `Function`, and AC-27 scopes the flag to test-kind and suite-kind symbols. | src/symbols/mod.rs:520 |
| FND-011 | low | `coverage-v1.schema.json` was re-serialized wholesale: every inline array or object was expanded, about 200 lines of whitespace-only churn in a published contract file. It is mixed into the feature diff, which hides the real schema delta (three `$defs` and one property). Not produced by a generator. | schemas/output/coverage-v1.schema.json:164 |
| FND-012 | low | `VerifiesRelation.kind: String` copies a `SymbolKind` label where the enum is available. The wire label belongs at serialization only (Rust idiom: typed domain values internally). `is_false` is also duplicated in `coverage.rs` and `symbol_table.rs`. | src/symbols/trace.rs:64, src/symbol_table.rs:101 |
| FND-013 | low | Two-writer spec seam: FR-051-AC-28 calls the finding "`TraceDiagnostic`-shaped", while FR-050-AC-50 routes it to `CoverageReport.diagnostics`. The code adds a third shape (`SymbolGraph.range_diagnostics: Vec<RangeInTraceTag>`), not `graph.diagnostics`. Align the FR-051 wording via CR. | spec/functional/FR-051-source-symbol-extraction.md:172, src/symbols/trace.rs:322 |

## New findings (disposition pass 1)

Reviewed at `agent-ix/quire-rs@12490853bcae9bdaef017eb4e4f6f6b80c3086ba`, rebased onto main `ff901d2`. The fix commits are `2414ebb`, `ad239e8`, `1673a5d`, `b69a5c9`, `383f255` and `d7c8f95` (CR-189). All 13 original probes now behave correctly, and 23 of 24 round-1 mutants are killed. The four findings below are new. FND-014 is a PR-introduced regression, and FND-015 was introduced by this fix round.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-014 | medium | A legacy range on a production symbol (a #312 misplaced tag) is now recorded only as generic `mentions`. The production path still discards the ranges (`let (production_forms, _ranges) = verifies_form_ids(...)`), so no `range-in-trace-tag` finding is raised. The endpoints no longer reach `non_binding_tags` and so land in `mentions` as plain `Mention`s. Probed: `// Trace: FR-001-AC-1..FR-001-AC-3` above `pub fn prod()` gives no range, no non-binding tag, and mentions FR-001-AC-1 and FR-001-AC-3. This breaks FR-051-AC-28 ("whether read by a canonical marker or a legacy textual form ... the range finding is the only record"). It also regresses main, where #312 reported the misplaced tag (its left endpoint) in `non_binding_tags`. Fix: push `_ranges` into `graph.range_diagnostics` on that path. | src/symbols/trace.rs:560 |
| FND-015 | low | Introduced by `1673a5d`. The `pytestmark` pre-pass scans raw physical lines (`lines.iter().any(\|line\| !line.starts_with([' ', '\t']) && is_pytestmark_skip_line(line))`), so a column-0 `pytestmark = pytest.mark.skip(...)` inside a module docstring or any triple-quoted string marks the whole module and every test in it ignored. Probed: that text in a module docstring gives `test_a` ignored=true. The previous AST walk read only statement start lines and did not have this bug. Scan the module's top-level `expression_statement` nodes instead. | src/symbols/python.rs:172 |
| FND-016 | low | The legacy-list continuation after a range binds ids through a hard-coded generic grammar (`list_item_pattern`) rather than the declared form's own pattern. Probed with `iso-obligations`: `// FR-001-AC-1..FR-001-AC-3, XYZ-9` binds `XYZ-9` through `comment-id`, whose declared pattern admits only the TC, FR, NFR, StR, US and IT prefixes. Trace-tag forms are module data (TC-745), so this widens a module's grammar without the module saying so. | src/symbols/trace.rs:1635 |
| FND-017 | low | No test covers TypeScript class-scope passthrough of ignored (`ignored: enclosing_ignored(scopes)` on a class scope). Mutating it to `false` survives. The comment claims a class inside a skipped suite passes ignored-ness to registrations nested inside it. Add a fixture or drop the claim. | src/symbols/typescript.rs:312 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 1673a5d | TS inheritance now reads `enclosing_ignored(scopes)` off the real scope stack. All three title-collision probes and a nested same-title suite probe are correct; mutant M4a is killed. |
| FND-002 | fixed 1673a5d | `is_pytestmark_skip_line` compares the head exactly (`head == "pytest.mark.skip"`). Probe: skipif gives ignored=false. Mutant M14 is killed. |
| FND-003 | fixed 1673a5d | `UnittestImports.skip_names` plus module-alias `.skip`. Probe: `@skip`, `@ut.skip` and `@sk` are all ignored. Mutant M15 is killed. |
| FND-004 | fixed 1673a5d | `flat_leading_span_and_test` returns `is_ignored`, gated on `is_test`. Probe: the proptest `#[ignore]` test is ignored. Mutants M12b and M12c are killed. |
| FND-005 | fixed b69a5c9 | `bind_implements` routes a marker range through `marker_range`. Probe: implements is empty and one range is reported. Mutant M5b is killed. |
| FND-006 | fixed b69a5c9 | `find_mentions` skips range endpoints keyed on (path, line, id). Probe: marker-range mentions are empty. Mutant M5d is killed. The production-symbol legacy path is still open, and is raised as the new FND-014. |
| FND-007 | fixed b69a5c9 | TC-1932 asserts `statement` and binder order. TC-1936 asserts exactly one finding, the path, line 5 and the symbol. The Rust non-test `#[ignore]` control was added in 1673a5d (M12a is killed). |
| FND-008 | fixed b69a5c9 | `read_legacy_match` resumes the list after a range. Probe: `A..B, C` binds C only, and the rewrite suggestion names only bound ids. A plain list without a range still produces the same rewrite; `A...` prose is not a range. Mutant M5c is killed. No rewrite regression found. The grammar-widening side effect is raised as the new FND-016. |
| FND-009 | fixed b69a5c9 | The dedup key now equals the sort key (including the symbol), and a two-form test was added. Mutant M9 is killed. |
| FND-010 | fixed 1673a5d | Python and TS gate inheritance to test-kind and suite-kind symbols. Helper tests were added; M16 and M17 are killed. Probe: the TS class method in a skipped suite has ignored=false. |
| FND-011 | fixed 2414ebb | The schema diff against origin/main is now 112 pure insertions, with no reformatting. |
| FND-012 | fixed ad239e8 | `VerifiesRelation.kind: SymbolKind`. The single `is_false` is in `symbol_table`. |
| FND-013 | fixed d7c8f95 | CR-189 rewords AC-28 to FR-050-AC-50's shape and line rule. |
| FND-014 | fixed ca6f2ba | Production-symbol legacy ranges are reported once against the innermost production symbol, their endpoints are kept out of `mentions`, and the #312 non-binding tag is kept (probe `q3`: range on `prod`, non-binding on `prod`, no mentions). Mutants N2, N4 and N5 are killed. The fix's side effect on test placement is raised as the new FND-018. |
| FND-015 | fixed 4721259 | `pytestmark` is read from module-level `expression_statement` assignments in the tree. Probes: docstring gives false; real assignment before or after the test gives true; `skipif` gives false; annotated assignment gives true; an `if`-block gives false (conditional, so correctly excluded). The M14b prefix mutant is killed. |
| FND-016 | fixed ca6f2ba | The list after a range is re-read with the declared form's own regex. `XYZ-9` no longer binds and is reported as an unmatched near-miss, while `TC-009` still binds. Probes: multi-range, plain-list, ellipsis and prose lines all behave. Mutants N6 and N9 are killed. |
| FND-017 | fixed 4721259 | Added `tc1940_a_class_passes_an_enclosing_skip_through`. Mutant M4c is now killed. |
| FND-018 | fixed cc49805 | A range non-binding tag is now dropped when an evidence symbol already reported that occurrence (path, line, text). Probes: a legacy range on a test inside `mod tests`, a Python module, a Python class or a TS `describe` gives one range against the test and no non-binding tag. A range on a production fn or a helper inside `mod tests` is still reported as misplaced. The same text on a prod fn and on a test, in one file or two, stays independent. Corpus cases through `coverage::compute`: every range case in all 3 languages emits only `range-in-trace-tag`, and every control emits neither. Mutant R1 is killed. |
| FND-019 | fixed cc49805 | Added `tc1937_a_container_loses_a_tie_for_a_production_range` (N3 is now killed) and `tc1937_a_list_resumes_only_where_the_form_rereads_it` (R2, R3 and R4 on the remaining guard clauses are all killed). |

### Round 1 gate (own CARGO_TARGET_DIR, head 1249085)

- `make fmt-check lint test`: exit 0, 50 binaries, 1156 passed, 0 failed.
- `cargo test --test coverage_matrix --test coverage_baseline --test output_contract --test corpus_cases --test corpus_recall`: 8, 2, 11, 19 and 2 passed.
- GitHub CI (`ci.yml`, workflow_dispatch): branch run 36442385187 and main (ff901d2) run 36442390037 fail the same two steps. `Rust Checks / Test` fails on `tests/quality_lints.rs::tc868_ears_and_ac_findings_are_unchanged`. `Static Audits / Run every static audit` fails because `check_dep_pins` hits `no matching package named libfuzzer-sys` offline. Neither is specific to this branch, so neither is a finding against this PR.

### Leader's judgment request: a test nested inside an `it.skip(...)` callback

This is **not a defect under AC-27 as worded (CR-189)**, so there is no finding. AC-27's inheritance clause runs from "a suite's or class's `ignored`" to its members. A test registration is neither a suite nor a class, so an `it` inside an `it.skip` callback carries only its own marking. Probe: `outer` ignored=true, `inner` ignored=false, which matches the text. Jest, Vitest and Playwright also reject nested tests at runtime ("Tests cannot be nested"), so no test that actually runs is misreported.

## New findings (disposition pass 2)

Reviewed at `agent-ix/quire-rs@74460aeb173ae5f4c5b5c925f0c6a9ae467bada8`. The fix commits are `ca6f2ba`, `4721259` and `74460ae`. FND-014..FND-017 are fixed, but `ca6f2ba` introduces the regression below.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-018 | medium | Introduced by `ca6f2ba`: a legacy range on a **test** now also produces a false `tag-on-non-binding-symbol` diagnostic against the enclosing container, in all three languages. The production path pushes a `NonBindingTag` for every legacy range read in any production symbol's span, and a container (`mod tests`, the Python module or class, the TS file module) spans its tests. The #312 "bound elsewhere" filter drops a tag only when its id bound somewhere, and a range never binds. Nothing drops it, even though the evidence symbol already reported the same occurrence as `range-in-trace-tag`. Probed: `mod tests { // Trace: A..B  #[test] fn t() }` gives non_binding_tags `[("tests", "A..B")]`. The corpus's own banked case `cases/attachment/range-in-legacy-trace-tag` (rust, python and typescript), run through `coverage::compute` with the ecosystem module, emits `tag-on-non-binding-symbol` naming the range beside `range-in-trace-tag`. Its control emits neither. The message tells the author their test is "a kind that does not bind trace ids" and to use `Implements:`. That is a wrong finding with a wrong remedy on correct placement. Fix: drop a range `NonBindingTag` whose (path, line, range_text) occurrence is in the evidence-reported set that `bind` already builds, and add a test. The case's own `expect.yaml` should also assert the reason is absent (SR-002 FND-003). | src/symbols/trace.rs:581, src/symbols/trace.rs:731 |
| FND-019 | low | Two branches of the round-2 range code are unpinned. Flipping the container-loses-ties key in the production-range ownership sort survives. Removing the `resumed_whole.start() != 0 \|\| resumed_group.start() != prefix.len() \|\| ...` anchor guard in `read_legacy_match` also survives. Add a same-`leading_line` container/function fixture and a case where the form's pattern re-matches away from the stand-in. | src/symbols/trace.rs:735, src/symbols/trace.rs:1702 |


### Round 2 gate (own CARGO_TARGET_DIR, head 74460ae)

- `make fmt-check lint test`: exit 0, 50 binaries, 1160 passed, 0 failed.
- `cargo test --test coverage_matrix --test coverage_baseline --test output_contract --test corpus_cases --test corpus_recall`: 8, 2, 11, 19 and 2 passed.
- Round-2 mutants: 9 run. 6 killed. N3 and N7 survived (FND-019), and N8 (statement-kind guard) survived as an equivalent mutant.

### Round 3 (head 165a1d3, fix cc49805)

- **Guard simplification (`resumed_whole.start() != 0` removed).** The removal is safe and the clause is unreachable for any practical pattern. When `resumed_group.start() == prefix.len()`, a match starting at k > 0 means the pre-group part matched `probe[k..prefix.len()]` and the group matched from `prefix.len()`. The original match shows the same pre-group can consume `probe[0..prefix.len()]`, which is the same bytes, and capture group 1 opens at one point in the pattern. So a match from 0 also exists, and leftmost-first semantics return it. The only escape is a leading assertion that reads text before position 0 (for example a pattern beginning `\b\W` or `\B`), because the probe starts at text start. Even then the kept clauses still pin the group to the stand-in, so the resumed ids are exactly this list's continuation. Mutating the kept clauses (R2, R3, R4) is caught.
- Re-ran the probes. FND-015 (docstring, after-test, skipif) and FND-016 (XYZ-9, multi-range, plain list, ellipsis, prose) still behave, with no regression.
- Gate (own CARGO_TARGET_DIR): `make fmt-check lint test` exit 0, 1163 passed; coverage_matrix 8, coverage_baseline 2, output_contract 11, corpus_cases 19 and corpus_recall 2 pass.
- No new findings. SR-120 has no open finding.
