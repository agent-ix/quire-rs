---
id: StR-003
title: "Byte-Parity Parser with agent-ix/quire (TypeScript)"
type: StR
relationships:
  - target: "ix://agent-ix/quire"
    type: "implements"
    cardinality: "1:1"
---

## Stakeholder Need

`agent-ix/quire` (TypeScript) is the reference markdown parser used by the Filament document stack. The old `agent-ix/quire-py` port is archived; quire-rs is the canonical implementation consumed by Python through the `quire` wheel.

`quire-rs` SHALL match the TypeScript reference at the level of test fixtures. Specifically, given identical markdown input, both implementations SHALL produce equivalent:

- Frontmatter parse (or `None` on malformed YAML)
- Section tree (same hierarchy, same headings, same content slices)
- Slug-line IDs (same format, same line numbers)
- Behavior on edge cases (fenced-block headings, unclosed fences, level skips)

A divergence is a bug; parser changes land in quire-rs and the TypeScript reference through an explicit coordinated change.

## Rationale

This need exists because `agent-ix/quire` (TypeScript) is the reference markdown parser for the Filament document stack, and `quire-rs` is now the canonical implementation consumed by Python through the `quire` wheel (the old `quire-py` port is archived). If the Rust parser diverged from the TypeScript reference on frontmatter handling, section-tree hierarchy, slug-line IDs, or edge cases such as fenced-block headings, unclosed fences, and level skips, every downstream consumer would silently inherit two incompatible notions of "what this document parses to." Treating any divergence as a bug — with parser changes coordinated explicitly across both implementations — is what keeps the two surfaces interchangeable.

## Validation Criteria

This need is considered satisfied when the TS reference test suite in `~/dev/quire/tests/` is transliterated to Rust under `~/dev/quire-rs/tests/parser_parity/` so every TS test has a Rust counterpart, and all transliterated tests pass. Satisfaction is further judged by a property test that takes a corpus of real markdown documents (sampled from `spec-artifacts-*` and `spec-objects-business`) and asserts the `quire-rs` parse output is deterministic and structurally equivalent to the TypeScript fixture contract.

## Priority

Must-Have

## Acceptance

- **StR-003-AC-1**: The TS reference test suite in `~/dev/quire/tests/` is transliterated to Rust and lives in `~/dev/quire-rs/tests/parser_parity/`. Every TS test has a Rust counterpart.
- **StR-003-AC-2**: All transliterated tests pass.
- **StR-003-AC-3**: A property test takes a corpus of real markdown documents (sampled from `spec-artifacts-*` and `spec-objects-business`) and asserts quire-rs parse output is deterministic and structurally equivalent to the TypeScript fixture contract.
