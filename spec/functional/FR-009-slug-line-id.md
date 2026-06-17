---
id: FR-009
title: "Stable Slug-Line ID Format for QuireSection"
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

`QuireSection.id` SHALL be `<slug>-L<line>` where:

1. `slug` is derived from the heading text by:
   a. Lowercasing via Unicode-default lowercasing (`str::to_lowercase`).
   b. Replacing every run of one-or-more characters that are NOT in the ASCII alphanumeric set `[a-z0-9]` (post-lowercase) with a single `-`. This intentionally collapses Unicode letters (e.g. `é`, `ü`) into `-`, matching the TS/Py reference implementations' regex `[^a-z0-9]+`.
   c. Stripping leading and trailing `-`.
2. `line` is the 0-based line index of the heading line within the body (NOT within the full markdown input; the frontmatter, if present, is stripped first).

Specifically: heading `"2.1 In Scope"` at body line index 6 produces id `"2-1-in-scope-L6"`. Heading `"Café Menu"` produces slug `"caf-menu"` (the `é` collapses to `-`). This matches the TS `slug + "-L" + startLine` rule and the Python sibling.

### Empty slug

A heading text that produces an empty slug after normalization (e.g. `"## !!!"`, `"## ❤️"`) yields id `"-L<line>"`. This is unusual but legal — preserving the line index keeps the ID unique even when the slug is degenerate. Authors are responsible for choosing headings that produce meaningful slugs.

### Internationalization (deferred to v1.1)

Slug normalization is ASCII-only at v1 (matching TS/Py reference). Non-ASCII headings parse correctly (the section, content, line index, and heading text are preserved), but the slug collapses non-ASCII characters to `-`. A heading "Café Menu" produces slug `caf-menu`, not `cafe-menu`. Full Unicode slug normalization (e.g. NFD + strip diacritics) is deferred — adding it later is non-breaking for the few documents that depend on the empty-slug behavior, but breaking for any consumer that has cached `caf-menu-L4` style IDs.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-009-AC-1 | Heading `"Hello, World!"` at line 0 yields id `"hello-world-L0"`. | Test |
| FR-009-AC-2 | Heading `"2.1 In Scope"` at line 6 yields id `"2-1-in-scope-L6"`. | Test |
| FR-009-AC-3 | Heading `"   leading spaces   "` at line 3 yields id `"leading-spaces-L3"`. | Test |
| FR-009-AC-4 | With a frontmatter block of N lines, a heading on the first body line has line index 0 — frontmatter is NOT counted. | Test |
| FR-009-AC-5 | A Rust-owned parity test covering each slug variant passes. | Test |
| FR-009-AC-6 | Heading `"Café Menu"` at line 4 yields id `"caf-menu-L4"` (non-ASCII collapses; mirrors TS/Py reference). | Test |
| FR-009-AC-7 | Heading `"!!!"` at line 7 yields id `"-L7"` (degenerate empty slug, line index preserved). | Test |

## Dependencies

- **Upstream**: US-002, StR-003
- **Downstream**: none
