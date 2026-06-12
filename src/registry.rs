//! `Registry` — the compiled, immutable, thread-safe lookup of
//! every archetype loaded from disk (FR-013).
//!
//! Construction is via the [`Registry::load_from`] /
//! [`Registry::from_env`] / [`Registry::from_default`] constructors,
//! which delegate to [`crate::loader::load_modules`]. After
//! construction the registry is frozen — there is no `add` method;
//! to swap the active archetype set, build a new `Registry` and drop
//! the previous one. Outstanding `Arc<CompiledArchetype>` clones
//! continue to work for the duration of any in-flight validate/extract.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::diagnostic::Diagnostic;
use crate::error::{ArchetypeLoadFailure, QuireError};
use crate::loader::compile::CompiledArchetype;
use crate::loader::paths::PathDiagnostic;
use crate::loader::{
    flatten_into_registry, flatten_into_registry_strict, load_modules, RegistryShape,
};

/// Compiled, immutable archetype registry. `Send + Sync` and cheap to
/// clone (`Arc<Inner>`).
#[derive(Clone)]
pub struct Registry {
    inner: Arc<Inner>,
}

struct Inner {
    archetypes: std::collections::BTreeMap<String, Arc<CompiledArchetype>>,
    by_module_and_name: std::collections::BTreeMap<(String, String), Arc<CompiledArchetype>>,
    module_paths: std::collections::BTreeMap<String, PathBuf>,
    module_versions: std::collections::BTreeMap<String, Option<String>>,
    lint_rules: Vec<crate::lint::LintRule>,
    failures: Vec<ArchetypeLoadFailure>,
    diagnostics: Vec<Diagnostic>,
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
    /// (same as [`from_env`](Registry::from_env)). Module and archetype
    /// name collisions surface as [`Diagnostic`]s, not errors —
    /// see [`load_strict`](Registry::load_strict) for the strict variant.
    pub fn load_from(paths: &[&Path]) -> Result<Self, QuireError> {
        let outcome = load_modules(paths);
        Ok(Self::finish_tolerant(outcome))
    }

    /// Like [`load_from`](Registry::load_from), but the first module-
    /// or archetype-name collision is promoted to a `QuireError`.
    pub fn load_strict(paths: &[&Path]) -> Result<Self, QuireError> {
        let outcome = load_modules(paths);
        Self::finish_strict(outcome)
    }

    /// Load a single module directory. Unlike
    /// [`load_from`](Registry::load_from), which treats each argument as
    /// a **search root** whose children are inspected for module
    /// manifests, this entry point treats `module_root` as a single
    /// module: it MUST contain `manifest.yaml` directly, and no sibling
    /// directories under its parent are inspected.
    ///
    /// Use this when a caller has already resolved the path of a
    /// specific module (e.g. a CLI receiving `--module <path>`).
    /// Promoting to the parent and reusing `load_from` would silently
    /// expose every sibling directory under that parent as an
    /// additional candidate module, which is both surprising and a
    /// path-safety concern when the argument is user-controlled.
    ///
    /// If `module_root/manifest.yaml` is missing, the returned registry
    /// has zero modules and a single `ArchetypeLoadFailure` describing
    /// the absent manifest.
    pub fn load_module(module_root: &Path) -> Result<Self, QuireError> {
        let outcome = crate::loader::load_single_module(module_root);
        Ok(Self::finish_tolerant(outcome))
    }

    /// Strict counterpart of [`load_module`](Registry::load_module):
    /// the first collision diagnostic is promoted to a fatal
    /// `QuireError`.
    pub fn load_module_strict(module_root: &Path) -> Result<Self, QuireError> {
        let outcome = crate::loader::load_single_module(module_root);
        Self::finish_strict(outcome)
    }

    /// Load from `IX_SCHEMA_PATH` (then default `~/.ix/schemas/`).
    pub fn from_env() -> Result<Self, QuireError> {
        let outcome = load_modules(&[]);
        Ok(Self::finish_tolerant(outcome))
    }

    /// Load from `~/.ix/schemas/` only.
    pub fn from_default() -> Result<Self, QuireError> {
        let outcome = crate::loader::load_from_default();
        Ok(Self::finish_tolerant(outcome))
    }

