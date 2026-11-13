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

When walking the document body to collect heading positions, the parser SHALL track fenced code block state per the TS/Py reference algorithm:

1. A line whose `trim_start()` begins with three backticks (` ``` `) toggles `in_fence` state.
2. While `in_fence` is true, lines beginning with `#` are NOT recognized as headings.
3. Tilde fences (`~~~`) toggle `in_fence` in the same way.
4. Mismatched fence types (a backtick fence opened, a tilde line in between) do NOT close the backtick fence — only a matching fence character does.
5. If the document ends with `in_fence` still true (unclosed fence), trailing lines are treated as inside the block — they are not parsed as headings.

6. Fences follow the CommonMark rules (shared with the code-block scanner of [FR-010](./FR-010-query-api.md) CR-008): an opening fence is a run of three or more backticks or tildes indented at most three spaces, and a fence closes only on the same character, a run at least as long as the opener, indented at most three spaces, followed by whitespace only.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | For input `## Real\n\`\`\`\n# fake\n\`\`\`\n## Real2`, the parser returns 2 sections (`Real`, `Real2`) — the `# fake` line is inside the fence and not a heading. | Test |
| FR-007-AC-2 | For input `## Real\n\`\`\`\n## still-inside\n` (no closing fence), the parser returns 1 section (`Real`) — the trailing `## still-inside` is inside the unclosed block. | Test |
| FR-007-AC-3 | For input `~~~\n# fake\n~~~\n## Real`, the parser returns 1 section (`Real`); the `# fake` is inside the tilde fence. | Test |
| FR-007-AC-4 | A test transliterated from `~/dev/quire/tests/` covering each variant passes. | Test |
| FR-007-AC-5 | For input `## A\n````\n```\n# fake\n````\n## B` the shorter inner fence does not close the block, so `# fake` is not a heading and the parser returns `A` and `B`; a four-space-indented ` ``` ` line opens no fence. | Test |

## Dependencies

- **Upstream**: [US-002](../usecase/US-002-developer-parses-spec-doc.md), [StR-003](../stakeholder/StR-003-parse-parity-with-quire.md)
- **Downstream**: [FR-005](./FR-005-parse-document-api.md) (`parse_document` heading walk relies on this fence tracking)
