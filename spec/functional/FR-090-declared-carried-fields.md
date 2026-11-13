---
id: FR-090
title: "Declared carried fields: a stated value reaches the coverage matrix and the obligation records as written"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-053"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-082"
    type: "references"
  - target: "ix://agent-ix/spec-artifacts-iso/FR-009"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "traces_to"
---
# FR-090: Declared carried fields: a stated value reaches the coverage matrix and the obligation records as written

Tracked by Linear PLAT-1526 (implemented by PLAT-1577 and PLAT-1579).

## Description

A person states some facts on a requirement that downstream tools must see on every obligation derived from
it. [FR-082](./FR-082-coverage-matrix-criticality.md) does this for `criticality`, with the field name
known to the engine. The software safety class
([spec-artifacts-iso FR-009](ix://agent-ix/spec-artifacts-iso/FR-009)) is the second such fact, and a third
will follow, so this requirement states the general mechanism and the engine learns no field name from it.

`quire-rs` SHALL read the active module's `traceability.carried_fields`, each entry naming:

| Field | Meaning |
|---|---|
| `name` | The output key the value is carried under, for example `software_safety_class` |
| `from` | Archetypes whose documents state the value |
| `key` | The top-level frontmatter key the value is read from |

For every document of a `from` archetype the engine SHALL carry the value of `key`, as written, onto the
document's `requirements[]` entry in the `coverage_matrix` ([FR-050](./FR-050-declarative-coverage-computation.md)-AC-48)
and onto each obligation record ([FR-053](./FR-053-obligation-record.md)) derived from that document's rows, under
`name`. The JSON of `quire trace` and `quire matrix` carries it on both. The engine holds no archetype, no frontmatter
key and no value list: the manifest names the first two, and the permitted values are the `enum` of the field in the
`from` archetypes' frontmatter schema ([FR-059](./FR-059-declared-vocabulary-coverage.md) reads a field's vocabulary
the same way).

### Absent is absent

A document with no `key`, or with the key present and null, yields entries and obligations with no `name` key at all.
The engine writes no label, default or placeholder. The renderer's `unassigned` ([quire-cli FR-028](ix://agent-ix/quire-cli/spec/functional/FR-028))
is a display of absence, not a value. A present value outside the `enum` is carried as written and left to schema
validation; the engine neither drops nor replaces it.

### No inference, no write

The engine SHALL NOT derive a carried value from a related document, whatever the relationship verb, from the
value of another carried field, from `criticality`, or from the highest or lowest value among related documents.
It SHALL NOT write a value to any document. Each carried field is independent: it neither supplies, overrides nor
fills in `criticality` ([FR-082](./FR-082-coverage-matrix-criticality.md), [FR-053](./FR-053-obligation-record.md))
nor any other carried field, and a module that declares both reads both from their own sources.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-090-AC-1 | A module declaring `carried_fields` with `name: software_safety_class`, `from: [FR]`, `key: software_safety_class` yields, for an `FR` document stating `class-2`, a `requirements[]` entry and every obligation derived from it carrying `software_safety_class` equal to `class-2`. | Test |
| FR-090-AC-2 | An `FR` document with no such key, and one with the key present and null, yield an entry and obligations with no `software_safety_class` key at all, not an empty string, null or `unassigned`. | Test |
| FR-090-AC-3 | Two documents related by each relationship type in the corpus, one stating `class-3` and one stating nothing, leave the second with no carried key. | Test |
| FR-090-AC-4 | A document stating `criticality: high` and no class yields entries and obligations with `criticality` and no `software_safety_class`; a document stating a class and no criticality yields the reverse; a document stating both carries both as written. | Test |
| FR-090-AC-5 | A module with no `carried_fields` declaration yields output byte-identical to the output before this requirement. | Test |
| FR-090-AC-6 | A value outside the field's `enum` is carried as written. | Test |
| FR-090-AC-7 | A `carried_fields` entry whose `from` names an archetype the module does not declare, or whose `key` is a property of none of the `from` frontmatter schemas, fails the module load naming the entry, as [FR-058](./FR-058-upward-trace-completeness.md) fails a bad declaration. | Test |
| FR-090-AC-8 | The JSON of `quire trace` and `quire matrix` carries the carried key on each obligation that has it. | Test |
| FR-090-AC-9 | Running the matrix leaves every document byte-identical, and two runs over an identical corpus serialize identically, with and without the key. | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-090-CON-1 | The engine holds no frontmatter key name, archetype name or value list for any carried field. | Design | Inspection |
| FR-090-CON-2 | No code path assigns, defaults, infers or writes back a carried value. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md), [FR-053](./FR-053-obligation-record.md), [FR-082](./FR-082-coverage-matrix-criticality.md), [FR-059](./FR-059-declared-vocabulary-coverage.md)
- **Downstream**: [FR-091](./FR-091-ordered-field-consistency.md) (reads the carried value), [quoin FR-160](ix://agent-ix/quoin/FR-160) (per-class floors), quire-cli FR-028 renders the value
