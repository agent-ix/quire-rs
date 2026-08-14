---
id: FR-022
title: "Write-back Primitives"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-006"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-020"
    type: "requires"
    cardinality: "1:1"
---
# FR-022: Write-back Primitives

## Description

`quire-rs` SHALL edit a markdown document by **splicing bytes**, and SHALL
return the full updated markdown. It SHALL NOT rebuild the file from the parsed
tree: a round-trip through the parser normalizes whatever the parser does not
model — trailing whitespace, blank-line runs, the author's list markers — and
the diff of such an edit is unreadable however small the change was.

Two primitives are exposed, differing only in what they replace:

- **Section content.** Everything after the heading line, up to the next heading
  or end of body. The heading line itself is untouched.
- **A whole block.** The heading line and its content together, addressed by
  stable block id ([FR-019](./FR-019-stable-block-ids.md)).

Everything the edit did not target — frontmatter, sibling sections, nested
blocks — SHALL be byte-identical in the result. Addressing a target that does
not exist SHALL be an error, and SHALL leave the document unchanged.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-022-CON-1 | Both primitives SHALL operate on byte ranges in the raw document, never on the parsed tree. The on-disk markdown stays canonical. | Architecture | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-022-AC-1 | Replacing a section's content updates that content range and leaves the heading line, the frontmatter and every other section byte-identical. | Test (TC-430) |
| FR-022-AC-2 | Replacing a block by its id updates the heading line and content together, and resolves a block nested below the top level. | Test (TC-431) |
| FR-022-AC-3 | After a block replacement, untouched blocks are byte-identical, including trailing whitespace and nested list markers. | Test (TC-432) |
| FR-022-AC-4 | Frontmatter is byte-identical through both primitives. | Test (TC-433) |
| FR-022-AC-5 | An unknown heading and an unknown block id each return `MissingField` and leave the document unchanged. | Test (TC-434, TC-435) |

> **CR-042 note (2026-08-14):** Authored after the fact. This shipped in v0.2 and
> was never written up. The tests were there the whole time — the eight cases in
> `src/writeback.rs` — so the criteria are read off working code rather than
> proposed (agent-ix/quire-rs#60).

## Dependencies

- **Upstream**: [FR-008](./FR-008-byte-exact-slicing.md) (the byte-exact content ranges being spliced), [FR-020](./FR-020-block-addressing.md) (resolving a block id)
- **Downstream**: consumers editing artifacts in place (`quire edit`)
