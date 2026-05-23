---
id: US-001
title: "LLM Emits a Validated Patch That Renders an FR Artifact"
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

As an **LLM agent producing spec artifacts**, I want the FR block schema exposed to me as a JSON Schema (via `schemars`), so that my tool definition rejects structurally invalid patches before I emit them, and a server-side `quire-rs::render` accepts my patch and produces a canonical FR markdown file byte-identical to what the Python Jinja2 renderer would have produced.

## Context

Today LLMs proposing FR/NFR/StR/etc. artifacts emit free-form markdown that downstream code has to repair. With schema-derived tool definitions, the model is constrained at the tool-call layer: it can only emit a `data` shape that `serde` will deserialize and `garde` will validate. When the patch fails, the field-keyed error goes back to the model and it retries the same tool call with the offending field corrected.

## Acceptance

- **US-001-AC-1**: A test exercises `schemars::schema_for!(FrData)` and asserts the generated JSON Schema contains all required fields, type constraints, and `pattern` constraints (e.g. `^[A-Z]{2,4}-[0-9]+$` for the id field).
- **US-001-AC-2**: A test feeds a structurally-valid `FrData` instance through `quire-rs::render` and asserts byte-equality with the Python Jinja2 reference output.
- **US-001-AC-3**: A test feeds a structurally-invalid patch (e.g. `id: "lowercase"`) and asserts the returned error names the offending field path and the expected pattern.
