//! Filesystem-first archetype loader (FR-013, Task 005).
//!
//! Walks each resolved search-path entry one level deep, looking for
//! module roots (sub-directories containing a `manifest.yaml`). For
//! each module, parses the manifest and compiles every declared
//! archetype (schema + optional template) into a [`CompiledArchetype`]
//! that the [`Registry`](crate::registry::Registry) then exposes to
//! render/extract consumers.
//!
//! Failures don't abort the load: per-archetype errors are aggregated
//! into [`QuireError::ArchetypeLoadError`], the rest of the registry
//! loads normally, and consumers decide whether to treat the failures
//! as fatal (`load_strict`, FR-014).

pub mod compile;
pub mod manifest;
pub mod paths;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use minijinja::Environment;
use serde_json::Value;

use crate::diagnostic::Diagnostic;
use crate::error::{ArchetypeLoadFailure, QuireError};
use crate::loader::compile::{
    compile_schema, failure, read_schema, register_template, CompiledArchetype,
};
use crate::loader::manifest::{load_manifest, ArtifactType, Manifest, ObjectType};
use crate::loader::paths::{home_dir, resolve_search_paths, PathDiagnostic};

/// Module-level entry produced by [`load_modules`].
#[derive(Debug)]
pub struct LoadedModule {
    pub name: String,
    pub root: PathBuf,
    pub version: Option<String>,
    pub archetypes: Vec<Arc<CompiledArchetype>>,
}

/// Outcome of a full load pass.
#[derive(Debug)]
pub struct LoadOutcome {
    pub modules: Vec<LoadedModule>,
    pub failures: Vec<ArchetypeLoadFailure>,
    pub diagnostics: Vec<Diagnostic>,
    pub path_diagnostics: Vec<PathDiagnostic>,
    pub env: Environment<'static>,
}

/// Build a strict MiniJinja environment per FR-004. Delegates to
/// `render::env::build_strict_env` so the env construction has one
/// owner.
fn build_strict_env() -> Environment<'static> {
    crate::render::env::build_strict_env()
}

/// Load every module reachable from `explicit` (or `IX_SCHEMA_PATH` /
/// `~/.ix/schemas` when `explicit` is empty).
pub fn load_modules(explicit: &[&Path]) -> LoadOutcome {
    #[cfg(feature = "tracing")]
    let _span = tracing::info_span!("quire_rs::load", paths = explicit.len()).entered();
    let env_value = std::env::var_os("IX_SCHEMA_PATH");
    let path_diagnostics = resolve_search_paths(explicit, env_value);

    let mut env = build_strict_env();
    let mut modules: Vec<LoadedModule> = Vec::new();
    let mut failures: Vec<ArchetypeLoadFailure> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut visited: Vec<PathBuf> = Vec::new();

    // Surface path-resolution problems as diagnostics too (they were
    // previously only accessible via Registry::path_diagnostics; both
    // channels now carry the information for consumer ergonomics).
    for diag in &path_diagnostics {
        match diag {
            PathDiagnostic::Missing(p) => {
                diagnostics.push(Diagnostic::SearchPathMissing { path: p.clone() })
            }
            PathDiagnostic::NotADirectory(p) => {
                diagnostics.push(Diagnostic::SearchPathNotADirectory { path: p.clone() })
            }
            PathDiagnostic::Unreadable { path, reason } => {
                diagnostics.push(Diagnostic::SearchPathUnreadable {
                    path: path.clone(),
                    reason: reason.clone(),
                })
            }
            PathDiagnostic::Ok(_) => {}
        }
    }

    for diag in &path_diagnostics {
        if let PathDiagnostic::Ok(root) = diag {
            walk_search_root(
                root,
                &mut env,
                &mut modules,
                &mut failures,
                &mut diagnostics,
                &mut visited,
            );
        }
    }

    LoadOutcome {
        modules,
        failures,
        diagnostics,
        path_diagnostics,
        env,
    }
}

/// Convenience: same as [`load_modules`] but uses only the
/// `IX_SCHEMA_PATH` env var (or the default `~/.ix/schemas/`).
pub fn load_from_env() -> LoadOutcome {
    load_modules(&[])
}

/// Convenience: load only from `~/.ix/schemas/`, ignoring
/// `IX_SCHEMA_PATH`.
pub fn load_from_default() -> LoadOutcome {
    let default_root = home_dir().map(|h| h.join(".ix").join("schemas"));
    match default_root {
        Some(root) => load_modules(&[&root]),
        None => LoadOutcome {
            modules: Vec::new(),
            failures: Vec::new(),
            diagnostics: Vec::new(),
            path_diagnostics: Vec::new(),
            env: build_strict_env(),
        },
    }
}

