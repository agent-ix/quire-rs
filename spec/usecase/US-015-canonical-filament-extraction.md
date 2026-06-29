---
id: US-015
title: "Consume canonical Filament extraction from one engine"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "traces_to"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-005"
    type: "traces_to"
---
# [US-015] Consume canonical Filament extraction from one engine

## Story

**As a** Filament sync or IDE runtime maintainer
**I want** one canonical engine to extract document objects, graph edges, and diagnostics
**So that** Python, WASM, and Electron consumers produce the same core data without sidecars or duplicated policy.

## Context

Filament extraction policy currently spans `filament-parser-lib`, the `quire` Python
binding, and the published `@agent-ix/quire-wasm` package. The cutover is valuable only
if the canonical policy lives in `quire-rs` and the bindings are thin wrappers over the
same Rust implementation.

## Acceptance Examples (Illustrative)

### [US-015-EX-1] IDE worker extracts without Python

- **Given** an IDE worker receives markdown plus ObjectType snapshots
- **When** it extracts core data
- **Then** the worker calls the `quire-wasm` binding in-process and receives the same graph/object payload shape as Python consumers

### [US-015-EX-2] Parser library delegates without drift

- **Given** `filament-parser-lib` receives the same markdown and ObjectType snapshots
- **When** it calls the Python `quire` binding
- **Then** the returned core extraction payload matches the WASM binding for the same fixture

## Dependencies (Contextual)

Upstream requirements include [FR-011](../functional/FR-011-body-extraction-dsl.md),
[FR-026](../functional/FR-026-intra-spec-reference-resolution.md), and the Python binding
surface in [FR-028](../functional/FR-028-expanded-python-binding-surface.md).

## Priority and Risk (Informative)

Priority is high because the IDE Python bridge and parser-lib compatibility shim depend
on this behavior. Risk is high if unmet because duplicated extraction behavior will
continue to drift across runtimes.

## Traceability (Informative)

This story drives [FR-040](../functional/FR-040-filament-core-extraction-engine.md) and
[FR-041](../functional/FR-041-filament-extraction-bindings.md).
