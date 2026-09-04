---
id: SR-076
title: "Code review — semantic extraction boundary (#388, Plan-003)"
type: SpecReview
analysis: code-review
scope: "src/semantic/, src/loader/, src/filament.rs, src/validate_document.rs, src/python/mod.rs, schemas/, tests/"
review_set: subset
---

## Summary

Reviewed branch `spec/388-semantic-extraction-boundary` against `origin/main`
(8 commits, 126 files, +15,697/−41) under the `code-review` skill's Rust lane:
every gate was run rather than assumed, every AC of FR-069..FR-072 and NFR-021
was traced to its `#[trace]`-tagged test, and the cell grammars, fence
scanner, resolver, and cross-module checks were probed with inputs beyond the
fixture set (through the freshly built 0.46.0 wheel). All gates pass. No high
finding; seven mediums, of which the first four are behavioral defects with a
reproducing input and the other three are test-vs-AC shortfalls.

## Verdict

**CONDITIONAL** — no `high`; seven `medium` findings (four code defects with a
concrete failure scenario, three spec-test alignment gaps) and eleven `low`.

## Gate results (run on the branch, 2026-09-03)

| Gate | Result |
| --- | --- |
| `make fmt-check` | exit 0 (only the usual nightly-option warnings from `rustfmt.toml`) |
| `make lint` (`clippy --locked --all-targets -- -D warnings`) | exit 0, no warnings |
| `cargo test --quiet` | exit 0 — 592 unit + every integration suite green, 0 failed, 0 ignored |
| `make deny` | exit 0, `licenses ok` (three unmatched-allowance warnings, pre-existing) |
| `make audit-static` | exit 0 — all 11 scripts OK incl. `check_semantic_boundary`, `check_no_schemars`, `check_hashmap_audit`; `check_status_agreement` 52 advisories (pre-existing, plus FR-072 citing external TC-1636) |
| `make check-wasm` | exit 0 |
| `make ci-python` (wheel + `pytest tests/python/`) | exit 0 — 39 passed, 1 skipped (TC-456 perf) |
| `cargo test --no-default-features --features wasm --test semantic_{contract,properties,clauses,surface}` (not a gate; run for FR-069-AC-5) | 37 passed |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-440 | medium | Fence bodies are read on the clause/operation path, violating FR-071-CON-1 ("only span recording and byte copy"): the `Clause:` scan and the `Returns:`/`Pre:`/`Post:` scan iterate every line of the `###` section including fence interiors. Repro: `### inv` + an `ocl` fence whose first line is `Clause: foo` → `semantic.duplicate-clause-authority` error, clause kind `unavailable`, `clauseText` dropped; an Operations fence containing `Post: zz` → `semantic.dangling-clause-ref`. Filter these scans through the same `inside_fence` predicate `level3_headings` already uses. | src/semantic/clauses.rs:216, src/semantic/clauses.rs:221, src/semantic/clauses.rs:423, src/semantic/clauses.rs:425 |
| FND-441 | medium | Reader-rule loci are misattributed: `reader_rules` maps field index `i` onto `rows_lines(block)`, but for the fence form that vector holds every body line (blanks and errored lines included). Repro: `sysml` fence `attribute a`, blank, `attribute b`, blank, `attribute a` → `DUPLICATE_NAME` reported at line 12 (the blank) and `row-errors: lines 12`; the offending row is line 14. Carry the source line on `RowInput`/`FieldDecl` mapping instead of positional alignment. | src/semantic/properties.rs:793, src/semantic/properties.rs:796 |
| FND-442 | medium | The Filament surface mints a second schema digest over the re-serialized JSON (`json_string(&schema)`), so `schemaDigest` for the same `Entity.json` differs between surfaces (Filament `sha256:3be5fc…`, loader/`validate_document` `sha256:869299…` = manifest digest). FR-069 Outputs: "no second digest is computed"; NFR-021-AC-4/FR-072 require the surfaces to agree as JSON values. Either carry the registry digest on the snapshot or omit `schemaDigest` on this surface. | src/filament.rs:1180, src/filament.rs:1181, spec/functional/FR-069-semantic-module-contract-at-load.md:60 |
| FND-443 | medium | `extract_semantic_json` (Python `extract_semantic`, future WASM `extractSemantic`) applies no contract gate: `semanticCore: "9.9.9"` returns a record claiming `semanticCore: 9.9.9` with `fields` `available` and zero diagnostics, because `field_decl_validator`/`model_validator` swallow the missing bundle with `.ok()`/`let Some … else return` and silently skip the `internal-invalid-decl` gate. FR-072 refuses unsupported versions on the snapshot surface; the library surface should refuse (or at least diagnose) the same way rather than fall open. | src/semantic/python_entry.rs:56, src/semantic/properties.rs:857, src/semantic/properties.rs:869, src/semantic/clauses.rs:569 |
| FND-444 | medium | User-input shapes reach the "engine defect" gate: `min: NaN`, `min: inf` (`number_or_string` → JSON `null`), and `Decimal(10,2) []` (operator-precedence bug in the unit condition: `A && B && C \|\| D` lets an empty unit through for a parenthesised head) all surface as `semantic.internal-invalid-decl` at `line: 0` with the message "engine defect"; `enumValues:` with an empty value is accepted as `values: [""]`. Each should be a row-level error at the row (or a refusal in the cell grammar), and `line: 0` is not a locus (`semantic-v1.schema.json` had to allow `minimum: 0` to admit it). | src/semantic/properties.rs:402, src/semantic/properties.rs:704, src/semantic/properties.rs:769, src/semantic/properties.rs:849, schemas/output/semantic-v1.schema.json:53 |
| FND-445 | medium | TC-1631 is weaker than FR-072-AC-2 ("`available`, `not_applicable`, `missing`, and `unavailable` for each declaration kind"): the per-kind loop asserts only `available`/`not_applicable`; `missing`/`unavailable` are checked over the union. In `cases.expected.json` `clauses` never reaches `missing` and `operations` never reaches `missing` or `unavailable`, so the `RequiredSections` path for Invariants/Operations and the operations `entry-errors` path have no fixture. | tests/semantic_surface.rs:178, tests/semantic_surface.rs:198, tests/fixtures/semantic/cases.json:1 |
| FND-446 | medium | FR-072-AC-6/TC-1635 claim `extract_filament_core` parity "for every fixture case"; `test_semantic.py` compares `extract_semantic` case-by-case against the Rust oracle (good) but the Filament half is one hand-built input with four spot assertions, never a JSON-equality against a Rust-produced expectation. `RequiredSections::from_dsl` (Filament/`validate_document` path) is likewise never driven to `missing`. | tests/python/test_semantic.py:43, src/semantic/surface.rs:60 |
| FND-447 | low | FR-069-AC-5/TC-1603 state the `$ref` cases "pass under `--no-default-features --features wasm`", but no gate runs tests under that feature (`make check-wasm` is `cargo check` only; TC-1649). Run manually for this review: 37/37 pass. Add a `test-wasm-feature` lane or drop the clause from the AC. | Makefile:334, spec/tests.md:788 |
| FND-448 | low | `validate_document` surface: the `semantic.record-invalid` branch has no test (only the Filament twin is covered by TC-1605); the context is built with path `<document>`, no scope and no source identity, so every validated document with an Invariants fence gets a `semantic.source-identity-defaulted` warning the author cannot silence, with spans reading `ix://local/scope/spec`. | src/validate_document.rs:360, src/validate_document.rs:399 |
| FND-449 | low | Validators are recompiled per document: `field_decl_validator` and two `model_validator` calls each re-walk and clone the embedded bundle into a fresh `JSONSchema` on every extraction (3 compiles per document); `block_validator()`/`target_registry()` re-parse the manifest schema per module. Fine for tests, measurable on a corpus walk (cf. CR-072 lesson). Cache behind `OnceLock` with the sanctioned exemption note. | src/semantic/properties.rs:857, src/semantic/clauses.rs:569, src/semantic/contract.rs:131, src/semantic/contract.rs:143 |
| FND-450 | low | Unresolved-type `reason` is approximate: `semantic_findings` inserts every loaded module's own package into `imports`, so the doc-comment's "resolve as `no-bundle-index`" is wrong (reason becomes `unknown-token`, own exports resolve as imports); and any typo resolves as `import-unresolved` whenever *any* module import is unprovided, regardless of which import the token would belong to. | src/validate_document.rs:345, src/semantic/properties.rs:579 |
| FND-451 | low | On the Filament snapshot path `legacy_forms` is hard-wired to `warning` and `compatibility_posture` to `additive`, so a module that sets `legacy_forms: error` is demoted to a warning on that surface. Spec-consistent (FR-069 Inputs list only five snapshot keys) but a surface divergence worth a CR note or two more snapshot keys. | src/filament.rs:1165, spec/functional/FR-069-semantic-module-contract-at-load.md:53 |
| FND-452 | low | FR-070 Behavior names both `semantic.invalid-multiplicity` ("a flag on a single value") and `agent-ix.semantic-core.FLAGS_ON_NON_COLLECTION` for the same input; the code emits only the former and `FLAGS_ON_NON_COLLECTION` is never produced. Spec-internal contradiction to resolve by CR. | src/semantic/properties.rs:662, spec/functional/FR-070-typed-properties-extraction.md:120 |
| FND-453 | low | Under an operation heading a second `Returns:`/`Pre:`/`Post:` line silently replaces the first (last-wins, verified: two `Returns:` lines → `Integer` with no diagnostic) and a second param table is appended without diagnosis. Emit `semantic.duplicate-operation-clause` (or similar) at the second occurrence. | src/semantic/clauses.rs:419, src/semantic/clauses.rs:424 |
| FND-454 | low | One erroring invariant cascades: `extract_semantic` passes an empty slice when clauses are `unavailable`, so every `Pre:`/`Post:` in Operations then fails `semantic.dangling-clause-ref` and the operations kind goes `unavailable` too (verified: one `tla` fence → two errors, both kinds down). Resolve refs against the parsed ids, not the surviving entries. | src/semantic/surface.rs:104, src/semantic/clauses.rs:521 |
| FND-455 | low | Hygiene: `read_module_semantic` takes `module`/`source` only to discard them (`let _ = (module, source)`); the `module_version` doc comment now sits on `semantic_module`; `ValidationReason` (public, not `#[non_exhaustive]`) gains a variant, a source-breaking change for exhaustive matches; `semantic.data-schema-reference-without-block` is emitted but appears in no FR. | src/loader/mod.rs:731, src/registry.rs:379, src/validate_document.rs:33, src/loader/mod.rs:790 |
| FND-456 | low | TC-1628 (FR-071-CON-2) asserts the parser-golden half only as "the section still parses"; byte-identity of `parse_document` is delegated to `tests/parser_golden.rs` without citing that row. Cite it in the trace or assert the golden inline. | tests/semantic_boundary.rs:200, tests/semantic_boundary.rs:243 |
| FND-457 | low | A `## Properties` section holding only prose, or a `sysml` fence indented by 1–3 spaces (CommonMark-legal, not recognised at column 1), yields `available`, `fields: []`, `fieldsForm: table` with no diagnostic — an unreadable section reads as an empty typed table. Prefer `unavailable` (`no-declaration-block`) or an advisory. | src/semantic/properties.rs:124, src/semantic/scan.rs:31 |

