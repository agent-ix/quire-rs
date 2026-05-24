//! `Registry` — the compiled, immutable, thread-safe lookup of
//! every archetype loaded from disk (FR-013).
//!
//! Construction is via the [`Registry::load_from`] /
//! [`Registry::from_env`] / [`Registry::from_default`] constructors,
//! which delegate to [`crate::loader::load_modules`]. After
//! construction the registry is frozen — there is no `add` method;
//! to swap the active archetype set, build a new `Registry` and drop
//! the previous one. Outstanding `Arc<CompiledArchetype>` clones
//! continue to work for the duration of any in-flight render.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use minijinja::Environment;

use crate::error::{ArchetypeLoadFailure, QuireError};
use crate::loader::compile::CompiledArchetype;
use crate::loader::paths::PathDiagnostic;
use crate::loader::{flatten_into_registry, load_modules, RegistryShape};

/// Compiled, immutable archetype registry. `Send + Sync` and cheap to
/// clone (`Arc<Inner>`).
#[derive(Clone)]
pub struct Registry {
    inner: Arc<Inner>,
}

struct Inner {
    archetypes: std::collections::BTreeMap<String, Arc<CompiledArchetype>>,
    module_paths: std::collections::BTreeMap<String, PathBuf>,
    env: Environment<'static>,
    failures: Vec<ArchetypeLoadFailure>,
    path_diagnostics: Vec<PathDiagnostic>,
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registry")
            .field(
                "modules",
                &self.inner.module_paths.keys().collect::<Vec<_>>(),
            )
            .field(
                "archetypes",
                &self.inner.archetypes.keys().collect::<Vec<_>>(),
            )
            .field("failure_count", &self.inner.failures.len())
            .finish()
    }
}

impl Registry {
    /// Load every module reachable from `paths` (one level deep).
    ///
    /// Empty `paths` falls back to `IX_SCHEMA_PATH` / `~/.ix/schemas/`
    /// (same as [`from_env`](Registry::from_env)).
    pub fn load_from(paths: &[&Path]) -> Result<Self, QuireError> {
        let outcome = load_modules(paths);
        Self::finish(outcome)
    }

    /// Load from `IX_SCHEMA_PATH` (then default `~/.ix/schemas/`).
    pub fn from_env() -> Result<Self, QuireError> {
        let outcome = load_modules(&[]);
        Self::finish(outcome)
    }

    /// Load from `~/.ix/schemas/` only.
    pub fn from_default() -> Result<Self, QuireError> {
        let outcome = crate::loader::load_from_default();
        Self::finish(outcome)
    }

    fn finish(outcome: crate::loader::LoadOutcome) -> Result<Self, QuireError> {
        let RegistryShape {
            archetypes,
            module_paths,
            env,
            failures,
            path_diagnostics,
        } = flatten_into_registry(outcome)?;
        Ok(Self {
            inner: Arc::new(Inner {
                archetypes,
                module_paths,
                env,
                failures,
                path_diagnostics,
            }),
        })
    }

    /// Look up a compiled archetype by name.
    pub fn archetype(&self, name: &str) -> Option<&CompiledArchetype> {
        self.inner.archetypes.get(name).map(|a| a.as_ref())
    }

    /// Iterate over every loaded archetype name.
    pub fn archetype_names(&self) -> impl Iterator<Item = &str> {
        self.inner.archetypes.keys().map(|s| s.as_str())
    }

    /// Iterate over every loaded module name.
    pub fn module_names(&self) -> impl Iterator<Item = &str> {
        self.inner.module_paths.keys().map(|s| s.as_str())
    }

    /// Per-archetype failures collected during load. The Registry can
    /// still be used; consumers choose whether to treat any failure as
    /// fatal (`load_strict`, FR-014).
    pub fn failures(&self) -> &[ArchetypeLoadFailure] {
        &self.inner.failures
    }

    /// Path-resolution diagnostics (missing dirs, file-not-dir,
    /// permission-denied entries). Always advisory.
    pub fn path_diagnostics(&self) -> &[PathDiagnostic] {
        &self.inner.path_diagnostics
    }

    /// Shared MiniJinja environment with every loaded template
    /// registered under `<module>::<archetype>`.
    pub fn env(&self) -> &Environment<'static> {
        &self.inner.env
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn tmpdir(suffix: &str) -> PathBuf {
        let mut p = env::temp_dir();
        p.push(format!(
            "quire-rs-registry-test-{}-{suffix}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).expect("mkdir");
        p
    }

    fn write_minimal_module(root: &Path, name: &str) {
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::create_dir_all(root.join("templates")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            format!(
                "name: {name}\nartifact_types:\n- name: foo\n  template_ref: templates/foo.md.j2\n  frontmatter_schema_ref: schemas/foo.schema.json\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("schemas/foo.schema.json"),
            r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#,
        )
        .unwrap();
        fs::write(root.join("templates/foo.md.j2"), "id: {{ id }}\n").unwrap();
    }

    #[test]
    fn load_from_loads_a_minimal_module() {
        let parent = tmpdir("ok");
        let module_root = parent.join("mod-x");
        fs::create_dir_all(&module_root).unwrap();
        write_minimal_module(&module_root, "mod-x");
        let r = Registry::load_from(&[&parent]).expect("ok");
        assert!(r.archetype("foo").is_some());
        let names: Vec<&str> = r.archetype_names().collect();
        assert_eq!(names, vec!["foo"]);
        let mods: Vec<&str> = r.module_names().collect();
        assert_eq!(mods, vec!["mod-x"]);
    }

    #[test]
    fn registry_is_send_sync_and_cheap_to_clone() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Registry>();
        let parent = tmpdir("clone");
        write_minimal_module(&parent.join("mod-y"), "mod-y");
        let r = Registry::load_from(&[&parent]).expect("ok");
        let r2 = r.clone();
        assert!(r2.archetype("foo").is_some());
    }

    #[test]
    fn module_collision_surfaces_at_load() {
        let p1 = tmpdir("col-1");
        let p2 = tmpdir("col-2");
        write_minimal_module(&p1.join("dup"), "dup");
        write_minimal_module(&p2.join("dup"), "dup");
        let err = Registry::load_from(&[&p1, &p2]).expect_err("collision");
        assert!(matches!(err, QuireError::ModuleCollision { .. }));
    }
}
