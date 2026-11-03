---
id: SR-190
title: "STD-227 QA bounds scope and boundary analysis"
type: SpecReview
analysis: scope-boundary
scope: "FR-065 corpus-reader boundary, AP-201 independence statement, QA Rust dependency, quire-rs corpus tests"
review_set: subset
---

## Summary

The QA repository owns corpus discovery, conformance, bounds derivation, and its Rust runner. Quire owns its engine grading and tests the shared QA data through its own harness; the submodule is the authoritative dependency reference.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The old assurance wording assigned independent-reader status to Python after the direct bounds call moved to the QA Rust crate. | spec/assurance/AP-201-detection-minting.md:72 |

## Dispositions

- FND-001: fixed. AP-201 now distinguishes Quire's separate reader and grading path from bounds assertions that share QA's Rust implementation.

## Verdict

The responsibility boundary is clear. QA behavior is guaranteed by its own CI and by Quire's direct integration calls at the pinned submodule commit; Quire grading parity is asserted by TC-1028 against the QA control mapping. No QA source is copied into quire-rs.

## Boundary Allocation

| Responsibility | Owner | Evidence |
| --- | --- | --- |
| Corpus discovery and bounds | qa-corpus | QA Rust tests and Quire TC-1021/1025/1027/1032 direct API calls |
| Engine execution and grading | quire-rs | TC-1028 pair comparison and `corpus_recall` |
| Submodule commit selection | quire-rs | Pinned QA merge SHA `9b2b760042ca7d4bf123f5ada0a7914aa95e8d2d` |

The QA owner reported passing `make ci` and all-target tests before the merge. Quire-rs's final full gate remains pending. No compatibility reader or copied schema is introduced.
