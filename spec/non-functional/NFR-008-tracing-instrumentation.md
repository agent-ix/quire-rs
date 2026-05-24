---
id: NFR-008
title: "Production tracing Instrumentation (Feature-Gated)"
artifact_type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "implements"
    cardinality: "1:1"
---

## Statement

`quire-rs` SHALL emit `tracing` crate spans for major public entry points so downstream consumers (Filament editor, batch tools, observability stacks) can observe its behavior. Instrumentation is **feature-gated** behind a Cargo `tracing` feature; disabled-by-default to keep the dependency lean for callers who don't need it.

### Required spans

| Entry point | Span name | Level | Fields |
|---|---|---|---|
| `Registry::load_from` | `quire_rs::load` | `info` | `paths`, `archetype_count`, `module_count`, `duration_ms` |
| `Registry::archetype` | (no span — too hot) | — | — |
| `render(archetype, data)` | `quire_rs::render` | `debug` | `archetype_name`, `data_bytes`, `output_bytes`, `duration_us` |
| `apply_patch(archetype, current, patch)` | `quire_rs::apply_patch` | `debug` | `archetype_name`, `duration_us` |
| `parse_document(markdown)` | `quire_rs::parse` | `debug` | `input_bytes`, `section_count`, `duration_us` |
| `extract(doc, dsl)` | `quire_rs::extract` | `debug` | `record_count`, `edge_count`, `diagnostic_count`, `duration_us` |
| `harvest_edges(...)` | `quire_rs::harvest_edges` | `debug` | `edge_count`, `diagnostic_count`, `duration_us` |

### Required events

| Event | Where | Level | Fields |
|---|---|---|---|
| Per-archetype load completion | inside `load_from` | `trace` | `archetype_name`, `module`, `duration_us` |
| Diagnostic emitted | wherever a `Diagnostic` is created | `warn` (collisions) or `debug` (informational) | the diagnostic's discriminant + key fields |

### Feature flag

`Cargo.toml`:

```toml
[features]
default = []
tracing = ["dep:tracing"]
```

When the feature is disabled, all instrumentation expands to nothing (zero-cost via the `tracing` crate's macros).

## Rationale

Downstream observers (Filament editor latency dashboards, batch-job operators) need to see what the engine is doing without strace-level diving. Without instrumentation, the engine is opaque. Feature-gating means the default-stripped binary stays minimal for embedded / no-std-adjacent consumers.

## Acceptance Criteria

- **NFR-008-AC-1**: With the `tracing` feature enabled, calling `parse_document` produces a `quire_rs::parse` span at `debug` level with `input_bytes`, `section_count`, `duration_us` fields populated.
- **NFR-008-AC-2**: With the `tracing` feature disabled, the same call produces no spans (verified via a test that installs a counting subscriber and asserts zero events).
- **NFR-008-AC-3**: A `Diagnostic::DuplicateArchetype` emitted during `load_from` is also surfaced as a `tracing::warn!` event with the `name` and `modules` fields.
- **NFR-008-AC-4**: A criterion bench compares `render` p50 with the feature enabled (subscriber attached, drained to noop) vs disabled; overhead < 5%.

## Verification

- Unit tests in `tests/tracing_shape.rs` using `tracing::subscriber::with_default` and a custom recording subscriber.
- Bench in `benches/render.rs` with a `--features tracing` flavor.
