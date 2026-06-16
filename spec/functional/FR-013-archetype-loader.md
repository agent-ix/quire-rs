---
id: FR-013
title: "Archetype Loader: Filesystem-First, Sync-Agnostic"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-001"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

> **CR note (render removal — 2026-06-04):** The render/templating feature is
> **removed** from `quire-rs` (no backward-compatibility layer). The loader is now
> **schema-only**: it no longer parses or registers MiniJinja templates and no longer
> reads `template_ref`. Manifest entries reference their schema by `schema_ref` /
> `frontmatter_schema_ref`; a `template_ref` field is ignored at load (and rejected
> by the unified shape, FR-031). The Behavior and acceptance criteria below are
> updated to the schema-only contract. See `spec.md` §2bis.

> **CR note (module-dir repoint — 2026-06-15):** The default search root moves
> from `~/.ix/schemas/` to **`~/.ix/filament/modules/`**, and the env override
> gains a preferred name `IX_FILAMENT_MODULES_PATH` with the legacy
> `IX_SCHEMA_PATH` retained as a back-compat alias (read only when the new var
> is unset). This aligns the engine with the single directory that `ix-spec`
> (via `@agent-ix/ts-plugin-kit`) materializes installed Filament modules into,
> so one install serves both the CLI and the engine. The change is additive and
> non-breaking: explicit `load_from`/`load_module` paths are unaffected, and
> `IX_SCHEMA_PATH` keeps working. The Search-path section and AC-1 below are
> updated; no new AC is added (the alias is exercised by AC-2/AC-8).

## Behavior

`quire-rs` SHALL load archetypes from the **local filesystem**. The engine has no network calls, no Filament API client, and no required runtime services. Whatever populates the local directory tree (Filament sync, hand-authoring, git checkout, an unzipped distribution tarball) is outside the engine's concern.

### Search path

The engine resolves archetype roots from, in priority order:

1. Explicit `Registry::load_from(paths: &[&Path])` constructor argument.
2. `IX_FILAMENT_MODULES_PATH` environment variable (preferred), then the legacy `IX_SCHEMA_PATH` alias when the preferred var is unset — colon-separated list of directories (PATH-style).
3. Default: `~/.ix/filament/modules/`.

Each path entry is treated as a directory containing one or more **module roots**. The engine walks one level deep to discover modules.

### Module layout (convention)

Each module root SHALL match the existing `spec-artifacts-*` shape:

```
<module-root>/
├── manifest.yaml           # artifact_types: [...] (and/or object_types: [...])
└── schemas/
    └── <name>-frontmatter.schema.json
```

The `manifest.yaml` is the authoritative entry point. It enumerates archetypes; each entry references its schema by relative path (`schema_ref` / `frontmatter_schema_ref`). The loader does NOT auto-discover schemas from the filesystem layout — every archetype is explicitly declared in a manifest so authoring is intentional. (Templates and `template_ref` are removed with the render feature.)

### Public API

```rust
pub struct Registry { /* ... */ }

impl Registry {
    pub fn load_from(paths: &[&Path]) -> Result<Registry, QuireError>;
    pub fn load_module(module_root: &Path) -> Result<Registry, QuireError>;
    pub fn from_env() -> Result<Registry, QuireError>;        // IX_FILAMENT_MODULES_PATH / IX_SCHEMA_PATH then default
    pub fn from_default() -> Result<Registry, QuireError>;    // ~/.ix/filament/modules/ only

    pub fn archetype(&self, name: &str) -> Option<&CompiledArchetype>;
    pub fn archetype_names(&self) -> impl Iterator<Item = &str>;
    pub fn module_names(&self) -> impl Iterator<Item = &str>;
}
```

### Search root vs. single module

`Registry::load_from(paths)` treats each entry in `paths` as a **search root** whose direct children are candidate module directories (one level deep). `Registry::load_module(module_root)` treats its single argument as a **module directory** — `manifest.yaml` MUST live directly under it, and no siblings are inspected.

Callers that have already resolved a specific module path (e.g. a CLI receiving `--module <path>`) SHALL use `load_module` rather than promoting to the parent and calling `load_from`. Promoting to the parent silently exposes every sibling directory as a candidate module, which is both surprising and a path-safety concern when the argument is user-controlled.

