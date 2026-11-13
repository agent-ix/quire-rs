---
id: FR-091
title: "Ordered-field consistency and required-field findings, declared as module data"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-058"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-057"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-090"
    type: "references"
  - target: "ix://agent-ix/spec-objects-architecture/FR-012"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "traces_to"
---
# FR-091: Ordered-field consistency and required-field findings, declared as module data

Tracked by Linear PLAT-1526 (implemented by PLAT-1578).

## Description

[FR-058](./FR-058-upward-trace-completeness.md)'s `required_relations` ask whether an edge exists. They cannot ask
whether the two documents an edge joins agree on a value: a requirement that states a software safety class lower than
the class of the item it is allocated to, for instance, has every edge it needs and is still wrong. This requirement adds
two declarations beside `required_relations`, both module data, both reporting only.

### `field_consistency`

```yaml
traceability:
  field_consistency:
    - name: requirement-class-below-allocated-item
      from: FR                               # archetype stating the value (a list is allowed)
      field: software_safety_class           # frontmatter key on the from document
      edges: [allocated_to]
      direction: outgoing
      to: [part]
      to_field: safetyClass.class            # record key path on the target
      order_from: { archetype: FR, field: software_safety_class }
      rule: not_below
      check: class-below-item
```

For each `from` document that states `field`, and each document reached over `edges` in `direction`, in a `to`
archetype, that states `to_field`: under `rule: not_below` the `from` value SHALL NOT rank lower than the target value.
The rank is the position in the `enum` of `order_from`, read from the frontmatter schema as
[FR-059](./FR-059-declared-vocabulary-coverage.md) reads a vocabulary; the engine holds no list and no weight. A
violation is a finding under `trace:<check>` ([FR-057](./FR-057-corpus-check-severity.md)), advisory by default,
naming both documents, both fields and both values as written. A pair where either side states no value is not
compared, because absence is the next declaration's concern. A stated value outside the `enum` is reported once per
document under `trace:undeclared-field-value`, naming the document, field and value, and that document is not compared.

### `required_fields`

```yaml
traceability:
  required_fields:
    - name: item-needs-class
      from: [part]
      field: safetyClass.class
      exclude: []
      check: unassigned-class
```

A `from` document that states no `field` (absent, null or empty) yields a `trace:<check>` finding naming the document
and the field. `exclude` and the FR-058 `where` selector narrow the obliged documents exactly as for
`required_relations`. A module that declares no `required_fields` entry requires nothing. A module may ship an entry with its
`trace:<check>` severity set to `off` in the module's severity defaults ([FR-057](./FR-057-corpus-check-severity.md));
a project that enables the check thereby declares the field required, and the engine has no other switch.

### Reports, never assigns

Neither declaration produces a value. The engine SHALL NOT write a field to any document, SHALL NOT emit a key
carrying a computed or suggested value, and SHALL NOT use the rank of one document to fill in or raise another. The
findings state a disagreement or an absence between things people wrote.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-091-AC-1 | With the `field_consistency` entry above, an `FR` stating `class-1` allocated to a part whose `safetyClass.class` is `class-3` yields one `trace:class-below-item` finding naming both documents and both values. | Test |
| FR-091-AC-2 | An `FR` stating `class-3` allocated to a `class-1` part, and one stating the same class as its part, yield no finding. | Test |
| FR-091-AC-3 | An `FR` stating no class, and a part stating no class, each yield no consistency finding for the pair. | Test |
| FR-091-AC-4 | A requirement allocated to two parts of different classes yields a finding for each part whose class outranks the requirement's, and none for the others. | Test |
| FR-091-AC-5 | With the `required_fields` entry above, a part with no `safetyClass.class` yields one `trace:unassigned-class` finding naming the part and the field; a part with one yields none. | Test |
| FR-091-AC-6 | A corpus whose module declares neither entry yields no finding and output byte-identical to before this requirement. | Test |
| FR-091-AC-7 | A stated value outside the `order_from` `enum` yields one `trace:undeclared-field-value` finding and that document is excluded from comparison. | Test |
| FR-091-AC-8 | Findings are advisory by default and tunable per `check` through the FR-057 registry. | Test |
| FR-091-AC-9 | After a run over a corpus with findings of each kind, every document is byte-identical and no output key carries a class value that no document states. | Test |
| FR-091-AC-10 | A `field_consistency` entry whose `from`, `to`, `field` or `order_from` names an undeclared archetype or a property absent from its schema, or a `rule` other than `not_below`, fails the module load naming the entry. | Test |
| FR-091-AC-11 | Two runs over an identical corpus produce findings in identical order. | Test |
| FR-091-AC-12 | Given the manifest data above and fixture corpora, no engine source mentions `software_safety_class`, `safetyClass`, `allocated_to` or `part`. | Inspection |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-091-CON-1 | The engine holds no archetype, field, verb, rank or value for either declaration. | Design | Inspection |
| FR-091-CON-2 | No code path assigns, suggests or writes a field value; findings only name values people wrote. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-058](./FR-058-upward-trace-completeness.md), [FR-057](./FR-057-corpus-check-severity.md), [FR-059](./FR-059-declared-vocabulary-coverage.md), [FR-090](./FR-090-declared-carried-fields.md)
- **Downstream**: the iso and architecture modules declare the entries ([spec-artifacts-iso FR-009](ix://agent-ix/spec-artifacts-iso/FR-009), [spec-objects-architecture FR-012](ix://agent-ix/spec-objects-architecture/FR-012)); the design-join work that supplies allocation reads is tracked separately (PLAT-1417)
