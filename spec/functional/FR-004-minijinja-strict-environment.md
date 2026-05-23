---
id: FR-004
title: "MiniJinja Environment Configured Strict and Long-Lived"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-002"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

The render layer SHALL construct exactly one `minijinja::Environment` per process (or per `quire_rs::Renderer` instance, when consumers explicitly opt for an isolated instance). The environment SHALL be configured with:

1. `set_undefined_behavior(UndefinedBehavior::Strict)` — accessing a template field the schema did not provide produces an error rather than silently substituting empty.
2. All archetype templates pre-loaded at construction. Templates are NOT loaded on demand from disk during render — the environment is fully populated before the first render call.
3. No template auto-reload. Hot-reload is opt-in via a `Renderer::new_with_autoreload()` constructor intended only for dev / test paths.

The environment is `Send + Sync` and shared across threads.

## Acceptance

- **FR-004-AC-1**: A test renders an FR with a template that references a field absent from `FrData` and asserts the returned error is `QuireError::TemplateError` naming the missing field.
- **FR-004-AC-2**: A test constructs a `Renderer`, calls `render` from N=64 threads concurrently for 10000 iterations total, and asserts no panic, no race, byte-identical outputs.
- **FR-004-AC-3**: A criterion bench measures the cost of `Renderer::new()` and reports it; FR-004 documents the expected one-time cost (in the µs range). Subsequent render calls do NOT pay this cost.
