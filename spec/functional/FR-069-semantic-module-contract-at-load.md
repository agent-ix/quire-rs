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
  `exports`/`imports`/`import_types`/`targets`/`mappings`, including the target
  enum), from
  `agent-ix/filament-core-data`'s published `module-semantic-block.schema.json`
  (`schema/semantic/v1/`); the `legacy_forms` and `compatibility_posture`
  value sets are this engine's own contract, not drawn from that schema. The
  `import_types` key is added to that schema by the upstream change named in
  Dependencies; until that release the loader's block validation refuses it
  as `semantic.unknown-key`.
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
  `{ contractVersion, package, exports, imports, importTypes?, mappings? }`, with the
  data schema already inline (producer: `agent-ix/filament-core-service#23`).

## Outputs

- A `SemanticModule` record on the loaded module: `contract_version`,
  `package`, `exports`, `imports`, `import_types`
  (empty when absent), `targets`, `compatibility_posture`, `legacy_forms`, and `mappings` (the named
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
  embedded bundle.
- The loader SHALL validate the block against the module-manifest
  schema. If the block carries an unknown key, then the loader SHALL refuse
  with `semantic.unknown-key` naming the key. If `exports` names an
  undeclared type in the manifest's `artifact_types`, `object_types`, or
  `archetypes` sections, then the loader SHALL refuse with
  `semantic.export-undeclared` naming it. If `package` is not `<org>/<repo>`,
  then the loader SHALL refuse with `semantic.invalid-package` naming the
  value. If a target is outside the target registry, then the loader refuses
  with `semantic.unknown-target` naming the value.
  If `semantic.import_types` is not an object, a key is absent from
  `semantic.imports`, or a value is not an array, is empty, holds a non-string
  entry, an empty string, or a duplicate type name, then the loader SHALL
  refuse with `semantic.invalid-value`. The location is the unescaped package
  key with no index, `semantic.import_types.<org>/<repo>` (or
  `semantic.import_types` for a non-object); it is not the JSON-pointer form
  the schema-error mapping would build from the schema path, so the loader
  maps these refusals specially.
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
- The resolver SHALL admit a `$ref` whose URL is under the schema base
  `https://schemas.agent-ix.org/<org>/<repo>/` of a package the module declares
  in `semantic.imports` (a declared-import reference) without following it into
  the module bundle and without fetching it. For the module's own compiled
  validator an admitted declared-import `$ref` is an opaque subschema that
  accepts any value; the provider's own schema is validated when the provider
  loads. A `$ref` outside the module bundle, semantic-core and the declared
  imports' schema bases still fails `semantic.schema-ref-unshipped`. Whether
  the `$ref` names a type the provider exports is a cross-module check, below.
- The type name of a `$ref` is its file name stem: the final path segment of
  the URL with a trailing `.json` removed and then a trailing `.schema`
  removed if present. A `#fragment` or `$defs` pointer after the file is
  ignored for the type name.
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
- If two loaded modules export one semantic type name, then the loader SHALL
  refuse the later module with `semantic.duplicate-export` naming the type and
  both modules; type names are compared exactly, so spelling variants are a
  module concern; a refused module's exports do not count against later
  modules; no load order selects a definition (`agent-ix/quoin#338`,
  PLAT-214).
- A provider is loaded for an import when a loaded module, loaded without
  refusal, has a package identity equal to the import key and a version equal
  to the version `semantic.imports` declares. Otherwise the import is
  unchecked by every rule below, and `semantic.import-unresolved` applies.
- `semantic.import_types` is an optional sibling of `imports`:
  `{ "<org>/<repo>": ["TypeName", ...] }`, naming the types of an imported
  package the module uses. `imports` keeps its `{ package: exact semver }`
  shape; a manifest without `import_types` loads exactly as before.
- If an `import_types` entry names a type that the loaded provider's `exports`
  does not name, then the loader SHALL refuse the module with
  `semantic.unknown-import-type` (ERROR) at the locus
  `semantic.import_types.<package>` naming the package and the type.
- If the type name of a declared-import `$ref` in one of the module's own data
  schemas is not one of the loaded provider's `exports`, then the loader SHALL
  refuse the module with `semantic.unknown-import-type` at the locus
  `<section>[<type>].data_schema.schema` naming the package and the type.
- If a `semantic.imports` entry names a package that no loaded module provides
  at that exact version, then the loader SHALL emit the warning
  `semantic.import-unresolved` naming the package and the loaded versions;
  the module loads and its tokens from that import resolve as
  `unresolved` with reason `import-unresolved` ([FR-070](./FR-070-typed-properties-extraction.md)).
- If a package declared in `semantic.imports` is referenced by no `$ref` of
  the module's own data schemas (a reference is a declared-import `$ref`, the
  same scan that yields the type-level edges below), then the loader SHALL emit
  the warning `semantic.unused-import` naming the package; the module loads.
