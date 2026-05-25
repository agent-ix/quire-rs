# Task 034: Concurrency + FFI Hardening (loom / TSAN / ASAN)

Status: Part A complete — `tests/concurrency.rs` loom test passes under `--cfg loom`; `scripts/audits/check_no_shared_mutable.sh` (TC-502) wired into `make audit-static`; `make loom` + `make sanitize` targets added; `[lints.rust]` registers `cfg(loom)`. Part B (TSAN/ASAN) blocked: needs the python-feature wheel (Task 032) + nightly sanitizers.

## Scope

Implement the v0.3 hardening re-review (spec.md §19): prove the parallel-walk path is race-free (loom), cover the FFI boundary that miri can't reach (TSAN + ASAN), and enforce the scoping invariants that keep the existing guarantees honest. Splits into two independently-schedulable halves.

## Subtasks

### Part A — Concurrency (after task 028)
- [ ] **No-shared-mutable-state audit (FR-024-AC-9).** Static check (`rg` for `Mutex`/`RwLock`/`Atomic*` in first-party `src/`) + confirm the parallel parse collects owned results (`par_iter().map().collect()`), not a shared buffer push.
- [ ] **loom test (NFR-017).** `#[cfg(loom)]` test modeling a 2–3 file / 2 thread parallel parse collect; assert no race + identical path-sorted output across all interleavings. Keep scope small enough for the lane to finish ≤ 30 min.
- [ ] **loom CI lane.** Scheduled + workflow_dispatch + tag push; `make loom` local target.

### Part B — FFI sanitizers (after task 032 / Gate G6)
- [ ] **TSAN lane (NFR-018-AC-1).** Build the `python`-feature extension with `-Z sanitizer=thread`; run a two-thread `load_repo` harness (the GIL-release window, TC-464) + concurrent `parse_document`. Zero races.
- [ ] **ASAN lane (NFR-018-AC-2).** Build with `-Z sanitizer=address`; run the object-handoff test set (results crossing to Python then dropped). Zero leaks/UAF in first-party handoff; maintain `asan.supp` for interpreter-internal noise (each suppression carries a rationale).
- [ ] **Sanitizer CI lanes (NFR-018-AC-3).** Scheduled + workflow_dispatch + tag push; `make sanitize` local target.
- [ ] **miri scope note (NFR-012-AC-5).** Confirm the `miri` job runs without `python`; record the FFI-out-of-scope note in the workflow/§19.
- [ ] **unsafe scope (NFR-003-AC-4).** Confirm `rg 'unsafe {' src/` is zero with `--features python`.

## Owns
- NFR-017, NFR-018 (+ FR-024-AC-9, NFR-003-AC-4, NFR-012-AC-5)

## Dependencies
- Part A: task 028 (`load_repo` parallel parse)
- Part B: task 032 (the built `python`-feature extension), Gate G6 PASS

## Unblocks
- v0.3 hardening completeness (no downstream feature task)

## Deliverables
- `#[cfg(loom)]` concurrency test + `make loom`; loom CI lane
- TSAN + ASAN CI lanes + `make sanitize` + `asan.supp`
- Static audits for FR-024-AC-9 / NFR-003-AC-4 / NFR-012-AC-5

## Primary Tests
- TC-502 (no-shared-mutable-state audit), TC-503 (loom), TC-504 (TSAN), TC-505 (ASAN), TC-506 (python-feature unsafe-free), TC-507 (miri FFI scope)

## Notes
- Parallel-ready in two halves: Part A can start as soon as 028 lands (pure Rust, no Python); Part B waits for the wheel (032) + G6.
- All lanes are **scheduled, not per-PR** — they add zero PR latency, matching the miri/fuzz/mutants cadence (§19 implementation notes).
- If loom finds a race, it means FR-024's data-parallel invariant was violated — fix by restoring the owned-collect pattern, not by adding a lock.
