---
id: NFR-007
title: "Archetype Load Cost Amortized Across Renders"
artifact_type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-001"
    type: "requires"
    cardinality: "1:1"
---

## Statement

Archetype compilation (JSON Schema → validator, template source → parsed `minijinja::Template`) SHALL happen once at `Registry::load_from(...)` time. Per-call operations (`render`, `apply_patch`, `extract`) SHALL NOT re-read disk and SHALL NOT re-parse schemas or templates.

Targets:

- **Load cost (cold)**: `Registry::load_from(...)` for the v1 baseline corpus (17 artifact archetypes + 87 object archetypes across all spec-objects-* modules) completes in **under 100 ms median** on baseline Apple Silicon M-class hardware.
- **Per-render cost (warm)**: After load, `render` median is under 1 ms (NFR-001).
- **Per-extract cost (warm)**: After load, `extract` median against a 32 KB document is under 5 ms for typical multi-yield DSLs.
- **No hidden re-compile**: `tracing` instrumentation confirms zero `Template::parse` or `JSONSchema::compile` calls during per-call operations.

## Rationale

The engine must be cheap to use in hot paths (Filament editor live-preview, bulk batch jobs). Up-front compilation is the standard pattern for high-perf template/validation engines; the cost is paid once per process and amortized over many calls. Per-call disk I/O or schema re-parsing would defeat the entire perf model.

## Acceptance Criteria

- **NFR-007-AC-1**: A criterion bench `bench_registry_load` measures cold load of the v1 baseline corpus and reports median under 100 ms.
- **NFR-007-AC-2**: A criterion bench `bench_render_after_load` measures 10000 sequential renders against a pre-loaded registry and reports per-call median under 1 ms with no I/O or compile events captured.
- **NFR-007-AC-3**: A test instrumented via `tracing` asserts zero `Template::parse` and zero `JSONSchema::compile` calls during a `render` operation.
- **NFR-007-AC-4**: Holding a `Registry` reference across many `render` calls produces no monotonically growing memory footprint (verified by a long-running soak test).

## Verification

- Criterion benches in `benches/load.rs` execute on every PR with stored baselines.
- Tracing-based assertions in `tests/no_hidden_recompile.rs`.
