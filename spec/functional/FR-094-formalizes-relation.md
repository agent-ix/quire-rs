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

When a QSL source carries a `formalizes` marker declared by the active module directly above a
declaration, the engine SHALL record a `formalizes` relation from that declaration to each id the
marker names, SHALL report a marker that binds no declaration or names an id absent from the corpus,
and SHALL NOT count the relation as `verifies` or `implements`.

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
  pattern: '^//\s*formalizes:\s*(.*)$'
  template: '// formalizes: {ids}'
```

The pattern is applied to each `//` comment line of the block directly above a declaration; capture
group 1 is a comma-separated id list, split on `,` with surrounding whitespace trimmed. Each id uses
the id grammar of the `implements` forms (requirement ids and criterion ids), and may be qualified as
`ix://<org>/<repo>/<id>` as the corpus already resolves them. An empty entry (a trailing or doubled
comma) is skipped when the marker has at least one other entry. A marker with no entry at all
(`// formalizes:` followed only by whitespace, or `// formalizes: ,`) is an id-less marker: it
mints no relation and is reported as `formalizes-unknown-id` with an empty id, naming the file and
the marker line. An entry that does not match the id grammar (`not-an-id`) is reported as
`formalizes-unknown-id`, naming the entry verbatim, and mints no relation.

### QSL declaration symbol

The QSL adapter (`src/symbols/qsl.rs`) reads `*.native` files line by line and mints a symbol for a
line that satisfies all of:

