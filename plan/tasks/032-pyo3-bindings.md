# Task 032: PyO3 Binding Surface + Gate G6

Status: core complete — `src/python/mod.rs` (feature `python`, pyo3 ~0.28, abi3-py39) + `pyproject.toml`. Wheel builds on CPython 3.14 (`quire-0.1.0-cp39-abi3`); `tests/python/test_bindings.py` (5 tests) pass in a venv: parse parity, load_repo, Spec queries on real spec/, GIL release. Default `cargo test` (221) + audits + clippy --features python all green; first-party src/ stays unsafe-free (NFR-003-AC-4). **Gate G6: core PASS.** Deferred: `validate` binding (TC-462) + the ≥5× speedup bench vs pure-Python filament_parser (TC-456, needs that lib present).

## Scope

Expose the engine to Python via feature-gated PyO3 + maturin (`quire` wheel): parse, extract, validate, render, `load_repo`, and the corpus. Then run G6 (parity + ≥5× speedup).

## Subtasks
- [ ] **Feature gate.** `python` Cargo feature gates `pyo3`/`pyo3-build-config`/maturin config. Default build unchanged + interpreter-free (StR-001 boundary); pin the new deps (NFR-009).
- [ ] **Surface.** `quire` module: `parse_document`, `extract`, `validate`, `render`/`render_block`, `Registry`, `load_repo`, `Spec` + queries. Return structured Python objects/`dict`/`list` — never JSON strings.
- [ ] **Error mapping.** `QuireError` variants → `quire` exception hierarchy carrying the same field path / file path / archetype name (NFR-005 parity across the boundary).
- [ ] **GIL release.** Release the GIL during heavy Rust work (`load_repo`, corpus construct, large parse) so multi-threaded Python callers aren't serialized.
- [ ] **abi3 packaging.** Build abi3 wheels; one wheel imports across CPython 3.x minors.
- [ ] **Overhead bench.** `parse_document` per-crossing overhead <50µs vs in-crate Rust (TC-469).
- [ ] **Speedup bench.** ≥500-doc corpus through binding vs pure-Python `filament_parser` ≥5× (TC-456).

## Owns
- FR-023, NFR-016 (+ NFR-009 extension for pyo3/maturin)

## Dependencies
- 028..031 (engine surface incl. corpus), Gate G5 PASS

## Unblocks
- `filament-parser-lib` consuming `quire` (downstream repo, out of this plan)

## Deliverables
- `python` feature + `src/python/` module; maturin config; abi3 wheel CI lane; overhead + speedup benches

## Primary Tests
- TC-460 (feature gate), TC-461 (parse parity), TC-462 (validate parity), TC-463 (load_repo via binding), TC-464 (GIL release), TC-465 (abi3 cross-version), TC-466 (no subprocess), TC-467 (structured objects), TC-469 (overhead bench), TC-456 (speedup bench)

## Gate G6: Binding parity + speedup
- **Measures:** binding outputs == Rust; one abi3 wheel imports on two CPython minors; corpus ≥5× faster than pure-Python.
- **Pass criteria:** TC-460..467 + TC-456 + TC-465 pass.
- **If fails:** parity miss → per-field object-conversion bug. Speedup miss → confirm GIL released (TC-464) and exactly one FFI crossing per `load_repo` (no per-file crossing).

## Notes
- abi3 costs a few % on hot calls — accepted (NFR-016 rationale). Keep `pyo3`/`maturin` strictly behind the feature so crates.io consumers never pull them.
- This is the only task needing the Python toolchain; everything upstream is pure Rust.
