---
id: FR-085
title: "Cross-repository edge resolution"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
---
# FR-085: Cross-repository edge resolution

## Description

[FR-026](./FR-026-intra-spec-reference-resolution.md) bounds resolution to one loaded repository, so
an edge whose target lives in a sibling repository is `Dangling`, and under
[FR-058](./FR-058-upward-trace-completeness.md) it cannot satisfy a relation with a constrained
`to`. A safety requirement in one repository that mitigates a hazard held in another is therefore
reported as if it mitigated nothing.

This FR lets one corpus hold **several repositories** and resolves edges between them. Resolution
stays bounded to what is loaded: the engine still does no I/O beyond the roots it was given and
never fetches a reference.

### Corpus members

A corpus is built from one or more **members**, each a `RepoLoad` ([FR-024](./FR-024-parallel-repo-walk.md))
with an optional **identity** `<org>/<repo>`, supplied by the caller and never derived from content
or the filesystem. Two members declaring the same identity are an error naming it. A corpus of one
member with no identity behaves exactly as [FR-025](./FR-025-spec-corpus-model.md) and
[FR-026](./FR-026-intra-spec-reference-resolution.md) specify.

Document ids are unique within a member, not across members. A document is addressed across the
corpus by `(identity, id)`.

### Resolution

An `ix://<org>/<repo>/...` target contributes the pair `(<org>/<repo>, <ID>)`, where `<ID>` is the
trailing artifact id as in FR-026. In a corpus where at least one member declares an identity:

- if a member declares `<org>/<repo>`, the target is **Resolved** when that member holds `<ID>`, and
  **Dangling** when it does not;
- if no member declares `<org>/<repo>`, the target is **Dangling**, even when another member holds
  the same `<ID>`;
- a bare `<ID>` and a relative-path target resolve within the source document's own member only.

A `Dangling` edge records the target exactly as authored, so a report can name it.

A resolved cross-member edge appears in the source's outgoing set and in the target's incoming set
like any other, and [FR-058](./FR-058-upward-trace-completeness.md) relations read the corpus-wide
edge set: a document in member A that `mitigates` a document in member B satisfies a relation on B
with `direction: incoming`, and satisfies a relation on A with `direction: outgoing` when the target
archetype is in `to`.

### Kind of a resolved target

When a document has an edge whose verb is accepted by a relation but whose resolved target's
archetype is outside the relation's non-empty `to`, the edge SHALL NOT satisfy the relation, and the
engine SHALL report a **kind mismatch** under `trace:target-kind-mismatch`, advisory by default
([FR-057](./FR-057-corpus-check-severity.md)). The finding names the source document, the target
reference (with repo identity when the target is in another member), the target's actual archetype,
the accepted archetypes, and the relation. This applies equally within one member and across
members.

### Dangling references are named

A document whose only accepted-verb edges are `Dangling` is reported by its relation as
unsatisfied, and the finding SHALL name each dangling reference as authored. A `Dangling` edge
never satisfies a relation whose `to` is non-empty, whether the target repository is unloaded or
loaded without the id ([FR-058-AC-4](./FR-058-upward-trace-completeness.md)).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-085-AC-1 | A corpus built from two members with identities `o/a` and `o/b` resolves an `ix://o/b/...` edge from a document in `o/a` to the `o/b` document with that id, as `Resolved`, present in the target's incoming set. | Test |
| FR-085-AC-2 | A requirement in `o/a` with `mitigates` to a hazard-kind document in `o/b` satisfies a `direction: incoming` relation on that hazard, and the same corpus with `o/b` absent leaves the relation unsatisfied. | Test |
| FR-085-AC-3 | The same edge satisfies a `direction: outgoing` relation on the `o/a` requirement whose `to` names the hazard kind. | Test |
| FR-085-AC-4 | An `ix://o/b/...` target whose id is absent from loaded member `o/b` is `Dangling`, and the relation finding names the reference as authored. | Test |
| FR-085-AC-5 | An `ix://o/c/...` target naming a repository no member declares is `Dangling` even when `o/a` or `o/b` holds a document with the same id. | Test |
| FR-085-AC-6 | An edge to a resolved target of an archetype outside the relation's non-empty `to` does not satisfy the relation and yields one `trace:target-kind-mismatch` finding naming source, target reference, actual archetype, accepted archetypes and relation; the finding appears for a target in another member and for a target in the same member. | Test |
| FR-085-AC-7 | A relation with `to: []` has no kind mismatch, and a kind-mismatch finding is advisory by default and tunable through the FR-057 key `trace:target-kind-mismatch`. | Test |
| FR-085-AC-8 | The same id held by two members is not a duplicate; a bare id and a relative-path link resolve only within the source's member. | Test |
| FR-085-AC-9 | Two members declaring one identity is an error naming the identity, and no corpus is built. | Test |
| FR-085-AC-10 | A corpus of one member with no identity produces output byte-identical to a corpus built before this FR, including the lexical `ix://` id resolution of FR-026. | Test |
| FR-085-AC-11 | Resolution does no I/O beyond the supplied members; resolved and dangling classification is identical across runs and independent of member and document order ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test |
| FR-085-AC-12 | Over a two-member fixture, one test corpus holds all three cases (satisfied cross-member edge, dangling edge, wrong-kind edge) and each yields exactly its stated outcome and no other. | Test |

## Dependencies

- **Upstream**: [FR-024](./FR-024-parallel-repo-walk.md), [FR-025](./FR-025-spec-corpus-model.md), [FR-026](./FR-026-intra-spec-reference-resolution.md), [FR-058](./FR-058-upward-trace-completeness.md), [FR-057](./FR-057-corpus-check-severity.md)
- **Downstream**: [FR-084](./FR-084-two-way-relation-coverage-report.md)
