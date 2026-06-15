---
id: StR-001
title: "Single Generic Rust Engine for Render + Parse + Extract"
artifact_type: StR
---

## Stakeholder Need

Today the ecosystem fragments document-processing across three languages:

- **Rendering** lives in `agent-ix/spec-artifacts-iso`, `spec-artifacts-app`, `spec-artifacts-process` (Python + Jinja2)
- **Parsing** lived in `agent-ix/quire` (TypeScript) with a historical `quire-py` Python port now superseded by this engine
- **Body extraction** lived in `agent-ix/filament-parser-lib` (Python tier-1/2/3 pipeline)

Consumers (Filament editor, batch extractors, future CLI tools) coordinate two interpreters or wire bindings across languages. Hot paths (re-render on patch, bulk extraction across hundreds of objects) pay interpreter and IPC overhead. Worse, behavior drifts subtly across implementations.

`quire-rs` SHALL be the single Rust engine that owns render, parse, extract, frontmatter splitting, edge harvesting, and schema validation. Python consumers may orchestrate the pipeline through the PyO3 wheel, but they SHALL NOT maintain parallel parser, extractor, or validator implementations for these hot paths. Critically, the engine SHALL be **generic over archetypes**: it has no `FR`-specific or `NFR`-specific Rust code. Archetypes are loaded as data (JSON Schema + MiniJinja template + manifest) from the local filesystem at startup. Filament is the authoring environment for those archetypes; ix-cli syncs them to disk; quire-rs reads from disk. No part of the engine talks to Filament directly.

This decoupling has three pay-offs:

1. Adding a new archetype is a data-only change — no Rust release needed.
2. The engine works offline (no Filament reachability required).
3. The same engine serves Filament-synced corpora, hand-authored corpora, and test fixtures interchangeably.

## Priority

Must-Have

## Acceptance

- **StR-001-AC-1**: A single `cargo add quire-rs` is sufficient to obtain render, parse, and extract APIs in a downstream Rust crate.
- **StR-001-AC-2**: No call site in `quire-rs` shells out to Python, Node, or any other interpreter.
- **StR-001-AC-3**: `Cargo.lock` audit confirms no HTTP/RPC client crates (`reqwest`, `hyper`, `tonic`, `grpc-*`, etc.) — engine is filesystem-only.
- **StR-001-AC-4**: Adding a brand-new archetype kind (new JSON Schema + new template) to `~/.ix/filament/modules/<some-module>/` and restarting the consumer is sufficient for `Registry::archetype("new-name")` to return Some, with no `quire-rs` source change.
- **StR-001-AC-5**: A regression suite runs `quire-rs` with `IX_SCHEMA_PATH` pointing at three different on-disk corpora (Filament-synced, hand-authored, test fixture) and confirms identical behavior across all three.
