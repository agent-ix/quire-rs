# Task 014: Performance Gates (G3)

Status: blocked on Task 013

> **CR note (render removal — 2026-06-04):** Render latency (NFR-001) is retired
> with the render feature. This gate now covers **parse / validate_document /
> extract / load** latency only — no render bench. See `spec.md` §2bis.

## Scope

**Quality Gate G3.** Bench the parse path, `validate_document` path, extract path,
and load path against NFR-002 / NFR-007. Establish baselines + regression bands.

## Subtasks

- [ ] **bench_parse_5mb** — median <500ms; verifies roundtrip (TC-053). (NFR-002-AC-1/3)
- [ ] **bench_validate_document** — typical FR-sized artifact median <1ms, warm registry (NFR-002-AC-4).
- [ ] **bench_extract** — `parse_document` + `extract` on a 10 KB doc (US-010 perf benches retained).
- [ ] **bench_registry_regression_gate** — 10% slowdown vs baseline fails CI.
- [ ] **bench_registry_load** — full schema-only corpus loads in <100ms median (NFR-007).
- [ ] **bench_validate_after_load** — 10k sequential `validate_document` calls against a warm registry; no I/O, no re-read (tracing audit TC-121, repurposed).

## Owns

NFR-002, NFR-007 (gate G3). (NFR-001 retired.)

## Dependencies

Task 013 (full corpus loaded).

## Unblocks

(End of Track A critical path. Track C continues independently if G2 passed.)

## Deliverables

- `benches/parse.rs`, `benches/validate.rs`, `benches/load.rs` (no `benches/render.rs`)
- Stored criterion baselines under `target/criterion/`
- `tests/no_hidden_recompile.rs` (tracing audit)

## Primary Tests

TC-052, TC-053, TC-083, TC-121, plus the `validate_document` latency TC (NFR-002-AC-4).

## Notes

- Baseline hardware: Apple Silicon M-class. CI hardware (Ubuntu x86_64) will be slower; set per-runner bands.
- If a perf target misses by >5×, do NOT widen the NFR — find the structural fix (likely validator choice).
