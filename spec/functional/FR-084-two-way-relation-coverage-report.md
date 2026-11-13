---
id: FR-084
title: "Two-way relation coverage report"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-058"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-085"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
---
# FR-084: Two-way relation coverage report

## Description

[FR-058](./FR-058-upward-trace-completeness.md) reports a document that lacks a required edge as an
advisory finding. A finding list names the gaps and nothing else: it does not show what covers the
documents that pass, and it does not say whether that cover is verified. This FR adds a **report**
over the same declarations: one row per obliged document, listing what is linked to it and how well
that is verified.

The report is a function of the loaded corpus, the module's `traceability.required_relations`, and
the `coverage_matrix` of [FR-050](./FR-050-declarative-coverage-computation.md). It adds no
declaration, no walk and no second graph.

### One report, both directions

Each declared relation yields a **row set**: one row for every document the relation obliges, which
is the FR-058 selection (`from` kind, then `where`, then `exclude`). Because a relation carries a
`direction`, the two views the safety and security chains need are two declarations over the same
verb, read from opposite ends, and the engine tells them apart by nothing but the declaration:

- a relation with `direction: incoming` over a risk-bearing kind gives the **forward view**: each
  such document with the documents that point at it;
- a relation with `direction: outgoing` and a `where` selector over a requirement kind gives the
  **reverse view**: each selected requirement with the documents it points at. A selected
  requirement with none is a **gap**.

The engine holds no archetype, verb, field or value belonging to either chain
([FR-058](./FR-058-upward-trace-completeness.md)-CON-2 and CON-3 apply unchanged).

### The row

A row carries:

| Field | Meaning |
|---|---|
| `relation` | The declaration's `name` |
| `check` | The declaration's `check` token, so a consumer can tell `unmitigated-hazard` from `uncontrolled-threat` without knowing either word |
| `direction` | The declaration's `direction` |
| `document` | The obliged document: `id`, `path`, `archetype`, and `repo` when its corpus member declares an identity ([FR-085](./FR-085-cross-repository-edge-resolution.md)) |
| `selector` | Present only when the declaration has `where`: the `field` and the document's value as authored |
| `state` | `uncovered` or `covered` |
| `verification` | The aggregate verification status of the row, below |
| `linked` | The documents on the other end of an accepted edge, ordered by repo, then id |
| `dangling` | Accepted-verb edges whose target is absent, each naming the reference as authored |
| `kind_mismatch` | Accepted-verb edges whose target resolved to an archetype outside `to`, each naming the target and its archetype |

A `linked` entry carries the document's `id`, `path`, `archetype`, `repo` when present, the accepted
verb, and its own `verification`.

`state` is `uncovered` when `linked` is empty and `covered` otherwise. A `dangling` or
`kind_mismatch` entry never makes a row covered: it is evidence of an attempted edge that does not
count ([FR-058](./FR-058-upward-trace-completeness.md), [FR-085](./FR-085-cross-repository-edge-resolution.md)).

### Verification status comes from the matrix

The verification status of a document is read from its `requirements[]` entry in the
`coverage_matrix` ([FR-050-AC-48](./FR-050-declarative-coverage-computation.md)); the report
computes no second notion of tested. A criterion is **backed** when its computed `status`
([FR-050-AC-49](./FR-050-declarative-coverage-computation.md)) is `tagged` or `qualified-record`. A
criterion with status `retired` is not counted. The document's status is:

| Status | Condition |
|---|---|
| `verified` | At least one non-retired criterion, and every one is backed |
| `partial` | At least one backed and at least one non-retired criterion not backed |
| `unverified` | At least one non-retired criterion, none backed |
| `not-tracked` | No `requirements[]` entry for the document, or an entry with no non-retired criterion |

A row's aggregate `verification` is computed over its **tracked** documents: the obliged document
and every `linked` document whose status is not `not-tracked`. It is `verified` when all are
`verified`, `unverified` when none is `verified` or `partial`, `partial` otherwise, and
`not-tracked` when no document is tracked. The aggregate is the same rule for both views: in the
forward view the tracked documents are the mitigating requirements, and in the reverse view the
obliged requirement itself, since a risk-bearing document carries no criteria.

A module declaring no `obligations:` source has no `coverage_matrix` ([FR-050-AC-51](./FR-050-declarative-coverage-computation.md));
every status is then `not-tracked` and the report is still produced.

