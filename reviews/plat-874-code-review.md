---
id: SR-16310
title: "Code review — PLAT-874 unextractable language refuses measurement (quire-rs PR 596)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; Cargo.toml, src/coverage.rs, src/symbol_table.rs, src/symbols/mod.rs, src/symbols/trace.rs, src/traceability.rs, tests/it/coverage_rollup.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
---

## Summary

Ticket: PLAT-874. Frozen PR head: `5b129915de90eb833c198b86d87ce9288243c1cc`. Code review with the Rust lane folded in. The change replaces the per-language "BUILD CONFIGURATION" per-file diagnostic with a first-class `SymbolExtraction.unavailable_languages` set (via `SourceLanguage::is_compiled_in`), copies it onto `SymbolGraph` and the symbol-table report, and makes `coverage::compute_inner` return `CoverageError::ExtractionUnavailable`. Both `compute` and `compute_with_evidence` route through `compute_inner`, so coverage refuses on every coverage entry point. No vendoring or duplication was introduced.

The coder widened the change from Rust to Python and TypeScript. That is sound: all three adapters had the identical per-file fallback, and leaving two of them would keep the silent-zero for Python/TypeScript trees in the `wasm` build. Default builds enable all three features, so `is_compiled_in` is true for every language and the default path is unchanged. The full `tests/it` suite (528) and the `symbols::` lib tests (137) pass under default features. The `pyproject.toml` wheel build uses default features plus `python`, so it is also unchanged. Only non-default builds (`wasm`, and the CI `--no-default-features --features python-symbols` / `typescript-symbols` check legs) change behaviour, and there the refusal is the intended outcome.

## Verdict

**FAIL**. Coverage refuses correctly, but the other consumers of `SymbolExtraction`/`SymbolGraph` still return a successful empty answer for a tree the build cannot read. That is the same silent-zero failure the ticket exists to remove. Trace search reports `resolved: false`, the assurance export emits a schema-valid record with no symbols, and FR-045 graph records emit no nodes. None of them carries or checks `unavailable_languages`. The symbol-table report carries the field (AC-29 asks only that it be recorded), so it is not a finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `trace_search::search` never consults `graph.unavailable_languages`. On a no-adapter build an id query whose evidence lives in Rust returns `resolved: false` with empty `verifies`, which reads as "nothing backs this id". `SearchResult` has no field to say the build could not read the language, so the trace surface still reports a silent zero. | src/symbols/trace_search.rs:139-150 |
| FND-002 | high | `build_assurance_export` ("fail-closed" per its own error doc) projects `input.symbols`/`input.symbol_graph` without checking `unavailable_languages`, so a no-adapter build exports a schema-valid `quire-assurance` v1 record with zero source symbols and relations for a tree holding Rust/Python/TypeScript source. It should return an `AssuranceError` variant naming the languages. | src/assurance.rs:262-276 |
| FND-003 | medium | `trace::graph_records` emits FR-045 node/edge records from `extraction.symbols`/`graph.verifies` only. With an unavailable language it emits an empty set with no record or error saying so, and downstream graph consumers see a tree with no symbols. | src/symbols/trace.rs:2125-2131 |
| FND-004 | low | Capability has two sources of truth: `is_compiled_in` (cfg! per feature) gates the early return, and the `#[cfg(not(feature = ...))]` arms are kept as dead `Ok(Vec::new())`. If the two ever drift (a new language, or a renamed feature), the dead arm becomes a live silent zero, which is exactly the bug. Recording unavailability inside the `cfg(not)` arms would leave one source of truth. Otherwise make the dead arms `unreachable!` with a reason. | src/symbols/mod.rs:452-466 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| `SourceLanguage::is_compiled_in` (src/traceability.rs) | examined | Correct mapping per feature |
| `SymbolExtraction::extend_with_file` unavailable arm | examined | Records once per language, sorted; no symbols, no diagnostic, file not in `files`. FND-004 on structure |
| `SymbolGraph.unavailable_languages` copy in `trace::bind` | examined | Copied |
| `coverage::compute_inner` refusal / `CoverageError::ExtractionUnavailable` | examined | Refuses before model lookup on both `compute` and `compute_with_evidence` |
| `symbol_table::build` new field | examined | Recorded, `skip_serializing_if` empty, so default JSON is unchanged |
| `trace_search::search`, `assurance::build_assurance_export`, `trace::graph_records` | examined | FND-001..003 |
| `skeptic` suspicions | context_only | Folded into `SymbolGraph.suspicions`, so they are covered by the graph consumers |
| Python/TypeScript widening | examined | Sound; no default-build behaviour change |

## Verification

Default features: `cargo clippy --locked --all-targets -- -D warnings` clean, `cargo fmt --check` clean, `cargo test --locked --lib symbols::` 137 passed, `--lib symbol_table` 4 passed, `cargo test --locked --test it` 528 passed. No-adapter configuration (`--no-default-features --features resolve-file`): `cargo clippy --all-targets -D warnings` clean, `missing_adapter_is_unavailable_not_a_diagnostic` passes. 49 other `symbols::` lib tests fail in that configuration because they assert Rust symbols; that is expected without an adapter and is not introduced by this PR. `cargo check --target wasm32-unknown-unknown --no-default-features --features wasm` succeeds. Aggregate gate not run.
