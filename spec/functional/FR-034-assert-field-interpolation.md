---
id: FR-034
title: "Assert Field Interpolation"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-033"
    type: "extends"
    cardinality: "1:1"
---

## Behavior

Per ADR 0005 (#3), assert pattern fields (e.g. `id_pattern`) MAY contain
`{field}` tokens that are resolved at **validate time** from the document's own
frontmatter. `{id}` resolves to `frontmatter.id`, `{title}` to
`frontmatter.title`, etc. Interpolation runs before the pattern is compiled as a
regex.

This enables traceability/completeness assertions that tie body content to the
document's own identity — e.g. `id_pattern: '^{id}-AC-\d+$'` asserts that every
Acceptance-Criteria id is prefixed with *this* document's id.

Resolution rules:

- A `{field}` whose frontmatter value is absent SHALL produce a line-numbered
  diagnostic naming the archetype, the locator, and the unresolved field; the
  assert SHALL NOT silently pass.
- Interpolated values SHALL be regex-escaped before substitution so a value
  containing regex metacharacters does not alter the pattern's meaning.
- Interpolation is limited to frontmatter scalar fields; it is not a general
  expression language.

## Acceptance

- **FR-034-AC-1**: For a document with frontmatter `id: FR-003`, `id_pattern: '^{id}-AC-\\d+$'` accepts rows `FR-003-AC-1`, `FR-003-AC-2` and rejects a row `FR-009-AC-1`.
- **FR-034-AC-2**: A `{field}` referencing a frontmatter key absent from the document yields a diagnostic naming the archetype, locator, and missing field; validation does not pass on that assert.
- **FR-034-AC-3**: A frontmatter value containing regex metacharacters (e.g. `id: A.B+`) is regex-escaped, so `{id}` matches the literal value and is not interpreted as a pattern.
- **FR-034-AC-4**: An assert pattern with no `{field}` token behaves as a plain static regex (no interpolation pass observable).
