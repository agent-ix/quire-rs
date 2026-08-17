---
id: FR-054
title: "Verification-Method Catalog"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-053"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-014"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
---
# FR-054: Verification-Method Catalog

## Description

`quire-rs` SHALL merge a module-declared **verification-method catalog** — a
registry of the methods by which a requirement can be verified — across every
active module and SHALL expose the merged registry through the `Registry`.

The toolchain's purpose is to help define specs *and testing plans* for the
software being specced, then track that the testing happens. Today "how should
this requirement be verified" has no machine-readable answer. The vocabulary is
scattered and prose-bound: the ISO 29148 `Inspection | Analysis | Demonstration
| Test` set lives only inside two `spec-artifacts-iso` lint rules; test
techniques live in `traceability.vocabularies.test_type`; and quoin's
`spec-evidence-analysis` skill assigns a method per requirement from a
**skill-local prose table** that no manifest declares and no code reads. Nothing
can advise a method, and nothing can check that a chosen one was honoured.

### The catalog is a registry, like every other vocabulary

`verification_catalog:` is a top-level manifest map of method id → definition,
merged **first-wins** across modules exactly as `edge_types`, `lexicon`,
`observable_verbs` and `property_idioms` merge, for the same determinism reason
([NFR-006](../non-functional/NFR-006-determinism.md)): the merged answer must
not depend on which module happened to load first.

```yaml
verification_catalog:
  property-based-testing:
    name: Property-based testing
    class: Test
    definition: >-
      Execute a property over generated inputs, asserting an invariant rather
      than a fixed expected value.
    evidence_kind: test-run
    applicability:
      property_shapes: [round-trip, idempotence, invariant]
      characteristics: [universally-quantified]
    tooling: [proptest, fast-check, hypothesis]
```

### `class` is a free string, deliberately

`class` carries the IADT axis in this ecosystem, and the engine still SHALL NOT
close it to those four values. The generality constraint is not decorative: an
external user classifying methods by ISO 29119-4 technique family, by assurance
level, or by anything else must be able to, and an engine-side enum would make
the catalog agent-ix's rather than theirs. The engine knows the *shape* of an
entry and never the meaning of any value in it.

### Applicability rules are opaque to the engine

`applicability:` is a map of rule name to a list of values. The engine SHALL
store and surface it and SHALL NOT interpret it. Which requirement
characteristics, object types or property shapes trigger advising a method is
the **advisor's** judgement (agent-ix/quoin#89) over data the module authored;
an engine that understood `property_shapes` would then need to understand
`object_types`, `characteristics`, and every axis a future module invents. The
engine that stays out of it stays general.

### The catalog is addressable as a vocabulary

`Registry::column_vocabulary` exists (CR-015) and matches exactly one hardcoded
name, `test_type`. It becomes a real named lookup, and the catalog contributes
two **derived** names computed from the merged entries rather than authored a
second time:

| Name | Value |
|---|---|
| `test_type` | the declared `traceability.vocabularies.test_type` (unchanged) |
| `verification_method` | every merged catalog key, sorted |
| `verification_class` | every distinct `class` in the merged catalog, sorted |

This is what makes the catalog the single source rather than a fourth copy of
the same vocabulary. A consumer asking "what may a `Verification` cell say"
reads `verification_method`, which cannot drift from the catalog because it *is*
the catalog.

> **`from_vocabulary` on `LocatorAssert` does not ship here — deliberately.**
> agent-ix/quire-rs#133 left the decision to Specify. The duplication it targets
> is real: a `TestMatrix` contract's `column_choices.Type` restates
> `traceability.vocabularies.test_type`, kept honest only by a
> `spec-artifacts-process` test. But resolving a vocabulary *reference* inside
> an assert has to happen after the cross-module merge, which means either
> threading a `Registry` through `evaluate_assert` — a public API signature
> change on the per-document validation path — or rewriting compiled archetypes
> at registry construction. Each is a real change with its own acceptance
> surface, and bundling either into the catalog release would make one version
> carry two unrelated risks, against a duplication that is currently **held
> honest by a passing test** rather than silently rotting. The half that the
> catalog genuinely needs — the *lookup* — ships here, so `process#35` can
> reconcile toward one source immediately. The assert-side key is filed
> separately with this design recorded.

## Inputs

- Zero or more module manifests declaring `verification_catalog:`.

## Outputs

- `VerificationMethod { name, class, definition, evidence_kind, applicability,
  tooling }`, keyed by method id, merged first-wins.
