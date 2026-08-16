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

## Description

`quire-rs` SHALL provide a `Spec` (corpus) value: a bounded, in-memory set of loaded documents indexed by stable artifact id, with their intra-spec references resolved ([FR-026](./FR-026-intra-spec-reference-resolution.md)). It is constructed from a `RepoLoad` ([FR-024](./FR-024-parallel-repo-walk.md)) and is the substrate for whole-spec queries ([FR-027](./FR-027-whole-spec-query-api.md)).

The corpus is a **data structure, not a stateful engine**. It performs no persistence, no background reload, no incremental update, and no resolution outside the loaded set ([StR-006-AC-4](../stakeholder/StR-006-whole-spec-corpus.md)). Its lifecycle is *construct, query, drop*.

### Two-tier document model (CR-047)

Each loaded document holds **two tiers** (the [FR-005](./FR-005-parse-document-api.md) header/body split, CR-046, applied to the corpus):

- **Header tier — eager at construction**: path, `id`, `uuid`, the full frontmatter mapping, and the verbatim document text. Indexing, [FR-026](./FR-026-intra-spec-reference-resolution.md) reference resolution, and every [FR-027](./FR-027-whole-spec-query-api.md) query that reads only identity/frontmatter/raw text are answered from this tier.
- **Body tier — lazy**: the parsed `QuireDocument` is materialised on **first access, exactly once**; concurrent first accessors receive the **identical** value. Materialisation is a pure function of the stored verbatim text — it performs no filesystem read.

External immutability is unchanged: `Spec` stays `Send + Sync` and externally immutable — no query ever returns a different answer twice.

The two tiers give consumers a **caller-declared depth** (CR-049): a caller touches exactly the bodies its own declarations name — coverage materialises the archetypes its `traceability:` model declares ([FR-050](./FR-050-declarative-coverage-computation.md) AC-18), validation reads every document because validating every document is what it declares it does — rather than every caller silently receiving maximum depth. Depth is expressed by *touching*, never by a mode flag: the corpus has one shape and the lazy tier makes selective consumption free.

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

