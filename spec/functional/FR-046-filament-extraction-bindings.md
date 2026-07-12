---
id: FR-046
title: "Expose Filament extraction through Python and WASM"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-016"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-045"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-023"
    type: "extends"
---
# [FR-046] Expose Filament extraction through Python and WASM

## Description

The `quire-rs` distribution SHALL expose the canonical Filament extraction engine through
both the Python `quire` wheel and the `@agent-ix/quire-wasm` package, with both bindings
delegating to the same Rust implementation.

## Inputs

- The same JSON-serializable one-document extraction input defined by [FR-045](./FR-045-filament-core-extraction-engine.md)

## Outputs

- The same JSON-serializable `CoreExtractionResult`-compatible output defined by
  [FR-045](./FR-045-filament-core-extraction-engine.md)

## Behavior

- The Python binding SHALL accept native Python dict/list/scalar values and return native
  Python dict/list/scalar values without spawning a subprocess.
- The WASM binding SHALL accept JavaScript objects and return JavaScript objects with the
  same field names and values as the Python binding for the same fixture input.
- The `@agent-ix/quire-wasm` package SHALL continue to export the existing
  `parseDocument`, `extractFromBlob`, and `validateFromBlob` surfaces.
- Binding layers SHALL NOT contain independent extraction policy.
- Binding layers SHALL translate inputs and outputs only.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-046-CON-1 | The Python binding SHALL NOT use `subprocess`, sockets, or JSON-over-stdio for this extraction path. | Architecture | Inspection |
| FR-046-CON-2 | The WASM binding SHALL be loadable by Electron worker code without requiring a Python runtime or local sidecar process. | Architecture | Test |
| FR-046-CON-3 | TypeScript declarations for `@agent-ix/quire-wasm` SHALL include the new Filament extraction export. | Compatibility | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-046-AC-1 | Python and WASM bindings return equivalent JSON for Tier 1, Tier 2, relationship, malformed-link, and extraction-error fixtures. | Test (TC-686) |
| FR-046-AC-2 | `@agent-ix/quire-wasm` exports the new extraction API and updated `.d.ts` declarations while existing `parseDocument`, `extractFromBlob`, and `validateFromBlob` smoke tests continue to pass. | Test (TC-687) |
| FR-046-AC-3 | Static inspection finds no extraction-policy branches in binding code beyond input/output conversion and error mapping. | Inspection (TC-688) |
| FR-046-AC-4 | A default Rust build without the Python feature remains free of Python/CPython linkage, and a WASM target check succeeds with filesystem-free features. | Test (TC-689) |

## Dependencies

- **Upstream**: [FR-045](./FR-045-filament-core-extraction-engine.md), [FR-023](./FR-023-python-binding-surface.md)
- **Downstream**: `filament-parser-lib` [FR-118](ix://agent-ix/filament-parser-lib/FR-118) and Filament IDE [FR-046](ix://agent-ix/filament-ide/FR-046)
