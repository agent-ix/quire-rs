---
id: SR-20261113104845
title: "base review of PLAT-1519 frontmatter projection spec"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md and spec/functional/FR-081-record-sections-extraction.md"
review_set: subset
---

## Summary

Reviewed the changed FR-069/FR-081 statements and ACs for IDs, links, consistency, testability, and criterion coverage. The new coverage ACs are currently untagged as expected in this spec-first PR; two contract ambiguities should be resolved before implementation: FR-081-AC-2 still reads as a prohibition on frontmatter values in the output record, and FR-069 does not define precedence when frontmatter and body/declaration sources supply the same record key.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-069 located record validation paragraph | examined | Quire composes authored domain frontmatter into the existing nested `SemanticExtraction.record`, alongside semantic declaration and FR-081 record-section values. |
| FR-069-AC-21 | examined | `data_schema` validates the composed declaration record, including projected frontmatter fields, while `record_validation` validates the yield record. |
| FR-069-AC-32 | examined | ...projects authored domain frontmatter into the existing nested `SemanticExtraction.record`, composing it with FR-081 section-extracted and semantic declaration values... |
| FR-069-AC-33 | examined | `known_anomalies: []` remains an empty array, an absent `known_anomalies` key remains absent, `supplier: unknown` remains the string `unknown`... |
| FR-069-AC-34 | examined | Excludes metadata... every other authored frontmatter key, including undeclared keys, is retained with its JSON type... |
| FR-069-AC-35 | examined | A valid composed declaration record validates... invalid values... makes document validation invalid with a `semantic.record-invalid` diagnostic... |
| FR-069-AC-36 | examined | Invalid frontmatter value is located at the source line of its key; missing required property at opening frontmatter line; unknown-property findings use its source line. |
| FR-069-AC-37 | examined | Without a resolved `data_schema`, no frontmatter-projection schema findings; output remains valid against `semantic-v1.schema.json`. |
| FR-081 frontmatter-reader paragraph | examined | The `record-sections` body reader SHALL NOT read a record key from frontmatter. The separate frontmatter projection into the nested record is specified by FR-069. |
| FR-081-AC-2 | examined | ...and a frontmatter `status: accepted` yields no `status`. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-081-AC-2 says a frontmatter `status: accepted` yields no `status`, while FR-069-AC-32/34 project authored domain frontmatter into that same nested `SemanticExtraction.record`. The CR-420 note narrows the prohibition to the body reader, but AC-2 does not state that its expected absence is limited to the record-sections contribution; clarify the oracle so the AC does not contradict the new output contract. | FR-081-AC-2, FR-069-AC-32, FR-069-AC-34 |
| FND-002 | medium | FR-069-AC-32 composes frontmatter, FR-081 section values, and semantic declaration values into one record but specifies no behavior when two sources author the same key. Different merge precedence or overwrite behavior changes the validated value and its source line; define the collision rule and its diagnostic/location behavior. | FR-069-AC-32, FR-069-AC-36 |

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
