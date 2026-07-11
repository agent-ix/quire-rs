---
id: FR-006
title: "Frontmatter Extraction with Malformed-Fallback Semantics"
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

`extract_frontmatter(markdown: &str) -> FrontmatterResult` SHALL:

0. **Byte-Order Mark (BOM) handling**: if `markdown` begins with the UTF-8 BOM (`\xEF\xBB\xBF`), the BOM is stripped before frontmatter detection. The stripped BOM is NOT included in the returned `body`. This matches what editors silently inject and prevents BOM-prefixed files from falling through to the "no frontmatter" branch.
1. If `markdown` (post-BOM-strip) does not begin with `---\n` (allowing for `\r\n` line endings), return `FrontmatterResult { frontmatter: None, body: markdown.into() }`.
2. If `markdown` begins with `---\n` but no closing `---\n` line exists, return `FrontmatterResult { frontmatter: None, body: markdown.into() }`.
3. If a closing `---\n` exists but the content between the fences is not valid YAML, return `FrontmatterResult { frontmatter: None, body: markdown.into() }` — **the entire input is body**. This matches the TS/Py reference: malformed frontmatter is NOT an error; it is treated as content.
4. If the content is valid YAML, return `FrontmatterResult { frontmatter: Some(parsed), body: text_after_closing_fence }`.
5. **Frontmatter status (CR-011).** The result SHALL additionally carry a typed `status ∈ {Absent, Present, Malformed}` reporting *why* `frontmatter` is `None`, or that it parsed: branches 1–2 yield `Absent` (no block, or an unterminated fence); an invalid-YAML block (branch 3) or a valid-but-non-mapping value (array/scalar/bool/number) yields `Malformed`; an **empty / whitespace / comment-only** block (YAML null) yields `Absent` (it carries no metadata and is indistinguishable from having no frontmatter — e.g. a `---`…`---` pair of thematic breaks — so it is not a parse failure); and branch 4 yields `Present`. This is a parity-preserving extension: the `frontmatter` and `body` outputs are byte-identical across all branches (like the BOM/CRLF extensions), so the TS/Py reference comparison is unaffected. The status exists so a boundary consumer (e.g. the Filament extraction engine, [FR-045](./FR-045-filament-core-extraction-engine.md)) can distinguish an absent block from a malformed one without re-deriving it from the raw markdown.

The parsed frontmatter is a `serde_json::Map<String, Value>` (JSON-compatible value tree), not a typed struct — typed deserialization is the consumer's responsibility.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-006-AC-1 | `extract_frontmatter("# heading")` returns `frontmatter: None, body: "# heading"`. | Test |
| FR-006-AC-2 | [FR-001](./FR-001-render-dispatch.md) returns frontmatter with [FR-001](./FR-001-render-dispatch.md) and body `"body"`. | Test |
| FR-006-AC-3 | `extract_frontmatter("---\nid: : malformed\n---\nbody")` (invalid YAML) returns `frontmatter: None, body: full original input` — not an error. | Test |
| FR-006-AC-4 | [FR-001](./FR-001-render-dispatch.md) returns `frontmatter: None, body: full original input`. | Test |
| FR-006-AC-5 | [FR-001](./FR-001-render-dispatch.md) (BOM-prefixed input) returns the same result as the BOM-free equivalent — frontmatter parsed, body equal to `"body"` (no BOM in body). | Test |
| FR-006-AC-6 | `extract_frontmatter("\u{FEFF}# heading")` (BOM-prefixed, no frontmatter) returns `frontmatter: None, body: "# heading"` (BOM stripped from body). | Test |
| FR-006-AC-7 | The result `status` (CR-011) classifies each branch: absent/unterminated → `Absent`; a valid mapping → `Present`; invalid YAML or a non-null non-mapping value (array/scalar) → `Malformed`; an empty/whitespace/comment-only block (YAML null) → `Absent` (not a parse failure). | Test (TC-706) |

## Dependencies

- **Upstream**: [US-002](../usecase/US-002-developer-parses-spec-doc.md), [StR-003](../stakeholder/StR-003-parse-parity-with-quire.md)
- **Downstream**: [FR-005](./FR-005-parse-document-api.md) (`parse_document` populates `QuireDocument.frontmatter` via this)
