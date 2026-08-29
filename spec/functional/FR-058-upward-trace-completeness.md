---
id: FR-058
title: "Upward-Trace Completeness"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-057"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
---
# FR-058: Upward-Trace Completeness

## Description

[FR-049](./FR-049-verification-reference-integrity.md) and
[FR-050](./FR-050-declarative-coverage-computation.md) build **downward** coverage — FR → AC → TC
→ code — which answers *is what we wrote verified*. Nothing answers *is anything missing*, and
nothing operating over the existing spec text can: a requirement nobody wrote leaves no trace to
follow.

**Upward tracing is the only analysis class that finds a missing requirement.** A functional
requirement with no upstream need is a feature nobody asked for. A stakeholder requirement with no
downstream implementation is a need nobody built. Both are invisible to every check that starts at
a document and walks down, and both are ordinary findings once the graph is read in the other
direction.

This FR adds a corpus check pack over the edge set [FR-026](./FR-026-intra-spec-reference-resolution.md)
already resolves. It is **not** a new model: no new walk, no new index, no second graph.

### The declaration

Nothing about the chain is engine knowledge. `quire-rs` SHALL read the contract from the active
module's declared `traceability.required_relations`, each entry naming:

| Field | Meaning |
|---|---|
| `name` | The declaration's own name, reported in the finding |
| `from` | Archetype whose documents carry the obligation |
| `edges` | Accepted verbs — **any one** satisfies the relation |
| `to` | Accepted archetypes at the other end; empty means any document |
| `direction` | `outgoing` (the `from` document is the source) or `incoming` (it is the target) |
| `check` | The `<check>` half of the `trace:<check>` severity key ([FR-057](./FR-057-corpus-check-severity.md)) |
| `exclude` | Scope-relative globs exempt from this relation |

**Direction is why this is one declaration and not two checks.** "Every FR traces to a need" and
"every need is implemented" are the same edge read from opposite ends, so they are one type with a
direction rather than two engine checks that can drift apart.

The engine SHALL know no archetype name, no verb, and no chain. A module that declares
`from: hazard, edges: [mitigated_by], to: [FR], direction: outgoing` gets hazard-coverage checking
from the same code — which is how `agent-ix/spec-objects-security#5` becomes manifest data instead
of a second engine FR.

`traceability.acyclic_edges` names verbs that MUST NOT form a cycle. A requirement that
transitively refines itself states nothing, and no per-document check can see it because the defect
is a property of the graph. Each cycle SHALL be reported **once**, keyed on its smallest member, so
a three-node cycle is one finding rather than three.

### Severity

Findings ship **advisory**, tunable per declaration through the FR-057 registry — the `check` token
is chosen by the module precisely so `trace:orphan-fr` and `trace:unimplemented-str` are separate
keys. Promotion to `error` requires a corpus sweep and is user-gated, as every check before it.

### What a dangling edge cannot do

An edge whose target is absent from the bundle SHALL NOT satisfy a relation whose `to` is
constrained. The target is not loaded, so nothing can say what archetype it is, and accepting it
would let a typo satisfy the very requirement it broke. With `to: []` the verb alone is the
contract and a cross-repo edge counts — which is a reading a module opts into deliberately.

### Why a bad declaration must fail at load

Every other declaration in this model fails at load when it cannot be executed, and this one has a
sharper reason to: **its failure modes are quiet and plausible rather than obviously broken.**

A relation with `edges: []` accepts no verb, so nothing can satisfy it and *every* `from` document
is reported. On a real repository that is hundreds of findings against documents that are correctly
linked — a report that reads as a corpus-wide defect and sends someone to fix the specs. The mirror
image is just as bad: a `check` token that cannot form a `trace:<check>` severity key leaves the
relation running with a severity no `--severity` flag and no module override can ever name, so it
cannot be tuned or switched off. A blank verb in `acyclic_edges` walks a graph no edge matches, and
the cycle check silently covers nothing while reading as declared.

None of these is visible in the output. A check reporting everything and a check reporting nothing
both look like "no bug here" from the outside, and the only place they can be reported *against the
declaration that caused them* rather than against innocent documents is module load.

The `check` token is validated with `grammar::is_severity_key` — the same predicate the `--severity`
CLI parser uses — so the manifest and the command line accept exactly one vocabulary rather than
drifting into two.

### A declaration that names a kind nothing has

`edges: []` fails loudly. The opposite typo fails **silently**, and it is the worse of the two.

