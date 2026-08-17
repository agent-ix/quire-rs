---
id: NFR-013
title: "Mutation Testing on High-Value Code Paths"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "traces_to"
    cardinality: "1:1"
---

## Statement

`quire-rs` SHALL run `cargo-mutants` on a CI schedule (weekly + workflow_dispatch) targeted at two high-value code paths:

1. **Parser primitives** (`src/parser/`) — [FR-005](../functional/FR-005-parse-document-api.md)..010
2. **DSL evaluator** (`src/extract/`) — [FR-011](../functional/FR-011-body-extraction-dsl.md), [FR-016](../functional/FR-016-secondary-locators.md)

Mutation testing modifies source code (e.g. changes `>` to `<`, drops a `return`, replaces a literal) and asserts that the test suite catches the change. Surviving mutants indicate tests that exercise code but don't actually verify behavior.

### Operational policy

- CI workflow (`mutants.yml` or as a job in `ci.yml`) runs weekly + workflow_dispatch.
- Reports a "caught / missed / unviable" summary per target path.
- A surviving mutant is NOT a build failure; it's a signal. The PR/maintainer decides whether the surviving mutant represents a real test gap or an irrelevant mutation.
- A `mutants_baseline.txt` tracks acceptable survivors with rationale.

### Coverage targets

- **Parser**: aim for >95% mutants caught
- **DSL evaluator**: aim for >90% (some branches are tested at integration level only)
- **Edge harvester**: aim for >90%

These are targets, not gates; the actual percentage is recorded in CI artifacts.

## Rationale

Coverage metrics measure code exercised; mutation testing measures behavior verified. A function with 100% coverage but a missing `assert!` will fail mutation testing. This is the highest-signal test-quality measurement we have.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-013-AC-1 | `Cargo.toml` has a `[package.metadata.mutants]` config (or `.cargo/mutants.toml`) declaring the three target paths. | Inspection |
| NFR-013-AC-2 | A CI workflow runs `cargo mutants -p quire-rs --in-place --check` on weekly schedule + workflow_dispatch. | Inspection |
| NFR-013-AC-3 | The report is uploaded as a CI artifact (`mutants.json` or similar). | Demonstration |
| NFR-013-AC-4 | `mutants_baseline.txt` tracks accepted survivors with one-line rationale per entry. | Inspection |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Mutants caught in parser primitives (`src/parser/`) | > 95% | > 95% (target, not gate) | mutation-testing (cargo-mutants, scheduled) |
| Mutants caught in DSL evaluator (`src/extract/`) | > 90% | > 90% (target, not gate) | mutation-testing (cargo-mutants, scheduled) |
| Mutants caught in edge harvester | > 90% | > 90% (target, not gate) | mutation-testing (cargo-mutants, scheduled) |
| Surviving mutants tracked in `mutants_baseline.txt` with rationale | Pass | Pass | Inspection |

## Verification

- CI workflow visible; passing run on schedule.
- `make mutants` local target.
