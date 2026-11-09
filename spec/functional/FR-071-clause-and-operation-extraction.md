---
id: FR-071
title: "Clause and operation extraction to ClauseRef[] and OperationDecl[]"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/it/semantic_clauses.rs
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-019"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-070"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "requires"
    cardinality: "1:1"
---
# FR-071: Clause and operation extraction to ClauseRef[] and OperationDecl[]

## Description

When the engine extracts an object artifact whose module carries a
`semantic` block, the engine SHALL extract `## Invariants` into `ClauseRef[]`
and `## Operations` into `OperationDecl[]` per the `agent-ix/quoin` FR-072
mapping.

The engine SHALL carry each fence body verbatim with a source span, parsing
none of it.

## Inputs

- The parsed document, its `## Invariants` and `## Operations` sections as
  byte-exact section content with the section's start line
  ([FR-005](./FR-005-parse-document-api.md)).
- The `SemanticContext` of [FR-070](./FR-070-typed-properties-extraction.md),
  extended with `sourceIdentity`: the caller-supplied `ix://<org>/<repo>/spec`
  identity of the document's repository. The Filament API derives it from
  `org` and `repo_name`; the library and corpus surfaces take it as an input;
  when a caller supplies none, the engine uses `ix://local/<scope directory name>/spec`
  and emits one advisory `semantic.source-identity-defaulted` per document.
- The vendored `operations.md`, `operations.expected.json`, and
  `operations-cases.json` fixtures from `agent-ix/quoin`.

## Outputs

- `clauses[]`: `ClauseRef` entries `{ language, clauseId, sourceSpan }` in
  authored order, where `sourceSpan` is a semantic-core `SourceLocus`:
  `sourceIdentity` from the context, `path` the document's corpus-relative
  path, `startLine` the opening fence line, `startColumn` 1, `endLine` the
  closing fence line, `endColumn` one past the closing fence line's byte
  length.
- `clauseText`: a map from `clauseId` to the fence body: the bytes between
  the end of the opening fence line and the start of the closing fence line,
  final line terminator excluded, CR bytes preserved.
- `operations[]`: `OperationDecl` entries `{ name, params, returns?, pre, post }`
  whose `params` are `FieldDecl` entries (empty when no table) and whose
  `pre`/`post` are `ClauseRef` entries `{ language, clauseId }` copying the
  referenced clause's language.
