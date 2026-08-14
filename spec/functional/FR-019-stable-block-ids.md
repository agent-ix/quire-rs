---
id: FR-019
title: "Stable Block Identifiers"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-020"
    type: "required_by"
    cardinality: "1:1"
---
# FR-019: Stable Block Identifiers

## Description

A section's parser-derived `id` is `<slug>-L<line>`, which moves whenever a line
above it moves. That is fine for reporting and useless for addressing: an editor
that resolved a block by `id`, then wrote to the document, would resolve
somewhere else on the next pass.

`quire-rs` SHALL therefore read a **stable block identifier** off the heading
itself, from the Pandoc heading-attribute form `## Heading {#blk-id}`. The
attribute is authored, so it survives every edit that does not touch it — which
is what makes it addressable.

The attribute SHALL be parsed **off** the heading text rather than left in it: a
consumer reading `heading` gets `Behavior`, not `Behavior {#blk-7af2}`. A
heading carrying no attribute SHALL report no block id, rather than a derived or
invented one.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-019-CON-1 | The engine SHALL NOT synthesize a block id for a heading that has none. An id nobody authored is not stable, and a consumer cannot tell the two apart. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-019-AC-1 | A heading `## Behavior {#blk-7af2}` parses to `block_id = Some("blk-7af2")` with heading text `Behavior`, and a heading with no attribute parses to `block_id = None` with its text byte-identical to the input. | Test (TC-400, TC-403) |
| FR-019-AC-2 | The attribute is stripped from the heading text on parse: no `{#…}` span remains in `QuireSection.heading`. | Test (TC-402) |
| FR-019-AC-3 | A block id survives a write-back addressed by that id and a reparse of the result. | Test (TC-443) |

> **CR-042 note (2026-08-14):** Authored after the fact. This requirement
> shipped in v0.2 and was never written up — `spec/tests.md` carried TC-400..403
> against an `FR-019` that had no document, which is how the rows could claim
> `apply_block_patch`, an API the render removal deleted. AC-3 is stated against
> `update_block`, the write-back that actually exists (agent-ix/quire-rs#60).

## Dependencies

- **Upstream**: [FR-007](./FR-007-fenced-block-heading-walk.md) (the heading walk the attribute is read during)
- **Downstream**: [FR-020](./FR-020-block-addressing.md) (addressing by this id), [FR-022](./FR-022-writeback-primitives.md) (the write-back that resolves it)
