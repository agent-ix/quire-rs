---
id: US-013
title: "Agent Resolves Intra-Spec References to Find Orphans and Dangling Links"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "exercises"
---

## Story

As an **agent navigating a loaded spec**, I want the corpus to resolve each reference (a frontmatter `relationships` entry or an `ix://` body link) against the other documents in the same loaded set — classifying each edge as *resolved* (target present) or *dangling* (target absent) — so that I can follow `implements` / `exercises` / `requires` edges between artifacts and flag broken references without writing my own id-matching logic.

## Context

Reference resolution is the join step that turns a bag of parsed documents into a navigable set. The edge *stubs* come from two places already parsed by the engine: frontmatter `relationships` arrays (target id + type + cardinality) and `ix://` links in the markdown body. [FR-026](../functional/FR-026-intra-spec-reference-resolution.md) resolves those stubs **within the loaded corpus only** — it matches a stub's target against the corpus index ([FR-025](../functional/FR-025-spec-corpus-model.md)) and records whether it landed.

This is deliberately *not* a graph engine: there is no traversal query language, no transitive-closure precomputation, no persistence. It is the minimal join that makes [FR-027](../functional/FR-027-whole-spec-query-api.md)'s reverse-edge and orphan queries possible, and it is bounded to one spec ([StR-006](../stakeholder/StR-006-whole-spec-corpus.md)). A stub whose target is in another spec resolves to *dangling* — the corpus does not reach outside the loaded set.

## Acceptance

- **US-013-AC-1**: A frontmatter `relationships` entry whose target id is present in the corpus resolves to an edge with kind = resolved, carrying the source id, target id, and edge type.
- **US-013-AC-2**: An `ix://` body link whose target id is present resolves the same way (both stub sources feed one edge set).
- **US-013-AC-3**: A reference whose target id is absent from the loaded set is recorded as a *dangling* edge — queryable, non-fatal; corpus construction still succeeds ([StR-006-AC-3](../stakeholder/StR-006-whole-spec-corpus.md)).
- **US-013-AC-4**: Resolution is confined to the loaded set: a reference to an id that exists only in a different spec is dangling, not silently resolved against anything outside the corpus ([StR-006-AC-4](../stakeholder/StR-006-whole-spec-corpus.md)).
- **US-013-AC-5**: The resolved edge set is the substrate for [FR-027](../functional/FR-027-whole-spec-query-api.md)'s `referencing` / `outgoing` queries — a resolved edge appears in both the source's outgoing set and the target's incoming set.

## Efficiency Analysis

**Cost shape:** O(edges) one-time pass at corpus construction — each stub is one hash lookup against the id index ([FR-025](../functional/FR-025-spec-corpus-model.md)). No per-query resolution; [FR-027](../functional/FR-027-whole-spec-query-api.md) reads the precomputed edge set.

**Why bounded matters:** because resolution never leaves the loaded set, there is no I/O, no network, and no unbounded fan-out. The work is proportional to the spec's own reference count, which is small (hundreds of edges for a typical spec).

**When NOT to use:** following a reference into another spec or repo — that is inter-spec resolution and belongs to the service layer; here it simply reports dangling.

## Performance Criteria

- **US-013-PC-1**: Resolving all references in a 200-artifact spec (~hundreds of edges) completes in p50 < 5 ms on a single thread as part of corpus construction (folded into [US-012](./US-012-agent-audits-whole-spec.md)-PC-1's load+resolve budget). Bench: **TC-459**.
- **US-013-PC-2**: Resolution is O(edges) with a single hash lookup per stub; no quadratic pairwise matching. Verified by scaling the edge count and confirming linear growth.
- **US-013-PC-3**: Determinism — identical loaded set yields an identical resolved/dangling classification across runs and threads ([NFR-006](../non-functional/NFR-006-determinism.md)).