- `operationExpressions?`: `{ operation, slot, expression }[]` of the
  inline expressions of `Pre:`/`Post:` lines (Behavior, "Inline
  expressions"), an additive optional key of the semantic-v1 record.
- Diagnostics with a `semantic.*` code, severity, and locus; per-kind
  availability for `clauses` and `operations`.

## Behavior

Invariants:

- Under `## Invariants`, each `### <clauseId>` heading SHALL own exactly one
  fenced block; deeper headings inside it belong to that clause. If the
  heading text is not a semantic-core `Identifier`, then the engine SHALL
  emit `semantic.clause-id-not-identifier` at the heading.
- If a fence has no owning `###` heading, then the engine SHALL emit
  `semantic.clause-without-id` at the fence. If a heading owns no fence and
  no `Clause:` line, then the engine SHALL emit `semantic.clause-missing-body`
  at the heading. If a heading owns two fences, then the engine SHALL emit
  `semantic.clause-multiple-bodies` at the second fence.
- The engine SHALL recognize a fence as the parser does: a run of three or
  more backticks or tildes at column 1, closed by a run of the same character
  at least as long; an unterminated fence is `semantic.clause-fence-unterminated`
  at the opening line.
- The engine SHALL take the fence language tag as the clause `language`.
- If the tag is missing, then the engine SHALL emit
  `semantic.clause-language-missing` at the fence.
- If the tag is outside the `ClauseLanguage` pattern of the embedded
  semantic-core, then the engine SHALL emit
  `semantic.clause-language-invalid` at the fence.
- When the tag is `quire`, the
  engine SHALL extract the clause with no advisory.
- When the tag is `ocl`, `sysml`, `fretish`, or namespaced such as
  `acme:tla`, the engine SHALL extract the clause, emit the advisory
  `semantic.clause-language-unchecked`, and mark the kind lossy.
- `quire` is the only checked clause language. `tla` is outside the
  `ClauseLanguage` pattern.
- The engine SHALL NOT check clause text in any language (FR-071-CON-1).
- If a second clause carries an id already declared under `## Invariants`,
  then the engine SHALL emit `semantic.duplicate-clause-id` at the second
  heading. If a clause is declared both by a fence and by a
  `Clause: ./<file>.md#<id>` line, then the engine SHALL emit
  `semantic.duplicate-clause-authority` at the second occurrence.
- A `Clause: ./<file>.md#<id>` line without a fence SHALL yield the advisory
  `semantic.clause-external-unsupported` at that line and no entry; the
  engine SHALL NOT read the named file.

Operations:

- Under `## Operations`, each `### <name>` heading (a semantic-core
  `Identifier`; otherwise `semantic.operation-name-not-identifier` at the
  heading) SHALL own an optional table with the header
  `Param | Type | Multiplicity | Constraints` (cells mapped by FR-070; a
  `unit` on `Returns` is `agent-ix.semantic-core.UNIT_ON_RETURNS`), an
  optional `Returns: <Type>[<mult>]` line, and optional `Pre:` and
  `Post:` lines, each holding either one or more comma-separated clause ids
  of the same artifact's `## Invariants` or one inline expression
  ("Inline expressions" below). `Pre:` fills the operation's `pre` and
  `Post:` its `post`.
- If a `Pre:` or `Post:` clause id names no invariant clause, then the engine SHALL
  emit `semantic.dangling-clause-ref` at that line. If a second operation
  carries a name already declared, then the engine SHALL emit
  `semantic.duplicate-operation` at the second heading.
- If any clause or operation in a section carries an `error`, then the engine SHALL set that kind `unavailable` with reason `entry-errors` listing the loci, emitting no partial array. A fixture case states those loci as `reasonLoci` (FR-076-AC-22).

Inline expressions (CR-199, PLAT-57):

- A `Pre:` or `Post:` line, and a `state_machine` transition `Guard` cell
  ([FR-075](./FR-075-model-feature-extraction.md)), is classified by its
  text alone, never by what is declared. The text is trimmed; if it is
  empty, see the empty-expression rule below. A `Pre:`/`Post:` line is split
  on commas and its empty pieces are dropped (as `comma_list` does today)
  BEFORE the test; if the remaining pieces are each a semantic-core
  `Identifier`, the line is a clause-id reference list, as before, so
  `Pre: a,` and `Pre: a,,b` stay reference lists. A `Guard` cell that is one
  `Identifier` is a clause-id reference. Any other `Pre:`/`Post:` line (for
  example `a, x > 1`), and any other `Guard` cell, is ONE inline expression:
  its whole trimmed text, never split on commas. The comma split is a
  classification step only; an expression line SHALL NOT be comma-split into
  references, ids or frame entries anywhere.
- A reference is resolved against the artifact's `## Invariants` and an id
  that names no clause stays `semantic.dangling-clause-ref`; it is never
  reinterpreted as an expression. A bare identifier is therefore always a
  reference, even though a bare name is also valid expression text; an
  author who means the expression writes it so that it is not a bare
  identifier (for example parenthesised). A mistyped id that is still an
  `Identifier` remains an error. A mistyped id that is not an `Identifier`
  (for example `Clause 1`, `a.b`, or an id with a trailing `;`) is not
  reference-shaped: it becomes an expression and yields only the advisory
  `semantic.expression-unchecked`, left to the consumer's syntax check.
- The engine SHALL carry each expression opaquely as an `Expression`
  `{ kind: "expression", text, sourceSpan }`: `text` the trimmed text
  byte-for-byte for a `Pre:`/`Post:` line (backslashes and backticks kept
  verbatim), and for a `Guard` cell the cell content as the table cell
  reader yields it after unescaping `\|` and stripping one outer backtick
  pair. `sourceSpan` is a semantic-core `SourceLocus` over the SOURCE bytes
  of the trimmed text or cell (`startLine` and `endLine` its line,
  `startColumn` the 1-based byte column of its first source byte,
  `endColumn` one past its last), so for a `Guard` cell the `text` length
  may differ from the span length. `kind` is
  the constant `"expression"`; it is what distinguishes an expression from
  a clause-id reference and what lets a later syntax check attach to the
  entry.
- The engine SHALL NOT parse, tokenize, typecheck, resolve names in, or
  evaluate an expression, and quire-rs has no expression grammar or parser.
  Syntax validation, name resolution inside an expression, and the meaning
  of QSpec's shared expression grammar (cited by reference only, with no copy
  of it here; its parser is tracked by QSL-1814, the consumer) belong to the
  consumer.
