//! Schema bundle embedded at compile time (FR-069 Inputs).
//!
//! The real reason semantic-core bytes must be *embedded* (rather than read
//! at call time from a registry, a module, or the filesystem) is
//! `properties.rs`'s and `clauses.rs`'s internal gate: `field_decl_validator`
//! and `model_validator` synthesize a throwaway schema that `$ref`s straight
//! into the semantic-core bundle to validate a `FieldDecl`/`ClauseRef`/etc.
//! record quire-rs *itself* just produced, deep inside extraction with no
//! registry, module, or caller in scope (`&|_| None` for the sibling
//! resolver) — so the only bytes available to validate against are whatever
//! this module has compiled in. This is unrelated to the `wasm` feature: no
//! `wasm`-gated code path reads the filesystem, and the resolver
//! (`resolver.rs`) never did — that used to be this file's stated reason,
//! and it was false.
//!
//! Sourced from `agent-ix-semantic-schema` — a plain Cargo `git` dependency
//! on `filament-core-data` (public repo, the tag pinned in `Cargo.toml`), not
//! a `build.rs` npm fetch. That crate embeds the exact same bytes
//! `@agent-ix/semantic-core`/`@agent-ix/semantic-schema` publish to npm, via
//! `include_str!` from filament-core-data's own tree — the canonical,
//! public source. A git dependency on a public repo needs no npm, no node,
//! and no registry authentication anywhere in this crate's build, which a
//! `build.rs` npm fetch did (and which needed working `REGISTRY_TOKEN`
//! plumbing, npm/node present on every runner including inside the
//! manylinux container, and a token visible to a build-script subprocess
//! four layers deep — none of which the actual wheel build needs when the
//! source is a git dependency instead of a network fetch at build time).

/// The `semantic.contract_version` this engine understands.
pub const CONTRACT_VERSION: &str = "1.0.0";
/// Base of every semantic-core `$id`.
pub const SEMANTIC_CORE_BASE: &str = "https://schemas.agent-ix.org/semantic-core/";
/// Base of every module-emitted `$id`.
pub const MODULE_SCHEMA_BASE: &str = "https://schemas.agent-ix.org/";

/// `@agent-ix/semantic-schema`'s `semantic/v1/module-manifest.schema.json`.
pub const MODULE_MANIFEST_SCHEMA: &str = agent_ix_semantic_schema::MODULE_MANIFEST;

/// The semantic-core version of the embedded bundle, read from the bundle's
/// own `$id` (`https://schemas.agent-ix.org/semantic-core/<version>/<Name>.json`)
/// on first use. The bundle is the only source of this version.
pub fn embedded_semantic_core_version() -> &'static str {
    static VERSION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    VERSION.get_or_init(|| {
        let (_, text) = agent_ix_semantic_schema::SEMANTIC_CORE_SCHEMAS
            .first()
            .expect("the embedded semantic-core bundle is never empty");
        let doc: serde_json::Value =
            serde_json::from_str(text).expect("embedded semantic-core schemas are valid JSON");
        let id = doc["$id"]
            .as_str()
            .expect("embedded semantic-core schemas carry an $id");
        id.strip_prefix(SEMANTIC_CORE_BASE)
            .and_then(|rest| rest.split('/').next())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| panic!("embedded semantic-core $id {id:?} has no version segment"))
            .to_string()
    })
}

/// Semantic-core versions with an embedded bundle: exactly the one the
/// embedded bundle itself declares.
pub fn semantic_core_versions() -> &'static [&'static str] {
    static VERSIONS: std::sync::OnceLock<[&'static str; 1]> = std::sync::OnceLock::new();
    VERSIONS.get_or_init(|| [embedded_semantic_core_version()])
}

/// The embedded bundle for `version`, if any.
pub fn semantic_core_bundle(version: &str) -> Option<&'static [(&'static str, &'static str)]> {
    if version == embedded_semantic_core_version() {
        Some(agent_ix_semantic_schema::SEMANTIC_CORE_SCHEMAS)
    } else {
        None
    }
}
