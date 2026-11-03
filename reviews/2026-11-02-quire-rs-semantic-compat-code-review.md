---
id: SR-16290
title: "Code review of quire-rs semantic compatibility fix"
type: SpecReview
analysis: code-review
scope: ".github/workflows/ci.yml, src/semantic/contract.rs, tests/semantic_contract.rs, src/skeptic.rs, src/symbols/mod.rs, src/symbols/trace.rs, tests/fixtures/corpus_cases/coverage.json, spec/functional/FR-069-semantic-module-contract-at-load.md, spec/tests.md"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/FR-069"
    type: references
---

## Summary

Rust review examined semantic manifest loading, trace binding, source indexing, oracle comparison, FR-069, and the regression fixtures. A first hosted full-test run exposed a coverage attribution regression in the trace-binding optimization; the fix moves the stable sort out of each symbol bind and runs it once after binding. The current combined branch passes local formatting, Clippy, and the complete Cargo test suite; hosted CI is running on the combined fix.

## Verdict

**PASS** — no open code findings remain after the fixed trace-binding regression. Hosted CI is required before merge.

## Assurance Context

AP-201 at spec/assurance/AP-201-detection-minting.md was assessed because module admission affects the registry consumed by coverage and diagnostics. The change adapts semantic-block admission: the legacy informational key is excluded from validation, while the embedded schema remains authoritative for all other keys; export names are checked against all declared module types. It does not change coverage collection, trace binding, row minting, or published coverage schemas. `quoin advise --repo` ran for evidence context; it returned 1015 recommendations (122 mismatches, 52 inconclusive, 14 uncatalogued), which are repository-wide recommendations, not evidence of this change failing. The matrix has no bound run evidence. MP-201 through MP-208 and three measurement records exist, but those records are dated 2026-08-29 and measure the `filament-ide-rs` corpus; none records this change or the current source revision, so they are not comparable evidence for this fix. AP-201 has no artifact relationships and no active exceptions are declared. Scope boundary was checked against FR-069 and `Manifest::all_archetypes()`.
AP-201 at spec/assurance/AP-201-detection-minting.md applies because both semantic module admission and trace binding feed coverage and diagnostics. The semantic loader accepts the legacy informational key and checks exports against all declared types; the binder compiles declared patterns once, indexes source paths, and sorts the accumulated verification relations once after binding. It does not change coverage collection semantics. `quoin advise --repo` returned 1015 repository-wide recommendations (122 mismatches, 52 inconclusive, 14 uncatalogued); these are not evidence that this change fails. The computed matrix reports 1015 criteria, including 293 untagged, and no bound run evidence. MP-201 through MP-208 and three measurement records exist, but those records are dated 2026-08-29 and measure `filament-ide-rs`; they are not comparable evidence for these fixes. AP-201 has no artifact relationships and no active exceptions are declared. Scope boundaries were checked against FR-069, FR-051, and `Manifest::all_archetypes()`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
| FND-002 | high | The first trace-binding optimization used the appended `verifies` slice after `bind_symbol` sorted the entire accumulated relation list, reattributing tags and changing the coverage baseline (unmatched-tag rows 1 to 4). | src/symbols/trace.rs:685 |

## Dispositions

FND-002: fixed. `bind_symbol` now appends without sorting; `bind` sorts once after the symbol loop, preserving the append-slice boundary and deterministic final order. The coverage baseline and full local test suite pass. FND-001 is the no-findings placeholder and has no disposition.

## Rust Review

Rust review: regression checks covered 18 semantic-contract tests, the coverage baseline, 684 library tests, and 19 corpus-case tests. The first full hosted run caught the append-slice ordering bug; after moving sort to the end of `bind`, the coverage baseline passes and the full local Cargo test suite passes. `cargo fmt --all -- --check` and Clippy pass. On the current QSL checkout, `quoin matrix --repo ... --json` returned a 1.4 MB matrix in about 100 seconds with installed Quoin 0.29.2 and Quire 0.37.1; a patched-engine run is required to compare against that production baseline. The code adds no public API, unsafe code, or wire format. The implementation reuses the declared trace patterns and existing source index rather than adding a second grammar or data source.