/// Walk one search-path root looking for module sub-directories
/// (each containing a `manifest.yaml`).
fn walk_search_root(
    root: &Path,
    env: &mut Environment<'static>,
    modules: &mut Vec<LoadedModule>,
    failures: &mut Vec<ArchetypeLoadFailure>,
    diagnostics: &mut Vec<Diagnostic>,
    visited: &mut Vec<PathBuf>,
) {
    let canonical = match std::fs::canonicalize(root) {
        Ok(p) => p,
        Err(_) => return,
    };
    if visited.iter().any(|p| p == &canonical) {
        diagnostics.push(Diagnostic::SymlinkLoop { path: canonical });
        return; // FR-013-AC-7: symlink-loop guard
    }
    visited.push(canonical.clone());

    let entries = match std::fs::read_dir(&canonical) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let candidate = entry.path();
        if !candidate.is_dir() {
            continue;
        }
        let canon = match std::fs::canonicalize(&candidate) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if visited.iter().any(|p| p == &canon) {
            diagnostics.push(Diagnostic::SymlinkLoop { path: canon });
            continue; // already loaded via another search path
        }
        visited.push(canon.clone());

        if !canonical_has_manifest(&canon) {
            continue; // not a module root
        }
        match load_one_module(&canon, env, diagnostics) {
            Ok((module, mut per_module_failures)) => {
                if !per_module_failures.is_empty() {
                    failures.append(&mut per_module_failures);
                }
                modules.push(module);
            }
            Err(fail) => failures.push(fail),
        }
    }
}

fn canonical_has_manifest(path: &Path) -> bool {
    path.join("manifest.yaml").is_file()
}

/// Load a single module directory: parse manifest, compile every
/// declared archetype. Per-archetype failures don't abort the module —
/// they're returned alongside the partial module so the loader can
/// aggregate them.
fn load_one_module(
    module_root: &Path,
    env: &mut Environment<'static>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(LoadedModule, Vec<ArchetypeLoadFailure>), ArchetypeLoadFailure> {
    let manifest: Manifest = load_manifest(module_root).map_err(|reason| ArchetypeLoadFailure {
        module: module_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_string(),
        archetype: "<manifest>".to_string(),
        path: module_root.join("manifest.yaml"),
        reason,
    })?;

    let module_name: String = manifest.name.clone().unwrap_or_else(|| {
        let derived = module_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_string();
        diagnostics.push(Diagnostic::ManifestMissingName {
            path: module_root.join("manifest.yaml"),
            derived_name: derived.clone(),
        });
        derived
    });

    let mut archetypes: Vec<Arc<CompiledArchetype>> = Vec::new();
    let mut failures: Vec<ArchetypeLoadFailure> = Vec::new();

    for at in &manifest.artifact_types {
        match compile_artifact_type(&module_name, module_root, at, env) {
            Ok(c) => archetypes.push(Arc::new(c)),
            Err(f) => failures.push(f),
        }
    }
    for ot in &manifest.object_types {
        match compile_object_type(&module_name, module_root, ot) {
            Ok(c) => archetypes.push(Arc::new(c)),
            Err(f) => failures.push(f),
        }
    }

    Ok((
        LoadedModule {
            name: module_name,
            root: module_root.to_path_buf(),
            version: manifest.version.clone(),
            archetypes,
        },
        failures,
    ))
}

fn compile_artifact_type(
    module: &str,
    module_root: &Path,
    at: &ArtifactType,
    env: &mut Environment<'static>,
) -> Result<CompiledArchetype, ArchetypeLoadFailure> {
    let schema_path = module_root.join(&at.frontmatter_schema_ref);
    let template_path = module_root.join(&at.template_ref);

    let raw_schema =
        read_schema(&schema_path).map_err(|r| failure(module, &at.name, schema_path.clone(), r))?;
    let validator = compile_schema(&raw_schema)
        .map_err(|r| failure(module, &at.name, schema_path.clone(), r))?;

    let template_src = std::fs::read_to_string(&template_path)
        .map_err(|e| failure(module, &at.name, template_path.clone(), e.to_string()))?;
    let template_name = qualified_template_name(module, &at.name);
    register_template(env, template_name.clone(), template_src)
        .map_err(|r| failure(module, &at.name, template_path.clone(), r))?;

    Ok(CompiledArchetype {
        name: at.name.clone(),
        module: module.to_string(),
        raw_schema: Arc::new(raw_schema),
        validator: Arc::new(validator),
        template_path: Some(template_path),
        template_name: Some(template_name),
    })
}

