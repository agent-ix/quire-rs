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

### Path-handling edge cases

- A search path entry that is a **file, not a directory**: warning diagnostic; entry is skipped, other entries process normally.
- A search path entry whose **read permission is denied**: warning diagnostic with `errno`; entry is skipped.
- **Tilde (`~`) expansion**: the loader SHALL expand a leading `~/` or `~` to the user's home directory before resolving. No mid-path tilde expansion.
- **Environment variables** in `IX_SCHEMA_PATH` entries are NOT expanded — entries are taken literally after splitting on `:`.
- **Duplicate entries** in `IX_SCHEMA_PATH` (same canonical path appearing more than once): the loader SHALL deduplicate before walking; modules are loaded at most once per canonical path.
- **Symlink loops**: the directory walker SHALL maintain a `visited` set of canonical paths and break cycles. Cycle detection emits a warning diagnostic; the cyclic branch is skipped.

### Concurrency model

- `Registry` is `Send + Sync`. Cloning a `Registry` is reference-counted (cheap, `Arc<Inner>`-style) — clones share underlying `CompiledArchetype` instances.
- A `Registry` is immutable after construction. To change the active archetype set, construct a new `Registry` via `load_from` and drop the previous one. Outstanding references to the previous `Registry`'s `CompiledArchetype` keep it alive until the last reference drops.
- `Registry::load_from(...)` SHALL NOT mutate any global state; multiple registries may coexist in the same process.

### File-race assumptions

The loader assumes the upstream sync tool (canonically `ix-cli`, see Appendix A in `spec.md`) writes files **atomically** — i.e. writes to a temp path and renames, never in-place. The loader does NOT acquire file locks. If a non-atomic writer modifies files during load, partial-read errors surface as `QuireError::ArchetypeLoadError` for the affected archetype; the rest of the registry loads normally.

## Acceptance

- **FR-013-AC-1**: `Registry::from_env()` with `IX_SCHEMA_PATH` unset and no `~/.ix/schemas/` returns a registry with zero archetypes and no error.
- **FR-013-AC-2**: Pointing `IX_SCHEMA_PATH` at a path containing a copy of `spec-artifacts-iso/spec_artifacts_iso/` produces a registry with the 8 ISO archetypes (FR, NFR, StR, US, IT, TC, AC, CON) all loaded and `archetype("fr")` returns Some.
- **FR-013-AC-3**: A manifest entry referencing a missing `schema_ref` path produces a `QuireError::ArchetypeLoadError` listing the bad path; other archetypes in the same module load successfully.
- **FR-013-AC-4**: A criterion bench measures `Registry::load_from(&[corpus_path])` for the full 17-archetype baseline corpus and reports a one-time cost (target: under 50 ms median on baseline hardware).
- **FR-013-AC-5**: After `Registry::load_from(...)`, calling `render(archetype, data)` against a loaded archetype does NOT read from disk (verified via a `tracing` or `strace`-style audit in CI).
- **FR-013-AC-6**: A test confirms `quire-rs` has no `reqwest`, `hyper`, `tonic`, or other network-client crate in its `Cargo.lock` (verified via dependency audit).
- **FR-013-AC-7**: A test creates a symlink loop (`a → b → a`) inside a search path and asserts the loader completes with a warning diagnostic and skips the loop.
- **FR-013-AC-8**: A test sets `IX_SCHEMA_PATH="~/foo:~/foo"` (duplicate canonical path) and asserts modules under `~/foo/` are loaded exactly once.
- **FR-013-AC-9**: A test confirms `Registry: Send + Sync` (compile-time bound assertion via a generic helper function).
- **FR-013-AC-10**: A test sets a search-path entry that points to a regular file (not a directory) and asserts the loader emits a warning and processes remaining entries normally.
