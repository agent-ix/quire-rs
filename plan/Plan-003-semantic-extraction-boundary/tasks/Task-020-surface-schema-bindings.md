---
id: Task-020
title: "FR-072 — surface, semantic-v1 schema, bindings"
type: Task
status: todo
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-019
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-072
    type: references
  - target: ix://agent-ix/quire-rs/TC-1630
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1631
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1632
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1634
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1635
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1637
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1638
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1644
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1650
    type: verifies
---
# Task-020: FR-072 — surface, semantic-v1 schema, bindings

## Scope

Publish one `SemanticExtraction` record with availability and `lossy`, attach it to the Filament API, `validate_document`, and the Python binding, and pin it with a hand-authored schema and compatibility fixture.

## Subtasks

- [ ] `src/semantic/surface.rs`: `extract_semantic`, availability/`lossy` computation, diagnostic ordering, `formatVersion: 1`.
- [ ] `schemas/output/semantic-v1.schema.json` (hand-authored, `additionalProperties: false`) + `tests/fixtures/semantic/semantic-v1.json` compatibility fixture; extend `check_no_schemars.sh`.
- [ ] Filament API: `dataJson.semantic`, mirrored diagnostics with `locus` and mapped severity; no-context path byte-identical to baseline.
- [ ] `validate_document`: semantic findings with locus; error fails validation.
- [ ] Python: `extract_semantic` + additive payload in `extract_filament_core`; parity test over `cases.json` under `make ci-python`.
- [ ] Fill `cases.json` for every state token and both `lossy` values; run the harness in `tests/semantic_surface.rs`.

## Deliverables

- `src/semantic/surface.rs`, `schemas/output/semantic-v1.schema.json`, `tests/semantic_surface.rs`, `tests/python/test_semantic.py`

## Notes

- The WASM binding is agent-ix/quire-wasm#3; this task ships the fixture file it consumes.
