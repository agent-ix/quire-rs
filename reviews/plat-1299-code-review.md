---
id: SR-20261103118130
title: "Code review — PLAT-1299 semantic refusals carry their manifest location (quire-rs PR 606)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/semantic/contract.rs, tests/it/semantic_contract.rs, spec/functional/FR-069-semantic-module-contract-at-load.md, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-1299. Frozen PR head: `baa7cb0`. Code review with the Rust lane folded in. `SemanticFailure::reason()` now formats `<code>: <path>: <message>`, and falls back to `<code>: <message>` when `path` is empty. `reason()` is the only formatter for `SemanticFailure`, so the new form applies to every refusal kind that goes through it, not only the five that TC-2081 tests. The covered paths are the contract.rs block checks (invalid-value, unsupported-contract-version, unknown-key, unknown-target, invalid-package, missing-key, export-undeclared), the resolver refusals, the loader's ambiguous and reference-without-block refusals, and duplicate-package and import-cycle. The resolver's `$ref` refusals (`schema-ref-cycle`, `schema-ref-unshipped`, sibling `data-schema-not-json`, compile `data-schema-not-schema`) are built with an empty path. `resolve_reference` overwrites that path with `<section>[<type>].data_schema.schema` (resolver.rs:148-151), so no current load refusal reaches the empty-path branch.

I checked the consumers. The other `reason()` methods (model.rs:891, relations.rs:468) are separate functions. The only things that read `ArchetypeLoadFailure.reason` are tests using `starts_with(code)` or `contains(...)`, and the assurance error passthrough (src/assurance.rs:349). The assurance passthrough is a free string, and `schemas/output/assurance-v1` only constrains `minLength: 1`. The repo has no `python/` tree. The wasm bindings live in an external crate. `src/semantic/python_entry.rs` and `src/filament.rs` format their own strings and are unaffected. No schema under `schemas/output` changed. Tests run: `cargo test --test it semantic` passed 120 and `cargo test --lib` passed 720.

## Verdict

**PASS** with low findings. The change is correct, minimal and idiomatic. The Rust lane is clean: there is no new allocation pattern of concern, no panics, and no change to the public API shape.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The `semantic.duplicate-authority` refusal is a hand-built `format!` string that bypasses `SemanticFailure::reason()`. It is the one load-time `semantic.*` refusal that never takes the `<code>: <path>: <message>` form, even though it has a manifest locus (the type's `yield_pattern` locator). | src/loader/mod.rs:945-948 |
| FND-002 | low | Several messages already restate the path. The reason now reads, for example, `semantic.export-undeclared: semantic.exports.ghost: semantic.exports names ghost, …` and `semantic.unsupported-contract-version: semantic.contract_version: semantic.contract_version "x" is not …`. The locus is printed twice. | src/semantic/contract.rs:157-164, 240-243 |
| FND-003 | low | The CHANGELOG `[Unreleased]` section has no entry for the changed `ArchetypeLoadFailure.reason` text, which is user-visible CLI/library output. | CHANGELOG.md:8 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| `SemanticFailure::reason` (src/semantic/contract.rs:56-64) | examined | Correct; empty-path fallback present |
| Every `SemanticFailure::error` producer feeding `reason()` (contract.rs, resolver.rs, loader/mod.rs:975-998, 1716, 1829) | examined | All carry a non-empty path at the point `reason()` is called |
| Consumers of `ArchetypeLoadFailure.reason` (src/assurance.rs, src/registry.rs tests, tests/it/*) | examined | No parser of the old `<code>: <message>` shape |
| `schemas/output/*` | examined | Unchanged |
| `refusals_carry_their_manifest_path` test | examined | Asserts locus prefix per case; passes |
