---
id: SR-179
title: "STD-228 Rust review of benchmark and measurement port"
type: SpecReview
analysis: code-review
scope: "STD-228 combined benchmark and measurement diff: tools/quire-bench, deleted Python scripts and tests, Makefile, CI, Cargo and deny configuration, and changed FR-050 evidence pointers"
review_set: subset
---

## Summary

Reviewed the Rust port against quire-rs conventions, the prior Python behavior, changed CI gates, and the Rust review checklist. The workspace gates add the tool crate's tests, Clippy lane, and license check without narrowing existing lanes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The initial scorer converted integer counts to `f64` before ordering them. Adjacent counts above 2^53 rounded to the same float, so a regressed count could be reported as held. | tools/quire-bench/src/lib.rs |
| FND-002 | low | The first port copied `check_engine.py` capability helpers into public Rust functions after their ceremony-script callers had been deleted. They had only an isolated test and added unused API surface. | tools/quire-bench/src/engine.rs |

## Verdict

Both introduced findings are fixed. New tests carry `#[trace]` markers, exercise the real scorer and exporter functions, and include the large-integer regression. No new unsafe, async, vendored files, or compatibility path appears in the diff.

## Dispositions

| Finding | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Integer-to-integer comparisons now use exact `u64` or `i64` ordering; the focused regression test asserts both ratchet directions above 2^53. |
| FND-002 | fixed | Removed the unused capability helpers and isolated test. The consumer engine build function remains because benchmark collection and export call it. |

## Validation

Focused benchmark and measurement tests, focused Clippy, formatting, workspace license check, and scoped spec validation passed. Full repository gates remain scheduled under the shared heavy-build lock.
