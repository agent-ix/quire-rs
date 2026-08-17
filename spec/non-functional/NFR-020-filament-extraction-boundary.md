---
id: NFR-020
title: "Filament extraction boundary remains pure and deterministic"
type: NFR
quality_attribute: maintainability
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-045"
    type: "constrains"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-046"
    type: "constrains"
---
# [NFR-020] Filament extraction boundary remains pure and deterministic

## Statement

The Filament extraction path SHALL remain a pure document-semantics boundary whose
outputs are deterministic for identical markdown and ObjectType snapshot inputs.

## Scope

- Applies to: the canonical Filament extraction module and its Python/WASM bindings.
- Operational context: IDE worker sync, parser-lib compatibility calls, and future sync
  consumers.

## Rationale

The migration removes duplicated policy only if all consumers can trust one engine to
produce identical results without hidden runtime dependencies or side effects.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Runtime side-effect dependencies in extraction module | 0 | 0 | inspection (static) |
| Repeated-output mismatch count for parity fixtures | 0 | 0 | unit-testing |
| Python/WASM binding divergence count for shared fixtures | 0 | 0 | integration-testing |

## Verification

Static inspection and parity tests verify that extraction does not reach into persistence,
IPC, network, CloudManager sync, or embedding concerns, and that repeated runs produce
the same JSON output for the same inputs.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-020-AC-1 | Static inspection finds no PGlite, Electron, HTTP/auth, CloudManager sync, workspace watcher, or embedding dependencies in the extraction module or bindings. | Inspection (TC-690) |
| NFR-020-AC-2 | A deterministic fixture suite runs the same extraction input repeatedly and observes identical output ordering and record ids. | Test (TC-685) |
| NFR-020-AC-3 | Binding parity fixtures compare Python and WASM output as JSON values, not stringified subprocess payloads. | Test (TC-686) |

## Dependencies

- **Upstream**: [FR-045](../functional/FR-045-filament-core-extraction-engine.md), [FR-046](../functional/FR-046-filament-extraction-bindings.md)
- **Downstream**: parser-lib and IDE cutover tasks
