---
id: FR-014
title: "Module Activation: Multiple Archetype Modules, Namespaced Coexistence"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL support **multiple archetype modules coexisting in a single `Registry`**. A module is one of:

- A `spec-artifacts-*` directory (renders artifacts)
- A `spec-objects-*` directory (defines object types with `body_extraction` DSLs)
- A `ix-spec-objects`-style directory
- Any other directory matching the manifest convention (FR-013)

When `Registry::load_from(...)` is given a path containing multiple module roots, each module is loaded independently. The registry tracks:

- The set of archetypes contributed by each module
- Per-archetype provenance: which module defined this archetype, at what filesystem path, with what manifest version

### Module identity

A **module** is identified by the `name` field declared in its `manifest.yaml`, NOT by its filesystem path. Two manifests at different filesystem paths but declaring the same `name` collide.

The loader resolves module-name collisions identically to archetype-name collisions (below): first-wins by search-path order, with a `Diagnostic::DuplicateModuleName { name, paths: [a, b] }` listing all contributing paths. `Registry::load_strict(...)` promotes module-name collisions to `QuireError::ModuleCollision`.

If a manifest does NOT declare a `name`, the loader uses the immediate parent directory name as the module name and emits an informational diagnostic recommending an explicit `name` declaration.

### Namespace and collisions

Archetype names are bare strings (`"fr"`, `"adr"`, etc.). When two modules contribute archetypes with the same name, the loader SHALL:

1. Emit a `Diagnostic::DuplicateArchetype { name, modules: [a, b] }` listing all contributing modules.
2. Apply a deterministic resolution policy: **first-wins, ordered by the registry's search path** (FR-013). The first module discovered for a given name takes precedence; later modules are shadowed but their definitions remain queryable via `Registry::archetype_in_module(module, name)` for diagnostics.
3. NOT panic, NOT silently merge.

Consumers who need strict no-collision behavior MAY call `Registry::load_strict(...)` which promotes any duplicate-archetype diagnostic to a `QuireError::ArchetypeCollision`.

### Module versioning

Each module's `manifest.yaml` MAY declare a `version: <semver>` field. The registry surfaces it via:

```rust
pub fn module_version(&self, module: &str) -> Option<&str>;
```

Version is informational at v1 — the engine does NOT perform version-range resolution or activation gating. Consumers that need cross-module version constraints implement them at the manifest authoring layer.

### Activation lifecycle

"Module activation" in this context means "appears in the registry after load." There is no separate activate/deactivate API at v1. To change the active module set, call `Registry::load_from(...)` again with a different path set — the previous registry is dropped.

## Acceptance

- **FR-014-AC-1**: Loading two paths each containing a different `spec-artifacts-*` module produces a registry whose `module_names()` iterator yields both module names in deterministic order (sorted by load order).
- **FR-014-AC-2**: Two modules both defining archetype `"fr"` produces a `Diagnostic::DuplicateArchetype` and `Registry::archetype("fr")` returns the first-loaded module's archetype; `archetype_in_module("module_b", "fr")` returns the shadowed one.
- **FR-014-AC-3**: `Registry::load_strict(...)` with the same input returns `Err(QuireError::ArchetypeCollision { name: "fr", modules })`.
- **FR-014-AC-4**: A module manifest with `version: "0.3.1"` is queryable via `module_version("module-name")` returning `Some("0.3.1")`.
- **FR-014-AC-5**: A test loads `spec-artifacts-iso` + `spec-artifacts-app` + `spec-artifacts-process` simultaneously and asserts the registry contains exactly the union of their archetypes (17 at v1 baseline) with no collisions.
- **FR-014-AC-6**: Two manifests at different paths both declaring `name: foo` produce a `Diagnostic::DuplicateModuleName` and the first-loaded module wins; `load_strict` returns `QuireError::ModuleCollision`.
- **FR-014-AC-7**: A manifest without a `name` field uses its parent directory's basename as the module name and emits an informational diagnostic.
