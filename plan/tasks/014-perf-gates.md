# Task 014: Performance Gates (G3)

Status: blocked on Task 013

## Scope

**Quality Gate G3.** Bench the render path, parse path, and load path against NFR-001 / NFR-002 / NFR-007. Establish baselines + regression bands.

## Subtasks

- [ ] **bench_render_<archetype>** for each of the 17 archetypes — median <1ms each (NFR-001-AC-1..2).
- [ ] **bench_render_regression_gate** — 10% slowdown vs baseline fails CI.
- [ ] **bench_parse_5mb** — median <500ms; verifies roundtrip (TC-053).
- [ ] **bench_registry_load** — full 17-archetype + 87-object corpus loads in <100ms median.
- [ ] **bench_render_after_load** — 10k sequential renders against a warm registry; no I/O, no recompile (verified by tracing audit TC-121).
- [ ] **soak_test_1m_renders** — TC-122; memory footprint flat.

## Owns

NFR-001, NFR-002, NFR-007 (gates G3).

## Dependencies

Task 013 (full corpus loaded).

## Unblocks

(End of Track A critical path. Track C continues independently if G2 passed.)

## Deliverables

- `benches/render.rs`, `benches/parse.rs`, `benches/load.rs`
- Stored criterion baselines under `target/criterion/`
- `tests/no_hidden_recompile.rs` (tracing audit)
- `tests/soak.rs` (soak test, opt-in)

## Primary Tests

TC-042, TC-052, TC-083, TC-120, TC-121, TC-122, TC-053.

## Notes

- Baseline hardware: Apple Silicon M-class. CI hardware (Ubuntu x86_64) will be slower; set per-runner bands.
- If a perf target misses by >5×, do NOT widen the NFR — find the structural fix (likely validator choice or template recompile loop).
