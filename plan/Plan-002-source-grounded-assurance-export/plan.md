---
id: Plan-002
title: "quire-rs — source-grounded assurance export"
type: Plan
status: completed
relationships:
  - target: ix://agent-ix/quire-rs/StR-007
    type: references
  - target: ix://agent-ix/quire-rs/US-018
    type: references
  - target: ix://agent-ix/quire-rs/FR-067
    type: references
  - target: ix://agent-ix/quire-rs/FR-068
    type: references
  - target: ix://agent-ix/quire-rs/IT-001
    type: references
  - target: ix://agent-ix/quire-rs/NFR-006
    type: references
---
# Implementation Plan: Source-grounded assurance export

TDD plan for the stable, offline quire-rs assurance export requested by #386. The plan owns only the producer library and its published v1 schema; Quoin consumer implementation and assurance verdicts remain downstream.

## Requirements Summary

### Stakeholder Requirements

- [x] **StR-007**: Assurance decisions use source-grounded, interpretable data.

### User Stories

- [x] **US-018**: A reviewer consumes one validated export without reconstructing Quire semantics.

### Functional Requirements

- [x] **FR-067**: Versioned assurance export envelope, schema, premise validation, and compatible reader.
- [x] **FR-068**: Source-grounded artifacts, obligations, symbols, relations, and relation observations.

### Integration and Cross-cutting Requirements

- [x] **IT-001**: Quire supplies the pinned offline producer contract and fixture (Quoin consumer implementation remains downstream).
- [x] **NFR-006**: Every observable collection and serialized byte stream is deterministic.

## Dependency Graph

- `FR-025 + FR-051 + FR-053 + FR-055 -> FR-067`
  Reason: the envelope inventories the existing corpus, symbol, obligation, and published-output contracts.
- `FR-026 + FR-051 + FR-053 + FR-067 -> FR-068`
  Reason: projection copies the existing graph and record identities into the versioned envelope.
- `FR-067 + FR-068 -> IT-001`
  Reason: the compatibility fixture is useful to Quoin only after both envelope and record semantics are stable.
- `NFR-006 -> every task`
  Reason: ordering and byte identity are part of the exported contract, not a later optimization.

The seams are `Registry` for module/schema premises, `Spec` for documents and edges, `obligation::derive` for declared obligations, and `symbols::{SymbolExtraction, SymbolGraph}` for source bindings. New code lives behind one `assurance` module and reuses those public records rather than harvesting Markdown or tags again.

## Test Plan

### Contract and Unit Tests

- [x] **TC-1084**: compile the hand-authored draft-2020-12 schema and validate the complete v1 fixture.
- [x] **TC-1085**: reject each incomplete source/module/root premise without a partial export.
- [x] **TC-1086**: inventory active schemas once, in canonical order, with semantic SHA-256 digests.
- [x] **TC-1087**: reject unsupported format, module version, and schema digest before returning records.
- [x] **TC-1088**: prove byte identity and source-revision isolation.
- [x] **TC-1089**: round-trip and mutation-check the checked-in v1 compatibility fixture.
- [x] **TC-1090**: prove existing coverage/properties contracts remain unchanged.
- [x] **TC-1095**: keep verifies evidence distinct from implements scope.
- [x] **TC-1097**: restrict Quire-produced freshness to unknown/not_applicable.

### Integration and Property Tests

- [x] **TC-1091**: reproduce artifact identities and exact-byte locators.
- [x] **TC-1092**: reproduce obligation records and both statement hashes.
- [x] **TC-1093**: preserve stable symbol identities and capabilities.
- [x] **TC-1094**: prove corpus-relation projection is a bijection, including declared zero-edge kinds and dangling edges.
- [x] **TC-1096**: exercise available, missing, not_applicable, and unknown relation observations.
- [x] **TC-1098**: prove projection determinism and unrelated-document isolation.

### Static Verification

- [x] **TC-1099**: reject forbidden frontmatter, Markdown-query, source-tag, Git, network, and persistence dependencies in the exporter.

## Remaining Work

### Track A: Critical Path (serial)

- **A1 = Task-011** v1 schema, typed records, and fail-closed reader — Hard; exit: a complete golden validates and every unsupported premise returns no typed record.
- **A2 = Task-012** deterministic premise inventory and exporter — Medium; exit: valid registry/source inputs produce byte-stable envelope premises and invalid inputs fail atomically.
- **A3 = Task-013** authoritative source projection — Hard; exit: artifacts, obligations, symbols, relations, and observation states reproduce their owning records and locators.

### Track B: Parallel after the API stabilizes

- **B1 = Task-014** boundary and compatibility gates — Medium; exit: static dependency and legacy-output regressions fail in CI.

## Parallel Execution Summary

```text
Track A: Task-011 -> Task-012 -> Task-013
Track B:                         Task-014
Final:                           full CI + review gates
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| Task-011 | A | FR-067 | TC-1084, TC-1087, TC-1089 | completed |
| Task-012 | A | FR-067, NFR-006 | TC-1085, TC-1086, TC-1088 | completed |
| Task-013 | A | FR-068, StR-007 | TC-1091..TC-1098 | completed |
| Task-014 | B | FR-067, FR-068, IT-001 | TC-1090, TC-1099 | completed |

## Coordination Rules

- Treat `assurance-v1.schema.json` and its golden fixture as immutable once Task-011 passes; a breaking correction mints v2.
- Keep `Registry`, `Spec`, obligation, and symbol changes limited to read-only accessors required by the exporter.
- Do not add a CLI, Git invocation, network access, persistence, freshness verdict, or Quoin policy.
- Run the full repository CI after Task-014; no Python-binding gate is required unless implementation touches `src/grammar`, `src/python`, or `tests/python`.
