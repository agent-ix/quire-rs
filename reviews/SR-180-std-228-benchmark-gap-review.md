---
id: SR-180
title: "STD-228 benchmark port traceability gap review"
type: SpecReview
analysis: gap-analysis
scope: "STD-228 combined benchmark and measurement Rust port, FR-050-AC-30 and FR-050-AC-32"
review_set: subset
---

## Summary

Compared the changed FR-050 verification pointers, new trace-tagged Rust tests, benchmark code, and computed matrix. The port's tests cover ratchet and measurement export behavior; the existing FR-050 table interruption prevents the computed matrix from listing those authored criteria.

## Verdict

**FAIL** for computed traceability until QSL-943 repairs the FR-050 table or matrix reader. The affected criteria and tests are authored, but matrix output does not bind them. This gap predates the Rust port.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed matrix stops FR-050 at AC-26. Authored AC-27 onward, including the changed AC-30 and AC-32, are absent, so their new Rust trace tags do not appear as verification evidence. | FR-050-AC-30; FR-050-AC-32; QSL-943 |

## Dispositions

| Finding | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred: QSL-943 | The acceptance table already has blank-line interruptions on origin/main. The owning follow-up covers the table or parser repair; this port leaves unrelated FR structure unchanged. |

## Coverage

`quire matrix --scope . --format tsv` produced 966 rows. FR-050-AC-26 was the last FR-050 criterion at output line 405. `rg 'FR-050-AC-(27|28|29|30|31|32)'` found no row. Passing an explicit absolute process module path produced the same 966 rows. The changed `tools/quire-bench/tests/benchmark.rs` and `measurement_export.rs` have `#[trace("FR-050-AC-30")]` and `#[trace("FR-050-AC-32")]` on the relevant tests. Reverse-gap inspection found those tests call the production scorer, metric reader, and exporter rather than a duplicate test implementation. No plan was supplied; plan completion: not assessed. Optional semantic review: not selected.