    /// Build a `Registry` from an in-memory module blob — no filesystem
    /// access.
    ///
    /// This is the FR-013 "wasm amendment" entry point: a caller (browser
    /// host, WASM binding, embedded server) that already holds the
    /// module's manifest + schema files in memory can construct a
    /// registry directly, bypassing `loader::*`'s filesystem walk.
    ///
    /// Arguments:
    /// - `manifest_yaml`: the raw bytes of `manifest.yaml`.
    /// - `schemas`: map of `<schema_ref>` (the manifest's relative path
    ///   string) to schema JSON text. Every
    ///   `artifact_type.frontmatter_schema_ref` must have a matching
    ///   entry; object-type `data_schema` lives inline in the manifest
    ///   and does not need an entry here.
    ///
    /// The render/templating feature is removed, so no `templates` map is
    /// accepted; a manifest declaring `template_ref` is rejected as a
    /// retired field.
    ///
    /// Per-archetype failures surface in `failures()` exactly as with
    /// the filesystem loader; module collisions emit diagnostics. Use
    /// [`Self::from_inline_parts_strict`] to promote collisions to a
    /// fatal `QuireError`.
    pub fn from_inline_parts(
        manifest_yaml: &[u8],
        schemas: &std::collections::BTreeMap<String, String>,
    ) -> Result<Self, QuireError> {
        let outcome = crate::loader::load_inline_module(manifest_yaml, schemas);
        Ok(Self::finish_tolerant(outcome))
    }

    /// Strict counterpart of [`Self::from_inline_parts`]: the first
    /// collision diagnostic is promoted to a fatal `QuireError`.
    pub fn from_inline_parts_strict(
        manifest_yaml: &[u8],
        schemas: &std::collections::BTreeMap<String, String>,
    ) -> Result<Self, QuireError> {
        let outcome = crate::loader::load_inline_module(manifest_yaml, schemas);
        Self::finish_strict(outcome)
    }

    fn finish_tolerant(outcome: crate::loader::LoadOutcome) -> Self {
        let shape = flatten_into_registry(outcome);
        Self::from_shape(shape)
    }

    fn finish_strict(outcome: crate::loader::LoadOutcome) -> Result<Self, QuireError> {
        let shape = flatten_into_registry_strict(outcome)?;
        Ok(Self::from_shape(shape))
    }

    fn from_shape(shape: RegistryShape) -> Self {
        let RegistryShape {
            archetypes,
            by_module_and_name,
            module_paths,
            module_versions,
            lint_rules,
            failures,
            diagnostics,
            path_diagnostics,
        } = shape;
        Self {
            inner: Arc::new(Inner {
                archetypes,
                by_module_and_name,
                module_paths,
                module_versions,
                lint_rules,
                failures,
                diagnostics,
                path_diagnostics,
            }),
        }
    }

    /// Look up a compiled archetype by name.
    /// Look up a block type by name. In v0.2 each archetype is a
    /// block type (1:1) — this method is the canonical block-model
    /// entry point per INPUT.md vocabulary. `archetype()` remains as
    /// a synonym for code internal to the parity port.
    pub fn block_type(&self, name: &str) -> Option<&CompiledArchetype> {
        self.archetype(name)
    }

    pub fn archetype(&self, name: &str) -> Option<&CompiledArchetype> {
        self.inner.archetypes.get(name).map(|a| a.as_ref())
    }

    /// Return the JSON Schema document loaded for `name` (FR-003).
    /// The returned `Value` is the same shape loaded from disk so LLM
    /// tool-call definitions can read what the engine validates against.
    pub fn schema_for(&self, name: &str) -> Result<&serde_json::Value, QuireError> {
        match self.inner.archetypes.get(name) {
            Some(a) => Ok(a.raw_schema.as_ref()),
            None => Err(QuireError::UnknownArchetype {
                name: name.to_string(),
            }),
        }
    }

    /// Iterate over every loaded archetype name.
    pub fn archetype_names(&self) -> impl Iterator<Item = &str> {
        self.inner.archetypes.keys().map(|s| s.as_str())
    }

    /// Iterate over every loaded module name.
    pub fn module_names(&self) -> impl Iterator<Item = &str> {
        self.inner.module_paths.keys().map(|s| s.as_str())
    }

    /// Resolve a specific (module, archetype) pair — used to inspect
    /// shadowed archetypes after a `DuplicateArchetype` diagnostic.
    pub fn archetype_in_module(&self, module: &str, name: &str) -> Option<&CompiledArchetype> {
        self.inner
            .by_module_and_name
            .get(&(module.to_string(), name.to_string()))
            .map(|a| a.as_ref())
    }

