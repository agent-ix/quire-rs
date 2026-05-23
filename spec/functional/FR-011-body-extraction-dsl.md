---
id: FR-011
title: "Body-Extraction DSL Evaluator"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-003"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

`quire_rs::extract(doc: &QuireDocument, dsl: &ExtractionDsl) -> Result<serde_json::Map<String, Value>, QuireError>` SHALL evaluate the YAML-encoded DSL used by `spec-objects-architecture` and `ix-spec-objects`.

For each entry under `dsl.yield_pattern.match`, the evaluator SHALL:

1. **`from: frontmatter_field`** — read the value at `path` (a JSONPath-like array of keys) from `doc.frontmatter`. If the path is missing and `required: true`, return `QuireError::MissingField`. If missing and `required: false`, omit the key from the output.
2. **`from: section_body`** — locate the section whose heading matches `after_heading` (exact match) and emit `section.content` as the value. Missing section + `required: true` → error.
3. **`from: code_block`** — locate a fenced code block of the specified `language` (e.g. `json`, `mermaid`), optionally constrained to be inside a section named by `under_section`. Return the block's source. Missing + required → error.

The DSL itself is loaded from YAML via `serde_yaml::from_str` into the `ExtractionDsl` struct. The evaluator does NOT parse the DSL — it consumes already-deserialized values.

## Acceptance

- **FR-011-AC-1**: A test loads the `api_endpoint` DSL from `~/dev/spec-objects-architecture/spec_objects_architecture/manifest.yaml`, parses a real fixture document with `parse_document`, runs `extract`, and asserts the returned map contains `id`, `title`, `endpoint`, `routes`, `api_contract`.
- **FR-011-AC-2**: A test runs the `event` DSL (code_block with `language: json`) and asserts the extracted JSON is byte-equal to the fenced block content.
- **FR-011-AC-3**: A test runs a DSL with `required: true` against a document missing the required section and asserts `QuireError::MissingField` with the DSL key name.
- **FR-011-AC-4**: Every object_type defined across `spec-objects-architecture` and `ix-spec-objects` has at least one extraction test against a real document.
