# Task 037: Unified Archetype Shape

Status: complete

## Scope

Merge the `artifact_type` / `object_type` compiled paths into a single
`CompiledArchetype` per ADR 0003 / FR-031. One archetype carries optional
frontmatter schema + optional `body_extraction` + optional template + optional
`data_schema` + carry-over fields (`id_pattern`, `allowed_links`, `has_plugin`,
`grammar_ref`). Renderable/validatable is derived from which parts are present.
`required_sections` is rejected per the no-compat rule (see Notes).

## Subtasks

- [ ] **Manifest parse (FR-031).** `src/loader/manifest.rs`: an archetype entry may carry `template_ref` (optional), `frontmatter_schema_ref`, `body_extraction`, `data_schema`, and carry-over fields in one shape. Keep `extras` passthrough.
- [ ] **Compile merge.** `src/loader/compile.rs`: collapse `compile_artifact_type`/`compile_object_type` into one `CompiledArchetype` builder; populate template (when present), schema validators (frontmatter + data), body_extraction. Accessors: `is_renderable()`, `is_validatable()`, `body_extraction()`.
- [ ] **No-compat rejection.** A manifest archetype declaring `required_sections` or `variants` is a hard `ArchetypeLoadFailure` (NOT ignored) — no dual-read. Tests assert rejection.
- [ ] **Registry.** `src/registry.rs`: lookup by name unchanged; `by_module_and_name` unaffected.

## Owns

FR-031 (AC-1..6).

## Dependencies

FR-013 archetype loader (Task 005), FR-011 DSL (Task 015) — both complete.

## Unblocks

Task 036 (validate_document needs unified body_extraction + schema), Task 039.

## Deliverables

- `src/loader/manifest.rs`, `src/loader/compile.rs`, `src/registry.rs`

## Primary Tests

TC-522, TC-523, TC-524, TC-525, TC-526, TC-527.

## Notes

No backward-compatibility layer (project rule, FR-035 CR-002 alignment):
deprecated `required_sections`/`variants` are rejected, not tolerated. Existing
manifests are migrated (spec-artifacts-iso Phase 5), not dual-read.

> **CR note (deviation from FR-031-AC-5):** FR-031-AC-5 specs that a manifest
> still declaring `required_sections` *loads* with the field *ignored* and *one
> non-fatal diagnostic*. The no-backward-compatibility HARD RULE (this task +
> ADR 0003) overrides that softer behavior: a `required_sections` (or `variants`)
> declaration is a **hard `ArchetypeLoadFailure`** — the archetype does NOT load.
> Implemented as the hard failure (TC-526). This deviation is intentional and
> matches the project's "no deprecated-format fallbacks" directive; FR-031-AC-5's
> tolerant wording should be amended to "hard failure" in a future spec CR (spec
> is frozen for this implementation, so it is flagged here rather than edited).
