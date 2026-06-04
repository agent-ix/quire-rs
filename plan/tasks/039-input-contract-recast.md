# Task 039: Archetype Input Contract Recast (skeleton/example)

Status: complete

## Scope

Recast FR-029: the per-archetype input contract is no longer derived from a render
template's variables. With direct-markdown authoring it is a **skeleton/example**
derived from `frontmatter_schema_ref` + the `body_extraction` asserts (the
structure an author fills and `validate_document` checks). FR-030's
`required_sections` model is superseded (no implementation; CR note only).

## Subtasks

- [ ] **Contract surface (FR-029 recast).** `input_contract_for(registry, archetype)` returns the frontmatter schema + the asserted body structure (sections/levels, table columns, list/id rules) as deterministic JSON-serializable data — derived from the loaded module (manifest + schema), never from rendered markdown.
- [ ] **Skeleton emission.** Produce a markdown skeleton (heading scaffold + literal table headers + contract comments + placeholders) from the same contract — the artifact handed to an authoring agent (/specify).
- [ ] **FR-030 supersession.** No `required_sections` validator is implemented; FR-030 stays a historical CR-noted FR. Ensure no code path reads `required_sections`.

## Owns

FR-029 (AC-1..6, recast). FR-030 (superseded — no new code).

## Dependencies

Task 037 (unified archetype provides body_extraction + schema).

## Unblocks

/specify authoring workflow (hands the skeleton to the agent).

## Deliverables

- `input_contract_for` + skeleton emitter (module TBD, e.g. `src/contract.rs`), PyO3 surface if consumed from Python.

## Primary Tests

TC-548, TC-549, TC-550, TC-551, TC-552, TC-553.

## Notes

The contract MUST be derived from the module, never inferred from rendered
markdown (FR-029 invariant). Deterministic JSON (byte-identical across calls).
