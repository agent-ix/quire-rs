---
id: FR-004
title: "MiniJinja Environment Configured Strict and Long-Lived"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-002"
    type: "implements"
    cardinality: "1:1"
---

> **RETIRED (render removal — 2026-06-04):** The render/templating feature is
> **removed** from `quire-rs` (no backward-compatibility layer). This FR (the strict,
> long-lived MiniJinja `Environment`) is **retired**: the `minijinja` dependency is
> dropped and no template environment is constructed. This document is kept for
> history and traceability only; its acceptance criteria are dropped from the
> required-coverage tally. The retirement and rationale are recorded in `spec.md`
> §2bis. New work does not target this FR.

## Description

The render layer SHALL construct exactly one `minijinja::Environment` per process (or per `quire_rs::Renderer` instance, when consumers explicitly opt for an isolated instance). The environment SHALL be configured with:

1. `set_undefined_behavior(UndefinedBehavior::Strict)` — accessing a template field the schema did not provide produces an error rather than silently substituting empty.
2. All archetype templates pre-loaded at construction. Templates are NOT loaded on demand from disk during render — the environment is fully populated before the first render call.
3. No template auto-reload. Hot-reload is opt-in via a `Renderer::new_with_autoreload()` constructor intended only for dev / test paths.

The environment is `Send + Sync` and shared across threads.

### Cross-archetype includes

MiniJinja's `{% include %}` and `{% extends %}` directives are **disabled at v1**. Templates are loaded as isolated units; each archetype's template references its own data context only. A template attempting `{% include %}` SHALL fail at load time with `QuireError::ArchetypeLoadError { reason: "{% include %} is not supported at v1" }`.

This is enforced by configuring MiniJinja's loader to return an error for any include resolution attempt. Cross-archetype composition is a follow-on concern and requires explicit design before being enabled.

### Custom filters

Custom MiniJinja filters added by `quire-rs` SHALL be pure (no I/O, no global state). At v1 the engine ships no archetype-specific filters; only MiniJinja's built-in safe filters are available. Adding custom filters is a follow-on FR.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-004-AC-1 | (RETIRED) A test renders an FR with a template that references a field absent from `FrData` and asserts the returned error is `QuireError::TemplateError` naming the missing field. | Test |
| FR-004-AC-2 | (RETIRED) A test constructs a `Renderer`, calls `render` from N=64 threads concurrently for 10000 iterations total, and asserts no panic, no race, byte-identical outputs. | Test |
| FR-004-AC-3 | (RETIRED) A criterion bench measures the cost of `Renderer::new()` and reports it; FR-004 documents the expected one-time cost (in the µs range). Subsequent render calls do NOT pay this cost. | Test |
| FR-004-AC-4 | (RETIRED) A test loads a template containing `{% include "other.j2" %}` and asserts `QuireError::ArchetypeLoadError` with the "include not supported" reason. | Test |

## Dependencies

- **Upstream**: StR-002
- **Downstream**: none (retired)
