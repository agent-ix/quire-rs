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
    /// A surface-supplied `grammar_severity` map layered over the
    /// module-declared one (FR-048-AC-5, `--severity`). Held beside `Arc<Inner>`
    /// rather than inside it so a registry stays cheap to clone and `Inner`
    /// stays non-`Clone` — the loaded module set is immutable, and this is the
    /// one policy knob a caller may vary per invocation.
    severity_override: Option<crate::grammar::GrammarSeverityMap>,
}

struct Inner {
    archetypes: std::collections::BTreeMap<String, Arc<CompiledArchetype>>,
    by_module_and_name: std::collections::BTreeMap<(String, String), Arc<CompiledArchetype>>,
    module_paths: std::collections::BTreeMap<String, PathBuf>,
    module_versions: std::collections::BTreeMap<String, Option<String>>,
    lint_rules: Vec<crate::lint::LintRule>,
    edge_types: std::collections::BTreeMap<String, crate::vocab::EdgeTypeDef>,
    inverse_edges: std::collections::BTreeMap<String, String>,
    roles: std::collections::BTreeMap<String, crate::vocab::RoleDef>,
    /// Merged concrete-term lexicon (FR-043) + its precompiled matcher.
    lexicon: std::collections::BTreeMap<String, crate::vocab::LexiconTermDef>,
    lexicon_matcher: crate::grammar::GrammarLexicon,
    /// Merged per-check grammar severity registry (FR-048).
    grammar_severity: crate::grammar::GrammarSeverityMap,
    /// Merged observable-result verb registry (FR-047) + its precompiled
    /// matcher (built-in defaults ∪ module declarations).
    observable_verbs: std::collections::BTreeMap<String, crate::vocab::ObservableVerbDef>,
    observable_verbs_matcher: crate::grammar::ObservableVerbs,
    /// Merged vacuous-predicate registry (FR-047, CR-014) + its matcher.
    vacuous_predicates: std::collections::BTreeMap<String, crate::vocab::VacuousPredicateDef>,
    vacuous_predicates_matcher: crate::grammar::VacuousPredicates,
    /// Merged property-idiom registry (FR-052) + its matcher.
    property_idioms: std::collections::BTreeMap<String, crate::vocab::PropertyIdiomDef>,
    property_idioms_matcher: crate::grammar::property::PropertyIdioms,
    /// Merged verification-method catalog (FR-054).
    verification_catalog: std::collections::BTreeMap<String, crate::vocab::VerificationMethodDef>,
    /// Sorted catalog keys — the `verification_method` vocabulary, derived from
    /// the catalog at construction rather than authored a second time (CON-4).
    verification_methods: Vec<String>,
    /// Sorted distinct `class` values — the `verification_class` vocabulary,
    /// derived the same way.
    verification_classes: Vec<String>,
    /// Merged ambiguity lexicon (FR-056).
    ambiguity_terms: std::collections::BTreeMap<String, crate::vocab::AmbiguityTermDef>,
    /// Precompiled matcher, module terms layered over the engine built-ins.
    ambiguity_terms_matcher: crate::grammar::quality::AmbiguityTerms,
    /// Merged declarative traceability model (FR-050).
    traceability: crate::traceability::TraceabilityModel,
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
    /// Empty `paths` falls back to `IX_FILAMENT_MODULES_PATH` /
    /// `IX_SCHEMA_PATH` / `~/.ix/filament/modules/`
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

    /// Load from `IX_FILAMENT_MODULES_PATH` / `IX_SCHEMA_PATH` (then the
    /// default `~/.ix/filament/modules/`).
    pub fn from_env() -> Result<Self, QuireError> {
        let outcome = load_modules(&[]);
        Ok(Self::finish_tolerant(outcome))
    }

