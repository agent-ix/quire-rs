---
id: SR-20261113104846
title: "EARS conformance review of PLAT-1519 frontmatter projection spec"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md and spec/functional/FR-081-record-sections-extraction.md"
review_set: subset
---

## Summary

Reviewed the edited normative requirement statements and new FR-069-AC-32..37 against EARS, including the scoped Quire grammar diagnostics. No EARS defect was introduced by this diff; the findings in the base/integrity reviews concern conflicting or underspecified behavior, not trigger grammar.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-069 located record validation behavior | examined | Quire composes authored domain frontmatter into the existing nested `SemanticExtraction.record`... |
| FR-069-AC-21 | examined | `data_schema` validates the composed declaration record... `record_validation` validates the yield record. |
| FR-069-AC-32 | examined | For an object archetype with a resolved `data_schema`, registry-backed semantic extraction projects authored domain frontmatter... |
| FR-069-AC-33 | examined | In the nested record, a present frontmatter list value `known_anomalies: []` remains an empty array... |
| FR-069-AC-34 | examined | The frontmatter projection excludes `id`, `title`, `type`, `object`, `uuid`, `relationships`, `description`, and `tags`... |
| FR-069-AC-35 | examined | A valid composed declaration record validates against the resolved `data_schema`... |
| FR-069-AC-36 | examined | An invalid frontmatter value is located at the source line of its key... |
| FR-069-AC-37 | examined | An archetype without a resolved `data_schema` produces no frontmatter-projection schema findings... |
| FR-081 body-reader prohibition | examined | The `record-sections` body reader SHALL NOT read a record key from frontmatter. |
| FR-081-AC-2 | examined | A frontmatter `status: accepted` yields no `status`. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