### Compiled archetype surface

`CompiledArchetype` SHALL expose the parsed `body_extraction` DSL for object types as a public field/accessor (`body_extraction: Option<ExtractionDsl>` and `fn body_extraction(&self) -> Option<&ExtractionDsl>`). The DSL is populated from the same parse pass that validates it at load time (FR-011-AC-6/7/8), so downstream `extract()` callers do NOT need to re-read `manifest.yaml` to drive the extractor.

### Path-safety diagnostic

Path-safety violations raised by consumers (CLIs, services) that resolve user-controlled path strings before calling the loader SHALL be expressed as `Diagnostic::PathTraversal { argument, path, reason }` (with `PathTraversalReason::{DotDotSegment, SymlinkEscape, EscapesModuleRoot}`) rather than as per-consumer parallel `PathSafetyViolation` enums. The engine itself does not currently emit this variant — it is provided so the diagnostic vocabulary stays centralized in `quire-rs`.

### Load-time work (amortized)

For each archetype the loader SHALL:

1. Parse the JSON Schema document and compile it into a runtime validator (e.g. `jsonschema::JSONSchema::compile`).
2. Cache the (validator, metadata, optional `body_extraction` DSL) tuple as a `CompiledArchetype` keyed by archetype name.

Templates are NOT parsed or registered (render is removed). Per-validate and per-extract operations SHALL NOT re-read disk and SHALL NOT re-parse schemas. Load cost is amortized over the process lifetime (see NFR-007).

### Error model

- A manifest entry whose `schema_ref` does not exist on disk: `QuireError::ArchetypeLoadError` with file path and reason. The loader continues with other archetypes and aggregates errors.
- A schema document that is itself malformed JSON Schema: `QuireError::ArchetypeLoadError` with schema path and parse error.
- A missing search path entry on disk: warning diagnostic (not a fatal error) — empty modules are valid.

### Path-handling edge cases

- A search path entry that is a **file, not a directory**: warning diagnostic; entry is skipped, other entries process normally.
- A search path entry whose **read permission is denied**: warning diagnostic with `errno`; entry is skipped.
- **Tilde (`~`) expansion**: the loader SHALL expand a leading `~/` or `~` to the user's home directory before resolving. No mid-path tilde expansion.
- **Environment variables** in `IX_FILAMENT_MODULES_PATH` / `IX_SCHEMA_PATH` entries are NOT expanded — entries are taken literally after splitting on `:`.
- **Duplicate entries** in the search-path env var (same canonical path appearing more than once): the loader SHALL deduplicate before walking; modules are loaded at most once per canonical path.
- **Symlink loops**: the directory walker SHALL maintain a `visited` set of canonical paths and break cycles. Cycle detection emits a warning diagnostic; the cyclic branch is skipped.

### Concurrency model

- `Registry` is `Send + Sync`. Cloning a `Registry` is reference-counted (cheap, `Arc<Inner>`-style) — clones share underlying `CompiledArchetype` instances.
- A `Registry` is immutable after construction. To change the active archetype set, construct a new `Registry` via `load_from` and drop the previous one. Outstanding references to the previous `Registry`'s `CompiledArchetype` keep it alive until the last reference drops.
- `Registry::load_from(...)` SHALL NOT mutate any global state; multiple registries may coexist in the same process.

### WASM feature: filesystem-free loader

The crate exposes an additive Cargo feature `wasm` (v0.3.1) that drops the `jsonschema/resolve-file` activation, allowing `quire-rs` to compile against `wasm32-unknown-unknown` (where `url::Url::to_file_path` is unavailable). Under `--no-default-features --features wasm`:

