---
id: SR-16326
title: "Code review — PLAT-877 report cfg-gated duplicate symbol ids (quire-rs PR 607)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/symbols/mod.rs (extend_with_file, id_collision_diagnostics, tc2082 tests); consumers: src/symbol_table.rs, src/coverage.rs, examples/mutants_scope.rs, quire-cli src/commands/{coverage,symbols,assurance,extract}.rs (read only)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
---

## Summary

Ticket: PLAT-877. PR head: `1e404e9`. Code review with the rust-review lane folded in. The change adds `id_collision_diagnostics`. For each file, `extend_with_file` groups the file's symbols by id in a `BTreeMap` and emits one `symbol-id-collision` `SymbolDiagnostic` for every group with two or more members. Each group is sorted by `(line, column)`, and the output is sorted by reason. Ids are unchanged. The lead's ruling (keep identity, report the collision) is implemented as written.

## Verdict

**PASS with low findings.**

- **One diagnostic per group per file.** This holds. The grouping is per call and `extend_with_file` runs once per file.
- **Deterministic.** Ordering and message are fully determined: BTreeMap key order, then a `(line, column)` sort inside each group, then a reason sort.
- **No cross-file collisions.** None can occur, because `compute_id` hashes `(language, path, qualified_name, kind)`.
- **Performance.** The cost is O(n log n) per file with no clones. On ecaz (15,937 symbols, 121 collisions) a whole-tree extraction took about 4 s, and that time is dominated by parsing.
- **Consumers.** Extraction diagnostics reach only `SymbolTableReport.diagnostics` and quire-cli's stderr in `coverage`, `symbols`, `assurance` and `extract`. Exit codes are unchanged. `CoverageReport.diagnostics`, the coverage baseline fixture, quire-bench and the Python and wasm bindings do not carry them. No fixture, baseline or test expectation changes.
- **Self-extraction.** `src/` now emits 2 new diagnostics, both from `From<A>`/`From<B>` impl pairs in `evidence.rs`. `tests/` emits 0, and quire-cli `src/` emits 1 (the cfg-gated `libc_eloop`).
- **Checks run.** 142 `symbols::` lib tests pass. 55 `it` tests pass across implements_relation, coverage_matrix, coverage_baseline, trace_dogfood, plat_validation_batch and symbol. clippy with `-D warnings` is clean under `--all-features` and under `--no-default-features`.
- **Ceremony.** None found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The message embeds each attribute's raw source text verbatim, joined by spaces. A rustfmt-wrapped multi-line attribute (`#[cfg(all(\n ...))]`) therefore puts newlines into `reason`, which breaks quire-cli's one-line-per-diagnostic stderr. Every non-cfg attribute (`#[inline]`, `#[trace(...)]`, `#[test]`) is also listed. None occurred on ecaz (0 of 121), but nothing prevents it. Collapse whitespace in the attribute text. | src/symbols/mod.rs:588-597 |
| FND-002 | low | There is no CHANGELOG `[Unreleased]` entry for a new diagnostic class that now appears in `quire symbols` JSON `diagnostics` and on stderr for `coverage`, `symbols` and `assurance`, including 2 new lines on quire-rs's own tree. | CHANGELOG.md:8 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/symbols/mod.rs extend_with_file | examined | Correct; symbols are moved into the collision pass with no clone |
| src/symbols/mod.rs id_collision_diagnostics | examined | Correct and deterministic; FND-001 |
| src/symbols/mod.rs tc2082 tests (3) | examined | Pass; see the gap-analysis SR |
| src/symbol_table.rs diagnostics passthrough | context_only | Carries the new reason unchanged |
| src/coverage.rs | context_only | Does not read extraction diagnostics |
| quire-cli coverage/symbols/assurance/extract | context_only | stderr only through `emit_diagnostic`; exit unchanged |

## Verification

These ran on a `git archive` copy of `1e404e9`, with `corpus` symlinked from a checkout at the pinned submodule commit, and used a scratchpad target that was deleted afterwards: `cargo test --locked --lib symbols::` (142 ok); `cargo test --locked --test it -- implements_relation coverage_matrix coverage_baseline trace_dogfood plat_validation_batch symbol` (55 ok); `cargo clippy --lib --tests --all-features -D warnings` and `cargo clippy --lib --no-default-features -D warnings` (both clean). A scratch probe ran `extract_tree` over quire-rs `src`/`tests`, quire-cli `src`, ecaz, auth-fastapi and chat-window to count collisions and check for multi-line reasons.