`from` selects documents by kind. A `from` naming a kind nothing has selects **zero** documents, so
the relation checks nothing at all. Measured on the fixture: changing `from: FR` to `from: FRR`
leaves a genuine orphan requirement unreported and the run comes back clean. One character disables
the check, and the result is indistinguishable from a bundle with nothing wrong.

So a relation naming a kind that **no loaded module declares and no document in the bundle is**
SHALL report itself. Either condition alone makes the kind live: a declared kind with no documents
yet is a contract waiting for content, and an undeclared kind that documents do use is a different
defect other checks already report.

This cannot be a load-time rule. `TraceabilityModel::validate` runs per module at manifest-parse
time, before the merge, and a relation legitimately names kinds another module contributes —
`from: hazard` in `spec-objects-safety` pointing `to: [FR]` from `spec-artifacts-iso`. The merged
registry and the walked bundle are both in hand only at validation.

**Verbs are deliberately excluded from this rule.** A first attempt checked `edges` against the
module's `edge_types` and immediately fired on the fixture's own working declaration, because a
single-module fixture declares no vocabulary of its own while still using `satisfies` correctly —
and [FR-041](./FR-041-authorable-inverse-edges.md)-AC-2 already permits verbs absent from
`edge_types`. That is a bad rule, not a bad fixture. A misspelt verb also fails loudly, which is the
case that needs no help.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-058-AC-1 | A document of the declared `from` archetype with no accepted edge is reported, naming the document and the declaration that asked for it; one that has the edge is not reported. | Test (TC-898) |
| FR-058-AC-2 | Any one of the declared `edges` satisfies the relation; a verb the declaration does not list does not. | Test (TC-899) |
| FR-058-AC-3 | `direction: incoming` reports a document nothing points at over the accepted verbs — the same declaration read the other way. | Test (TC-900) |
| FR-058-AC-4 | A **dangling** edge does not satisfy a relation whose `to` is non-empty, so a typo'd target cannot satisfy the requirement it broke. | Test (TC-901) |
| FR-058-AC-5 | A cycle over a declared `acyclic_edges` verb is reported exactly once, naming the path; the finding is keyed on the cycle's smallest member so rotations collapse. | Test (TC-902) |
| FR-058-AC-6 | Each declared relation carries its own `trace:<check>` severity key, so mapping one `off` leaves its siblings reporting and mapping one `error` promotes only it. | Test (TC-903) |
| FR-058-AC-7 | Findings are advisory by default: a bundle whose only faults are upward-trace findings stays valid under both postures unless a module says otherwise. | Test (TC-903) |
| FR-058-AC-8 | A module declaring neither `required_relations` nor `acyclic_edges` produces byte-identical output to one that never heard of this FR. | Test (TC-904) |
| FR-058-AC-9 | Findings are ordered by document then id, and identical across runs over one bundle ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test (TC-902) |
| FR-058-AC-10 | A declaration that cannot be executed SHALL fail at module load, naming the offending entry: no accepted `edges`, an empty `from`/`check`/`edges`/`to` entry, a duplicate relation name, a `check` token that cannot form a `trace:<check>` severity key, an uncompilable `exclude` glob, or a blank verb in `acyclic_edges`. | Test (TC-906, TC-907) |
| FR-058-AC-11 | A relation naming a document kind that no loaded module declares **and** no document in the bundle is SHALL report itself under `trace:undeclared-relation-vocabulary`, naming the kind and the declaration. Verbs are deliberately **not** checked this way — an absent `edge_types` entry is legal (FR-041-AC-2) and a misspelt verb already fails loudly. | Test (TC-908) |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-058-CON-1 | Every field of `TraceabilityModel` SHALL survive both `is_empty` and the cross-module merge. Neither is an exhaustive `match`, so a field added to the struct and to nothing else compiles and is silently dropped — which is exactly what happened while building this FR, twice in one change. | Design | Test (TC-905) |
| FR-058-CON-2 | The engine SHALL contain no archetype name, verb, or direction belonging to any particular traceability chain. A second chain is a manifest entry, never a second check. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-026](./FR-026-intra-spec-reference-resolution.md) (the edge set this reads), [FR-057](./FR-057-corpus-check-severity.md) (the severity knob it ships behind), [FR-050](./FR-050-declarative-coverage-computation.md) (the declaration model it extends)
- **Downstream**: `spec-objects-security#5` bidirectional hazard/threat coverage, which becomes declarations against this FR rather than engine code
