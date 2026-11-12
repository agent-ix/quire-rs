---
id: Task-020
title: "FR-072 — surface, semantic-v1 schema, bindings"
type: Task
status: completed
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-019
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-072
    type: references
---
# Task-020: FR-072 — surface, semantic-v1 schema, bindings

## Scope

Publish one `SemanticExtraction` record with availability and `lossy`, attach it to the Filament API, `validate_document`, and the Python binding, and pin it with a hand-authored schema and the expected-records fixture.

## Subtasks

- [x] `src/semantic/surface.rs`: `extract_semantic`, availability/`lossy` computation, diagnostic ordering, `formatVersion: 1`.
- [x] `schemas/output/semantic-v1.schema.json` (hand-authored, `additionalProperties: false`) + `tests/fixtures/semantic/cases.expected.json` expected records; extend `check_no_schemars.sh`.
- [x] Filament API: `dataJson.semantic`, mirrored diagnostics with `locus` and mapped severity; no-context path byte-identical to baseline.
- [x] `validate_document`: semantic findings with locus; error fails validation.
- [x] Python: `extract_semantic` + additive payload in `extract_filament_core`; parity test over `cases.json` under `make ci-python`.
- [x] Fill `cases.json` for every state token and both `lossy` values; run the harness in `tests/it/semantic_surface.rs`.

## Deliverables

- `src/semantic/surface.rs`, `schemas/output/semantic-v1.schema.json`, `tests/it/semantic_surface.rs`, `tests/python/test_semantic.py`

## Notes

- The WASM binding is agent-ix/quire-wasm#3; this task ships the fixture file it consumes.
