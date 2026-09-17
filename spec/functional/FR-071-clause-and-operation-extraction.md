---
id: FR-071
title: "Clause and operation extraction to ClauseRef[] and OperationDecl[]"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/semantic_clauses.rs
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
  `operations-cases.json` fixtures from `agent-ix/quoin` at revision `3e842ce`.

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
- If the tag is outside the `ClauseLanguage` pattern of the pinned
  semantic-core version, then the engine SHALL emit
  `semantic.clause-language-invalid` at the fence.
- When the tag is `quire` under a semantic-core version that admits it, the
  engine SHALL extract the clause with no advisory.
- When the tag is `ocl`, `sysml`, `fretish`, or namespaced such as
  `acme:tla`, the engine SHALL extract the clause, emit the advisory
  `semantic.clause-language-unchecked`, and mark the kind lossy, under every
  semantic-core version.
- `quire` is the only checked clause language. `tla` is outside every
  version's `ClauseLanguage` pattern; `quire` is outside the `0.1.0` pattern,
  so a `0.1.0` module has no checked language.
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
  `Post:` lines each naming one or more comma-separated clause ids of the
  same artifact's `## Invariants`. `Pre:` fills the operation's `pre`
  and `Post:` its `post`.
- If a `Pre:` or `Post:` id names no invariant clause, then the engine SHALL
  emit `semantic.dangling-clause-ref` at that line. If a second operation
  carries a name already declared, then the engine SHALL emit
  `semantic.duplicate-operation` at the second heading.
- If any clause or operation in a section carries an `error`, then the engine SHALL set that kind `unavailable` with reason `entry-errors` listing the loci, emitting no partial array.

General:

- Every produced `ClauseRef` and `OperationDecl` SHALL validate against the
  vendored `ClauseRef.json` and `OperationDecl.json`; an entry that does not
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

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-071-CON-1 | No code path SHALL tokenize, typecheck, or evaluate clause text; the only operations on a fence body are span recording and byte copy. | Architecture | Test |
| FR-071-CON-2 | The engine SHALL derive spans from the `code_block` locator's fence recognition, leaving `parse_document` output byte-identical to the checked-in parser golden. | Integrity | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-071-AC-1 | The vendored `operations.md` extracts to the `clauses`, `operations`, and `clauseText` of `operations.expected.json`, and `config-version.table.md` extracts its `immutable` clause with the span recorded in `config-version.expected.json`; every `clauseText` value equals the fence body byte-for-byte. | Test |
| FR-071-AC-2 | The six `fence-` cases of `operations-cases.json` (semantic-core `0.2.0`), `fence-ocl-carried` among them, yield the recorded code, severity, column, message, and fence locus, and the recorded `clauses`, `clauseText`, and clause availability where present; `quire` yields no advisory. | Test |
| FR-071-AC-3 | `duplicate-clause-id` fails at the second heading, `clause-id-not-identifier` at the heading, `inline-and-external` at the second occurrence; an ownerless fence, a bodiless heading, a heading with two fences, an unterminated fence, and an external-only `Clause:` line each yield their named code at the named locus. | Test |
| FR-071-AC-4 | `dangling-post` fails at the `Post:` line with `semantic.dangling-clause-ref`, `duplicate-operation` at the second heading, a non-`Identifier` operation heading with `semantic.operation-name-not-identifier`; `Pre: a, b` resolves both ids into `pre`; an operation without a table yields `params: []`. | Test |
| FR-071-AC-5 | Every produced `ClauseRef` and `OperationDecl` validates against the vendored schemas; an artifact without the sections reports both kinds `not_applicable`; a section with one erroring entry reports that kind `unavailable` (`entry-errors`) with no partial array. | Test |
| FR-071-AC-6 | For generated fence bodies of arbitrary UTF-8 text (backticks, tilde fences, longer closing runs, CRLF, nested shorter fences), `clauseText` equals the body bytes and the span's `startLine`, `endLine`, and `endColumn` match the fence lines. | Test |
| FR-071-AC-7 | With no caller-supplied `sourceIdentity`, spans carry `ix://local/<scope>/spec` and one `semantic.source-identity-defaulted` advisory per document; with one supplied, spans carry it and no advisory. | Test |
| FR-071-AC-8 | A `quire` fence under `0.2.0` extracts with no diagnostic and the kind available and not lossy; an `ocl` fence under `0.2.0` and under `0.1.0` extracts with only the advisory `semantic.clause-language-unchecked` at the fence, its body carried verbatim, and the kind lossy; a `quire` fence under `0.1.0` is `semantic.clause-language-invalid`. | Test |
| FR-071-AC-9 | The `fence-quire-invalid` case of `clause-language-0.1.0-cases.json` (semantic-core `0.1.0`) yields `semantic.clause-language-invalid` at the fence, column 1, with the recorded message, empty `clauseText`, and the clause kind unavailable with reason `entry-errors` and not lossy. | Test |

## Dependencies

- **Upstream**: [FR-070](./FR-070-typed-properties-extraction.md), [FR-005](./FR-005-parse-document-api.md); `agent-ix/quoin` FR-072
- **Downstream**: [FR-072](./FR-072-semantic-extraction-surface.md); `agent-ix/quire-contract-ir#55`
