---
id: FR-091
title: "Ordered-field consistency, required-field and vocabulary findings, declared as module data"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-058"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-057"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-059"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-085"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-090"
    type: "references"
  - target: "ix://agent-ix/spec-artifacts-iso/FR-009"
    type: "requires"
  - target: "ix://agent-ix/spec-objects-architecture/FR-012"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "traces_to"
---
# FR-091: Ordered-field consistency, required-field and vocabulary findings, declared as module data

Tracked by Linear PLAT-1526 (implemented by PLAT-1578).

## Description

[FR-058](./FR-058-upward-trace-completeness.md)'s `required_relations` ask whether an edge exists. They cannot ask whether the
two documents an edge joins agree on a value: a requirement that states a software safety class lower than the class of the part
it is allocated to has every edge it needs and is still wrong. This requirement adds three declarations beside
`required_relations`, all module data, all reporting only.

### `field_consistency`

The allocation of a requirement to a part is two hops in the architecture vocabulary: an `allocation` document authors `allocates`
toward the requirement and `allocated_to` toward the part. A declaration therefore carries an optional intermediate hop, `via`:

```yaml
traceability:
  field_consistency:
    - name: requirement-class-below-allocated-part
      from: FR                          # archetype stating the value (a list is allowed)
      field: software_safety_class      # top-level frontmatter key on the from document
      via: { archetype: allocation, edges: [allocates], direction: incoming }
      edges: [allocated_to]             # from the via document (or from the from document when no via)
      direction: outgoing
      to: [part]
      to_field: software_safety_class   # top-level frontmatter key on the target document
      order_from: { archetype: FR, field: software_safety_class }
      rule: not_below
      check: class-below-part
```

`field` and `to_field` are both **top-level frontmatter keys**, read as [FR-082](./FR-082-coverage-matrix-criticality.md) and
[FR-090](./FR-090-declared-carried-fields.md) read them; no extracted body-record key is read. The path is `from` document, then
the `via` hop when present (`direction: incoming` means the `via` document authors the verb toward the `from` document), then the
`edges` hop in `direction` to a document of an archetype in `to`. The pair compared is the `from` document and the target
document.

For each pair where both documents state their field, under `rule: not_below` the engine SHALL report a `trace:<check>` finding
when the `from` value ranks lower than the target value ([FR-057](./FR-057-corpus-check-severity.md), advisory by default),
naming the `from` document, **every** `via` document that reaches the target (one finding per `from`/target pair, the `via` documents listed in lexical id order), the target document, both fields and both values as written. The rank is the
position of the value in the `enum` of `order_from`, read from the frontmatter schema as
[FR-059](./FR-059-declared-vocabulary-coverage.md) reads a vocabulary; the engine holds no list and no weight. A pair where either
side states no value is not compared, because absence is the next declaration's concern. When a stated value, on either side, is
outside the `enum`, the engine SHALL report once per document a `trace:undeclared-field-value` finding naming the document, the
field and the value, and SHALL NOT compare that document.

### How edges are read

