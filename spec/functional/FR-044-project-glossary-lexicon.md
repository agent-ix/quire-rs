---
id: FR-044
title: "Project Ubiquitous-Language Lexicon for the Grammar"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-043"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-027"
    type: "requires"
    cardinality: "1:1"
---

# FR-044: Project Ubiquitous-Language Lexicon for the Grammar

## Description

A repository MAY author its own **concrete vocabulary** — a `Glossary` artifact carrying a
`## Terms` table (`Term | Definition`), or a `## Ubiquitous Language` section on a domain object. The
engine SHALL harvest those project terms and feed them, alongside the merged module lexicon
([FR-043](./FR-043-module-concrete-lexicon.md)), into the EARS object-aware vague-response check
([FR-042](./FR-042-requirement-grammar-check.md)), so a repo's domain terms are accepted as concrete
objects in its own grammar check.

The engine SHALL provide a **harvester** that reads project terms from a loaded `Spec` corpus
([FR-027](./FR-027-whole-spec-query-api.md)). The harvester SHALL collect the `Term` column of every
`Glossary` artifact's `## Terms` table. The harvester SHALL also collect the bold term of every
`## Ubiquitous Language` bullet. A repository that authors no glossary SHALL harvest an empty term
set.

The merged module lexicon lives on the immutable `Registry`; the project terms vary per repository,
so the engine SHALL NOT store them on the `Registry`. Instead, the orchestrator SHALL compose an
ad-hoc `GrammarLexicon` from the registry's lexicon keys and the harvested project terms, and SHALL
pass that combined lexicon to the grammar.

The engine SHALL expose `validate_document_in_registry_with_lexicon`, which validates a document
against an explicitly supplied `GrammarLexicon` rather than the registry's own. The existing
`validate_document_in_registry` SHALL delegate to it with `Registry::lexicon_matcher()`, so the two
entry points share one validation body.

When a `Spec` corpus is validated as a bundle, `validate_bundle` SHALL harvest the corpus's project
terms once and SHALL validate each document with the combined lexicon.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-044-CON-1 | The project-glossary layer SHALL remain advisory — a harvested term only suppresses `vague-response` findings and never blocks validation | Operational | Unit Test |
| FR-044-CON-2 | A malformed or absent glossary SHALL harvest zero terms without failing validation | Operational | Unit Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-044-AC-1 | The harvester collects the `Term` column of a `Glossary` artifact's `## Terms` table into the project term set. | Test |
| FR-044-AC-2 | The harvester collects the bold term of each `## Ubiquitous Language` bullet (`- **Term** — …`) into the project term set. | Test |
| FR-044-AC-3 | A combined `GrammarLexicon` contains both the registry lexicon keys and the harvested project terms; a project-only term is recognised as concrete. | Test |
| FR-044-AC-4 | `validate_document_in_registry_with_lexicon` injects the supplied lexicon: a document whose response object is a project term yields no `vague-response` finding, while the same document under the module-only lexicon yields one. | Test |
| FR-044-AC-5 | `validate_bundle` harvests the loaded `Spec`'s project terms and applies the combined lexicon to every document in the bundle. | Test |
| FR-044-AC-6 | The project-glossary suppression is advisory: a document with project-glossary-suppressed and remaining findings still reports `is_valid` per its structural errors alone. | Test |
| FR-044-AC-7 | A repository that authors no glossary harvests an empty term set, and its validation is identical to the module-only lexicon path. | Test |

## Dependencies

- **Upstream**: [FR-043](./FR-043-module-concrete-lexicon.md) (the merged module lexicon this composes with), [FR-027](./FR-027-whole-spec-query-api.md) (the whole-spec corpus the harvester reads)
- **Downstream**: `quire-cli` `validate` harvests the `--scope` repo's glossary terms and injects the combined lexicon; the `Glossary` artifact ships in `spec-artifacts-iso`
