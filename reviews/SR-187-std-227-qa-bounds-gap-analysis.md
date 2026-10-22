---
id: SR-187
title: "STD-227 gap analysis of QA bounds evidence"
type: SpecReview
analysis: gap-analysis
scope: "STD-227 tests/corpus_cases.rs, tests/corpus_recall.rs, FR-065-AC-19/30-48, AP-201, and pinned qa-corpus bounds and corpus APIs"
review_set: subset
---

## Summary

Checked the changed FR-065 obligations against the computed matrix, executable tests, QA Rust source, and the quire-rs CI invocation. The migrated tests exercise the pinned QA implementation directly; Quire's own grading remains a separate reader.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-19 was tagged by TC-1021, but the prior assertions did not derive a bounds report or cause an undeclared-state refusal. | spec/functional/FR-065-controlled-corpus-contract.md:624, tests/corpus_cases.rs:400 |
| FND-002 | medium | The first assurance text still described a Python reader as the independent implementation path after QA moved to Rust. | spec/assurance/AP-201-detection-minting.md:72 |

## Dispositions

- FND-001: fixed. TC-1021 now asserts the derived state-count sum equals declared cells and removes `covered` from a scratch declaration to require a named undeclared-state refusal.
- FND-002: fixed. AP-201 names the two Rust readers and states that direct QA bounds calls do not provide an independent bounds oracle.

## Verdict

The introduced evidence gaps are fixed. The computed matrix marks all changed FR-065 criteria (AC-19 and AC-30 through AC-48) tagged. The pinned QA merge commit contains focused AC-47 tests for unknown, non-list, and missing-family witness channels. The QA crate's unit tests ran in its owning repository; quire-rs `cargo test` runs the changed integration tests but does not execute dependency unit tests.

## Coverage

Plan completion: not assessed. Repository-wide matrix output has 619 tagged, 241 untagged, and 105 method-without-symbol criteria; those pre-existing counts prevent a repository-wide PASS claim. The changed tests contain concrete refusal and derived-value assertions. Final PR-head full gate remains pending the shared host slot.
