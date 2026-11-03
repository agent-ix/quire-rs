---
id: SR-189
title: "STD-227 QA bounds evidence analysis"
type: SpecReview
analysis: evidence
scope: "FR-065-AC-19/30-48 test citations, AP-201 evidence expectations, QA Rust source and quire-rs CI"
review_set: subset
---

## Summary

Compared the authored Test methods and citations with the executable Quire integration tests and the QA crate's own tests. The changed FR-065 criteria retain Test methods, and the advisor reports no mismatch for them.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-19's prior TC-1021 tag only checked two declaration strings and did not test bounds derivation or refusal. | spec/functional/FR-065-controlled-corpus-contract.md:624, tests/corpus_cases.rs:400 |
| FND-002 | medium | AC-47's QA Rust test citation lacked cases for a non-list witness entry and a mode family with no entry; the QA source has refusal branches but the pinned QA tests do not exercise both. | spec/functional/FR-065-controlled-corpus-contract.md:651, corpus/src/verify.rs:417 |

## Dispositions

- FND-001: fixed. TC-1021 now asserts the derived count sum and a declared-state mutation refusal.
- FND-002: fixed upstream. QA #36 merged at `9b2b760042ca7d4bf123f5ada0a7914aa95e8d2d` with focused non-list and missing-family tests alongside its unknown-channel test; this branch pins that commit.

## Verdict

The Quire-side evidence gap is fixed, and the pinned QA merge commit contains all three AC-47 refusal tests. `cargo test` in quire-rs executes this PR's integration tests but does not execute dependency unit tests; the QA owner reported its full `make ci` and 26 all-target tests passing before merge.

## Coverage

`quoin advise --mismatch-only` found no changed FR-065 criterion in its mismatch output. The computed matrix marks the changed criteria tagged, but a tag alone does not prove the asserted behavior; the mutation and derived-value checks were reviewed directly. Quire-rs's final full-gate result remains pending.