- `Registry::verification_catalog()` — the merged map, or `None` when no active
  module declares one.
- `Registry::column_vocabulary(name)` — extended to the three names above.

## Behavior

The engine SHALL parse `verification_catalog:` as a map of id → entry. `name`,
`class` and `definition` are required; `evidence_kind`, `applicability` and
`tooling` are optional. The entry struct SHALL be `deny_unknown_fields`, so a
typo fails module load rather than being silently discarded — the house rule
that costs an engine release before a module may declare a new key, and the
reason this FR ships before any module declares the block.

An entry whose id is already merged SHALL be skipped, and its module recorded in
a `DuplicateVerificationMethod` diagnostic, matching how `DuplicateEdgeType`
reports the same situation.

An empty or whitespace-only `name`, `class` or `definition` SHALL fail module
load naming the offending method id: a catalog entry that cannot say what it is
advises nothing.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-054-CON-1 | The engine SHALL NOT name a verification method, a method class, an evidence kind or an applicability rule anywhere on this path. Every one is module data, so an external user's catalog drives their advice exactly as this ecosystem's drives ours. | Architecture | Inspection |
| FR-054-CON-2 | The engine SHALL NOT interpret `applicability:`. It stores and surfaces the rules; deciding which requirement they match is the advisor's judgement, and an engine that understood one axis would owe an understanding of every axis a future module invents. | Architecture | Inspection |
| FR-054-CON-3 | Catalog entries SHALL NOT participate in validation: no finding, no severity, no [FR-048](./FR-048-per-check-grammar-severity.md) key. A declared method a document does not use is data, and an undeclared method a document does use is the auditor's finding, not the engine's. | Architecture | Test |
| FR-054-CON-4 | The derived vocabularies SHALL be computed from the merged catalog, never authored alongside it. A second authored copy is the duplication this FR exists to remove. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-054-AC-1 | A module declaring `verification_catalog:` exposes every entry through `Registry::verification_catalog()` with its name, class, definition, evidence kind, applicability rules and tooling intact. | Test (TC-844) |
| FR-054-AC-2 | Two modules declaring the same method id merge first-wins, the later declaration is skipped, and a `DuplicateVerificationMethod` diagnostic names both modules. | Test (TC-845) |
| FR-054-AC-3 | A module declaring no catalog yields `None` rather than an empty map, so a consumer reports the catalog as undeclared rather than as containing no methods. | Test (TC-846) |
| FR-054-AC-4 | An entry carrying an unknown key fails module load naming the key, so a typo cannot be silently discarded. | Test (TC-847) |
| FR-054-AC-5 | An entry whose `name`, `class` or `definition` is empty or whitespace-only fails module load naming the offending method id. | Test (TC-848) |
| FR-054-AC-6 | `column_vocabulary("verification_method")` returns exactly the merged catalog keys in sorted order, and `column_vocabulary("verification_class")` returns each distinct class once, sorted. | Test (TC-849) |
| FR-054-AC-7 | `column_vocabulary("test_type")` returns the declared traceability vocabulary unchanged, and an unknown name returns an empty slice rather than a default. | Test (TC-850) |
| FR-054-AC-8 | A catalog whose entries declare no `applicability` is exposed with an empty rule map, and one declaring rules the engine has never heard of is exposed with those rules verbatim (CON-2). | Test (TC-851) |
| FR-054-AC-9 | A corpus validated with a catalog declared produces the same findings, in the same order and with the same fields, as the same corpus with no catalog declared (CON-3). | Test (TC-852) |
| FR-054-AC-10 | The derived vocabularies change when the catalog changes and are never read from a separate declaration: a module declaring a catalog and no vocabulary block still answers `verification_method` (CON-4). | Test (TC-853) |

## Dependencies

- **Upstream**: [FR-014](./FR-014-module-activation.md) (manifest loading and the first-wins merge this follows), [FR-050](./FR-050-declarative-coverage-computation.md) (the `column_vocabulary` accessor this generalizes)
- **Downstream**: [FR-053](./FR-053-obligation-record.md) (an obligation's method is conformant against this catalog, checked by the auditor rather than here); `spec-artifacts-process` authors the 29119-4 catalog content (agent-ix/spec-artifacts-process#35); quoin's test-plan advisor reads the merged catalog (agent-ix/quoin#89) and its auditor checks method conformance against it (agent-ix/quoin#80)
