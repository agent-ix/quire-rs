---
id: SR-20261113085041
title: "Code review of quire-rs PLAT-1519 record validation"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/extract/mod.rs, src/semantic/resolver.rs, src/semantic/surface.rs, src/validate_document.rs, tests/it/located_findings.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1519 item 1, reviewed in quire-rs#681 at `2967fb1c69b69e0dff632e5032f1b5e992da42a5` against `plat-1519-data-schema-spec` (`0650af73ec33b68ed8b3b75bfc5dabd1bf40b78f`). The implementation projects frontmatter record values and validates them against `data_schema`, but the duplicate-source early return skips validation of other unambiguous keys, and unknown-property source localization fails for nested or multiple unknown properties.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-069-AC-32 | examined | For an object archetype with a resolved `data_schema`, registry-backed semantic extraction projects authored domain frontmatter into the existing nested `SemanticExtraction.record`, composing it with FR-081 section-extracted and semantic declaration values; the top-level `semantic-v1` envelope remains unchanged. |
| FR-069-AC-33 | examined | In the nested record, a present frontmatter list value `known_anomalies: []` remains an empty array, an absent `known_anomalies` key remains absent, `supplier: unknown` remains the string `unknown`, and an authored `disposition_clause` is retained unchanged; no field is inferred or defaulted. |
| FR-069-AC-34 | examined | The frontmatter projection excludes `id`, `title`, `type`, `object`, `uuid`, `relationships`, `description`, and `tags`; every other authored frontmatter key, including undeclared keys, is retained with its JSON type so the resolved `data_schema` validates it and can reject unknown properties under a closed schema. |
| FR-069-AC-35 | examined | A valid composed declaration record validates against the resolved `data_schema`; a wrong JSON type, invalid `support_status`/`severity` enum, invalid `end_of_support` format/date, or undeclared `digest` under a closed schema makes document validation invalid with a `semantic.record-invalid` diagnostic naming the failing property path, while conforming values pass. |
| FR-069-AC-36 | examined | An invalid frontmatter value is located at the source line of its key; a missing required property is located at the opening frontmatter line; unknown-property findings name the undeclared property and use its source line. |
| FR-069-AC-37 | examined | An archetype without a resolved `data_schema` produces no frontmatter-projection schema findings, and the existing semantic extraction output remains valid against `semantic-v1.schema.json`. |
| FR-069-AC-38 | examined | When semantic declaration, FR-081 section, and frontmatter sources provide the same record key, each later source occurrence emits a `semantic.duplicate-record-key` error at its source line naming the key and the earliest and later source kinds; the key is omitted from the composed record, document validation is invalid, and `data_schema` validates the remaining unambiguous values. |
| FR-081-AC-2 | examined | A `failure_mode` `## Analysis` table extracts an `analysis` object; `## Status` holding `accepted` and a `## Provenance` table contribute `status` and `provenance` through the record-sections body reader, the same fixture without `## Provenance` has no body-reader contribution for `provenance`, and a frontmatter `status: accepted` has no body-reader contribution for `status` (the composed nested record may still contain its frontmatter projection as specified by FR-069). |
| record_validation yield-only behavior | context_only | The change is scoped to composed record validation against `data_schema`; existing yield-only validation behavior remains unchanged. |
| FR-173-AC-13 | context_only | The caller required that this engine change not weaken FR-173-AC-13; Quoin's criterion/implementation was outside the PR diff. |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | A duplicate record key suppresses `data_schema` validation of the remaining unambiguous values. | src/validate_document.rs:459-460, src/semantic/surface.rs:463-466; FR-069-AC-38 | implementation-bug-despite-evidence |
| FND-002 | medium | Unknown-property diagnostics lose the property path/source line for nested or multiple unknown keys. | src/validate_document.rs:474-488; FR-069-AC-36 | implementation-bug-despite-evidence |
| FND-003 | medium | The existing FR-081 fixture still expects frontmatter status to be absent from the composed record, so the full integration suite fails under the new FR-069 projection. | crates/quire-fixtures/fixtures/record-sections/cases.json:202-207; FR-069-AC-32; FR-081-AC-2 | correct-requirement-no-evidence |

## Verdict

Request changes. Two implementation issues violate explicit edge-case requirements. The aggregate gate also exposed one stale fixture oracle that conflicts with the updated FR-081-AC-2 body-reader/composed-record distinction. FR-173-AC-13 was treated as a context-only constraint: this diff is limited to Quire extraction and validation and does not alter or weaken Quoin's criterion. The yield-only behavior of existing `record_validation` was preserved in this diff.

## Coverage

The changed projection, resolver, validator, and located integration tests were reviewed. The required and edge-case checks were run as recorded in the review comment; the aggregate `make ci` run was still in progress when this artifact was authored.
