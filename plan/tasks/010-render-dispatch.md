# Task 010: render Dispatch

Status: blocked on Tasks 006, 007, 008, 009

## Scope

The public `render(archetype, data) -> Result<String, QuireError>` and `render_by_name(registry, name, data)` entry points. Validates input via the compiled schema (Task 009 path), then renders via the strict env (Task 007).

## Subtasks

- [ ] **render(archetype, data).** Validate data against archetype's schema → render via archetype's template in the shared env → return string.
- [ ] **render_by_name(registry, name, data).** Look up archetype, dispatch to render.
- [ ] **Concurrency.** `Send + Sync` for all input + output. Stress test with TC-008.
- [ ] **Errors.** UnknownArchetype, SchemaViolation, TemplateError — all field-keyed via NFR-005.

## Owns

FR-001 (5 ACs).

## Dependencies

Tasks 006, 007, 008, 009.

## Unblocks

Task 011 (parity harness calls render), Task 012 (parity gate G2).

## Deliverables

- `src/render/mod.rs` — public entry

## Primary Tests

TC-003, TC-004, TC-005, TC-006, TC-008.

## Notes

- The public API takes `&CompiledArchetype` directly OR `&Registry + &str`. Both are non-owning; the registry's `Arc<CompiledArchetype>` keeps everything live.
- The "data-only-change" property (TC-005, FR-001-AC-5, StR-001-AC-4) is verified here by integration test: add a new (manifest entry + schema + template) to a fixture directory; reload registry; render via the new name; success without any Rust source change.
