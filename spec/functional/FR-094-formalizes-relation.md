---
id: FR-094
title: "The requirement-to-formal-rule relation"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-062"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "traces_to"
---

# FR-094: The requirement-to-formal-rule relation

## Description

A requirement written in Markdown has no machine-checked link to the QSL rule that states it
formally. [FR-051](./FR-051-source-symbol-extraction.md) mints `verifies` (a test is evidence for a
criterion) and [FR-062](./FR-062-implements-relation.md) mints `implements` (production code is the
scope of a requirement). This requirement adds a third relation, `formalizes`: a QSL declaration is
the formal statement of a requirement or criterion (VER-74).

The link is written in the QSL source, as a `//` line-comment marker directly above the declaration
it belongs to. The requirement Markdown is not edited. QSL comments are trivia, so a marker changes
no compiled output; the engine neither compiles nor checks QSL.

### Three relations, three lists

| Relation | Means | Declared in | May back an acceptance criterion |
|---|---|---|---|
| `verifies` | a test would fail if the behaviour broke | `trace_tags.markers` | yes |
| `implements` | this code is what the requirement is about | `trace_tags.implements` | never |
| `formalizes` | this declaration is the formal statement | `trace_tags.formalizes` | never |

The separation is structural, as FR-062 requires of `implements`: a separate declared list
(`trace_tags.formalizes`), a separate type (`FormalizesRelation`, never `VerifiesRelation` or
`ImplementsRelation` with a field), and a separate symbol-record list (`formalizes`).

### Marker form

The module declares the form; the engine declares no pattern (CON-3). The shipped form is:

```yaml
formalizes:
- name: qsl-formalizes-comment
  language: qsl
  syntax: comment
  pattern: '^//\s*formalizes:\s*(.+)$'
  template: '// formalizes: {ids}'
```

The pattern is applied to each `//` comment line of the block directly above a declaration; capture
group 1 is a comma-separated id list, split on `,` with surrounding whitespace trimmed. Each id uses
the id grammar of the `implements` forms (requirement ids and criterion ids), and may be qualified as
`ix://<org>/<repo>/<id>` as the corpus already resolves them.

### QSL declaration symbol

The QSL adapter (`src/symbols/qsl.rs`) reads `*.native` files line by line and mints a symbol for a
line that satisfies all of:

1. the line starts at column 0 (its first character is not whitespace);
2. it lies at brace depth 0, where `{` and `}` inside a `//` comment or a string literal do not
   count;
3. it matches `^([a-z][a-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)`, a lowercase keyword followed by an
   identifier; and
4. the keyword is not `language`, `profile` or `model`.

The identifier is the declaration label. The symbol kind is `declaration`, the qualified name is the
label, and the symbol id is `qsl:<repo-relative path>::<label>`. The adapter validates no other
keyword: it is not a second QSL grammar (CON-1).

### Binding

A marker binds to the first declaration after it, provided only blank lines or further `//` lines lie
between them. A marker followed by any other line, or by end of file, binds nothing and yields
`formalizes-unbound`, naming the file and the 1-based marker line (the mechanism of FR-051-AC-44).

A declaration rename leaves the link intact, because the marker binds to whatever declaration follows
it. A requirement rename or retirement leaves the marker naming an absent id, which is reported as
`formalizes-unknown-id`.

### Cardinality

One declaration may formalize several ids, and one id may be formalized by several declarations,
including in different files. Every relation is reported, in (path, label) order. Several claims on
one id are not a conflict and raise no diagnostic. Two declarations in one file with the same label
yield the existing `symbol-id-collision` warning (FR-051-AC-31).

### Diagnostics

All three are reported in `CoverageReport.diagnostics` at severity `warning`, naming the id (where
one applies), the file and the 1-based line:

| Reason | Condition |
|---|---|
| `formalizes-unbound` | a marker binds no declaration |
| `formalizes-unknown-id` | a marker names an id that no requirement or criterion in the loaded corpus carries, including an id whose requirement was renamed or retired |
| `duplicate-formalizes-claim` | the same id appears twice on one marker, or two markers on one declaration name the same id |

