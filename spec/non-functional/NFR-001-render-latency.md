---
id: NFR-001
title: "Render Latency: Sub-Millisecond Per Archetype"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-004"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-001"
    type: "depends_on"
    cardinality: "1:1"
---

> **RETIRED (render removal — 2026-06-04):** The render/templating feature is
> **removed** from `quire-rs` (no backward-compatibility layer). This NFR (render
> latency, sub-millisecond per archetype) is **retired** along with the render path
> ([FR-001](../functional/FR-001-render-dispatch.md)) and its user story ([US-004](../usecase/US-004-filament-editor-rerender.md)). The perf gate (G3 / task 014) now covers
> parse / validate / `validate_document` / extract latency only — see [NFR-002](./NFR-002-parse-latency.md)
> (parse) and [NFR-002-AC-4](./NFR-002-parse-latency.md) (`validate_document`). This document is kept for history
> and traceability only; its acceptance criteria are dropped from the
> required-coverage tally. The retirement is recorded in `spec.md` §2bis.

## Statement

`quire_rs::render(archetype, data)` for an artifact of typical size SHALL complete in under **1 ms median** on the canonical baseline runner (Apple Silicon M2 Pro, release build, long-lived `Environment` already constructed per [FR-004](../functional/FR-004-minijinja-strict-environment.md)).

"Typical size" means: the artifact's typed `data` serialized as JSON is under 8 KB, the rendered markdown output is under 32 KB, and the template has no `{% for %}` loop iterating more than 100 elements.

For artifacts outside the typical envelope (large embedded blocks, hundreds of relationships), latency targets are linear in input size — no quadratic blowup.

### Cross-runner policy

CI runs on Ubuntu x86_64 (per `.github/workflows/ci.yml`). Per-runner baselines are stored separately:

| Runner | Baseline file | Target |
|---|---|---|
| Apple Silicon M2 Pro (canonical) | `target/criterion/render/*/base/estimates.json` | Median < 1 ms |
| Ubuntu x86_64 (CI) | `target/criterion/render/*/ci-x86_64/estimates.json` | Median within +50% of canonical (i.e. < 1.5 ms) |

The +50% allowance reflects expected single-core perf gap. Other M-class models (M1, M3, M4) may run faster but SHOULD NOT regress against M2 Pro baseline.

A criterion regression test compares the latest measurement against the same-runner stored baseline; a >10% slowdown fails CI.

## Rationale

The Filament editor ([US-004](../usecase/US-004-filament-editor-rerender.md)) re-renders on every accepted patch. To stay below a frame budget (16 ms) with room for spare, the render alone must be well under a millisecond. Sub-ms also makes bulk re-rendering of hundreds of artifacts tractable for batch jobs.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-001-AC-1 | (RETIRED) A criterion benchmark `bench_render_fr` measures `render("fr", baseline_value)` and reports median below 1 ms. | Analysis |
| NFR-001-AC-2 | (RETIRED) A criterion benchmark for each of the 10 archetypes reports median below 1 ms. | Analysis |
| NFR-001-AC-3 | (RETIRED) A regression test compares the latest measurement against a stored baseline; a >10% slowdown fails CI. | Test |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| `render(archetype, data)` latency (typical artifact, canonical M2 Pro) | < 1 ms median | 1 ms median | Criterion Benchmark |
| Render latency on Ubuntu x86_64 CI runner | within +50% of canonical | < 1.5 ms median | Criterion Benchmark |
| Same-runner regression vs stored baseline | no slowdown | < 10% slowdown | Criterion Benchmark |

## Verification

- Criterion benches in `benches/render.rs` execute on every PR with a stored baseline.
- `make bench-render` runs the suite locally; results stored in `target/criterion/`.
