---
id: FR-027
title: "Whole-Spec Query API"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL expose read-only whole-spec queries over a constructed `Spec` (FR-025) and its resolved edge set (FR-026). These are views over an already-resolved in-memory structure — none re-reads the filesystem and none re-parses (StR-006-AC-1).

### Public API

```rust
impl Spec {
    // direct lookups
    pub fn by_id(&self, id: &str) -> Option<&LoadedDocument>;
    pub fn by_type(&self, artifact_type: &str) -> impl Iterator<Item = &LoadedDocument>;

    // edge navigation (over FR-026 resolved edges)
    pub fn outgoing(&self, id: &str) -> impl Iterator<Item = &Edge>;     // edges FROM id
    pub fn referencing(&self, id: &str) -> impl Iterator<Item = &Edge>;  // edges TO id (reverse)
    pub fn dangling(&self) -> impl Iterator<Item = &Edge>;               // unresolved references

    // coverage / traceability
    pub fn orphans(&self, of_type: &str, missing_edge_type: &str,
                   toward_type: Option<&str>) -> impl Iterator<Item = &LoadedDocument>;
}
```

### Query semantics

- **`by_id`** — O(1) hash lookup against the id index (FR-025).
- **`by_type`** — every loaded document whose frontmatter `type`/`artifact_type` matches. The corpus does not interpret the type beyond string equality. A document with **no** `type`/`artifact_type` frontmatter field is *untyped*: it is never returned by any `by_type` query and is reachable only via `by_id`. Untyped documents are recorded as a `Diagnostic::UntypedArtifact` at construction so coverage audits can surface them.
- **`outgoing(id)`** — resolved + dangling edges whose source is `id`.
- **`referencing(id)`** — resolved edges whose target is `id` (reverse-edge lookup). Dangling edges have no resolved target document, so they appear only in `outgoing` and `dangling`, never in `referencing` (US-012-AC-3).
- **`orphans(of_type, missing_edge_type, toward_type)`** — every document of `of_type` that has **no** resolved outgoing edge of `missing_edge_type` (optionally constrained to a target of `toward_type`). This is the traceability-gap query: e.g. `orphans("FR", "implements", Some("StR"))` → every FR with no `implements` edge to a StR (US-012-AC-2).
- All iterators yield in a **deterministic order** (sorted by id) so results are reproducible (NFR-006, US-012-PC-3).

### Cost

- Queries SHALL answer in O(1) (`by_id`) or O(matches)/O(edges) (the rest) against in-memory indices — no per-query filesystem or parse cost (US-012-PC-2). Edge indices (forward + reverse) are built once at construction (FR-026), not per query.

### Scope guard

- The query surface is **read-only**: no method mutates the corpus, and there is no traversal/query DSL to interpret. Transitive closure, path-finding, and a query language are explicitly **not** provided — they are stateful-engine / service-layer concerns (StR-006). Callers that need transitive reachability compose `outgoing`/`referencing` themselves.

## Acceptance

- **FR-027-AC-1**: `by_type("FR")` over a fixture spec returns exactly the FR documents; `by_type("US")` returns exactly the user stories (US-012-AC-1).
- **FR-027-AC-2**: `referencing("FR-021")` returns every artifact whose resolved edges target FR-021, and excludes artifacts that do not reference it (US-012-AC-3).
- **FR-027-AC-3**: `orphans("FR", "implements", Some("StR"))` returns every FR lacking a resolved `implements` edge to a StR, and excludes FRs that have one (US-012-AC-2).
- **FR-027-AC-4**: A user story with no resolved edge to a test case is returned by the coverage query; one with such an edge is not (US-012-AC-4).
- **FR-027-AC-5**: `outgoing(id)` includes that document's dangling edges; `referencing` and `dangling` agree — every dangling edge is in exactly one `outgoing` set and the `dangling()` set, and in no `referencing` set.
- **FR-027-AC-6**: Every query iterator yields in sorted-by-id order; two runs produce identical sequences (NFR-006).
- **FR-027-AC-7**: A test (tracing/strace-style, parity with FR-013-AC-5) confirms no filesystem read occurs during any query after construction (StR-006-AC-1, US-012-AC-5).
- **FR-027-AC-8**: A criterion bench measures `by_id`, `referencing`, and `orphans` over a 200-artifact corpus and confirms sub-millisecond per-query latency (US-012-PC-2; feeds TC-458).
- **FR-027-AC-9**: A document lacking a `type`/`artifact_type` field is never returned by `by_type`, is returned by `by_id`, and produces a `Diagnostic::UntypedArtifact` at construction.
