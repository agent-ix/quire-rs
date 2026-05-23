---
id: StR-003
title: "Byte-Parity Parser with agent-ix/quire (TypeScript) and quire-py (Python)"
artifact_type: StR
relationships:
  - target: "ix://agent-ix/quire"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-py"
    type: "implements"
    cardinality: "1:1"
---

## Stakeholder Need

`agent-ix/quire` (TypeScript) is the reference markdown parser used by the Filament document stack. `agent-ix/quire-py` is its Python port. Both ship the same acceptance test fixtures and produce identical `QuireDocument` shapes for identical input.

`quire-rs` SHALL be a third sibling — a Rust port whose output structure matches both references at the level of test fixtures. Specifically, given identical markdown input, all three implementations SHALL produce equivalent:

- Frontmatter parse (or `None` on malformed YAML)
- Section tree (same hierarchy, same headings, same content slices)
- Slug-line IDs (same format, same line numbers)
- Behavior on edge cases (fenced-block headings, unclosed fences, level skips)

A divergence in any direction is a bug; coordination across all three implementations is required to ship parser changes.

## Priority

Must-Have

## Acceptance

- **StR-003-AC-1**: The TS reference test suite in `~/dev/quire/tests/` is transliterated to Rust and lives in `~/dev/quire-rs/tests/parser_parity/`. Every TS test has a Rust counterpart.
- **StR-003-AC-2**: All transliterated tests pass.
- **StR-003-AC-3**: A property test takes a corpus of real markdown documents (sampled from `spec-artifacts-*` and `ix-spec-objects`) and asserts: for every document, `quire-rs::parse(doc)` and `quire-py::parse_document(doc)` produce structurally equivalent `QuireDocument` values.
