---
id: Task-008
title: "FR-049 — verification-reference integrity in validate_bundle"
type: Task
status: completed
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-004
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
  - target: ix://agent-ix/quire-rs/TC-724
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-725
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-726
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-727
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-728
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-729
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-730
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-731
    type: verifies
---
# Task-008: FR-049 — verification-reference integrity in `validate_bundle`

## Scope

Extend bundle validation (FR-038) with the model-driven
`dangling-trace-reference` check: for each document-reference declaration in
the Task-004 model, extract referenced ids from the declared cells (multiple
annotations resolve independently), resolve them against the union of
bundle-document ids and auxiliary-trace-source ids (targeted scan, FR-044
harvester pattern), and report unresolved ids with document path + declaration
provenance. Posture-degradable exactly like `dangling-reference` (error under
`Strict`, warning under `Okf`). No model declared → zero findings.
Deterministic finding order.

## Subtasks
- [x] **Declared extraction.** Pattern/column/target kinds from the model —
  no ISO hardcoding (TC-727); multi-annotation cells (TC-730).
- [x] **Resolution set.** Bundle docs + auxiliary source harvest (TC-724,
  TC-728).
- [x] **Findings + postures.** (TC-725, TC-726); no-model no-op (TC-729);
  deterministic order (TC-731, Property).

## Deliverables
- Check in `src/corpus/validate.rs` alongside `dangling-reference`; tests
  tagged TC-724..731.

## Notes
- Depends only on Task-004 (model loader) — parallel to Task-005/006.
- Real-world activation on ISO repos awaits the `spec-artifacts-iso`
  declaration (external); fixtures cover the engine contract.

## Implementation record (2026-08-04)

- `src/corpus/trace_refs.rs`, called from `validate_bundle` after the `ix://`
  dangling check. A no-op when `Registry::traceability()` is `None` or declares
  no document references (FR-049-AC-6).
- Resolution set per declared target kind = ids of bundle documents of the
  declared archetype (their own `id` **and** the values of the declared minting
  column, so an FR mints its AC ids) ∪ the ids harvested from a declared
  auxiliary document via a targeted `read_to_string` + parse, the FR-044
  glossary-harvester pattern. An absent auxiliary file contributes nothing and
  raises nothing — the declaration names a source, not a requirement.
- Findings are collected, sorted, and deduped before routing, so their order is
  a property of the bundle rather than of the walk (TC-731). Routing uses the
  existing `BundleReport::degradable`, giving posture degradation for free.
- Fixture note: the ISO fixture gained a `test-case-document` target
  (archetype `TC`) so FR-049-AC-1's "a TC document **or** a declared
  trace-source row" union is actually exercised.
- Test-authoring gotcha worth keeping: `query::section` returns the *first*
  heading that matches, so a fixture titled `# Checks` shadows its own
  `## Checks` table section.
- TC-724..TC-731 green (`tests/trace_references.rs`); `make ci` green.
