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

> **CR note (CR-067, the `ix://` URI grammar, 2026-08-18):** what counts as an `ix://` URI is
> now **stated as a grammar** rather than guessed by listing the characters that end one. The
> harvester matched `ix://[^\s)\]>"']+` — a blacklist that never treated a backtick as a
> delimiter — so prose naming the protocol, a bare `` `ix://` ``, matched as ``ix://` `` and
> minted a reference whose target was the closing backtick (agent-ix/quire-rs#89). The same
> blacklist accepted the documentation templates `ix://{org}/{repo}/{code}`,
> `ix://<org>/<repo>/spec/<class>/<ID>`, `ix://org/repo/...` and an `ix://([^)]+)` regex quoted
> in prose. This FR gains **FR-026-AC-12..13 and CON-1**.
>
> A well-formed `ix://` URI is a reference **wherever it appears**. Backticks and fenced blocks
> are not consulted, and CON-1 records that as a decision rather than an omission: a code span
> is typography, and [FR-039](./FR-039-unlinked-reference-detection.md) already takes the same
> position from the other side — it converts a *backticked* artifact id **into** a link
> (FR-039-AC-3). A rule that made a backticked link invisible here would contradict it.

> **CR note (internal relative-path links, ADR 0007, 2026-06-17):** intra-bundle
> references are authored as **relative-path Markdown links**
> (`[FR-002](./FR-002-schema-validation-pipeline.md)`), with `ix://` retained for external /
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

### The `ix://` URI grammar (CR-067)

A body occurrence of `ix://` contributes an edge stub only when it satisfies:

```
ix-uri   = "ix://" segment ( "/" segment )+ ( "#" fragment )?
segment  = one or more of [A-Za-z0-9._~@%+-], containing at least one [A-Za-z0-9]
fragment = one or more of [A-Za-z0-9._~-]
```

Two properties of the ecosystem's authored corpus are load-bearing here and were measured, not
assumed, over all 237 `~/dev` spec bundles:

- **Two segments is the minimum, not three.** `ix://agent-ix/workflow-service` — a repo-level
  reference with no artifact id — occurs 225 times. Only one single-segment form exists in the
  entire corpus.
- **The last segment is NOT required to look like an artifact id.**
  `ix://agent-ix/spec-artifacts-iso/master-requirements` (55),
  `ix://agent-ix/ecaz/spire-partition-object-header` (20),
  `ix://agent-ix/identity/aggregate_root/User` and `ix://npm/react-router-dom` reference
  declared objects and external packages. A `^[A-Z]{2,4}-[0-9]+$` rule on the target would
  discard all of them.

Characters that cannot legally appear in a URI — the backtick, `(`, `)`, `[`, `]`, `<`, `>`,
`{`, `}`, `^`, quotes and whitespace — therefore end one by construction rather than by
enumeration, and a segment made only of punctuation (`...`, `--`) is not a segment.

A match **immediately followed by `/`** SHALL be discarded whole. A trailing slash means the
next segment failed the grammar, so the URI is truncated rather than complete: `ix://org/repo/...`
must contribute nothing, not an edge to `repo`.

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
| FR-026-AC-1 | A frontmatter `relationships` entry whose `target` id is present in the corpus produces a `Resolved` edge carrying source id, target id, and edge type ([US-013-AC-1](../usecase/US-013-agent-resolves-intra-spec-refs.md)). | Test (TC-486) |
| FR-026-AC-2 | An `ix://` body link to a present id produces a `Resolved` edge in the same edge set ([US-013-AC-2](../usecase/US-013-agent-resolves-intra-spec-refs.md)). | Test (TC-487) |
| FR-026-AC-3 | A reference to an id absent from the loaded set produces a `Dangling` edge and a queryable diagnostic; construction succeeds ([StR-006-AC-3](../stakeholder/StR-006-whole-spec-corpus.md), [US-013-AC-3](../usecase/US-013-agent-resolves-intra-spec-refs.md)). | Test (TC-488) |
| FR-026-AC-4 | A reference whose target id exists only in a different fixture spec is `Dangling`, not resolved ([US-013-AC-4](../usecase/US-013-agent-resolves-intra-spec-refs.md)); the test confirms no filesystem access occurs during resolution. | Test (TC-489) |
| FR-026-AC-5 | A `Resolved` edge appears in both `referencing(target)` and `outgoing(source)` query results ([FR-027](./FR-027-whole-spec-query-api.md) substrate, [US-013-AC-5](../usecase/US-013-agent-resolves-intra-spec-refs.md)). | Test (TC-490) |
| FR-026-AC-6 | `ix://agent-ix/quire-rs/spec/functional/FR-021` as a target contributes `target_id = "FR-021"`; a bare `FR-021` target contributes the same — both resolve identically. | Test (TC-491) |
| FR-026-AC-7 | A proptest scales the edge count and confirms resolution time grows linearly (O(edges)) and the classification is identical across thread counts ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test (TC-492) |
| FR-026-AC-8 | A fixture declaring the identical `(source, target, type)` edge via both a frontmatter `relationships` entry and an `ix://` body link produces exactly one edge; a same-pair edge with a different `type` from each source produces two. | Test (TC-501) |
| FR-026-AC-9 | A relative-path body link `[FR-002](./FR-002-….md)` whose normalized destination matches a loaded document produces a `Resolved` `references` edge to that document's id (independent of the link text and the file slug); a relative-path link whose normalized destination matches no loaded document is `Dangling`, like an absent `ix://` target. | Test (TC-620) |
| FR-026-AC-10 | Relative-path links in an `index.md` or `log.md` contribute **no** `references` edges (navigation documents are excluded as a relative-path source), while a relative-path link in an ordinary artifact document is harvested. | Test (TC-621) |
| FR-026-AC-11 | The identical [FR-002](./FR-002-schema-validation-pipeline.md) edge declared via both a relative-path link and an `ix://` body link (or a frontmatter `references` entry) to the same target produces exactly one edge (dedup parity across all three sources). | Test (TC-622) |
| FR-026-AC-12 | Every `ix://` shape the ecosystem authors matches the grammar and still contributes its stub: `org/repo/ID`, `org/repo/spec/class/ID`, `org/repo`, `org/repo/spec/class/subdir/ID`, a target that is a declared object slug rather than an artifact id, an `object_type/Name` pair, a non-`agent-ix` authority, and a `#fragment`. Closing delimiters (`)`, `>`) still end the URI in `[t](ix://…)` and `<ix://…>` form. | Test (TC-880) |
| FR-026-AC-13 | A `ix://` occurrence that does not satisfy the grammar contributes **no** stub and **no** dangling diagnostic: the bare protocol `` `ix://` `` written in prose about the link format (the reported defect — its harvested target was the closing backtick), a single-segment URI, the `<org>`/`<ID>` and `{code}` documentation templates, an `ix://([^)]+)` regex quoted in prose, and a URI truncated by an elided segment (`ix://org/repo/...`), which is discarded whole rather than matching its first two segments. | Test (TC-881) |
| FR-026-AC-14 | Every clause of the relative-destination filter is checked **one exclusion at a time**: an empty destination, any `scheme://` form, a bare `#anchor`, `mailto:`, `tel:`, and a non-`.md` extension each yield no stub, including where the excluded form also carries a `.md` tail. AC-9 stated this and nothing tested it — the gap was found by mutating each `&&` in the filter to `||` with no test failing (CR-071). | Test (TC-897) |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-026-CON-1 | A well-formed `ix://` URI SHALL contribute its stub **regardless of markdown context** — inside an inline code span, inside a fenced block, or in plain prose. The harvester MUST NOT consult backticks or fences. A code span is typography; making a link inside one invisible would silently drop real references and would contradict [FR-039](./FR-039-unlinked-reference-detection.md)-AC-3, which converts a *backticked* artifact id into a link. Known consequence, measured at **75 of ~5,950 `ix://` lines (1.3%)**: an `ix://` inside a fenced block that is genuinely illustrative rather than a reference will contribute an edge. Narrowing that is a separate decision and requires its own corpus measurement. | Design | Test (TC-882) |

## Dependencies

- **Upstream**: [StR-006](../stakeholder/StR-006-whole-spec-corpus.md), [FR-025](./FR-025-spec-corpus-model.md), [FR-006](./FR-006-frontmatter-with-fallback.md)
- **Downstream**: [FR-027](./FR-027-whole-spec-query-api.md)
