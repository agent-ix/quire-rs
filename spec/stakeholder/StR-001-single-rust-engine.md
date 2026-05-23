---
id: StR-001
title: "Single Rust Crate for Render + Parse in the Filament/Quire Ecosystem"
artifact_type: StR
---

## Stakeholder Need

Today the ecosystem fragments responsibility for canonical spec/document handling across three languages:

- **Rendering** lives in `agent-ix/spec-artifacts-iso` and `agent-ix/spec-artifacts-app` (Python + Jinja2)
- **Parsing** lives in `agent-ix/quire` (TypeScript) and `agent-ix/quire-py` (Python port)
- **Body extraction** lives in `agent-ix/filament-parser-lib` (Python tier-1/2/3 pipeline)

Consumers (Filament editor, future CLI tools, CI parity checks) end up shelling out to two interpreters or wiring up bindings across languages. Performance-sensitive paths (re-render on patch, bulk extraction across hundreds of objects) suffer from interpreter startup and IPC overhead.

`quire-rs` SHALL be a single Rust crate that exposes both rendering (schema + template pair, MiniJinja-based) and parsing (markdown → typed AST) so that downstream consumers can depend on one binary with no language boundaries between the two halves.

## Priority

Must-Have

## Acceptance

- **StR-001-AC-1**: A single `cargo add quire-rs` is sufficient to obtain both render and parse APIs in a downstream Rust crate.
- **StR-001-AC-2**: No call site in `quire-rs` shells out to Python or Node to perform a render or a parse.
- **StR-001-AC-3**: The render and parse APIs are independently usable — a consumer that only renders does not pay parser compile-time or runtime cost, and vice versa, via feature flags.
