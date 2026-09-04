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
---
# FR-071: Clause and operation extraction to ClauseRef[] and OperationDecl[]

## Description

For an object artifact whose module carries a `semantic` block, the engine
SHALL extract `## Invariants` into `ClauseRef[]` and `## Operations` into
`OperationDecl[]` per the `agent-ix/quoin` FR-072 mapping, carrying each
fence body verbatim with a source span and parsing none of it.

## Inputs

- The parsed document, its `## Invariants` and `## Operations` sections, and
  the byte offsets the parser already records for fenced blocks.
- The module's `SemanticModule` record and the bundle index
  ([FR-070](./FR-070-typed-properties-extraction.md) resolution rules apply
  to parameter and return types).
- The vendored `operations.md`, `operations.expected.json`, and
  `operations-cases.json` fixtures from `agent-ix/quoin` at revision `3e842ce`.

## Outputs

- `clauses[]`: `ClauseRef` entries `{ language, clauseId, sourceSpan }` in
  authored order, where `sourceSpan` is a semantic-core `SourceLocus` with
  `sourceIdentity` `ix://<org>/<repo>/spec`, the corpus-relative path, the
  fence's opening line as `startLine`, `startColumn` 1, the closing line as
  `endLine`, and `endColumn` 4.
- `clauseText`: a map from `clauseId` to the fence body, byte-identical.
- `operations[]`: `OperationDecl` entries `{ name, params, returns?, pre, post }`
  whose `params` are `FieldDecl` entries and whose `pre`/`post` are
  `ClauseRef` entries without a span.
- Diagnostics with a `semantic.*` code, severity, and locus.

## Behavior

- Under `## Invariants`, each `### <clauseId>` heading SHALL own exactly one
  fenced block. The heading text SHALL be a semantic-core `Identifier`; a
  heading that is not (`### not-archived`) fails with
  `semantic.clause-id-not-identifier` at the heading.
- The fence language tag SHALL be the clause `language`: a missing tag fails
  with `semantic.clause-language-missing` at the fence; a tag outside the
  `ClauseLanguage` pattern (`tla`) fails with `semantic.clause-language-invalid`;
  `sysml`, `fretish`, and a namespaced tag (`acme:tla`) extract with the
  advisory `semantic.clause-language-unchecked`; `ocl` extracts with no
  advisory.
- A second clause with the same id SHALL fail with
  `semantic.duplicate-clause-id` at the second heading; a clause declared both
  by a fence and by a `Clause: ./<file>.md#<id>` line SHALL fail with
  `semantic.duplicate-clause-authority` at the second occurrence.
- Under `## Operations`, each `### <name>` heading SHALL own an optional
  parameter table with the header `Param | Type | Multiplicity | Constraints`
  (cells mapped by FR-070), an optional `Returns: <Type>[<mult>]` line, and
  `Pre: <clauseId>` and `Post: <clauseId>` lines naming clauses of the same
  artifact. A `Pre`/`Post` naming no clause fails with
  `semantic.dangling-clause-ref` at that line; a second operation with the
  same name fails with `semantic.duplicate-operation` at the second heading.
- Every produced `ClauseRef` and `OperationDecl` SHALL validate against the
  vendored `ClauseRef.json` and `OperationDecl.json`; a produced entry that
  does not is an engine defect and SHALL fail extraction.
- An artifact without `## Invariants` or `## Operations` SHALL yield
  `clauses`/`operations` as `not_applicable`, not as empty arrays that claim
  an authored absence.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-071-CON-1 | No code path SHALL tokenize, typecheck, or evaluate clause text; the only operations on a fence body are span recording and byte copy. | Architecture | Inspection |
| FR-071-CON-2 | Spans SHALL be derived from the parser's fence offsets, not from a second scan of the Markdown. | Integrity | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-071-AC-1 | The vendored `operations.md` extracts to the `clauses`, `operations`, and `clauseText` of `operations.expected.json`, and `config-version.table.md` extracts its `immutable` clause with the span recorded in `config-version.expected.json`; every `clauseText` value equals the fence body byte-for-byte. | Test |
| FR-071-AC-2 | The `fence-no-language`, `fence-bare-unknown`, `fence-sysml-advisory`, `fence-fretish-advisory`, and `fence-namespaced-advisory` cases of `operations-cases.json` yield the recorded code, severity, and fence locus; `ocl` yields no advisory. | Test |
| FR-071-AC-3 | `duplicate-clause-id` fails at the second heading, `clause-id-not-identifier` at the heading, and `inline-and-external` at the second occurrence, each with the recorded code. | Test |
| FR-071-AC-4 | `dangling-post` fails at the `Post:` line with `semantic.dangling-clause-ref` and `duplicate-operation` at the second heading with `semantic.duplicate-operation`. | Test |
| FR-071-AC-5 | Every produced `ClauseRef` and `OperationDecl` validates against the vendored semantic-core schemas, and an artifact without the sections reports both as `not_applicable`. | Test |
| FR-071-AC-6 | A static boundary test finds no clause tokenizer, parser, or evaluator symbol reachable from the semantic module, and a fence body containing arbitrary bytes round-trips unchanged into `clauseText`. | Inspection |

## Dependencies

- **Upstream**: [FR-070](./FR-070-typed-properties-extraction.md), [FR-005](./FR-005-markdown-parser.md); `agent-ix/quoin` FR-072
- **Downstream**: [FR-072](./FR-072-semantic-extraction-surface.md); `agent-ix/quire-contract-ir#55`