    /// Manifest version declared by `module`, if any (FR-014-AC-4).
    pub fn module_version(&self, module: &str) -> Option<&str> {
        self.inner
            .module_versions
            .get(module)
            .and_then(|v| v.as_deref())
    }

    /// Non-fatal load-time diagnostics (duplicate names, missing
    /// manifest `name`, search-path issues, symlink loops).
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.inner.diagnostics
    }

    /// Per-archetype failures collected during load. The Registry can
    /// still be used; consumers choose whether to treat any failure as
    /// fatal (`load_strict`, FR-014).
    pub fn failures(&self) -> &[ArchetypeLoadFailure] {
        &self.inner.failures
    }

    /// Advisory lint rules declared by the loaded modules, in load
    /// order (FR-036). Evaluate via [`crate::lint::lint_document`].
    pub fn lint_rules(&self) -> &[crate::lint::LintRule] {
        &self.inner.lint_rules
    }

    /// Path-resolution diagnostics (missing dirs, file-not-dir,
    /// permission-denied entries). Always advisory.
    pub fn path_diagnostics(&self) -> &[PathDiagnostic] {
        &self.inner.path_diagnostics
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

    // FR-011-AC-6/7/8: a body_extraction DSL that's structurally
    // invalid (both `match` and `iterate_over` set) surfaces as an
    // ArchetypeLoadFailure at load time, NOT at extract() time.
    #[test]
    fn invalid_dsl_in_object_type_fails_at_load() {
        let parent = tmpdir("bad-dsl");
        let m = parent.join("ot-mod");
        fs::create_dir_all(&m).unwrap();
        fs::write(
            m.join("manifest.yaml"),
            r#"
name: ot-mod
object_types:
- name: bad
  body_extraction:
    yield_pattern:
      match:
        a:
          from: heading
      iterate_over:
        section_path: [X]
        kind: heading
      per_match:
        b:
          from: heading
"#,
        )
        .unwrap();
        let r = Registry::load_from(&[&parent]).expect("tolerant load ok");
        // No `bad` archetype registered — compile failed.
        assert!(r.archetype("bad").is_none());
        // Failure was aggregated.
        assert!(
            r.failures()
                .iter()
                .any(|f| f.archetype == "bad" && f.reason.contains("mutually exclusive")),
            "got: {:?}",
            r.failures()
        );
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
    fn module_collision_in_tolerant_mode_emits_diagnostic() {
        let p1 = tmpdir("col-1");
        let p2 = tmpdir("col-2");
        write_minimal_module(&p1.join("dup"), "dup");
        write_minimal_module(&p2.join("dup"), "dup");
        let r = Registry::load_from(&[&p1, &p2]).expect("tolerant ok");
        assert!(r
            .diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::DuplicateModuleName { .. })));
        // First-wins still resolves bare names.
        assert!(r.archetype("foo").is_some());
    }

    #[test]
    fn module_collision_in_strict_mode_returns_error() {
        let p1 = tmpdir("strict-1");
        let p2 = tmpdir("strict-2");
        write_minimal_module(&p1.join("dup"), "dup");
        write_minimal_module(&p2.join("dup"), "dup");
        let err = Registry::load_strict(&[&p1, &p2]).expect_err("strict collision");
        assert!(matches!(err, QuireError::ModuleCollision { .. }));
    }

    // FR-013-AC-12: load_module treats its argument as a single module
    // (manifest.yaml directly under it), distinct from load_from which
    // treats it as a search root whose children are candidate modules.
    #[test]
    fn load_module_loads_single_module_directly() {
        let parent = tmpdir("lm-ok");
        let module_root = parent.join("solo");
        fs::create_dir_all(&module_root).unwrap();
        write_minimal_module(&module_root, "solo");
        // Drop a real sibling module under `parent`. load_from would
        // happily pick it up; load_module MUST ignore it.
        let sibling = parent.join("noise");
        fs::create_dir_all(&sibling).unwrap();
        write_minimal_module(&sibling, "noise");

        let r = Registry::load_module(&module_root).expect("ok");
        let mods: Vec<&str> = r.module_names().collect();
        assert_eq!(mods, vec!["solo"]);
        assert!(!mods.contains(&"noise"));
        assert!(r.archetype("foo").is_some());
    }

    // FR-013-AC-13: load_module against a directory with no manifest
    // surfaces a single ArchetypeLoadFailure rather than walking the
    // parent dir as a search root.
    #[test]
    fn load_module_without_manifest_reports_failure() {
        let parent = tmpdir("lm-nomanifest");
        let module_root = parent.join("empty");
        fs::create_dir_all(&module_root).unwrap();
        // No manifest.yaml. Drop a real sibling — it MUST NOT load.
        write_minimal_module(&parent.join("other"), "other");

        let r = Registry::load_module(&module_root).expect("tolerant");
        assert_eq!(r.module_names().count(), 0);
        assert_eq!(r.failures().len(), 1);
        assert!(r.failures()[0].reason.contains("manifest.yaml"));
    }

    // FR-036: manifest lint_rules are typed at load and surface via
    // Registry::lint_rules(), in load order.
    #[test]
    fn lint_rules_flow_from_manifest_to_registry() {
        let parent = tmpdir("lint");
        let module_root = parent.join("mod-lint");
        fs::create_dir_all(module_root.join("schemas")).unwrap();
        fs::write(
            module_root.join("manifest.yaml"),
            r#"
name: mod-lint
artifact_types: []
lint_rules:
- type: table_column_values
  id: ac-verification-method
  archetypes: [FR]
  section: Acceptance Criteria
  column: Verification
  allowed: [Inspection, Analysis, Demonstration, Test]
  annotation_pattern: '\(TC-\d+\)'
  severity: warning
"#,
        )
        .unwrap();
        let r = Registry::load_module(&module_root).expect("ok");
        assert_eq!(r.lint_rules().len(), 1);
        assert_eq!(r.lint_rules()[0].id(), "ac-verification-method");
    }

    // FR-036: a malformed lint rule fails manifest parse (typed, not
    // inert passthrough).
    #[test]
    fn malformed_lint_rule_fails_module_load() {
        let parent = tmpdir("lint-bad");
        let module_root = parent.join("mod-lint-bad");
        fs::create_dir_all(&module_root).unwrap();
        fs::write(
            module_root.join("manifest.yaml"),
            "name: mod-lint-bad\nartifact_types: []\nlint_rules:\n- type: bogus_rule\n  id: x\n",
        )
        .unwrap();
        let r = Registry::load_module(&module_root).expect("tolerant");
        assert_eq!(r.module_names().count(), 0);
        assert_eq!(r.failures().len(), 1);
    }

    // FR-014-AC-4: module_version surfaces the manifest version.
    #[test]
    fn module_version_surfaces_manifest_version() {
        let parent = tmpdir("ver");
        let module_root = parent.join("mod-v");
        fs::create_dir_all(&module_root).unwrap();
        fs::create_dir_all(module_root.join("schemas")).unwrap();
        fs::write(
            module_root.join("manifest.yaml"),
            "name: mod-v\nversion: \"0.3.1\"\nartifact_types: []\n",
        )
        .unwrap();
        let r = Registry::load_from(&[&parent]).expect("ok");
        assert_eq!(r.module_version("mod-v"), Some("0.3.1"));
    }

    // FR-014-AC-7: manifest without `name` derives one from the parent dir.
    #[test]
    fn manifest_without_name_derives_from_parent_dir() {
        let parent = tmpdir("noname");
        let module_root = parent.join("derived");
        fs::create_dir_all(&module_root).unwrap();
        fs::write(
            module_root.join("manifest.yaml"),
            "version: \"0.0.1\"\nartifact_types: []\n",
        )
        .unwrap();
        let r = Registry::load_from(&[&parent]).expect("ok");
        assert!(r.module_names().any(|n| n == "derived"));
        assert!(r
            .diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::ManifestMissingName { .. })));
    }

    // FR-013 wasm amendment: from_inline_parts builds a registry from
    // in-memory manifest + schemas with no filesystem access (render
    // removed — no templates map).
    #[test]
    fn from_inline_parts_loads_minimal_module() {
        let manifest = b"name: mod-inline\nartifact_types:\n- name: foo\n  frontmatter_schema_ref: schemas/foo.schema.json\n";
        let mut schemas = std::collections::BTreeMap::new();
        schemas.insert(
            "schemas/foo.schema.json".to_string(),
            r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#
                .to_string(),
        );
        let r = Registry::from_inline_parts(manifest, &schemas).expect("ok");
        assert!(r.archetype("foo").is_some());
        assert_eq!(r.module_names().collect::<Vec<_>>(), vec!["mod-inline"]);
        // Schema round-trips byte-faithfully.
        let s = r.schema_for("foo").expect("schema");
        assert_eq!(s["required"][0], "id");
    }

    #[test]
    fn from_inline_parts_missing_schema_aggregates_failure() {
        let manifest = b"name: mod-inline-missing\nartifact_types:\n- name: foo\n  frontmatter_schema_ref: schemas/missing.json\n";
        let schemas = std::collections::BTreeMap::new();
        let r = Registry::from_inline_parts(manifest, &schemas).expect("tolerant");
        assert!(r.archetype("foo").is_none());
        assert_eq!(r.failures().len(), 1);
        assert!(r.failures()[0].reason.contains("not provided"));
    }

    // FR-013 wasm amendment: a manifest declaring `template_ref` is
    // rejected as a retired field even via the inline loader.
    #[test]
    fn from_inline_parts_rejects_template_ref() {
        let manifest = b"name: mod-inline-tpl\nartifact_types:\n- name: foo\n  template_ref: templates/foo.md.j2\n  frontmatter_schema_ref: schemas/foo.schema.json\n";
        let mut schemas = std::collections::BTreeMap::new();
        schemas.insert(
            "schemas/foo.schema.json".to_string(),
            r#"{"type":"object"}"#.to_string(),
        );
        let r = Registry::from_inline_parts(manifest, &schemas).expect("tolerant");
        assert!(r.archetype("foo").is_none());
        assert_eq!(r.failures().len(), 1);
        assert!(r.failures()[0].reason.contains("template_ref"));
    }

    // FR-003-AC-1: schema_for returns the loaded schema document.
    #[test]
    fn schema_for_returns_loaded_schema() {
        let parent = tmpdir("sf");
        write_minimal_module(&parent.join("m"), "m");
        let r = Registry::load_from(&[&parent]).expect("ok");
        let s = r.schema_for("foo").expect("schema");
        assert_eq!(s["type"], "object");
        assert_eq!(s["required"][0], "id");
    }

    // FR-003-AC-2: schema_for of unknown name returns UnknownArchetype.
    #[test]
    fn schema_for_unknown_returns_unknown_archetype() {
        let parent = tmpdir("sf-unknown");
        write_minimal_module(&parent.join("m"), "m");
        let r = Registry::load_from(&[&parent]).expect("ok");
        let err = r.schema_for("nope").expect_err("unknown");
        assert!(matches!(err, QuireError::UnknownArchetype { .. }));
    }

    // FR-031-AC-5 (render removed): a manifest declaring `template_ref`
    // is rejected at load — the archetype does not register.
    #[test]
    fn template_ref_is_rejected_at_load_time() {
        let parent = tmpdir("tpl");
        let module_root = parent.join("tpl-mod");
        fs::create_dir_all(module_root.join("schemas")).unwrap();
        fs::write(
            module_root.join("manifest.yaml"),
            "name: tpl-mod\nartifact_types:\n- name: foo\n  template_ref: templates/foo.md.j2\n  frontmatter_schema_ref: schemas/foo.schema.json\n",
        )
        .unwrap();
        fs::write(
            module_root.join("schemas/foo.schema.json"),
            r#"{"type":"object"}"#,
        )
        .unwrap();
        let r = Registry::load_from(&[&parent]).expect("ok");
        // The archetype is NOT registered (template_ref rejected at load).
        assert!(r.archetype("foo").is_none());
        // A per-archetype failure was aggregated instead.
        assert!(r
            .failures()
            .iter()
            .any(|f| f.archetype == "foo" && f.reason.contains("template_ref")));
    }

    // FR-014-AC-2: archetype-name collision keeps the shadowed copy queryable.
    #[test]
    fn archetype_collision_keeps_shadowed_via_archetype_in_module() {
        let parent = tmpdir("arch-col");
        let a = parent.join("m1");
        let b = parent.join("m2");
        write_minimal_module(&a, "m1");
        write_minimal_module(&b, "m2");
        let r = Registry::load_from(&[&parent]).expect("ok");
        // Both modules contribute "foo"; first-wins picks one.
        let active = r.archetype("foo").expect("active");
        // The other one is reachable via archetype_in_module.
        let shadow_module = if active.module == "m1" { "m2" } else { "m1" };
        assert!(r.archetype_in_module(shadow_module, "foo").is_some());
        assert!(r
            .diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::DuplicateArchetype { .. })));
    }
}