fn compile_object_type(
    module: &str,
    module_root: &Path,
    ot: &ObjectType,
) -> Result<CompiledArchetype, ArchetypeLoadFailure> {
    let schema: Value = ot.data_schema.clone().unwrap_or_else(|| {
        // Empty schema is permissive — matches Py reference behavior
        // where an object_type with no data_schema accepts anything.
        Value::Object(serde_json::Map::new())
    });
    let validator = compile_schema(&schema)
        .map_err(|r| failure(module, &ot.name, module_root.to_path_buf(), r))?;
    // FR-011-AC-6/7/8: validate the body_extraction DSL at load time
    // when present. Authoring tools see structural errors immediately,
    // not when `extract()` runs.
    if let Some(dsl) = &ot.body_extraction {
        crate::extract::dsl::validate_dsl(&ot.name, dsl).map_err(|e| {
            failure(
                module,
                &ot.name,
                module_root.join("manifest.yaml"),
                e.to_string(),
            )
        })?;
    }
    Ok(CompiledArchetype {
        name: ot.name.clone(),
        module: module.to_string(),
        raw_schema: Arc::new(schema),
        validator: Arc::new(validator),
        template_path: None,
        template_name: None,
    })
}

/// Templates are registered under `<module>::<archetype>` so two
/// modules can declare the same archetype name without colliding in
/// the shared MiniJinja env.
fn qualified_template_name(module: &str, archetype: &str) -> String {
    format!("{module}::{archetype}")
}

/// Aggregate per-module load results into a single
/// `BTreeMap<name, Arc<CompiledArchetype>>` keyed by archetype name,
/// surfacing collisions as a fatal `ArchetypeCollision` error
/// (FR-014).
pub fn flatten_into_registry(mut outcome: LoadOutcome) -> RegistryShape {
    let mut module_paths: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut module_versions: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut module_collisions: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for module in &outcome.modules {
        match module_paths.get(&module.name) {
            Some(_) => {
                module_collisions
                    .entry(module.name.clone())
                    .or_default()
                    .push(module.root.clone());
            }
            None => {
                module_paths.insert(module.name.clone(), module.root.clone());
                module_versions.insert(module.name.clone(), module.version.clone());
            }
        }
    }
    for (name, mut later_paths) in module_collisions {
        let first = module_paths.get(&name).cloned().unwrap_or_default();
        let mut all_paths = vec![first];
        all_paths.append(&mut later_paths);
        outcome.diagnostics.push(Diagnostic::DuplicateModuleName {
            name,
            paths: all_paths,
        });
    }

    let mut active_archetypes: BTreeMap<String, Arc<CompiledArchetype>> = BTreeMap::new();
    let mut by_module_and_name: BTreeMap<(String, String), Arc<CompiledArchetype>> =
        BTreeMap::new();
    let mut arch_collisions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for module in &outcome.modules {
        for arch in &module.archetypes {
            by_module_and_name.insert((module.name.clone(), arch.name.clone()), Arc::clone(arch));
            match active_archetypes.get(&arch.name) {
                Some(existing) => {
                    let entry = arch_collisions.entry(arch.name.clone()).or_default();
                    if !entry.contains(&existing.module) {
                        entry.push(existing.module.clone());
                    }
                    entry.push(module.name.clone());
                }
                None => {
                    active_archetypes.insert(arch.name.clone(), Arc::clone(arch));
                }
            }
        }
    }
    for (name, modules) in arch_collisions {
        outcome
            .diagnostics
            .push(Diagnostic::DuplicateArchetype { name, modules });
    }

    RegistryShape {
        archetypes: active_archetypes,
        by_module_and_name,
        module_paths,
        module_versions,
        env: outcome.env,
        failures: outcome.failures,
        diagnostics: outcome.diagnostics,
        path_diagnostics: outcome.path_diagnostics,
    }
}

/// Strict counterpart of [`flatten_into_registry`]: promotes the first
/// collision diagnostic to a fatal `QuireError`.
pub fn flatten_into_registry_strict(outcome: LoadOutcome) -> Result<RegistryShape, QuireError> {
    let shape = flatten_into_registry(outcome);
    for diag in shape.diagnostics.iter() {
        match diag {
            Diagnostic::DuplicateModuleName { name, paths } => {
                let first = paths.first().cloned().unwrap_or_default();
                let second = paths.get(1).cloned().unwrap_or_default();
                return Err(QuireError::ModuleCollision {
                    name: name.clone(),
                    first_path: first,
                    second_path: second,
                });
            }
            Diagnostic::DuplicateArchetype { name, modules } => {
                let first_module = modules.first().cloned().unwrap_or_default();
                let second_module = modules.get(1).cloned().unwrap_or_default();
                return Err(QuireError::ArchetypeCollision {
                    name: name.clone(),
                    first_module,
                    second_module,
                });
            }
            _ => {}
        }
    }
    Ok(shape)
}

