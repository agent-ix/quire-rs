---
id: SR-077
title: "Gap analysis — Plan-003 semantic extraction boundary"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-003-semantic-extraction-boundary/ (Task-015..Task-022); spec/usecase/US-019; spec/functional/FR-069..FR-072; spec/non-functional/NFR-021; spec/tests.md rows TC-1599..TC-1650; src/semantic/**, src/loader/, src/filament.rs, src/validate_document.rs, src/python/mod.rs, schemas/; tests/semantic_*.rs, tests/python/test_semantic.py, tests/fixtures/semantic/**"
review_set: all
relationships:
  - target: ix://agent-ix/quire-rs/Plan-003
    type: reviews
  - target: ix://agent-ix/quire-rs/TM-001
    type: references
  - target: ix://agent-ix/quire-rs/US-019
    type: references
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
  - target: ix://agent-ix/quire-rs/FR-070
    type: references
  - target: ix://agent-ix/quire-rs/FR-071
    type: references
  - target: ix://agent-ix/quire-rs/FR-072
    type: references
  - target: ix://agent-ix/quire-rs/NFR-021
    type: references
---

## Summary

Post-implementation gap analysis of Plan-003 (agent-ix/quire-rs#388, branch
`spec/388-semantic-extraction-boundary`) over US-019, FR-069..FR-072, NFR-021,
matrix rows TC-1599..TC-1650, and the `src/semantic` implementation. Seven of
eight tasks are `completed`; the review gate Task-022 is `todo`. Every
acceptance criterion of FR-069, FR-070, FR-071 and NFR-021 is engine-backed
(32/32); FR-072 is 8/9 with FR-072-AC-6 open on its declared-external WASM leg
(TC-1636, agent-ix/quire-wasm#3). Every executable gate I ran is green. The
findings are traceability and record-vs-spec drift: two matrix rows bound to
two tests each, two rows whose bound test asserts less than the row claims,
one contract claim (`--features wasm` parity) verified by no test run, and a
fence scanner that is a parallel implementation rather than the locator reuse
FR-071-CON-2 and Task-019 describe.

## Verdict

**FAIL** — Task-022 (review gate) is `status: todo` with all four subtasks
open, which the verdict rule scores as an incomplete task. No `high` finding
and no status lie in range; the remaining findings are `medium`/`low`, so the
plan would score CONDITIONAL once Task-022 closes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-460 | medium | Task-022 is `status: todo`; plan table row reads `todo`; all four subtasks open. This artifact is one of its deliverables, so the gate cannot be `completed` before it exists — reported, not excused | Task-022, plan.md Task File Mapping |
| FND-461 | medium | TC-1610 is bound to two tests: `tests/semantic_fixtures.rs::quoin_fixtures_match_provenance` (asserts fixture SHA-256 provenance only, no extraction) and `tests/semantic_properties.rs::golden_table_extracts_to_expected_fields` (asserts the row). Engine `shared_trace_ids`. The provenance test needs its own row or no TC tag | TC-1610, FR-070-AC-1, tests/semantic_fixtures.rs:45 |
| FND-462 | medium | TC-1630 is bound to two tests: `tests/semantic_fixtures.rs::semantic_cases_are_attributed_and_uniquely_named` (asserts `issue_ref`/unique names only) and `tests/semantic_surface.rs::case_suite` (asserts the row). Engine `shared_trace_ids` | TC-1630, FR-072-AC-1, tests/semantic_fixtures.rs:107 |
| FND-463 | low | TC-1635 is bound to two Python tests (`test_extract_semantic_matches_rust_for_every_case`, `test_extract_filament_core_carries_the_semantic_record`); TC-1644 to one Rust and one Python test as declared halves. Not one-to-one; the halves are named in docstrings, so the split is legible | TC-1635, TC-1644, tests/python/test_semantic.py, tests/semantic_surface.rs:365 |
| FND-464 | medium | TC-1639 and TC-1643 both bind to `tests/semantic_baseline.rs::filament_graph_cases_match_baseline`, which asserts the Filament graph baseline and the severity set only. The coverage-v1/properties-v1/assurance-v1 byte-identity both rows claim is delegated by comment to TC-1089/TC-1090 (untagged for these rows); "no existing contract schema gains a required key" (NFR-021-AC-3) is asserted nowhere. Rows overclaim relative to the bound test | TC-1639, TC-1643, FR-072-AC-9, NFR-021-AC-3, tests/semantic_baseline.rs:223 |
| FND-465 | medium | TC-1603 and FR-069-AC-5 claim identical results under `--no-default-features --features wasm`; no test executes under that feature set (Makefile and ci.yml carry only `cargo check` via `make check-wasm`). `ref_rules` runs under default features. The wasm-parity half of the ✅ is unverified by execution | TC-1603, FR-069-AC-5, Makefile:334, tests/semantic_contract.rs:377 |
| FND-466 | medium | FR-071-CON-2 says spans derive from the `code_block` locator's fence recognition and Task-019 lists "scanner refactor in `src/extract/locator.rs`". The branch leaves `locator.rs` untouched and adds a parallel scanner `src/semantic/scan.rs` that "follows the parser". TC-1628 asserts agreement with `extract_diagrams` on three fixtures. Record differs from spec: reuse was specified, agreement-by-test was built | FR-071-CON-2, Task-019, src/semantic/scan.rs, tests/semantic_boundary.rs:196 |
| FND-467 | low | TC-1628 claims `parse_document` output byte-identical to the parser golden before and after the change; the bound test asserts only that `operations.md` still parses a section. The byte-identity rests on `tests/parser_golden.rs`, which carries no TC-1628 tag. Row should say it is delegated | TC-1628, FR-071-CON-2, tests/semantic_boundary.rs:196 |
| FND-468 | low | TC-1618 claims a record equal to the checked-in baseline byte-for-byte; `no_block_record_is_unchanged` asserts only no semantic digest and no semantic diagnostic on `tests/fixtures/modules/bundle`. The byte comparison sits in TC-1607 (registry projection) and TC-1632 (Filament graph). Row should name the delegation | TC-1618, FR-070-AC-9, FR-070-CON-3, tests/semantic_properties.rs:568 |
| FND-469 | low | TC-1599 says `quire validate` validates the extracted record against the resolved schema; the bound test calls `data_validator().is_valid` on a hand-built record and `validate_document` is exercised under TC-1634 via `validate_document_in_registry`, not the CLI. Library-level, not CLI-level, evidence | TC-1599, FR-069-AC-1, tests/semantic_contract.rs:132, tests/semantic_surface.rs:324 |
| FND-470 | low | TC-1649 (Compile) binds no test symbol: evidence is the `check-wasm` Makefile target, `.github/workflows/ci.yml:62`, and string assertions inside TC-1642's test. `make check-wasm` passed in this session. The ✅ is a compile gate and the row should say "compile gate, no symbol" | TC-1649, NFR-021-AC-2, Makefile:333, tests/semantic_boundary.rs:79 |
| FND-471 | low | TC-1650 claims "introducing a generator-derived schema fails the audit"; the bound test asserts the audit script text names `semantic-v1.schema.json` and the schema lacks "schemars". The failing-injection half is not exercised. Static row | TC-1650, FR-072-CON-3, tests/semantic_surface.rs:517, scripts/audits/check_no_schemars.sh:31 |
| FND-472 | low | TC-1619 claims brace and pattern text round-trips opaque; the bound test is a source grep (`regex::`, `Regex::new`, no `sysml` crate). The round-trip is asserted under TC-1616 (`fence_lines`), untagged for TC-1619. Static row; say so | TC-1619, FR-070-CON-1, tests/semantic_boundary.rs:124 |
| FND-473 | low | TC-1629 `fence_bodies_round_trip` is flagged by the engine as vacuous-under-guard (7/7 assertions behind `if let Some(clauses)`). The else branch requires an error diagnostic, so a bypassing input is not unchecked, but the oracle weakens to "some error" for bodies that close the fence early | TC-1629, FR-071-AC-6, tests/semantic_clauses.rs:487 |
| FND-474 | low | Matrix declares TC-1636 external (🚧, agent-ix/quire-wasm#3) and nothing in this repo tags it — correct. Task-021 nonetheless declares `verifies TC-1636`, a claim the repo cannot discharge. FR-072-AC-6 is the one engine-unbacked AC in scope (FR-072 8/9) for this reason | TC-1636, FR-072-AC-6, Task-021 |
| FND-475 | low | Task deliverable drift: Task-016 lists `tests/fixtures/semantic/modules/` (absent; tests mutate `module-ok` copies in tempdirs); Task-018 lists `tests/props_semantic.rs` (absent; the proptest is inline in `tests/semantic_properties.rs`). Both tasks are `completed` | Task-016, Task-018 |
| FND-476 | low | Engine diagnostic: the `Functional Requirement Coverage` table (spec/tests.md:64) uses column `Coverage Status`, not the configured `Status`, so status classification is skipped for the FR-069..FR-072 summary rows. Pre-existing, whole-file | spec/tests.md:64, spec/tests.md:109 |
| FND-477 | low | The engine mints acceptance criteria only (41 targets in scope); FR-069-CON-1..4, FR-070-CON-1..3, FR-071-CON-1..2, FR-072-CON-1..3 are not minted, so their coverage here is grep-level: every CON has a `#[trace]` tag (TC-1608, 1606, 1607, 1609, 1619, 1620, 1618, 1627, 1628, 1632, 1640, 1650) | FR-069-CON-1, FR-072-CON-3, spec/tests.md |
| FND-478 | low | `UPDATE_SEMANTIC_BASELINES` in `tests/semantic_baseline.rs` and `tests/semantic_surface.rs` rewrites baselines and the compatibility fixture the plan calls immutable; the coordination rule ("a needed baseline change is a defect") has no owning requirement and no guard beyond a doc comment | tests/semantic_baseline.rs:26, tests/semantic_surface.rs:38, plan.md Coordination Rules |
| FND-479 | low | Engine `unmatched_tags`: TC id mentions in comments not bound as tags — TC-1632 on `no_block_record_is_unchanged`, TC-1089/TC-1090 on `filament_graph_cases_match_baseline`, `Task-018`/`Task-020` on two tests. Cosmetic; they read as cross-references, not claims | tests/semantic_properties.rs:570, tests/semantic_baseline.rs:228 |

## Coverage

- Reconciliation: `quire coverage --scope /home/peter/dev/quire-rs --json` (quire 0.31.0, engine 0.46.0@ca7362d4; module spec-artifacts-process). Split-root semantics apply (cli ≥ 0.16.0).
- Tasks done: 7 / 8 (Task-015..Task-021 `completed`; Task-022 `todo`).
- Rows backed by a tagged test, per minting document (engine `groups`):
  - FR-069: 11 / 11 acceptance criteria
  - FR-070: 10 / 10
  - FR-071: 7 / 7
  - FR-072: 8 / 9 (FR-072-AC-6 unbacked — TC-1636 external)
  - NFR-021: 4 / 4
  - Repository totals (all documents): 1173 / 1558.
- `status_lies` in range: none. `unbacked_rows` in range: TC-1636 (declared 🚧 external) and FR-072-AC-6 verification row. `no_symbol_rows` in range: none. `untracked_symbols` in range: none.
- TC → tag reconciliation (TC-1599..TC-1650, 52 rows):
  - Bound exactly once to a test asserting the row: TC-1599..TC-1609, TC-1611..TC-1617, TC-1620..TC-1627, TC-1631..TC-1634, TC-1637, TC-1638, TC-1640..TC-1642, TC-1645..TC-1648, TC-1650.
  - Bound twice: TC-1610, TC-1630 (FND-461, FND-462); TC-1635 (two Python tests), TC-1644 (Rust + Python halves) (FND-463).
  - Bound once, test asserts less than the row claims: TC-1618, TC-1619, TC-1628, TC-1639, TC-1643, TC-1650 (FND-464, FND-467, FND-468, FND-471, FND-472).
  - No test symbol: TC-1649 (compile gate — `make check-wasm`, FND-470); TC-1636 (external — agent-ix/quire-wasm#3, nothing here claims it).
  - Rows whose type is Static and whose bound test is a source/lockfile grep: TC-1608, TC-1619, TC-1620, TC-1627, TC-1640, TC-1641, TC-1642, TC-1650 — the matrix already labels them `Static`.
- Unmapped ACs: FR-072-AC-6 (engine). Every other AC of FR-069..FR-072 and NFR-021 carries a `#[trace]` tag naming it. Every CON carries a tag but is not minted (FND-477). US-019 is not minted by the model; its EX-1..4 row points at TC-1610/1612/1613/1600.
- Gates run in this session, all on the branch tree:
  - `cargo test --quiet`: 44 suites, all `test result: ok`, 0 failures.
  - `make lint` (`cargo clippy --locked --all-targets -- -D warnings`): exit 0.
  - `make ci-python`: 39 passed, 1 skipped (pre-existing benchmark skip), exit 0 — covers TC-1635 and the Python half of TC-1644.
  - `make check-wasm`: exit 0 — TC-1649.
- Untraced behaviors / stubs: 2 (parallel fence scanner `src/semantic/scan.rs`, FND-466; `UPDATE_SEMANTIC_BASELINES` re-mint path, FND-478). No stub bodies found under `src/semantic/`.
- Semantic review: skipped (not requested).
- Deliverables present: `src/semantic/{clauses,context,contract,decl,mod,properties,python_entry,resolver,scan,surface,vendored}.rs`; `schemas/vendored/{PROVENANCE.json,common.schema.json,module-manifest.schema.json,semantic-core/0.1.0/}`; `schemas/output/semantic-v1.schema.json`; `scripts/vendor-semantic-schemas.sh`; `scripts/audits/check_semantic_boundary.sh`; `tests/fixtures/semantic/{baseline/,quoin/,cases.json,cases.expected.json,config-version.bundle.json,semantic-v1.json}`; `tests/semantic_{baseline,boundary,clauses,contract,fixtures,properties,surface}.rs`; `tests/python/test_semantic.py`; Makefile `check-wasm` in `ci`; `.github/workflows/ci.yml` wasm32 target. Absent as named: `tests/fixtures/semantic/modules/`, `tests/props_semantic.rs`, locator.rs refactor (FND-466, FND-475).

## Dispositions (applied 2026-09-03, same branch)

| ID | Disposition |
| --- | --- |
| FND-460 | Fixed — Task-022 closed with this commit. |
| FND-461 | Fixed — provenance test untagged; TC-1610 bound once. |
| FND-462 | Fixed — attribution test untagged; TC-1630 bound once. |
| FND-463 | Accepted — Python and Rust halves of one row are one claim on two surfaces. |
| FND-464 | Fixed — TC-1639 now pins the three published schemas' bytes (`published-schemas.json` baseline); TC-1643 is the graph baseline. |
| FND-465 | Fixed — `make check-wasm` runs the semantic suites under the wasm feature. |
| FND-466 | Fixed (spec) — FR-071 states equivalence proven by agreement, not shared code. |
| FND-467 | Fixed (matrix) — TC-1628 cites `parser_golden.rs`. |
| FND-468 | Fixed (matrix) — TC-1618 row states what it asserts; byte identity is TC-1632. |
| FND-469 | Fixed (matrix) — TC-1599 names `validate_document_in_registry`. |
| FND-470 | Fixed (matrix) — TC-1649 is a Make gate with no symbol. |
| FND-471 | Accepted — the audit's failing half is a script property. |
| FND-472 | Accepted — TC-1619's round-trip lives in TC-1616. |
| FND-473 | Accepted — the else-branch asserts an error; not vacuous. |
| FND-474 | Fixed — Task-021 no longer claims TC-1636. |
| FND-475 | Fixed — deliverables named as built. |
| FND-476 | Accepted — pre-existing matrix column name. |
| FND-477 | Accepted — engine behavior. |
| FND-478 | Accepted — `UPDATE_SEMANTIC_BASELINES` is a minting switch documented in the test; Plan-003 coordination rules forbid re-minting. |
| FND-479 | Accepted — cosmetic. |
