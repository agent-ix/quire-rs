# Task 027: Hardening Suite (NFR-011, NFR-012, NFR-013, NFR-014)

Status: not started (Track B — can stub-start; ramps as code lands)

## Scope

Implement the four scheduled-hardening NFRs:

- NFR-011 cargo-fuzz on parser + schema loader + DSL evaluator
- NFR-012 miri UB check on schedule + tag
- NFR-013 cargo-mutants on parser + DSL + edges
- NFR-014 cargo-audit daily + on PR

## Subtasks

- [ ] **Fuzz infrastructure (NFR-011).** `fuzz/` via `cargo +nightly fuzz init`. 6 targets: `fuzz_parse_document`, `fuzz_extract_frontmatter`, `fuzz_apply_patch`, `fuzz_extract_dsl`, `fuzz_load_manifest`, `fuzz_load_schema`. `.github/workflows/fuzz.yml` weekly + workflow_dispatch. `make fuzz` 60s smoke per target.
- [~] **miri job (NFR-012).** **RETIRED (ADR 0006)** — job + `make miri` removed; first-party UB is compile-impossible via `forbid(unsafe_code)` (NFR-003-AC-5), dependency advisories via cargo-audit (NFR-014).
- [ ] **Mutants config + job (NFR-013).** `.cargo/mutants.toml` declaring target paths. `mutants:` job in CI (weekly + workflow_dispatch). `mutants_baseline.txt` placeholder. `make mutants` local.
- [ ] **Advisory check (NFR-014).** `audit:` job in CI on PR + push + daily 06:00 UTC schedule. `make cargo-audit` local.

## Owns

NFR-011, NFR-012, NFR-013, NFR-014 (15 ACs combined).

## Dependencies

Loose. Fuzz targets need the corresponding source modules to exist (Tasks 001-005 + 015), so target authoring is gated on those. miri / mutants / audit can run against placeholder code today; they ramp in usefulness as code lands.

## Unblocks

Pre-tag hardening run. Substantially raises the safety floor above what the cookiecutter ships.

## Deliverables

- `fuzz/fuzz_targets/*.rs` (6 targets)
- `.github/workflows/fuzz.yml`
- Updated `.github/workflows/ci.yml` (miri, mutants, audit jobs)
- Updated `Makefile` (fuzz, miri, mutants, cargo-audit, hardening composite)
- `.cargo/mutants.toml` + `mutants_baseline.txt`
- README sections documenting scheduled-vs-PR division

## Primary Tests

TC-350 through TC-382 (14 TCs).

## Notes

- Track B — start in parallel. Initial scope: wire the empty/stub jobs so the lanes exist; flesh out fuzz targets and mutant config as code lands.
- `make hardening` invokes the full scheduled set locally; useful before tagging.
- Per spec.md §19: kani, loom, shuttle, cargo-careful, cargo-vet, qemu, SIMD-differential are explicitly NOT in scope for v1.