- If the import graph has a cycle, then the loader SHALL refuse every module
  on the cycle with `semantic.import-cycle` naming the cycle. The type-level
  graph has nodes `<org>/<repo>#<Type>`, where a module's own `<Type>` is a
  type file of its schemas. A `$ref` in the module's own file `<X>` whose URL
  is under imported package Q's schema base and whose type name is Q's type
  file `<T>` gives the edge `<P>#<X> -> <Q>#<T>`; edges inside a provider come
  the same way from the provider's own emitted-schema `$ref`s. Every
  package-level import cycle is refused; the type-level graph only changes how
  the cycle is named: over the `<org>/<repo>#<Type>` nodes that form it only when the
  `$ref`-derived type-level edges themselves form a cycle, otherwise (including
  when such edges exist but form no type-level cycle, or a module on the cycle
  has no `$ref`s into it) over packages as the package graph defines it. Edges come from `$ref`s between type files, not from `import_types`
  lists.

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

### Located record validation (PLAT-1476)

`validate_document` validates the *declaration record*
([FR-072](./FR-072-semantic-extraction-surface.md) /
[FR-075](./FR-075-model-feature-extraction.md) /
[FR-081](./FR-081-record-sections-extraction.md)) against the archetype's
resolved `data_schema`, reports its first error only, and does so only for a
resolved `object:` archetype. The record that `body_extraction.yield_pattern`
builds ([FR-011](./FR-011-body-extraction-dsl.md): `extract::extract`) is built
by extraction and by Filament, and the validate path never builds it. A module
that wants conditional checks over the artifact's own fields (a `resolution`
required only when a frontmatter `status` is `closed`) has them checked by no
one inside the engine, and no requirement gives their findings a line. Its
`data_schema` cannot carry them: the data schema describes the *emitted*
record (a `description` object with line spans, a computed `provenance`, rows
with a computed `line`), which no yield record can satisfy, and the declaration
record and the yield record have different keys and sources.

An archetype of a module with a `semantic` block MAY therefore carry a
`record_validation` object, a sibling of `data_schema` and independent of it:

```yaml
record_validation:
  schema: schemas/AnomalyFields.json   # the yield record's own schema
  locate_absence_at: status            # a key of yield_pattern.match
```

- `schema` is a module-relative path resolved, escape-checked and `$ref`-resolved
  under the same rules as a reference-form `data_schema` (AC-4, AC-5, with the
  refusal path `<section>[<type>].record_validation.schema`). Property names in
  it are the archetype's `yield_pattern.match` keys verbatim; the engine
  performs no name mapping. `data_schema` keeps validating the declaration
  record only, and the two schemas are unrelated.
- `locate_absence_at` is required and SHALL name a key of the archetype's own
  `yield_pattern.match`. The engine knows no archetype, field or key name; the
  declaration carries them.
- The **yield record** is the record FR-011 defines for the document and
  nothing else: the value of `extract::extract` over the archetype's
  `yield_pattern.match` (strings as FR-011 projects them; a `table_row` is its
  `column` cell, or the row's cells joined by tabs; the first value unless
  `multiple: true`; `regex` projection and placeholder dropping as FR-011
  defines). This requirement redefines none of it.
- **Layer and entry points.** The check runs for the archetype resolved by the
  frontmatter `type:` (an `artifact_types` entry, which is how Anomaly is
  dispatched) and for the `object:` archetype when that also declares
  `record_validation`, in the registry-backed entry points
  (`validate_document_in_registry`, and the bundle path that calls it). The
  type-only `validate_document` runs no record check, as today.
- **When it runs.** If extraction aborts because a `required: true` locator
  located nothing, no yield record exists and no `semantic.record-invalid`
  finding is raised; the locator's own missing-value finding is the report.
