---
id: NFR-013
title: "Behavioral mutation testing on high-value code paths"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/FR-005"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/FR-011"
    type: "traces_to"
    cardinality: "1:1"
---

# NFR-013: Behavioral mutation testing on high-value code paths

## Statement

When the local mutation-verification lane runs, quire-rs SHALL execute the relevant tests against each generated viable mutation in its declared high-value paths and retain the observed outcomes.

## Scope

- Applies to: parser primitives (`src/parser/`), the DSL evaluator (`src/extract/`), and the edge harvester.
- Operational context: an explicitly invoked local pre-release verification run; this requirement does not require a CI schedule.
- Not applied to: mutations that cannot compile or cannot be executed; the retained results identify these as unviable rather than as killed.

## Rationale

Mutation testing modifies source code and observes whether tests distinguish the changed behavior. A compile-only check does not execute tests against mutations, and a swallowed command failure can make an incomplete run look successful. Retaining every outcome lets a maintainer review survivors and the executed scope without treating an absent result as a clean measurement.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Generated viable mutations with an observed test outcome | all | all | Demonstration |
| Killed, surviving and unviable outcomes absent from the retained result | 0 | 0 | Inspection |
| Surviving mutations without a human review disposition | 0 | 0 | Inspection |

## Verification

The local mutation target performs a real mutation run: each generated viable mutation is tested, and the retained result identifies the target path and each mutation's killed or surviving outcome; unviable mutations are reported separately. A compile-only check is not a completed mutation run. The result preserves the command's actual exit status, including failures during mutation or test execution. Every survivor remains visible even if a baseline or prior run names it; a person records the disposition, while the tool assigns no acceptance or rejection.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-013-AC-1 | The mutation configuration names the parser, DSL evaluator, and edge-harvester source paths whose outcomes this requirement measures. | Inspection |
| NFR-013-AC-2 | The local mutation target delegates to cargo-mutants in behavioral mode over the declared high-value paths; it does not enable compile-only mode. | Demonstration |
| NFR-013-AC-3 | The retained result contains the executed target scope and every mutation's killed or surviving outcome, with unviable mutations reported separately. | Inspection |
| NFR-013-AC-4 | A survivor remains present in the retained result and has a recorded human review disposition; a baseline does not suppress it or make the tool decide acceptance. | Inspection |
| NFR-013-AC-5 | A non-zero cargo-mutants result propagates as a non-zero local target result, and CI retains the mutation output even when the command fails. | Demonstration |

## Change Records

### CR-001 — PLAT-1818 real mutation outcomes

The previous target used `--check`, which compiled mutations without running tests, and its workflow used `|| true`, which suppressed failures. This contradicted the requirement's stated purpose of measuring behavioral discrimination. PLAT-1818 changes this NFR to require a local real run with retained killed/surviving/unviable results and human review of each survivor. The run remains an input to human verification; no threshold or acceptability decision is assigned by the tool.

### CR-002 — PLAT-1818 command and failure-path checks

AC-2 and AC-5 are demonstrated by focused tests of the local target's command
delegation and failure propagation. These checks do not claim that a real
mutation run completed; the real run remains backlogged until the baseline
integration failure recorded with PLAT-1818 is resolved.

## Dependencies

- **Upstream**: [FR-005](../functional/FR-005-parse-document-api.md), [FR-011](../functional/FR-011-body-extraction-dsl.md).
- **Downstream**: none.
