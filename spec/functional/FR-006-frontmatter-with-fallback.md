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

## Behavior

`extract_frontmatter(markdown: &str) -> FrontmatterResult` SHALL:

0. **Byte-Order Mark (BOM) handling**: if `markdown` begins with the UTF-8 BOM (`\xEF\xBB\xBF`), the BOM is stripped before frontmatter detection. The stripped BOM is NOT included in the returned `body`. This matches what editors silently inject and prevents BOM-prefixed files from falling through to the "no frontmatter" branch.
1. If `markdown` (post-BOM-strip) does not begin with `---\n` (allowing for `\r\n` line endings), return `FrontmatterResult { frontmatter: None, body: markdown.into() }`.
2. If `markdown` begins with `---\n` but no closing `---\n` line exists, return `FrontmatterResult { frontmatter: None, body: markdown.into() }`.
3. If a closing `---\n` exists but the content between the fences is not valid YAML, return `FrontmatterResult { frontmatter: None, body: markdown.into() }` — **the entire input is body**. This matches the TS/Py reference: malformed frontmatter is NOT an error; it is treated as content.
4. If the content is valid YAML, return `FrontmatterResult { frontmatter: Some(parsed), body: text_after_closing_fence }`.

The parsed frontmatter is a `serde_json::Map<String, Value>` (JSON-compatible value tree), not a typed struct — typed deserialization is the consumer's responsibility.

## Acceptance

- **FR-006-AC-1**: `extract_frontmatter("# heading")` returns `frontmatter: None, body: "# heading"`.
- **FR-006-AC-2**: `extract_frontmatter("---\nid: FR-001\n---\nbody")` returns frontmatter with `id == "FR-001"` and body `"body"`.
- **FR-006-AC-3**: `extract_frontmatter("---\nid: : malformed\n---\nbody")` (invalid YAML) returns `frontmatter: None, body: full original input` — **not an error**.
- **FR-006-AC-4**: `extract_frontmatter("---\nid: FR-001\nno closing fence\nbody")` returns `frontmatter: None, body: full original input`.
- **FR-006-AC-5**: `extract_frontmatter("\u{FEFF}---\nid: FR-001\n---\nbody")` (BOM-prefixed input) returns the same result as the BOM-free equivalent — frontmatter parsed, body equal to `"body"` (no BOM in body).
- **FR-006-AC-6**: `extract_frontmatter("\u{FEFF}# heading")` (BOM-prefixed, no frontmatter) returns `frontmatter: None, body: "# heading"` (BOM stripped from body).
