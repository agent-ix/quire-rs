---
id: SR-20261103118150
title: "Code review — PLAT-1369 remove the semantic_core strip (quire-rs PR 636)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; CHANGELOG.md, src/semantic/contract.rs, tests/it/semantic_contract.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-1369. PR head `3e3bf39` (branch `plat-1369-semantic-core-strip`), one commit over `origin/main`. Code review with the Rust lane folded in.

Measured on the PR head:

- `src/semantic/contract.rs` `read_semantic_block`: the `block.clone()` plus `map.remove("semantic_core")` strip and its comment are deleted; the validator now runs `iter_errors(block)` on the block as authored. `git grep semantic_core` over `src/` finds no remaining reference to the manifest key. Every other `semantic_core` hit in `src/` and `tests/` is the unrelated embedded-bundle API (`embedded_semantic_core_version()`, `semantic_core_bundle()`, resolver locals). No rejection test, deprecation path or shim was added, per the owner's ruling.
- `tests/it/semantic_contract.rs` TC-1599: the three-module loop that injected `semantic_core` values (0.3.2, 0.3.3, 99.0.0) is replaced by one unmutated module-ok load with the same assertions. The test still asserts the full AC-1 behaviour (no failures, `SemanticModule` fields, resolved data validator accepts the golden set and rejects a bad record).
- `cargo test --test it semantic_contract` (via locked-build.sh, debug=0, worktree target): 26 passed, 0 failed. `rustfmt --check` on both Rust files: clean.
- `CHANGELOG.md`: one line under `## [Unreleased]` / `### Changed`, accurate.
- No unrelated edits in the diff (5 files, all on topic).
- Downstream: all ten live module manifests on their `main` (spec-artifacts-iso/process/app, spec-objects-business/enterprise/security/safety/architecture/operational, engineering-assurance) carry no `semantic_core` key, so removing the strip refuses no live module at its current `main`. Locally installed module copies under `~/.ix/filament/modules/` still carry it until reinstalled.

## Verdict

**APPROVE (mergeable).** The strip is gone with no replacement shim, the test backs AC-1 against the schema in its normal state, and the focused tests pass. FND-001 is a low cleanup nit and does not block merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | With the strip gone, the bare `{ ... }` block wrapping the `for error in validator.iter_errors(block)` loop scopes nothing (no binding is declared inside it). Drop the braces and dedent the loop. | src/semantic/contract.rs:165-205 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| `read_semantic_block` step 2 (contract.rs:162-208) | examined | Strip removed; FND-001 |
| `block_validator` | examined | No `semantic_core` handling left |
| TC-1599 `valid_block_and_reference_schema_load` | examined | Clean; passes |
| TC-1600 `unsupported_versions_are_refused_first` | context_only | Unchanged; passes |
| Remaining `semantic_core` identifiers in `src/`, `tests/` | examined | All embedded-bundle API, unrelated to the manifest key |
| CHANGELOG PLAT-1369 line | examined | Clean |
