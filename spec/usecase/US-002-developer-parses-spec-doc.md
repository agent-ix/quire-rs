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

As a **Rust developer integrating Filament**, I want to call `quire_rs::parse_document(markdown)` and receive a `QuireDocument` whose structure matches the Quire parser fixture contract, so that downstream Python consumers can call the same Rust parser through the `quire` wheel with no behavioral surprises.

## Context

`agent-ix/filament-parser-lib` historically parsed Filament documents using Python tier-1/2/3 logic. Hot paths (bulk re-extraction, editor live preview) suffer from interpreter overhead. The migration target is `quire-rs::parse_document` exposed through the `quire` PyO3 wheel, and no Python parser remains on the hot path.

## Acceptance

- **US-002-AC-1**: A developer can write `use quire_rs::parse_document;` and call `let doc = parse_document(markdown)?;` with no other setup.
- **US-002-AC-2**: For every test fixture in the reference suite, the Rust output's frontmatter, preamble, section tree, and slug-line IDs structurally match.
- **US-002-AC-3**: The function is pure — no IO, no mutable global state, no panics on malformed input.
