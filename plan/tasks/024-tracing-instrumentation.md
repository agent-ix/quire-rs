# Task 024: tracing Instrumentation (NFR-008)

Status: not started (can start NOW — parallel; small surface)

## Scope

Add feature-gated `tracing` crate instrumentation to major entry points (load, render, parse, extract, harvest). Zero-cost when disabled; spans + events when enabled.

## Subtasks

- [ ] **Cargo feature.** `[features] tracing = ["dep:tracing"]`. Default off.
- [ ] **Spans.** Per NFR-008 table: `quire_rs::load`, `render`, `apply_patch`, `parse`, `extract`, `harvest_edges`. Levels per table.
- [ ] **Events.** Per-archetype load completion; Diagnostic emission as `tracing::warn!` or `debug!`.
- [ ] **Zero-cost when disabled.** `#[cfg(feature = "tracing")]` or inline macro guards. Verified by TC-321.
- [ ] **Overhead bench.** Render p50 with feature enabled (no-op subscriber) vs disabled; assert < 5%.

## Owns

NFR-008 (4 ACs).

## Dependencies

Loose — can start in parallel with critical path. Instrumentation is added to each entry point as it lands; not blocking on those tasks.

## Unblocks

Downstream observability for Filament editor + batch tools.

## Deliverables

- Updates across `src/` (each entry point gets its instrumentation)
- `tests/tracing_shape.rs`
- Bench update in `benches/render.rs`

## Primary Tests

TC-320, TC-321, TC-322, TC-323.

## Notes

- Track B — start in parallel.
- Use `tracing::instrument` proc macro for simple spans; manual `info_span!` where field shape needs control.
