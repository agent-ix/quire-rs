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
use crate::loader::manifest::{load_manifest, Archetype, Manifest};
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

/// Load a single module directory.
///
/// `module_root` MUST contain `manifest.yaml` directly. Unlike
/// [`load_modules`], this does NOT walk siblings under
/// `module_root.parent()` and does NOT promote to a sibling-search-root
/// — only the named directory is considered a module.
///
/// If `manifest.yaml` is absent, the returned `LoadOutcome` has zero
/// modules and a single failure listing the missing manifest path.
pub fn load_single_module(module_root: &Path) -> LoadOutcome {
    let mut env = build_strict_env();
    let mut modules: Vec<LoadedModule> = Vec::new();
    let mut failures: Vec<ArchetypeLoadFailure> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let canonical = match std::fs::canonicalize(module_root) {
        Ok(p) => p,
        Err(e) => {
            failures.push(ArchetypeLoadFailure {
                module: module_root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("<unknown>")
                    .to_string(),
                archetype: "<manifest>".to_string(),
                path: module_root.to_path_buf(),
                reason: format!("canonicalize: {e}"),
            });
            return LoadOutcome {
                modules,
                failures,
                diagnostics,
                path_diagnostics: Vec::new(),
                env,
            };
        }
    };

    if !canonical_has_manifest(&canonical) {
        failures.push(ArchetypeLoadFailure {
            module: canonical
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<unknown>")
                .to_string(),
            archetype: "<manifest>".to_string(),
            path: canonical.join("manifest.yaml"),
            reason: "manifest.yaml not found in module root".to_string(),
        });
        return LoadOutcome {
            modules,
            failures,
            diagnostics,
            path_diagnostics: Vec::new(),
            env,
        };
    }

    match load_one_module(&canonical, &mut env, &mut diagnostics) {
        Ok((module, mut per_module_failures)) => {
            if !per_module_failures.is_empty() {
                failures.append(&mut per_module_failures);
            }
            modules.push(module);
        }
        Err(fail) => failures.push(fail),
    }

    LoadOutcome {
        modules,
        failures,
        diagnostics,
        path_diagnostics: Vec::new(),
        env,
    }
}

