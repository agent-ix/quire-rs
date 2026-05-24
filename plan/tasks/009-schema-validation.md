# Task 009: Schema Validation Pipeline (apply_patch)

Status: blocked on Task 005

## Scope

Implement `apply_patch(archetype, current, patch) -> Result<Value, QuireError>`: deep JSON merge, validate the merged result against the compiled JSON Schema. The merge-then-validate semantic is load-bearing.

## Subtasks

- [ ] **Deep JSON merge.** Recursive merge: scalars and arrays replaced; objects merged key-by-key. Pure function.
- [ ] **Validation.** Use the compiled validator from Task 005. Convert validator errors to `QuireError::SchemaViolation` with field paths.
- [ ] **additionalProperties:false.** A patch adding a forbidden key fails validation post-merge.
- [ ] **`$defs` + recursive `$ref`** within a single schema must work (FR-002-AC-6).
- [ ] **Cross-file `$ref`** must fail at LOAD time, not patch time (TC-171; tested in Task 005).
- [ ] **Bench.** `apply_patch` median < 100 µs for a typical artifact (TC-042b).

## Owns

FR-002 (7 ACs).

## Dependencies

Task 005 (loader produces compiled validator).

## Unblocks

Task 010 (render dispatch calls apply_patch internally, or validates separately first).

## Deliverables

- `src/merge.rs` — JSON deep merge
- `src/validate.rs` — apply_patch entry + validator-error → QuireError conversion

## Primary Tests

TC-007, TC-007b, TC-002b (proptest fuzz), TC-170, TC-205, TC-042b (bench).

## Notes

- Pure function: no IO, no global state.
- Merge semantics for arrays: WHOLESALE REPLACE (not append) — common Filament convention.
- Validator errors are unbounded in count (`oneOf` can enumerate variants); don't truncate.
