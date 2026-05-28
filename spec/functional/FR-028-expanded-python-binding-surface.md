---
id: FR-028
title: "Expanded Python Binding Surface (Render / Validate / Extract / Edges)"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-023"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-001"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-002"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

The `quire` Python module (FR-023) SHALL additionally expose the following module-level surfaces to allow downstream Python services (filament-core, spec-editor, RAG harnesses) to swap their own per-document parse / validate / extract code for the Rust engine without rebuilding a Registry per call.

### Module-level functions

- `render(archetype_name: str, module_root: str, data: dict) -> str` — load the module at `module_root`, resolve `archetype_name`, render `data`. Returns rendered markdown. Wraps `Registry::load_module` + `render_by_name`.
- `validate(archetype_name: str, module_root: str, data: dict) -> None` — same load path; validates `data` against the archetype's compiled JSON Schema. Raises `QuireValidationError` on the first violation (carrying the dotted field path per NFR-005).
- `validate_manifest(payload: dict, schema_path: str) -> None` — compile the JSON Schema at `schema_path` (using the same `jsonschema` validator the engine uses internally) and validate `payload` against it. Raises `QuireValidationError` on violation, `QuireSchemaError` on schema load / compile failure.
- `extract(archetype_name: str, module_root: str, document_text: str) -> dict` — parse `document_text`, evaluate the archetype's `body_extraction` DSL, and return `{"extraction": [...records], "edges": [{"target", "edge_type"}, ...]}`. Raises `QuireParseError` if the archetype has no DSL.
- `extract_frontmatter(text: str) -> dict | None` — extract just the frontmatter dict, or `None` if absent / malformed (FR-006 parity).
- `harvest_edges(doc: dict | str) -> list[dict]` — accepts either a raw markdown string or a parsed-document dict (from `parse_document`); returns deduplicated `[{"target", "edge_type"}, ...]` derived from frontmatter `relationships` and body `ix://` links (FR-026 per-doc harvest, no resolution).

### Exception hierarchy

The module SHALL expose:

- `QuireBaseError(Exception)` — base class.
- `QuireRenderError(QuireBaseError)` — `TemplateError`.
- `QuireValidationError(QuireBaseError)` — `SchemaViolation`, `MissingField`, schema-violation in `validate_manifest`.
- `QuireSchemaError(QuireBaseError)` — `UnknownArchetype`, `ArchetypeCollision`, `ModuleCollision`, `ArchetypeLoadError`, `ManifestError`, `InvalidSearchPath`, schema read / compile failures.
- `QuireParseError(QuireBaseError)` — `DslValidationError` and "no body_extraction DSL" sentinel.

### GIL

All five new module-level functions SHALL release the GIL for the duration of the Rust computation (parity with `load_repo` and `Spec.from_path` per NFR-016).

## Acceptance

- **FR-028-AC-1**: `quire.render(archetype, module_root, data)` returns the exact byte string `quire_rs::render_by_name` produces for the same inputs (byte-parity across the FFI boundary).
- **FR-028-AC-2**: `quire.validate(archetype, module_root, valid_data)` returns without raising; `quire.validate(archetype, module_root, invalid_data)` raises `QuireValidationError` whose message carries the same dotted field path the Rust validator produces (NFR-005).
- **FR-028-AC-3**: `quire.validate_manifest(payload, schema_path)` accepts valid payloads and raises `QuireValidationError` for schema violations; raises `QuireSchemaError` when `schema_path` is missing / unreadable / fails to compile.
- **FR-028-AC-4**: `quire.extract(archetype, module_root, text)` returns a dict with `extraction` (DSL records) and `edges` (frontmatter + body `ix://` harvest) keys; the `extraction` records match `quire_rs::extract().records` for the same inputs.
- **FR-028-AC-5**: `quire.extract_frontmatter(text)` returns the frontmatter dict or `None` for an empty / malformed frontmatter document (FR-006 parity).
- **FR-028-AC-6**: `quire.harvest_edges(text_or_dict)` accepts both raw markdown and a parsed-document dict; output is deduplicated and equal between the two input shapes for the same document.
- **FR-028-AC-7**: Each `Quire*Error` subclass is `issubclass(QuireBaseError, Exception)` and is importable as `from quire import QuireBaseError, QuireRenderError, QuireValidationError, QuireSchemaError, QuireParseError`.
- **FR-028-AC-8**: Calling each new module-level function from two Python threads concurrently completes in wall-clock < 2× single-call (GIL release parity with FR-023-AC-5).
