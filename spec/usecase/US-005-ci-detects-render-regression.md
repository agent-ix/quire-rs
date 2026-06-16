---
id: US-005
title: "CI Detects a Render Regression Against Python Reference Fixtures"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-002"
    type: "implements"
    cardinality: "1:1"
---

> **RETIRED (render removal — 2026-06-04):** This user story is entirely about the
> render byte-parity suite (`tests/render_parity/` + `cargo test --test render_parity`).
> With the render feature removed (no backward-compatibility layer), there is no render
> path to compare against the Python Jinja2 reference, the `render_parity/` fixtures and
> test are deleted, and StR-002 (render parity) is itself retired. This US is therefore
> **retired** alongside US-001/004/006/007/009 and FR-012 from the same render-removal
> slice. Kept for history and traceability only; its acceptance criteria are dropped from
> the required-coverage tally. See `spec.md` §2bis.

## Story

As a **maintainer landing a render-side change**, I want CI to run a byte-parity suite against the Python Jinja2 reference for every archetype and fail the build if any divergence is introduced, so that a refactor cannot silently produce different markdown output.

## Context

The parity suite is the regression-safety net for StR-002. Without it, a contributor changing the dispatch logic, the `Environment` configuration, or a single template could ship a release that produces near-identical-looking markdown that nonetheless breaks downstream byte-equality assumptions (git diffs, line-tools, scriptable parsers).

## Acceptance

- US-005-AC-1 (RETIRED): A test directory `tests/render_parity/` contains, for each of the 10 archetypes, one or more `(typed_input.json, expected_output.md)` pairs. The `expected_output.md` was produced by running the Python reference renderer.
- US-005-AC-2 (RETIRED): `cargo test --test render_parity` runs the Rust renderer for every input and asserts byte-equality with the expected output.
- US-005-AC-3 (RETIRED): CI runs `cargo test --test render_parity` on every PR; failures block merge.
- US-005-AC-4 (RETIRED): When the Python reference renderer is intentionally updated, a script regenerates the fixtures and the PR shows the diff to a human reviewer.
