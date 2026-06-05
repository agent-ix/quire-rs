---
id: NFR-019
title: "Input Robustness: No Panic on Arbitrary Input"
artifact_type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-027"
    type: "requires"
    cardinality: "1:1"
---

## Statement

The retained query/validate/extract surfaces — `parse_document`,
`validate_document`, `extract`, and the whole-spec query API (FR-027) — SHALL
**never panic** on arbitrary input. Given any `&str` (including non-UTF-8-adjacent
byte sequences passed as lossy text, truncated frontmatter, unbalanced fences,
megabyte-scale degenerate input, or adversarial Unicode) the engine SHALL return a
typed `Result`/`ValidationResult` or a well-formed (possibly empty) value — never
unwind via `panic!`, `unwrap`, `expect`, slice-index out of bounds, or arithmetic
overflow.

This is the engine-wide expression of the §11 failure model ("the library SHALL NOT
panic on malformed input") for the post-render surface, and it is enforced
mechanically by fuzzing rather than asserted only in prose.

## Rationale

`quire-rs` is consumed in-process through the PyO3 bindings (FR-023) by long-lived
services (filament-core, analysis workers). A panic there aborts the host process or
poisons the GIL. The render path that previously carried template-error panics is
gone; the remaining surfaces take **untrusted document text** directly and must
degrade to typed errors, not crashes.

## Acceptance Criteria

- **NFR-019-AC-1**: A `cargo-fuzz` target feeds arbitrary byte slices (as lossy
  `&str`) into `parse_document`, `validate_document`, and `extract` and runs clean
  (no panic / no UB) for the scheduled fuzz duration; any discovered crash is
  committed as a regression reproducer (parity with NFR-011-AC-4).
- **NFR-019-AC-2**: A proptest generates random strings (including empty, fence-only,
  frontmatter-only, and deeply nested heading inputs) and asserts `parse_document`,
  `validate_document`, and `extract` each return a value or typed error without
  panicking across all generated cases.

## Verification

- Fuzz target under `fuzz/fuzz_targets/` on the scheduled fuzz lane (NFR-011).
- Proptest in `tests/robustness.rs` run on every PR (`PROPTEST_CASES=512`).