- **Every error.** Each error of the validator over the yield record is one
  finding with reason `semantic.record-invalid`, naming the schema path and the
  keyword's message, in the validator's error order (identical across runs).
- **One defect, one finding.** A record error whose property (the first path
  segment, a match key) already has a locator finding in the same run (an
  `assert` failure, such as `choices`, on that key) is not reported again. A
  conditional absence has no locator finding, because its locator is
  `required: false`, and is reported.
- **Absence.** A `required` error (outright or from an `if`/`then`
  conditional) is located at the document line of the value the
  `locate_absence_at` locator located (the line per
  [FR-033](./FR-033-locator-assert-facet.md) AC-18). If that locator located
  nothing, the line is the first line of the document.
- **Present but invalid.** An error at a present property is located at the
  line of the value that became that record value. A `multiple: true` array
  element is located at the line of the value that became that element, after
  filtering (`regex`, missing `column` cell, dropped placeholders, fallback
  chain), because each located value carries its own line through filtering. A
  value with no line of its own (a nested path inside an object value) takes
  the line of its property's value. An error with no property of its own (the
  record root, `additionalProperties`, `unevaluatedProperties`) is located at
  the `locate_absence_at` line.
- **Load refusals.** `record_validation` is refused with `semantic.invalid-value`,
  in the AC-14 reason form, when it appears in a module without a `semantic`
  block; on an archetype whose `yield_pattern` has no `match` (none, or
  `iterate_over` only); with a key other than `schema` and
  `locate_absence_at`; with `schema` absent or unresolvable (the AC-4/AC-5
  codes at `<section>[<type>].record_validation.schema`); or with
  `locate_absence_at` absent, empty, not a string, or not a key of
  `yield_pattern.match` (at `<section>[<type>].record_validation.locate_absence_at`).
