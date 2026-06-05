---
id: NFR-006
title: "Determinism: Identical Input Produces Byte-Identical Output"
artifact_type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-003"
    type: "implements"
    cardinality: "1:1"
---

## Statement

`parse_document(markdown)`, `validate_document(archetype, text)`, and `extract(doc, dsl)` SHALL be deterministic: given identical inputs, they SHALL produce byte-identical outputs (parsed document, ordered diagnostics, extracted records + edges) across runs, across threads, across machines, and across rebuilds. (Render determinism is retired with the render feature.)

Sources of non-determinism that SHALL NOT appear in the implementation:

- HashMap iteration order (use `BTreeMap` when iteration order matters; or sort before serializing)
- `std::time` reads
- `std::process` IDs
- Random number generation outside explicit `?rand` callers
- Floating-point operations that depend on FPU state

A parsed `QuireDocument`, a `validate_document` `ValidationResult` (including its
ordered diagnostics), and an `extract` `ExtractionResult` (records + edges +
diagnostics) SHALL each be a pure function of their inputs.

## Rationale

The parity suite (FR-012) compares byte streams. Any non-determinism makes the parity suite flaky. Beyond CI, downstream consumers depend on rendered markdown landing in git diffs in a stable order across machines.

## Acceptance Criteria

- NFR-006-AC-1 — **RETIRED (render removal — 2026-06-04):** formerly asserted render determinism across 100 threaded runs. Render is removed; this criterion is dropped from the required-coverage tally (id retained, immutable). Determinism of the retained surfaces is covered by NFR-006-AC-2/AC-4.
- **NFR-006-AC-2**: A proptest parses the same document 100 times; all returned `QuireDocument` values are equal (Eq).
- **NFR-006-AC-3**: A static check via clippy lint or audit confirms no `std::collections::HashMap` usage in parse / extract / validate paths where iteration order is observable (the audit path covers `src/parser`, `src/extract`, `src/validate_document.rs`, `src/merge.rs`).
- **NFR-006-AC-4**: A proptest runs `validate_document` and `extract` on the same input 100 times across threads; the `ValidationResult` (including ordered diagnostics) and `ExtractionResult` (records + edges + diagnostics) are equal every time.

## Verification

- Proptest suite in `tests/determinism.rs`.
- CI runs the suite with `PROPTEST_CASES=512`.
