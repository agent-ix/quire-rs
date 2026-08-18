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

> **CR note (internal relative-path links, ADR 0007, 2026-06-17):** intra-bundle
> references are authored as **relative-path Markdown links**
> (`[FR-002](./FR-002-graph-edges.md)`), with `ix://` retained for external /
> cross-repo references only. This FR gains a **third edge-stub source** —
> relative-path body links resolved via a path→id index over the loaded corpus —
> and new ACs (FR-026-AC-9..11). This also makes good [FR-038](./FR-038-okf-bundle-validation.md)'s existing Okf prose
> ("broken `ix://` / relative references degrade to warnings"), which previously
> anticipated relative references that nothing harvested. Bare prose codes are
> still **not** harvested (that heuristic is what ADR 0007 removes); converting a
> bare code into an explicit link is [FR-039](./FR-039-unlinked-reference-detection.md)'s job.

## Description

At corpus construction ([FR-025](./FR-025-spec-corpus-model.md)), `quire-rs` SHALL resolve each document's reference stubs against the loaded set, producing a resolved edge set. Resolution is the join step that makes the reverse-edge and orphan queries of [FR-027](./FR-027-whole-spec-query-api.md) possible. It is bounded to the loaded corpus — it never reaches outside the set ([StR-006-AC-4](../stakeholder/StR-006-whole-spec-corpus.md)).

### Edge stub sources

A reference stub is harvested from three places already parsed/loaded by the engine:

1. **Frontmatter `relationships`** — each entry contributes `{ source_id, target_id, edge_type, cardinality? }`. `target_id` is extracted from the entry's `target` (an `ix://` URI or a bare id); `edge_type` is the entry's `type` (e.g. `implements`, `requires`, `exercises`, `supersedes`).
2. **`ix://` body links** — links in the markdown body whose URI resolves to an artifact id contribute `{ source_id, target_id, edge_type: "references" }`. `ix://` is the **external / cross-repo** form (ADR 0007).
3. **Relative-path body links** — Markdown links whose destination is a relative file path ([FR-002](./FR-002-schema-validation-pipeline.md), [StR-001](../stakeholder/StR-001-single-rust-engine.md)) are the **internal / intra-bundle** form (ADR 0007). The destination is normalized relative to the source document's directory and matched against the corpus **path→id index** (built from the loaded documents' paths, [FR-025](./FR-025-spec-corpus-model.md)); a match contributes `{ source_id, target_id, edge_type: "references" }`. A relative destination that matches no loaded path is `Dangling` like any other unresolved reference; non-relative destinations (`http(s)://`, `mailto:`, `ix://`, bare in-document `#anchor`) are not relative-path stubs and are ignored by this source.

   Navigation documents — `index.md` and `log.md` — are excluded as a relative-path **source**: their wall-to-wall relative links list the bundle contents and MUST NOT flood the graph with `references` edges.

All three sources feed one unified edge set ([US-013-AC-2](../usecase/US-013-agent-resolves-intra-spec-refs.md)). When the **same** `(source_id, target_id, edge_type)` triple is declared by more than one source (e.g. a frontmatter `relationships` entry and a body link, or an `ix://` and a relative-path link to the same target), the resolver SHALL record it **once** (deduplicated) so it is not double-counted in `referencing`/`outgoing`. A frontmatter-declared `implements` edge and a body `references` edge between the same pair are distinct triples (different `edge_type`) and both are kept.

### Resolution rule

For each stub, the resolver SHALL look up `target_id` in the corpus id index ([FR-025](./FR-025-spec-corpus-model.md)):

- **Resolved** — target present in the loaded set. The edge is recorded with `Resolution::Resolved` and appears in both the source's outgoing set and the target's incoming set ([US-013-AC-5](../usecase/US-013-agent-resolves-intra-spec-refs.md)).
- **Dangling** — target absent from the loaded set. The edge is recorded with `Resolution::Dangling { target_id }` and surfaced as a queryable diagnostic. Construction does not fail ([StR-006-AC-3](../stakeholder/StR-006-whole-spec-corpus.md)).

A stub whose target id exists only in a *different* spec resolves to **Dangling** — the resolver does no I/O and consults nothing outside the loaded corpus ([US-013-AC-4](../usecase/US-013-agent-resolves-intra-spec-refs.md)).

### Target id extraction

