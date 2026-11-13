---
id: FR-086
title: "Design-element join from requirement"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-058"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-084"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-085"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-041"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
---
# FR-086: Design-element join from requirement

## Description

A requirement with no design element is a requirement nobody decided how to build, and a design
element no requirement asks for is scope nobody asked for. [FR-084](./FR-084-two-way-relation-coverage-report.md)
already reports, per obliged document, what is linked to it across the edges an
[FR-058](./FR-058-upward-trace-completeness.md) relation declares, including across repositories
([FR-085](./FR-085-cross-repository-edge-resolution.md)). The **design-element join** is that report
over the relations a module names as its design relations: one row per requirement, listing the design
elements that realize it, and an explicit gap when there are none.

It is not a new walk, index or graph. The engine holds **no archetype name and no verb**: which
documents are requirements, which are design elements, and which verb realizes, are all the module's
`required_relations` entries and its `edge_types` vocabulary.

### The declaration

The module's `traceability` block MAY carry `design_relations`, an ordered list of relation names,
each naming a `required_relations` entry:

```yaml
traceability:
  design_relations: [fr-has-design-element, nfr-has-design-element]
```

A relation's `from` names one archetype, so requirements of several archetypes are several
entries. Each named relation is an ordinary FR-058 relation: it reports its own advisory
`trace:<check>` finding ([FR-057](./FR-057-corpus-check-severity.md)), tunable or switchable off like
any other. `design_relations` adds no obligation and no severity.

A `design_relations` entry that is blank, or listed twice, SHALL fail module load naming the entry
(the manner of FR-058-AC-10). An entry that names no relation declared by any loaded module cannot be
checked at load, because the relation may come from another module (FR-058); it SHALL report under
`trace:undeclared-relation-vocabulary`, naming the entry.

### The join

`design_join` takes the corpus and the per-member inputs FR-084 takes, and returns the FR-084 report
for the design relations alone, in `design_relations` order, serialized as FR-084 serializes it. A
row's `document` is the requirement, its `linked` entries are the design elements, each with its
`archetype` as the element's kind, its `id`, `path`, `repo` when present, and the verb as authored.
Everything FR-084 decides is unchanged: `state`, `dangling`, `kind_mismatch`, ordering, `summary` and
determinism.

**The gap is a row, never an absence.** A requirement with no design element is a row with
`state: uncovered`, empty `linked` and the relation's `check` token, so the reader sees which
requirement has no design element and which declaration asked. A requirement is never dropped from
the join because it has no edge, or only a dangling one.

**Either end may author the edge.** The join reads edges as FR-058 does, including the authored
inverse verbs of FR-058 (CR-416): the design element may author the realizing verb toward the
requirement, or the requirement may author the declared inverse toward the design element, and either
spelling joins the pair once. The engine knows neither verb.

**Cross-repository elements resolve.** A design element held in another corpus member joins by
FR-085: `repo` names its member, and a reference naming a member not loaded is a `dangling` entry
naming the reference as authored. The join sees what is loaded; a requirement whose design elements
live in a member the caller did not load is a gap in that corpus, as FR-085 states for resolution
generally.

**Criteria.** A requirement's acceptance criteria are rows in its table, not documents, so an edge
to a criterion resolves, by FR-026, to the requirement document: realizing one criterion joins the
element to the requirement. Where the authored target carries a `#fragment` equal to an acceptance-criterion
id of that requirement, the resolved `linked` entry carries `"criteria": ["<id>", ...]`, the
distinct such fragments of the pair in lexical order; with none it carries `"criteria": []`. The
engine does not report which criteria no element realizes.

### No design relations

A corpus whose loaded modules declare no `design_relations` yields `{"relations": []}`. Naming a
relation through the join that no loaded module lists in `design_relations` is an error naming it.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-086-AC-1 | Given a module naming one relation in `design_relations`, the join over a corpus of two requirements, one with a realizing edge from a design element and one with none, returns two rows: the first `covered` with that element in `linked` carrying its `id` and `archetype`, the second `uncovered` with empty `linked`. | Test |
| FR-086-AC-2 | A requirement whose only accepted-verb edge is dangling is a row, `uncovered`, with the reference in `dangling` as authored, and is not omitted. | Test |
| FR-086-AC-3 | A relation not named in `design_relations` produces no row in the join, while FR-084 over the same corpus reports it. | Test |
| FR-086-AC-4 | Two relations in `design_relations` (two requirement archetypes) yield two relation blocks in `design_relations` order, each with its own `summary`. | Test |
| FR-086-AC-5 | A requirement joined by an edge authored on the design element and one joined by the declared inverse authored on the requirement each appear `covered` with that element; a pair authored both ways yields one `linked` entry. | Test |
| FR-086-AC-6 | In a two-member corpus, a design element in member `o/b` realizing a requirement in `o/a` appears in the requirement's `linked` with `repo` `o/b`; with `o/b` not loaded the requirement is `uncovered` and the reference is in `dangling` as authored. | Test |
| FR-086-AC-7 | An edge from an element whose archetype is outside the relation's non-empty `to` does not join and is listed in `kind_mismatch`, the requirement being `uncovered`. | Test |
| FR-086-AC-8 | An edge whose target spelling is `<requirement>#<criterion-id>` joins the element to the requirement and the `linked` entry carries that criterion id in `criteria`; an edge with no fragment, or a fragment naming no criterion of the requirement, carries `"criteria": []`. | Test |
| FR-086-AC-9 | A corpus whose modules declare no `design_relations` yields `{"relations": []}`; naming a relation through the join that is not in `design_relations` is an error naming it and produces no report. | Test |
| FR-086-AC-10 | Module load fails naming the entry for a blank or duplicate `design_relations` entry; an entry naming no declared relation reports `trace:undeclared-relation-vocabulary` naming it, and the join errors naming it. | Test |
| FR-086-AC-11 | A relation in `design_relations` also reports its FR-058 `trace:<check>` finding for a requirement without a design element, advisory under both postures, and mapping its key `off` drops the finding without changing the join. | Test |
| FR-086-AC-12 | Over a fixture whose relation, archetype, verb and inverse names are all unrelated to requirements or design, the join is complete and correct. | Test |
| FR-086-AC-13 | The engine source contains no requirement or design archetype name, no realizing verb and no inverse label. | Inspection |
| FR-086-AC-14 | Two runs over one corpus give byte-identical joins independent of member and document order ([NFR-006](../non-functional/NFR-006-determinism.md)), and running the join leaves every document byte-identical. | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-086-CON-1 | `design_relations` SHALL survive both `is_empty` and the cross-module merge (FR-058-CON-1), merged in module order without duplicates. | Design | Test |
| FR-086-CON-2 | The join SHALL introduce no row, field or state beyond FR-084's, except the `criteria` list of a resolved `linked` entry. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-058](./FR-058-upward-trace-completeness.md), [FR-084](./FR-084-two-way-relation-coverage-report.md), [FR-085](./FR-085-cross-repository-edge-resolution.md), [FR-041](./FR-041-authorable-inverse-edges.md), [FR-026](./FR-026-intra-spec-reference-resolution.md)
- **Downstream**: the `quire-cli` relations subcommand's design mode (quire-cli FR-041); `spec-objects-architecture` FR-010, which declares the realizing verb and the design relations; the matrix design column (PLAT-1529)
