---
id: SR-077
title: "Evidence review of the issue 412 skeptic scan requirement"
type: SpecReview
analysis: evidence
scope: "NFR-022; src/skeptic.rs; issue_412_oracle_selection.json"
review_set: subset
---

## Summary

The checkpoint preserves the sampled candidate values and line offsets and its
Rust gates pass, but it supplies no durable evidence for the scan-count or
source-shape obligations. The regression fixture is also invisible to the
repository's canonical trace relation and covers only part of the specified
differential domain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-429 | high | No operation-count harness or automated static audit proves the bounded scans. Replacing the indexed join with the original nested assertion scan still leaves the new output-equivalence fixture green, so the performance defect can return without a failing gate. | NFR-022-AC-1, NFR-022-AC-2, NFR-022-AC-4; TC-1810; TC-1811; src/skeptic.rs:616-650 |
| FND-430 | medium | The new Rust test has neither the repository's `#[trace("TC-…", "…-AC-…")]` attribute nor its `tcNNNN_` name. A coverage run therefore cannot attribute this regression evidence, and removing it does not change the claimed TC-1061 binding. | src/skeptic.rs:1203-1256; TC-1061; TC-1812 |
| FND-431 | medium | The fixture samples one two-binding span per language and one Rust helper, but it has no repeated same-name binding, unmatched binding, UTF-8, or CRLF case. Those omitted cases leave the differential domain in NFR-022-AC-3 unverified. | NFR-022-AC-3; TC-1812; tests/fixtures/corpus_cases/issue_412_oracle_selection.json |
| FND-432 | low | The evidence advisor recommends performance benchmarking for numeric operation thresholds, but wall-clock evidence cannot prove a traversal-count invariant. Unit counters, static-quality analysis, and property-based differential checks remain the appropriate catalog methods. | NFR-022 Measurement and Evaluation |
