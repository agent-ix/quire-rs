# Task 005: Archetype Loader

Status: blocked on Gate G1 (Task 004)

## Scope

Implement the filesystem-first archetype loader: search-path resolution (`load_from`, `from_env`, `from_default`), manifest walking, schema compilation, template parsing, and per-archetype `CompiledArchetype` construction. **The engine's runtime entry point.**

## Subtasks

- [ ] **Search path resolution.** Honor explicit `paths` arg → `IX_SCHEMA_PATH` env var → `~/.ix/schemas/` default. Tilde expansion at leading position only. Dedup canonical paths. No env-var expansion mid-string.
- [ ] **Manifest parsing.** `manifest.yaml` → typed `Manifest` (artifact_types + object_types arrays). Validate structurally; load-time errors per FR-013-AC-3, FR-002-AC-7, FR-011-AC-6, AC-7.
- [ ] **Schema compilation.** Choose `jsonschema` crate (or equivalent). Compile each schema document once at load; reject cross-file `$ref`. Store the compiled validator + raw schema document in `CompiledArchetype`.
- [ ] **Template parsing.** Build the long-lived `minijinja::Environment` (`UndefinedBehavior::Strict`, no autoreload, includes disabled per FR-004). Add each template; reject `{% include %}` (TC-160).
- [ ] **Path edge cases.** Symlink-loop guard via visited canonical-paths set. Path-is-file → warning. Permission denied → warning. Duplicate `IX_SCHEMA_PATH` entries → dedupe.
- [ ] **Registry construction.** `Registry { archetypes: HashMap<String, Arc<CompiledArchetype>>, modules: HashMap<String, Module> }`. `Registry: Send + Sync` (TC-132).
- [ ] **Diagnostics.** `ArchetypeLoadError` aggregates per-archetype failures; non-failing archetypes still load.
- [ ] **Performance.** Load <100ms median for 17-archetype baseline (TC-083 bench).

## Owns

FR-013 (10 ACs).

## Dependencies

Task 004 (parser parity gate). Task 001 (frontmatter parse is reused for manifest.yaml parse? no — manifest is full YAML; use `serde_yaml`).

## Unblocks

Task 006 (module activation), Task 007 (env), Task 008 (schema surface), Task 009 (schema validation), Task 011 (parity harness), Task 015 (DSL).

## Deliverables

- `src/loader/{mod,manifest,paths,compile}.rs`
- `src/registry.rs`
- `src/error.rs` (QuireError variants for ArchetypeLoadError)

## Primary Tests

TC-080 thru TC-085, TC-130, TC-131, TC-132, TC-133, TC-160, TC-170, TC-171, TC-083 (bench).

## Notes

- Reference layout from existing spec-artifacts-iso: `spec_artifacts_iso/{manifest.yaml, schemas/, templates/}`.
- The validator choice is load-bearing for NFR-001 perf. Benchmark `jsonschema` vs `boon` early; fall back if `jsonschema` doesn't hit numbers.
- BOM-strip on schema files too (some authoring tools inject it).
