---
id: NFR-007
title: "Archetype Load Cost Amortized Across Renders"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "depends_on"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-001"
    type: "depends_on"
    cardinality: "1:1"
---

## Statement

Archetype compilation (JSON Schema → validator, template source → parsed `minijinja::Template`) SHALL happen once at `Registry::load_from(...)` time. Per-call operations (`render`, `apply_patch`, `extract`) SHALL NOT re-read disk and SHALL NOT re-parse schemas or templates.

Targets:

- **Load cost (cold)**: `Registry::load_from(...)` for the v1 baseline corpus (17 artifact archetypes + 87 object archetypes across all spec-objects-* modules) completes in **under 100 ms median** on the canonical baseline runner (Apple Silicon M2 Pro). Ubuntu x86_64 CI allowance: +50% (< 150 ms).
- **Per-render cost (warm)**: After load, `render` median is under 1 ms ([NFR-001](./NFR-001-render-latency.md)).
- **Per-extract cost (warm)**: After load, `extract` median against a 32 KB document is under 5 ms for typical multi-yield DSLs.
- **No hidden re-compile**: `tracing` instrumentation confirms zero `Template::parse` or `JSONSchema::compile` calls during per-call operations.

## Rationale

The engine must be cheap to use in hot paths (Filament editor live-preview, bulk batch jobs). Up-front compilation is the standard pattern for high-perf template/validation engines; the cost is paid once per process and amortized over many calls. Per-call disk I/O or schema re-parsing would defeat the entire perf model.

## Change Records

> **CR (2026-11-27):** Updated the `bench_registry_load` corpus reference to the current pinned 28-artifact snapshot documented in FR-013-AC-4. The broader historical v1 baseline statement in the target section includes 87 object archetypes and is not represented by this artifact-only benchmark; this criterion's explicitly named artifact corpus now has a reproducible source and count. The Ubuntu host run measured 93.217 ms median (89.657–96.789 ms interval), below this criterion's 100 ms target.

## Acceptance Criteria

> **CR note (2026-11-26):** The render and template runtime was removed with
> NFR-001's render path. These retained rows have no operation to instrument
> or soak in this repository and are not verification obligations.

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-007-AC-1 | A criterion bench `bench_registry_load` measures cold load of the pinned baseline artifact corpus (28 artifact types; provenance and fixture at `tests/fixtures/modules/baseline-artifacts/`) and reports median under 100 ms. | Analysis |
| NFR-007-AC-2 | A criterion bench `bench_render_after_load` measures 10000 sequential renders against a pre-loaded registry and reports per-call median under 1 ms with no I/O or compile events captured. | Analysis |
| NFR-007-AC-3 | RETIRED: A test instrumented via `tracing` asserts zero `Template::parse` and zero `JSONSchema::compile` calls during a `render` operation. | None |
| NFR-007-AC-4 | RETIRED: Holding a `Registry` reference across many `render` calls produces no monotonically growing memory footprint (verified by a long-running soak test). | None |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| `Registry::load_from` cold load, v1 baseline corpus (canonical M2 Pro) | < 100 ms median | 100 ms median | performance-benchmarking (Criterion) |
| Cold load on Ubuntu x86_64 CI runner | within +50% of canonical | < 150 ms median | performance-benchmarking (Criterion) |
| Warm per-extract latency (32 KB doc) | < 5 ms median | 5 ms median | performance-benchmarking (Criterion) |
| `Template::parse` / `JSONSchema::compile` calls during per-call ops | 0 | 0 | runtime-monitoring (tracing) |

## Verification

- Criterion benches in `benches/load.rs` execute on every PR with stored baselines.
- Tracing-based assertions in `tests/it/no_hidden_recompile.rs`.