- An `ix://` target of the shape `ix://<org>/<repo>/spec/<class>/<ID>` contributes `target_id = <ID>` (the trailing artifact id). A bare `<ID>` target contributes itself.
- Extraction for `ix://` / bare targets is purely lexical; the resolver does not fetch or validate the URI's authority. Whether `<ID>` is in the loaded set is the only thing that determines resolved vs. dangling.
- A **relative-path** target contributes the `target_id` of whichever loaded document occupies the normalized path (path→id index lookup), independent of the link's visible text or the file's slug. Normalization joins the destination onto the source document's directory and collapses `.`/`..` segments; a normalized path outside the corpus, or matching no loaded document, yields no `target_id` and the stub is `Dangling`.

### Cost and determinism

- Resolution SHALL be **O(edges)**: one hash lookup per stub against the id index ([US-013](../usecase/US-013-agent-resolves-intra-spec-refs.md)-PC-2). No pairwise/quadratic matching.
- The resolved/dangling classification SHALL be **deterministic** for a given loaded set, independent of document or thread ordering ([NFR-006](../non-functional/NFR-006-determinism.md), [US-013](../usecase/US-013-agent-resolves-intra-spec-refs.md)-PC-3).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-026-AC-1 | A frontmatter `relationships` entry whose `target` id is present in the corpus produces a `Resolved` edge carrying source id, target id, and edge type ([US-013-AC-1](../usecase/US-013-agent-resolves-intra-spec-refs.md)). | Test |
| FR-026-AC-2 | An `ix://` body link to a present id produces a `Resolved` edge in the same edge set ([US-013-AC-2](../usecase/US-013-agent-resolves-intra-spec-refs.md)). | Test |
| FR-026-AC-3 | A reference to an id absent from the loaded set produces a `Dangling` edge and a queryable diagnostic; construction succeeds ([StR-006-AC-3](../stakeholder/StR-006-whole-spec-corpus.md), [US-013-AC-3](../usecase/US-013-agent-resolves-intra-spec-refs.md)). | Test |
| FR-026-AC-4 | A reference whose target id exists only in a different fixture spec is `Dangling`, not resolved ([US-013-AC-4](../usecase/US-013-agent-resolves-intra-spec-refs.md)); the test confirms no filesystem access occurs during resolution. | Test |
| FR-026-AC-5 | A `Resolved` edge appears in both `referencing(target)` and `outgoing(source)` query results ([FR-027](./FR-027-whole-spec-query-api.md) substrate, [US-013-AC-5](../usecase/US-013-agent-resolves-intra-spec-refs.md)). | Test |
| FR-026-AC-6 | `ix://agent-ix/quire-rs/spec/functional/FR-021` as a target contributes `target_id = "FR-021"`; a bare `FR-021` target contributes the same — both resolve identically. | Test |
| FR-026-AC-7 | A proptest scales the edge count and confirms resolution time grows linearly (O(edges)) and the classification is identical across thread counts ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test |
| FR-026-AC-8 | A fixture declaring the identical `(source, target, type)` edge via both a frontmatter `relationships` entry and an `ix://` body link produces exactly one edge; a same-pair edge with a different `type` from each source produces two. | Test |
| FR-026-AC-9 | A relative-path body link `[FR-002](./FR-002-….md)` whose normalized destination matches a loaded document produces a `Resolved` `references` edge to that document's id (independent of the link text and the file slug); a relative-path link whose normalized destination matches no loaded document is `Dangling`, like an absent `ix://` target. | Test |
| FR-026-AC-10 | Relative-path links in an `index.md` or `log.md` contribute **no** `references` edges (navigation documents are excluded as a relative-path source), while a relative-path link in an ordinary artifact document is harvested. | Test |
| FR-026-AC-11 | The identical [FR-002](./FR-002-schema-validation-pipeline.md) edge declared via both a relative-path link and an `ix://` body link (or a frontmatter `references` entry) to the same target produces exactly one edge (dedup parity across all three sources). | Test |
| FR-026-AC-14 | Every clause of the relative-destination filter is checked **one exclusion at a time**: an empty destination, any `scheme://` form, a bare `#anchor`, `mailto:`, `tel:`, and a non-`.md` extension each yield no stub, including where the excluded form also carries a `.md` tail. AC-9 stated this and nothing tested it — the gap was found by mutating each `&&` in the filter to `||` with no test failing (CR-071). | Test |

> **AC-12 and AC-13 are reserved for CR-067** (agent-ix/quire-rs#89, the `ix://`
> URI grammar), which is open against `main` and numbered first. This criterion
> takes AC-14 so the two land without a numbering conflict, rather than
> colliding and being renumbered during a merge nobody would re-read.

## Dependencies

- **Upstream**: [StR-006](../stakeholder/StR-006-whole-spec-corpus.md), [FR-025](./FR-025-spec-corpus-model.md), [FR-006](./FR-006-frontmatter-with-fallback.md)
- **Downstream**: [FR-027](./FR-027-whole-spec-query-api.md)
