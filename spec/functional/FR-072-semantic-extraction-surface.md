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
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-069"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-070"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-071"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-046"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-055"
    type: "requires"
    cardinality: "1:1"
---
# FR-072: Semantic extraction surface with explicit availability across Rust, Python, and WASM

## Description

The engine SHALL expose the FR-070/FR-071 extraction as one additive
`semantic` record on the existing surfaces: the library, `validate_document`,
the Filament extraction API, and its Python and WASM bindings.

The engine SHALL give every absence an explicit state and every diagnostic a
locus, so that a consumer distinguishes what was not declared from what could
not be read.

Disposition of the ticket's shared-contract deliverables: graph node and edge
records of [FR-045](./FR-045-filament-core-extraction-engine.md) are unchanged
and the semantic record attaches to the node; the assurance export of
[FR-067](./FR-067-versioned-assurance-export.md) carries the module-schema
digest tuple only, and a semantic projection into it is deferred to
`agent-ix/filament-core-data#36`; lossless byte preservation is met by the
verbatim `clauseText` and the byte-identity constraints below.

## Inputs

- A loaded `Registry` with `SemanticModule` records
  ([FR-069](./FR-069-semantic-module-contract-at-load.md)) or Filament
  snapshots with a `semantic` context and an optional `semanticBundle` index.
- One document with its corpus-relative path and `sourceIdentity`, or a
  bounded corpus ([FR-025](./FR-025-spec-corpus-model.md)) that supplies the
  `BundleIndex`.
- The semantic fixture suite `tests/fixtures/semantic/cases.json`: declarative
  cases, each with `name`, `issue_ref`, `tags`, `input`, and `expect`; the
  attribution test that governs `tests/fixtures/corpus_cases/` SHALL also
  govern this file.
- The pre-change baselines minted on `main` before the first code commit:
  `tests/fixtures/semantic/baseline/registry-archetypes.json`,
  `tests/fixtures/semantic/baseline/filament-graph-cases.json` (the current
  `graph_cases.json` outputs), and the existing coverage-v1, properties-v1,
  and assurance-v1 fixture outputs.

## Outputs

- Library: `extract_semantic(document, context) -> SemanticExtraction`, a
  `serde` value `{ formatVersion: 1, contractVersion, semanticCore, package,
  schemaDigest, fields?, fieldsForm?, clauses?, clauseText?, operations?,
  availability, diagnostics }`.
- Filament API: the same value under the key `semantic` inside each extracted
  node's `dataJson`, present only when the snapshot carries a `semantic`
  context; each semantic diagnostic also appended to
  `CoreExtractionResult.diagnostics` with its code, its severity mapped onto
  the existing set (`advisory` to `warning`, others unchanged), and an
  additive optional `locus` `{ path, line, column }`.
- `validate_document`: each semantic diagnostic as a finding with its locus;
  `advisory` maps to the existing warning level, `error` fails validation as
  any error finding does.
- `schemas/output/semantic-v1.schema.json`: the hand-authored contract for
  `SemanticExtraction`.

## Behavior

- `availability` SHALL record, per declaration kind (`fields`, `clauses`,
  `operations`), a `state` in `available`, `not_applicable` (block present,
  section absent), `missing` (the module's `body_extraction` marks the
  section `required` and it is absent), or `unavailable` with a non-empty
  `reason`, and a boolean `lossy` that is true when the kind carries an
  entry the engine did not interpret (an `unresolved` placeholder, an
  `unchecked` clause language, opaque brace text) or the module declares
  `compatibility_posture: declared-lossy`. The engine SHALL NOT collapse two
  states into one token.
- When a module carries no `semantic` block, or a snapshot carries no
  `semantic` context, the engine SHALL emit no `semantic` key and no
  semantic diagnostic; the record is byte-identical to the pre-change
  baseline.
- Diagnostics SHALL be ordered by `(line, column, code)` on every surface.
- If a Filament snapshot's `semantic.contractVersion` is not `1.0.0`, or its
  `semantic.semanticCore` has no vendored bundle, then the engine SHALL
  refuse the snapshot with the FR-069 code and yield no node for that object
  type.
