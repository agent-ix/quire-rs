---
id: FR-069
title: "Semantic module contract at load"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/semantic_contract.rs
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-019"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-045"
    type: "requires"
    cardinality: "1:1"
---
# FR-069: Semantic module contract at load

## Description

When a module manifest carries a `semantic` block, or an object type's
`data_schema` uses the `{ schema, digest }` reference form, the loader SHALL
read both under the semantic module contract (`agent-ix/quoin` FR-070 and
FR-073, `agent-ix/filament-core-service` FR-035 CR-003) and SHALL refuse an
unsupported version, digest, or reference explicitly instead of loading an
empty or partial model.

A manifest without a `semantic` block loads exactly as it does today.

## Inputs

- `manifest.yaml` and the files under the module root (filesystem loader,
  [FR-013](./FR-013-archetype-loader.md)) or the inline parts supplied to
  `Registry::from_inline_parts`.
- The vendored `module-manifest.schema.json` (`agent-ix/filament-core-service`
  at revision `a77f31e`, path `filament_core_service/schemas/module-manifest.schema.json`).
- The vendored semantic-core JSON Schema bundle for each supported version
  (`agent-ix/filament-core-data` `packages/semantic-core/schemas/`, version
  `0.1.0` at revision `d48b8da`, digest recorded from its `toolchain.json`).
- For the Filament extraction API ([FR-045](./FR-045-filament-core-extraction-engine.md)):
  an optional `semantic` context on each `FilamentObjectType` snapshot carrying
  `contractVersion`, `semanticCore`, `package`, `exports`, and `imports`, with
  the resolved data schema inline.

## Outputs

- A `SemanticModule` record on the loaded module: `contract_version`,
  `semantic_core`, `package`, `exports`, `imports`, `targets`,
  `compatibility_posture`, `legacy_forms`.
- Per object type: the resolved data schema, its lowercase SHA-256 digest, and
  a compiled validator over the extracted record.
- `ArchetypeLoadFailure` entries with a `semantic.*` reason code for every
  refusal below; warning diagnostics for the advisory cases.

## Behavior

- If `semantic.contract_version` is not `1.0.0`, the loader SHALL fail the
  module with `semantic.unsupported-contract-version` before reading any other
  key of the block.
- If `semantic.semantic_core` names a version with no vendored bundle, the
  loader SHALL fail the module with `semantic.unsupported-semantic-core`,
  naming the requested and the vendored versions.
- The block SHALL be validated against the vendored module-manifest schema; an
  unknown key, an export naming an undeclared object type, a `package` that is
  not `<org>/<repo>`, or a target outside the vendored target registry SHALL
  each fail the module naming the key or value.
- A reference-form `data_schema` SHALL resolve to a file inside the module
  root whose bytes hash to `digest`, that parses as JSON, and that declares
  `$schema` `https://json-schema.org/draft/2020-12/schema` and
  `$id` `https://schemas.agent-ix.org/<package>/<module version>/<file>`; each
  failure SHALL name the path and the reason (`semantic.data-schema-missing`,
  `-digest-mismatch`, `-not-json`, `-not-schema`, `-id`, `-escape`).
- Every `$ref` in a referenced schema SHALL resolve offline to a sibling file
  in the module bundle or to the vendored semantic-core bundle at the
  manifest's `semantic_core` version; a `$ref` to another semantic-core
  version fails with `semantic.schema-ref-version`, a `$ref` to no shipped
  file with `semantic.schema-ref-unshipped`, and a reference cycle with
  `semantic.schema-ref-cycle`. A `$ref` to the document's own `$id` is a
  fragment, not a cycle.
- An inline `data_schema` under a module with a `semantic` block SHALL load
  and emit the warning `semantic.inline-data-schema`; without a block it is
  silent.
- The `{ schema, digest, type }` mixed form SHALL fail as ambiguous.
- In the Filament extraction API a snapshot carrying the reference form SHALL
  fail with `semantic.data-schema-unresolved-reference`: the caller owning the
  registry resolves the schema (`agent-ix/filament-core-service` FR-035),
  Quire never reads a module directory on that path.
- The `(package, object type, schema digest)` tuple recorded here SHALL be the
  one the assurance export lists as the active module-schema tuple
  ([FR-067](./FR-067-versioned-assurance-export.md) AC-3); no second digest is
  computed.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-069-CON-1 | Schema resolution SHALL read only the module bundle and the vendored semantic-core bundle; no `https://schemas.agent-ix.org` fetch and no filesystem read outside the module root. | Architecture | Test |
| FR-069-CON-2 | Each vendored schema file SHALL carry provenance (repository, revision, path, SHA-256) in source, and a test SHALL fail when the vendored bytes and the recorded digest disagree. | Integrity | Test |
| FR-069-CON-3 | A module without a `semantic` block SHALL produce a byte-identical `Registry` result before and after this change; no new required manifest key is introduced. | Compatibility | Test |
| FR-069-CON-4 | The loader SHALL NOT rewrite, normalize, or dereference a referenced schema before validation; the digest is over the shipped bytes. | Integrity | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-069-AC-1 | A module with a valid `semantic` block and a reference-form `data_schema` whose digest matches loads with a `SemanticModule` record, a resolved schema, and the recorded digest; `quire validate` over an artifact of that type validates its extracted record against the resolved schema. | Test |
| FR-069-AC-2 | `contract_version: 2.0.0` fails the module with `semantic.unsupported-contract-version` and no other `semantic.*` diagnostic; `semantic_core: 0.9.0` fails with `semantic.unsupported-semantic-core` naming `0.9.0` and `0.1.0`. | Test |
| FR-069-AC-3 | An unknown block key, an export of an undeclared object type, `package: ix://agent-ix/x`, and `targets: [go]` each fail the module naming the key or value. | Test |
| FR-069-AC-4 | A digest mismatch, a missing file, a non-JSON file, a file without `$schema`, a wrong `$id`, and a `..` or symlink escape each fail naming the path and reason; the mixed `{ schema, digest, type }` form fails as ambiguous. | Test |
| FR-069-AC-5 | A `$ref` to semantic-core `0.2.0` under `semantic_core: 0.1.0`, a `$ref` to an unshipped sibling, and a two-file `$ref` cycle each fail naming the `$ref`; a `$ref` to the schema's own `$id` fragment loads cleanly. | Test |
| FR-069-AC-6 | An inline `data_schema` under a `semantic` block loads with the warning `semantic.inline-data-schema`; the same manifest without the block loads with no semantic diagnostic. | Test |
| FR-069-AC-7 | A Filament snapshot whose `data_schema` is the reference form is refused with `semantic.data-schema-unresolved-reference`; the same snapshot with the schema inline and a `semantic` context extracts. | Test |
| FR-069-AC-8 | The vendored module-manifest schema and semantic-core bundle hash to the recorded provenance digests, and the semantic-core bundle digest equals the `toolchain.json` digest of `agent-ix/filament-core-data` at the recorded revision. | Test |
| FR-069-AC-9 | Every default and fixture module without a `semantic` block loads to a `Registry` whose serialized archetype set is byte-identical to the pre-change baseline. | Test |

## Dependencies

- **Upstream**: [FR-013](./FR-013-archetype-loader.md), [FR-045](./FR-045-filament-core-extraction-engine.md), [FR-067](./FR-067-versioned-assurance-export.md); `agent-ix/quoin` FR-070/FR-073; `agent-ix/filament-core-service` FR-035
- **Downstream**: [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md)
