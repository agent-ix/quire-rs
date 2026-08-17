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

### The version lives in the artifact, not the payload

[FR-008](./FR-008-byte-exact-slicing.md)-AC-5 bans a CLI-added payload field, so
there is no `"version"` key and SHALL NOT be one. Versioning lives in the
schema's `$id` and filename — `coverage-v1.schema.json`. A breaking payload
change mints `-v2` beside it rather than editing `-v1` in place, so a consumer
pinned to v1 keeps a schema that describes what it was written against.

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

An open machine vocabulary — the `diagnostics[].reason` token — SHALL NOT be
enumerated. A schema that closed it would reject a payload from a newer engine
that a consumer could otherwise read, converting a forward-compatible addition
into a break. A **closed** engine enum (`property`, `extraction`, `shape`) SHALL
be enumerated, because those are closed by construction
([FR-052](./FR-052-acceptance-criteria-property-classification.md)-CON-3) and a
value outside them is a defect a consumer should hear about.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-055-CON-1 | The schemas SHALL be authored, never generated. A derived contract changes silently with the type it derives from, which is the failure this FR closes. `schemars` stays out of the dependency graph. | Architecture | Test |
| FR-055-CON-2 | No payload SHALL gain a version key, a schema reference, or any other field for the benefit of this contract (FR-008-AC-5). The contract is carried by the published artifact alone. | Architecture | Test |
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
| FR-055-AC-7 | Neither payload contains a `version`, `$schema` or `schema_version` key, and `schemars` is absent from the dependency graph (CON-1, CON-2). | Test (TC-860) |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the coverage payload and its CR-057 baseline), [FR-052](./FR-052-acceptance-criteria-property-classification.md) (the classification records), [FR-053](./FR-053-obligation-record.md) (the obligation records both payloads carry)
- **Downstream**: `quire-cli` publishes the envelope conformance test alongside its emitter; quoin validates received payloads against these schemas and pins the version it targets (agent-ix/quoin#88)