1. the line starts at column 0 (its first character is not whitespace);
2. it lies at brace depth 0, where `{` and `}` inside a `//` comment or a string literal do not
   count. No QSL requirement defines string-literal lexing that this adapter can cite, so the
   delimiter and escape behaviour below is **this adapter's own approximation**, not QSL's lexer. It
   can diverge from QSL on a string form it does not model, in which case brace depth, and so
   which lines are declarations, may be wrong for that file; QSL owns its grammar (CON-1). For this
   scan a string literal opens at a `"` outside a comment, closes at the next `"` not
   preceded by an unescaped `\`, a `\` escapes the character after it, and a string does not span
   lines: an unterminated string ends at the end of its line;
3. it matches `^([a-z][a-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)`, a lowercase keyword followed by an
   identifier; and
4. the keyword is not `language`, `profile` or `model`.

A `.native` file whose braces do not balance (a `{` never closed by end of file, or a `}` at depth
0) yields one per-file diagnostic with reason `qsl-unbalanced-braces` at FR-057 severity `warning`,
naming the file and the 1-based line of the stray `}`, or, when braces remain open at end of
file, of the outermost unclosed `{` (the earliest-opened `{` that is never closed). Such a file yields no symbol
and no `formalizes` diagnostic, and the rest of the tree extracts normally (FR-051-CON-2).

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
| `formalizes-unknown-id` | a marker entry names no requirement or criterion in the loaded corpus: an id that is absent, renamed or retired; text that is not an id; or a marker with no entry at all (reported with an empty id) |
| `duplicate-formalizes-claim` | the same id appears twice on one marker, or two markers on one declaration name the same id |

### Census and mentions

A `declaration` symbol is neither a census candidate nor an `untracked_symbols` entry, and the
`qsl` language adds no row to `backed` or `totals`. `language_confidence` (FR-077-AC-5) maps `qsl`
to `line_heuristic`. An id-shaped token in a `.native` comment that is not part of a `formalizes`
marker is recorded as a `Mention` (FR-077-AC-2); it raises no `trace-id-outside-marker` advisory and
no `unmatched_tags` record.

### Out of scope

This requirement defines no draft or approved state, no formalization coverage count, no matrix
column and no evidence record; those belong to later tickets under the same program. It cites VER-74
as its origin.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-094-AC-1 | A `.native` fixture with `// formalizes: FR-001` directly above the line `pre P1 using v on M::Server::request { }` yields exactly one `formalizes` relation from `qsl:<path>::P1` to `FR-001`. | Test |
| FR-094-AC-2 | A marker naming `FR-001, FR-001-AC-2` yields two relations from the same symbol. | Test |
| FR-094-AC-3 | Two declarations in two files both naming `FR-001` yield two relations, listed in (path, label) order, and no diagnostic. | Test |
| FR-094-AC-4 | A marker followed by a non-declaration line, and a marker followed by end of file, each yield `formalizes-unbound` naming the file and the marker line, and no relation. A marker separated from its declaration only by blank lines and further `//` lines binds. | Test |
| FR-094-AC-5 | A marker naming `FR-999`, absent from the loaded corpus, yields `formalizes-unknown-id` naming the id, file and line; the relation is not minted. | Test |
| FR-094-AC-6 | Renaming the label `P1` to `Pre1` with the marker left in place yields a relation from `::Pre1` and no diagnostic. Renaming the requirement `FR-001` to `FR-050` without updating the marker yields `formalizes-unknown-id` for `FR-001`. | Test |
| FR-094-AC-7 | A `formalizes` relation never appears in `verifies` or `implements` output and never moves an acceptance criterion from unbacked to backed; `backed`, `totals`, the census `candidates` and `untracked_symbols` are identical for a tree with `.native` declarations and markers and the same tree without them. | Test |
| FR-094-AC-8 | A corpus with no `.native` file yields the same output as before this requirement under any module, except that the diagnostic reason registry lists `formalizes-unbound`, `formalizes-unknown-id` and `duplicate-formalizes-claim`. For a corpus with `.native` files under a module that declares no `trace_tags.formalizes`, the output restricted to records that do not originate from a `.native` file is byte-identical to the same corpus with those files removed, and no `formalizes` relation and no `formalizes-*` diagnostic is emitted. | Test |
| FR-094-AC-9 | A line inside a `{…}` body, an indented line, and lines whose first token is `language`, `profile` or `model` are never symbols; a `{` or `}` inside a `//` comment or string literal does not change brace depth. | Test |
| FR-094-AC-10 | The same id twice on one marker yields one relation and one `duplicate-formalizes-claim` warning; two markers on one declaration naming the same id yield one relation and one warning. | Test |
| FR-094-AC-11 | Two declarations with the same label in one `.native` file yield the existing `symbol-id-collision` warning; a `.native` file with markers but no declaration yields `formalizes-unbound` for each marker and no symbol. | Test |
| FR-094-AC-12 | A `ix://<org>/<repo>/FR-001` qualified id on a marker resolves as the corpus resolves it, and an unresolvable qualified id yields `formalizes-unknown-id`. | Test |
| FR-094-AC-15 | `// formalizes: FR-001,` mints one relation to `FR-001` and no diagnostic; `// formalizes: not-an-id` yields `formalizes-unknown-id` naming `not-an-id` and no relation; `// formalizes:`, `// formalizes:   ` and `// formalizes: ,` each yield one `formalizes-unknown-id` with an empty id naming the file and marker line, and no relation. | Test |
| FR-094-AC-16 | A `.native` file with an unclosed `{` yields one `qsl-unbalanced-braces` warning naming the file and the line of the outermost unclosed `{` (with two nested unclosed `{`, the outer one's line), no symbol and no `formalizes-unbound`; a stray `}` at depth 0 yields it naming that line; the rest of the tree extracts normally. | Test |
| FR-094-AC-17 | A string literal `"a\"{"` containing an escaped quote followed by `{` leaves brace depth unchanged, so the declaration on the next line at column 0 is still a symbol; an unterminated string ends at its line and does not suppress later lines. | Test |
| FR-094-AC-18 | An id-shaped token in a `.native` comment outside a marker is a `Mention` and raises no `trace-id-outside-marker` finding and no `unmatched_tags` record; `qsl` maps to confidence `line_heuristic`. | Test |
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