### Out of scope

This requirement defines no draft or approved state, no formalization coverage count, no matrix
column and no evidence record; those belong to later tickets under the same program. It cites VER-74
as its origin.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-094-AC-1 | A `.native` fixture with `// formalizes: FR-001` directly above `pre P1 …` yields exactly one `formalizes` relation from `qsl:<path>::P1` to `FR-001`. | Test |
| FR-094-AC-2 | A marker naming `FR-001, FR-001-AC-2` yields two relations from the same symbol. | Test |
| FR-094-AC-3 | Two declarations in two files both naming `FR-001` yield two relations, listed in (path, label) order, and no diagnostic. | Test |
| FR-094-AC-4 | A marker followed by a non-declaration line, and a marker followed by end of file, each yield `formalizes-unbound` naming the file and the marker line, and no relation. A marker separated from its declaration only by blank lines and further `//` lines binds. | Test |
| FR-094-AC-5 | A marker naming `FR-999`, absent from the loaded corpus, yields `formalizes-unknown-id` naming the id, file and line; the relation is not minted. | Test |
| FR-094-AC-6 | Renaming the label `P1` to `Pre1` with the marker left in place yields a relation from `::Pre1` and no diagnostic. Renaming the requirement `FR-001` to `FR-050` without updating the marker yields `formalizes-unknown-id` for `FR-001`. | Test |
| FR-094-AC-7 | A `formalizes` relation never appears in `verifies` or `implements` output and never moves an acceptance criterion from unbacked to backed; `backed`, `totals` and `untracked_symbols` are identical with and without the marker. | Test |
| FR-094-AC-8 | A module that declares no `trace_tags.formalizes` yields byte-identical output to today over a corpus containing `.native` files, and the same corpus under that module reports no `formalizes` relation. | Test |
| FR-094-AC-9 | A line inside a `{…}` body, an indented line, and lines whose first token is `language`, `profile` or `model` are never symbols; a `{` or `}` inside a `//` comment or string literal does not change brace depth. | Test |
| FR-094-AC-10 | The same id twice on one marker yields one relation and one `duplicate-formalizes-claim` warning; two markers on one declaration naming the same id yield one relation and one warning. | Test |
| FR-094-AC-11 | Two declarations with the same label in one `.native` file yield the existing `symbol-id-collision` warning; a `.native` file with markers but no declaration yields `formalizes-unbound` for each marker and no symbol. | Test |
| FR-094-AC-12 | A `ix://<org>/<repo>/FR-001` qualified id on a marker resolves as the corpus resolves it, and an unresolvable qualified id yields `formalizes-unknown-id`. | Test |
| FR-094-AC-13 | Two runs over identical input emit identical relation and diagnostic order. | Test |
| FR-094-AC-14 | This feature adds no dependency to `Cargo.toml`, and the QSL adapter reads each file once. | Inspection |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-094-CON-1 | The engine SHALL hold no QSL keyword list beyond the `language`, `profile` and `model` header exclusions; QSL owns its grammar. | Design | Inspection |
| FR-094-CON-2 | The engine SHALL add no QSL compiler or crate dependency. | Architecture | Inspection |
| FR-094-CON-3 | The engine SHALL NOT declare a `formalizes` marker pattern; the forms are module data. | Design | Inspection |
| FR-094-CON-4 | `formalizes` SHALL NOT reuse `VerifiesRelation` or `ImplementsRelation` and does not contribute to `backed_trace_ids` or any FR-050 rollup figure. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (symbol extraction and the marker grammar), [FR-062](./FR-062-implements-relation.md) (the sibling relation and its separation rule)
- **Downstream**: [FR-077](./FR-077-trace-search-index.md) (search results carry `formalizes`); `agent-ix/spec-artifacts-process` FR-004 (the module declaration of the form)
