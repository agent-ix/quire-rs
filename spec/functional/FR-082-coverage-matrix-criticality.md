---
id: FR-082
title: "Requirement-level criticality in the coverage matrix"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-053"
    type: "references"
  - target: "ix://agent-ix/spec-artifacts-iso/FR-008"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "traces_to"
---
# FR-082: Requirement-level criticality in the coverage matrix

> **CR note (PLAT-1413, 2026-11-11):** A frontmatter `criticality` that is present with a null value is treated as absent: the entry carries no key.

## Description

The trace lookup of [FR-077](./FR-077-trace-search-index.md) is keyed by trace id (often an
AC or TC id) and has no requirement rows. The requirement-level model is the `requirements[]`
list of FR-050's `coverage_matrix` ([FR-050-AC-48](./FR-050-declarative-coverage-computation.md)):
one entry per source document, each with its criteria. Criticality therefore lives there.

This requirement extends each `requirements[]` entry with an optional `criticality` key. The
engine reads the value from the document's frontmatter `criticality` field
([spec-artifacts-iso FR-008](ix://agent-ix/spec-artifacts-iso/FR-008)) and SHALL carry it as written.
When the document has no such field, or the field is present with a null value (`criticality:` with no value, or `~`), the engine SHALL leave the key absent. The engine writes no
label, default or placeholder for a missing value; the human-readable label `unassigned` belongs
to the renderer ([quire-cli FR-028](ix://agent-ix/quire-cli/spec/functional/FR-028)).

### Relation to FR-053 criticality

[FR-053](./FR-053-obligation-record.md) carries a per-criterion `criticality`, read from a
module-declared column of an obligation source table. The two are different facts at different
levels: FR-053's rates one obligation row, and this field rates the requirement document. The
engine SHALL keep both as read and SHALL NOT let either supply, override or fill in the other.
Both use one missing-value convention: absent when not stated.

### No inference

The engine SHALL NOT derive a requirement's criticality from any other requirement, whatever the
relationship type (`traces_to`, `satisfies`, `implements`, `refines`, or any other verb), from its
criteria, or from the highest value among them. The engine SHALL NOT write a value back to any
document.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-082-AC-1 | A `requirements[]` entry whose document has frontmatter `criticality: high` carries `criticality` equal to `high`. | Test |
| FR-082-AC-2 | A `requirements[]` entry whose document has no `criticality` field has no `criticality` key at all, not an empty string, null or `unassigned`. | Test |
| FR-082-AC-3 | A document whose frontmatter is literally `criticality: unassigned` yields an entry with the key present and equal to `unassigned`, distinguishable from the absent-key entry of FR-082-AC-2. | Test |
| FR-082-AC-4 | For a corpus with values `low`, `medium`, `high` and one missing, each entry reports its own value and only the missing one lacks the key. | Test |
| FR-082-AC-5 | A document with no `criticality` that is related to a `criticality: high` document by each relationship type in the corpus (`traces_to`, `satisfies`, `implements`, `refines`) has no `criticality` key under any of them. | Test |
| FR-082-AC-6 | A document with an out-of-enumeration `criticality` is carried as written; the engine neither drops it nor replaces it. | Test |
| FR-082-AC-7 | For a document with a `criticality: high` frontmatter value whose obligation records ([FR-053](./FR-053-obligation-record.md)) carry no criticality, the obligation records of a source declaring no `criticality_field` still carry none and the `requirements[]` entry carries `high`; for a document with no frontmatter value whose obligation records carry a criticality, the entry has no `criticality` key and the obligation records keep theirs. | Test |
| FR-082-AC-8 | Running the matrix leaves every requirement file byte-identical. | Test |
| FR-082-AC-9 | Two runs over an identical corpus serialize `requirements[]` byte-identically, with and without the key (the FR-050-AC-51 determinism applies). | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-082-CON-1 | The engine holds no list of permitted `criticality` values; the module schema owns the enumeration. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (`coverage_matrix`), [FR-053](./FR-053-obligation-record.md) (per-criterion criticality), spec-artifacts-iso FR-008 (the field; lands in the same change set)
- **Downstream**: quire-cli FR-028 renders the value in `quire matrix`