## Spec-code faithfulness (per AC)

Every AC id in FR-069-AC-1..11, FR-070-AC-1..10, FR-071-AC-1..7,
FR-072-AC-1..9 and NFR-021-AC-1..4, plus every CON, resolves to a
`#[trace("TC-…", "FR-…-AC-n")]` test whose body asserts the named behavior
(codes, loci, states, values), not existence. Exceptions recorded above:
FR-072-AC-2 (FND-445), FR-072-AC-6 Filament half (FND-446), FR-069-AC-5 wasm
clause (FND-447), FR-071-CON-2 golden half (FND-456). TC-1636 is external
(`agent-ix/quire-wasm#3`) and TC-1649 is a compile gate, as the matrix says.
Every TC-1599..TC-1650 id except TC-1636/TC-1649 appears in a test tag.

Checked and clean: refusal order in `read_semantic_block` matches FR-069;
escape (`..`, absolute, symlink via `canonicalize().starts_with`) and the
inline `..` key; `$ref` cycle/version/unshipped/self-fragment; digest over
shipped bytes at load (TC-1609); cross-module duplicate/unresolved/cycle in
sorted-root order; `BTreeMap` everywhere order is observable (`check_hashmap_audit`
OK); no `HashMap`, no clock, no env, no network on the semantic path; the only
filesystem read is `read_module_file` on the loader path; the `expect`s are
on vendored/compile-time data only, and every slice index is guarded.
`schemas/vendored/**` hash to `PROVENANCE.json` and the 0.1.0 bundle digest
equals the pinned `toolchain.json` constant (TC-1606). Baselines reproduce
byte-for-byte (TC-1607, TC-1632, TC-1643).