Every hop is read as [FR-058](./FR-058-upward-trace-completeness.md) relation matching reads an edge: an edge authored with the
declared counterpart label from the opposite end is read as the listed verb (CR-416, quire-rs#668), a pair authored both ways
counts once, and the edge set itself is unchanged. A dangling edge, or an edge whose far end is outside the hop's archetypes, joins
nothing and the pair is not compared. A far end held in another corpus member resolves as [FR-085](./FR-085-cross-repository-edge-resolution.md)
resolves it; one in a member that is not loaded is dangling and not compared. Several routes between the same two documents
yield one pair and one finding.

### `required_fields`

```yaml
traceability:
  required_fields:
    - name: part-needs-class
      from: [part]
      field: software_safety_class     # top-level frontmatter key on the from document
      exclude: []
      check: unassigned-class
```

When a `from` document states no `field` (absent, null or empty), the engine SHALL report a `trace:<check>` finding naming the
document and the field. `exclude` and the FR-058 `where` selector narrow the obliged documents exactly as for `required_relations`.
A module that declares no `required_fields` entry requires nothing. A module may ship an entry with its `trace:<check>` severity set
to `off` in the module's severity defaults ([FR-057](./FR-057-corpus-check-severity.md)); a project that enables the check thereby
declares the field required, and the engine has no other switch.

### `vocabulary`

```yaml
traceability:
  vocabulary:
    - name: part-class-in-vocabulary
      from: [part]
      field: software_safety_class     # top-level frontmatter key on the from document
      carried: software_safety_class   # the carried_fields name whose vocabulary applies (FR-090)
      check: undeclared-class-value
```

For **every document of every `from` archetype**, whether or not it takes part in any compared pair of a `field_consistency` entry, when it states `field` with a value that is not in the run's `carried_vocabulary` for `carried`
([FR-090](./FR-090-declared-carried-fields.md)), the engine SHALL report a `trace:<check>` finding (advisory by default) naming
the document, the field and the value as written. This is how a document whose archetype holds no `enum` of its own, such as an
object type with open frontmatter, is held to the vocabulary another module declares, so that no module copies the list. Matching is
exact (no case folding or trimming). A document that states no value yields nothing here (`required_fields` covers absence). When
`carried_vocabulary` has no entry for `carried`, the entry reports `trace:undeclared-relation-vocabulary` naming it and compares
nothing. The entry is validated like the others: structural defects at its module's load, archetype, field and carried-name existence
at the corpus merge.

### Ownership and when a declaration is checked

An entry belongs to the module that declares it; the architecture module owns the entries above, although `FR` and its field are
declared by the ISO module. Structural defects fail the declaring module's load naming the entry: a blank `name` or `check`,
an empty `from`, `edges` or `to`, a `via` without `archetype` or `edges`, a `direction` other than `incoming` or `outgoing`, a `rule`
other than `not_below`, a `vocabulary` entry without `carried` or `field`, or a duplicate `name` within the module. Whether an archetype, field or `order_from` exists is checked
when the corpus is merged, against the merged registry of every loaded module, and never at single-module load: an unknown archetype,
or a `field`, `to_field` or `order_from` that is a property of none of the named archetypes' frontmatter schemas and is carried by
none of their documents, reports under `trace:undeclared-relation-vocabulary` naming the entry, as FR-058 reports a `from` that
names a kind nothing has. An `order_from` field with no `enum` reports the same way, because no rank exists.

### Reports, never assigns

No declaration produces a value. The engine SHALL NOT write a field to any document, SHALL NOT emit a key carrying a computed
or suggested value, and SHALL NOT use the rank of one document to fill in or raise another. Findings are ordered by `check`, then
`from` document path, then the first (lowest) `via` id, then target document id, all lexical, so that the order does not depend on the order
documents are read.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-091-AC-1 | With the `field_consistency` entry above, an `FR` stating `class-1`, an `allocation` that `allocates` it and is `allocated_to` a `part` stating `class-3` yields one `trace:class-below-part` finding naming the `FR`, the `allocation`, the `part` and both values. | Test |
| FR-091-AC-2 | An `FR` stating `class-3` allocated to a `class-1` part, and one stating the same class as its part, yield no finding. | Test |
| FR-091-AC-3 | An `FR` stating no class, and a part stating no class, each yield no consistency finding for the pair. | Test |
| FR-091-AC-4 | A requirement allocated through two allocations to two parts of different classes yields a finding for each part whose class outranks the requirement's, and none for the others; two allocations reaching the same part yield exactly one finding for that pair, which names both `via` documents in lexical id order. | Test |
| FR-091-AC-5 | With the `required_fields` entry above, a `part` with no `software_safety_class` yields one `trace:unassigned-class` finding naming the part and the field; a part with one yields none. | Test |
| FR-091-AC-6 | A corpus whose modules declare none of the three (`field_consistency`, `required_fields`, `vocabulary`) yields no finding and output identical to the output before this requirement. | Test |
| FR-091-AC-7 | A stated value outside the `order_from` `enum`, on either side, yields one `trace:undeclared-field-value` finding for that document and that document is excluded from comparison. | Test |
| FR-091-AC-8 | Findings are advisory by default and tunable per `check` through the FR-057 registry; a `part` fixture with the severity key `off` produces no `trace:unassigned-class` finding. | Test |
| FR-091-AC-9 | After a run over a corpus with findings of each kind, every document is byte-identical and no output key carries a class value that no document states. | Test |
| FR-091-AC-10 | An entry with a blank `name`, empty `from`, `edges` or `to`, a `via` lacking `archetype` or `edges`, an unknown `rule` or `direction`, or a duplicate `name` within the module fails that module's load naming the entry. | Test |
| FR-091-AC-11 | An entry in the architecture module naming the ISO module's `FR` archetype and field loads without error; with both modules loaded it works, and with the ISO module not loaded the merged corpus reports `trace:undeclared-relation-vocabulary` naming the entry. | Test |
| FR-091-AC-12 | An `FR` whose allocation is authored on the allocation as the declared inverse from the requirement end, and a pair authored both ways, are each compared once. | Test |
| FR-091-AC-13 | An allocation whose `allocated_to` target is dangling, one whose target is a `port` rather than a `part`, and one whose target is in an unloaded member yield no finding and no comparison; a target in a loaded other member is compared. | Test |
| FR-091-AC-14 | Findings are in `check`, `from` path, lowest `via` id, target id order, identical for two runs and for the corpus documents supplied in reverse order. | Test |
| FR-091-AC-15 | A fixture corpus whose manifest names neutral archetypes, fields and verbs (for example `alpha`, `beta`, `rank_field`, `links_to`) and no name from this specification yields the same findings, so the engine holds no name. | Test |
| FR-091-AC-16 | A corpus where a `field_consistency` entry's `order_from` names a field with no `enum` reports `trace:undeclared-relation-vocabulary` naming the entry and compares nothing. | Test |
| FR-091-AC-17 | With the `vocabulary` entry above, a `part` stating `class-9` or `Class-1` yields one `trace:undeclared-class-value` finding naming the part, the field and the value; one stating a value in `carried_vocabulary` yields none; one stating nothing yields none. | Test |
| FR-091-AC-18 | A `vocabulary` entry whose `carried` name has no `carried_vocabulary` entry reports `trace:undeclared-relation-vocabulary` naming the entry and produces no value finding; the same finding set results whichever module loads first. | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-091-CON-1 | The engine holds no archetype, field, verb, rank or value for either declaration. | Design | Test (FR-091-AC-15) |
| FR-091-CON-2 | No code path assigns, suggests or writes a field value; findings only name values people wrote. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-058](./FR-058-upward-trace-completeness.md) (relation matching incl. CR-416), [FR-057](./FR-057-corpus-check-severity.md), [FR-059](./FR-059-declared-vocabulary-coverage.md), [FR-085](./FR-085-cross-repository-edge-resolution.md), [spec-artifacts-iso FR-009](ix://agent-ix/spec-artifacts-iso/FR-009) (the field and its enum), [FR-090](./FR-090-declared-carried-fields.md) (the same frontmatter reading)
- **Downstream**: [spec-objects-architecture FR-012](ix://agent-ix/spec-objects-architecture/FR-012) declares the entries; they land after this engine requirement. The design-join work (PLAT-1417, FR-086 (quire-rs#668, merging first)) is a separate report over the same edges.