/// Build a `LoadOutcome` from an in-memory module blob — no filesystem
/// access (FR-013 wasm amendment).
///
/// `manifest_yaml` is the raw `manifest.yaml` bytes. `schemas` maps the
/// manifest's relative `frontmatter_schema_ref` strings to schema JSON
/// text; `templates` maps `template_ref` strings to template source.
///
/// Per-archetype failures aggregate like the filesystem loader. Module
/// name is taken from the manifest; if absent it falls back to the
/// sentinel `"<inline>"` and emits a `ManifestMissingName` diagnostic
/// (mirroring FR-014-AC-7's path-derived behavior).
pub fn load_inline_module(
    manifest_yaml: &[u8],
    schemas: &BTreeMap<String, String>,
    templates: &BTreeMap<String, String>,
) -> LoadOutcome {
    use crate::loader::compile::{compile_schema, failure, register_template, CompiledArchetype};
    use crate::loader::manifest::parse_manifest;

    let mut env = build_strict_env();
    let mut modules: Vec<LoadedModule> = Vec::new();
    let mut failures: Vec<ArchetypeLoadFailure> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let inline_root = PathBuf::from("<inline>");

    let manifest = match parse_manifest(manifest_yaml) {
        Ok(m) => m,
        Err(reason) => {
            failures.push(ArchetypeLoadFailure {
                module: "<inline>".to_string(),
                archetype: "<manifest>".to_string(),
                path: inline_root.clone(),
                reason,
            });
            return LoadOutcome {
                modules,
                failures,
                diagnostics,
                path_diagnostics: Vec::new(),
                env,
            };
        }
    };

    let module_name = manifest.name.clone().unwrap_or_else(|| {
        diagnostics.push(Diagnostic::ManifestMissingName {
            path: inline_root.clone(),
            derived_name: "<inline>".to_string(),
        });
        "<inline>".to_string()
    });

    let mut archetypes: Vec<Arc<CompiledArchetype>> = Vec::new();

    'next_archetype: for a in manifest.all_archetypes() {
        // No-compat rule (ADR 0003): retired fields are a hard failure.
        if let Some(field) = a.retired_field() {
            failures.push(failure(
                &module_name,
                &a.name,
                inline_root.clone(),
                format!(
                    "retired field `{field}` is not supported; move its intent into `body_extraction` asserts (FR-033)"
                ),
            ));
            continue;
        }

        // Frontmatter schema (optional, supplied via `schemas`).
        let (frontmatter_schema, frontmatter_validator) = match &a.frontmatter_schema_ref {
            None => (None, None),
            Some(rel) => {
                let schema_ref_str = rel.to_string_lossy().to_string();
                let schema_text = match schemas.get(&schema_ref_str) {
                    Some(s) => s,
                    None => {
                        failures.push(failure(
                            &module_name,
                            &a.name,
                            PathBuf::from(&schema_ref_str),
                            format!("inline schema '{schema_ref_str}' not provided"),
                        ));
                        continue;
                    }
                };
                let raw_schema: Value = match serde_json::from_str(schema_text) {
                    Ok(v) => v,
                    Err(e) => {
                        failures.push(failure(
                            &module_name,
                            &a.name,
                            PathBuf::from(&schema_ref_str),
                            format!("schema is not valid JSON: {e}"),
                        ));
                        continue;
                    }
                };
                let validator = match compile_schema(&raw_schema) {
                    Ok(v) => v,
                    Err(r) => {
                        failures.push(failure(
                            &module_name,
                            &a.name,
                            PathBuf::from(&schema_ref_str),
                            r,
                        ));
                        continue;
                    }
                };
                (Some(Arc::new(raw_schema)), Some(Arc::new(validator)))
            }
        };

        // Data schema (optional, inline in the manifest).
        let (data_schema, data_validator) = match &a.data_schema {
            None => (None, None),
            Some(schema) => match compile_schema(schema) {
                Ok(v) => (Some(Arc::new(schema.clone())), Some(Arc::new(v))),
                Err(r) => {
                    failures.push(failure(&module_name, &a.name, inline_root.clone(), r));
                    continue;
                }
            },
        };

        // Template (optional, supplied via `templates`).
        let (template_path, template_name) = match &a.template_ref {
            None => (None, None),
            Some(rel) => {
                let template_ref_str = rel.to_string_lossy().to_string();
                let template_src = match templates.get(&template_ref_str) {
                    Some(s) => s.clone(),
                    None => {
                        failures.push(failure(
                            &module_name,
                            &a.name,
                            PathBuf::from(&template_ref_str),
                            format!("inline template '{template_ref_str}' not provided"),
                        ));
                        continue;
                    }
                };
                let template_name = format!("{module_name}::{}", a.name);
                if let Err(r) = register_template(&mut env, template_name.clone(), template_src) {
                    failures.push(failure(
                        &module_name,
                        &a.name,
                        PathBuf::from(&template_ref_str),
                        r,
                    ));
                    continue;
                }
                (Some(PathBuf::from(&template_ref_str)), Some(template_name))
            }
        };

        // body_extraction DSL validated at load time.
        if let Some(dsl) = &a.body_extraction {
            if let Err(e) = crate::extract::dsl::validate_dsl(&a.name, dsl) {
                failures.push(failure(
                    &module_name,
                    &a.name,
                    inline_root.clone(),
                    e.to_string(),
                ));
                continue 'next_archetype;
            }
        }

        archetypes.push(Arc::new(finish_compiled(
            &module_name,
            a,
            frontmatter_schema,
            frontmatter_validator,
            data_schema,
            data_validator,
            template_path,
            template_name,
        )));
    }

    modules.push(LoadedModule {
        name: module_name,
        root: inline_root,
        version: manifest.version.clone(),
        archetypes,
    });

    LoadOutcome {
        modules,
        failures,
        diagnostics,
        path_diagnostics: Vec::new(),
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

    for at in manifest.all_archetypes() {
        match compile_archetype(&module_name, module_root, at, env) {
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

/// Compile one unified archetype (FR-031). Resolves the optional
/// frontmatter schema + template + data_schema, validates the
/// `body_extraction` DSL and `assert` facets, and rejects the retired
/// `required_sections`/`variants` fields (no backward-compat layer).
fn compile_archetype(
    module: &str,
    module_root: &Path,
    a: &Archetype,
    env: &mut Environment<'static>,
) -> Result<CompiledArchetype, ArchetypeLoadFailure> {
    // No-compat rule (ADR 0003): retired fields are a hard failure.
    if let Some(field) = a.retired_field() {
        return Err(failure(
            module,
            &a.name,
            module_root.join("manifest.yaml"),
            format!(
                "retired field `{field}` is not supported; move its intent into `body_extraction` asserts (FR-033)"
            ),
        ));
    }

    // Frontmatter schema (optional).
    let (frontmatter_schema, frontmatter_validator) = match &a.frontmatter_schema_ref {
        Some(rel) => {
            let schema_path = module_root.join(rel);
            let raw = read_schema(&schema_path)
                .map_err(|r| failure(module, &a.name, schema_path.clone(), r))?;
            let validator = compile_schema(&raw)
                .map_err(|r| failure(module, &a.name, schema_path.clone(), r))?;
            (Some(Arc::new(raw)), Some(Arc::new(validator)))
        }
        None => (None, None),
    };

    // Data schema (optional).
    let (data_schema, data_validator) = match &a.data_schema {
        Some(schema) => {
            let validator = compile_schema(schema)
                .map_err(|r| failure(module, &a.name, module_root.join("manifest.yaml"), r))?;
            (Some(Arc::new(schema.clone())), Some(Arc::new(validator)))
        }
        None => (None, None),
    };

    // Template (optional/legacy).
    let (template_path, template_name) = match &a.template_ref {
        Some(rel) => {
            let template_path = module_root.join(rel);
            let template_src = std::fs::read_to_string(&template_path)
                .map_err(|e| failure(module, &a.name, template_path.clone(), e.to_string()))?;
            let template_name = qualified_template_name(module, &a.name);
            register_template(env, template_name.clone(), template_src)
                .map_err(|r| failure(module, &a.name, template_path.clone(), r))?;
            (Some(template_path), Some(template_name))
        }
        None => (None, None),
    };

    // body_extraction DSL + assert facets validated at load time
    // (FR-011-AC-6/7/8, FR-033-AC-5).
    if let Some(dsl) = &a.body_extraction {
        crate::extract::dsl::validate_dsl(&a.name, dsl).map_err(|e| {
            failure(
                module,
                &a.name,
                module_root.join("manifest.yaml"),
                e.to_string(),
            )
        })?;
    }

    Ok(finish_compiled(
        module,
        a,
        frontmatter_schema,
        frontmatter_validator,
        data_schema,
        data_validator,
        template_path,
        template_name,
    ))
}

/// Assemble a `CompiledArchetype` from resolved parts, picking the
/// primary `raw_schema`/`validator` (frontmatter when present, else
/// data, else an empty permissive object) for the FR-003 `schema_for`
/// surface and back-compat `validate()` path.
#[allow(clippy::too_many_arguments)]
fn finish_compiled(
    module: &str,
    a: &Archetype,
    frontmatter_schema: Option<Arc<Value>>,
    frontmatter_validator: Option<Arc<jsonschema::JSONSchema>>,
    data_schema: Option<Arc<Value>>,
    data_validator: Option<Arc<jsonschema::JSONSchema>>,
    template_path: Option<PathBuf>,
    template_name: Option<String>,
) -> CompiledArchetype {
    let (raw_schema, validator) = match (&frontmatter_schema, &frontmatter_validator) {
        (Some(s), Some(v)) => (Arc::clone(s), Arc::clone(v)),
        _ => match (&data_schema, &data_validator) {
            (Some(s), Some(v)) => (Arc::clone(s), Arc::clone(v)),
            _ => empty_schema_and_validator(),
        },
    };
    CompiledArchetype {
        name: a.name.clone(),
        module: module.to_string(),
        raw_schema,
        validator,
        frontmatter_schema,
        frontmatter_validator,
        data_schema,
        data_validator,
        template_path,
        template_name,
        body_extraction: a.body_extraction.clone(),
        carry_over: a.carry_over(),
    }
}

/// An empty (permissive) object schema + its compiled validator, used
/// when an archetype declares neither a frontmatter nor a data schema.
fn empty_schema_and_validator() -> (Arc<Value>, Arc<jsonschema::JSONSchema>) {
    let schema = Value::Object(serde_json::Map::new());
    let validator = compile_schema(&schema).expect("empty object schema always compiles");
    (Arc::new(schema), Arc::new(validator))
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

    // FR-013-AC-11: CompiledArchetype.body_extraction surfaces the
    // parsed DSL from the source object_type so downstream extract()
    // callers don't have to re-read manifest.yaml.
    #[test]
    fn compiled_archetype_exposes_body_extraction_dsl() {
        let parent = tmpdir("bx");
        let module_root = parent.join("bx-mod");
        fs::create_dir_all(&module_root).unwrap();
        fs::write(
            module_root.join("manifest.yaml"),
            r#"
name: bx-mod
object_types:
- name: with_dsl
  body_extraction:
    yield_pattern:
      match:
        title:
          from: heading
- name: without_dsl
"#,
        )
        .unwrap();
        let outcome = load_modules(&[&parent]);
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        let module = outcome
            .modules
            .iter()
            .find(|m| m.name == "bx-mod")
            .expect("loaded");
        let with_dsl = module
            .archetypes
            .iter()
            .find(|a| a.name == "with_dsl")
            .expect("with_dsl loaded");
        let without_dsl = module
            .archetypes
            .iter()
            .find(|a| a.name == "without_dsl")
            .expect("without_dsl loaded");
        assert!(with_dsl.body_extraction.is_some());
        assert!(with_dsl.body_extraction().is_some());
        assert!(without_dsl.body_extraction.is_none());
        assert!(without_dsl.body_extraction().is_none());
        let dsl = with_dsl.body_extraction().unwrap();
        assert!(dsl.yield_pattern.r#match.is_some());
        assert!(dsl.yield_pattern.iterate_over.is_none());
    }

    // ── Task 037: unified archetype shape (FR-031) ──────────────────

    // TC-522 (FR-031-AC-1): template_ref + frontmatter_schema_ref +
    // body_extraction compiles to one CompiledArchetype, renderable,
    // resolvable body contract.
    #[test]
    fn tc522_unified_renderable_archetype_with_body_extraction() {
        let parent = tmpdir("u-522");
        let root = parent.join("u-mod");
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::create_dir_all(root.join("templates")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
artifact_types:
- name: FR
  template_ref: templates/fr.md.j2
  frontmatter_schema_ref: schemas/fr.schema.json
  body_extraction:
    yield_pattern:
      match:
        purpose:
          from: section_body
          after_heading: Purpose
"#,
        )
        .unwrap();
        fs::write(
            root.join("schemas/fr.schema.json"),
            r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#,
        )
        .unwrap();
        fs::write(root.join("templates/fr.md.j2"), "id: {{ id }}\n").unwrap();
        let outcome = load_modules(&[&parent]);
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        let arch = &outcome.modules[0].archetypes[0];
        assert_eq!(arch.name, "FR");
        assert!(arch.is_renderable());
        assert!(arch.is_validatable());
        assert!(arch.body_extraction().is_some());
        assert!(arch.frontmatter_validator().is_some());
    }

    // TC-523 (FR-031-AC-2): body_extraction but no template_ref →
    // compiles, not renderable, still validatable + extractable.
    #[test]
    fn tc523_unified_archetype_without_template_not_renderable() {
        let parent = tmpdir("u-523");
        let root = parent.join("u-mod");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
object_types:
- name: domain
  data_schema:
    type: object
  body_extraction:
    yield_pattern:
      match:
        title:
          from: heading
"#,
        )
        .unwrap();
        let outcome = load_modules(&[&parent]);
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        let arch = &outcome.modules[0].archetypes[0];
        assert!(!arch.is_renderable());
        assert!(arch.is_validatable());
        assert!(arch.body_extraction().is_some());
    }

    // TC-524 (FR-031-AC-3): carry-over fields retained + readable.
    #[test]
    fn tc524_carry_over_fields_retained() {
        let parent = tmpdir("u-524");
        let root = parent.join("u-mod");
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::create_dir_all(root.join("templates")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
artifact_types:
- name: FR
  template_ref: templates/fr.md.j2
  frontmatter_schema_ref: schemas/fr.schema.json
  grammar_ref: iso-spec-core
  has_plugin: true
  allowed_links: [implements, refines]
  defaults:
    id_pattern: "FR-{next:03d}"
"#,
        )
        .unwrap();
        fs::write(root.join("schemas/fr.schema.json"), r#"{"type":"object"}"#).unwrap();
        fs::write(root.join("templates/fr.md.j2"), "x\n").unwrap();
        let outcome = load_modules(&[&parent]);
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        let arch = &outcome.modules[0].archetypes[0];
        assert_eq!(arch.id_pattern(), Some("FR-{next:03d}"));
        assert_eq!(arch.grammar_ref(), Some("iso-spec-core"));
        assert!(arch.has_plugin());
        assert_eq!(
            arch.allowed_links(),
            &["implements".to_string(), "refines".to_string()]
        );
    }

    // TC-525 (FR-031-AC-4): frontmatter_schema_ref + data_schema are
    // two distinct compiled validators, neither collapsed.
    #[test]
    fn tc525_frontmatter_and_data_schemas_are_distinct() {
        let parent = tmpdir("u-525");
        let root = parent.join("u-mod");
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::create_dir_all(root.join("templates")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
artifact_types:
- name: FR
  template_ref: templates/fr.md.j2
  frontmatter_schema_ref: schemas/fr.schema.json
  data_schema:
    type: object
    required: [extracted_id]
    properties:
      extracted_id: { type: string }
"#,
        )
        .unwrap();
        fs::write(
            root.join("schemas/fr.schema.json"),
            r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#,
        )
        .unwrap();
        fs::write(root.join("templates/fr.md.j2"), "x\n").unwrap();
        let outcome = load_modules(&[&parent]);
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        let arch = &outcome.modules[0].archetypes[0];
        // Frontmatter validator requires `id`.
        let fv = arch.frontmatter_validator().expect("frontmatter validator");
        assert!(fv.is_valid(&serde_json::json!({"id": "FR-1"})));
        assert!(!fv.is_valid(&serde_json::json!({"other": 1})));
        // Data validator requires `extracted_id` — distinct schema.
        let dv = arch.data_validator().expect("data validator");
        assert!(dv.is_valid(&serde_json::json!({"extracted_id": "x"})));
        assert!(!dv.is_valid(&serde_json::json!({"id": "FR-1"})));
    }

    // TC-526 (FR-031-AC-5): required_sections is rejected as a hard
    // ArchetypeLoadFailure (no-compat rule overrides FR-031-AC-5's
    // softer "non-fatal diagnostic" — see CR note in task 037).
    #[test]
    fn tc526_required_sections_is_hard_load_failure() {
        let parent = tmpdir("u-526");
        let root = parent.join("u-mod");
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::create_dir_all(root.join("templates")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
artifact_types:
- name: FR
  template_ref: templates/fr.md.j2
  frontmatter_schema_ref: schemas/fr.schema.json
  required_sections:
  - Description
  - Specification
"#,
        )
        .unwrap();
        fs::write(root.join("schemas/fr.schema.json"), r#"{"type":"object"}"#).unwrap();
        fs::write(root.join("templates/fr.md.j2"), "x\n").unwrap();
        let outcome = load_modules(&[&parent]);
        assert_eq!(outcome.modules[0].archetypes.len(), 0);
        assert_eq!(outcome.failures.len(), 1);
        let f = &outcome.failures[0];
        assert_eq!(f.archetype, "FR");
        assert!(
            f.reason.contains("required_sections") && f.reason.contains("body_extraction"),
            "got: {}",
            f.reason
        );
    }

    // TC-526b: `variants` is likewise a hard failure (no-compat rule).
    #[test]
    fn tc526_variants_is_hard_load_failure() {
        let parent = tmpdir("u-526b");
        let root = parent.join("u-mod");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
object_types:
- name: domain
  variants:
  - selector: kind
    value: a
"#,
        )
        .unwrap();
        let outcome = load_modules(&[&parent]);
        assert_eq!(outcome.modules[0].archetypes.len(), 0);
        assert_eq!(outcome.failures.len(), 1);
        assert!(outcome.failures[0].reason.contains("variants"));
    }

    // TC-527 (FR-031-AC-6): Registry::archetype resolves unified
    // archetype identically (same keying + first-wins).
    #[test]
    fn tc527_registry_resolves_unified_archetype() {
        let parent = tmpdir("u-527");
        let root = parent.join("u-mod");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
object_types:
- name: domain
  body_extraction:
    yield_pattern:
      match:
        title:
          from: heading
"#,
        )
        .unwrap();
        let r = crate::Registry::load_from(&[&parent]).expect("ok");
        let arch = r.archetype("domain").expect("resolved");
        assert_eq!(arch.name, "domain");
        assert!(arch.body_extraction().is_some());
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
