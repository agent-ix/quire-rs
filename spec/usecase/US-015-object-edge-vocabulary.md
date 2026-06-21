---
id: US-015
title: "Author Declares an Object's Typed Relationship Vocabulary"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "traces_to"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-040"
    type: "satisfied_by"
---

## Story

**As a** spec author or LLM agent writing a document that sets `object:`
(e.g. `aggregate_root`, `api_endpoint`, `state_machine`)
**I want** that object type's relationship vocabulary — the typed edges it may
declare *and* the kinds of target those edges may point at, including
cross-domain targets matched by capability rather than a hardcoded type name —
surfaced in the authoring skeleton and checked when the corpus loads
**So that** I declare correct, typed, self-describing edges without guessing
which verbs are legal or which targets they may reach.

## Context

Today typed edges are declared only on the **artifact** axis (FR/NFR/StR…); the
**object** axis contributes no edge vocabulary, so an `aggregate_root` cannot say
it `aggregates` member entities and an `api_endpoint` cannot say it `exposes` a
domain object. Worse, the verb is a free string that nothing validates
([FR-032](../functional/FR-032-validate-document.md) checks structure, not edge
type), and the canonical manifest schema already disagrees with the engine about
`allowed_links`' shape. This story closes the gap: the object's vocabulary is
declared in its module, **merged** with the host artifact's, presented to the
author by the skeleton ([FR-029](../functional/FR-029-archetype-input-contract.md)
recast), and validated by [FR-040](../functional/FR-040-object-edge-vocabulary.md).
Cross-domain edges resolve targets by **role tag** so a module never hardcodes
another module's type names.

## Acceptance

- **US-015-AC-1**: When authoring a document of type `T` with `object: O`, the
  skeleton shows a Relationships block listing the **union** of `T`'s and `O`'s
  allowed edge verbs, each with its category, description, and valid targets
  (exercises [FR-040](../functional/FR-040-object-edge-vocabulary.md)).
- **US-015-AC-2**: Declaring a `relationships[].type` that is in neither the
  artifact's nor the object's resolved vocabulary surfaces a `DisallowedEdgeType`
  finding naming the document and the offending verb (exercises
  [FR-040](../functional/FR-040-object-edge-vocabulary.md)).
- **US-015-AC-3**: Declaring an edge to a target whose object type — or any role
  it carries — does not satisfy that verb's target list surfaces a
  `DisallowedEdgeTarget` finding; an edge whose target carries the required
  **role** passes even when source and target live in different modules
  (exercises [FR-040](../functional/FR-040-object-edge-vocabulary.md)).
- **US-015-AC-4**: A verb absent from the merged `edge_types` registry, or a role
  absent from the merged `roles` registry, is rejected at module load — the
  author cannot reference an undefined edge or role (exercises
  [FR-040](../functional/FR-040-object-edge-vocabulary.md)).

## Efficiency Analysis

**Round trips:** 1 — the author reads the composed skeleton once, writes the
edges, and the same `body_extraction`/edge-harvest pass that already runs reports
any vocabulary violation. No extra fetch per edge.

**LLM context cost:** the per-(artifact, object) Relationships block is a small,
fixed addition to the skeleton the author already reads; target typing is
expressed against ~8 shared roles rather than enumerating cross-module types, so
the vocabulary stays compact as the object catalog grows.
