---
id: StR-006
title: "Examine a Whole Spec as a Bounded, In-Memory Unit"
type: StR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-003"
    type: "requires"
    cardinality: "1:1"
---

## Stakeholder Need

A spec is not a single document — it is a **bounded set of related artifacts** (a `spec/` tree: StR, US, FR, NFR, test cases, `spec.md`) whose references point at *each other*: an FR `implements` a StR, a US `exercises` an FR, a test case verifies a requirement. Agents and analysis tooling (the `spec-analysis-*` and `spec-matrix` skills) repeatedly need to load such a set and examine it **as a whole** — "which FRs trace to no stakeholder need?", "which user stories have no test?", "show me FR-021 and everything that references it."

Today that work is done in Python by re-walking the directory and re-running regexes on every query. `quire-rs` already parses each document; what it lacks is a value that holds a *loaded set* of documents, resolves the references *among them*, and answers whole-set questions.

`quire-rs` SHALL provide the ability to load an entire spec as a bounded in-memory **corpus**, resolve its **intra-spec** references, and expose read-only whole-spec queries (traceability, coverage, reference navigation).

## Rationale

This need exists because a spec is not a single document but a bounded set of related artifacts (a `spec/` tree of StR, US, FR, NFR, test cases, `spec.md`) whose references point at *each other*, and agents and analysis tooling (the `spec-analysis-*` and `spec-matrix` skills) repeatedly need to examine such a set as a whole — "which FRs trace to no stakeholder need?", "which user stories have no test?", "show me FR-021 and everything that references it." Today that work is done in Python by re-walking the directory and re-running regexes on every query. `quire-rs` already parses each document; what it lacks is a value that holds a *loaded set*, resolves the references among them, and answers whole-set questions — a data structure with a load-examine-discard lifecycle, deliberately distinct from the stateful graph engine that was torn out of `filament-parser-lib`.

## Validation Criteria

This need is considered satisfied when a consumer can load a `spec/` directory into a single `Spec`/corpus value with one call and query it (by-type, referencing/reverse-edge, orphans) entirely from the in-memory structure without re-reading the filesystem, and when a reference whose target id is not present in the loaded set is reported as a queryable *dangling* diagnostic rather than an error. Satisfaction is further judged by the corpus performing no persistence, no background reload, and no resolution of references targeting a spec outside the loaded set, and by the corpus value being `Send + Sync` and immutable after construction (mirroring the `Registry` lifecycle) so it can be shared across threads for read-only analysis.

### Why this fits quire-rs (and why the old graph engine did not)

A graph engine was previously built into `filament-parser-lib` and torn out. The distinction that keeps this need inside `quire-rs` scope is **data structure vs. stateful engine**:

- **In scope (data structure):** load → resolve references within the loaded set → query by-id / by-type / referencing / orphans. The lifecycle is *load, examine, discard*. No persistence, no query language to interpret, no incremental update, no change-watching, no cross-corpus state.
- **Out of scope (stateful engine, → service layer):** persisting the resolved graph, a query DSL, caching across calls, incremental reparse on change, and resolving a reference that points into a *different* spec.

The rule: **intra-spec resolution is `quire-rs`; inter-spec or stateful is the service layer.** This is the bounded carve-out from the "cross-document graph queries" exclusion in `spec.md` §2.2 — it does not reopen the general graph engine.

## Priority

Should-Have

## Acceptance

- **StR-006-AC-1**: A consumer can load a `spec/` directory into a single `Spec`/corpus value with one call and then query it without re-reading the filesystem.
- **StR-006-AC-2**: Given a loaded spec, a query returns every artifact of a given type (e.g. all `FR`) and every artifact that references a given artifact id (reverse-edge lookup), entirely from the in-memory structure.
- **StR-006-AC-3**: A reference whose target id is **not present** in the loaded set is reported as a *dangling* reference rather than an error — the corpus loads successfully and surfaces the dangling edge as a queryable diagnostic.
- **StR-006-AC-4**: The corpus performs **no** persistence, no background reload, and no resolution of references that target a spec outside the loaded set — confirmed by the absence of any such API on the corpus surface.
- **StR-006-AC-5**: The corpus value is `Send + Sync` and immutable after construction (same lifecycle contract as `Registry`, StR-001-derived), so a long-lived consumer can share it across threads for read-only analysis.
