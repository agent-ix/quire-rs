---
id: NFR-011
title: "Fuzz Testing on Untrusted-Input Surfaces"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-002"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "traces_to"
    cardinality: "1:1"
---

## Statement

`quire-rs` SHALL ship `cargo-fuzz` targets for each surface that consumes untrusted input. Fuzz targets run on a nightly toolchain, on a CI schedule (weekly + workflow_dispatch), and find: panics, infinite loops, quadratic blowup, OOM, and UB.

### Required fuzz targets

| Target | Surface | Crash criteria |
|---|---|---|
| `fuzz_parse_document` | `parse_document(input: &str)` ([FR-005](../functional/FR-005-parse-document-api.md)) | any panic, timeout >5s, allocation >100 MB |
| `fuzz_extract_frontmatter` | `extract_frontmatter(input: &str)` ([FR-006](../functional/FR-006-frontmatter-with-fallback.md)) | any panic, timeout >2s |
| `fuzz_apply_patch` | `apply_patch(archetype, current, patch)` ([FR-002](../functional/FR-002-schema-validation-pipeline.md)) | any panic on arbitrary JSON values |
| `fuzz_extract_dsl` | `extract(doc, dsl)` ([FR-011](../functional/FR-011-body-extraction-dsl.md)) | any panic, runaway record emission |
| `fuzz_load_manifest` | Manifest YAML parsing inside [FR-013](../functional/FR-013-archetype-loader.md) | any panic on arbitrary YAML |
| `fuzz_load_schema` | JSON Schema loading inside [FR-013](../functional/FR-013-archetype-loader.md) | any panic on arbitrary JSON Schema documents |

### Operational policy

- Fuzz targets live under `fuzz/fuzz_targets/`.
- A CI workflow (`.github/workflows/fuzz.yml`) runs each target for 5 minutes on a weekly schedule.
- Manual workflow_dispatch with a longer duration (e.g. 1 hour) is supported.
- A discovered crash MUST be filed as an issue with the reproducer corpus.
- Fuzzing is NOT a pre-merge gate (too slow); discovered crashes are P0 bugs to fix.

## Rationale

Parser, schema loader, and DSL evaluator all consume input that may be hostile or malformed. Property tests ([NFR-006](./NFR-006-determinism.md) / proptest) provide some coverage but are bounded by their generators; coverage-guided fuzzing explores edge cases proptest will not.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-011-AC-1 | `fuzz/fuzz_targets/` contains the six targets listed above, each compiling against `cargo +nightly fuzz`. | Inspection |
| NFR-011-AC-2 | Each target runs cleanly for 60 seconds on the canonical baseline runner without producing a crash. | Test |
| NFR-011-AC-3 | `.github/workflows/fuzz.yml` runs all targets on a weekly schedule. | Inspection |
| NFR-011-AC-4 | A documented crash reproducer (when discovered) is committed under `fuzz/corpus/<target>/` with a regression test under `tests/regression/`. | Demonstration |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Required `cargo-fuzz` targets present and compiling | 6 | 6 | Inspection (`fuzz/fuzz_targets/`) |
| Crashes per target in 60s smoke run (canonical runner) | 0 | 0 | fuzzing (cargo-fuzz) |
| Weekly scheduled fuzz lane (5 min/target) green | Pass | Pass | fuzzing (weekly scheduled lane) |
| Discovered crash committed as reproducer + regression test | Pass | Pass | Inspection |

## Verification

- Weekly scheduled CI run with artifacts uploaded.
- `make fuzz` local target invokes each fuzz target in sequence for 60s smoke run.
