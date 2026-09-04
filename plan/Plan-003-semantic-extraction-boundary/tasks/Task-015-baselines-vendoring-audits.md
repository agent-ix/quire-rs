---
id: Task-015
title: "Baselines, vendored schemas, audit scaffold"
type: Task
status: completed
track: A
priority: P0
relationships:

  - target: ix://agent-ix/quire-rs/FR-069
    type: references
  - target: ix://agent-ix/quire-rs/NFR-021
    type: references
  - target: ix://agent-ix/quire-rs/TC-1606
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1607
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1639
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1643
    type: verifies
---
# Task-015: Baselines, vendored schemas, audit scaffold

## Scope

Freeze the pre-change contract surface and bring the upstream schemas into the tree with provenance before any extraction code lands.

## Subtasks

- [x] Mint `tests/fixtures/semantic/baseline/registry-archetypes.json` (archetype projection of every default and fixture module) and `filament-graph-cases.json` (current `graph_cases.json` outputs) from `main`; add the byte-identity tests (TC-1607, TC-1643) and the coverage/properties/assurance fixture comparison (TC-1639).
- [x] Create `schemas/vendored/` with `module-manifest.schema.json` (filament-core-service a77f31e), `semantic-core/0.1.0/` (filament-core-data d48b8da, `generated/json-schema/`), and `common.schema.json`; write `scripts/vendor-semantic-schemas.sh` that copies from pinned revisions and rewrites `schemas/vendored/PROVENANCE.json`; add the provenance test (TC-1606).
- [x] Add the static audit scripts under `scripts/audits/` (denylisted crates, forbidden symbols, net/process/fs-write on the semantic path) wired into `make audit-static`, initially green on an empty `src/semantic/`.

## Deliverables

- `tests/fixtures/semantic/baseline/*.json`, `tests/semantic_baseline.rs`
- `schemas/vendored/**`, `schemas/vendored/PROVENANCE.json`, `scripts/vendor-semantic-schemas.sh`
- `scripts/audits/check_semantic_boundary.sh`

## Notes

- Baselines are minted from `main` (e3352a0); a later diff against them is a defect.
