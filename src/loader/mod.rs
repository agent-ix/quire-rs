//! Filesystem-first archetype loader (FR-013, Task 005).
//!
//! Walks each resolved search-path entry one level deep, looking for
//! module roots (sub-directories containing a `manifest.yaml`). For
//! each module, parses the manifest and compiles every declared
//! archetype (schema only — render/templating is removed) into a
//! [`CompiledArchetype`] that the
//! [`Registry`](crate::registry::Registry) then exposes to
//! validate/extract consumers.
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

use serde_json::Value;

use crate::diagnostic::Diagnostic;
use crate::error::{ArchetypeLoadFailure, QuireError};
use crate::loader::compile::{compile_schema, failure, read_schema, CompiledArchetype};
use crate::loader::manifest::{load_manifest, Archetype, Manifest};
use crate::loader::paths::{
    default_module_root, module_path_env, resolve_search_paths, PathDiagnostic,
};
use crate::vocab::{EdgeTypeDef, LexiconTermDef, RoleDef};

/// Module-level entry produced by [`load_modules`].
#[derive(Debug)]
pub struct LoadedModule {
    pub name: String,
    pub root: PathBuf,
    pub version: Option<String>,
    pub archetypes: Vec<Arc<CompiledArchetype>>,
    /// Advisory lint rules declared by the module (FR-036).
    pub lint_rules: Vec<crate::lint::LintRule>,
    /// Edge-type registry contributed by this module (FR-040).
    pub edge_types: BTreeMap<String, EdgeTypeDef>,
    /// Role registry contributed by this module (FR-040).
    pub roles: BTreeMap<String, RoleDef>,
    /// Concrete-term lexicon contributed by this module (FR-043).
    pub lexicon: BTreeMap<String, LexiconTermDef>,
    /// Per-check grammar severity registry contributed by this module
    /// (FR-048).
    pub grammar_severity: BTreeMap<String, crate::grammar::GrammarSeverityLevel>,
    /// Observable-result verb registry contributed by this module (FR-047).
    pub observable_verbs: BTreeMap<String, crate::vocab::ObservableVerbDef>,
    /// Vacuous-predicate registry contributed by this module (FR-047, CR-014).
    pub vacuous_predicates: BTreeMap<String, crate::vocab::VacuousPredicateDef>,
    /// Property-idiom registry contributed by this module (FR-052).
    pub property_idioms: BTreeMap<String, crate::vocab::PropertyIdiomDef>,
    /// Verification-method catalog contributed by this module (FR-054).
    pub verification_catalog: BTreeMap<String, crate::vocab::VerificationMethodDef>,
    /// Ambiguity lexicon contributed by this module (FR-056).
    pub ambiguity_terms: BTreeMap<String, crate::vocab::AmbiguityTermDef>,
    /// Traceability model contributed by this module (FR-050).
    pub traceability: crate::traceability::TraceabilityModel,
}

/// Outcome of a full load pass.
#[derive(Debug)]
pub struct LoadOutcome {
    pub modules: Vec<LoadedModule>,
    pub failures: Vec<ArchetypeLoadFailure>,
    pub diagnostics: Vec<Diagnostic>,
    pub path_diagnostics: Vec<PathDiagnostic>,
}

/// Caller-facing reason for a retired manifest field (no
/// backward-compatibility layer — ADR 0003 / FR-031, render removed).
fn retired_field_reason(field: &str) -> String {
    match field {
        "template_ref" => "retired field `template_ref` is not supported; the render/templating \
             feature is removed (no backward-compatibility layer)"
            .to_string(),
        other => format!(
            "retired field `{other}` is not supported; move its intent into `body_extraction` \
             asserts (FR-033)"
        ),
    }
}

