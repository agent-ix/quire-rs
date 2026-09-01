---
id: SR-059
title: "Failure-domain review of source-grounded assurance export"
type: SpecReview
analysis: failure-domain
scope: "FR-067, FR-068"
review_set: all
---

## Summary

Reviewed version-premise trust boundaries, entity identities, export purity, and graph topology. CR-157 makes invalid roots and incomplete tolerant-loader state fail closed and represents unread input as `unknown`; no open failure-domain gap remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No open failure-domain issues | - |

## Analysis

- Extension boundary: import validates the complete v1 document before constructing records and then checks caller-accepted premises.
- Identity: module schema tuples, artifacts, obligations, symbols, relations, and observations each have explicit stable identity tuples.
- Purity: export construction performs no Git, network, persistence, or cross-corpus reads.
- Topology: the exporter copies the bounded `Spec::edges()` set and never traverses recursively; cycles and disconnected components therefore terminate by construction.
- Partial failure: incomplete provenance returns no export; unread source becomes a reasoned `unknown` observation rather than a false `missing` result.
