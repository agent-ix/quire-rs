---
id: SR-16290
title: "Code review of quire-rs semantic compatibility fix"
type: SpecReview
analysis: code-review
scope: ".github/workflows/ci.yml, src/semantic/contract.rs, tests/semantic_contract.rs, spec/functional/FR-069-semantic-module-contract-at-load.md, spec/tests.md"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/FR-069"
    type: references
---

## Summary

Rust review examined the loader boundary, schema adaptation, exported-type validation, FR-069, and the focused tests. No open code findings remain; local full CI is blocked by the missing wasm32 target.

## Verdict

**PASS** — no open code findings in the reviewed change; targeted regressions pass. Hosted CI remains required before merge.

## Assurance Context

AP-201 at spec/assurance/AP-201-detection-minting.md was assessed because module admission affects the registry consumed by coverage and diagnostics. The change adapts semantic-block admission: the legacy informational key is excluded from validation, while the embedded schema remains authoritative for all other keys; export names are checked against all declared module types. It does not change coverage collection, trace binding, row minting, or published coverage schemas. `quoin advise --repo` ran for evidence context; it returned 1015 recommendations (122 mismatches, 52 inconclusive, 14 uncatalogued), which are repository-wide recommendations, not evidence of this change failing. The matrix has no bound run evidence. MP-201 through MP-208 and three measurement records exist, but those records are dated 2026-08-29 and measure the `filament-ide-rs` corpus; none records this change or the current source revision, so they are not comparable evidence for this fix. AP-201 has no artifact relationships and no active exceptions are declared. Scope boundary was checked against FR-069 and `Manifest::all_archetypes()`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Rust Review

Rust review: `block_validator` adjusts the embedded schema’s required list only to preserve FR-069’s no-version requirement, then validates a view with the informational legacy field removed; it does not duplicate an upstream schema property. Export validation receives all declared module names. The code adds no public API, unsafe code, concurrency, or persistent wire format. The Perf Bench and License Check hosted jobs were missing the required corpus submodule checkout; both now initialize the same submodule used by Rust Checks. Validation also exposed five withdrawn matrix rows with placeholder values forbidden by the current schema; those rows were removed from the active test table and their IDs remain listed as reserved. With current local schema providers, `make validate` now reports 176 documents, 0 failures, 43 warnings, and 9 Phase-7 exclusions. `cargo fmt --check` and Clippy pass. Focused semantic-contract tests pass, including legacy versions and an artifact type export. The local fixture package warns that npm is unavailable; its two network-backed tests skip by repository policy.
