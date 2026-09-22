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
//! on `filament-core-data` (public repo, tag `semantic-schema-v0.1.0`), not
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

/// The semantic-core version `agent-ix-semantic-schema`'s pinned tag embeds
/// (filament-core-data's `packages/semantic-core/package.json` at that tag).
/// Update this alongside the crate's `tag` in `Cargo.toml` when repinning —
/// `tests/semantic_baseline.rs`'s bundle content digest (TC-1606) is what
/// actually enforces the two stay in sync: a repin that changes the bundle
/// bytes without updating this constant fails that test, not silently drifts.
const EMBEDDED_SEMANTIC_CORE_VERSION: &str = "0.3.0";

/// Semantic-core versions with an embedded bundle, ascending. Exactly one:
/// `0.1.0`/`0.2.0` never left the private `npm.ix` dev-mirror and were never
/// really published anywhere this engine can reach; `0.3.0` is the first
/// real publish (PLAT-899) and the only version a module manifest can
/// meaningfully declare.
pub const SEMANTIC_CORE_VERSIONS: &[&str] = &[EMBEDDED_SEMANTIC_CORE_VERSION];

/// `@agent-ix/semantic-schema`'s `semantic/v1/module-manifest.schema.json`.
pub const MODULE_MANIFEST_SCHEMA: &str = agent_ix_semantic_schema::MODULE_MANIFEST;

/// The embedded bundle for `version`, if any.
pub fn semantic_core_bundle(version: &str) -> Option<&'static [(&'static str, &'static str)]> {
    if version == EMBEDDED_SEMANTIC_CORE_VERSION {
        Some(agent_ix_semantic_schema::SEMANTIC_CORE_SCHEMAS)
    } else {
        None
    }
}