- Each `LoadedDocument` is keyed by its `id` ([FR-024](./FR-024-parallel-repo-walk.md) id derivation). The corpus builds a `HashMap<ArtifactId, usize>` (id → document slot) at construction.
- **Duplicate ids** within the loaded set SHALL be recorded as a `Diagnostic::DuplicateArtifactId` (mirrors the duplicate-archetype handling in the Registry); the first occurrence wins for lookup, and the duplicate is queryable. Construction does not fail.
- An `id` is treated as an opaque stable string; the corpus does not impose an id grammar (that is the schema's concern, per spec.md §2.2 "ID generation" exclusion).

### Lifecycle and concurrency

- `Spec` SHALL be **immutable after construction** and `Send + Sync` — the same lifecycle contract as `Registry` ([StR-006-AC-5](../stakeholder/StR-006-whole-spec-corpus.md)). To reflect on-disk changes, construct a new `Spec`; there is no in-place mutation, no add/remove, and no change subscription.
- Cloning a `Spec` is reference-counted (`Arc<Inner>`-style); clones share the underlying documents and edge set.

### Scope guard

- The corpus SHALL expose **no** API to persist itself, to register a filesystem watcher, or to resolve a reference against anything outside the loaded set. The absence of these is a verifiable property of the public surface ([StR-006-AC-4](../stakeholder/StR-006-whole-spec-corpus.md)). Inter-spec and stateful concerns belong to the service layer ([StR-006](../stakeholder/StR-006-whole-spec-corpus.md)).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-025-AC-1 | `Spec::from_path(spec_dir)` over a fixture spec returns a corpus whose `len()` equals the number of parsed markdown artifacts under the directory. | Test |
| FR-025-AC-2 | `Spec::from_repo(load)` indexes every document by its id; a by-id lookup ([FR-027](./FR-027-whole-spec-query-api.md)) returns the matching document and `None` for an absent id. | Test |
| FR-025-AC-3 | A fixture with two documents sharing an id produces a `Diagnostic::DuplicateArtifactId`; construction succeeds and the first occurrence is the one returned by id lookup. | Test |
| FR-025-AC-4 | A compile-time assertion confirms `Spec: Send + Sync` (generic-bound helper, parity with [FR-013-AC-9](./FR-013-archetype-loader.md)). | Test |
| FR-025-AC-5 | A test confirms the `Spec` public surface exposes no persistence, no watcher-registration, and no external-resolution method (scope guard, [StR-006-AC-4](../stakeholder/StR-006-whole-spec-corpus.md)) — enforced by an API-surface test/doc-test enumerating the allowed methods. | Test |
| FR-025-AC-6 | After construction, queries answer with no filesystem read (parity with [FR-013-AC-5](./FR-013-archetype-loader.md) audit approach), confirming the corpus is fully in-memory ([StR-006-AC-1](../stakeholder/StR-006-whole-spec-corpus.md)) — **including lazy body materialisation** (CR-047): first-touch `body()` parses the verbatim text captured at load and performs no filesystem read either. | Inspection (extended TC-485) |
| FR-025-AC-7 | `len`/`by_id`/`by_type`/`diagnostics` and the [FR-026](./FR-026-intra-spec-reference-resolution.md)/[FR-027](./FR-027-whole-spec-query-api.md) edge queries (`edges`/`outgoing`/`referencing`/`dangling`/`orphans`) complete with **zero body parses**; touching one document's body then parses exactly that document (CR-047). | Test (TC-817) |
| FR-025-AC-8 | Concurrent first-touch of the same document's body parses **exactly once** and every racer receives the identical `QuireDocument` (CR-047; the loom model in [NFR-017-AC-4](../non-functional/NFR-017-concurrency-permutation.md), raced for real under the [NFR-018](../non-functional/NFR-018-ffi-sanitizer-lanes.md) TSAN lane). | Test (TC-815, TC-816) |

> **CR-049 note (2026-08-15):** The caller-declared-depth paragraph is new
> (agent-ix/quire-rs#94, umbrella #90). It states what the lazy tier is
> *for*: the `traceability:` model was always a projection declared before
> the walk began, and the engine parsed everything and filtered afterwards.
> Depth is emergent from CR-047's first-touch semantics — no new API, no
> mode flag — so the testable claim lives on the consumer:
> FR-050-AC-18 pins that coverage leaves undeclared archetypes' bodies
> unmaterialised while the report stays byte-identical.

> **CR-047 note (2026-08-15):** Bodies are lazy (agent-ix/quire-rs#93, umbrella
> #90). Since CR-046 the walk parses **headers only** — membership, identity
> and the full frontmatter map come from one frontmatter extraction — so a
> caller that never reads a body no longer pays for parsing one:
> `Spec::from_path` on a corpus where no body-reading query runs parses zero
> bodies, and resolution stays eager over frontmatter + raw text alone. The
> body tier is a per-document once-init cell behind `Arc<SpecInner>`, seeded on
> first `body()` access. The trade this makes is deliberate: a lazy cache
> needs interior mutability, which the FR-024-AC-9 no-shared-mutable audit
> banned wholesale — the audit's pattern is therefore **widened** (it now also
> catches `OnceLock`/`OnceCell`) and the cell is a **named, justified
> exemption** in `scripts/audits/check_no_shared_mutable.sh`, not a silenced
> pattern; the FR-024-AC-9 guarantee itself narrows to the parallel walk
> fan-out, which never touches the cell. Concurrency risk is carried by
> NFR-017-AC-4 (loom, TC-815) and the NFR-018 TSAN lane (TC-816).

## Dependencies

- **Upstream**: [StR-006](../stakeholder/StR-006-whole-spec-corpus.md), [FR-024](./FR-024-parallel-repo-walk.md), [FR-026](./FR-026-intra-spec-reference-resolution.md)
- **Downstream**: [FR-027](./FR-027-whole-spec-query-api.md)
