# Task 013: Full Render Parity Sweep (17 Archetypes)

Status: blocked on Gate G2 (Task 012)

## Scope

Expand the harness fixtures from 1 (FR only, Task 011) to the full 17-archetype baseline: spec-artifacts-iso (8) + spec-artifacts-app (2) + spec-artifacts-process (7).

## Subtasks

- [ ] **NFR, StR, US, IT, TC, AC, CON fixtures** under `tests/render_parity/<name>/` from spec-artifacts-iso.
- [ ] **ApplicationSpec, MasterRequirements fixtures** from spec-artifacts-app.
- [ ] **ADR, Plan, Task, Review, Finding, TestMatrix, Standard fixtures** from spec-artifacts-process.
- [ ] **At least one valid-boundary + one beyond-boundary fixture per archetype** for constraint coverage (per tests.md).
- [ ] **Regenerate all expecteds** via the Python reference script.

## Owns

FR-012 full-corpus coverage.

## Dependencies

Gate G2 passing.

## Unblocks

Track A continues to Task 014 (perf gates).

## Deliverables

- 30-40 new fixture pairs under `tests/render_parity/`
- Updated `tests/render_parity/corpus.yaml`

## Primary Tests

TC-030 (full sweep), TC-039.

## Notes

- The fixture authoring is data work, not code. Per task description, FR-012 archetype data lives in spec-artifacts-* — but the test FIXTURES (input.json shaped for our test harness) live here. Don't confuse the two.
- This task is the bulk of "implementation" effort for the render path's correctness story but it's almost entirely YAML/JSON authoring.
