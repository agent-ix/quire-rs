---
id: FR-025
title: "Spec Corpus Model: Bounded In-Memory Document Set"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-024"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL provide a `Spec` (corpus) value: a bounded, in-memory set of loaded documents indexed by stable artifact id, with their intra-spec references resolved (FR-026). It is constructed from a `RepoLoad` (FR-024) and is the substrate for whole-spec queries (FR-027).

The corpus is a **data structure, not a stateful engine**. It performs no persistence, no background reload, no incremental update, and no resolution outside the loaded set (StR-006-AC-4). Its lifecycle is *construct, query, drop*.

### Public API

```rust
pub struct Spec { /* documents + id index + resolved edges */ }

impl Spec {
    pub fn from_repo(load: RepoLoad) -> Spec;
    pub fn from_path(root: &Path) -> Spec;   // convenience: load_repo then from_repo

    pub fn len(&self) -> usize;
    pub fn diagnostics(&self) -> &[Diagnostic];   // carried over from load + resolution
    // queries: see FR-027
}
```

### Indexing

- Each `LoadedDocument` is keyed by its `id` (FR-024 id derivation). The corpus builds a `HashMap<ArtifactId, usize>` (id → document slot) at construction.
- **Duplicate ids** within the loaded set SHALL be recorded as a `Diagnostic::DuplicateArtifactId` (mirrors the duplicate-archetype handling in the Registry); the first occurrence wins for lookup, and the duplicate is queryable. Construction does not fail.
- An `id` is treated as an opaque stable string; the corpus does not impose an id grammar (that is the schema's concern, per spec.md §2.2 "ID generation" exclusion).

### Lifecycle and concurrency

- `Spec` SHALL be **immutable after construction** and `Send + Sync` — the same lifecycle contract as `Registry` (StR-006-AC-5). To reflect on-disk changes, construct a new `Spec`; there is no in-place mutation, no add/remove, and no change subscription.
- Cloning a `Spec` is reference-counted (`Arc<Inner>`-style); clones share the underlying documents and edge set.

### Scope guard

- The corpus SHALL expose **no** API to persist itself, to register a filesystem watcher, or to resolve a reference against anything outside the loaded set. The absence of these is a verifiable property of the public surface (StR-006-AC-4). Inter-spec and stateful concerns belong to the service layer (StR-006).

## Acceptance

- **FR-025-AC-1**: `Spec::from_path(spec_dir)` over a fixture spec returns a corpus whose `len()` equals the number of parsed markdown artifacts under the directory.
- **FR-025-AC-2**: `Spec::from_repo(load)` indexes every document by its id; a by-id lookup (FR-027) returns the matching document and `None` for an absent id.
- **FR-025-AC-3**: A fixture with two documents sharing an id produces a `Diagnostic::DuplicateArtifactId`; construction succeeds and the first occurrence is the one returned by id lookup.
- **FR-025-AC-4**: A compile-time assertion confirms `Spec: Send + Sync` (generic-bound helper, parity with FR-013-AC-9).
- **FR-025-AC-5**: A test confirms the `Spec` public surface exposes no persistence, no watcher-registration, and no external-resolution method (scope guard, StR-006-AC-4) — enforced by an API-surface test/doc-test enumerating the allowed methods.
- **FR-025-AC-6**: After construction, queries answer with no filesystem read (parity with FR-013-AC-5 audit approach), confirming the corpus is fully in-memory (StR-006-AC-1).