- The Python binding SHALL return the record through the existing
  `extract_filament_core` and through a new `extract_semantic` function.
- The WASM binding (`agent-ix/quire-wasm#3`) SHALL expose `extractSemantic`
  beside `extractFilamentCore`. Bindings convert input and output only and
  carry no extraction policy ([FR-046](./FR-046-filament-extraction-bindings.md) AC-3).
- Repeated extraction over identical input SHALL produce byte-identical JSON,
  and the three surfaces SHALL agree as JSON values on every fixture case.
- The record SHALL validate against `semantic-v1.schema.json` (JSON Schema
  2020-12, `additionalProperties: false` on every object, `$id` ending in
  `semantic-v1.schema.json`); a breaking change SHALL mint `semantic-v2` and
  leave v1 byte-unchanged ([FR-055](./FR-055-published-output-contract.md)).

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-072-CON-1 | No existing key of `CoreExtractionResult`, `dataJson`, `validate_document` findings, coverage-v1, properties-v1, or assurance-v1 SHALL change meaning, order, value set, or bytes; every addition is a new optional key. | Compatibility | Test |
| FR-072-CON-2 | The surface SHALL render nothing, generate no code, and write no file; it returns values. | Responsibility | Test |
| FR-072-CON-3 | Maintainers SHALL hand-author the published `semantic-v1` schema, under the existing generator audit (`scripts/audits/check_no_schemars.sh`, TC-062). | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-072-AC-1 | Every case in `tests/fixtures/semantic/cases.json` yields the expected `fields`, `clauses`, `operations`, `availability`, and diagnostics (code, severity, locus, order) through the library, and the record validates against `semantic-v1.schema.json`. | Test |
| FR-072-AC-2 | Fixtures exercise `available`, `not_applicable`, `missing`, and `unavailable` for each declaration kind and both values of `lossy`; each state is a distinct JSON token and `unavailable` carries a non-empty `reason`. | Test |
| FR-072-AC-3 | A Filament snapshot with a `semantic` context yields `dataJson.semantic` and diagnostics with `locus` and mapped severity; the same document without the context yields a result equal to the checked-in baseline byte-for-byte. | Test |
| FR-072-AC-4 | A snapshot naming `contractVersion: 2.0.0` or `semanticCore: 0.9.0` is refused with the FR-069 code before any node is produced. | Test |
| FR-072-AC-5 | `validate_document` over the vendored corpus copy reports the legacy-form warning at its line, and over `both-forms.md` the both-forms error at the fence line, with the library's codes; the error fails validation. | Test |
| FR-072-AC-6 | The Python binding's `extract_filament_core` and `extract_semantic` return JSON values equal to the Rust output for every fixture case under `make ci-python`; the WASM leg is verified by `agent-ix/quire-wasm#3` against the same fixture file. | Test |
| FR-072-AC-7 | Two extractions over identical input serialize byte-identically; editing an unrelated section changes no declaration, span, or diagnostic. | Test |
| FR-072-AC-8 | `semantic-v1.schema.json` is valid 2020-12 with `additionalProperties: false` everywhere, and a checked-in compatibility fixture pins every field and state token; removing or renaming a pinned field fails the fixture test. | Test |
| FR-072-AC-9 | Coverage-v1, properties-v1, and assurance-v1 outputs equal their checked-in fixtures byte-for-byte, and `CoreExtractionDiagnostic.severity` values remain within `info`, `warning`, `error`. | Test |

## Dependencies

- **Upstream**: [FR-069](./FR-069-semantic-module-contract-at-load.md), [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-046](./FR-046-filament-extraction-bindings.md), [FR-032](./FR-032-validate-document.md), [FR-055](./FR-055-published-output-contract.md)
- **Downstream**: `agent-ix/filament-core-data#36`, `agent-ix/filament-parser-lib#8`, `agent-ix/quire-wasm#3`, `agent-ix/filament-core-service#23`
