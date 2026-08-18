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

> **CR note (CR-069, the empty-section and end-of-file splices, 2026-08-18):**
> this FR gains **AC-6 and AC-7**. Both were found by the metamorphic property
> suite (agent-ix/quire-rs#84) the first time `update_section` was stated as a
> relation instead of as five examples. AC-1..AC-5 all held; what they never
> asked was whether the primitive is the **identity** when the replacement is
> the section's own content, and neither reaches a section with an empty byte
> range. An empty section gained a blank line on every rewrite, and a heading on
> the document's last line had the new content concatenated onto the heading
> text itself. The second is data loss, not formatting.

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
| FR-022-AC-6 | Replacing a section's content with **its own current content** returns the document byte-identical, for every section including an **empty** one — a heading immediately followed by the next heading, or by end of file. A speculative rewrite is therefore a no-op rather than a source of churn. | Test (TC-896) |
| FR-022-AC-7 | Writing content into a section whose heading line is the **last line of the document** (no trailing newline) inserts the line break that separates them, so the heading text is unchanged and the new content parses as content. Concatenating the two — turning `## Aa` plus `body` into a heading reading `Aa body` — is document corruption and MUST NOT occur. | Test (TC-896) |

> **CR-042 note (2026-08-14):** Authored after the fact. This shipped in v0.2 and
> was never written up. The tests were there the whole time — the eight cases in
> `src/writeback.rs` — so the criteria are read off working code rather than
> proposed (agent-ix/quire-rs#60).

## Dependencies

- **Upstream**: [FR-008](./FR-008-byte-exact-slicing.md) (the byte-exact content ranges being spliced), [FR-020](./FR-020-block-addressing.md) (resolving a block id)
- **Downstream**: consumers editing artifacts in place (`quire edit`)
