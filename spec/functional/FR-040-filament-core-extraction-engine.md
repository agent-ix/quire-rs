---
id: FR-040
title: "Canonical Filament core extraction engine"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-015"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "requires"
---
# [FR-040] Canonical Filament core extraction engine

## Description

The `quire-rs` library SHALL provide a Filament-facing extraction API that accepts one
markdown document plus caller-provided ObjectType snapshots and returns a JSON-serializable
core extraction result compatible with the shared `CoreExtractionResult` contract.

## Inputs

- Project id, document id, optional artifact id, relative path, repository name, and markdown text
- A caller-provided ObjectType snapshot containing name, schema, allowed links,
  optional `body_extraction`, and plugin flag fields

## Outputs

- Object type records
- Graph node records
- Graph edge records
- Extraction diagnostics
- Extraction errors

## Behavior

- The engine SHALL parse frontmatter and markdown using the existing Rust parser.
- When frontmatter is absent or unparsable, the engine SHALL return an empty extraction
  result with a `no_frontmatter` informational diagnostic.
- When `frontmatter.object` names an unknown ObjectType, the engine SHALL return no node
  for that object.
- When `frontmatter.object` names an unknown ObjectType, the engine SHALL emit an
  `unknown_object_type` diagnostic.
- When the engine receives an ObjectType with `body_extraction` absent and `has_plugin`
  false, the engine SHALL run Tier 1 frontmatter extraction.
- When an ObjectType has `body_extraction` and `has_plugin` is false, the engine SHALL
  evaluate the existing Rust body-extraction DSL and validate each emitted record.
- When an ObjectType has `has_plugin` true, the engine SHALL surface an extraction error
  or diagnostic and SHALL NOT execute Python plugin discovery.
- The engine SHALL harvest frontmatter relationship sugar and body `ix://` markdown links
  into graph edges with stable provenance metadata.
- The engine SHALL deduplicate duplicate graph edges by source, edge type, and target,
  preserving the first edge and emitting a diagnostic for duplicates.
- The engine SHALL produce stable SHA-256 ids for object type, node, and edge records.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-040-CON-1 | The extraction engine SHALL NOT read PGlite, Electron IPC, HTTP, auth, CloudManager sync, workspace watch queues, or embeddings. | Architecture | Inspection |
| FR-040-CON-2 | ObjectType snapshots SHALL be supplied by the caller. | Architecture | Test |
| FR-040-CON-3 | The engine SHALL NOT discover a runtime registry over the network or from service configuration. | Architecture | Test |
| FR-040-CON-4 | Non-`ix://` graph references SHALL normalize to `ix://agent-ix/<repo_name>/<value>` before record id generation. | Compatibility | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-040-AC-1 | A Tier 1 fixture emits one validated graph node whose `dataJson` includes frontmatter `id` as `code` and frontmatter `title` as `title`. | Test (TC-633) |
| FR-040-AC-2 | A Tier 2 fixture emits graph nodes and record-derived graph edges equivalent to the existing Rust DSL extraction result for the same ObjectType snapshot. | Test (TC-634) |
| FR-040-AC-3 | Unknown ObjectType, no-frontmatter, malformed `ix://`, duplicate-edge, and plugin-flag fixtures produce diagnostics or errors without panicking. | Test (TC-635) |
| FR-040-AC-4 | Frontmatter relationship sugar and body `ix://` links produce deterministic graph edges with source/original-target or original-uri metadata. | Test (TC-636) |
| FR-040-AC-5 | Repeating extraction with identical input produces byte-identical JSON output ordering and stable record ids. | Test (TC-637) |

## Dependencies

- **Upstream**: [FR-011](./FR-011-body-extraction-dsl.md), [FR-026](./FR-026-intra-spec-reference-resolution.md), [FR-028](./FR-028-expanded-python-binding-surface.md)
- **Downstream**: [FR-041](./FR-041-filament-extraction-bindings.md), `filament-parser-lib` compatibility shim, and Filament IDE worker sync
