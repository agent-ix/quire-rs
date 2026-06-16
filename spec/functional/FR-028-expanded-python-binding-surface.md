---
id: FR-028
title: "Expanded Python Binding Surface (Render / Validate / Extract / Edges)"
type: FR
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

> **CR note (render removal — 2026-06-04):** The render/templating feature is
> **removed** from `quire-rs` (no backward-compatibility layer). This FR drops the
> `render(...)` module-level function and the `QuireRenderError` exception class. The
> retained expanded surface is `validate` / `validate_manifest` / `extract` /
> `extract_frontmatter` / `harvest_edges` / `ExtractionContext` (plus
> `validate_document` from FR-023). FR-028-AC-1 (render byte-parity) is **retired**
> and FR-028-AC-7 is revised to the render-free exception hierarchy. See `spec.md`
> §2bis.

## Behavior

The `quire` Python module (FR-023) SHALL additionally expose the following surfaces to allow downstream Python services (filament-core, spec-editor, RAG harnesses) to swap their own per-document parse / validate / extract code for the Rust engine. Runtime extraction has two distinct inputs:

- Local authoring/rendering tools may load archetype modules from a filesystem root that was prepared by a CLI or by hand.
- Service/parser runtimes pass ObjectType rows directly from their caller. They SHALL NOT read `.ix`, package manifests, or a local module registry to discover ObjectTypes.

### Module-level functions

- `validate(archetype_name: str, module_root: str, data: dict) -> None` — load the module at `module_root`, resolve `archetype_name`; validates `data` against the archetype's compiled JSON Schema. Raises `QuireValidationError` on the first violation (carrying the dotted field path per NFR-005).
- `validate_manifest(payload: dict, schema_path: str) -> list[dict]` — compile the JSON Schema at `schema_path` (using the same `jsonschema` validator the engine uses internally) and validate `payload` against it. Returns `[]` when valid; returns structured violations of shape `{path, message, schema_keyword}` when invalid. Raises `QuireSchemaError` on schema load / compile failure. Consumers SHALL NOT parse exception strings to recover validation detail.
- `extract(archetype_name: str, module_root: str, document_text: str) -> dict` — parse `document_text`, evaluate the archetype's `body_extraction` DSL, and return `{"extraction": [...records], "edges": [{"target", "edge_type"}, ...]}`. Raises `QuireParseError` if the archetype has no DSL.
- `extract_frontmatter(text: str) -> dict` — returns `{"frontmatter": dict | None, "body": str}` from the Rust FR-006 parser. Python consumers SHALL use this body directly; they SHALL NOT re-split YAML frontmatter locally.
- `harvest_edges(doc: dict | str) -> list[dict]` — accepts either a raw markdown string or a parsed-document dict (from `parse_document`); returns deduplicated `[{"target", "edge_type"}, ...]` derived from frontmatter `relationships` and body `ix://` links (FR-026 per-doc harvest, no resolution).

### ExtractionContext

For service/parser runtimes, the module SHALL expose `ExtractionContext`:

- `ExtractionContext.from_object_types(object_types: list[dict] | {"items": list[dict]}) -> ExtractionContext` — compile caller-supplied ObjectType rows into the same Rust DSL/schema structures used by the native extractor. No filesystem, network, or module-package reads occur.
- `ctx.object_type_names() -> list[str]` — return compiled ObjectType names.
- `ctx.validate(object_type_name: str, data: dict) -> None` — validate extracted data against that ObjectType's schema, raising `QuireValidationError` with structured violation semantics.
- `ctx.extract(object_type_name: str, document_text: str) -> dict` — parse `document_text`, evaluate that ObjectType's `body_extraction`, validate each record, and return `{"extraction": [...records], "edges": [{"record_index", "type", "target"}, ...], "diagnostics": [...]}`.

`ExtractionContext` is the required path for `filament-parser-lib`, `filament-analysis-worker`, and `cloudmanager-local-sync` once they have fetched ObjectTypes from `filament-core-service`. Adding a Python-side parser, splitter, local registry fallback, or exception-string parser to those consumers is non-compliant.

### Exception hierarchy

The module SHALL expose:

- `QuireBaseError(Exception)` — base class.
- `QuireValidationError(QuireBaseError)` — `SchemaViolation`, `MissingField`.
- `QuireSchemaError(QuireBaseError)` — `UnknownArchetype`, `ArchetypeCollision`, `ModuleCollision`, `ArchetypeLoadError`, `ManifestError`, `InvalidSearchPath`, schema read / compile failures.
- `QuireParseError(QuireBaseError)` — `DslValidationError` and "no body_extraction DSL" sentinel.

### GIL

All new module-level functions SHALL release the GIL for the duration of the Rust computation (parity with `load_repo` and `Spec.from_path` per NFR-016).

## Acceptance

- FR-028-AC-1 — **RETIRED (render removal — 2026-06-04):** formerly asserted `quire.render(...)` byte-parity with `quire_rs::render_by_name`. The render binding is removed; this criterion is dropped from the required-coverage tally (id retained, immutable).
- **FR-028-AC-2**: `quire.validate(archetype, module_root, valid_data)` returns without raising; `quire.validate(archetype, module_root, invalid_data)` raises `QuireValidationError` whose message carries the same dotted field path the Rust validator produces (NFR-005).
- **FR-028-AC-3**: `quire.validate_manifest(payload, schema_path)` returns `[]` for valid payloads, returns one or more structured `{path, message, schema_keyword}` records for schema violations, and raises `QuireSchemaError` when `schema_path` is missing / unreadable / fails to compile.
- **FR-028-AC-4**: `quire.extract(archetype, module_root, text)` returns a dict with `extraction` (DSL records) and `edges` (frontmatter + body `ix://` harvest) keys; the `extraction` records match `quire_rs::extract().records` for the same inputs.
- **FR-028-AC-5**: `quire.extract_frontmatter(text)` returns the exact `frontmatter` and `body` produced by Rust `extract_frontmatter`; BOM, CRLF, malformed-YAML, non-object YAML, and missing-fence behavior match FR-006 without any Python-side splitter.
- **FR-028-AC-6**: `quire.harvest_edges(text_or_dict)` accepts both raw markdown and a parsed-document dict; output is deduplicated and equal between the two input shapes for the same document.
- **FR-028-AC-7**: Each `Quire*Error` subclass is `issubclass(QuireBaseError, Exception)` and is importable as `from quire import QuireBaseError, QuireValidationError, QuireSchemaError, QuireParseError`. `QuireRenderError` is not exported (render removed).
- **FR-028-AC-8**: Calling each new module-level function from two Python threads concurrently completes in wall-clock < 2× single-call (GIL release parity with FR-023-AC-5).
- **FR-028-AC-9**: `ExtractionContext.from_object_types([...]).extract(name, text)` returns the same records and emitted edges as the Rust extractor for equivalent compiled ObjectTypes, without reading a module root or `.ix` registry.
- **FR-028-AC-10**: `ExtractionContext` accepts both a bare list of ObjectType dicts and the core API envelope `{items: [...]}`.
