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
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-067"
    type: "requires"
    cardinality: "1:1"
---
# FR-069: Semantic module contract at load

## Description

When the loader reads a module manifest that carries a `semantic` block, or an
object type whose `data_schema` uses the `{ schema, digest }` reference form,
the loader SHALL apply the semantic module contract (`agent-ix/quoin` FR-070
and FR-073; `agent-ix/filament-core-service` FR-035-AC-13..15).

When a manifest is outside that contract, the loader SHALL fail the module's
object types with a `semantic.*` reason instead of loading an empty or
partial model.

When a manifest carries no `semantic` block, the loader SHALL load it exactly
as it does today.

## Inputs

- `manifest.yaml` and the files under the module root (filesystem loader,
  [FR-013](./FR-013-archetype-loader.md)), or the inline parts supplied to
  `Registry::from_inline_parts`, where the `schemas` map also supplies every
  reference-form `data_schema` file keyed by its manifest-relative path.
- Vendored inputs, all under one directory `schemas/vendored/` with a
  provenance record `(repository, revision, path, sha256)` per file:
  - `module-manifest.schema.json` from `agent-ix/filament-core-service` at
    `a77f31e`, path `filament_core_service/schemas/module-manifest.schema.json`
    (source of the `semantic` block shape and the `legacy_forms` and
    `compatibility_posture` value sets).
  - The semantic-core JSON Schema bundle, one directory per supported
    version, from `agent-ix/filament-core-data` at `d48b8da`, path
    `packages/semantic-core/generated/json-schema/`, with its
    `packages/semantic-core/generated/toolchain.json` digest
    (`sha256:dd33c886f70e908b14507c35e078d163b76308c3d170d2b54ddf933d1a4ebb52`
    for `0.1.0`) copied into the provenance record.
  - The target registry `schema/semantic/v1/common.schema.json` from
    `agent-ix/filament-core-data` at `d48b8da` (`target` and
    `representationFormat` values).
- For the Filament extraction API ([FR-045](./FR-045-filament-core-extraction-engine.md)):
  an optional `semantic` context on each `FilamentObjectType` snapshot,
  `{ contractVersion, semanticCore, package, exports, imports }`, with the
  data schema already inline (producer: `agent-ix/filament-core-service#23`).

## Outputs

- A `SemanticModule` record on the loaded module: `contract_version`,
  `semantic_core`, `package`, `exports`, `imports`, `targets`,
  `compatibility_posture`, `legacy_forms`. The admitted keys `mappings` and
  `sweep_report` are Quoin install-time keys; the loader accepts and ignores
  them.
- Per object type: the resolved data schema, its lowercase SHA-256 digest
  over the shipped bytes, and a compiled validator over the extracted record.
  This `(module, archetype, schema_digest)` tuple is the one
  [FR-067](./FR-067-versioned-assurance-export.md) AC-3 lists; no second
  digest is computed.
- One `ArchetypeLoadFailure` per object type of a refused module whose
  `reason` starts with the `semantic.*` code below, followed by the message.
- Warning diagnostics for the advisory cases.

## Behavior

Refusals, in evaluation order:

- If `semantic.contract_version` is not `1.0.0`, then the loader SHALL refuse
  the module with `semantic.unsupported-contract-version` and read no other
  key of the block.
- If `semantic.semantic_core` names a version with no vendored bundle, then
  the loader SHALL refuse the module with `semantic.unsupported-semantic-core`,
  naming the requested and the vendored versions.
- The loader SHALL validate the block against the vendored module-manifest
  schema. If the block carries an unknown key, then the loader SHALL refuse
  with `semantic.unknown-key` naming the key. If `exports` names an
  undeclared object type, then the loader SHALL refuse with
  `semantic.export-undeclared` naming it. If `package` is not `<org>/<repo>`,
  then the loader SHALL refuse with `semantic.invalid-package` naming the
  value. If a target is outside the vendored target registry, then the loader refuses
  with `semantic.unknown-target` naming the value.
- If an exported object type's `data_schema` is not the reference form, then
  the loader SHALL refuse with `semantic.export-without-schema` naming the
  type.
- For a reference-form `data_schema`, the loader SHALL resolve `schema`
  inside the module root (a `..` segment or a symlink leaving the root is
  `semantic.data-schema-escape`), read the bytes (absent:
  `semantic.data-schema-missing`), compare their SHA-256 with `digest`
  (`semantic.data-schema-digest-mismatch`), parse JSON
  (`semantic.data-schema-not-json`), require `$schema`
  `https://json-schema.org/draft/2020-12/schema`
  (`semantic.data-schema-not-schema`), and require `$id`
  `https://schemas.agent-ix.org/<package>/<module version>/<file>`
  (`semantic.data-schema-id`). Each refusal SHALL name the path and the
  reason. The mixed form `{ schema, digest, type }` is
  `semantic.data-schema-ambiguous`; the reference form on a manifest without
  a `semantic` block is `semantic.data-schema-reference-without-block`.
- The loader SHALL resolve every `$ref` of a referenced schema offline against
  an in-memory map of `$id` to document built from the module bundle's
  sibling files and the vendored semantic-core bundle at the manifest's
  `semantic_core` version; it SHALL NOT use filesystem or HTTP `$ref`
  resolution of the schema library, so the same path runs under the `wasm`
  feature. A `$ref` naming another semantic-core version is refused with
  `semantic.schema-ref-version`; a `$ref` naming no shipped file with
  `semantic.schema-ref-unshipped`; a cycle in the reference graph with
  `semantic.schema-ref-cycle`. A `$ref` to the
  document's own `$id` is a fragment, not a cycle.
- The loader SHALL NOT rewrite, normalize, or dereference a referenced schema
  before validation.

