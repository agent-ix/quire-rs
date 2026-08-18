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

## Constraints

| ID | Constraint | Verification |
|----|-----------|--------------|
| FR-058-CON-1 | Every field of `TraceabilityModel` SHALL survive both `is_empty` and the cross-module merge. Neither is an exhaustive `match`, so a field added to the struct and to nothing else compiles and is silently dropped — which is exactly what happened while building this FR, twice in one change. | Test (TC-905) |
| FR-058-CON-2 | The engine SHALL contain no archetype name, verb, or direction belonging to any particular traceability chain. A second chain is a manifest entry, never a second check. | Inspection |

## Dependencies

- **Upstream**: [FR-026](./FR-026-intra-spec-reference-resolution.md) (the edge set this reads), [FR-057](./FR-057-corpus-check-severity.md) (the severity knob it ships behind), [FR-050](./FR-050-declarative-coverage-computation.md) (the declaration model it extends)
- **Downstream**: `spec-objects-security#5` bidirectional hazard/threat coverage, which becomes declarations against this FR rather than engine code
