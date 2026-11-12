---
id: FR-082
title: "Criticality column in the requirements traceability model"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-077"
    type: "extends"
  - target: "ix://agent-ix/spec-artifacts-iso/spec/functional/FR-008"
    type: "consumes"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "traces_to"
---
# FR-082: Criticality column in the requirements traceability model

## Description

The traceability model answers "what backs this requirement" ([FR-077](./FR-077-trace-search-index.md)).
A reader triaging that answer also needs to know how much the requirement matters. Requirement
archetypes may carry an optional, human-assigned `criticality` frontmatter value
([spec-artifacts-iso FR-008](ix://agent-ix/spec-artifacts-iso/spec/functional/FR-008)).

The trace model SHALL expose that value as a `criticality` column on every requirement row, and
`quire trace` output SHALL include the column. The engine reads the value from the requirement's
frontmatter as written; it holds no list of permitted values (the module's schema owns that).

A requirement with no `criticality` is reported with the literal value `unassigned`. This is a
reporting label, not a data value: the engine SHALL NOT default, infer, inherit from a parent
requirement, or write one back. The rendering of the column in the `quire trace` command lives in
quire-cli and consumes the engine's row field.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-082-AC-1 | A trace row for a requirement whose frontmatter has `criticality: high` carries `criticality` equal to `high`. | Test |
| FR-082-AC-2 | A trace row for a requirement with no `criticality` field carries `criticality` equal to `unassigned`. | Test |
| FR-082-AC-3 | For a corpus of requirements with values `low`, `medium`, `high` and one missing, each row reports its own value and only the missing one reports `unassigned`; no row's value is taken from another row. | Test |
| FR-082-AC-4 | A child requirement without `criticality` whose parent (via `traces_to`/`satisfies`) has `criticality: high` reports `unassigned`, not `high`. | Test |
| FR-082-AC-5 | A requirement whose `criticality` is present but outside the module-declared enumeration is reported with the value as written, alongside the validation diagnostic; it is not coerced to `unassigned` or to a valid value. | Test |
| FR-082-AC-6 | The `quire trace` table output has a `criticality` column header, and its JSON output has a `criticality` key on each requirement row, in both the forward (`--id`) and inverse (`--file`/`--symbol`) lookups. | Test (CLI snapshot) |
| FR-082-AC-7 | Running the trace over any corpus leaves every requirement file byte-identical; no `criticality` value is written. | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-082-CON-1 | The engine SHALL NOT embed the enumeration values of `criticality`; the only literal it owns is the reporting label `unassigned`. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-077](./FR-077-trace-search-index.md) (trace lookup), spec-artifacts-iso FR-008 (the field)
- **Downstream**: the `quire trace` command in quire-cli renders the column
