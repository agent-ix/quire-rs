---
id: SRE-316
title: "Evidence review of FR-050 trace target sources"
type: SpecReview
analysis: evidence
scope: "spec/functional/FR-050-declarative-coverage-computation.md (FR-050-AC-45)"
review_set: subset
---

## Summary

The earlier duplicate-ID finding is resolved in the current draft: the status-column criterion remains `FR-050-AC-45` / `TC-1804`, and the new trace-source criterion is `FR-050-AC-58` / `TC-1991`. Both IDs are unique in FR-050. `quoin advise --json` returns a distinct AC-58 recommendation of `property-based-testing` (`Test`, `Property` evidence kind), with no mismatch or inconclusive result; the authored `Test` method agrees with the catalog.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Resolved in the current draft: `FR-050-AC-45` identifies the status-column override (`Test (TC-1804)`), while the new `table_row`/`list_item`/`heading` criterion is uniquely `FR-050-AC-58` and maps to planned test `TC-1991`. The advisor now returns a distinct `FR-050-AC-58` `property-based-testing` recommendation with no mismatch or inconclusive result. | FR-050-AC-45, FR-050-AC-58, TC-1991 |
