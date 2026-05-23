---
id: US-001
title: "LLM Emits a Validated Patch That Renders an Archetype"
artifact_type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-002"
    type: "implements"
    cardinality: "1:1"
---

## Story

As an **LLM agent producing canonical artifacts**, I want the archetype's JSON Schema surfaced to my tool definition (from the schema file that Filament authored and ix-cli synced to disk), so that my tool input is constrained at the call layer. A server-side `quire-rs::render(archetype, my_data)` accepts my emitted value and produces canonical markdown byte-identical to what the Python Jinja2 reference would have produced.

## Context

The JSON Schema for each archetype is **data, not derived from Rust types** — it lives on disk under `~/.ix/schemas/<module>/schemas/<name>-frontmatter.schema.json`. The engine surfaces it via `quire-rs::schema_for(registry, "fr")`. The agent's tool-call layer uses that schema as the tool's input contract.

When the model emits a structurally invalid patch (or one violating field constraints introduced by the merge), `render` returns a typed `QuireError::SchemaViolation` with the offending field path. The model retries with the field corrected.

## Acceptance

- **US-001-AC-1**: A test calls `schema_for(registry, "fr")` and asserts the returned `Value` has `required: ["id", "title", "artifact_type"]`, `id.pattern == "^[A-Z]{2,4}-[0-9]+$"`, `artifact_type.const == "FR"`.
- **US-001-AC-2**: A test deserializes that schema as the input contract for a mock LLM tool, generates a valid value, calls `render(compiled_archetype, value)`, and asserts byte-equality with the Python Jinja2 reference output.
- **US-001-AC-3**: A test feeds a structurally invalid value (e.g. `id: "lowercase"`) and asserts the returned error names the offending field path and the violated constraint.
- **US-001-AC-4**: A test confirms the schema returned by `schema_for(...)` is byte-equal (modulo whitespace) to the original on-disk schema file — the engine surfaces it unmodified.
