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
