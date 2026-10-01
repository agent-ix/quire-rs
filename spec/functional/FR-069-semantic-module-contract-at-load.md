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
object type whose `data_schema` uses the `{ schema }` reference form,
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
- The `semantic` block's shape (`contract_version`/`semantic_core`/`package`/
  `exports`/`imports`/`targets`/`mappings`, including the target enum), from
  `agent-ix/filament-core-data`'s published `module-semantic-block.schema.json`
  (`schema/semantic/v1/`); the `legacy_forms` and `compatibility_posture`
  value sets are this engine's own contract, not drawn from that schema.
- The semantic-core JSON Schema bundle, one directory per supported version,
  sourced from `agent-ix-semantic-schema` — a plain Cargo `git` dependency on
  `agent-ix/filament-core-data` (PLAT-948) — never a copy committed to this
  repository.
- The module-manifest schema (`MODULE_MANIFEST_SCHEMA`, whose
  `properties.semantic` is what `block_validator()` compiles), sourced from
  that same `agent-ix-semantic-schema` dependency's
  `semantic/v1/module-manifest.schema.json` — never a copy committed to this
  repository.

- For the Filament extraction API ([FR-045](./FR-045-filament-core-extraction-engine.md)):
  an optional `semantic` context on each `FilamentObjectType` snapshot,
  `{ contractVersion, semanticCore, package, exports, imports, mappings? }`, with the
  data schema already inline (producer: `agent-ix/filament-core-service#23`).

## Outputs

- A `SemanticModule` record on the loaded module: `contract_version`,
  `semantic_core`, `package`, `exports`, `imports`, `targets`,
  `compatibility_posture`, `legacy_forms`, and `mappings` (the named
  representation mappings, whose model feature tokens gate
  [FR-075](./FR-075-model-feature-extraction.md) extraction). The admitted key
  `sweep_report` is a Quoin install-time key; the loader accepts and ignores
  it.
- Per object type: the resolved data schema and a compiled validator over the
  extracted record.
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
- For a reference-form `data_schema`, the loader SHALL resolve `schema`
  inside the module root (a `..` segment or a symlink leaving the root is
  `semantic.data-schema-escape`), read the bytes (absent:
  `semantic.data-schema-missing`), parse JSON
  (`semantic.data-schema-not-json`), and require `$schema`
  `https://json-schema.org/draft/2020-12/schema`
  (`semantic.data-schema-not-schema`). Each refusal SHALL name the path and
  the reason. The mixed form `{ schema, type }` is
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

- If a module with a `semantic` block declares an inline `data_schema` on an
  object type, then the loader SHALL emit the warning
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
| FR-069-CON-1 | The loader SHALL resolve schemas from the module bundle and the embedded bundle only, with no fetch of `https://schemas.agent-ix.org` and no read outside the module root. | Architecture | Test |
| FR-069-CON-3 | A module without a `semantic` block SHALL produce a `Registry` whose archetype projection (name, `body_extraction` JSON, extras) equals the checked-in baseline `tests/fixtures/semantic/baseline/registry-archetypes.json` minted on `main` before this change. | Compatibility | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-069-AC-1 | A module with a valid `semantic` block and a reference-form `data_schema` loads with a `SemanticModule` record and a resolved schema, and `validate_document` over an artifact of that type validates the extracted record against the resolved schema. | Test |
| FR-069-AC-2 | `contract_version: 2.0.0` fails every object type of the module with a reason starting `semantic.unsupported-contract-version` and no other `semantic.*` reason; `semantic_core: 0.9.0` fails with `semantic.unsupported-semantic-core` naming `0.9.0` and `0.3.0`. | Test |
| FR-069-AC-3 | An unknown block key, an export of an undeclared object type, `package: ix://agent-ix/x`, and `targets: [go]` each fail with their named code and value. | Test |
| FR-069-AC-4 | A missing file, a non-JSON file, a file without `$schema`, a `..` escape, and a symlink escape each fail with their named code, path, and reason; `{ schema, type }` fails with `semantic.data-schema-ambiguous`. | Test |
| FR-069-AC-5 | A `$ref` to semantic-core `0.2.0` under `semantic_core: 0.3.0`, a `$ref` to an unshipped sibling, an `https://` `$ref` outside both bundles, and a two-file `$ref` cycle each fail naming the `$ref`; a `$ref` to the schema's own `$id` fragment loads cleanly; the same cases pass under `--no-default-features --features wasm`. | Test |
| FR-069-AC-6 | An inline `data_schema` on a type under a `semantic` block loads with the warning `semantic.inline-data-schema`; the same manifest without the block loads with no semantic diagnostic. | Test |
| FR-069-AC-7 | A Filament snapshot whose `data_schema` is the reference form is refused with `semantic.data-schema-unresolved-reference` and yields no node; the same snapshot with the schema inline and a `semantic` context extracts. | Test |
| FR-069-AC-8 | Every semantic-core version a `semantic` block may declare is a complete embedded bundle of valid JSON Schema documents, sourced from the published `@agent-ix/semantic-core` package at that exact version. | Test |
| FR-069-AC-9 | Every default and fixture module without a `semantic` block loads to the archetype projection recorded in the checked-in baseline. | Test |
| FR-069-AC-10 | Two loaded modules with one `semantic.package` fail the later sorted root with `semantic.duplicate-package` naming both; an import no loaded module provides warns `semantic.import-unresolved` and still loads; a two-module import cycle fails both with `semantic.import-cycle`. | Test |
| FR-069-AC-11 | `Registry::from_inline_parts` with a reference-form `data_schema` resolves the file from the `schemas` map, applies the same escape and `$ref` rules, and refuses a key with a `..` segment. | Test |
| FR-069-AC-12 | A module whose `semantic` block lists `mappings` loads with those tokens recorded on its `SemanticModule`, in order; a block without `mappings` records an empty list. | Test |

## Dependencies

- **Upstream**: [FR-013](./FR-013-archetype-loader.md), [FR-045](./FR-045-filament-core-extraction-engine.md), [FR-067](./FR-067-versioned-assurance-export.md); `agent-ix/quoin` FR-070/FR-073/FR-075; `agent-ix/filament-core-service` FR-035-AC-13..15, `#23`
- **Downstream**: [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md)
