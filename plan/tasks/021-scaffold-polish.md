# Task 021: Scaffold Polish + StR-004 Verification

Status: not started (can start NOW — parallel)

## Scope

Verify the safety scaffolding inherited from `rust-lib-cookiecutter` is intact and add the missing artifacts the spec calls for: `spec/assets/render-parity-notes.md`, CI workflow render_parity job entry, backport-tracking docs.

## Subtasks

- [ ] **Verify cookiecutter inheritance (TC-203).** `clippy.toml`, `deny.toml`, `rustfmt.toml`, `rust-toolchain.toml`, `scripts/check_unsafe_comments.sh`, CI workflow, Makefile targets — byte-equal or documented MSRV-bump. Write a `scripts/verify_cookiecutter_inheritance.sh`.
- [ ] **Create render-parity-notes.md** under `spec/assets/` (empty placeholder; future divergences live here per StR-002-AC-2).
- [ ] **Wire render_parity into CI.** Update `.github/workflows/ci.yml` to add a `render_parity:` job calling `cargo test --test render_parity` (TC-204).
- [ ] **Backport tracking.** Add a comment / line to `CLAUDE.md` referencing the backport-code skill convention from StR-004-AC-3.
- [ ] **Confirm `make ci` is current.** Run it; should be green; this is the baseline before implementation starts.

## Owns

StR-004 (3 ACs), partial NFR-001/NFR-002 (CI wiring), partial US-005.

## Dependencies

None.

## Unblocks

Confidence that the foundation is intact. Provides the CI lane render_parity uses (Task 011 fills it with content).

## Deliverables

- `scripts/verify_cookiecutter_inheritance.sh`
- `spec/assets/render-parity-notes.md` (placeholder)
- Updated `.github/workflows/ci.yml`

## Primary Tests

TC-202, TC-203, TC-204, TC-050, TC-051 (existing CI gates).

## Notes

- Track B — start in parallel. Small surface; mostly admin.
