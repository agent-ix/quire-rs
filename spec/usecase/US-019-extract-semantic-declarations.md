---
id: US-019
title: "Extract semantic declarations from an object artifact"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "traces_to"
    cardinality: "1:1"
---
# US-019: Extract semantic declarations from an object artifact

## Story

**As a** domain-package compiler frontend lifting a spec bundle into IR
**I want** Quire to extract an object artifact's typed Properties, invariants, and operations as semantic-core declarations with source spans
**So that** I can build the package from one validated extraction instead of re-parsing Markdown or guessing what an unresolved type token means.

## Context

`agent-ix/quoin#293` fixed the Markdown mapping (typed `## Properties` table
or a `sysml` fence to `FieldDecl[]`; `## Invariants` and `## Operations` to
`ClauseRef[]` and `OperationDecl[]`) and the module `semantic` block that
names the semantic-core version and the emitted JSON Schema by digest. Quire
already extracts the `Properties` section as one string (`section_body`).
This story adds the declaration-level extraction under that contract while
keeping Quire an extraction engine: fence bodies are carried verbatim, never
parsed, and nothing is rendered.

## Acceptance Examples (Illustrative)

### US-019-EX-1: The FR-006 table extracts to the golden declaration set

- **Given** the read-only config-service `FR-006` copy authored as a typed Properties table under a module whose `semantic` block pins semantic-core `0.1.0`
- **When** the frontend runs Quire extraction over it
- **Then** the record carries the seven `FieldDecl` entries of the `agent-ix/quoin` golden fixture, byte-identical after normalization, plus one `ocl` `ClauseRef` with a source span

### US-019-EX-2: Two authoring forms in one artifact are refused

- **Given** an artifact with both a typed table and a `sysml` fence under `## Properties`
- **When** extraction runs
- **Then** it yields no `fields` and one error at the second form's start line

### US-019-EX-3: An unknown type is a finding, not a string

- **Given** a row whose `Type` cell names nothing in the bundle, its imports, or the kernel scalars
- **When** extraction runs
- **Then** the field records a placeholder identity under `unresolved/`, and an advisory finding names the row line

### US-019-EX-4: An unsupported contract is refused before anything is read

- **Given** a module whose `semantic.semantic_core` names a version Quire does not vendor
- **When** the module loads
- **Then** loading fails naming both versions and no declaration is extracted

## Constraints (Contextual)

Extraction only. Fence content is opaque bytes with a span; type checking of
clauses, rendering, code generation, and package publishing belong to the
compiler (`agent-ix/filament-core-data#36`, `#37`).

## Dependencies (Contextual)

Upstream: `agent-ix/quoin#293` (mapping contract and golden fixtures),
`agent-ix/filament-core-data#35` (semantic-core 0.1.0 grammar), `#34` (IR
v1.1 constraint vocabulary), `agent-ix/filament-core-service#22`
(module-manifest schema with the `semantic` block). Downstream:
`agent-ix/filament-core-data#36`, `agent-ix/spec-objects-business#4`,
`agent-ix/spec-artifacts-iso#34`.

## Priority and Risk (Informative)

P1. The risk is scope creep into parsing clause bodies or rendering; both are
excluded by requirement and guarded by a static boundary test.
