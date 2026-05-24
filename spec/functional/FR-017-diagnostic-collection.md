---
id: FR-017
title: "Diagnostic Collection API"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-014"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-015"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

`quire-rs` emits `Diagnostic` values from multiple paths (loader, module activation, edge harvesting, fallback locators, DSL evaluation). These are non-fatal informational messages distinct from `QuireError`. Consumers SHALL have an explicit API to collect, filter, and inspect them.

### Diagnostic type

```rust
#[non_exhaustive]
pub enum Diagnostic {
    DuplicateArchetype { name: String, modules: Vec<String> },
    DuplicateModuleName { name: String, paths: Vec<PathBuf> },
    DuplicateEdge { source: String, type_: String, target: String, sources: Vec<String> },
    UnresolvedRelationshipTarget { source: String, bare_id: String },
    FallbackLocatorUsed { key: String, position: usize, locator_repr: String },
    IterateRootMissing { path: Vec<String> },
    SymlinkLoop { canonical: PathBuf },
    PathIsFile { path: PathBuf },
    PermissionDenied { path: PathBuf, errno: String },
    ModuleNameDefaulted { module: String, parent_dir: String },
    DslExtraIgnored { /* fields */ },
    // ... others as added
}

pub struct Diagnostics(pub Vec<Diagnostic>);

impl Diagnostics {
    pub fn is_empty(&self) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic>;
    pub fn filter<F>(&self, f: F) -> impl Iterator<Item = &Diagnostic>;
    pub fn by_kind(&self, discriminant: DiagnosticKind) -> impl Iterator<Item = &Diagnostic>;
}
```

### Collection surfaces

Diagnostics SHALL be surfaced from three points:

1. **`Registry` construction**: `Registry::load_from(...)` returns `Result<Registry, QuireError>` AND accumulates load-time diagnostics. Accessor: `registry.load_diagnostics() -> &Diagnostics`.
2. **Per-call results**: `ExtractionResult` (FR-011) and `EdgeHarvest` (FR-015) carry `diagnostics: Vec<Diagnostic>` fields per their existing definitions; no change.
3. **Render and apply_patch**: return `Result<RenderOutput, QuireError>` where `RenderOutput { markdown: String, diagnostics: Diagnostics }` — diagnostics is usually empty but may contain template-side notes from future features.

### Determinism

Diagnostics SHALL be emitted in deterministic order: by source-of-emission (load order → call order → DSL declaration order). NFR-006 applies — identical input produces identical diagnostic sequences.

### No diagnostic dedup

The engine does NOT dedup diagnostics. If the same `DuplicateArchetype { name: "fr", modules: [a, b] }` would be emitted twice from two evaluation paths, both appear. Consumers MAY dedupe at the display layer.

## Acceptance

- **FR-017-AC-1**: `Diagnostic` enum is `#[non_exhaustive]`, `Send + Sync`, `Debug`, `Clone`, `PartialEq`, `Eq`.
- **FR-017-AC-2**: `Registry::load_from(...)` with a search path containing two modules declaring the same archetype name produces a registry AND `registry.load_diagnostics()` returns a `Diagnostics` containing one `DuplicateArchetype` entry.
- **FR-017-AC-3**: A test calls `render` on an archetype with no diagnostic-producing path; `RenderOutput.diagnostics.is_empty()` is true.
- **FR-017-AC-4**: An integration test loads a corpus with intentional collisions + missing schema_ref + symlink loop, and asserts `load_diagnostics()` contains exactly the expected diagnostics in deterministic order.
- **FR-017-AC-5**: `Diagnostics::by_kind` filters by discriminant tag (e.g. all `DuplicateEdge` variants) without enumerating other kinds.
