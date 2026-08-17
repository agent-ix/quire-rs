---
id: NFR-019
title: "Input Robustness: No Panic on Arbitrary Input"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "depends_on"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "depends_on"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-027"
    type: "depends_on"
    cardinality: "1:1"
---

## Statement

The retained query/validate/extract surfaces — `parse_document`,
`validate_document`, `extract`, and the whole-spec query API ([FR-027](../functional/FR-027-whole-spec-query-api.md)) — SHALL
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

`quire-rs` is consumed in-process through the PyO3 bindings ([FR-023](../functional/FR-023-python-binding-surface.md)) by long-lived
services (filament-core, analysis workers). A panic there aborts the host process or
poisons the GIL. The render path that previously carried template-error panics is
gone; the remaining surfaces take **untrusted document text** directly and must
degrade to typed errors, not crashes.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-019-AC-1 | A `cargo-fuzz` target feeds arbitrary byte slices (as lossy `&str`) into `parse_document`, `validate_document`, and `extract` and runs clean (no panic / no UB) for the scheduled fuzz duration; any discovered crash is committed as a regression reproducer (parity with [NFR-011-AC-4](./NFR-011-fuzz-testing.md)). | Test |
| NFR-019-AC-2 | A proptest generates random strings (including empty, fence-only, frontmatter-only, and deeply nested heading inputs) and asserts `parse_document`, `validate_document`, and `extract` each return a value or typed error without panicking across all generated cases. | Test |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Panics / UB from `parse_document`/`validate_document`/`extract` on arbitrary bytes | 0 | 0 | fuzzing (cargo-fuzz, scheduled lane) |
| property-based-testing (proptest) cases returning value-or-typed-error without panic | 100% | 100% | property-based-testing (proptest, `PROPTEST_CASES=512`) |
| Discovered crash committed as regression reproducer | Pass | Pass | Inspection |

## Verification

- Fuzz target under `fuzz/fuzz_targets/` on the scheduled fuzz lane ([NFR-011](./NFR-011-fuzz-testing.md)).
- Proptest in `tests/robustness.rs` run on every PR (`PROPTEST_CASES=512`).
