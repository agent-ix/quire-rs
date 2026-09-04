---
id: Task-018
title: "FR-070 — typed Properties extraction"
type: Task
status: completed
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-016
    type: depends_on
  - target: ix://agent-ix/quire-rs/Task-017
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-070
    type: references
  - target: ix://agent-ix/quire-rs/TC-1610
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1611
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1612
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1613
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1614
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1615
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1616
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1617
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1618
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1621
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1647
    type: verifies
---
# Task-018: FR-070 — typed Properties extraction

## Scope

Extract the typed table or `sysml` fence into normalized `FieldDecl[]` under the `BundleIndex` resolution rules, with legacy-form detection and whole-kind unavailability on row errors.

## Subtasks

- [x] `src/semantic/properties.rs`: form recognition (typed header, fence, legacy, both-forms, duplicate section, empty table), cell grammars (Type precedence, Multiplicity, Constraints incl. `nullable`, reader rules), fence line subset, normalization, FieldDecl validation gate.
- [x] `SemanticContext` + `BundleIndex` types; corpus-mode index builder from `Spec` (id, title, frontmatter `name`; imports from loaded modules).
- [x] Legacy-form diagnostics with `legacy_forms` policy; `properties` string untouched (TC-1618 vs baseline).
- [x] Property test generators for cells (TC-1621).

## Deliverables

- `src/semantic/{context,properties}.rs`, `tests/semantic_properties.rs`, `tests/props_semantic.rs`

## Notes

- Golden cases run under `config-version.bundle.json`; corpus-mode naming for real modules is a module-authoring question (spec-objects-business#4).
