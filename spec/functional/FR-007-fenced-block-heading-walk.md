---
id: FR-007
title: "Fenced-Code-Block-Aware Heading Walk"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-003"
    type: "implements"
    cardinality: "1:1"
---

## Description

When walking the document body to collect heading positions, the parser SHALL track fenced code block state with the fence model below, which is shared with the code-block scanner of [FR-010](./FR-010-query-api.md) CR-008 and [FR-011](./FR-011-body-extraction-dsl.md) CR-004:

1. An opening fence is a run of three or more backticks, or of three or more tildes, indented at most three columns past the content column of the enclosing list item (column 0 outside a list). A backtick opening line whose text after the run contains a backtick is not a fence. A tab advances to the next multiple of four columns.
2. While a fence is open, lines beginning with `#` are NOT recognized as headings.
3. A fence closes only on a line with the same fence character, a run at least as long as the opening run, indented at most three columns past the same list-item content column, and nothing but whitespace after the run. Any other line, including a shorter run, a run of the other character, and a run followed by an info string, is content of the open block (the mismatched-character case is FR-007-AC-4).
4. A fence left open runs to the end of the document when it is at the top level. A fence left open inside a list item ends, without a closing fence, at the first non-blank line indented less than that item's content column; the lines before it are the block's content (an empty block is dropped, as for an unclosed top-level block), and the ending line is read as an ordinary line, so it may be a heading.
5. The content column of a list item is the column after its marker (a bullet, or up to nine digits followed by `.` or `)`) and the one to four spaces that follow it. A non-blank line indented less than the content column of the innermost item leaves that item, except that a paragraph-continuation line right after a paragraph line (a lazy continuation: not blank, not a list marker, fence, ATX heading, block quote or thematic break) keeps the item open. A marker line that would start a new list right after a paragraph line is a list item only if it has content and an ordered marker is `1`; a line indented from the marker of the item just left is a sibling item and is always an item. Items nest, and a fence nested in one is measured from the innermost enclosing item that still holds the line.
6. Out of scope: block quotes (a fence behind `>` is not recognised), indented code blocks, a fence that opens on the same line as a list marker, and setext headings.

> **CR-009 (CommonMark fence model, PLAT-1682):** the original rules were the TS/Py
> reference algorithm: any line whose `trim_start()` begins with three backticks or
> three tildes toggled the fence, whatever its length or indentation. That let a
> shorter inner fence, a closing line with an info string, or a line indented four
> spaces open or close a block. This is an intentional divergence from the TS/Py
> reference, like the `~~~` extension; list items follow the CommonMark container rules in
> rule 5 only as far as fence indentation needs, not the full list grammar; the production `quire` surfaces consume this crate, so the fix
> covers both.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | For input `## Real\n\`\`\`\n# fake\n\`\`\`\n## Real2`, the parser returns 2 sections (`Real`, `Real2`) — the `# fake` line is inside the fence and not a heading. | Test |
| FR-007-AC-2 | For input `## Real\n\`\`\`\n## still-inside\n` (no closing fence), the parser returns 1 section (`Real`) — the trailing `## still-inside` is inside the unclosed block. | Test |
| FR-007-AC-3 | For input `~~~\n# fake\n~~~\n## Real`, the parser returns 1 section (`Real`); the `# fake` is inside the tilde fence. | Test |
| FR-007-AC-4 | A test transliterated from `~/dev/quire/tests/` covering each variant passes. | Test |
| FR-007-AC-5 | For input `## A\n````\n```\n# fake\n````\n## B` the shorter inner fence does not close the block, so `# fake` is not a heading and the parser returns `A` and `B`; a four-space-indented ` ``` ` line opens no fence. | Test |
| FR-007-AC-6 | In the heading walk, a closing-position line carrying an info string (` ```text ` inside a ` ``` ` block) is content, so a heading after it stays inside the block until a bare closing fence; a shorter tilde run does not close a longer tilde fence. | Test |
| FR-007-AC-7 | A fence nested in an ordered list item (`1.  ` marker) or in a nested bullet is recognised at its content column, so a heading-like line inside it is not a heading; once a heading line ends the list (FR-007-AC-9), indentation is measured from column 0 again. | Test |
| FR-007-AC-8 | A fence left open inside a list item ends at the first non-blank line indented less than the item's content column: for `- item\n\n  \`\`\`\n  x\n## H` the walk returns the heading `H`, and `extract_diagrams` returns the block holding `x` with no section; an unclosed top-level fence still runs to the end of the document. | Test |
| FR-007-AC-9 | A lazy continuation line keeps its list item open (a fence four spaces in after `- item\ntext\n\n` opens, one after a heading line does not), and a `2.` marker after a paragraph line opens no item while `1.` does and a sibling `2.` of an open list does. | Test |

## Dependencies

- **Upstream**: [US-002](../usecase/US-002-developer-parses-spec-doc.md), [StR-003](../stakeholder/StR-003-parse-parity-with-quire.md)
- **Downstream**: [FR-005](./FR-005-parse-document-api.md) (`parse_document` heading walk relies on this fence tracking)
