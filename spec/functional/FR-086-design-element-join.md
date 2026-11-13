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

`quire-rs` SHALL produce the design-element join for any loaded corpus and any set of declared design
relations, including none: for each relation named in the loaded modules' `design_relations`, in that
order, one row per requirement the relation obliges, each listing the design elements that realize it
and, when there are none, a row with `state: uncovered`.

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

[FR-084](./FR-084-two-way-relation-coverage-report.md)-CON-2 forbids the *report* a manifest key of
its own, because the report reports the FR-058 declarations. `design_relations` is a key of this
join, not of the report: FR-084 stays keyless and reports every declared relation whether or not it is
named here.

A `design_relations` entry that is blank, or listed twice, SHALL fail module load naming the entry
(the manner of FR-058-AC-10). An entry cannot be checked against `required_relations` at load, because
the relation may come from another module (FR-058). So an entry that names no relation declared by
any loaded module SHALL report under `trace:undeclared-relation-vocabulary`, naming the entry, and
the join, which runs with every module loaded, SHALL fail with an error naming the entry and produce
no report, as FR-084-AC-9 does for an undeclared relation name.

### The join

`design_join` takes the corpus and the per-member inputs FR-084 takes, and returns the FR-084 report
for the design relations alone, in `design_relations` order, serialized as FR-084 serializes it. A
row's `document` is the requirement, its `linked` entries are the design elements, each with its
`archetype` as the element's kind, its `id`, `path`, `repo` when present, and the accepted verb.
Everything FR-084 decides is unchanged: `state`, `dangling`, `kind_mismatch`, ordering, `summary` and
determinism.

**The gap is a row, never an absence.** A requirement with no design element is a row with
`state: uncovered`, empty `linked` and the relation's `check` token, so the reader sees which
requirement has no design element and which declaration asked. A requirement is never dropped from
the join because it has no edge, or only a dangling or wrong-kind one. A wrong-kind source under an
`incoming` relation leaves the requirement a gap and yields no `kind_mismatch` entry
(FR-085-AC-25); only a relation whose direction is `outgoing` or `either` lists `kind_mismatch`.

**Either end may author the edge.** The join reads edges as FR-058 does, including the counterpart
reading of FR-058 (CR-416): the design element may author the realizing verb toward the requirement,
or the requirement may author the declared inverse toward the design element, and either spelling
joins the pair once, as one `linked` entry carrying the listed verb. The engine knows neither verb.

**Cross-repository elements resolve.** A design element held in another corpus member joins by
FR-085: `repo` names its member. A requirement that authors the inverse toward a member not loaded
(or an id absent from a loaded member) leaves a `dangling` entry naming the reference as authored,
and, for a non-empty `to`, a gap. An edge the design element authors in an unloaded member is not in
the corpus, so the requirement is a gap there; the join sees what is loaded, as FR-085 states for
resolution generally.

**Criteria.** A requirement's acceptance criteria are rows in its table, not documents, and are
addressed by their own ids, which are minted by a trace target named by an `obligations:` source ([FR-049](./FR-049-verification-reference-integrity.md),
[FR-050](./FR-050-declarative-coverage-computation.md)). A design element that realizes a criterion
authors the edge directly to the criterion id. FR-026 classifies such an edge `Dangling`, so FR-058
(CR-417) reads it, for relation matching, as an edge to the requirement document that minted the
criterion. The join therefore attributes the element to the owning requirement, and each resolved
`linked` entry carries `"realized_criteria": ["<id>", ...]`: the distinct criterion ids of the pair's
edges, in lexical order, `[]` when the element realizes the requirement as a whole. The engine does not
report which criteria no element realizes. The key is `realized_criteria`, because the `criteria` key
of a document entry is FR-084's counts object. This needs a trace target named by an `obligations:` source (the module's own declaration, no engine
name); with none, an edge to a criterion id stays dangling and the requirement is a gap.

### No design relations

