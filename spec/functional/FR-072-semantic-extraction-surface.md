---
id: FR-072
title: "Semantic extraction surface with explicit availability across Rust, Python, and WASM"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/semantic_surface.rs
  - kind: test_case
    ref: tests/fixtures/semantic/cases.json
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-019"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-046"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
---
# FR-072: Semantic extraction surface with explicit availability across Rust, Python, and WASM

## Description

The engine SHALL expose the FR-070/FR-071 extraction as one additive
`semantic` record on the existing surfaces (library, `validate_document`,
the Filament extraction API and its Python and WASM bindings), with every
absence carrying an explicit state and every diagnostic a locus, so that a
consumer distinguishes what was not declared from what could not be read.

## Inputs

- A loaded `Registry` with `SemanticModule` records
  ([FR-069](./FR-069-semantic-module-contract-at-load.md)) or Filament
  snapshots with a `semantic` context.
- One document, or a bounded corpus ([FR-025](./FR-025-spec-corpus-model.md))
  for bundle-wide type resolution.
- The semantic fixture suite `tests/fixtures/semantic/cases.json`, declarative
  cases in the `corpus_cases` shape with `issue_ref`, built from the vendored
  `agent-ix/quoin` mapping fixtures.

## Outputs

- Library: `extract_semantic(document, context) -> SemanticExtraction`, a
  `serde` value with `contractVersion`, `semanticCore`, `package`,
  `schemaDigest`, `fields`, `fieldsForm`, `clauses`, `clauseText`,
  `operations`, `availability`, and `diagnostics`.
- Filament API: the same value under the key `semantic` inside each extracted
  node's `dataJson`, and each semantic diagnostic appended to
  `CoreExtractionResult.diagnostics` with an additive `locus`
  `{ path, line, column }` field.
- `validate_document` / `quire validate`: each semantic diagnostic as a
  finding with its locus and severity.

## Behavior

- `availability` SHALL record one state per declaration kind (`fields`,
  `clauses`, `operations`): `available`, `not_applicable` (no `semantic`
  block, or no section), `missing` (a section the module's `body_extraction`
  marks `required` is absent), or `unavailable` with a non-empty `reason`
  (legacy form, both forms, extraction error). Per-entry states stay on the
  entry: an unresolved type is the FR-070 placeholder plus its advisory; an
  unchecked clause language is the FR-071 advisory. The engine SHALL NOT
  collapse two states into one token.
- Severities SHALL be `advisory`, `warning`, or `error`. Inside
  `CoreExtractionResult.diagnostics` the string `advisory` is emitted as-is,
  an additive value beside the existing `info`, `warning`, `error`.
- A Filament snapshot whose `semantic.contractVersion` is not `1.0.0`, or
  whose `semantic.semanticCore` has no vendored bundle, SHALL be refused with
  the FR-069 code and yield no node for that object type; snapshots without a
  `semantic` context extract exactly as before.
- The Python binding SHALL return the same value through the existing
  `extract_filament_core` and a new `extract_semantic` function; the WASM
  binding (`agent-ix/quire-wasm`) SHALL expose `extractSemantic` beside
  `extractFilamentCore`. Bindings convert input and output only and carry no
  extraction policy ([FR-046](./FR-046-filament-extraction-bindings.md) AC-3).
- Repeated extraction over identical input SHALL produce byte-identical JSON,
  and the three surfaces SHALL agree as JSON values on every fixture case.
- The record SHALL validate against a published `schemas/output/semantic-v1.schema.json`
  (hand-authored, `additionalProperties: false`, `$id` ending in
  `semantic-v1.schema.json`), whose `format_version` is `1`; a breaking change
  mints `semantic-v2` and leaves v1 byte-unchanged.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-072-CON-1 | No existing key of `CoreExtractionResult`, `dataJson`, `validate_document` findings, coverage-v1, properties-v1, or assurance-v1 SHALL change meaning, order, or bytes; every addition is a new optional key. | Compatibility | Test |
| FR-072-CON-2 | The surface SHALL render nothing, generate no code, and write no file; it returns values. | Responsibility | Inspection |
| FR-072-CON-3 | The published `semantic-v1` schema SHALL be hand-authored; a schema generator is prohibited from defining the contract. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-072-AC-1 | Every case in `tests/fixtures/semantic/cases.json` yields the expected `fields`, `clauses`, `operations`, `availability`, and diagnostics (code, severity, locus) through the library, and the record validates against `semantic-v1.schema.json`. | Test |
| FR-072-AC-2 | Fixtures cover `available`, `not_applicable`, `missing`, and `unavailable` for each declaration kind, and each state is a distinct JSON token with `unavailable` carrying a non-empty `reason`. | Test |
| FR-072-AC-3 | A Filament snapshot with a `semantic` context yields the record under `dataJson.semantic` and its diagnostics in `CoreExtractionResult.diagnostics` with `locus`; the same document without the context yields a result byte-identical to the pre-change extraction. | Test |
| FR-072-AC-4 | A snapshot naming `contractVersion: 2.0.0` or `semanticCore: 0.9.0` is refused with the FR-069 code before any node is produced. | Test |
| FR-072-AC-5 | `quire validate` over the vendored corpus copy reports the legacy-form warning at its line, and over `both-forms.md` reports the both-forms error at the fence line, with the same codes the library emits. | Test |
| FR-072-AC-6 | The Python binding's `extract_filament_core` and `extract_semantic`, and the WASM `extractFilamentCore` and `extractSemantic`, return JSON values equal to the Rust output for every fixture case. | Test |
| FR-072-AC-7 | Two extractions over identical input serialize byte-identically, and changing an unrelated section leaves every declaration, span, and diagnostic unchanged. | Test |
| FR-072-AC-8 | The published `semantic-v1.schema.json` is valid JSON Schema 2020-12 with `additionalProperties: false` on every object, and a checked-in compatibility fixture pins every field and state token; removing or renaming a pinned field fails the fixture test. | Test |

## Dependencies

- **Upstream**: [FR-069](./FR-069-semantic-module-contract-at-load.md), [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-046](./FR-046-filament-extraction-bindings.md), [FR-032](./FR-032-validate-document.md), [FR-055](./FR-055-published-output-contract.md)
- **Downstream**: `agent-ix/filament-core-data#36`, `agent-ix/filament-parser-lib#8`
