---
id: FR-069
title: "Semantic module contract at load"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/it/semantic_contract.rs
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
archetype (any of `artifact_types`, `object_types`, `archetypes`) whose `data_schema` uses the `{ schema }` reference form,
the loader SHALL apply the semantic module contract (`agent-ix/quoin` FR-070
and FR-073; `agent-ix/filament-core-service` FR-035-AC-13..15).

When a manifest is outside that contract, the loader SHALL fail the module's
archetypes with a `semantic.*` reason instead of loading an empty or
partial model.

When a manifest carries no `semantic` block, the loader SHALL load it exactly
as it does today.

> **CR note (PLAT-237):** The prose previously said "object type" throughout. The loader has always resolved `semantic.exports` against every archetype (`manifest.all_archetypes()`), as FR-069-AC-3 states, so the wording was corrected to "archetype" to match the code. Behavior is unchanged.

> **CR-205 note (PLAT-1299):** `SemanticFailure::reason()` formatted only `<code>: <message>`, so a refusal dropped the manifest location its `path` recorded. The reason now carries it (`<code>: <path>: <message>`); `ArchetypeLoadFailure` gains no field. FR-069-AC-14 asserts one refusal of each kind.

## Inputs

- `manifest.yaml` and the files under the module root (filesystem loader,
  [FR-013](./FR-013-archetype-loader.md)), or the inline parts supplied to
  `Registry::from_inline_parts`, where the `schemas` map also supplies every
  reference-form `data_schema` file keyed by its manifest-relative path.
- The `semantic` block's shape (`contract_version`/`package`/
  `exports`/`imports`/`targets`/`mappings`, including the target enum), from
  `agent-ix/filament-core-data`'s published `module-semantic-block.schema.json`
  (`schema/semantic/v1/`); the `legacy_forms` and `compatibility_posture`
  value sets are this engine's own contract, not drawn from that schema.
- The embedded semantic-core JSON Schema bundle,
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
  `{ contractVersion, package, exports, imports, mappings? }`, with the
  data schema already inline (producer: `agent-ix/filament-core-service#23`).

## Outputs

- A `SemanticModule` record on the loaded module: `contract_version`,
  `package`, `exports`, `imports`, `targets`,
  `compatibility_posture`, `legacy_forms`, and `mappings` (the named
  representation mappings, whose model feature tokens gate
  [FR-075](./FR-075-model-feature-extraction.md) extraction). The admitted key
  `sweep_report` is a Quoin install-time key; the loader accepts and ignores
  it.
- Per archetype: the resolved data schema and a compiled validator over the
  extracted record.
- One `ArchetypeLoadFailure` per archetype of a refused module whose
  `reason` starts with the `semantic.*` code below, followed by the manifest
  path of the refusal (for example `object_types[entity].data_schema.schema`
  or `semantic.exports.ghost`) and the message, as
  `<code>: <path>: <message>`. A refusal with no manifest locus omits the
  path segment.
- Warning diagnostics for the advisory cases.

## Behavior

Refusals, in evaluation order:

- If `semantic.contract_version` is not `1.0.0`, then the loader SHALL refuse
  the module with `semantic.unsupported-contract-version` and read no other
  key of the block.
- A module declares no semantic-core version; the loader SHALL use the one
  embedded bundle and SHALL NOT refuse a module over a `semantic_core` value.
- The loader SHALL validate the block against the module-manifest
  schema. If the block carries an unknown key, then the loader SHALL refuse
  with `semantic.unknown-key` naming the key. If `exports` names an
  undeclared type in the manifest's `artifact_types`, `object_types`, or
  `archetypes` sections, then the loader SHALL refuse with
  `semantic.export-undeclared` naming it. If `package` is not `<org>/<repo>`,
  then the loader SHALL refuse with `semantic.invalid-package` naming the
  value. If a target is outside the target registry, then the loader refuses
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
  sibling files and the embedded semantic-core bundle; it SHALL NOT use filesystem or HTTP `$ref`
  resolution of the schema library, so the same path runs under the `wasm`
  feature. A semantic-core `$ref` resolves to the embedded bundle's file of
  that name whatever its version segment (`semantic.schema-ref-version` is
  removed); a `$ref` naming no shipped file with
  `semantic.schema-ref-unshipped`; a cycle in the reference graph with
  `semantic.schema-ref-cycle`. A `$ref` to the
  document's own `$id` is a fragment, not a cycle.
- The loader SHALL NOT rewrite, normalize, or dereference a referenced schema
  before validation.

Duplicate declaration authority, per module after its data schemas resolve:

- If a module names `relationships` in `semantic.mappings` and one of its
  object types declares in `body_extraction` a `table_row` locator whose
  match key equals a typed relation key of that object type, then the
  loader SHALL refuse the module with `semantic.duplicate-authority`.
  A typed relation key is a top-level `SemanticId[]` property of the type's
  resolved `data_schema` whose name is a verb in the type's `allowed_links`
  ([FR-076](./FR-076-relationships-extraction.md)). The trigger is the
  per-type locator under the module-level mapping token: a locator keyed
  `emits` on a type with no such key, or a module not naming `relationships`,
  is not refused.
- The failure shape is the existing `ArchetypeLoadFailure` per archetype of
  the refused module, with `reason` starting `semantic.duplicate-authority`
  followed by a message naming the module, the object type and the locator
  key.
- A match key is a locator's key in `yield_pattern.match` or
  `yield_pattern.per_match`, a fallback chain included, as
  [FR-075](./FR-075-model-feature-extraction.md) Inputs reads it.

