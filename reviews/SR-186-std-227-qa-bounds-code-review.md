---
id: SR-186
title: "STD-227 code review of QA bounds integration"
type: SpecReview
analysis: code-review
scope: "STD-227 diff in .github/workflows/ci.yml, Cargo.toml, Cargo.lock, tests/corpus_cases.rs, tests/corpus_recall.rs; pinned qa-corpus src/corpus.rs, src/bounds.rs, src/verify.rs; AP-201 assurance context"
review_set: subset
---

## Summary

Reviewed the Rust integration against the repository's conventions, the rust-review checklist, the pinned QA Rust API, and the removed Python callers. The test suite now calls the QA crate through the pinned corpus submodule and keeps independent Quire grading checks.

## Assurance Context

AP-201 at `spec/assurance/AP-201-detection-minting.md` applies because the changed tests consume bounds and coverage evidence. The QA corpus Rust reader and Quire's Rust harness retain distinct case-loading and grading paths, while bounds calls into the QA crate share its implementation. The QA PR's final merge commit and CI evidence are unavailable until #36 merges. No active exception was identified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The first migrated TC-1025 assertion expected the old Python diagnostic for an empty YAML file, so a correct QA Rust refusal failed the test. | tests/corpus_cases.rs:1340 |
| FND-002 | low | The new TC-1027 mutation array used a complex inline function-pointer type and failed focused Clippy with `-D warnings`. | tests/corpus_cases.rs:1582 |

## Dispositions

- FND-001: fixed. Empty `expect.yaml` and `expect-pending.yaml` now assert the Rust reader's mapping refusal; the separate empty-assertion mutation still checks the graded assertion rule.
- FND-002: fixed. A local `Mutation` type alias makes the array readable and focused Clippy passes.

## Verdict

The introduced findings are fixed. The CI diff removes only the Python/PyYAML setup formerly needed by these tests; the Rust `cargo test` step still runs both changed integration targets. No foreign source, schema, or binary was copied into this repository.

## Validation

Focused TC-1021, TC-1025, TC-1027, TC-1028, TC-1032, and both `corpus_recall` tests passed. Focused Clippy for the two touched integration targets, `cargo fmt --all --check`, and `git diff --check` passed. The final PR-head full gate remains pending the QA merge and shared host slot.
