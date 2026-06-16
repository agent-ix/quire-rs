---
id: FR-026
title: "Intra-Spec Reference Resolution"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-006"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

At corpus construction (FR-025), `quire-rs` SHALL resolve each document's reference stubs against the loaded set, producing a resolved edge set. Resolution is the join step that makes the reverse-edge and orphan queries of FR-027 possible. It is bounded to the loaded corpus — it never reaches outside the set (StR-006-AC-4).

### Edge stub sources

A reference stub is harvested from two places already parsed by the engine:

1. **Frontmatter `relationships`** — each entry contributes `{ source_id, target_id, edge_type, cardinality? }`. `target_id` is extracted from the entry's `target` (an `ix://` URI or a bare id); `edge_type` is the entry's `type` (e.g. `implements`, `requires`, `exercises`, `supersedes`).
2. **`ix://` body links** — links in the markdown body whose URI resolves to an artifact id contribute `{ source_id, target_id, edge_type: "references" }`.

Both sources feed one unified edge set (US-013-AC-2). When the **same** `(source_id, target_id, edge_type)` triple is declared by both a frontmatter `relationships` entry and an `ix://` body link, the resolver SHALL record it **once** (deduplicated) so it is not double-counted in `referencing`/`outgoing`. A frontmatter-declared `implements` edge and a body `references` edge between the same pair are distinct triples (different `edge_type`) and both are kept.

### Resolution rule

For each stub, the resolver SHALL look up `target_id` in the corpus id index (FR-025):

- **Resolved** — target present in the loaded set. The edge is recorded with `Resolution::Resolved` and appears in both the source's outgoing set and the target's incoming set (US-013-AC-5).
- **Dangling** — target absent from the loaded set. The edge is recorded with `Resolution::Dangling { target_id }` and surfaced as a queryable diagnostic. Construction does not fail (StR-006-AC-3).

A stub whose target id exists only in a *different* spec resolves to **Dangling** — the resolver does no I/O and consults nothing outside the loaded corpus (US-013-AC-4).

### Target id extraction

- An `ix://` target of the shape `ix://<org>/<repo>/spec/<class>/<ID>` contributes `target_id = <ID>` (the trailing artifact id). A bare `<ID>` target contributes itself.
- Extraction is purely lexical; the resolver does not fetch or validate the URI's authority. Whether `<ID>` is in the loaded set is the only thing that determines resolved vs. dangling.

### Cost and determinism

- Resolution SHALL be **O(edges)**: one hash lookup per stub against the id index (US-013-PC-2). No pairwise/quadratic matching.
- The resolved/dangling classification SHALL be **deterministic** for a given loaded set, independent of document or thread ordering (NFR-006, US-013-PC-3).

## Acceptance

- **FR-026-AC-1**: A frontmatter `relationships` entry whose `target` id is present in the corpus produces a `Resolved` edge carrying source id, target id, and edge type (US-013-AC-1).
- **FR-026-AC-2**: An `ix://` body link to a present id produces a `Resolved` edge in the same edge set (US-013-AC-2).
- **FR-026-AC-3**: A reference to an id absent from the loaded set produces a `Dangling` edge and a queryable diagnostic; construction succeeds (StR-006-AC-3, US-013-AC-3).
- **FR-026-AC-4**: A reference whose target id exists only in a different fixture spec is `Dangling`, not resolved (US-013-AC-4); the test confirms no filesystem access occurs during resolution.
- **FR-026-AC-5**: A `Resolved` edge appears in both `referencing(target)` and `outgoing(source)` query results (FR-027 substrate, US-013-AC-5).
- **FR-026-AC-6**: `ix://agent-ix/quire-rs/spec/functional/FR-021` as a target contributes `target_id = "FR-021"`; a bare `FR-021` target contributes the same — both resolve identically.
- **FR-026-AC-7**: A proptest scales the edge count and confirms resolution time grows linearly (O(edges)) and the classification is identical across thread counts (NFR-006).
- **FR-026-AC-8**: A fixture declaring the identical `(source, target, type)` edge via both a frontmatter `relationships` entry and an `ix://` body link produces exactly one edge; a same-pair edge with a different `type` from each source produces two.