> **CR-195 note (2026-11-08, agent-ix/quire-rs#435, PLAT-55):** "match key"
> was not defined here. It is the FR-075 term: a locator declared in either
> `yield_pattern.match` or `yield_pattern.per_match` declares its key, so a
> `per_match` locator keyed on a typed relation key is the same second
> declaration site and is refused the same way. Affected criteria:
> FR-069-AC-13.

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
  archetype, then the loader SHALL emit the warning
  `semantic.inline-data-schema`; without the block the loader SHALL emit
  nothing.

Filament extraction API:

- If a `FilamentObjectType` snapshot carries a reference-form `data_schema`,
  then the engine SHALL refuse the snapshot with
  `semantic.data-schema-unresolved-reference` and produce no node for that
  object type; the caller owning the registry resolves the schema.
- If a snapshot's `semantic.contractVersion` is
  unsupported, then the engine SHALL refuse the snapshot with the code above
  before any node is produced.

Allocation note: `agent-ix/quoin` FR-070 says Quire "applies the same
schema at artifact-validation time". This requirement applies it at
module load, so a broken block fails every archetype of the module rather
than every artifact; Quoin's install guard makes that unreachable for
installed modules, and hand-authored or inline modules fail loudly. The
divergence is recorded for Quoin to reconcile in wording; behavior is the
stricter of the two.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-069-CON-1 | The loader SHALL resolve schemas from the module bundle and the embedded bundle only, with no fetch of `https://schemas.agent-ix.org` and no read outside the module root. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-069-AC-1 | A module with a valid `semantic` block and a reference-form `data_schema` loads with a `SemanticModule` record and a resolved schema, and `validate_document` over an artifact of that type validates the extracted record against the resolved schema. | Test |
| FR-069-AC-2 | `contract_version: 2.0.0` fails every archetype of the module with a reason starting `semantic.unsupported-contract-version` and no other `semantic.*` reason. (The former `semantic_core` clause of this criterion is removed: a module declares no semantic-core version.) | Test |
| FR-069-AC-3 | An unknown block key, an export of a type undeclared across `artifact_types`, `object_types`, and `archetypes`, `package: ix://agent-ix/x`, and `targets: [go]` each fail with their named code and value; an export declared under `artifact_types` loads. | Test |
| FR-069-AC-4 | A missing file, a non-JSON file, a file without `$schema`, a `..` escape, and a symlink escape each fail with their named code, path, and reason; `{ schema, type }` fails with `semantic.data-schema-ambiguous`. | Test |
| FR-069-AC-5 | A `$ref` to semantic-core `0.2.0` resolves to the embedded bundle's file with no version refusal; a `$ref` to an unshipped sibling, an `https://` `$ref` outside both bundles, and a two-file `$ref` cycle each fail naming the `$ref`; a `$ref` to the schema's own `$id` fragment loads cleanly; the same cases pass under `--no-default-features --features wasm`. | Test |
| FR-069-AC-6 | An inline `data_schema` on a type under a `semantic` block loads with the warning `semantic.inline-data-schema`; the same manifest without the block loads with no semantic diagnostic. | Test |
| FR-069-AC-7 | A Filament snapshot whose `data_schema` is the reference form is refused with `semantic.data-schema-unresolved-reference` and yields no node; the same snapshot with the schema inline and a `semantic` context extracts. | Test |
| FR-069-AC-8 | The embedded semantic-core bundle is complete and made of valid JSON Schema documents, sourced from the published `@agent-ix/semantic-core` package. | Test |
| FR-069-AC-10 | Two loaded modules with one `semantic.package` fail the later sorted root with `semantic.duplicate-package` naming both; an import no loaded module provides warns `semantic.import-unresolved` and still loads; a two-module import cycle fails both with `semantic.import-cycle`. | Test |
| FR-069-AC-11 | `Registry::from_inline_parts` with a reference-form `data_schema` resolves the file from the `schemas` map, applies the same escape and `$ref` rules, and refuses a key with a `..` segment. | Test |
| FR-069-AC-12 | A module whose `semantic` block lists `mappings` loads with those tokens recorded on its `SemanticModule`, in order; a block without `mappings` records an empty list. | Test |
| FR-069-AC-13 | A module naming `relationships` whose `process` type declares a `table_row` locator keyed `emits` (a typed relation key of its `data_schema`), under `yield_pattern.match` or `yield_pattern.per_match`, is refused: every archetype of the module fails with a reason starting `semantic.duplicate-authority` and naming the module, `process` and `emits`; the same module with the locator removed loads; a locator keyed `emits` on a type whose schema has no `emits`, and the same locator in a module not naming `relationships`, load. | Test |
| FR-069-AC-14 | Every `semantic.*` refusal reason has the form `<code>: <path>: <message>`, where `<path>` is the refusal's manifest locus and appears once (the message does not repeat it), or `<code>: <message>` when the refusal has no manifest locus (empty path). Covered kinds: an unknown block key names `semantic`, an undeclared export names `semantic.exports.<type>`, a mixed `data_schema` and a reference-form `data_schema` without a `semantic` block name `<section>[<type>].data_schema`, a missing schema file names `<section>[<type>].data_schema.schema`, and `semantic.duplicate-authority` names `<section>[<type>].body_extraction`. | Test |

## Dependencies

- **Upstream**: [FR-013](./FR-013-archetype-loader.md), [FR-076](./FR-076-relationships-extraction.md) (typed relation keys), [FR-045](./FR-045-filament-core-extraction-engine.md), [FR-067](./FR-067-versioned-assurance-export.md); `agent-ix/quoin` FR-070/FR-073/FR-075; `agent-ix/filament-core-service` FR-035-AC-13..15, `#23`
- **Downstream**: [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md)