## Edge cases probed beyond the fixtures

`Clause:`/`Post:` inside fence bodies (FND-440); fence with blank lines and a
duplicate (FND-441); unsupported `semanticCore` via the binding (FND-443);
`min: NaN`, `min: inf`, `Decimal(10,2) []`, `enumValues:`, `maxLength: -1`,
`minLength: 1.5` (FND-444 — the last two are correctly refused at the row);
two `Returns:` lines (FND-453); one bad invariant with a `Pre:` (FND-454);
legacy-then-typed (both-forms, correct); prose-only and indented fence
(FND-457); typo token with an unprovided import (FND-450); pipe-escaped
`enumValues`, `pattern: /a,b|c/` (correct, verbatim).

## Dispositions (applied 2026-09-03, same branch)

| ID | Disposition |
| --- | --- |
| FND-440 | Fixed — `Clause:`/`Returns:`/`Pre:`/`Post:` read outside fences only (`lines_outside_fences`); case `fence-interior-is-opaque`. |
| FND-441 | Fixed — reader rules receive the source line of each produced field. |
| FND-442 | Fixed — Filament passes the snapshot's `schemaDigest` through; no second digest (TC-1632 asserts equality with the library record). |
| FND-443 | Fixed — `extract_semantic_json` refuses unsupported versions; schema gates no longer fall open. |
| FND-444 | Fixed — `semantic.invalid-constraint-value` for empty/non-finite values; empty unit brackets refused; unit precedence bug fixed. |
| FND-445 | Fixed — cases for clauses/operations `missing` and operations `unavailable`; TC-1631 asserts all four states per kind. |
| FND-446 | Fixed — Python test compares the Filament payload with the Rust oracle record; a `missing` case exists. |
| FND-447 | Fixed — `make check-wasm` also runs the four semantic suites under `--no-default-features --features wasm`. |
| FND-448 | Fixed (test) — `semantic.record-invalid` branch tested; the defaulted-identity warning on this surface is per FR-071 and stays. |
| FND-449 | Accepted — validator caching is a follow-up; no behavior impact. |
| FND-450 | Accepted — reason heuristic documented; `no-bundle-index` and `unknown-token` exact, `import-unresolved` approximate. |
| FND-451 | Accepted — the snapshot producer (filament-core-service#23) may carry `legacyForms`; today's default is documented. |
| FND-452 | Fixed (spec) — FR-070 says flags on a non-collection are `semantic.invalid-multiplicity`. |
| FND-453 | Fixed — `semantic.duplicate-operation-line` for a second line or table. |
| FND-454 | Accepted — cascaded dangling refs follow from the unavailable clause kind. |
| FND-455 | Fixed — discarded params removed; code documented in FR-069. |
| FND-456 | Fixed (matrix) — TC-1628 row names `parser_golden.rs`. |
| FND-457 | Fixed — prose-only Properties is `unavailable` (`no-block`) with a warning; fences may be indented up to three spaces. |
