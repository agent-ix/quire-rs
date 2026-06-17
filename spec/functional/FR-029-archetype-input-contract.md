---
id: FR-029
title: "Archetype Input Contract Surface"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-003"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "requires"
    cardinality: "1:1"
---

> **CR note (recast by ADR 0004):** The input contract is no longer derived from a
> render template's variables. With direct-markdown authoring (ADR 0004) there is no
> required render template; the per-archetype contract is a **skeleton/example**
> derived from `frontmatter_schema_ref` + the `body_extraction` asserts ([FR-033](./FR-033-locator-assert-facet.md)) — the
> structure an author fills and `validate_document` ([FR-032](./FR-032-validate-document.md)) checks. Below, "template
> variables" and "required-section → variable mapping" are superseded by "asserts →
> required structure"; the contract still SHALL be derived from the loaded module
> (manifest + schema), never inferred from rendered markdown. See ADR 0004.

## Description

`quire-rs` SHALL expose an input contract for each loaded archetype that is suitable for LLM render agents and other non-Rust consumers. The contract SHALL combine:

1. The archetype name.
2. The frontmatter JSON Schema already exposed by [FR-003](./FR-003-archetype-schema-surface.md).
3. The manifest `required_sections` entries for that archetype.
4. The template variables referenced by the archetype template.
5. A mapping from each required section to the template variables that can populate that section, when the template structure makes that mapping statically knowable.

The engine SHALL NOT infer the contract from rendered markdown. It SHALL derive it from the loaded module manifest, schema document, and template source used by the same compiled archetype that rendering uses.

The contract output SHALL be deterministic JSON-serializable data. It SHALL contain enough information for an agent to populate all required sections before calling render, without relying on friendly template defaults such as `TODO`, `TBD`, or empty placeholder tables.

When a required-section-to-variable mapping cannot be determined statically, the contract SHALL still include the section and SHALL mark the mapping as unresolved with an actionable diagnostic naming the archetype and section.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-029-AC-1 | `input_contract_for(registry, "FR")` returns the FR frontmatter schema, required sections `Description`, `Specification`, `Acceptance Criteria`, and `Dependencies`, and the FR template variables used to populate those sections. | Test |
| FR-029-AC-2 | `input_contract_for(registry, "NFR")` returns the NFR required sections and includes variables that feed `Scope`, `Measurement and Evaluation`, and `Verification`. | Test |
| FR-029-AC-3 | For every archetype in `spec-artifacts-iso`, the contract contains each manifest `required_sections` entry exactly once, preserving manifest order. | Test |
| FR-029-AC-4 | The JSON serialization of a contract is byte-identical across repeated calls against the same loaded module. | Test |
| FR-029-AC-5 | `input_contract_for(registry, "nonexistent")` returns `Err(QuireError::UnknownArchetype)`. | Test |
| FR-029-AC-6 | A fixture with a required section whose variables cannot be mapped still returns a contract with that section and an unresolved-mapping diagnostic; it does not silently omit the section. | Test |

## Dependencies

- **Upstream**: [FR-003](./FR-003-archetype-schema-surface.md), [FR-013](./FR-013-archetype-loader.md)
- **Downstream**: none
