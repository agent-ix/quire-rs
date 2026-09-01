---
id: FR-055
title: "Published JSON Output Contract"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-052"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
---
# FR-055: Published JSON Output Contract

## Description

`quire-rs` SHALL publish a versioned JSON Schema for each machine-readable
payload it defines — the [FR-050](./FR-050-declarative-coverage-computation.md)
coverage report and the [FR-052](./FR-052-acceptance-criteria-property-classification.md)
property-classification records — as artifacts shipped with the crate, and SHALL
gate the emitted payloads against them on every test run.

`quire coverage --json` and `quire properties --json` are called "the stable
interface" in three places — `quire-cli`'s own source, `quire-cli` FR-018, and
every quoin skill that reads them. Nothing made that true:

- **No output schema existed anywhere.** The only `.schema.json` files in the
  tree are *input* frontmatter schemas, and `schemars` is deliberately banned
  (`scripts/audits/check_no_schemars.sh`, [FR-003](./FR-003-archetype-schema-surface.md)-AC-4:
  "consumers supply pre-built schemas"). No pre-built schema was ever published.
- **No shape test pinned `properties`.** `quire-cli` IT-094 asserts run-to-run
  byte-identity and the presence of `row_id`, which a payload that dropped every
  other field would still satisfy.
- **quoin consumed both shapes as prose** in skill markdown, and its own
  `spec/review.md` Finding 8 records "no contract test against quire".

### Hand-authored, because the ban is the point

The schemas are written by hand and reviewed like code. Deriving them from the
Rust types would make the contract a *shadow* of the implementation, changing
silently whenever a struct did — which is the failure this FR exists to close,
not a convenience it should adopt. `schemars` stays banned; FR-003-AC-4 already
said consumers supply pre-built schemas, and this is the crate supplying them.

### The CONTRACT version lives in the artifact, not the payload

There is no `"version"` key naming which revision of this contract a coverage
or properties payload conforms to, and SHALL NOT be one. Versioning lives in
the schema's `$id` and filename — `coverage-v1.schema.json`. A breaking payload
change mints `-v2` beside it rather than editing `-v1` in place, so a consumer
pinned to v1 keeps a schema that describes what it was written against.

> **CR-156** (`agent-ix/quire-rs#386`, 2026-08-29) narrows "a payload" to the
> two payloads this requirement defines. The new offline assurance interchange
> contract in [FR-067](./FR-067-versioned-assurance-export.md) is
> self-identifying so an importer can refuse an unknown format before reading
> records. That does not add a key to either existing payload or change either
> published v1 schema.

