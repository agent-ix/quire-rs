# Task 022: Diagnostic Collection API (FR-017)

Status: blocked on Task 005

## Scope

Define the `Diagnostic` enum + `Diagnostics` collector type + integration into `Registry`, `RenderOutput`, `ExtractionResult`, `EdgeHarvest`. Provides the consumer-facing surface for non-fatal informational messages.

## Subtasks

- [ ] **Diagnostic enum.** `#[non_exhaustive] enum Diagnostic { ... }` with all variants from FR-013/014/015/016/011. `Send + Sync + Debug + Clone + PartialEq + Eq`.
- [ ] **Diagnostics collector.** `pub struct Diagnostics(Vec<Diagnostic>)` with `is_empty`, `iter`, `filter`, `by_kind` helpers.
- [ ] **Surface from Registry.** `registry.load_diagnostics() -> &Diagnostics`.
- [ ] **Surface from per-call results.** `ExtractionResult` and `EdgeHarvest` already carry diagnostics fields (FR-011, FR-015); confirm. Add `RenderOutput { markdown, diagnostics }` for render.
- [ ] **Determinism.** Order is fixed by emission point.

## Owns

FR-017 (5 ACs).

## Dependencies

Task 005 (Registry construction is where load-time diagnostics accumulate).

## Unblocks

Cleaner consumer integration story.

## Deliverables

- `src/diagnostic.rs` (or `src/error.rs` extension)
- Updates to `src/registry.rs`, `src/render/mod.rs`, `src/extract/mod.rs`, `src/edges/mod.rs`

## Primary Tests

TC-300, TC-301, TC-302, TC-303, TC-304.

## Notes

- This task formalizes what other tasks (5, 6, 11, 15, 17) implicitly emit. Best to define the type early so they target it.
- Coordinate with Task 019 (QuireError); the two types are siblings.
