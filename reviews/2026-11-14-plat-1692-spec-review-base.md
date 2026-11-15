---
id: SR-20261113104845
title: "Base review of FR-050-AC-68 unbacked matrix claims"
type: SpecReview
analysis: base
scope: "spec/functional/FR-050-declarative-coverage-computation.md#FR-050-AC-68; schemas/output/coverage-v1.schema.json"
review_set: all
---

## Summary

FR-050-AC-68 clearly defines deterministic diagnostic-only claims for row-rejected declared tags and unmarked trace IDs, including source location, deduplication, and unchanged binding/status/totals/strict behavior. The contract omits the output schema change needed for the new criterion field, so a conforming payload would fail the repository's strict coverage schema.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-68 adds `unbacked_claims` to each coverage matrix criterion but does not require the strict coverage-v1 JSON schema to permit the field and its record shape. | FR-050-AC-48, FR-050-AC-65, FR-050-AC-68; schemas/output/coverage-v1.schema.json#/$defs/CoverageMatrixCriterion |

### FND-001 detail

The hand-maintained `CoverageMatrixCriterion` schema has `additionalProperties: false` and currently lists only `identity`, `record_binders`, `id`, `statement`, `method`, `binders`, and `status`. AC-68 requires the `coverage_matrix` emitted by `quire matrix` and `quoin matrix` to carry `unbacked_claims`, but does not state that the schema must be extended with its optional array and exact record shape. A payload implementing AC-68 therefore fails validation against the published schema. FR-050-AC-65 already makes schema membership and payload validity explicit for additive coverage output. Amend AC-68 to require the optional field and nested record schema, and require payload validation against `schemas/output/coverage-v1.schema.json`.

### Independent checks

- Targeted `quire validate spec/functional/FR-050-declarative-coverage-computation.md --module /home/peter/.ix/filament/modules/spec-artifacts-iso`: exit 0; only the existing non-canonical-shape warning on FR-050-AC-67.
- `git diff --check origin/main...HEAD`: exit 0.
- `quoin advise --repo . --json`: exit 0; FR-050-AC-68 authored method is Test with no mismatch; recommendations include negative-abuse-testing (input-validation) and property-based-testing (universal property shape).
- Broad `quire validate --scope . 'spec/**/*.md' --summary`: exit 3 on 10 unrelated existing structural errors under `spec/assets/*.md` and `spec/assurance/MP-*`; no new AC-68 warning.
- AP-201-detection-minting is active and requires `spec-review` acceptance (`review_policy.mode: require`). This report is an independent review artifact; the required human acceptance gate remains outstanding.

## Disposition — commit bc01843

**FND-001 resolved in the amended requirement.** AC-68 now explicitly requires the optional `unbacked_claims` property in strict `schemas/output/coverage-v1.schema.json` `$defs.CoverageMatrixCriterion`, constrained to an array of closed objects with exactly `reason`, `id`, `path`, `line`, and `symbol`; all fields are required, and `reason` is restricted to the two declared values. It explicitly requires both payload shapes (claims present and omitted when empty) to validate. The diagnostic-only clause continues to preserve binders, computed status, backing totals, and strict verdict. The FR amendment therefore resolves the spec defect; schema and payload implementation remain for the later implementation stage.

- Re-read `git show bc01843`: it changes only FR-050-AC-68; no output schema implementation is included in this spec commit, as expected at the spec gate.
- Targeted FR-050 validation: exit 0, with the same existing non-canonical-shape warning on AC-67 at line 385.
- `git diff origin/main...HEAD --check`: exit 0.
- AP-201 human acceptance remains outstanding; no acceptance was attempted.
