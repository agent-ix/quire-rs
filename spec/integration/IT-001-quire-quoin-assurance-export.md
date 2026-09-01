---
id: IT-001
title: "Quoin consumes a pinned Quire assurance export"
type: IT
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-067"
    type: "verifies"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-068"
    type: "verifies"
---
# IT-001: Quoin consumes a pinned Quire assurance export

## Objective

Verify the offline contract between Quire's assurance-export producer and
Quoin's assurance consumer: Quoin validates the published schema and version
premises, resolves every exported source locator at the pinned revision, and
uses Quire's relationships without re-reading specification frontmatter.

## Target Integration

The producer is the `quire-rs` assurance-export API exposed by the selected
Quire CLI. The consumer is Quoin's validated Quire adapter and read-only
assurance-case view. The boundary is the versioned JSON artifact plus the
vendored `assurance-v1.schema.json` contract; no network service participates.

## Preconditions

- A clean fixture repository is checked out at a full immutable revision.
- The fixture module version and schema digests are in Quoin's accepted premise
  set.
- The fixture contains one resolved and one dangling corpus relation, test and
  production bindings, an obligation, and all relation-availability states.
- Quoin vendors the assurance schema with the publisher revision and content
  digest recorded.

## Inputs

- The selected Quire binary and fixture module.
- The fixture repository identity and exact revision.
- Quoin's accepted format, module-version, and schema-digest premises.

## Test Procedure

1. Generate the assurance export twice from the pinned fixture.
   - IT-001-SC-01: both byte streams are identical and validate against the
     vendored assurance-v1 schema.
2. Import the export through Quoin's production adapter.
   - IT-001-SC-02: every artifact, obligation, symbol, and relationship is
     accepted exactly once.
3. Resolve every source locator against the pinned Git object and recompute its
   declared digest.
   - IT-001-SC-03: every path, line, and digest resolves without reading the
     working tree.
4. Render Quoin's assurance case from the imported graph and its existing
   auditor verdicts.
   - IT-001-SC-04: the rendered parent-child relationships equal the Quire
     export, and no Quoin frontmatter reader is invoked.
5. Change the format version, one module version, and one module-schema digest
   in three independent copies.
   - IT-001-SC-05: each copy is refused before any graph record is returned,
     naming the changed premise.
6. Read the fixture's missing and not-applicable relation observations.
   - IT-001-SC-06: Quoin preserves both states and does not convert either to a
     supported claim.

## Expected Results

The unchanged export validates, resolves to the pinned source, and produces the
same assurance graph Quire exported while Quoin retains sole ownership of
freshness verdicts. Every unsupported version premise fails closed, and absent
relation states remain distinguishable.

## Metadata

- Priority: P0
- Target Integration: Quire JSON export to Quoin adapter
- Automation: Automated cross-repository compatibility fixture

## Dependencies

**Upstream**: [FR-067](../functional/FR-067-versioned-assurance-export.md) and
[FR-068](../functional/FR-068-source-grounded-assurance-projection.md).
**Downstream**: Quoin's assurance-case and evidence-auditor regression suites.

## Traceability

This integration test verifies the boundary requested by
`agent-ix/quire-rs#386` and the Quoin consumer that replaces its duplicate
frontmatter graph reader.
