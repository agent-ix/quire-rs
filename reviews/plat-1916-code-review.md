---
id: SR-967175026302197768900474181941935084871
title: "Code review — PLAT-1916 semantic type index"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/corpus/validate.rs, src/lib.rs, src/semantic/properties.rs, src/validate_document.rs, tests/fixtures/corpus_cases/plat-1916-semantic-type-index.json, tests/it/semantic_surface.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-070", type: reviews }
---

## Summary

Reviewed PR #735 at head `58ef3db4b3a9d102bdbf6634e29c9f9b8d92b5a7` against PLAT-1916 and FR-070. The corpus index resolves the synthetic sibling to its stable artifact ID and keeps single-document validation unindexed, but the change leaves existing golden expectations stale, adds Clippy failures, and misbinds the new test's trace tag.

## Verdict

**FAIL** — the touched crate's full suite has eight semantic expectation failures caused by the new canonical IDs; strict all-target Clippy also fails on five needless borrows in the added test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Canonical artifact IDs change semantic outputs, but existing expected values were not updated; the full `quire-rs` suite fails eight semantic tests, including golden field, clause, model, and surface cases. | src/semantic/properties.rs:782; tests/it/semantic_properties.rs:269 |
| FND-002 | medium | The added integration test does not pass strict Clippy: five needless borrows fail `cargo clippy --workspace --all-targets --all-features -- -D warnings`. | tests/it/semantic_surface.rs:553 |
| FND-003 | medium | The new sibling-resolution assertions are bound to FR-070-AC-1, which specifies a different golden fixture; the applicable type-resolution criterion is FR-070-AC-4. | tests/it/semantic_surface.rs:540 |

## Rust review checklist

- New public API `validate_document_in_indexed_bundle` has a documented explicit-index contract and exercises it through the public API. The corpus path builds an index from each member view, so another member's declarations do not leak in.
- Sibling object and enumeration names now lower to the matched declaration ID; the synthetic fixture checks a positive ID resolution, absent target, wrong-kind quantity target, and the single-document unresolved path.
- No unsafe code, async work, locks, unbounded loops, or new wire representation was added. Changed production functions carry `#[implements("FR-070-AC-4")]`; the integration test has trace tags, with the mismatch described in FND-003.
- CLI compatibility did not need a change: PR description records that origin/main routes `--okf --repo` through `validate_corpus` with member identities.

## Gate results

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | pass |
| focused PLAT-1916 integration test | pass; 1 passed |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | fail; needless borrows at tests/it/semantic_surface.rs:553, 583, 614, 633, and 645 |
| `cargo test -p quire-rs` | fail; 983 passed, 20 failed. Eight failures are stale semantic expectations from canonical ID output; twelve corpus-case failures require unavailable sibling module checkouts (`VALIDATION_PROCESS_ROOT` / `VALIDATION_ISO_ROOT`). |
| `cargo deny check` | not reached; chained after the failing full test command |
| `quire validate --scope . "spec/**/*.md"` | existing repository validation exits 3 on grammar/frontmatter findings in untouched spec files |

## Findings detail

FND-001 follows from the specified stable identity change: outputs formerly used authored names such as `ConfigVersion` or `ConfigOverlay`, while the new resolver correctly emits the matched artifact IDs such as `FR-006` or `FR-005`. Existing assertions and checked-in semantic goldens still expect the old values. The outside-diff failures were observed in `cargo test -p quire-rs`, including `tests/it/semantic_properties.rs::type_cells` at line 269 and `tests/it/semantic_surface.rs::case_suite` at line 51.

FND-002 is directly reproducible with the repository's strict Clippy gate. The first diagnostic is `parse_document(&source)` at line 553; Clippy reports the same lint for four further `&` borrows of `&str` values in this new test.

FND-003: computed `quoin matrix --repo . --json` binds `indexed_validation_resolves_same_repo_types_and_preserves_single_document_mode` to FR-070-AC-1 and FR-070-AC-11. The AC-11 binding matches the Clock precedence assertion, but AC-1 is the config-version golden output requirement; the new same-repository resolution assertion is covered by FR-070-AC-4's Type resolution obligation and should carry that tag.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The fix updated the ConfigVersion/ConfigOverlay fixtures, but two affected assertions still expect title identities: `semantic_model::population_members_with_one_extent` expects `Order` while output is `FR-100`, and `semantic_systems::systems_tables_lower_to_record_keys_and_refuse_bad_rows` expects `QuantCodec` while output is `quant_codec`. Both exact tests still fail on the stable-ID change. |
| FND-002 | fixed | after_excerpt: `let source_fm = quire_rs::parse_document(source).frontmatter.unwrap();` Strict all-target/all-feature Clippy passes. |
| FND-003 | fixed | after_excerpt: `#[trace("FR-070-AC-4", "FR-070-AC-11")]` |
| FND-001 | fixed | after_excerpt: `json!({ "kind": "SemanticId", "value": "ix://agent-ix/shop/type/FR-100" })` and `json!({ "target": { "kind": "SemanticId", "value": format!("{pkg}/type/quant_codec") } })`; both formerly stale expected identities are updated. |