> **CR-104** (agent-ix/quire-cli#68, agent-ix/quire-rs#264 Wave 0) — *the
> contract version and the instrument version are different facts, and CON-2
> conflated them.*
>
> As written, CON-2 forbade "a version key, a schema reference, or **any other
> field** for the benefit of this contract", citing `quire-cli` FR-008-AC-5 ("no
> CLI version string appears in JSON output"). Read literally, that banned the
> one field the measurement programme most needs.
>
> The two are not the same claim. **Contract version** answers *which schema
> describes this shape* — and putting it in the payload is genuinely wrong,
> because it lets a payload assert its own conformance. That ban stands
> unchanged. **Instrument provenance** answers *which build computed these
> numbers* — and its absence is a measured defect, not a design property:
> `quire --version` reports the CLI crate version while the engine is a git
> dependency pinned by tag that **no surface reports**. The installed CLI 0.29.0
> pins engine v0.42.0; `binding_census` landed in v0.43.0. Every ecosystem figure
> in four battletest passes was produced by a binary that could not emit the one
> signal saying whether the binder read a single test, and nothing in its output
> said so. A payload saved to disk carried no way to find out afterwards.
>
> CON-2 is narrowed to the contract-version case it was actually arguing for.
> An `engine` object carrying `{cli, engine, capabilities}` is admitted, and
> AC-8 gates its shape. `capabilities` is a **token list, not version
> arithmetic**: a consumer asserts it needs `binding_census`, never that the
> engine is `>= 0.43.0`, because a version comparison in a consumer is a second
> place the contract lives. The key stays **optional**, so an in-process
> `CoverageReport::to_json` — which cannot know a CLI version — still conforms,
> and so does a payload from any CLI predating the field.
>
> The sibling narrowing of `quire-cli` FR-008-AC-5 is carried by
> agent-ix/quire-cli#69 and is **not** a fact about this repository — stating a
> cross-repo change in the present indicative is how a spec comes to assert
> something that never landed, and no gate here could notice. What that change
> argues is the same thing in the same terms: the ban on a CLI **version string
> standing alone** in the payload is what AC-5 meant and what survives.
>
> **This constraint is enforced over the schemas, not over one payload.** TC-860
> originally read a single payload instance and checked its root keys, which was
> a weaker gate than the constraint even before this change — and CR-104 is the
> change that introduces nesting for a banned key to hide in. It now walks every
> `properties` map in both published artifacts, so a declared
> `engine.schema_version` fails here rather than in a consumer.

### The gate rides the baseline that already exists

CR-057 checked in `tests/fixtures/coverage_baseline/expected.json` and byte-diffs
it on every run, with regeneration a deliberate reviewable act. The conformance
check validates **that same file** against the published schema. One corpus, two
gates, and a payload change now fails both unless the schema is updated with it —
which is exactly the review moment the contract needs.

### What is not enforced here

The `properties` **envelope** (`{documents: [{document, archetype, criteria}]}`)
is assembled in `quire-cli`, not in this crate, so this crate cannot emit it to
check it. The schema for it is published here anyway — one place, so the two
sides conform to the contract rather than to each other — and the conformance
test for the envelope belongs in `quire-cli` alongside the emitter. This crate
gates the part it emits: every criterion record.

## Inputs

- The checked-in coverage baseline corpus and its expected report.
- Classification records produced from a fixture document.

## Outputs

- `schemas/output/coverage-v1.schema.json`
- `schemas/output/properties-v1.schema.json`

## Behavior

Both schemas SHALL be JSON Schema draft 2020-12 documents that set
`additionalProperties: false` on every object they describe and carry a `$id`
ending in the versioned filename.

Each schema SHALL mark every field the engine omits when empty
(`no_symbol_rows`, `criteria`, `diagnostics`, `obligations`, `parameters`) as
optional and SHALL give it no default. "Absent" and "present and empty" are
different facts on these payloads, and the byte-identity property depends on the
difference.

A key **no** engine surface emits — one assembled by a consuming CLI, which is
the only layer holding the fact — SHALL likewise be optional and be admitted
only where its absence is a defect the payload alone can be read to
diagnose. `engine` (CR-104) is the first: `CoverageReport` has no such member
and never will, because this crate cannot know which binary is calling it. The
optionality is not a courtesy to older consumers here; it is what keeps an
in-process `to_json` caller conformant.

An open machine vocabulary — the `diagnostics[].reason` token, and the `engine`
block's `capabilities` list — SHALL NOT be enumerated. A schema that closed it would reject a payload from a newer engine
that a consumer could otherwise read, converting a forward-compatible addition
into a break. A **closed** engine enum (`property`, `extraction`, `shape`) SHALL
be enumerated, because those are closed by construction
([FR-052](./FR-052-acceptance-criteria-property-classification.md)-CON-3) and a
value outside them is a defect a consumer should hear about.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-055-CON-1 | Maintainers SHALL author the schemas by hand, never generate them. A derived contract changes silently with the type it derives from, which is the failure this FR closes. `schemars` stays out of the dependency graph. | Architecture | Test |
| FR-055-CON-2 | Neither coverage-v1 nor properties-v1 SHALL gain a key naming which revision of this contract it conforms to — no version key, no schema reference, no `$schema` (CR-104, CR-156). Their contract is carried by the published artifact alone. This does not reach instrument provenance or a separately defined interchange format. | Architecture | Test |
| FR-055-CON-3 | A breaking payload change SHALL mint a new versioned schema file rather than editing an existing one in place, so a consumer pinned to a version keeps a schema describing what it pinned. | Process | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-055-AC-1 | Both published schemas are themselves valid JSON Schema draft 2020-12 documents, and each carries a `$id` ending in its versioned filename. | Test (TC-854) |
| FR-055-AC-2 | The checked-in coverage baseline report validates against `coverage-v1.schema.json` with zero errors, so the byte-golden corpus and the contract are gated over one input. | Test (TC-855) |
| FR-055-AC-3 | A coverage report over a corpus exercising obligations, criteria counts and scan diagnostics validates against the schema, so the optional keys are covered by a payload that carries them rather than only by one that omits them. | Test (TC-856) |
| FR-055-AC-4 | Every criterion record emitted for a fixture document validates against the `Criterion` definition in `properties-v1.schema.json`. | Test (TC-857) |
| FR-055-AC-5 | A payload with an added field is rejected by its schema, confirming `additionalProperties: false` holds everywhere rather than only at the root. | Test (TC-858) |
| FR-055-AC-6 | Removing an optional key from a valid payload leaves it valid, and removing a required one makes it invalid, so the optional/required split matches the engine's skip-when-empty behaviour. | Test (TC-859) |
| FR-055-AC-7 | Neither coverage-v1 nor properties-v1 contains a `version`, `$schema` or `schema_version` key, and `schemars` is absent from the dependency graph (CON-1, CON-2). | Test (TC-860) |
| FR-055-AC-8 | Both schemas define an optional `engine` object requiring `cli`, `engine` and `capabilities` (CR-104). A payload carrying one conforms; a payload omitting it conforms; a payload whose `engine` is missing a required member, or carries an undeclared member, is rejected. | Test (TC-1010) |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the coverage payload and its CR-057 baseline), [FR-052](./FR-052-acceptance-criteria-property-classification.md) (the classification records), [FR-053](./FR-053-obligation-record.md) (the obligation records both payloads carry)
- **Downstream**: `quire-cli` publishes the envelope conformance test alongside its emitter; quoin validates received payloads against these schemas and pins the version it targets (agent-ix/quoin#88)
