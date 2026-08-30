---
id: FR-067
title: "Versioned assurance export contract"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-018"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-055"
    type: "extends"
    cardinality: "1:1"
---
# FR-067: Versioned assurance export contract

## Description

When a caller supplies a loaded corpus, its registry, source-symbol extraction,
and an immutable repository identity, `quire-rs` SHALL emit one deterministic
`AssuranceExport` that validates against the hand-authored
`schemas/output/assurance-v1.schema.json` contract.

The export is a new offline interchange payload. It does not change the
coverage-v1 or properties-v1 payloads governed by
[FR-055](./FR-055-published-output-contract.md), and it is not a command: a CLI
may serialize the library value without becoming the owner of its shape.

## Inputs

- A loaded [FR-025](./FR-025-spec-corpus-model.md) `Spec` and the `Registry`
  used to interpret it.
- Source-symbol extraction and binding results produced under that registry.
- A caller-supplied source identity containing a non-empty repository identity
  and an immutable revision.

## Outputs

- `AssuranceExport`, serialized as JSON.
- `schemas/output/assurance-v1.schema.json`, shipped with the crate.
- A typed contract error that names the first unsupported or missing version
  premise and returns no partial export.

## Behavior

The envelope SHALL carry:

1. `format: "quire-assurance"` and `format_version: 1`.
2. `source.repository` and `source.revision`, supplied by the caller and copied
   verbatim after validation; the revision SHALL be a full 40-character
   lowercase Git object id rather than a moving ref or abbreviated hash.
3. One module premise per loaded module, ordered by name, containing its name,
   declared semantic version, and one SHA-256 schema digest per active
   archetype ordered by archetype name.
4. The source-grounded artifact, obligation, symbol, relation, and relation-
   observation projections defined by
   [FR-068](./FR-068-source-grounded-assurance-projection.md).

- `read_assurance_export` SHALL validate JSON against the selected published
  schema before constructing typed records.
- The caller SHALL supply the accepted module-version and schema-digest
  premises.
- When an export names a tuple outside the accepted set,
  `read_assurance_export` SHALL fail closed.

The exporter SHALL compute each schema digest over canonical JSON with object keys
sorted recursively, no insignificant whitespace, and arrays retained in
authored order. A formatting-only change therefore preserves the premise while
a semantic schema change invalidates it.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-067-CON-1 | Maintainers SHALL hand-author the assurance schema; an implementation-derived schema generator is prohibited from defining the contract. | Architecture | Test |
| FR-067-CON-2 | Export construction SHALL perform no Git command, network read, persistence, or cross-corpus resolution. The caller owns revision selection; Quire records the selected identity. | Architecture | Inspection |
| FR-067-CON-3 | A breaking shape or semantic change SHALL mint `assurance-v2.schema.json` and a new `format_version`; `assurance-v1.schema.json` remains byte-unchanged. | Compatibility | Test |
| FR-067-CON-4 | The explicit assurance `format_version` SHALL NOT be added to coverage-v1 or properties-v1. Those payloads retain FR-055's artifact-only versioning contract. | Compatibility | Test |
| FR-067-CON-5 | The assurance schema SHALL set `additionalProperties: false` on every object. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-067-AC-1 | The published assurance-v1 schema is valid JSON Schema draft 2020-12, its `$id` ends in `assurance-v1.schema.json`, and a complete fixture export validates with zero errors. | Test (TC-1084) |
| FR-067-AC-2 | Export construction refuses an empty repository identity, a non-immutable revision, an unnamed module, a module with no declared version, or an active archetype whose schema digest cannot be computed; no partial payload is returned. | Test (TC-1085) |
| FR-067-AC-3 | The envelope lists every loaded module exactly once with its declared version and every active `(module, archetype, schema_digest)` tuple exactly once, in deterministic order. | Test (TC-1086) |
| FR-067-AC-4 | Import refuses an unknown `format_version`, an unaccepted module version, or an unaccepted module-schema digest before returning any artifact, relation, or evidence record, naming the rejected premise. | Test (TC-1087) |
| FR-067-AC-5 | Two exports over identical inputs are byte-identical; changing only the caller-supplied source revision changes the source premise and no projected identity or relation. | Test (TC-1088) |
| FR-067-AC-6 | A checked-in assurance-v1 compatibility fixture pins every field, identity, relation kind, ordering rule, and state token; an additive compatible implementation continues to read it, while changing or removing a pinned v1 field fails the contract gate. | Test (TC-1089) |
| FR-067-AC-7 | Coverage-v1 and properties-v1 output remain byte-identical to their pre-export baselines and contain no assurance `format` or `format_version` key. | Test (TC-1090) |

## Dependencies

- **Upstream**: [FR-025](./FR-025-spec-corpus-model.md),
  [FR-051](./FR-051-source-symbol-extraction.md),
  [FR-053](./FR-053-obligation-record.md), and
  [FR-055](./FR-055-published-output-contract.md).
- **Downstream**: [FR-068](./FR-068-source-grounded-assurance-projection.md)
  defines the records inside the envelope; [IT-001](../integration/IT-001-quire-quoin-assurance-export.md)
  verifies a consumer against the published artifact.
