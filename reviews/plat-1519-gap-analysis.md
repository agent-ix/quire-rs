---
id: SR-20261113085042
title: "Traceability gap analysis of quire-rs PLAT-1519"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, src/extract/mod.rs, src/semantic/resolver.rs, src/semantic/surface.rs, src/validate_document.rs, tests/it/located_findings.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1519 item 1, PR quire-rs#681. The direct Quire static matrix reports all seven new FR-069 AC-32..38 as tagged to `tests/it/located_findings.rs`. Review of the behavior/test assertions found an incomplete edge case under AC-38 and incomplete localization coverage under AC-36. The current `quoin matrix --repo . --json` output disagrees with Quire's static matrix, reporting all 1,249 criteria as untagged; this mismatch is recorded as a tooling/scope caveat, not treated as evidence that these seven bindings are absent.

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

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The AC-38 test asserts duplicate-key omission and diagnostics, but does not assert that `data_schema` still validates the remaining unambiguous values; implementation returns before that validation. | FR-069-AC-38; tests/it/located_findings.rs:861- |
| FND-002 | medium | The AC-36 test does not cover nested or multiple unknown properties, where diagnostic localization falls back to the opening frontmatter line rather than the undeclared property's source line. | FR-069-AC-36; tests/it/located_findings.rs:775- |

## Verdict

The seven ACs in the changed contract have declared test tags in Quire's computed static matrix. The assertions are not complete for the two findings above, so matrix presence alone does not establish coverage of those required behaviors. `quoin matrix --repo . --json` produced a conflicting all-untagged result (1,249 criteria, including these seven); direct `quire matrix --scope . --format tsv` showed the new ACs tagged and 230 untagged repository-wide. Treat this as an unresolved matrix command discrepancy. No broader repository gap audit was used to attribute historical gaps to this PR.