### Selecting and ordering

The report covers every declared relation unless a caller names relations, in which case it covers
exactly those, and naming a relation no module declares is an error naming it. Rows are ordered by
relation in declaration order, then by repo, document path and id. Output is identical across runs
over one corpus ([NFR-006](../non-functional/NFR-006-determinism.md)).

### Report, not verdict

The report states what is linked and verified. It carries a `summary` per relation: row count and
the count in each `state` and `verification`. Whether a gap blocks anything is policy for the
consumer, and the engine writes nothing back to any document.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-084-AC-1 | Given a relation `from: K, edges: [e], direction: incoming` and three `K` documents, one with no incoming `e` edge, one with an incoming `e` edge from a document with all criteria tagged, one with an incoming `e` edge from a document with no criteria tagged, the report has three rows: the first `uncovered`, the second `covered` and `verified`, the third `covered` and `unverified`. | Test |
| FR-084-AC-2 | A row's `linked` lists each accepted-edge counterpart with its archetype and its own `verification`, ordered by repo then id, and a counterpart reached by a verb outside `edges` is not listed. | Test |
| FR-084-AC-3 | Given a relation `from: R, where: {field: F, values: [V]}, edges: [e], to: [K], direction: outgoing`, a `R` document with `F: V` and no `e` edge to a `K` document is a row with `state: uncovered` (a gap), one with an edge is `covered`, and a `R` document whose `F` is outside `values` or absent has no row. | Test |
| FR-084-AC-4 | A row from a relation with `where` carries `selector` with the field and the document's value as authored; a row from a relation without `where` has no `selector` key. | Test |
| FR-084-AC-5 | Document verification is derived from the `coverage_matrix` criteria: all non-retired criteria `tagged` or `qualified-record` gives `verified`; a mix gives `partial`; none backed (`untagged`, `tagged-by-ignored-test`, `method-without-symbol`) gives `unverified`; a retired criterion is not counted; a document with no entry or only retired criteria gives `not-tracked`. | Test |
| FR-084-AC-6 | A hazard-like row linked only to documents with status `not-tracked` is `covered` with `verification: not-tracked`; a row linked to one `verified` and one `unverified` document is `partial`; the aggregate ignores `not-tracked` documents when any tracked document exists. | Test |
| FR-084-AC-7 | In the reverse view the row aggregate is that of the obliged requirement itself when its linked documents are `not-tracked`, so a classified requirement with a tagged criterion and a linked risk document is `covered` and `verified`. | Test |
| FR-084-AC-8 | A module declaring no `obligations:` source yields rows with every `verification` equal to `not-tracked`, and the report is produced without error. | Test |
| FR-084-AC-9 | Naming a relation that no loaded module declares is an error naming the relation and produces no report; naming a declared relation produces rows for it alone. | Test |
| FR-084-AC-10 | A relation selecting zero documents yields a `summary` with zero rows rather than an absent relation, so an empty selection is distinguishable from an unreported relation. | Test |
| FR-084-AC-11 | The `summary` of each relation counts its rows by `state` and by `verification`, and the counts sum to the row count. | Test |
| FR-084-AC-12 | Two runs over one corpus produce byte-identical serialized reports, ordered by relation declaration order, then repo, path and id ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test |
| FR-084-AC-13 | Running the report leaves every document byte-identical and raises no FR-058 finding or severity that the same corpus does not already raise without it. | Test |
| FR-084-AC-14 | Over a fixture whose relation, kind, verb and field names are all unrelated to hazards or threats, the report is complete and correct, and the engine source contains no hazard, threat, mitigation or safety-classification name ([FR-058](./FR-058-upward-trace-completeness.md)-CON-2). | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-084-CON-1 | The report SHALL derive verification only from the `coverage_matrix`. | Design | Inspection |
| FR-084-CON-2 | The report SHALL NOT introduce a manifest key: the relations it reports are the FR-058 declarations. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-058](./FR-058-upward-trace-completeness.md) (the declarations), [FR-050](./FR-050-declarative-coverage-computation.md) (the matrix), [FR-085](./FR-085-cross-repository-edge-resolution.md) (edge resolution across repositories)
- **Downstream**: the `quire-cli` relations subcommand (quire-cli FR-026), which renders this report
