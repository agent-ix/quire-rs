---
id: FR-090
title: "Declared carried fields: a stated value reaches the requirement entries of the coverage matrix as written"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-082"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-059"
    type: "requires"
  - target: "ix://agent-ix/spec-artifacts-iso/FR-009"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "traces_to"
---
# FR-090: Declared carried fields: a stated value reaches the requirement entries of the coverage matrix as written

Tracked by Linear PLAT-1526 (implemented by PLAT-1577 and PLAT-1579).

## Description

A person states some facts on a requirement that downstream tools must read beside its coverage.
[FR-082](./FR-082-coverage-matrix-criticality.md) does this for `criticality`, with the field name known to the engine, and
only on the `requirements[]` entry of the `coverage_matrix` ([FR-050](./FR-050-declarative-coverage-computation.md)-AC-48):
obligation records keep the criticality of their own FR-053 column ([FR-082](./FR-082-coverage-matrix-criticality.md)-AC-7).
The software safety class ([spec-artifacts-iso FR-009](ix://agent-ix/spec-artifacts-iso/FR-009)) is a second such fact, so this
requirement states the general mechanism, and the engine learns no field name from it. A consumer that needs the fact for an
obligation joins the obligation to its requirement entry, as it does for FR-082 criticality.

`quire-rs` SHALL read the active modules' `traceability.carried_fields`, each entry naming:

| Field | Meaning |
|---|---|
| `name` | The key the value is carried under, matching `^[a-z][a-z0-9_]*$` |
| `from` | Archetypes whose documents state the value |
| `key` | The top-level frontmatter key the value is read from |

### The carried shape

For every document of a `from` archetype that states `key` with a non-null value, the engine SHALL carry the value as written
under `name` inside **one fixed key**, `carried`, an object, on the document's `requirements[]` entry. Nothing else carries it: not an obligation record, not a relation report row. A document
that states nothing has no `carried` key on its entry; a document that states some but not all of the declared names has
`carried` holding only those it states. The engine adds no other key to any entry, row or obligation record, writes no label,
default or placeholder, and leaves obligation records untouched.

The engine SHALL also report, once per run, the **ordered** permitted values of each carried field as a second fixed key,
`carried_vocabulary`, on the **top-level coverage report object** (the object that holds the `coverage_matrix` array; the matrix
itself is an array and gains no key). `carried_vocabulary` is an object keyed by `name`, each value the array of `enum` values of
`key` in schema order ([FR-059](./FR-059-declared-vocabulary-coverage.md) reads a field's vocabulary the same way).

The vocabulary does not depend on module load order. The `enum`s that exist among the `from` archetypes' frontmatter schemas for
`key` must all be identical in content and order; the vocabulary is that enum. A `from` archetype with no `enum` for `key` (for
example an object type whose frontmatter is open) contributes nothing and does not conflict. When two existing `enum`s differ, the
engine reports `trace:carried-vocabulary-mismatch` naming the entry, the two archetypes and their modules, and emits no
`carried_vocabulary` entry for that `name`. A `name` whose `key` has no `enum` in any `from` schema has no entry. A consumer
therefore learns the legal values and their rank from the run and never holds a list. A stated value outside the `enum` is carried
as written and left to schema validation; the engine neither drops nor replaces it.

The published coverage schema (`schemas/output/coverage-v1.schema.json`) is `additionalProperties: false` on
`CoverageMatrixRequirement` and on the top-level report object. This requirement SHALL amend it to add the optional `carried`
object (any string-keyed values) on `CoverageMatrixRequirement` and the optional `carried_vocabulary` object (string-keyed arrays
of strings) on the top-level report object, and no other key.

### Names, ownership and merging

A `carried_fields` entry belongs to the module that declares it. Structural defects fail that module's load naming the entry:
a blank or malformed `name`, a blank `key`, an empty `from`. Whether a `from` archetype or a `key` exists is not checked at
module load, because the archetype may come from another module ([FR-058](./FR-058-upward-trace-completeness.md)); it is checked
when the corpus is merged, against the merged registry of the loaded modules, and an unknown archetype, or a `key` that is a
property of none of the `from` archetypes' frontmatter schemas and carried by none of their documents, reports under
`trace:undeclared-relation-vocabulary` naming the entry.

Two entries with the same `name` are one carried field, and merge when their `key` is equal: the `from` lists are united. Two
entries with the same `name` and different `key` fail the merge naming both modules. This is how the requirement archetypes
(spec-artifacts-iso) and the part archetype (spec-objects-architecture) both state the same field.

### Absent is absent

A document with no `key`, or with the key present and null, has no entry under `name`. The renderer's `unassigned`
([quire-cli FR-028](ix://agent-ix/quire-cli/spec/functional/FR-028)) is a display of absence, not a value.

### No inference, no write

The engine SHALL NOT derive a carried value from a related document, whatever the relationship verb, from the value of another
carried field, from `criticality`, or from the highest or lowest value among related documents. It SHALL NOT write a value to any
document. Each carried field is independent: it neither supplies, overrides nor fills in `criticality`
([FR-082](./FR-082-coverage-matrix-criticality.md)) or another carried field.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-090-AC-1 | A module declaring `carried_fields` with `name: software_safety_class`, `from: [FR]`, `key: software_safety_class` yields, for an `FR` document stating `class-2`, a `requirements[]` entry whose `carried` equals `{"software_safety_class": "class-2"}`; the entry has no new top-level key. | Test |
| FR-090-AC-2 | An `FR` document with no such key, and one with the key present and null, yield entries with no `carried` key at all. | Test |
| FR-090-AC-3 | Two documents related by each relationship type in the corpus, one stating `class-3` and one stating nothing, leave the second with no `carried` key. | Test |
| FR-090-AC-4 | A document stating `criticality: high` and no class yields an entry with `criticality` and no `carried`; a document stating a class and no criticality yields `carried` and no `criticality`; one stating both carries both as written; obligation records of all three are byte-identical to their output without `carried_fields`. | Test |
| FR-090-AC-5 | A module with no `carried_fields` declaration yields output identical to the output before this requirement, with no `carried` and no `carried_vocabulary` key. | Test |
| FR-090-AC-6 | A value outside the field's `enum` is carried as written. | Test |
| FR-090-AC-7 | A `carried_fields` entry with a blank or malformed `name`, a blank `key` or an empty `from` fails the declaring module's load naming the entry. | Test |
| FR-090-AC-8 | An entry whose `from` names an archetype that no loaded module declares, or whose `key` is a property of none of the `from` frontmatter schemas and carried by no document, reports `trace:undeclared-relation-vocabulary` naming the entry when the corpus is merged; the same entry in a module whose `from` archetype another loaded module declares reports nothing and does not fail that module's load. | Test |
| FR-090-AC-9 | The top-level coverage report carries `carried_vocabulary.software_safety_class` equal to the `enum` of the key in the `from` schemas, in schema order, and no entry for a name whose key has no `enum`; `coverage_matrix` remains an array of entries. | Test |
| FR-090-AC-10 | Two entries with the same `name` and `key` in two modules merge to one field whose `from` is the union; the same `name` with different `key` fails the merge naming both modules. | Test |
| FR-090-AC-11 | The published coverage schema validates a run's output carrying `carried` on a requirement entry and `carried_vocabulary` on the top-level report, and still rejects any other unknown key on the entry and on the top-level report. | Test |
| FR-090-AC-12 | Running the matrix leaves every document byte-identical, and two runs over an identical corpus serialize identically, with and without the keys. | Test |
| FR-090-AC-13 | When the `from` archetypes' `enum`s for the key are identical except that one archetype (an open frontmatter) has none, the vocabulary is the existing enum whichever module loads first; when two existing `enum`s differ, `trace:carried-vocabulary-mismatch` names the entry, both archetypes and modules, and no vocabulary entry is emitted. | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-090-CON-1 | The engine holds no frontmatter key name, archetype name or value list for any carried field. | Design | Inspection |
| FR-090-CON-2 | No code path assigns, defaults, infers or writes back a carried value. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md), [FR-082](./FR-082-coverage-matrix-criticality.md), [FR-059](./FR-059-declared-vocabulary-coverage.md), [FR-058](./FR-058-upward-trace-completeness.md)
- **Downstream**: [FR-091](./FR-091-ordered-field-consistency.md), [spec-artifacts-iso FR-009](ix://agent-ix/spec-artifacts-iso/FR-009) (declares the entry; lands after this engine requirement), [quoin FR-160](ix://agent-ix/quoin/FR-160) and [quoin FR-161](ix://agent-ix/quoin/FR-161) (read `carried` and `carried_vocabulary`), quire-cli FR-028 renders the value
