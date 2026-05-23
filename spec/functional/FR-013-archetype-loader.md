---
id: FR-013
title: "Archetype Loader: Filesystem-First, Sync-Agnostic"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-001"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL load archetypes from the **local filesystem**. The engine has no network calls, no Filament API client, and no required runtime services. Whatever populates the local directory tree (Filament sync, hand-authoring, git checkout, an unzipped distribution tarball) is outside the engine's concern.

### Search path

The engine resolves archetype roots from, in priority order:

1. Explicit `Registry::load_from(paths: &[&Path])` constructor argument.
2. `IX_SCHEMA_PATH` environment variable — colon-separated list of directories (PATH-style).
3. Default: `~/.ix/schemas/`.

Each path entry is treated as a directory containing one or more **module roots**. The engine walks one level deep to discover modules.

### Module layout (convention)

Each module root SHALL match the existing `spec-artifacts-*` shape:

```
<module-root>/
├── manifest.yaml           # artifact_types: [...] (and/or object_types: [...])
├── schemas/
│   └── <name>-frontmatter.schema.json
└── templates/
    └── <name>.md.j2
```

The `manifest.yaml` is the authoritative entry point. It enumerates archetypes; each entry references its schema and template by relative path (`schema_ref`, `template_ref`). The loader does NOT auto-discover schemas/templates from the filesystem layout — every archetype is explicitly declared in a manifest so authoring is intentional.

### Public API

```rust
pub struct Registry { /* ... */ }

impl Registry {
    pub fn load_from(paths: &[&Path]) -> Result<Registry, QuireError>;
    pub fn from_env() -> Result<Registry, QuireError>;        // honors IX_SCHEMA_PATH then default
    pub fn from_default() -> Result<Registry, QuireError>;    // ~/.ix/schemas/ only

    pub fn archetype(&self, name: &str) -> Option<&CompiledArchetype>;
    pub fn archetype_names(&self) -> impl Iterator<Item = &str>;
    pub fn module_names(&self) -> impl Iterator<Item = &str>;
}
```

### Load-time work (amortized)

For each archetype the loader SHALL:

1. Parse the JSON Schema document and compile it into a runtime validator (e.g. `jsonschema::JSONSchema::compile`).
2. Parse the MiniJinja template and register it with the long-lived `Environment` (FR-004).
3. Cache the (validator, template, metadata) tuple as a `CompiledArchetype` keyed by archetype name.

Per-render and per-extract operations SHALL NOT re-read disk and SHALL NOT re-parse schemas or templates. Load cost is amortized over the process lifetime (see NFR-007).

### Error model

- A manifest entry whose `schema_ref` or `template_ref` does not exist on disk: `QuireError::ArchetypeLoadError` with file path and reason. The loader continues with other archetypes and aggregates errors.
- A schema document that is itself malformed JSON Schema: `QuireError::ArchetypeLoadError` with schema path and parse error.
- A template that fails MiniJinja parse: `QuireError::ArchetypeLoadError` with template path and parse error.
- A missing search path entry on disk: warning diagnostic (not a fatal error) — empty modules are valid.

## Acceptance

- **FR-013-AC-1**: `Registry::from_env()` with `IX_SCHEMA_PATH` unset and no `~/.ix/schemas/` returns a registry with zero archetypes and no error.
- **FR-013-AC-2**: Pointing `IX_SCHEMA_PATH` at a path containing a copy of `spec-artifacts-iso/spec_artifacts_iso/` produces a registry with the 8 ISO archetypes (FR, NFR, StR, US, IT, TC, AC, CON) all loaded and `archetype("fr")` returns Some.
- **FR-013-AC-3**: A manifest entry referencing a missing `schema_ref` path produces a `QuireError::ArchetypeLoadError` listing the bad path; other archetypes in the same module load successfully.
- **FR-013-AC-4**: A criterion bench measures `Registry::load_from(&[corpus_path])` for the full 17-archetype baseline corpus and reports a one-time cost (target: under 50 ms median on baseline hardware).
- **FR-013-AC-5**: After `Registry::load_from(...)`, calling `render(archetype, data)` against a loaded archetype does NOT read from disk (verified via a `tracing` or `strace`-style audit in CI).
- **FR-013-AC-6**: A test confirms `quire-rs` has no `reqwest`, `hyper`, `tonic`, or other network-client crate in its `Cargo.lock` (verified via dependency audit).