Cross-module checks, after every module of a load has been read, in sorted
module-root order:

- If two loaded modules declare one `semantic.package`, then the loader SHALL
  refuse the later module with `semantic.duplicate-package` naming both.
- If a `semantic.imports` entry names a package that no loaded module provides
  at that exact version, then the loader SHALL emit the warning
  `semantic.import-unresolved` naming the package and the loaded versions;
  the module loads and its tokens from that import resolve as
  `unresolved` with reason `import-unresolved` ([FR-070](./FR-070-typed-properties-extraction.md)).
- If the import graph has a cycle, then the loader SHALL refuse every module
  on the cycle with `semantic.import-cycle` naming the cycle.

Advisory cases:

- If a module with a `semantic` block declares an inline `data_schema` on a
  non-exported object type, then the loader SHALL emit the warning
  `semantic.inline-data-schema`; without the block the loader SHALL emit
  nothing.

Filament extraction API:

- If a `FilamentObjectType` snapshot carries a reference-form `data_schema`,
  then the engine SHALL refuse the snapshot with
  `semantic.data-schema-unresolved-reference` and produce no node for that
  object type; the caller owning the registry resolves the schema.
- If a snapshot's `semantic.contractVersion` or `semantic.semanticCore` is
  unsupported, then the engine SHALL refuse the snapshot with the code above
  before any node is produced.

Allocation note: `agent-ix/quoin` FR-070 says Quire "applies the same
vendored schema at artifact-validation time". This requirement applies it at
module load, so a broken block fails every object type of the module rather
than every artifact; Quoin's install guard makes that unreachable for
installed modules, and hand-authored or inline modules fail loudly. The
divergence is recorded for Quoin to reconcile in wording; behavior is the
stricter of the two.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-069-CON-1 | The loader SHALL resolve schemas from the module bundle and the vendored bundle only, with no fetch of `https://schemas.agent-ix.org` and no read outside the module root. | Architecture | Test |
| FR-069-CON-2 | Each vendored file SHALL carry a provenance record in source whose SHA-256 a test compares with the vendored bytes. | Integrity | Test |
| FR-069-CON-3 | A module without a `semantic` block SHALL produce a `Registry` whose archetype projection (name, schema digest, `body_extraction` JSON, extras) equals the checked-in baseline `tests/fixtures/semantic/baseline/registry-archetypes.json` minted on `main` before this change. | Compatibility | Test |
| FR-069-CON-4 | The digest recorded for an object type SHALL be over the shipped file bytes, computed once at load. | Integrity | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-069-AC-1 | A module with a valid `semantic` block and a reference-form `data_schema` whose digest matches loads with a `SemanticModule` record, a resolved schema, and the recorded digest, and `validate_document` over an artifact of that type validates the extracted record against the resolved schema. | Test |
| FR-069-AC-2 | `contract_version: 2.0.0` fails every object type of the module with a reason starting `semantic.unsupported-contract-version` and no other `semantic.*` reason; `semantic_core: 0.9.0` fails with `semantic.unsupported-semantic-core` naming `0.9.0` and `0.1.0`. | Test |
| FR-069-AC-3 | An unknown block key, an export of an undeclared object type, `package: ix://agent-ix/x`, `targets: [go]`, and an export whose `data_schema` is inline each fail with their named code and value. | Test |
| FR-069-AC-4 | A digest mismatch, a missing file, a non-JSON file, a file without `$schema`, a wrong `$id`, a `..` escape, and a symlink escape each fail with their named code, path, and reason; `{ schema, digest, type }` fails with `semantic.data-schema-ambiguous`. | Test |
| FR-069-AC-5 | A `$ref` to semantic-core `0.2.0` under `semantic_core: 0.1.0`, a `$ref` to an unshipped sibling, an `https://` `$ref` outside both bundles, and a two-file `$ref` cycle each fail naming the `$ref`; a `$ref` to the schema's own `$id` fragment loads cleanly; the same cases pass under `--no-default-features --features wasm`. | Test |
| FR-069-AC-6 | An inline `data_schema` on a non-exported type under a `semantic` block loads with the warning `semantic.inline-data-schema`; the same manifest without the block loads with no semantic diagnostic. | Test |
| FR-069-AC-7 | A Filament snapshot whose `data_schema` is the reference form is refused with `semantic.data-schema-unresolved-reference` and yields no node; the same snapshot with the schema inline and a `semantic` context extracts. | Test |
| FR-069-AC-8 | Every vendored file hashes to its recorded provenance SHA-256, and the semantic-core `0.1.0` provenance digest equals `sha256:dd33c886f70e908b14507c35e078d163b76308c3d170d2b54ddf933d1a4ebb52`. | Test |
| FR-069-AC-9 | Every default and fixture module without a `semantic` block loads to the archetype projection recorded in the checked-in baseline. | Test |
| FR-069-AC-10 | Two loaded modules with one `semantic.package` fail the later sorted root with `semantic.duplicate-package` naming both; an import no loaded module provides warns `semantic.import-unresolved` and still loads; a two-module import cycle fails both with `semantic.import-cycle`. | Test |
| FR-069-AC-11 | `Registry::from_inline_parts` with a reference-form `data_schema` resolves the file from the `schemas` map, applies the same digest, `$id`, escape, and `$ref` rules, and refuses a key with a `..` segment. | Test |

## Dependencies

- **Upstream**: [FR-013](./FR-013-archetype-loader.md), [FR-045](./FR-045-filament-core-extraction-engine.md), [FR-067](./FR-067-versioned-assurance-export.md); `agent-ix/quoin` FR-070/FR-073/FR-075; `agent-ix/filament-core-service` FR-035-AC-13..15, `#23`
- **Downstream**: [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md)
