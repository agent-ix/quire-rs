---
id: US-003
title: "Spec-Objects Extractor Evaluates body_extraction DSL on a Document"
artifact_type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/ix-spec-objects"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/spec-objects-architecture"
    type: "consumes"
    cardinality: "1:1"
---

## Story

As an **operator or service runtime with ObjectType definitions already in hand**, I want to feed `quire-rs` a parsed `QuireDocument` plus a `body_extraction.yield_pattern.match` DSL map (frontmatter_field / section_body / code_block keys), and receive a typed map of extracted values, so that all ObjectTypes defined across core-activated modules can be evaluated by one engine without Python locator execution.

## Context

The DSL used to be interpreted by `filament-parser-lib` tier-2 in Python. Each call walked the document, looked up section bodies, decoded fenced code blocks, and returned a flat map. `quire-rs` SHALL expose an equivalent `extract(doc, dsl)` API and a Python `ExtractionContext.from_object_types(...)` API that operate against caller-supplied data — no Python re-parsing, no Python locator path, no IO required for service extraction.

## Acceptance

- **US-003-AC-1**: A test uses the `api_endpoint` DSL from `~/dev/spec-objects-architecture/spec_objects_architecture/manifest.yaml` and asserts the Rust extractor produces a map containing `id`, `title`, `endpoint`, `routes`, `api_contract` for a real fixture document.
- **US-003-AC-2**: A test uses the `event` DSL (code_block with `language: json`) from `~/dev/ix-spec-objects/object_types/.../event/` and asserts the extracted JSON is byte-equal to the fenced block content.
- **US-003-AC-3**: When a `required: true` key fails to extract, `extract()` returns a typed error naming the missing key and the DSL path that failed.
- **US-003-AC-4**: A Python test constructs `ExtractionContext.from_object_types([...])` from an in-memory ObjectType snapshot and extracts a real document without reading a module root, `.ix`, or package manifest.
