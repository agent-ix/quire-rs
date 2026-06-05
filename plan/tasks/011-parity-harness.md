# Task 011: Render Parity Test Harness

Status: **RETIRED** (render removal — 2026-06-04)

> The render/templating feature is removed (no backward-compatibility layer). This
> task (the `tests/render_parity/` harness, FR-012) is retired; the harness and
> `corpus.yaml` are removed. See `spec.md` §2bis and the retired FR-012. Kept for
> history.

Original status: blocked on Task 010

## Scope

Build the corpus-driven parity harness. Reads `tests/render_parity/corpus.yaml`, loads the listed archetype modules into a Registry, enumerates `(input.json, expected.md)` fixture pairs, and asserts byte-equality of `render` output vs. expected.

This task is the **harness only**, not the full sweep. A single archetype (FR) fixture suffices to prove the wiring; Task 013 expands to the full 17-archetype corpus.

## Subtasks

- [ ] **corpus.yaml format.** Define the schema: `modules: [{path: <local>, name: <module-name>}]`. Document in `tests/render_parity/README.md`.
- [ ] **Enumerator.** Walk the corpus, load Registry, glob `tests/render_parity/<archetype>/*.json`.
- [ ] **Test runner.** `cargo test --test render_parity` invokes the harness. Per-archetype + per-fixture failures surface with diff.
- [ ] **Fixture regeneration script.** `scripts/regenerate_parity_fixtures.sh` runs the Python reference renderer against each `input.json` and writes `expected.md`. Manual invocation only.
- [ ] **Initial fixture: FR archetype with one valid input + expected output.**

## Owns

FR-012 (5 ACs). Harness portion only; full sweep is Task 013.

## Dependencies

Task 010.

## Unblocks

Task 012 (parity gate).

## Deliverables

- `tests/render_parity/{corpus.yaml, README.md}`
- `tests/render_parity/fr/input_001.json` + `expected_001.md`
- `tests/render_parity.rs` (cargo test entry)
- `scripts/regenerate_parity_fixtures.sh`

## Primary Tests

TC-031 (corpus.yaml exists), TC-030 (sweep — but only 1 archetype at this task), TC-039 (data-only-extension property), TC-041 (regression catch).

## Notes

- The expected.md is byte-exact ground truth from the Python Jinja2 reference. Pin the Python venv version in the regeneration script.
- The corpus.yaml lists the LOCAL clones used during test (e.g. `~/dev/spec-artifacts-iso/spec_artifacts_iso/`). For CI portability, vendor a small fixture under `tests/render_parity/` rather than depending on sibling repos.