    /// Load from `~/.ix/filament/modules/` only.
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
            failures,
            diagnostics,
            path_diagnostics,
        } = shape;
        // FR-043: precompile the matcher once from the merged lexicon keys.
        let lexicon_matcher =
            crate::grammar::GrammarLexicon::from_terms(lexicon.keys().map(String::as_str));
        // FR-047: precompile the observable-verb matcher once, layering the
        // merged module registry over the engine's built-in defaults.
        let observable_verbs_matcher = crate::grammar::ObservableVerbs::with_module_verbs(
            observable_verbs.keys().map(String::as_str),
        );
        // CR-014: likewise precompile the vacuity matcher once.
        let vacuous_predicates_matcher = crate::grammar::VacuousPredicates::with_module_predicates(
            vacuous_predicates.keys().map(String::as_str),
        );
        // FR-052: and the property-idiom matcher, phrase → shape, layered over
        // the engine's built-in idioms first-wins.
        let property_idioms_matcher = crate::grammar::property::PropertyIdioms::with_module_idioms(
            property_idioms
                .iter()
                .map(|(phrase, def)| (phrase.as_str(), def.shape)),
        );
        // FR-054-CON-4: both vocabularies are *derived* from the merged
        // catalog here, never read from a separate declaration. A second
        // authored copy is the duplication this FR exists to remove.
        let verification_methods: Vec<String> = verification_catalog.keys().cloned().collect();
        let verification_classes: Vec<String> = {
            let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            for method in verification_catalog.values() {
                set.insert(method.class.clone());
            }
            set.into_iter().collect()
        };
        // FR-056: precompile the ambiguity matcher once, module terms layered
        // over the engine built-ins first-wins.
        let ambiguity_terms_matcher = crate::grammar::quality::AmbiguityTerms::with_module_terms(
            ambiguity_terms.keys().map(String::as_str),
        );
        // FR-060: dereference named vocabularies in body-extraction asserts.
        //
        // HERE and not earlier: the vocabulary a contract names may be declared
        // by a DIFFERENT MODULE than the archetype naming it, so resolution
        // cannot happen at module compile time — only after the cross-module
        // merge, which is exactly this point. And here rather than in the
        // evaluator, so `evaluate_assert` keeps its signature and the
        // per-document hot path never sees a vocabulary name at all.
        let lookup = |name: &str| -> Vec<String> {
            named_vocabulary(
                name,
                &traceability.vocabularies.test_type,
                &verification_methods,
                &verification_classes,
            )
            .to_vec()
        };
        let archetypes = crate::loader::vocabulary_refs::resolve_vocabularies(archetypes, &lookup);
        let by_module_and_name =
            crate::loader::vocabulary_refs::resolve_vocabularies(by_module_and_name, &lookup);
        Self {
            inner: Arc::new(Inner {
                archetypes,
                by_module_and_name,
                module_paths,
                module_versions,
                lint_rules,
                edge_types,
                inverse_edges,
                roles,
                lexicon,
                lexicon_matcher,
                grammar_severity,
                observable_verbs,
                observable_verbs_matcher,
                vacuous_predicates,
                vacuous_predicates_matcher,
                property_idioms,
                property_idioms_matcher,
                verification_catalog,
                verification_methods,
                verification_classes,
                ambiguity_terms,
                ambiguity_terms_matcher,
                traceability,
                failures,
                diagnostics,
                path_diagnostics,
            }),
            severity_override: None,
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

    /// Merged edge-type registry (FR-040), first-wins across modules.
    pub fn edge_types(&self) -> &std::collections::BTreeMap<String, crate::vocab::EdgeTypeDef> {
        &self.inner.edge_types
    }

    /// Merged role registry (FR-040), first-wins across modules.
    pub fn roles(&self) -> &std::collections::BTreeMap<String, crate::vocab::RoleDef> {
        &self.inner.roles
    }

    /// Merged concrete-term lexicon (FR-043), first-wins across modules. The
    /// EARS object-aware vague-response check consumes these as accepted
    /// concrete objects; the engine carries no hardcoded noun list.
    pub fn lexicon(&self) -> &std::collections::BTreeMap<String, crate::vocab::LexiconTermDef> {
        &self.inner.lexicon
    }

    /// The precompiled matcher over the merged lexicon (FR-043), passed to the
    /// grammar on the registry-backed validation path.
    pub fn lexicon_matcher(&self) -> &crate::grammar::GrammarLexicon {
        &self.inner.lexicon_matcher
    }

    /// Merged per-check grammar severity registry (FR-048), first-wins across
    /// modules: `<grammar>:<check>` → `off` | `warning` | `error`. The grammar
    /// framework keys each emitted finding against this map; an absent key
    /// means `warning`, so an empty map is the all-default map.
    pub fn grammar_severity(&self) -> &crate::grammar::GrammarSeverityMap {
        self.severity_override
            .as_ref()
            .unwrap_or(&self.inner.grammar_severity)
    }

    /// Return a registry whose `grammar_severity()` is `severity`, sharing the
    /// same loaded module set. This is how a surface layers its `--severity`
    /// overrides over the module-declared map (FR-048-AC-5): merge with
    /// [`crate::grammar::merge_severity_overrides`] first, then install the
    /// result here.
    ///
    /// Cheap — the `Arc<Inner>` is shared, not copied. Called once per
    /// invocation before any document is read, never on a hot path.
    #[must_use]
    pub fn with_grammar_severity(&self, severity: crate::grammar::GrammarSeverityMap) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            severity_override: Some(severity),
        }
    }

    /// Merged observable-result verb registry (FR-047), first-wins across
    /// modules. The `ac` grammar's `unclassifiable` and `vacuous-outcome`
    /// checks consume these on top of the engine's built-in defaults (CR-014).
    pub fn observable_verbs(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::vocab::ObservableVerbDef> {
        &self.inner.observable_verbs
    }

    /// The precompiled observable-verb matcher (FR-047): built-in defaults ∪
    /// the merged module registry.
    pub fn observable_verbs_matcher(&self) -> &crate::grammar::ObservableVerbs {
        &self.inner.observable_verbs_matcher
    }

    /// Merged vacuous-predicate registry (FR-047, CR-014), first-wins across
    /// modules. The `ac` grammar's `vacuous-outcome` check consumes these on top
    /// of the engine's built-in vacuity set.
    pub fn vacuous_predicates(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::vocab::VacuousPredicateDef> {
        &self.inner.vacuous_predicates
    }

    /// The precompiled vacuity matcher (FR-047, CR-014).
    pub fn vacuous_predicates_matcher(&self) -> &crate::grammar::VacuousPredicates {
        &self.inner.vacuous_predicates_matcher
    }

    /// The merged property-idiom registry (FR-052), first-wins across modules.
    pub fn property_idioms(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::vocab::PropertyIdiomDef> {
        &self.inner.property_idioms
    }

    /// The precompiled property-idiom matcher: engine built-ins with the
    /// merged module registry layered over them (FR-052-AC-8).
    pub fn property_idioms_matcher(&self) -> &crate::grammar::property::PropertyIdioms {
        &self.inner.property_idioms_matcher
    }

    /// The merged declarative traceability model (FR-050), or `None` when no
    /// active module declares one — consumers report the model as **undeclared**
    /// rather than computing an empty rollup (FR-050-AC-2/AC-9).
    pub fn traceability(&self) -> Option<&crate::traceability::TraceabilityModel> {
        (!self.inner.traceability.is_empty()).then_some(&self.inner.traceability)
    }

    /// The merged `ambiguity_terms` registry (FR-056).
    pub fn ambiguity_terms(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::vocab::AmbiguityTermDef> {
        &self.inner.ambiguity_terms
    }

    /// The precompiled ambiguity matcher (FR-056): module terms layered over
    /// the engine built-ins, which are never replaced.
    pub fn ambiguity_terms_matcher(&self) -> &crate::grammar::quality::AmbiguityTerms {
        &self.inner.ambiguity_terms_matcher
    }

    /// Implements: FR-054
    /// The merged verification-method catalog (FR-054), or `None` when no
    /// active module declares one — consumers report the catalog as
    /// **undeclared** rather than as containing no methods, exactly as
    /// [`traceability`](Registry::traceability) does for its model.
    pub fn verification_catalog(
        &self,
    ) -> Option<&std::collections::BTreeMap<String, crate::vocab::VerificationMethodDef>> {
        (!self.inner.verification_catalog.is_empty()).then_some(&self.inner.verification_catalog)
    }

    /// The declared vocabulary for a named column (CR-015, widened by FR-054).
    /// Empty when no active module declares one — the caller reports the
    /// vocabulary as undeclared rather than inventing a default.
    ///
    /// `verification_method` and `verification_class` are **derived from the
    /// merged catalog**, so they cannot drift from it: they *are* it. That is
    /// what makes the catalog a single source rather than a fourth copy of the
    /// same vocabulary (FR-054-CON-4).
    pub fn column_vocabulary(&self, column: &str) -> &[String] {
        named_vocabulary(
            column,
            &self.inner.traceability.vocabularies.test_type,
            &self.inner.verification_methods,
            &self.inner.verification_classes,
        )
    }

    /// Compose an ad-hoc `GrammarLexicon` (FR-044) from the merged module
    /// lexicon keys plus `extra` project terms (a repo's harvested
    /// Ubiquitous-Language vocabulary). Project terms are per-repo, so they are
    /// never stored on the immutable `Registry` — this builds a fresh matcher.
    pub fn lexicon_with(&self, extra: &[String]) -> crate::grammar::GrammarLexicon {
        let module = self.inner.lexicon.keys().map(String::as_str);
        let project = extra.iter().map(String::as_str);
        crate::grammar::GrammarLexicon::from_terms(module.chain(project))
    }

    /// Inverse-label → forward-verb index (FR-041). A declared `inverse:`
    /// label is an authorable verb (a derived view of its forward edge);
    /// this maps each such label to the forward verb that declared it.
    /// Excludes labels that are themselves forward `edge_types` keys (the
    /// forward registration governs).
    pub fn inverse_index(&self) -> &std::collections::BTreeMap<String, String> {
        &self.inner.inverse_edges
    }

    /// Implements: FR-041
    /// Resolve the edge vocabulary for a document with artifact archetype
    /// `artifact` and optional `object:` archetype (FR-040-AC-6).
    ///
    /// The result is the **union** of both axes' `allowed_links`: a verb
    /// allowed by either axis is allowed; for a verb on both, the target
    /// lists are unioned and `"*"` absorbs concrete/role tokens. With
    /// `object = None` it returns the artifact vocabulary alone.
    pub fn resolve_allowed_links(
        &self,
        artifact: &CompiledArchetype,
        object: Option<&CompiledArchetype>,
    ) -> crate::vocab::AllowedLinks {
        let mut out = artifact.allowed_links().clone();
        if let Some(o) = object {
            for (verb, targets) in o.allowed_links() {
                let entry = out.entry(verb.clone()).or_default();
                for t in targets {
                    if !entry.contains(t) {
                        entry.push(t.clone());
                    }
                }
            }
        }
        // `"*"` absorbs: any verb whose target list contains the wildcard
        // collapses to just `["*"]`.
        for targets in out.values_mut() {
            if targets.iter().any(|t| t == "*") {
                *targets = vec!["*".to_string()];
            }
        }
        out
    }

    /// True when `token` (a target token from a verb's allowed list) is
    /// satisfied by `candidate` (the resolved target archetype):
    /// `token == "*"`, `token` equals the candidate's name, or `token` is
    /// a role the candidate carries (FR-040-AC-7).
    pub fn target_satisfies(&self, token: &str, candidate: &CompiledArchetype) -> bool {
        token == "*" || token == candidate.name || candidate.roles().iter().any(|r| r == token)
    }

    /// Path-resolution diagnostics (missing dirs, file-not-dir,
    /// permission-denied entries). Always advisory.
    pub fn path_diagnostics(&self) -> &[PathDiagnostic] {
        &self.inner.path_diagnostics
    }
}

/// The values behind a vocabulary name (FR-054, FR-060).
///
/// A free function rather than a method because it is needed **twice**: by
/// `Registry::column_vocabulary` for callers, and by `Registry::from_shape`
/// while building the registry, where no `Registry` exists yet.
///
/// Sharing it is the point. Two copies of the name→vocabulary mapping would be
/// exactly the duplication FR-060 exists to remove — a contract's
/// `from_vocabulary: test_type` and a caller's `column_vocabulary("test_type")`
/// resolving through different matches is how they drift.
fn named_vocabulary<'v>(
    name: &str,
    test_type: &'v [String],
    verification_methods: &'v [String],
    verification_classes: &'v [String],
) -> &'v [String] {
    match name {
        "test_type" => test_type,
        "verification_method" => verification_methods,
        "verification_class" => verification_classes,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocab::EdgeCategory;
    use ix_trace_rs::trace;
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

    #[trace("TC-411", "FR-020-AC-2")]
    // block type is an alias over the archetype registry.
    //
    // Written when FR-020 was authored (CR-042): the row had been ✅ since v0.2
    // with nothing behind it, because the requirement it claimed had no
    // document. CON-1 is what this asserts — one registry, two names for it,
    // never a second store that can drift.
    #[test]
    fn tc411_block_type_is_an_alias_for_archetype() {
        let root = tmpdir("block-type-alias");
        // The helper names the module; its one archetype is always `foo`.
        write_minimal_module(&root, "note");
        let registry = Registry::load_module(&root).expect("load module");

        let by_block_type = registry.block_type("foo").expect("registered");
        let by_archetype = registry.archetype("foo").expect("registered");
        assert!(
            std::ptr::eq(by_block_type, by_archetype),
            "both names must resolve to the same compiled archetype, not to equal copies"
        );

        // And an unregistered name resolves through neither.
        assert!(registry.block_type("nope").is_none());
        assert!(registry.archetype("nope").is_none());
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

    #[trace("FR-013-AC-12")]
    // load_module treats its argument as a single module
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

    #[trace("FR-013-AC-13")]
    // load_module against a directory with no manifest
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

    #[trace("FR-036-AC-1")]
    // manifest lint_rules are typed at load and surface via
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

    #[trace("TC-588", "FR-036-AC-5")]
    // lint evaluation never affects extraction or
    // validation. The same document through the same archetype, once under a
    // module carrying a lint rule it violates and once under the identical
    // module without the rule, must produce byte-identical `extract()` and
    // `validate_document()` results — the rules are advisory by construction,
    // not by convention. The row claimed this since v0.4 with no test
    // (CR-058).
    #[test]
    fn tc588_lint_rules_leave_extract_and_validate_byte_identical() {
        let parent = tmpdir("lint-identity");
        let archetype = r#"
- name: FR
  frontmatter_schema_ref: schemas/fr.json
  body_extraction:
    yield_pattern:
      match:
        acceptance_criteria:
          from: section_body
          after_heading: Acceptance Criteria
          required: true
"#;
        let lint = r#"
lint_rules:
- type: table_column_values
  id: ac-verification-method
  archetypes: [FR]
  section: Acceptance Criteria
  column: Verification
  allowed: [Inspection, Analysis, Demonstration, Test]
  annotation_pattern: '\(TC-\d+\)'
  severity: warning
"#;
        // A document that VIOLATES the rule ("Docs audit" is not allowed), so
        // the lint layer is doing work rather than sitting inert.
        let doc = "---\nid: FR-001\ntype: FR\n---\n\
                   ## Acceptance Criteria\n\
                   | ID | Criteria | Verification |\n\
                   | - | - | - |\n\
                   | FR-001-AC-1 | does x | Test (TC-035) |\n\
                   | FR-001-AC-2 | does y | Docs audit |\n";

        let build = |name: &str, rules: &str| {
            let module_root = parent.join(name);
            fs::create_dir_all(module_root.join("schemas")).unwrap();
            fs::write(
                module_root.join("manifest.yaml"),
                format!("name: {name}\nartifact_types:{archetype}{rules}"),
            )
            .unwrap();
            fs::write(
                module_root.join("schemas/fr.json"),
                r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#,
            )
            .unwrap();
            Registry::load_module(&module_root).expect("module loads")
        };

        let with_rules = build("mod-with-lint", lint);
        let without_rules = build("mod-without-lint", "");

        assert_eq!(
            with_rules.lint_rules().len(),
            1,
            "the rule is loaded; failures={:?}",
            with_rules.failures()
        );
        assert!(without_rules.lint_rules().is_empty());

        let a = with_rules.archetype("FR").expect("archetype");
        let b = without_rules.archetype("FR").expect("archetype");

        // The lint layer sees a real violation under the rule-carrying module.
        let parsed = crate::parser::parse_document(doc);
        assert_eq!(
            crate::lint::lint_document(with_rules.lint_rules(), Some("FR"), &parsed).len(),
            1,
            "the fixture must actually violate the rule, or this proves nothing"
        );

        // …and neither downstream result moves a byte. `ExtractionResult`
        // and `ValidationResult` compare structurally; the debug rendering is
        // the byte-level form the row's wording asks for.
        let extract_with = crate::extract(&parsed, a.body_extraction().expect("dsl")).unwrap();
        let extract_without = crate::extract(&parsed, b.body_extraction().expect("dsl")).unwrap();
        assert_eq!(
            extract_with, extract_without,
            "lint rules changed extraction"
        );
        assert_eq!(
            format!("{extract_with:?}"),
            format!("{extract_without:?}"),
            "lint rules changed extraction output bytes"
        );

        let validate_with = crate::validate_document(a, doc);
        let validate_without = crate::validate_document(b, doc);
        assert_eq!(
            validate_with, validate_without,
            "lint rules changed validation"
        );
        assert_eq!(
            format!("{validate_with:?}"),
            format!("{validate_without:?}"),
            "lint rules changed validation output bytes"
        );
    }

    #[trace("FR-036-AC-1")]
    // a malformed lint rule fails manifest parse — the negative
    // path of "typed at load" (typed, not
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

    #[trace("FR-014-AC-4")]
    // module_version surfaces the manifest version.
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

    #[trace("FR-014-AC-7")]
    // manifest without `name` derives one from the parent dir.
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

    #[trace("FR-003-AC-1")]
    // schema_for returns the loaded schema document.
    #[test]
    fn schema_for_returns_loaded_schema() {
        let parent = tmpdir("sf");
        write_minimal_module(&parent.join("m"), "m");
        let r = Registry::load_from(&[&parent]).expect("ok");
        let s = r.schema_for("foo").expect("schema");
        assert_eq!(s["type"], "object");
        assert_eq!(s["required"][0], "id");
    }

    #[trace("FR-003-AC-2")]
    // schema_for of unknown name returns UnknownArchetype.
    #[test]
    fn schema_for_unknown_returns_unknown_archetype() {
        let parent = tmpdir("sf-unknown");
        write_minimal_module(&parent.join("m"), "m");
        let r = Registry::load_from(&[&parent]).expect("ok");
        let err = r.schema_for("nope").expect_err("unknown");
        assert!(matches!(err, QuireError::UnknownArchetype { .. }));
    }

    #[trace("FR-031-AC-5")]
    // a manifest declaring `template_ref` (render removed)
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

    // ── FR-040: object-axis typed edge vocabulary ──────────────────

    fn write_vocab_module(root: &Path, name: &str, body: &str) {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join("manifest.yaml"), format!("name: {name}\n{body}")).unwrap();
    }

    #[trace("TC-636", "FR-040-AC-1")]
    // edge_types + roles registries load and the
    // merged Registry exposes both; identical re-declaration across two
    // modules is silently idempotent (no diagnostic).
    #[test]
    fn tc636_edge_types_and_roles_load_and_merge_idempotently() {
        let p = tmpdir("vocab-636");
        let body = r#"object_types:
- name: entity
  roles: [domain-object]
edge_types:
  contains:
    description: composition
    category: structural
    inverse: part_of
roles:
  domain-object:
    description: a business-model type
"#;
        // Two modules declare the SAME edge_types/roles bodies.
        write_vocab_module(&p.join("m1"), "m1", body);
        write_vocab_module(&p.join("m2"), "m2", body);
        let r = Registry::load_from(&[&p]).expect("ok");
        assert!(r.edge_types().contains_key("contains"));
        assert_eq!(
            r.edge_types()["contains"].category,
            EdgeCategory::Structural
        );
        assert_eq!(
            r.edge_types()["contains"].inverse.as_deref(),
            Some("part_of")
        );
        assert!(r.roles().contains_key("domain-object"));
        // Idempotent: identical re-declaration emits no Duplicate diagnostic.
        assert!(!r.diagnostics().iter().any(|d| matches!(
            d,
            Diagnostic::DuplicateEdgeType { .. } | Diagnostic::DuplicateRole { .. }
        )));
    }

    #[trace("TC-637", "FR-040-AC-2")]
    // differing re-declaration is first-wins +
    // non-fatal Duplicate{EdgeType,Role}; default load still succeeds.
    #[test]
    fn tc637_conflicting_redeclaration_is_first_wins_diagnostic() {
        let p = tmpdir("vocab-637");
        write_vocab_module(
            &p.join("m1"),
            "m1",
            "edge_types:\n  calls:\n    description: sync\n    category: behavioral\n",
        );
        write_vocab_module(
            &p.join("m2"),
            "m2",
            "edge_types:\n  calls:\n    description: DIFFERENT\n    category: dataflow\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        // First-wins keeps the earliest body.
        assert_eq!(r.edge_types()["calls"].category, EdgeCategory::Behavioral);
        assert!(r
            .diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::DuplicateEdgeType { name, .. } if name == "calls")));
    }

    #[trace("TC-667", "FR-043-AC-1")]
    // a `lexicon` term loads, is readable via
    // `Registry::lexicon()`, and the precompiled matcher recognises it.
    #[test]
    fn tc667_lexicon_loads_and_accessor() {
        let p = tmpdir("lexicon-667");
        write_vocab_module(
            &p.join("m"),
            "m",
            "lexicon:\n  pagination:\n    definition: page-splitting\n  cursor:\n    definition: position token\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        assert!(r.lexicon().contains_key("pagination"));
        assert!(r.lexicon().contains_key("cursor"));
        assert!(r
            .lexicon_matcher()
            .contains_term("supports pagination today"));
        assert!(!r.lexicon_matcher().contains_term("unrelated text"));
    }

    #[trace("TC-668", "FR-043-AC-2")]
    // a term re-declared with a differing body across
    // modules is first-wins + emits one `DuplicateLexiconTerm`.
    #[test]
    fn tc668_lexicon_merge_first_wins() {
        let p = tmpdir("lexicon-668");
        write_vocab_module(
            &p.join("m1"),
            "m1",
            "lexicon:\n  pagination:\n    definition: page-splitting\n",
        );
        write_vocab_module(
            &p.join("m2"),
            "m2",
            "lexicon:\n  pagination:\n    definition: DIFFERENT\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        assert_eq!(r.lexicon()["pagination"].definition, "page-splitting"); // first-wins
        assert!(r.diagnostics().iter().any(
            |d| matches!(d, Diagnostic::DuplicateLexiconTerm { name, .. } if name == "pagination")
        ));
    }

    #[trace("TC-672", "FR-043-AC-6")]
    // the registry-backed path applies the merged
    // lexicon; the type-only path applies an empty one (more findings).
    #[test]
    fn tc672_registry_vs_type_only_lexicon_paths() {
        let p = tmpdir("lexicon-672");
        let m = p.join("m");
        fs::create_dir_all(m.join("schemas")).unwrap();
        fs::write(
            m.join("manifest.yaml"),
            "name: m\nartifact_types:\n- name: FR\n  frontmatter_schema_ref: schemas/fr.schema.json\n  grammar_ref: iso-spec-core\nlexicon:\n  pagination:\n    definition: page splitting\n",
        )
        .unwrap();
        fs::write(m.join("schemas/fr.schema.json"), r#"{"type":"object"}"#).unwrap();
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        let fr = r.archetype("FR").expect("FR archetype");
        let doc = "---\ntype: FR\n---\n## Description\n\nThe system shall support pagination.\n";
        // Registry path: lexicon has `pagination` → no vague-response.
        let with_reg = crate::validate_document_in_registry(&r, fr, doc);
        assert!(!with_reg
            .warnings
            .iter()
            .any(|w| w.message.contains("[ears:vague-response]")));
        // Type-only path: empty lexicon → vague-response present.
        let type_only = crate::validate_document(fr, doc);
        assert!(type_only
            .warnings
            .iter()
            .any(|w| w.message.contains("[ears:vague-response]")));
    }

    // ── FR-048: per-check grammar severity ─────────────────────────

    /// A module whose `FR` archetype is bound to the EARS grammar, with an
    /// optional `grammar_severity` block appended to the manifest.
    fn write_grammar_module(root: &Path, name: &str, extra: &str) {
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            format!(
                "name: {name}\nartifact_types:\n- name: FR\n  \
                 frontmatter_schema_ref: schemas/fr.schema.json\n  \
                 grammar_ref: iso-spec-core\n{extra}"
            ),
        )
        .unwrap();
        fs::write(root.join("schemas/fr.schema.json"), r#"{"type":"object"}"#).unwrap();
    }

    #[trace("TC-766", "FR-048-AC-5")]
    // a surface layers its `--severity` overrides over
    // the module-declared map. The returned registry shares the same loaded
    // module set — only the severity policy differs.
    #[test]
    fn tc766_with_grammar_severity_overrides_the_module_map() {
        use crate::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
        let p = tmpdir("severity-766");
        write_vocab_module(
            &p.join("m"),
            "m",
            "grammar_severity:\n  \"ac:unclassifiable\": error\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        assert_eq!(
            r.grammar_severity().get("ac:unclassifiable"),
            Some(&GrammarSeverityLevel::Error)
        );

        let mut over = GrammarSeverityMap::new();
        over.insert("ac:unclassifiable".into(), GrammarSeverityLevel::Off);
        let scoped = r.with_grammar_severity(over);
        assert_eq!(
            scoped.grammar_severity().get("ac:unclassifiable"),
            Some(&GrammarSeverityLevel::Off),
            "the override must win for its key"
        );
        // The module set is shared, not rebuilt...
        assert_eq!(scoped.module_names().count(), r.module_names().count());
        // ...and the original registry is untouched.
        assert_eq!(
            r.grammar_severity().get("ac:unclassifiable"),
            Some(&GrammarSeverityLevel::Error)
        );
    }

    #[trace("TC-716", "FR-048-AC-1")]
    // a manifest `grammar_severity` registry loads and
    // `Registry::grammar_severity()` returns the merged map.
    #[test]
    fn tc716_grammar_severity_loads_and_accessor() {
        let p = tmpdir("severity-716");
        write_vocab_module(
            &p.join("m"),
            "m",
            "grammar_severity:\n  \"ac:unclassifiable\": error\n  \"ac:vague-response\": off\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        assert_eq!(
            r.grammar_severity().get("ac:unclassifiable"),
            Some(&crate::grammar::GrammarSeverityLevel::Error)
        );
        assert_eq!(
            r.grammar_severity().get("ac:vague-response"),
            Some(&crate::grammar::GrammarSeverityLevel::Off)
        );
    }

    #[trace("TC-717", "FR-048-AC-2")]
    // conflicting redeclarations merge first-wins with
    // one `DuplicateGrammarSeverity`; identical redeclaration emits none.
    #[test]
    fn tc717_grammar_severity_merge_first_wins() {
        let p = tmpdir("severity-717");
        write_vocab_module(
            &p.join("m1"),
            "m1",
            "grammar_severity:\n  \"ac:unclassifiable\": error\n  \"ac:non-singular\": warning\n",
        );
        write_vocab_module(
            &p.join("m2"),
            "m2",
            // `ac:unclassifiable` conflicts; `ac:non-singular` is identical.
            "grammar_severity:\n  \"ac:unclassifiable\": off\n  \"ac:non-singular\": warning\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        // First-wins keeps the earliest level.
        assert_eq!(
            r.grammar_severity()["ac:unclassifiable"],
            crate::grammar::GrammarSeverityLevel::Error
        );
        let dups: Vec<&str> = r
            .diagnostics()
            .iter()
            .filter_map(|d| match d {
                Diagnostic::DuplicateGrammarSeverity { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(dups, vec!["ac:unclassifiable"]);
    }

    #[trace("TC-723", "FR-048-AC-8")]
    // a malformed `grammar_severity` entry fails module
    // load like any other manifest shape error.
    #[test]
    fn tc723_malformed_grammar_severity_fails_load() {
        // Unknown level.
        let p = tmpdir("severity-723-level");
        write_vocab_module(
            &p.join("m"),
            "m",
            "grammar_severity:\n  \"ac:unclassifiable\": fatal\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        assert!(r.module_names().next().is_none(), "module must not load");
        assert!(r
            .failures()
            .iter()
            .any(|f| f.archetype == "<manifest>" && f.reason.contains("fatal")));

        // Non-string key.
        let p2 = tmpdir("severity-723-key");
        write_vocab_module(&p2.join("m"), "m", "grammar_severity:\n  12: error\n");
        let r2 = Registry::load_from(&[&p2]).expect("tolerant ok");
        assert!(r2.module_names().next().is_none(), "module must not load");
        assert!(
            r2.failures()
                .iter()
                .any(|f| f.archetype == "<manifest>"
                    && f.reason.contains("grammar_severity key '12'"))
        );
    }

    #[trace("TC-722", "FR-048-AC-7")]
    // the type-only `validate_document` path applies the
    // all-default map — every grammar finding is a warning regardless of the
    // module's manifest, which promotes the same check to `error`.
    #[test]
    fn tc722_type_only_path_applies_all_default_severity() {
        let p = tmpdir("severity-722");
        write_grammar_module(
            &p.join("m"),
            "m",
            "grammar_severity:\n  \"ears:vague-response\": error\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        let fr = r.archetype("FR").expect("FR archetype");
        let doc = "---\ntype: FR\n---\n## Description\n\nThe system shall support pagination.\n";

        // Registry path: the manifest promotes the check to `error`.
        let with_reg = crate::validate_document_in_registry(&r, fr, doc);
        assert!(!with_reg.is_valid);
        assert!(with_reg
            .errors
            .iter()
            .any(|e| e.message.contains("[ears:vague-response]")));

        // Type-only path: all-default map → the same finding is a warning.
        let type_only = crate::validate_document(fr, doc);
        assert!(type_only.is_valid);
        assert!(type_only
            .warnings
            .iter()
            .any(|w| w.message.contains("[ears:vague-response]")));
        assert!(type_only.errors.is_empty());
    }

    // TC-718, FR-048-AC-3, end-to-end half: with `ac:unclassifiable` mapped
    // to `error`, a real unclassifiable criteria cell lands in
    // `ValidationResult.errors` and clears `is_valid`, while an `ears` finding
    // with no map entry stays a warning. (The framework-level contract is
    // pinned in `validate_document::tests::tc718_per_check_error_routing`.)
    #[test]
    fn tc718_ac_error_routing_end_to_end() {
        let p = tmpdir("severity-718");
        write_grammar_module(
            &p.join("m"),
            "m",
            "grammar_severity:\n  \"ac:unclassifiable\": error\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        let fr = r.archetype("FR").expect("FR archetype");
        let doc = "---\ntype: FR\n---\n## Description\n\n\
                   The system shall support publishing.\n\n\
                   ## Acceptance Criteria\n\n\
                   | ID | Criteria | Verification |\n|----|----------|--------------|\n\
                   | FR-001-AC-1 | Structural evaluation | Test |\n";
        let result = crate::validate_document_in_registry(&r, fr, doc);

        assert!(!result.is_valid, "a promoted `ac` check must block");
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("[ac:unclassifiable]")));
        // The unmapped `ears` finding on the Description stays advisory.
        assert!(result
            .warnings
            .iter()
            .any(|w| w.message.contains("[ears:vague-response]")));
        assert!(!result.errors.iter().any(|e| e.message.contains("[ears:")));
    }

    // ── FR-050: declarative traceability model ─────────────────────

    /// The repo's traceability fixture modules. They live outside
    /// `tests/fixtures/modules` on purpose: that directory is a shared search
    /// root other tests load wholesale, and a fixture declaring its own `FR`
    /// would shadow the ISO one there.
    fn fixture_modules() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("traceability")
    }

    #[trace("TC-732", "FR-050-AC-1")]
    // a manifest `traceability:` section declaring trace
    // targets, document references, a status vocabulary, and a trace-tag
    // grammar loads, and the Registry exposes the declared model.
    #[test]
    fn tc732_traceability_model_loads_and_accessor() {
        let root = fixture_modules().join("iso");
        let r = Registry::load_module(&root).expect("load");
        let model = r.traceability().expect("declared model");

        let target = model
            .target("acceptance-criterion")
            .expect("acceptance-criterion target");
        assert_eq!(target.archetype, "FR");
        assert_eq!(target.section, "Acceptance Criteria");
        assert_eq!(target.id_column, "ID");
        // CR-062: every target is archetype-bound, the Test Matrix included.
        assert_eq!(model.target("test-case").unwrap().archetype, "TestMatrix");

        let verification = model
            .document_references
            .iter()
            .find(|d| d.name == "verification")
            .expect("verification reference");
        assert_eq!(verification.column, "Verification");
        // The reference resolves against both the auxiliary matrix rows and
        // TC documents authored in the bundle.
        assert_eq!(
            verification.targets,
            vec!["test-case", "test-case-document"]
        );

        let status = model.status.as_ref().expect("status vocabulary");
        assert_eq!(status.column, "Status");
        assert_eq!(
            status.class_of("✅"),
            crate::traceability::StatusClass::Complete
        );
        assert_eq!(
            status.class_of("🚧"),
            crate::traceability::StatusClass::Pending
        );

        assert!(model
            .trace_tags
            .markers
            .iter()
            .any(|m| m.language == crate::traceability::SourceLanguage::Python));
        assert!(!model.trace_tags.legacy.is_empty());

        // The non-ISO fixture declares an entirely different vocabulary.
        let alt = Registry::load_module(&fixture_modules().join("alt"))
            .expect("load")
            .traceability()
            .cloned()
            .expect("declared model");
        assert!(alt.target("clause").is_some());
        assert_eq!(alt.status.as_ref().unwrap().column, "State");
    }

    #[trace("TC-733", "FR-050-AC-2")]
    // a malformed `traceability:` section fails module
    // load like any other manifest shape error; an absent section loads and
    // marks the model undeclared.
    #[test]
    fn tc733_malformed_model_fails_load_absent_is_undeclared() {
        // Absent section → loads, model undeclared.
        let p = tmpdir("trace-733-absent");
        write_vocab_module(&p.join("m"), "m", "artifact_types: []\n");
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        assert!(r.module_names().next().is_some(), "module must load");
        assert!(r.traceability().is_none(), "model must be undeclared");

        // Malformed: a reference to an undeclared target.
        let bad = tmpdir("trace-733-bad");
        write_vocab_module(
            &bad.join("m"),
            "m",
            "traceability:\n  document_references:\n  - name: r\n    archetype: FR\n    \
             section: S\n    column: C\n    pattern: '(TC-\\d+)'\n    targets: [nope]\n",
        );
        let r2 = Registry::load_from(&[&bad]).expect("tolerant ok");
        assert!(r2.module_names().next().is_none(), "module must not load");
        assert!(r2
            .failures()
            .iter()
            .any(|f| f.archetype == "<manifest>" && f.reason.contains("undeclared target")));

        // Malformed: an unknown field inside the section.
        let typo = tmpdir("trace-733-typo");
        write_vocab_module(
            &typo.join("m"),
            "m",
            "traceability:\n  trace_targets:\n  - name: t\n    archetype: FR\n    section: S\n    \
             id_column: ID\n    typo: x\n",
        );
        let r3 = Registry::load_from(&[&typo]).expect("tolerant ok");
        assert!(r3.module_names().next().is_none(), "module must not load");
    }

    #[trace("TC-676", "FR-044-AC-3")]
    // `lexicon_with` composes module keys ∪ project terms.
    #[test]
    fn tc676_lexicon_with_combines_module_and_project() {
        let p = tmpdir("lexicon-676");
        write_vocab_module(
            &p.join("m"),
            "m",
            "lexicon:\n  endpoint:\n    definition: an HTTP path\n",
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        let lex = r.lexicon_with(&["widget".to_string()]);
        assert!(lex.contains_term("provide an endpoint")); // module term
        assert!(lex.contains_term("provide a widget")); // project term
        assert!(!lex.contains_term("provide flexibility")); // neither
    }

    #[trace("TC-677", "FR-044-AC-4")]
    // validate_document_in_registry_with_lexicon injects
    // the supplied lexicon — a project term suppresses; module-only flags.
    #[test]
    fn tc677_with_lexicon_injection_suppresses_project_term() {
        let p = tmpdir("lexicon-677");
        let m = p.join("m");
        fs::create_dir_all(m.join("schemas")).unwrap();
        fs::write(
            m.join("manifest.yaml"),
            "name: m\nartifact_types:\n- name: FR\n  frontmatter_schema_ref: schemas/fr.schema.json\n  grammar_ref: iso-spec-core\n",
        )
        .unwrap();
        fs::write(m.join("schemas/fr.schema.json"), r#"{"type":"object"}"#).unwrap();
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        let fr = r.archetype("FR").expect("FR archetype");
        let doc = "---\ntype: FR\n---\n## Description\n\nThe system shall provide a widget.\n";
        // Module-only (empty) lexicon → `widget` is vague.
        let mod_only = crate::validate_document_in_registry(&r, fr, doc);
        assert!(mod_only
            .warnings
            .iter()
            .any(|w| w.message.contains("[ears:vague-response]")));
        // Inject a lexicon with the project term → suppressed.
        let lex = r.lexicon_with(&["widget".to_string()]);
        let injected = crate::validate_document_in_registry_with_lexicon(&r, fr, doc, &lex);
        assert!(!injected
            .warnings
            .iter()
            .any(|w| w.message.contains("[ears:vague-response]")));
    }

    #[trace("TC-650", "FR-040-AC-3")]
    // unknown verb/role → non-fatal diagnostic
    // (default load succeeds); load_strict escalates to an error.
    #[test]
    fn tc650_unknown_verb_and_role_diagnostic_and_strict_escalation() {
        let p = tmpdir("vocab-650");
        // `exposes` verb + `mystery` target-role are undeclared; the
        // `wat` role on the object type is undeclared too.
        write_vocab_module(
            &p.join("m1"),
            "m1",
            r#"object_types:
- name: api_endpoint
  roles: [wat]
  allowed_links:
    exposes: [mystery]
edge_types: {}
roles: {}
"#,
        );
        let r = Registry::load_from(&[&p]).expect("tolerant ok");
        assert!(r.diagnostics().iter().any(
            |d| matches!(d, Diagnostic::UnknownEdgeType { edge_type, .. } if edge_type == "exposes")
        ));
        assert!(r
            .diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::UnknownRole { role, .. } if role == "mystery")));
        assert!(r
            .diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::UnknownRole { role, .. } if role == "wat")));
        // Strict escalates.
        let err = Registry::load_strict(&[&p]).expect_err("strict");
        assert!(matches!(err, QuireError::EdgeVocabularyViolation { .. }));
    }

    #[trace("TC-651", "FR-040-AC-5")]
    // object `roles:` list is parsed onto the
    // compiled archetype and readable via roles(); none reads empty.
    #[test]
    fn tc651_object_roles_parsed_onto_archetype() {
        let p = tmpdir("vocab-651");
        write_vocab_module(
            &p.join("m1"),
            "m1",
            r#"object_types:
- name: aggregate_root
  roles: [domain-object, persistable]
- name: enumeration
roles:
  domain-object: { description: x }
  persistable: { description: y }
"#,
        );
        let r = Registry::load_from(&[&p]).expect("ok");
        assert_eq!(
            r.archetype("aggregate_root").unwrap().roles(),
            &["domain-object".to_string(), "persistable".to_string()]
        );
        assert!(r.archetype("enumeration").unwrap().roles().is_empty());
    }

    #[trace("TC-639", "FR-040-AC-6")]
    // resolve_allowed_links unions both axes;
    // shared verb unions targets and "*" absorbs; object=None → artifact only.
    #[test]
    fn tc639_resolve_allowed_links_unions_axes() {
        let p = tmpdir("vocab-639");
        write_vocab_module(
            &p.join("m1"),
            "m1",
            r#"artifact_types:
- name: FR
  allowed_links: [references]
object_types:
- name: aggregate_root
  allowed_links:
    references: [aggregate_root]
    emits: [event]
edge_types:
  references: { description: x, category: traceability }
  emits: { description: y, category: dataflow }
"#,
        );
        let r = Registry::load_from(&[&p]).expect("ok");
        let fr = r.archetype("FR").unwrap();
        let agg = r.archetype("aggregate_root").unwrap();
        // Union with object.
        let resolved = r.resolve_allowed_links(fr, Some(agg));
        assert!(resolved.contains_key("emits"));
        // `references`: artifact had ["*"], object adds [aggregate_root] → "*" absorbs.
        assert_eq!(resolved["references"], vec!["*".to_string()]);
        assert_eq!(resolved["emits"], vec!["event".to_string()]);
        // object=None → artifact vocabulary alone.
        let artifact_only = r.resolve_allowed_links(fr, None);
        assert!(!artifact_only.contains_key("emits"));
        assert_eq!(artifact_only["references"], vec!["*".to_string()]);
    }

    #[trace("TC-640", "FR-040-AC-7")]
    // target_satisfies by name, role, or "*".
    #[test]
    fn tc640_target_satisfies_name_role_or_star() {
        let p = tmpdir("vocab-640");
        write_vocab_module(
            &p.join("m1"),
            "m1",
            r#"object_types:
- name: entity
  roles: [domain-object]
roles:
  domain-object: { description: x }
"#,
        );
        let r = Registry::load_from(&[&p]).expect("ok");
        let entity = r.archetype("entity").unwrap();
        assert!(r.target_satisfies("*", entity));
        assert!(r.target_satisfies("entity", entity));
        assert!(r.target_satisfies("domain-object", entity));
        assert!(!r.target_satisfies("aggregate_root", entity));
        assert!(!r.target_satisfies("persistable", entity));
    }

    #[trace("FR-014-AC-2")]
    // archetype-name collision keeps the shadowed copy queryable.
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