/// Pre-Registry shape returned by [`flatten_into_registry`]. The
/// `Registry` constructor wraps this in `Arc<Inner>` for cheap cloning.
#[derive(Debug)]
pub struct RegistryShape {
    /// First-wins active archetype set keyed by bare archetype name.
    pub archetypes: BTreeMap<String, Arc<CompiledArchetype>>,
    /// Every (module, archetype) pair — includes shadowed copies for
    /// inspection via `Registry::archetype_in_module`.
    pub by_module_and_name: BTreeMap<(String, String), Arc<CompiledArchetype>>,
    pub module_paths: BTreeMap<String, PathBuf>,
    pub module_versions: BTreeMap<String, Option<String>>,
    pub env: Environment<'static>,
    pub failures: Vec<ArchetypeLoadFailure>,
    pub diagnostics: Vec<Diagnostic>,
    pub path_diagnostics: Vec<PathDiagnostic>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmpdir(suffix: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "quire-rs-loader-test-{}-{suffix}",
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
    fn loads_minimal_module_from_explicit_path() {
        let parent = tmpdir("min");
        let module_root = parent.join("mod-a");
        fs::create_dir_all(&module_root).unwrap();
        write_minimal_module(&module_root, "mod-a");
        let outcome = load_modules(&[&parent]);
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        assert_eq!(outcome.modules.len(), 1);
        assert_eq!(outcome.modules[0].archetypes.len(), 1);
        assert_eq!(outcome.modules[0].archetypes[0].name, "foo");
    }

    #[test]
    fn aggregates_failure_for_missing_schema() {
        let parent = tmpdir("fail");
        let module_root = parent.join("mod-b");
        fs::create_dir_all(&module_root).unwrap();
        fs::write(
            module_root.join("manifest.yaml"),
            "name: mod-b\nartifact_types:\n- name: foo\n  template_ref: t/foo.j2\n  frontmatter_schema_ref: s/missing.json\n",
        )
        .unwrap();
        let outcome = load_modules(&[&parent]);
        assert_eq!(outcome.failures.len(), 1);
        // FR-013-AC-3: continues — module loads (with zero archetypes).
        assert_eq!(outcome.modules.len(), 1);
        assert!(outcome.modules[0].archetypes.is_empty());
    }

    #[test]
    fn module_collision_emits_diagnostic_but_loads_first_wins() {
        let p1 = tmpdir("col-a");
        let p2 = tmpdir("col-b");
        write_minimal_module(&p1.join("dup"), "dup");
        fs::create_dir_all(p2.join("dup")).unwrap();
        write_minimal_module(&p2.join("dup"), "dup");
        let outcome = load_modules(&[&p1, &p2]);
        let shape = flatten_into_registry(outcome);
        assert!(shape
            .diagnostics
            .iter()
            .any(|d| matches!(d, Diagnostic::DuplicateModuleName { .. })));
        // First-wins: the bare name "dup" still resolves.
        assert!(shape.archetypes.contains_key("foo"));
    }

    #[test]
    fn module_collision_is_fatal_in_strict_mode() {
        let p1 = tmpdir("strict-a");
        let p2 = tmpdir("strict-b");
        write_minimal_module(&p1.join("dup"), "dup");
        write_minimal_module(&p2.join("dup"), "dup");
        let outcome = load_modules(&[&p1, &p2]);
        let err = flatten_into_registry_strict(outcome).expect_err("collision");
        assert!(matches!(err, QuireError::ModuleCollision { .. }));
    }

    #[test]
    fn registry_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CompiledArchetype>();
    }

    // FR-013-AC-7: symlink loop is broken without panic.
    #[test]
    #[cfg(unix)]
    fn symlink_loop_does_not_panic_or_recurse() {
        let parent = tmpdir("loop");
        let mod_a = parent.join("a");
        let mod_b = parent.join("b");
        write_minimal_module(&mod_a, "a");
        write_minimal_module(&mod_b, "b");
        // Add a → b/back symlink and b → a/back symlink (mutual loop).
        let _ = std::os::unix::fs::symlink(&mod_b, mod_a.join("back-to-b"));
        let _ = std::os::unix::fs::symlink(&mod_a, mod_b.join("back-to-a"));
        let outcome = load_modules(&[&parent]);
        // We finish at all — the visited set breaks the loop.
        assert!(outcome.modules.len() >= 2);
    }
}