- `Registry::load_from`, `load_module`, `from_env`, `from_default` remain available but degrade to whatever filesystem the host exposes (typically none under `--target web`).
> **CR note (render removal — 2026-06-04):** AC-15 originally documented a 3-arg `from_inline_parts(manifest_yaml, schemas, templates)` with a `templates` map keyed by `template_ref`. With the render feature removed, the real signature (`src/registry.rs`) is 2-arg — no `templates` param, no `template_ref`.
- Callers SHALL use `Registry::from_inline_parts(manifest_yaml, schemas)` (and the strict variant `from_inline_parts_strict`) to build a registry from an in-memory module blob — no filesystem access. `schemas` is a `BTreeMap<String, String>` keyed by the manifest's relative-reference strings (`frontmatter_schema_ref`).
- Diagnostics and per-archetype failure aggregation behave identically to the filesystem loader; missing entries in the `schemas` map surface as `ArchetypeLoadFailure` with reason `"inline schema '<ref>' not provided"`.
- Cross-file `$ref` resolution is unavailable under `wasm` (consistent with FR-002-AC-7's existing rejection).

This is an additive amendment — the default (native) build is unchanged. `cargo check --features python` and `cargo check` continue to include `resolve-file`. Verified by `cargo check --no-default-features --features wasm --target wasm32-unknown-unknown --lib`.

### File-race assumptions

The loader assumes the upstream sync tool (canonically `ix-cli`, see Appendix A in `spec.md`) writes files **atomically** — i.e. writes to a temp path and renames, never in-place. The loader does NOT acquire file locks. If a non-atomic writer modifies files during load, partial-read errors surface as `QuireError::ArchetypeLoadError` for the affected archetype; the rest of the registry loads normally.

## Acceptance

- **FR-013-AC-1**: `Registry::from_env()` with `IX_FILAMENT_MODULES_PATH` and `IX_SCHEMA_PATH` unset and no `~/.ix/filament/modules/` returns a registry with zero archetypes and no error.
- **FR-013-AC-2**: Pointing `IX_SCHEMA_PATH` at a path containing a copy of `spec-artifacts-iso/spec_artifacts_iso/` produces a registry with the 8 ISO archetypes (FR, NFR, StR, US, IT, TC, AC, CON) all loaded and `archetype("fr")` returns Some.
- **FR-013-AC-3**: A manifest entry referencing a missing `schema_ref` path produces a `QuireError::ArchetypeLoadError` listing the bad path; other archetypes in the same module load successfully.
- **FR-013-AC-4**: A criterion bench measures `Registry::load_from(&[corpus_path])` for the full 17-archetype baseline corpus and reports a one-time cost (target: under 50 ms median on baseline hardware).
- **FR-013-AC-5**: After `Registry::load_from(...)`, calling `validate`/`validate_document`/`extract` against a loaded archetype does NOT read from disk (verified via a `tracing` or `strace`-style audit in CI).
- **FR-013-AC-6**: A test confirms `quire-rs` has no `reqwest`, `hyper`, `tonic`, or other network-client crate in its `Cargo.lock` (verified via dependency audit).
- **FR-013-AC-7**: A test creates a symlink loop (`a → b → a`) inside a search path and asserts the loader completes with a warning diagnostic and skips the loop.
- **FR-013-AC-8**: A test sets `IX_SCHEMA_PATH="~/foo:~/foo"` (duplicate canonical path) and asserts modules under `~/foo/` are loaded exactly once.
- **FR-013-AC-9**: A test confirms `Registry: Send + Sync` (compile-time bound assertion via a generic helper function).
- **FR-013-AC-10**: A test sets a search-path entry that points to a regular file (not a directory) and asserts the loader emits a warning and processes remaining entries normally.
- **FR-013-AC-11**: After `Registry::load_from(...)` (or `load_module`), `CompiledArchetype::body_extraction` (field) and `body_extraction()` (accessor) return `Some(ExtractionDsl)` for object types that declared `body_extraction:`, and `None` for archetypes that did not. The returned DSL is the same parsed value validated at load time.
- **FR-013-AC-12**: `Registry::load_module(module_root)` loads exactly the named module (the directory containing `manifest.yaml`) and does NOT walk siblings under `module_root.parent()`. A test places a real module sibling alongside the target and asserts the sibling is not loaded.
- **FR-013-AC-13**: `Registry::load_module(module_root)` against a directory with no `manifest.yaml` returns a registry with zero modules and a single `ArchetypeLoadFailure` describing the absent manifest; sibling directories are not promoted.
- **FR-013-AC-14**: `Diagnostic::PathTraversal { argument, path, reason }` is a defined variant of the (internal) `Diagnostic` enum. A unit test constructs the variant and asserts both human (`Display`) and JSON (`to_json`) renderings carry the variant name, argument, path, and reason discriminator, covering all three `PathTraversalReason` values.
