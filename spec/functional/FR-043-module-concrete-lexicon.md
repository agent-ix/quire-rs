---
id: FR-043
title: "Module-Supplied Concrete Lexicon for the Grammar"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-042"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-040"
    type: "requires"
    cardinality: "1:1"
---

# FR-043: Module-Supplied Concrete Lexicon for the Grammar

## Description

A Filament module MAY declare a `lexicon` registry in its `manifest.yaml` — a map of accepted
**concrete terms** to their definitions. The loader SHALL merge the per-module `lexicon` registries
across all loaded modules first-wins, mirroring the `edge_types`/`roles` registries ([FR-040](./FR-040-object-edge-vocabulary.md)).
A conflicting redeclaration of a term SHALL emit a non-fatal `DuplicateLexiconTerm` diagnostic.
The `Registry` SHALL expose the merged map through a `lexicon()` accessor. The `Registry` SHALL
also expose a precompiled matcher, built once at load, that recognises a lexicon term in a span of
text.

The EARS object-aware `vague-response` check ([FR-042](./FR-042-requirement-grammar-check.md)) SHALL
consume the merged lexicon as its set of concrete object nouns. The engine SHALL NOT carry a
hardcoded concrete-noun list — concrete vocabulary lives in the domain modules that define it, not
in the engine. A statement whose response object matches a lexicon term SHALL be treated as
concrete and SHALL NOT be flagged. The engine SHALL retain the bounded vague-quality lexicon, the
mechanism/numeric qualifiers, and the backticked-identifier suppression
([FR-042](./FR-042-requirement-grammar-check.md)) — these are generic, not domain vocabulary.

The merged lexicon reaches the grammar through the registry-backed validation path. When
`validate_document_in_registry` runs the grammar, the engine SHALL pass the `Registry`'s merged
lexicon to the check. When the type-only `validate_document` path runs the grammar without a
registry, the engine SHALL apply an empty lexicon: the mechanism, numeric-bound, and backticked
suppressions still apply, but bare domain-noun suppression does not. This degradation SHALL be
documented as the type-only path's known limitation.

The `check_grammar` Python binding SHALL accept an optional `module_root`. When `module_root` is
given, the binding SHALL load that module registry and apply its merged lexicon; when it is absent,
the binding SHALL apply an empty lexicon.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-043-CON-1 | A malformed `lexicon` entry SHALL fail module load like any other manifest shape error | Operational | Unit Test |
| FR-043-CON-2 | An empty or absent `lexicon` registry SHALL load without error | Operational | Unit Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-043-AC-1 | A manifest declaring a `lexicon` entry loads, and `Registry::lexicon()` returns the merged map containing that term. | Test |
| FR-043-AC-2 | Two modules declaring the same term with different definitions are first-wins and emit one `DuplicateLexiconTerm` diagnostic; identical redeclaration emits none. | Test |
| FR-043-AC-3 | With a registry whose lexicon contains `pagination`, the statement `The system shall support pagination.` yields no `vague-response` finding; with the term removed, the same statement yields one. | Test |
| FR-043-AC-4 | The engine contains no hardcoded concrete-noun list: a bare domain noun absent from the lexicon (and not backticked, with no mechanism/bound) yields a `vague-response` finding under an empty lexicon. | Test |
| FR-043-AC-5 | Backticked-identifier, mechanism, and numeric-bound suppression (FR-042) still hold under an empty lexicon — they do not depend on the lexicon. | Test |
| FR-043-AC-6 | `validate_document_in_registry` applies the registry's merged lexicon; the type-only `validate_document` applies an empty lexicon (more findings) and never errors on the difference. | Test |
| FR-043-AC-7 | The `check_grammar` Python binding with a `module_root` applies that registry's lexicon; without one it applies an empty lexicon. | Test |

## Dependencies

- **Upstream**: [FR-042](./FR-042-requirement-grammar-check.md) (the grammar consumer), [FR-040](./FR-040-object-edge-vocabulary.md) (the merge-vocab registry pattern this mirrors)
- **Downstream**: the `spec-objects-*` modules ship the domain `lexicon` registries; a project Ubiquitous-Language artifact adds a project-scoped layer (future)