- For each expression the engine SHALL emit the advisory
  `semantic.expression-unchecked` at its line and mark the owning kind
  lossy, as for `semantic.clause-language-unchecked`.
- A `Pre:` or `Post:` line with no text after the key, or only commas and
  whitespace, is `semantic.empty-expression` at that line, an error. An empty `Guard`
  cell is an absent guard, as before.
- Expressions do not enter `OperationDecl`: its `pre` and `post` are closed
  semantic-core `ClauseRef[]` and carry reference ids only. Expressions are
  carried in the sibling record key `operationExpressions`, an array of
  `{ operation, slot, expression }` in authored order, `slot` `pre` or
  `post`, `operation` the operation name; the key is absent when no
  operation carries an expression. When `operations` is `unavailable`, or
  an operation is dropped, its expressions are omitted with it, and no
  expression is ever emitted without its operation. A transition `Guard` expression is
  carried as `guardExpression` ([FR-075](./FR-075-model-feature-extraction.md)).
  Mapping an expression into the declaration record is not specified here.

General:

- Every produced `ClauseRef` and `OperationDecl` SHALL validate against the
  embedded `ClauseRef.json` and `OperationDecl.json`; an entry that does not
  is an engine defect and SHALL fail extraction with
  `semantic.internal-invalid-decl`, never be dropped.
- An artifact without `## Invariants` or `## Operations` SHALL report that
  kind `not_applicable`.
- The engine SHALL compute spans with fence recognition equivalent to the
  `code_block` locator's (a run of three or more backticks or tildes, up to
  three leading spaces, closed by a run of the same character at least as
  long), proven by agreement on every fixture, leaving the
  [FR-005](./FR-005-parse-document-api.md) document shape unchanged.
- The engine SHALL read `Clause:`, `Returns:`, `Pre:`, and `Post:`
  lines outside fences only; a second `Returns:`, `Pre:`, or `Post:`
  line, or a
  second parameter table under one operation, is
  `semantic.duplicate-operation-line`.

