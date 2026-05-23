---
id: US-002
title: "Developer Parses a Markdown Spec Doc into a Typed Heading Tree"
artifact_type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-003"
    type: "implements"
    cardinality: "1:1"
---

## Story

As a **Rust developer integrating Filament**, I want to call `quire_rs::parse_document(markdown)` and receive a `QuireDocument` whose structure exactly matches what `agent-ix/quire-py::parse_document(markdown)` returns, so that I can swap from the Python tier-2 extractor to the Rust one with no behavioral surprises.

## Context

`agent-ix/filament-parser-lib` currently parses Filament documents using Python tier-1/2/3 logic. Hot paths (bulk re-extraction, editor live preview) suffer from interpreter overhead. The migration target is `quire-rs::parse_document` exposed via a thin FFI layer, but the migration only works if behavior matches at the test-fixture level.

## Acceptance

- **US-002-AC-1**: A developer can write `use quire_rs::parse_document;` and call `let doc = parse_document(markdown)?;` with no other setup.
- **US-002-AC-2**: For every test fixture in the TS/Py reference suites, the Rust output's frontmatter, preamble, section tree, and slug-line IDs structurally match.
- **US-002-AC-3**: The function is pure — no IO, no mutable global state, no panics on malformed input.
