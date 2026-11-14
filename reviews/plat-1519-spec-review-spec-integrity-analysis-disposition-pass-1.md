---
id: SR-20261113104847
title: "Integrity review of PLAT-1519 frontmatter projection spec"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md and spec/functional/FR-081-record-sections-extraction.md"
review_set: subset
---

## Summary

Reviewed edited FR-069 and FR-081 contract text and ACs for consistency, completeness, atomicity, testability, traceability, and failure oracles. The new ACs cover JSON type preservation, metadata exclusions, schema failures, source lines, and the semantic-v1 envelope; two unresolved consistency/merge cases remain.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-069 located record validation paragraph | examined | Composes authored domain frontmatter with FR-081 section-extracted and semantic declaration values; keeps the `semantic-v1` envelope unchanged. |
| FR-069-AC-21 | examined | `data_schema` validates composed declaration record; `record_validation` validates yield record. |
| FR-069-AC-32 | examined | Projects authored frontmatter into nested `SemanticExtraction.record`, composing section and declaration values. |
| FR-069-AC-33 | examined | Empty list, absent key, `unknown` sentinel, and authored `disposition_clause` are preserved without inference/defaults. |
| FR-069-AC-34 | examined | Excludes reserved metadata and retains every other authored frontmatter key with JSON type. |
| FR-069-AC-35 | examined | Invalid types, enums, date/format, and closed-schema unknown property produce `semantic.record-invalid` with property path. |
| FR-069-AC-36 | examined | Invalid frontmatter value and unknown property use source line; missing required property uses opening frontmatter line. |
| FR-069-AC-37 | examined | No resolved schema means no projection schema findings and semantic-v1 remains valid. |
| FR-081 CR-420 note and prohibition | examined | The prohibition applies to the body reader; separate projection is specified by FR-069. |
| FR-081-AC-2 | examined | A frontmatter `status: accepted` yields no `status`. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-081-AC-2 says a frontmatter `status` yields no `status` without limiting that assertion to the body-reader mapping. FR-069-AC-32/34 now project domain frontmatter into the same nested output record, so the ACs can assert opposite observable outputs for the same document. Clarify that AC-2 tests only the body reader's contribution, or update its expected composed record. | FR-081-AC-2, FR-069-AC-32, FR-069-AC-34 |
| FND-002 | medium | The composition rule does not specify what happens when frontmatter and section/declaration sources provide the same key. This leaves the final record value, schema result, and diagnostic source line open to multiple valid interpretations; specify precedence or an explicit duplicate-source diagnostic. | FR-069-AC-32, FR-069-AC-36 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-081-AC-2 now limits its absence assertion to the record-sections body-reader contribution and explicitly allows the composed record to contain FR-069's frontmatter projection. |
| FND-002 | fixed | FR-069 now specifies that duplicate source occurrences emit `semantic.duplicate-record-key`, omit the ambiguous key, and invalidate document validation; FR-069-AC-38 gives the test oracle. |

### Fixed excerpts

**FND-001 after_excerpt:**

> A `failure_mode` `## Analysis` table extracts an `analysis` object; `## Status` holding `` `accepted` `` and a `## Provenance` table contribute `status` and `provenance` through the record-sections body reader, the same fixture without `## Provenance` has no body-reader contribution for `provenance`, and a frontmatter `status: accepted` has no body-reader contribution for `status` (the composed nested record may still contain its frontmatter projection as specified by FR-069).

**FND-002 after_excerpt:**

> When semantic declaration, FR-081 section, and frontmatter sources provide the same record key, each later source occurrence emits a `semantic.duplicate-record-key` error at its source line naming the key and the earliest and later source kinds; the key is omitted from the composed record, document validation is invalid, and `data_schema` validates the remaining unambiguous values.