- An archetype without `record_validation` behaves exactly as before.

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
| FR-069-AC-1 | A module with a valid `semantic` block and a reference-form `data_schema` loads with a `SemanticModule` record and a resolved schema, and `validate_document` over an artifact of that type validates the extracted record against the resolved schema; a declared-import `$ref` in that schema is an opaque accept-any subschema. | Test |
| FR-069-AC-2 | `contract_version: 2.0.0` fails every archetype of the module with a reason starting `semantic.unsupported-contract-version` and no other `semantic.*` reason. | Test |
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
| FR-069-AC-15 | Two loaded modules with different `semantic.package` values that both list one type in `semantic.exports` fail every archetype of the later sorted root with a reason starting `semantic.duplicate-export` naming the type and both modules, and the earlier module loads; type names are compared exactly; a refused module's other exports do not count, so a third module exporting one of them loads; the same modules with disjoint exports all load. | Test |
| FR-069-AC-16 | A manifest whose `semantic.imports` is `{ pkg: "1.0.0" }` with no `import_types` loads exactly as before; one adding `import_types: { pkg: ["T"] }` loads and records the list on its `SemanticModule`; each of a key absent from `imports`, an empty list, a duplicate type name, an empty type name, a non-array value and a non-string entry is refused with `semantic.invalid-value` at `semantic.import_types.<org>/<repo>` (unescaped package key, no index; or `semantic.import_types` for a non-object), in the AC-14 reason form. | Test |
| FR-069-AC-17 | With the provider loaded (a module loaded without refusal, package equal to the import key, at the declared version), an `import_types` entry naming a type absent from the provider's `exports` fails the module with a reason starting `semantic.unknown-import-type` naming the package and the type at `semantic.import_types.<package>`; every listed type exported by the provider loads; with the provider not loaded, loaded at another version, or refused, the entry is unchecked and the warning `semantic.import-unresolved` is emitted where no provider matches. | Test |
| FR-069-AC-18 | A module declaring an import that none of its own data schemas references by a `$ref` under the package's schema base loads with the warning `semantic.unused-import` naming the package; the same module with one such `$ref` emits no `semantic.unused-import`. | Test |
| FR-069-AC-19 | A `$ref` under the schema base of a declared import is admitted without being followed or fetched, and a value that would violate the provider's schema validates against the module's own compiled validator (the `$ref` accepts any value); with the provider loaded, one whose type name (file stem without `.json` and `.schema`, `#fragment` ignored) the provider does not export fails `semantic.unknown-import-type` at `<section>[<type>].data_schema.schema`; a `$ref` under the schema base of a package not in `semantic.imports` still fails `semantic.schema-ref-unshipped` naming the `$ref`. | Test |
| FR-069-AC-20 | Two modules whose `$ref`s give a type-level cycle (`<org>/<repo>#<Type>` nodes) fail with `semantic.import-cycle` naming the types that form the cycle; two modules whose packages import each other with no type-level edges fail naming the packages; a cycle where one module has no `$ref`s into it is still refused and named over packages; a package cycle refused although the `$ref` edges form no type-level cycle is named over packages. | Test |
| FR-069-AC-21 | A module with a `semantic` block whose archetype declares `record_validation: { schema: schemas/AnomalyFields.json, locate_absence_at: status }` over a `yield_pattern.match` key `status` loads and records the declaration and its resolved schema on the compiled archetype; `data_schema` still validates the declaration record only; the same manifest without `record_validation` loads and validates exactly as before. | Test |
| FR-069-AC-22 | Each of `record_validation` in a module without a `semantic` block; on an archetype whose `yield_pattern` has no `match` (absent, or `iterate_over` only); with an unknown sibling key; with `schema` absent or unresolvable; and with `locate_absence_at` absent, empty, non-string or naming a key absent from `yield_pattern.match`, is refused with `semantic.invalid-value` or the AC-4/AC-5 code at its manifest path in the AC-14 reason form, and a valid declaration in the same module is not refused. | Test |
| FR-069-AC-23 | `validate_document_in_registry` over an artifact dispatched by frontmatter `type:` to an opted-in archetype validates FR-011's yield record (`extract::extract` over `yield_pattern.match`, unmodified) against `record_validation.schema` and reports one `semantic.record-invalid` finding per validator error in the validator's order, so two missing required properties give two findings and a conforming document none; the type-only `validate_document` reports none. | Test |
| FR-069-AC-24 | A `required` error (outright or from an `if`/`then` conditional) is located at the document line of the value the `locate_absence_at` locator located; when that locator located nothing the finding is at line 1; no archetype, field or key name appears in the engine. | Test |
| FR-069-AC-25 | A present-but-invalid scalar is located at its value's line; an error in a `multiple: true` array element at the line of the value that became that element, still correct when a `regex` filter, a missing `column` cell or a dropped placeholder removed earlier values; an error in a value with no line of its own at its property's value line; and an error with no property of its own (record root, `additionalProperties`, `unevaluatedProperties`) at the `locate_absence_at` line. | Test |
| FR-069-AC-26 | A document whose `required: true` locator located nothing yields the locator's missing-value finding and no `semantic.record-invalid` finding; a value failing both a locator `assert: choices` and the schema `enum` yields the assert finding only; a conditional absence (locator `required: false`) yields the record finding. | Test |
| FR-069-AC-27 | A fixture module taking the real spec-artifacts-process Anomaly `yield_pattern` (`status`, `resolution`, `safety_impact`, `description`, `assessment` with its `table_row` locator) plus a `closed_by` `frontmatter_field` locator and a `record_validation` schema carrying the closed/resolved/assessed conditionals reports, through `validate_document_in_registry`: for `status: closed` with no `closed_by` and no `resolution`, two findings both at the `status:` line; for `status: assessed` with no assessment section, one finding at the `status:` line; for a valid document, none. | Test |
| FR-069-AC-28 | A fixture module with a different archetype, `state` as `locate_absence_at` and a different schema locates the absence at its own `state` line, and repeated validation yields identical findings in identical order. | Test |

## Dependencies

- **Upstream**: [FR-013](./FR-013-archetype-loader.md), [FR-076](./FR-076-relationships-extraction.md) (typed relation keys), [FR-045](./FR-045-filament-core-extraction-engine.md), [FR-067](./FR-067-versioned-assurance-export.md); `agent-ix/quoin` FR-070/FR-073/FR-075; `agent-ix/filament-core-service` FR-035-AC-13..15, `#23`; `agent-ix/filament-core-data` (PLAT-212): `schema/semantic/v1/module-manifest.schema.json` and `module-semantic-block.schema.json` SHALL add optional `import_types` (map of package identity to an array of non-empty unique strings) and ship in a new `@agent-ix/semantic-schema` / `agent-ix-semantic-schema` release, which the loader consumes through its Cargo git dependency
- **Downstream**: [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md)
