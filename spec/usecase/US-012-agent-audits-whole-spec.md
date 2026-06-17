---
id: US-012
title: "Agent Audits a Whole Spec for Traceability and Coverage Gaps"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-027"
    type: "exercises"
---

## Story

As an **LLM agent (or the `spec-analysis-integrity` / `spec-matrix` skill) examining a spec**, I want to load an entire `spec/` directory into one corpus value and ask whole-spec questions — "which FRs trace to no StR?", "which user stories have no test case?", "list every artifact that references FR-021" — so that I can report traceability and coverage gaps without re-walking the directory and re-running regexes on every question.

## Context

This is the consumer that motivated [StR-006](../stakeholder/StR-006-whole-spec-corpus.md). A spec is a bounded set of artifacts (StR/US/FR/NFR/TC) cross-referencing each other via frontmatter `relationships` and `ix://` body links. The corpus ([FR-025](../functional/FR-025-spec-corpus-model.md)) loads the set once and resolves the intra-spec references ([FR-026](../functional/FR-026-intra-spec-reference-resolution.md)); the query API ([FR-027](../functional/FR-027-whole-spec-query-api.md)) answers whole-spec questions over the resolved structure.

The agent never needs a persistent graph database for this — the lifecycle is *load the spec, audit it, report, discard*. Inter-spec references (a link into a *different* spec) are reported as dangling, not resolved (that is the service layer's job per [StR-006](../stakeholder/StR-006-whole-spec-corpus.md)).

This is exactly what the `spec-analysis-*` skills do today in Python by hand; against a quire-rs corpus they query an already-resolved structure instead.

## Acceptance

- **US-012-AC-1**: Loading a `spec/` tree yields a corpus over which `by_type("FR")` returns every functional requirement and `by_type("US")` returns every user story ([FR-027](../functional/FR-027-whole-spec-query-api.md)).
- **US-012-AC-2**: `orphans(of_type="FR", missing_edge_type="implements", toward_type="StR")` (or equivalent) returns every FR with no `implements` edge to a StR — the traceability-gap query.
- **US-012-AC-3**: `referencing("FR-021")` returns every artifact in the spec whose frontmatter or body references FR-021 (reverse-edge lookup, [FR-027](../functional/FR-027-whole-spec-query-api.md)).
- **US-012-AC-4**: A user story with no test-case reference is reported by the coverage query; a user story with one is not.
- **US-012-AC-5**: All queries answer from the in-memory corpus with no filesystem re-read after construction ([StR-006-AC-1](../stakeholder/StR-006-whole-spec-corpus.md)).

## Efficiency Analysis

**Round trips:** 1 load (the [FR-024](../functional/FR-024-parallel-repo-walk.md) walk), then O(1)-amortized in-memory queries. Today's Python `spec-analysis` re-globs and re-greps per question.

**Cost shape:** resolution is O(edges) once at construction ([FR-026](../functional/FR-026-intra-spec-reference-resolution.md)); each query is a hash-index lookup or a single pass over the resolved edge set ([FR-027](../functional/FR-027-whole-spec-query-api.md)), not a re-parse. For a 200-artifact spec the whole audit is sub-millisecond after load.

**When NOT to use:** cross-spec questions ("does this FR's StR live in another repo's spec?") — those need the service layer; the corpus reports such edges as dangling and stops there.

## Performance Criteria

- **US-012-PC-1**: Constructing a corpus from a 200-artifact spec (load + resolve) completes in p50 < 50 ms on a single thread on the canonical runner (load bounded by [NFR-015](../non-functional/NFR-015-repo-walk-throughput.md); resolution O(edges)). Bench: **TC-457**.
- **US-012-PC-2**: `by_id` / `by_type` / `referencing` queries are O(1) / O(matches) against in-memory indices — no per-query filesystem or parse cost ([FR-027](../functional/FR-027-whole-spec-query-api.md)). Bench: **TC-458**.
- **US-012-PC-3**: Repeated audits over an unchanged corpus are deterministic — identical gap/orphan/reference results across runs and threads ([NFR-006](../non-functional/NFR-006-determinism.md); corpus is `Send + Sync` per [StR-006-AC-5](../stakeholder/StR-006-whole-spec-corpus.md)).
