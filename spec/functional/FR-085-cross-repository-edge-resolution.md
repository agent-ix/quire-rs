---
id: FR-085
title: "Cross-repository edge resolution"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-027"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-038"
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
with an **identity** `<org>/<repo>`, supplied by the caller and never derived from content or the
filesystem. The rules are:

- A corpus of two or more members requires every member to declare an identity; a member without one
  is an error naming it, and no corpus is built.
- Two members declaring the same identity are an error naming it.
- A corpus of exactly one member MAY omit the identity. Such a corpus resolves exactly as
  [FR-025](./FR-025-spec-corpus-model.md) and [FR-026](./FR-026-intra-spec-reference-resolution.md)
  specify, with lexical `ix://` id resolution.

Document ids are unique within a member, not across members. A document is addressed across the
corpus by `(identity, id)` ([FR-027](./FR-027-whole-spec-query-api.md) `by_qualified_id`); a bare
`by_id` answers only when exactly one member holds the id.

### Resolution

An `ix://<org>/<repo>/...` target contributes the pair `(<org>/<repo>, <ID>)`, where `<ID>` is the
trailing artifact id as in FR-026. In a corpus whose members declare identities (every multi-member corpus, and a single member that declares one):

- if a member declares `<org>/<repo>`, the target is **Resolved** when that member holds `<ID>`, and
  **Dangling** when it does not;
- if no member declares `<org>/<repo>`, the target is **Dangling**, even when another member holds
  the same `<ID>`;
- a target naming the source document's own member resolves within that member;
- a bare `<ID>` and a relative-path target resolve within the source document's own member only.

A `Dangling` edge records the target exactly as authored, so a report can name it.

A resolved cross-member edge appears in the source's outgoing set and in the target's incoming set
like any other, and [FR-058](./FR-058-upward-trace-completeness.md) relations read the corpus-wide
edge set: a document in member A that `mitigates` a document in member B satisfies a relation on B
with `direction: incoming`, and satisfies a relation on A with `direction: outgoing` when the target
archetype is in `to`.

### Bundle validation over a multi-member corpus

[FR-038](./FR-038-okf-bundle-validation.md) validates one bundle with `validate_bundle(spec,
registry, posture, root)`. This FR extends it to a corpus of several members, the engine behaviour
behind `quire validate --okf --repo` (quire-cli FR-030):

- `validate_corpus(corpus, registry, posture, roots)` takes one root per member, in member order,
  and runs the FR-038 bundle validation of each member against the shared corpus, so edges resolve
  across members. The result is one report whose findings are the concatenation of each member's
  findings in member order. `posture` applies to every member.
- A member's identity is that member's declared bundle package (`Spec::with_bundle_package`): the
  `ix://` authority-versus-package comparison of FR-038-AC-9 uses the member's identity for its own
  documents. A single-member corpus with no identity validates exactly as FR-038 specifies.
- Identity comparison folds ASCII case on both sides (FR-038-AC-10), for member lookup in resolution
  as much as for the package comparison; the identity is stored as given and only the comparison
  folds. Two members whose identities differ only by case are the duplicate-identity error.
- Every finding, including FR-058 relation findings and `trace:target-kind-mismatch`, names the
  member identity of its **source document** in a `repo` field, omitted for a member with no identity.

### Kind of a resolved target

When a document has an edge whose verb is accepted by a relation but whose resolved target's
archetype is outside the relation's non-empty `to`, the edge SHALL NOT satisfy the relation, and the
engine SHALL report a **kind mismatch** under `trace:target-kind-mismatch`, advisory by default
([FR-057](./FR-057-corpus-check-severity.md)): the key defaults to `warning` like every `trace:` key
added after FR-057. It is distinct from `trace:reference-kind-mismatch` ([FR-049-AC-12](./FR-049-verification-reference-integrity.md)):
that key reports a *trace reference* whose target is of the wrong kind, and this one reports a
*relation edge* whose resolved target is outside the relation's `to`. The two never fire for the same
edge. The finding names the source document, the target
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
| FR-085-AC-8 | A bare id and a relative-path link resolve only within the source's member. | Test |
| FR-085-AC-9 | Two members declaring one identity is an error naming the identity, and no corpus is built. | Test |
| FR-085-AC-10 | Over a corpus of one member with no identity, the set of edges and each edge's resolved or dangling classification equal those FR-026 lexical resolution gives for the same documents, including an `ix://` target whose id exists locally. This guarantee covers resolution only: the `trace:target-kind-mismatch` finding of AC-6 applies to such a corpus too and is outside it. | Test |
| FR-085-AC-11 | Resolution does no I/O beyond the supplied members; resolved and dangling classification is identical across runs and independent of member and document order ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test |
| FR-085-AC-12 | Over a two-member fixture, one test corpus holds all three cases (satisfied cross-member edge, dangling edge, wrong-kind edge) and each yields exactly its stated outcome and no other. | Test |
| FR-085-AC-13 | A corpus of two members in which one declares no identity is an error naming that member, and no corpus is built; two identity-less members are the same error. | Test |
| FR-085-AC-14 | In a corpus of one member that declares identity `o/a`, an `ix://o/a/...` target resolves within that member and an `ix://o/b/...` target is `Dangling` even when the local member holds the id. | Test |
| FR-085-AC-15 | In a two-member corpus where both members hold id `X`, no `DuplicateArtifactId` diagnostic is raised, a bare `X` link resolves within its own member, `by_id("X")` returns `None`, and `by_qualified_id` returns each member's document ([FR-027-AC-10](./FR-027-whole-spec-query-api.md)); two documents with one id inside a single member still raise `DuplicateArtifactId` ([FR-025-AC-3](./FR-025-spec-corpus-model.md)). | Test |
| FR-085-AC-16 | `validate_corpus` over two members validates each member's documents against the shared corpus: a document in member A whose edge targets a document in member B raises no `dangling-reference`, and the same corpus without B raises it. | Test |
| FR-085-AC-17 | In `validate_corpus`, an `ix://o/b/...` reference to an id absent from loaded member `o/b` is a `dangling-reference` finding, while an `ix://o/c/...` reference whose authority names no loaded member is not reported as `dangling-reference` (FR-038-AC-9: the authority differs from the package, so the corpus cannot see it). | Test |
| FR-085-AC-18 | Identity matching folds ASCII case: `ix://O/B/X` resolves to the member declared `o/b`, and two members declared `o/b` and `O/B` are the duplicate-identity error of AC-9. | Test |
| FR-085-AC-19 | Every finding from `validate_corpus` carries the `repo` of its source document's member, for an FR-058 relation finding, a `trace:target-kind-mismatch` finding and a `dangling-reference`; a finding from a member with no identity carries no `repo`. | Test |
| FR-085-AC-20 | A corpus of one member with no identity run through `validate_corpus` yields findings identical to `validate_bundle` for the same root. | Test |

## Dependencies

- **Upstream**: [FR-024](./FR-024-parallel-repo-walk.md), [FR-025](./FR-025-spec-corpus-model.md) (amended for the member-keyed index), [FR-027](./FR-027-whole-spec-query-api.md) (`by_id`, `by_qualified_id`), [FR-038](./FR-038-okf-bundle-validation.md) (bundle validation, extended above), [FR-026](./FR-026-intra-spec-reference-resolution.md), [FR-058](./FR-058-upward-trace-completeness.md), [FR-057](./FR-057-corpus-check-severity.md)
- **Downstream**: [FR-084](./FR-084-two-way-relation-coverage-report.md)
