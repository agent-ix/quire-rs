---
id: SR-178
title: "STD-228 base review of benchmark and measurement port"
type: SpecReview
analysis: base
scope: "STD-228 changes to FR-050-AC-30, FR-050-AC-32, and spec/tests.md"
review_set: subset
---

## Summary

Reviewed the changed verification pointers and the unchanged benchmark obligations against the Rust port, manifest, and former Python runner. The port changes the implementation language and test locations without changing requirement IDs or ratchet policy.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-050-AC-30 requires a tier-2 pinned SHA, but the existing `filament-ide-rs` manifest entry declares only `require_clean`; both the removed Python runner and the Rust port check local cleanliness without checking a revision. A clean checkout at another commit can therefore be scored. | FR-050-AC-30; bench/manifest.json; STD-228 |

## Verdict

The verification-pointer changes are consistent and introduce no new pin or requirement gap. The pre-existing SHA requirement and implementation disagree; QSL-942 owns reconciliation of the requirement and benchmark behavior.

## Dispositions

| Finding | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred: QSL-942 | The discrepancy predates the port. This change preserves the measured behavior and adds no tracking record or compatibility path. |

## Validation

Scoped `quire validate` passed for FR-050 and `spec/tests.md`. The manifest's tier-2 `filament-ide-rs` row has `require_clean: true` and no SHA; origin/main `scripts/bench.py` checks Git status but does not inspect `rev-parse HEAD`.
