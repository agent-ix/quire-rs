---
id: SR-20261103118156
title: "failure-domain review of FR-038-AC-11 (quire-rs PR 646)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs@07890bc3; spec/functional/FR-038-okf-bundle-validation.md (FR-038-AC-11)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-038
    type: references
---

# Failure-domain review of FR-038-AC-11 (quire-rs PR 646)

## Summary

Ticket: PLAT-1407. Failure-domain lens on FR-038-AC-11: identity confusion between a source id and a source path. Context read: FR-025 (duplicate ids: first occurrence wins for lookup, construction does not fail), FR-026 (edges are `(source_id, target_id, edge_type)` triples deduplicated across harvest sources), and `src/corpus/resolve.rs` (`Edge` carries `source: ArtifactId` only, no path). Mapping an edge back to a path therefore goes through the id index, which is not one-to-one when ids collide.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-11 does not say which path a `dangling-reference` carries when its source id is shared by two documents (FR-025-AC-3 allows that and returns the first). Edges are keyed by source id and deduplicated by `(source, target, type)`, so a dangling edge authored in the second document is reported at the first document's path, and identical dangling edges in both collapse to one finding naming one file: the "locate the broken edge" goal fails exactly in the case it is hardest to find by id. State the rule, e.g. one finding per source document that authored the edge, each with that document's path | spec/functional/FR-038-okf-bundle-validation.md:107 |

## Verdict

One medium identity gap. No other failure mode found: id-less (path-keyed) documents map to their own path, and the foreign-authority skip of AC-9/AC-10 emits nothing so carries no path.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | 6d279b6 states the duplicate-id rule: edges dedup by (source, target, type) and a broken edge declared in both documents is reported once, at the first document's path; TC-2227 also pins the second-document-only case to the first document's path. The rule is now explicit and testable; the collision itself is located by FR-025's DuplicateArtifactId diagnostic |
