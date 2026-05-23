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

Both `render(block_type, data)` and `parse_document(markdown)` SHALL be deterministic: given identical inputs, they SHALL produce byte-identical outputs across runs, across threads, across machines, and across rebuilds.

Sources of non-determinism that SHALL NOT appear in the implementation:

- HashMap iteration order (use `BTreeMap` when iteration order matters; or sort before serializing)
- `std::time` reads
- `std::process` IDs
- Random number generation outside explicit `?rand` callers
- Floating-point operations that depend on FPU state

Rendered markdown SHALL be a pure function of (block_type, data, registered templates). Parsed `QuireDocument` SHALL be a pure function of input.

## Rationale

The parity suite (FR-012) compares byte streams. Any non-determinism makes the parity suite flaky. Beyond CI, downstream consumers depend on rendered markdown landing in git diffs in a stable order across machines.

## Acceptance Criteria

- **NFR-006-AC-1**: A proptest renders the same archetype with the same input 100 times across threads; all outputs are byte-identical.
- **NFR-006-AC-2**: A proptest parses the same document 100 times; all returned `QuireDocument` values are equal (Eq).
- **NFR-006-AC-3**: A static check via clippy lint or audit confirms no `std::collections::HashMap` usage in render or parse paths where iteration order is observable.

## Verification

- Proptest suite in `tests/determinism.rs`.
- CI runs the suite with `PROPTEST_CASES=512`.
