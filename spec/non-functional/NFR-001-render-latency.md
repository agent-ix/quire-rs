---
id: NFR-001
title: "Render Latency: Sub-Millisecond Per Archetype"
artifact_type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-001"
    type: "requires"
    cardinality: "1:1"
---

## Statement

`quire_rs::render(block_type, data)` for an artifact of typical size SHALL complete in under **1 ms median** on a baseline Apple Silicon M-class CPU running a release build, with the long-lived `Environment` already constructed (see FR-004).

"Typical size" means: the artifact's typed `data` serialized as JSON is under 8 KB, the rendered markdown output is under 32 KB, and the template has no `{% for %}` loop iterating more than 100 elements.

For artifacts outside the typical envelope (large embedded blocks, hundreds of relationships), latency targets are linear in input size — no quadratic blowup.

## Rationale

The Filament editor (US-004) re-renders on every accepted patch. To stay below a frame budget (16 ms) with room for spare, the render alone must be well under a millisecond. Sub-ms also makes bulk re-rendering of hundreds of artifacts tractable for batch jobs.

## Acceptance Criteria

- **NFR-001-AC-1**: A criterion benchmark `bench_render_fr` measures `render("fr", baseline_value)` and reports median below 1 ms.
- **NFR-001-AC-2**: A criterion benchmark for each of the 10 archetypes reports median below 1 ms.
- **NFR-001-AC-3**: A regression test compares the latest measurement against a stored baseline; a >10% slowdown fails CI.

## Verification

- Criterion benches in `benches/render.rs` execute on every PR with a stored baseline.
- `make bench-render` runs the suite locally; results stored in `target/criterion/`.