A corpus whose loaded modules declare no `design_relations` yields `{"relations": []}`.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-086-AC-1 | Given a module naming one relation in `design_relations`, the join over a corpus of two requirements, one with a realizing edge from a design element and one with none, returns two rows: the first `covered` with that element in `linked` carrying its `id` and `archetype`, the second `uncovered` with empty `linked`. | Test |
| FR-086-AC-2 | A requirement whose only accepted-verb edge is dangling (it authors the declared inverse toward an absent id, `to` non-empty) is a row, `uncovered`, with the reference in `dangling` as authored, and is not omitted. | Test |
| FR-086-AC-3 | A relation not named in `design_relations` produces no row in the join, while FR-084 over the same corpus reports it. | Test |
| FR-086-AC-4 | Two relations in `design_relations` (two requirement archetypes) yield two relation blocks in `design_relations` order, each with its own `summary`. | Test |
| FR-086-AC-5 | A requirement joined by an edge the design element authors and one joined by the declared inverse the requirement authors each appear `covered` with that element; a pair authored both ways yields one `linked` entry carrying the listed verb. | Test |
| FR-086-AC-6 | In a two-member corpus, a design element in member `o/b` realizing a requirement in `o/a` appears in the requirement's `linked` with `repo` `o/b`. In the same corpus where the requirement instead authors the declared inverse toward `ix://o/b/<id>` and `o/b` is not loaded, the requirement is `uncovered` (`to` non-empty) and the reference is in `dangling` as authored. | Test |
| FR-086-AC-7 | An edge the design element authors, from an archetype outside the relation's non-empty `to`, under an `incoming` relation does not join, leaves the requirement a gap, and yields no `kind_mismatch` entry (FR-085-AC-25). Under an `outgoing` relation the same wrong-kind target is listed in `kind_mismatch` and the requirement is a gap. | Test |
| FR-086-AC-8 | A design element whose edge targets an acceptance-criterion id of requirement `R` joins to `R`: `R` is `covered`, the element is in `linked` with `realized_criteria` listing that id, and an element with an edge to `R` itself carries `"realized_criteria": []`; two criterion ids of `R` in one pair are listed in lexical order, once each. | Test |
| FR-086-AC-9 | A corpus whose modules declare no `design_relations` yields `{"relations": []}`. | Test |
| FR-086-AC-10 | Module load fails naming the entry for a blank or duplicate `design_relations` entry; an entry naming no declared relation reports `trace:undeclared-relation-vocabulary` naming it, and the join fails with an error naming it and produces no report. | Test |
| FR-086-AC-11 | A relation in `design_relations` also reports its FR-058 `trace:<check>` finding for a requirement without a design element, advisory under both postures, and mapping its key `off` drops the finding without changing the join. | Test |
| FR-086-AC-12 | Over a fixture whose relation, archetype, verb, inverse and criterion-id spellings are all unrelated to requirements or design, the join is complete and correct. | Test |
| FR-086-AC-13 | The engine source contains no requirement or design archetype name, no realizing verb and no inverse label. | Inspection |
| FR-086-AC-14 | Two runs over one corpus give byte-identical joins independent of member and document order ([NFR-006](../non-functional/NFR-006-determinism.md)), and running the join leaves every document byte-identical. | Test |
| FR-086-AC-15 | An edge to a criterion id, with no `obligations:` source in the loaded modules, or whose id is minted only by a trace target no `obligations:` source names, leaves the requirement a gap and the edge in `dangling` as authored. | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-086-CON-1 | `design_relations` SHALL survive both `is_empty` and the cross-module merge (FR-058-CON-1), merged in module order without duplicates. | Design | Test |
| FR-086-CON-2 | The join SHALL introduce no row, field or state beyond FR-084's, except the `realized_criteria` list of a resolved `linked` entry. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-058](./FR-058-upward-trace-completeness.md) (CR-416, CR-417), [FR-084](./FR-084-two-way-relation-coverage-report.md), [FR-085](./FR-085-cross-repository-edge-resolution.md), [FR-041](./FR-041-authorable-inverse-edges.md), [FR-026](./FR-026-intra-spec-reference-resolution.md), [FR-049](./FR-049-verification-reference-integrity.md), [FR-050](./FR-050-declarative-coverage-computation.md), [FR-053](./FR-053-obligation-record.md)
- **Downstream**: the `quire-cli` relations subcommand's design mode (quire-cli FR-041); `spec-objects-architecture` FR-010, which declares the realizing verb and the design relations; the matrix design column (PLAT-1529)
