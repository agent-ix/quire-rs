---
id: FR-005
title: "parse_document API and QuireDocument Shape"
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

The crate SHALL export:

```rust
pub fn parse_document(markdown: &str) -> QuireDocument;
```

and the types:

```rust
pub struct QuireSection {
    pub id: String,
    pub heading: String,
    pub level: u8,
    pub content: String,
    pub children: Vec<QuireSection>,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct QuireDocument {
    pub preamble: Option<String>,
    pub sections: Vec<QuireSection>,
    pub raw: String,
    pub frontmatter: Option<serde_json::Map<String, serde_json::Value>>,
}
```

Both structs SHALL `derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)`.

`parse_document` SHALL:

1. Be pure — no IO, no panics, no global mutable state.
2. Accept arbitrary UTF-8 input including empty string.
3. For empty input, return `QuireDocument { preamble: None, sections: vec![], raw: "".into(), frontmatter: None }`.
4. Be re-entrant from any thread.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | `parse_document("")` returns the empty-document value above. | Test |
| FR-005-AC-2 | `parse_document("preamble only")` returns `QuireDocument { preamble: Some("preamble only"), sections: vec![], raw: "preamble only", frontmatter: None }`. | Test |
| FR-005-AC-3 | For the canonical TS fixture (`## Parent\nparent content\n### Child\nchild content\n## Sibling\nsibling content`), the returned document has 2 top-level sections; `Parent` has 1 child `Child`; section IDs are `parent-L0`, `child-L2`, `sibling-L4` (or the equivalent with the TS slug rule applied — see FR-009). | Test |
| FR-005-AC-4 | A proptest checks `parse_document` does not panic on 10000 random UTF-8 strings. | Test |

## Dependencies

- **Upstream**: US-002, StR-003
- **Downstream**: FR-006, FR-007, FR-008, FR-009 (parse internals that build the `QuireDocument`)
