---
id: FR-020
title: "Block Addressing"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-019"
    type: "requires"
    cardinality: "1:1"
---
# FR-020: Block Addressing

## Description

A **block** in this engine is a heading-bounded section that carries a stable
block id ([FR-019](./FR-019-stable-block-ids.md)). It SHALL NOT be a separate
data type: `QuireSection` already carries the heading, the level, the byte-exact
content and the nested children, and a parallel `Block` struct would be the same
information under a second name, kept in sync by hand.

Lookup by block id SHALL walk the **nested** section tree, not only the top
level, because a block's depth is an authoring choice and an addressing scheme
that stopped at depth one would silently miss the block a consumer asked for.

An archetype MAY be referred to as a block type. That is a naming convenience
for consumers that think in blocks, and SHALL resolve to exactly the same
compiled archetype as the type name — never to a second registry.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-020-CON-1 | Block-type lookup SHALL be an alias over the archetype registry, never a separate store. Two registries would drift, and the second one would be the stale one. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-020-AC-1 | Lookup by block id resolves a block nested below the top level of the section tree, and resolves nothing for an id no heading declares. | Test (TC-410) |
| FR-020-AC-2 | `Registry::block_type(name)` returns the same compiled archetype as `Registry::archetype(name)` for every registered name. | Test (TC-411) |

> **CR-042 note (2026-08-14):** Authored after the fact, and narrower than the
> row it replaces. `spec/tests.md` recorded FR-020 as *"⚠️ Partial (no dedicated
> `Block` struct; v0.2 stores block_id on QuireSection + treats archetype as
> block_type)"* — describing the shipped design as a shortfall against a
> document that never existed. Having now written the requirement, the absence
> of a `Block` struct is the design, stated as CON-1, rather than a gap
> (agent-ix/quire-rs#60).

## Dependencies

- **Upstream**: [FR-019](./FR-019-stable-block-ids.md) (the id being addressed), [FR-013](./FR-013-archetype-loader.md) (the archetype registry the alias reads)
- **Downstream**: [FR-022](./FR-022-writeback-primitives.md) (resolves a block before splicing it)
