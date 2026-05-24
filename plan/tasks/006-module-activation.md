# Task 006: Module Activation

Status: blocked on Task 005

## Scope

Layer multi-module coexistence over the single-module loader from Task 005. Track per-archetype provenance, detect name collisions, expose `load_strict`, surface module versions.

## Subtasks

- [ ] **Module identity.** Per FR-014: identity from `manifest.yaml` `name:` field; fallback to parent dir name with diagnostic.
- [ ] **Module-name collisions.** Two manifests at different paths declaring same `name` → `DuplicateModuleName` diagnostic; first-wins.
- [ ] **Archetype-name collisions.** Two modules contributing same archetype name → `DuplicateArchetype` diagnostic; first-wins by search-path order.
- [ ] **load_strict.** Same loader but promotes collision diagnostics to `QuireError::ModuleCollision` / `ArchetypeCollision`.
- [ ] **module_version.** Surface manifest's `version: <semver>` if declared.
- [ ] **archetype_in_module.** Resolve by (module, name) for shadow inspection.

## Owns

FR-014 (7 ACs).

## Dependencies

Task 005 (loader produces per-module loaded data).

## Unblocks

Task 010 (render dispatch can call `registry.archetype(name)` confidently).

## Deliverables

- Augmented `src/registry.rs` with `Module` struct + provenance
- Diagnostic types

## Primary Tests

TC-090, TC-091, TC-092, TC-093, TC-094, TC-134, TC-135.

## Notes

- Search-path order is canonical: `IX_SCHEMA_PATH` is colon-split and entries are dedup-then-process in order.
- `Diagnostic` enum is shared across many FRs — define non-exhaustive variants.