/// Load every module reachable from `explicit` (or
/// `IX_FILAMENT_MODULES_PATH` / `IX_SCHEMA_PATH` / `~/.ix/filament/modules`
/// when `explicit` is empty).
pub fn load_modules(explicit: &[&Path]) -> LoadOutcome {
    let env_value = module_path_env(
        std::env::var_os("IX_FILAMENT_MODULES_PATH"),
        std::env::var_os("IX_SCHEMA_PATH"),
    );
    let path_diagnostics = resolve_search_paths(explicit, env_value);

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
        };
    }

    match load_one_module(&canonical, &mut diagnostics) {
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
    }
}

/// Build a `LoadOutcome` from an in-memory module blob — no filesystem
/// access (FR-013 wasm amendment).
///
/// `manifest_yaml` is the raw `manifest.yaml` bytes. `schemas` maps the
/// manifest's relative `frontmatter_schema_ref` strings to schema JSON
/// text. The render/templating feature is removed, so no `templates`
/// map is accepted; a manifest declaring `template_ref` is rejected
/// like any other retired field.
///
/// Per-archetype failures aggregate like the filesystem loader. Module
/// name is taken from the manifest; if absent it falls back to the
/// sentinel `"<inline>"` and emits a `ManifestMissingName` diagnostic
/// (mirroring FR-014-AC-7's path-derived behavior).
pub fn load_inline_module(manifest_yaml: &[u8], schemas: &BTreeMap<String, String>) -> LoadOutcome {
    use crate::loader::compile::{compile_schema, failure, CompiledArchetype};
    use crate::loader::manifest::parse_manifest;

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
                retired_field_reason(field),
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
        )));
    }

    modules.push(LoadedModule {
        name: module_name,
        root: inline_root,
        version: manifest.version.clone(),
        archetypes,
        lint_rules: manifest.lint_rules.clone(),
        edge_types: manifest.edge_types.clone(),
        roles: manifest.roles.clone(),
        lexicon: manifest.lexicon.clone(),
        grammar_severity: manifest.grammar_severity.clone(),
        observable_verbs: manifest.observable_verbs.clone(),
        vacuous_predicates: manifest.vacuous_predicates.clone(),
        property_idioms: manifest.property_idioms.clone(),
        verification_catalog: manifest.verification_catalog.clone(),
        ambiguity_terms: manifest.ambiguity_terms.clone(),
        traceability: manifest.traceability.clone(),
    });

    LoadOutcome {
        modules,
        failures,
        diagnostics,
        path_diagnostics: Vec::new(),
    }
}

/// Convenience: same as [`load_modules`] with no explicit paths — uses
/// `IX_FILAMENT_MODULES_PATH` / `IX_SCHEMA_PATH` (or the default
/// `~/.ix/filament/modules/`).
pub fn load_from_env() -> LoadOutcome {
    load_modules(&[])
}

/// Convenience: load only from `~/.ix/filament/modules/`, ignoring the
/// search-path env vars.
pub fn load_from_default() -> LoadOutcome {
    let default_root = default_module_root();
    match default_root {
        Some(root) => load_modules(&[&root]),
        None => LoadOutcome {
            modules: Vec::new(),
            failures: Vec::new(),
            diagnostics: Vec::new(),
            path_diagnostics: Vec::new(),
        },
    }
}

/// Walk one search-path root looking for module sub-directories
/// (each containing a `manifest.yaml`).
fn walk_search_root(
    root: &Path,
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
    // `read_dir` yields in filesystem order, which is unspecified and differs
    // between machines. Every merge in the registry is **first-wins**, so that
    // order decides which module's `lexicon` / `grammar_severity` /
    // `traceability` entry survives a collision — the outcome cannot depend on
    // how a directory happens to be laid out (NFR-006). Sorting by canonical
    // path makes module load order, and therefore first-wins, deterministic.
    let mut candidates: Vec<std::path::PathBuf> =
        entries.flatten().map(|entry| entry.path()).collect();
    candidates.sort();
    for candidate in candidates {
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
        match load_one_module(&canon, diagnostics) {
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
        match compile_archetype(&module_name, module_root, at) {
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
            lint_rules: manifest.lint_rules.clone(),
            edge_types: manifest.edge_types.clone(),
            roles: manifest.roles.clone(),
            lexicon: manifest.lexicon.clone(),
            grammar_severity: manifest.grammar_severity.clone(),
            observable_verbs: manifest.observable_verbs.clone(),
            vacuous_predicates: manifest.vacuous_predicates.clone(),
            property_idioms: manifest.property_idioms.clone(),
        verification_catalog: manifest.verification_catalog.clone(),
        ambiguity_terms: manifest.ambiguity_terms.clone(),
            traceability: manifest.traceability.clone(),
        },
        failures,
    ))
}

