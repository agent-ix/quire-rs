---
id: SR-2026112016223475846349
title: "Code review - PLAT-1873 cross-module semantic references (quire-rs PR 723)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@9a2ad6dec9c066781053502c8d98a531b20bb368; PLAT-1873; spec/functional/FR-075-model-feature-extraction.md (FR-075-AC-12, FR-075-AC-25), spec/non-functional/NFR-010-api-stability.md (NFR-010-AC-1, NFR-010-AC-3, NFR-010-AC-4), src/semantic/context.rs, src/semantic/mod.rs, tests/it/semantic_model.rs, tests/it/semantic_systems.rs, Cargo.toml, CHANGELOG.md"
review_set: subset
---

# Code review - PLAT-1873 (quire-rs PR 723)

## Summary

Ticket PLAT-1873. Reviewed draft PR head `9a2ad6dec9c066781053502c8d98a531b20bb368` against `origin/main`. The diff adds explicit imported-artifact resolution with target locations, typed wrong-kind/unresolved diagnostics, and synthetic integration coverage. No ambient loading was found. Rust review included public API compatibility, implementation paths, tests, diagnostics, serialization/schema, and repository conventions. Findings are tracked below. Public API compatibility must follow NFR-010. The added integration test's `line_of` helper duplicates an existing helper in `tests/it/semantic_features.rs`; the new fixture setup also repeats bundle request and schema validation setup.

## Assurance Context

AP-201 (active) applies because the change alters semantic reference collection, diagnostics, and the published semantic schema. The profile requires `make ci`, controlled-corpus and relevant mutation/self-tests, plus enforced `spec-review`. Reviewer evidence available: focused integration test passed (1/1), remote PR checks reported passed by lead, and `git diff --check origin/main...HEAD` passed. I did not run `make ci` (reserved for lead), controlled-corpus, or mutation/self-tests. The enforced SpecReview operation remains pending user-selected review scope and human acknowledgement; its provider run failed with `invariant_provider_exit_nonzero` (run `plat-1873-spec-review`). This code review does not claim that gate accepted.

## Verdict

**FAIL with 3 findings (2 high, 1 medium).** The two high findings are a documented pre-1.0 public API break without the required minor-version release/change record, and duplicated test setup under the repository's no-duplication review rule. The medium finding is missing direct coverage for the newly specified imported `Transitions.Emits` inventory behavior.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The change adds required Rust fields to public `BundleArtifact` and `ImportedArtifact`, breaking downstream struct literals while the crate remains at 0.62.3 and no PLAT-1873 breaking-change entry exists in `CHANGELOG.md`. NFR-010 requires pre-1.0 breaking changes to be documented across the next minor position. Preserve source compatibility or release/document the break at the policy-required minor position. | src/semantic/context.rs:59,82; Cargo.toml:29; CHANGELOG.md |
| FND-002 | high | `extract_with_imported_artifact` duplicates the request construction and semantic schema compile/validation block already implemented by `extract_with_bundle_artifacts` in the same test file. Generalize and reuse the existing helper so schema validation has one owner. | tests/it/semantic_systems.rs:30-125 |
| FND-003 | medium | Updated FR-075-AC-25 adds explicit-inventory `Transitions.Emits` cases (success, missing id, wrong kind, untyped target), but its only tagged test remains `reference_cell_rules`, which covers the legacy unchecked imported identity and no inventory. The new PR test is tagged AC-12 and exercises systems allocation, so AC-25's new branch has no test. Add direct AC-25 assertions for the inventory path. | tests/it/semantic_model.rs:1343; tests/it/semantic_systems.rs:591 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-075-AC-12 | examined | New explicit inventory systems-reference behavior and test reviewed |
| FR-075-AC-25 | examined | New imported Emits branch lacks a direct tagged test (FND-003) |
| NFR-010-AC-1 | examined | Version and public surface inspected |
| NFR-010-AC-3 | examined | Changelog inspected; no PLAT-1873 entry |
| NFR-010-AC-4 | examined | `cargo-semver-checks` executable unavailable; source compatibility break established from required public struct fields and prior definitions |
| src/semantic/target.rs | examined | Explicit inventory resolution and wrong-kind distinction reviewed; no ambient module loading found |
| src/semantic/model.rs | examined | Imported Emits path reviewed |
| src/semantic/systems.rs | examined | Reference/target source locations and diagnostics reviewed |
| schemas/output/semantic-v1.schema.json | examined | New output members reviewed |
| tests/it/semantic_systems.rs | examined | New regression coverage and duplicate helper setup reviewed |
| AP-201 | context_only | Enforced SpecReview gate remains pending; see Assurance Context |

## Verification

- `cargo test -j8 --test it explicitly_indexed_cross_module_references_carry_both_locations_and_refuse_bad_targets`: passed, 1 test.
- Remote PR checks: reported passed by lead.
- `git diff --check origin/main...HEAD`: passed.
- Not run by reviewer: `make ci` (lead-owned single aggregate gate), clippy, `cargo deny`, controlled corpus, mutation/self-tests. `cargo-semver-checks` was unavailable in PATH.
