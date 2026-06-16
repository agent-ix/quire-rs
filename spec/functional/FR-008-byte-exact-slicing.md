---
id: FR-008
title: "Byte-Exact Section Content Slicing"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-003"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

`QuireSection.content` SHALL be a byte-exact slice of the original input body between the heading line and the next heading (or end of body). Specifically:

1. The slice starts at the line **after** the heading line.
2. The slice ends at the line **before** the next sibling-or-ancestor heading, or at the end of body if no such heading exists.
3. Whitespace, blank lines, and trailing characters are preserved exactly — no trim, no normalization, no re-serialization.
4. Line endings present in the input (`\n` vs `\r\n`) are preserved in the slice.

Concretely: round-trip property: given `doc = parse_document(input)` and a section `s`, concatenating preamble + headings + `s.content` for each section in document order reproduces `input` exactly.

## Acceptance

- **FR-008-AC-1**: For input `## A\n  indented body  \n## B\n`, `section("A").content == "  indented body  "` (preserved leading/trailing spaces).
- **FR-008-AC-2**: For input containing both `\n` and `\r\n` endings interleaved, `section(...).content` preserves the original endings byte-for-byte.
- **FR-008-AC-3**: A proptest takes a random markdown document, parses it, and asserts that reconstructing the body from `(preamble, [(heading_line, section.content)])` byte-equals the original body.