/// Compile one unified archetype (FR-031). Resolves the optional
/// frontmatter schema + data_schema, validates the `body_extraction`
/// DSL and `assert` facets, and rejects the retired
/// `required_sections`/`variants`/`template_ref` fields (no
/// backward-compat layer, render removed).
fn compile_archetype(
    module: &str,
    module_root: &Path,
    a: &Archetype,
) -> Result<CompiledArchetype, ArchetypeLoadFailure> {
    // No-compat rule (ADR 0003): retired fields are a hard failure.
    if let Some(field) = a.retired_field() {
        return Err(failure(
            module,
            &a.name,
            module_root.join("manifest.yaml"),
            retired_field_reason(field),
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
    ))
}

/// Assemble a `CompiledArchetype` from resolved parts, picking the
/// primary `raw_schema`/`validator` (frontmatter when present, else
/// data, else an empty permissive object) for the FR-003 `schema_for`
/// surface and the `validate()` path.
fn finish_compiled(
    module: &str,
    a: &Archetype,
    frontmatter_schema: Option<Arc<Value>>,
    frontmatter_validator: Option<Arc<jsonschema::JSONSchema>>,
    data_schema: Option<Arc<Value>>,
    data_validator: Option<Arc<jsonschema::JSONSchema>>,
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

    let lint_rules: Vec<crate::lint::LintRule> = outcome
        .modules
        .iter()
        .flat_map(|m| m.lint_rules.iter().cloned())
        .collect();

    // ── FR-040: merge the edge_types + roles registries (first-wins,
    // mirroring archetype merge). A name re-declared with a *differing*
    // body emits a Duplicate{EdgeType,Role} diagnostic; identical
    // re-declaration is silently idempotent. ──
    let (edge_types, mut edge_type_dups) = merge_vocab(&outcome.modules, |m| &m.edge_types);
    for (name, modules) in edge_type_dups.drain(..) {
        outcome
            .diagnostics
            .push(Diagnostic::DuplicateEdgeType { name, modules });
    }
    let (roles, mut role_dups) = merge_vocab(&outcome.modules, |m| &m.roles);
    for (name, modules) in role_dups.drain(..) {
        outcome
            .diagnostics
            .push(Diagnostic::DuplicateRole { name, modules });
    }
    // FR-043: merge the concrete-term lexicon (same first-wins machinery).
    let (lexicon, mut lexicon_dups) = merge_vocab(&outcome.modules, |m| &m.lexicon);
    for (name, modules) in lexicon_dups.drain(..) {
        outcome
            .diagnostics
            .push(Diagnostic::DuplicateLexiconTerm { name, modules });
    }
    // FR-048: merge the per-check grammar severity registry (same machinery).
    // A key redeclared with a *differing* level is first-wins + one non-fatal
    // DuplicateGrammarSeverity; identical redeclaration is idempotent.
    let (grammar_severity, mut severity_dups) =
        merge_vocab(&outcome.modules, |m| &m.grammar_severity);
    for (name, modules) in severity_dups.drain(..) {
        outcome
            .diagnostics
            .push(Diagnostic::DuplicateGrammarSeverity { name, modules });
    }
    // FR-047: merge the observable-result verb registry (first-wins). The
    // engine's built-in defaults are layered underneath at matcher-build time,
    // so a module extends the vocabulary rather than replacing it.
    let (observable_verbs, _) = merge_vocab(&outcome.modules, |m| &m.observable_verbs);
    // CR-014: same first-wins merge for the vacuity vocabulary.
    let (vacuous_predicates, _) = merge_vocab(&outcome.modules, |m| &m.vacuous_predicates);
    // FR-052: same first-wins merge for the property-idiom vocabulary. It is
    // read only by the classifier, which emits no finding.
    let (property_idioms, _) = merge_vocab(&outcome.modules, |m| &m.property_idioms);
    // FR-054: same first-wins merge for the verification-method catalog. Unlike
    // the vocabularies above, a re-declared id IS reported — a catalog is a
    // registry of distinct methods, and two modules disagreeing about what
    // `mutation-testing` means is the kind of collision an operator must see.
    // FR-056: same first-wins merge as the other advisory vocabularies. A
    // conflict is not reported, matching `observable_verbs`/`vacuous_predicates`:
    // these are *sets*, and re-declaring a term with a different gloss changes
    // nothing the engine reads.
    let (ambiguity_terms, _) = merge_vocab(&outcome.modules, |m| &m.ambiguity_terms);
    let (verification_catalog, mut catalog_dups) =
        merge_vocab(&outcome.modules, |m| &m.verification_catalog);
    for (name, modules) in catalog_dups.drain(..) {
        outcome
            .diagnostics
            .push(Diagnostic::DuplicateVerificationMethod { name, modules });
    }

    // FR-050: merge the declarative traceability model across modules. Targets,
    // references, and tag forms accumulate first-wins by name; the singular
    // `status` vocabulary is taken from the first module that declares one.
    // A module that declares nothing contributes nothing, so the merged model
    // stays undeclared until some module declares one.
    let traceability = merge_traceability(&outcome.modules);

    // ── FR-041: derive the inverse-label → forward-verb index from the
    // merged edge_types. A declared `inverse:` label becomes an authorable
    // verb (a derived view of its forward edge). Precedence: a label that
    // is itself a forward `edge_types` key is governed by that forward
    // registration, never treated as an inverse. Two forward verbs
    // declaring the same inverse label are first-wins (edge_types is a
    // BTreeMap, so the lexicographically first forward wins
    // deterministically) and emit a non-fatal DuplicateInverseEdge. ──
    let mut inverse_edges: BTreeMap<String, String> = BTreeMap::new();
    let mut inverse_conflicts: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (verb, def) in &edge_types {
        let Some(label) = def.inverse.as_ref() else {
            continue;
        };
        if edge_types.contains_key(label) {
            continue; // forward registration governs the name
        }
        match inverse_edges.get(label) {
            None => {
                inverse_edges.insert(label.clone(), verb.clone());
            }
            Some(winner) => {
                inverse_conflicts
                    .entry(label.clone())
                    .or_insert_with(|| vec![winner.clone()])
                    .push(verb.clone());
            }
        }
    }
    for (name, forwards) in inverse_conflicts {
        outcome
            .diagnostics
            .push(Diagnostic::DuplicateInverseEdge { name, forwards });
    }

    // ── FR-040: advisory check that every verb used in an archetype's
    // allowed_links is declared in edge_types, and every role used in a
    // `roles:` list or as a target token is declared in roles. Open
    // until declared — non-fatal (load_strict escalates). Deterministic:
    // active_archetypes is a BTreeMap, allowed_links a BTreeMap. ──
    for arch in active_archetypes.values() {
        for (verb, targets) in arch.allowed_links() {
            // FR-041: a declared inverse label is a valid verb, so an
            // `allowed_links` key that is an inverse label is not unknown.
            if !edge_types.contains_key(verb) && !inverse_edges.contains_key(verb) {
                outcome.diagnostics.push(Diagnostic::UnknownEdgeType {
                    archetype: arch.name.clone(),
                    edge_type: verb.clone(),
                });
            }
            for token in targets {
                // A target token may be "*", a concrete archetype name,
                // or a role. Only flag tokens that are neither "*", a
                // known archetype, nor a known role.
                if token == "*"
                    || active_archetypes.contains_key(token)
                    || roles.contains_key(token)
                {
                    continue;
                }
                outcome.diagnostics.push(Diagnostic::UnknownRole {
                    archetype: arch.name.clone(),
                    role: token.clone(),
                });
            }
        }
        for role in arch.roles() {
            if !roles.contains_key(role) {
                outcome.diagnostics.push(Diagnostic::UnknownRole {
                    archetype: arch.name.clone(),
                    role: role.clone(),
                });
            }
        }
    }

    RegistryShape {
        archetypes: active_archetypes,
        by_module_and_name,
        module_paths,
        module_versions,
        lint_rules,
        edge_types,
        inverse_edges,
        roles,
        lexicon,
        grammar_severity,
        observable_verbs,
        vacuous_predicates,
        property_idioms,
        verification_catalog,
        ambiguity_terms,
        traceability,
        failures: outcome.failures,
        diagnostics: outcome.diagnostics,
        path_diagnostics: outcome.path_diagnostics,
    }
}

/// `(name, contributing-module-names)` for one conflicting vocabulary
/// re-declaration, used to build a `Duplicate{EdgeType,Role}` diagnostic.
type VocabConflicts = Vec<(String, Vec<String>)>;

/// Merge a per-module vocabulary map (edge_types or roles) across modules
/// first-wins. Returns the merged map plus, for each name re-declared
/// with a *differing* body, the contributing module names (the original
/// winner followed by each conflicting module) for a Duplicate
/// diagnostic. Identical re-declarations are silently idempotent.
fn merge_vocab<V: Clone + PartialEq>(
    modules: &[LoadedModule],
    pick: impl Fn(&LoadedModule) -> &BTreeMap<String, V>,
) -> (BTreeMap<String, V>, VocabConflicts) {
    let mut merged: BTreeMap<String, V> = BTreeMap::new();
    let mut origin: BTreeMap<String, String> = BTreeMap::new();
    let mut conflicts: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for module in modules {
        for (name, def) in pick(module) {
            match merged.get(name) {
                None => {
                    merged.insert(name.clone(), def.clone());
                    origin.insert(name.clone(), module.name.clone());
                }
                Some(existing) if existing == def => {} // idempotent
                Some(_) => {
                    let entry = conflicts
                        .entry(name.clone())
                        .or_insert_with(|| vec![origin.get(name).cloned().unwrap_or_default()]);
                    entry.push(module.name.clone());
                }
            }
        }
    }
    (merged, conflicts.into_iter().collect())
}

/// Merge the per-module [`TraceabilityModel`](crate::traceability::TraceabilityModel)s
/// first-wins: an entry whose name is already declared is skipped, and the
/// first declared `status` vocabulary wins. Declaration order is module load
/// order, so the merged model is deterministic (NFR-006).
fn merge_traceability(modules: &[LoadedModule]) -> crate::traceability::TraceabilityModel {
    let mut merged = crate::traceability::TraceabilityModel::default();
    for module in modules {
        let m = &module.traceability;
        // CR-060: the model-level exclusion is the one key that merges as a
        // **union**, not first-wins. It states a fact about the repository —
        // "these paths are not corpus data" — and a path one module declares
        // non-corpus must not become corpus because another module happened to
        // load first. The set it yields does not depend on load order, which
        // the named-entry merges get from their first-wins rule instead
        // (NFR-006).
        for pattern in &m.exclude {
            if !merged.exclude.contains(pattern) {
                merged.exclude.push(pattern.clone());
            }
        }
        for target in &m.trace_targets {
            if !merged.trace_targets.iter().any(|t| t.name == target.name) {
                merged.trace_targets.push(target.clone());
            }
        }
        for reference in &m.document_references {
            if !merged
                .document_references
                .iter()
                .any(|r| r.name == reference.name)
            {
                merged.document_references.push(reference.clone());
            }
        }
        // FR-053: obligation sources merge first-wins by name, like every
        // other named entry — a source one module declares must not be
        // redefined by another that happened to load later.
        for source in &m.obligations {
            if !merged.obligations.iter().any(|s| s.name == source.name) {
                merged.obligations.push(source.clone());
            }
        }
        for marker in &m.trace_tags.markers {
            if !merged
                .trace_tags
                .markers
                .iter()
                .any(|x| x.name == marker.name)
            {
                merged.trace_tags.markers.push(marker.clone());
            }
        }
        for legacy in &m.trace_tags.legacy {
            if !merged
                .trace_tags
                .legacy
                .iter()
                .any(|x| x.name == legacy.name)
            {
                merged.trace_tags.legacy.push(legacy.clone());
            }
        }
        if merged.status.is_none() {
            merged.status.clone_from(&m.status);
        }
        // CR-015: column vocabularies merge first-wins per column.
        if merged.vocabularies.test_type.is_empty() {
            merged
                .vocabularies
                .test_type
                .clone_from(&m.vocabularies.test_type);
        }
        // CR-041: the exemption vocabulary and the column it reads merge the
        // same way. They are separate keys, so a module may declare the
        // vocabulary while another supplies the test types it draws from.
        if merged.vocabularies.test_type_column.is_none() {
            merged
                .vocabularies
                .test_type_column
                .clone_from(&m.vocabularies.test_type_column);
        }
        if merged.vocabularies.no_source_symbol.is_empty() {
            merged
                .vocabularies
                .no_source_symbol
                .clone_from(&m.vocabularies.no_source_symbol);
        }
    }
    merged
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
            // FR-040-AC-3: load_strict escalates edge-vocabulary
            // diagnostics (conflicts + unknown verb/role) to errors.
            Diagnostic::DuplicateEdgeType { name, .. } => {
                return Err(QuireError::EdgeVocabularyViolation {
                    kind: "DuplicateEdgeType".to_string(),
                    name: name.clone(),
                });
            }
            Diagnostic::DuplicateRole { name, .. } => {
                return Err(QuireError::EdgeVocabularyViolation {
                    kind: "DuplicateRole".to_string(),
                    name: name.clone(),
                });
            }
            // FR-043: load_strict escalates a conflicting lexicon term.
            Diagnostic::DuplicateLexiconTerm { name, .. } => {
                return Err(QuireError::EdgeVocabularyViolation {
                    kind: "DuplicateLexiconTerm".to_string(),
                    name: name.clone(),
                });
            }
            // FR-041-AC-3: load_strict escalates a colliding inverse label.
            Diagnostic::DuplicateInverseEdge { name, .. } => {
                return Err(QuireError::EdgeVocabularyViolation {
                    kind: "DuplicateInverseEdge".to_string(),
                    name: name.clone(),
                });
            }
            Diagnostic::UnknownEdgeType { edge_type, .. } => {
                return Err(QuireError::EdgeVocabularyViolation {
                    kind: "UnknownEdgeType".to_string(),
                    name: edge_type.clone(),
                });
            }
            Diagnostic::UnknownRole { role, .. } => {
                return Err(QuireError::EdgeVocabularyViolation {
                    kind: "UnknownRole".to_string(),
                    name: role.clone(),
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
    /// Advisory lint rules aggregated across modules in load order
    /// (FR-036).
    pub lint_rules: Vec<crate::lint::LintRule>,
    /// Merged edge-type registry, first-wins across modules (FR-040).
    pub edge_types: BTreeMap<String, EdgeTypeDef>,
    /// Inverse-label → forward-verb index derived from `edge_types`
    /// (FR-041). An `inverse:` label that is itself a forward `edge_types`
    /// key is excluded (the forward registration governs).
    pub inverse_edges: BTreeMap<String, String>,
    /// Merged role registry, first-wins across modules (FR-040).
    pub roles: BTreeMap<String, RoleDef>,
    /// Merged concrete-term lexicon, first-wins across modules (FR-043).
    pub lexicon: BTreeMap<String, LexiconTermDef>,
    /// Merged per-check grammar severity registry, first-wins across modules
    /// (FR-048). An absent key means `warning`.
    pub grammar_severity: BTreeMap<String, crate::grammar::GrammarSeverityLevel>,
    /// Merged observable-result verb registry, first-wins across modules
    /// (FR-047). Layered over the engine's built-in defaults.
    pub observable_verbs: BTreeMap<String, crate::vocab::ObservableVerbDef>,
    /// Merged vacuous-predicate registry, first-wins across modules
    /// (FR-047, CR-014). Layered over the engine's built-in vacuity set.
    pub vacuous_predicates: BTreeMap<String, crate::vocab::VacuousPredicateDef>,
    /// Merged property-idiom registry, first-wins across modules (FR-052).
    /// Layered over the engine's built-in idioms.
    pub property_idioms: BTreeMap<String, crate::vocab::PropertyIdiomDef>,
    pub verification_catalog: BTreeMap<String, crate::vocab::VerificationMethodDef>,
    pub ambiguity_terms: BTreeMap<String, crate::vocab::AmbiguityTermDef>,
    /// Merged traceability model, first-wins across modules (FR-050).
    pub traceability: crate::traceability::TraceabilityModel,
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
        fs::write(
            root.join("manifest.yaml"),
            format!(
                "name: {name}\nartifact_types:\n- name: foo\n  frontmatter_schema_ref: schemas/foo.schema.json\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("schemas/foo.schema.json"),
            r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#,
        )
        .unwrap();
    }

    // TC-826 (CR-060): the model-level exclusion is the one traceability key
    // that merges as a **union**. Every other key is first-wins by name, which
    // is right for a named declaration and wrong for a statement of fact: a
    // path one module declares non-corpus must not become corpus because
    // another module happened to load first.
    #[test]
    fn model_level_exclude_merges_as_a_union() {
        let parent = tmpdir("merge-exclude");
        for (name, exclude, target) in [
            ("mod-a", "spec/fixtures/**", "acceptance-criterion"),
            ("mod-b", "vendor/**", "test-case"),
            // A third module repeating mod-a's pattern must not duplicate it.
            ("mod-c", "spec/fixtures/**", "eval-case"),
        ] {
            let root = parent.join(name);
            fs::create_dir_all(&root).unwrap();
            fs::write(
                root.join("manifest.yaml"),
                format!(
                    "name: {name}\nartifact_types:\n- name: foo\n\
                     traceability:\n  exclude: ['{exclude}']\n  trace_targets:\n\
                     \x20 - name: {target}\n    archetype: foo\n    section: S\n    id_column: ID\n"
                ),
            )
            .unwrap();
        }

        let outcome = load_modules(&[&parent]);
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        let merged = merge_traceability(&outcome.modules);
        assert_eq!(
            merged.exclude,
            vec!["spec/fixtures/**".to_string(), "vendor/**".to_string()],
            "both patterns survive, deduplicated"
        );
        assert_eq!(
            merged.trace_targets.len(),
            3,
            "and the named entries merge as before"
        );
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
            "name: mod-b\nartifact_types:\n- name: foo\n  frontmatter_schema_ref: s/missing.json\n",
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

    // TC-522 (FR-031-AC-1): frontmatter_schema_ref + body_extraction
    // compiles to one CompiledArchetype that is validatable (frontmatter
    // schema) and extractable (resolvable body contract); no
    // renderability concept is exposed (render removed).
    #[test]
    fn tc522_unified_archetype_with_body_extraction() {
        let parent = tmpdir("u-522");
        let root = parent.join("u-mod");
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
artifact_types:
- name: FR
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
        let outcome = load_modules(&[&parent]);
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        let arch = &outcome.modules[0].archetypes[0];
        assert_eq!(arch.name, "FR");
        assert!(arch.is_validatable());
        assert!(arch.body_extraction().is_some());
        assert!(arch.frontmatter_validator().is_some());
    }

    // TC-523 (FR-031-AC-2): body_extraction compiles, validatable +
    // extractable.
    #[test]
    fn tc523_unified_archetype_validatable_and_extractable() {
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
        assert!(arch.is_validatable());
        assert!(arch.body_extraction().is_some());
    }

    // TC-524 (FR-031-AC-3): carry-over fields retained + readable.
    #[test]
    fn tc524_carry_over_fields_retained() {
        let parent = tmpdir("u-524");
        let root = parent.join("u-mod");
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
artifact_types:
- name: FR
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
        let outcome = load_modules(&[&parent]);
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        let arch = &outcome.modules[0].archetypes[0];
        assert_eq!(arch.id_pattern(), Some("FR-{next:03d}"));
        assert_eq!(arch.grammar_ref(), Some("iso-spec-core"));
        assert!(arch.has_plugin());
        // FR-040 CR-001: the flat-array authoring form normalizes to the
        // `{verb: ["*"]}` map (allowed against any target).
        assert_eq!(
            arch.allowed_links().get("implements"),
            Some(&vec!["*".to_string()])
        );
        assert_eq!(
            arch.allowed_links().get("refines"),
            Some(&vec!["*".to_string()])
        );
    }

    // TC-525 (FR-031-AC-4): frontmatter_schema_ref + data_schema are
    // two distinct compiled validators, neither collapsed.
    #[test]
    fn tc525_frontmatter_and_data_schemas_are_distinct() {
        let parent = tmpdir("u-525");
        let root = parent.join("u-mod");
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
artifact_types:
- name: FR
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
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
artifact_types:
- name: FR
  frontmatter_schema_ref: schemas/fr.schema.json
  required_sections:
  - Description
  - Specification
"#,
        )
        .unwrap();
        fs::write(root.join("schemas/fr.schema.json"), r#"{"type":"object"}"#).unwrap();
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

    // TC-526c (FR-031-AC-5): `template_ref` is a hard ArchetypeLoadFailure
    // (render removed — no backward-compatibility layer).
    #[test]
    fn tc526_template_ref_is_hard_load_failure() {
        let parent = tmpdir("u-526c");
        let root = parent.join("u-mod");
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: u-mod
artifact_types:
- name: FR
  template_ref: templates/fr.md.j2
  frontmatter_schema_ref: schemas/fr.schema.json
"#,
        )
        .unwrap();
        fs::write(root.join("schemas/fr.schema.json"), r#"{"type":"object"}"#).unwrap();
        let outcome = load_modules(&[&parent]);
        assert_eq!(outcome.modules[0].archetypes.len(), 0);
        assert_eq!(outcome.failures.len(), 1);
        let f = &outcome.failures[0];
        assert_eq!(f.archetype, "FR");
        assert!(
            f.reason.contains("template_ref") && f.reason.contains("render"),
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

    // TC-762 (NFR-006-AC-5, CR-018): module discovery is sorted, so first-wins
    // resolves the same way on every machine. Directories are created in
    // reverse order to keep the fixture honest — `read_dir` order is
    // unspecified, and it was the load order before this.
    #[test]
    fn tc762_module_discovery_is_sorted() {
        let p = tmpdir("sorted-discovery");
        for name in ["m-zulu", "m-mike", "m-alpha"] {
            write_minimal_module(&p.join(name), name);
        }
        let outcome = load_modules(&[&p]);
        let names: Vec<&str> = outcome.modules.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["m-alpha", "m-mike", "m-zulu"],
            "modules must load in sorted path order, not filesystem order"
        );
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