> **CR-199 note (2026-11-08, agent-ix/quire-rs#433, PLAT-57):** the original
> text allowed only clause ids in `Pre:`/`Post:` and in a transition `Guard`,
> so a condition true only in some states had no place to live: an invariant
> must hold in every state, and naming one as a guard makes the guard vacuous.
> Both now also accept an inline expression, carried opaquely. The ticket
> calls the lines `Requires:`/`Ensures:`; the authored lines and this
> requirement are `Pre:`/`Post:`. Classification is by text shape rather than
> by resolution so that a mistyped clause id stays `semantic.dangling-clause-ref`.
> Affected criteria: FR-071-AC-10..14, FR-075-AC-28..29; no existing criterion
> changes.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-071-CON-1 | No code path SHALL tokenize, typecheck, or evaluate clause text; the only operations on a fence body are span recording and byte copy. | Architecture | Test |
| FR-071-CON-2 | The engine SHALL derive spans from the `code_block` locator's fence recognition, leaving `parse_document` output byte-identical to the checked-in parser golden. | Integrity | Test |
| FR-071-CON-3 | The only operations on expression text SHALL be the classification steps (trimming, the comma split with empty pieces dropped and the `Identifier` shape test on a `Pre:`/`Post:` line, the `Identifier` shape test, `\|` unescape and one backtick-pair strip on a `Guard` cell), span recording, and byte copy; an expression line is never split into references; quire-rs depends on no expression parser. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-071-AC-1 | The vendored `operations.md` extracts to the `clauses`, `operations`, and `clauseText` of `operations.expected.json`, and `config-version.table.md` extracts its `immutable` clause with the span recorded in `config-version.expected.json`; every `clauseText` value equals the fence body byte-for-byte. | Test |
| FR-071-AC-2 | The six `fence-` cases of `operations-cases.json`, `fence-ocl-carried` among them, yield the recorded code, severity, column, message, and fence locus, and the recorded `clauses`, `clauseText`, and clause availability where present; `quire` yields no advisory. | Test |
| FR-071-AC-3 | `duplicate-clause-id` fails at the second heading, `clause-id-not-identifier` at the heading, `inline-and-external` at the second occurrence; an ownerless fence, a bodiless heading, a heading with two fences, an unterminated fence, and an external-only `Clause:` line each yield their named code at the named locus. | Test |
| FR-071-AC-4 | `dangling-post` fails at the `Post:` line with `semantic.dangling-clause-ref`, `duplicate-operation` at the second heading, a non-`Identifier` operation heading with `semantic.operation-name-not-identifier`; `Pre: a, b` resolves both ids into `pre`; an operation without a table yields `params: []`. | Test |
| FR-071-AC-5 | Every produced `ClauseRef` and `OperationDecl` validates against the embedded schemas; an artifact without the sections reports both kinds `not_applicable`; a section with one erroring entry reports that kind `unavailable` (`entry-errors`) with no partial array. | Test |
| FR-071-AC-6 | For generated fence bodies of arbitrary UTF-8 text (backticks, tilde fences, longer closing runs, CRLF, nested shorter fences), `clauseText` equals the body bytes and the span's `startLine`, `endLine`, and `endColumn` match the fence lines. | Test |
| FR-071-AC-7 | With no caller-supplied `sourceIdentity`, spans carry `ix://local/<scope>/spec` and one `semantic.source-identity-defaulted` advisory per document; with one supplied, spans carry it and no advisory. | Test |
| FR-071-AC-8 | A `quire` fence under the embedded semantic-core extracts with no diagnostic and the kind available and not lossy; an `ocl` fence under the embedded semantic-core extracts with only the advisory `semantic.clause-language-unchecked` at the fence, its body carried verbatim, and the kind lossy. | Test |
| FR-071-AC-9 | Removed: the criterion covered semantic-core `0.1.0`, which no longer loads; the id is not reused. | Test |
| FR-071-AC-10 | `Pre: x > 1 and y(a, b)` yields one `operationExpressions` entry `{ operation, slot: "pre", expression: { kind: "expression", text: "x > 1 and y(a, b)", sourceSpan } }` with the span of the text on its line, `pre` empty, the advisory `semantic.expression-unchecked`, and the `operations` kind lossy; the expression text is not split on its comma. | Test |
| FR-071-AC-11 | `Pre: a, b` with both clauses declared yields references only and no `operationExpressions`; `Pre: a,` and `Pre: a,,b` stay reference lists with the empty pieces dropped; `Pre: a, x > 1` is one expression, its text unsplit; `Post: missing` where `missing` names no clause yields `semantic.dangling-clause-ref` and no expression; `Pre: (a)` yields an expression whereas `Pre: a` yields a reference, the choice depending on the text alone and not on whether `a` is declared. | Test |
| FR-071-AC-12 | A `Pre:` or `Post:` line with nothing after the key, or only commas, yields `semantic.empty-expression` at that line and the `operations` kind `unavailable` (`entry-errors`). | Test |
| FR-071-AC-13 | An expression with invalid Quire syntax (`Pre: ))) ===`) extracts byte-for-byte with only `semantic.expression-unchecked`: no syntax diagnostic is emitted, and `operationExpressions` validates against `semantic-v1.schema.json`. | Test |
| FR-071-AC-14 | With `operations` `unavailable` (an erroring entry) or an operation dropped, no `operationExpressions` entry is emitted for it, and none appears without its operation. | Test |

## Dependencies

- **Upstream**: [FR-070](./FR-070-typed-properties-extraction.md), [FR-005](./FR-005-parse-document-api.md); `agent-ix/quoin` FR-072; [FR-075](./FR-075-model-feature-extraction.md) (the `Guard` cell and `operationFrames` the inline-expression rules also govern)
- **Downstream**: [FR-072](./FR-072-semantic-extraction-surface.md); QSL-1814 (the expression syntax check, as the consumer of the `kind` tag); `agent-ix/quire-contract-ir#55`
