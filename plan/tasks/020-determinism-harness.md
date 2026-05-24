# Task 020: Determinism Test Harness (NFR-006)

Status: not started (can start NOW — parallel)

## Scope

Build the test infrastructure for proving determinism: proptest setup + the "no observable HashMap" audit. Applies across every FR; lives outside any individual one.

## Subtasks

- [ ] **proptest baseline.** Configure `proptest` for the project; set `PROPTEST_CASES=512` for CI.
- [ ] **Determinism strategy.** Given (input, archetype, resolver), run N=100 across threads; assert byte-identical outputs.
- [ ] **HashMap audit.** Static check (could be a `cargo clippy` lint config or a bash grep) confirming no `std::collections::HashMap` use in render or parse code paths where iteration is observable. `IndexMap` or `BTreeMap` is the substitute.
- [ ] **Tests.** TC-056 (render det), TC-057 (parse det), TC-058 (HashMap audit), TC-141 (harvest_edges det).

## Owns

NFR-006 (3 ACs).

## Dependencies

None (test infra). The specific tests (TC-056, 057, 141) attach after Tasks 010 / 002 / 017 respectively.

## Unblocks

Determinism verification across the codebase as FRs land.

## Deliverables

- `tests/determinism.rs`
- `scripts/audit_hashmap.sh` (or equivalent)
- `Cargo.toml` updates for `proptest`, possibly `indexmap`

## Primary Tests

TC-056, TC-057, TC-058, TC-141.

## Notes

- Track B — start in parallel. The harness can run vs. a stub `render` placeholder until Task 010 lands.
