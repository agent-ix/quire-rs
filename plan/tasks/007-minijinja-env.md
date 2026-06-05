# Task 007: Strict MiniJinja Environment

Status: **RETIRED** (render removal — 2026-06-04)

> The render/templating feature is removed (no backward-compatibility layer). This
> task (the strict MiniJinja `Environment`, FR-004) is retired; the `minijinja`
> dependency and `src/render/env.rs` are removed. See `spec.md` §2bis and the
> retired FR-004. Kept for history.

Original status: blocked on Task 005

## Scope

Configure the long-lived `minijinja::Environment` used by the render path. Strict undefined behavior, includes disabled, env shared `Send + Sync`.

## Subtasks

- [ ] **Strict undefined.** `set_undefined_behavior(Strict)`. Missing template field → `QuireError::TemplateError { template, field }`.
- [ ] **Include rejection.** Configure a custom loader that returns an error for any `{% include %}` / `{% extends %}` resolution. TC-160.
- [ ] **Template add-once.** At loader (Task 005) time, every archetype's template is parsed and added to the env. No re-parse at render call (NFR-007).
- [ ] **No autoreload.** Production constructor never enables it. Optional `Renderer::new_with_autoreload()` for dev paths.
- [ ] **Send + Sync.** Compile-time bound.

## Owns

FR-004 (4 ACs).

## Dependencies

Task 005 (env is built during loader run).

## Unblocks

Task 010 (render dispatch uses the env).

## Deliverables

- `src/render/env.rs`

## Primary Tests

TC-010, TC-008 (thread-safety), TC-011 (env cost bench), TC-160 (include rejection).

## Notes

- minijinja default filters only. No custom filters at v1 (FR-004 includes section).
- The "no recompile" property (NFR-007-AC-3) is verified at Task 014; this task ensures the precondition (env populated at load).
