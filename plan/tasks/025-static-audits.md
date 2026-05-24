# Task 025: Static Audit Scripts (NFR-006, NFR-009, FR-003, FR-013, StR-001, StR-004)

Status: not started (can start NOW — parallel)

## Scope

Consolidate the static-audit scripts referenced by multiple ACs into `scripts/audits/` and wire them into `make ci`.

## Subtasks

- [ ] **`check_no_net_deps.sh`** — fail if `Cargo.lock` contains `reqwest`/`hyper`/`tonic`/etc. (FR-013-AC-6).
- [ ] **`check_no_schemars.sh`** — fail if `Cargo.lock` contains `schemars` (FR-003-AC-4).
- [ ] **`check_no_shellout.sh`** — grep `src/` for `std::process::Command` invocations targeting python/node/npm (StR-001-AC-2).
- [ ] **`check_dep_pins.sh`** — parse `Cargo.toml`; verify tilde/equals pins per NFR-009 policy.
- [ ] **`check_hashmap_audit.sh`** — grep render/parse code paths for `std::collections::HashMap` (NFR-006-AC-3).
- [ ] **`verify_cookiecutter_inheritance.sh`** — diff safety scaffolding files against `rust-lib-cookiecutter` baseline (StR-004-AC-1).
- [ ] **Wire into Makefile.** New `make audit-static` target runs all of the above. Add to `make ci` composite.
- [ ] **Wire into CI workflow.** Add an `audit-static` job to `.github/workflows/ci.yml`.

## Owns

Static-audit infrastructure across multiple ACs. No FR owns it directly; it's cross-cutting.

## Dependencies

None.

## Unblocks

Several ACs that reference these scripts (TC-085, TC-062, TC-201, TC-330, TC-058, TC-203).

## Deliverables

- `scripts/audits/check_no_net_deps.sh`
- `scripts/audits/check_no_schemars.sh`
- `scripts/audits/check_no_shellout.sh`
- `scripts/audits/check_dep_pins.sh`
- `scripts/audits/check_hashmap_audit.sh`
- `scripts/audits/verify_cookiecutter_inheritance.sh`
- Updated `Makefile`, `.github/workflows/ci.yml`

## Primary Tests

TC-085, TC-062, TC-201, TC-330, TC-058, TC-203.

## Notes

- Track B — start in parallel.
- Scripts can stub-pass (exit 0) when their inputs don't exist yet; substantive checks land as code does.
