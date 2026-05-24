# Task 019: QuireError Shape + Field-Keyed Display

Status: not started (can start NOW — parallel)

## Scope

Define `QuireError` enum + `format_violation` helper that produces the four-tuple display per NFR-005 (field path / expected / observed / block type). This crosses every FR, so define it early.

## Subtasks

- [ ] **QuireError variants.** `UnknownArchetype`, `ArchetypeLoadError`, `SchemaViolation`, `TemplateError`, `MissingField`, `UnknownBlockType`, `ArchetypeCollision`, `ModuleCollision`. Non-exhaustive.
- [ ] **format_violation helper.** Single sink for user-facing display. Truncates long values at 80 chars with ellipsis.
- [ ] **Sanitization.** No raw `serde_json::Error` / validator native debug substrings.
- [ ] **Snapshot test.** Per archetype, pin a canonical error shape (TC-055).

## Owns

NFR-005 (3 ACs).

## Dependencies

None. The variant set will grow as FRs land, but the helper + base set can be authored now.

## Unblocks

All other tasks (consumers will return these variants).

## Deliverables

- `src/error.rs`

## Primary Tests

TC-006, TC-054, TC-055.

## Notes

- This is Track B — start in parallel with the critical path. Coordinate variant additions via PR review when new FRs land.
