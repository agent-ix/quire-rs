//! Relationship target resolution (Task 017, Task 023).
//!
//! Harvested edges carry bare IDs like `FR-002`. Consumers usually
//! want them normalized to fully-qualified `ix://<org>/<repo>/<name>`
//! URIs so a downstream graph can dedup across modules. The engine
//! itself doesn't know the URI shape — Filament or `ix-cli` owns that
//! mapping — so `quire-rs` exposes a `RelationshipResolver` trait
//! that consumers implement.
//!
//! `IdentityResolver` is the no-op default (returns the bare ID
//! unchanged). Tests use a [`MockResolver`] map.

use std::collections::HashMap;

/// Resolve a bare relationship target to a fully-qualified URI.
///
/// Implementations MUST be pure and panic-free. The engine calls
/// `resolve` once per harvested edge; an `Err(reason)` surfaces as a
/// `Diagnostic::UnresolvableEdgeTarget` and the bare target is
/// preserved.
pub trait RelationshipResolver: Send + Sync {
    fn resolve(&self, bare: &str) -> Result<String, String>;
}

/// Pass-through resolver — every bare ID stays bare. Useful when the
/// caller doesn't care about ix:// URIs (e.g. local-only graphs).
#[derive(Debug, Default, Clone, Copy)]
pub struct IdentityResolver;

impl RelationshipResolver for IdentityResolver {
    fn resolve(&self, bare: &str) -> Result<String, String> {
        Ok(bare.to_string())
    }
}

/// Lookup-based resolver for tests + simple cases.
#[derive(Debug, Default, Clone)]
pub struct MockResolver {
    map: HashMap<String, String>,
}

impl MockResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, bare: impl Into<String>, uri: impl Into<String>) {
        self.map.insert(bare.into(), uri.into());
    }

    pub fn with(mut self, bare: impl Into<String>, uri: impl Into<String>) -> Self {
        self.insert(bare, uri);
        self
    }
}

impl RelationshipResolver for MockResolver {
    fn resolve(&self, bare: &str) -> Result<String, String> {
        self.map
            .get(bare)
            .cloned()
            .ok_or_else(|| format!("no mapping for '{bare}'"))
    }
}

/// Reference `RelationshipResolver` for the canonical
/// `ix://<org>/<repo>/<name>` URI shape (FR-018, Task 023).
///
/// Construct with `IxUriResolver::new("agent-ix", "spec-artifacts-iso")`
/// (or via the registry-aware [`from_archetype_module`]) and pass to
/// `harvest_edges`. Three input shapes are accepted:
///
/// - **Bare ID** (`FR-001`) → `ix://agent-ix/spec-artifacts-iso/FR-001`
/// - **Fully-qualified `ix://`** → pass-through after structural check.
/// - **Anything else** (other scheme, malformed) → `Err(reason)` which
///   the harvester turns into a `Diagnostic::UnresolvableEdgeTarget`.
///
/// Pure + `Send + Sync` + panic-free per FR-018-AC.
///
/// [`from_archetype_module`]: IxUriResolver::from_archetype_module
#[derive(Debug, Clone)]
pub struct IxUriResolver {
    pub org_hint: String,
    pub repo_hint: String,
}

impl IxUriResolver {
    pub fn new(org: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            org_hint: org.into(),
            repo_hint: repo.into(),
        }
    }

    /// Convenience for the common case: derive the (org, repo) hint
    /// from the module that owns `archetype` in `registry`. Falls back
    /// to `("agent-ix", module_name)` if we can't sniff anything richer.
    pub fn from_archetype_module(registry: &crate::registry::Registry, archetype: &str) -> Self {
        let module = registry
            .archetype(archetype)
            .map(|a| a.module.clone())
            .unwrap_or_else(|| "unknown".to_string());
        Self::new("agent-ix", module)
    }
}

impl RelationshipResolver for IxUriResolver {
    fn resolve(&self, raw: &str) -> Result<String, String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err("empty target".to_string());
        }
        if let Some(rest) = trimmed.strip_prefix("ix://") {
            // Structural check: must be /-separated with at least one
            // non-empty segment after the scheme.
            if rest.split('/').any(|seg| seg.is_empty()) || rest.is_empty() {
                return Err(format!("malformed ix:// URI: {trimmed}"));
            }
            return Ok(trimmed.to_string());
        }
        if trimmed.contains("://") {
            return Err(format!("unsupported scheme: {trimmed}"));
        }
        // Bare ID: normalize to the canonical URI.
        if trimmed.chars().any(|c| c.is_whitespace()) {
            return Err(format!("bare ID contains whitespace: {trimmed:?}"));
        }
        Ok(format!(
            "ix://{}/{}/{}",
            self.org_hint, self.repo_hint, trimmed
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_returns_input() {
        let r = IdentityResolver;
        assert_eq!(r.resolve("FR-001").unwrap(), "FR-001");
    }

    #[test]
    fn mock_round_trips_via_map() {
        let r = MockResolver::new().with("FR-001", "ix://agent-ix/spec-iso/FR-001");
        assert_eq!(
            r.resolve("FR-001").unwrap(),
            "ix://agent-ix/spec-iso/FR-001"
        );
        assert!(r.resolve("FR-999").is_err());
    }

    // FR-018-AC: bare ID → canonical ix:// URI.
    #[test]
    fn ix_uri_resolver_normalizes_bare_id() {
        let r = IxUriResolver::new("agent-ix", "spec-artifacts-iso");
        assert_eq!(
            r.resolve("FR-001").unwrap(),
            "ix://agent-ix/spec-artifacts-iso/FR-001"
        );
    }

    #[test]
    fn ix_uri_resolver_passes_through_full_uri() {
        let r = IxUriResolver::new("a", "b");
        assert_eq!(
            r.resolve("ix://other-org/other-repo/X").unwrap(),
            "ix://other-org/other-repo/X"
        );
    }

    #[test]
    fn ix_uri_resolver_rejects_other_schemes() {
        let r = IxUriResolver::new("a", "b");
        assert!(r.resolve("https://example.com/X").is_err());
    }

    #[test]
    fn ix_uri_resolver_rejects_malformed_ix_uri() {
        let r = IxUriResolver::new("a", "b");
        assert!(r.resolve("ix:///empty-segment/").is_err());
    }

    #[test]
    fn ix_uri_resolver_rejects_whitespace_in_bare_id() {
        let r = IxUriResolver::new("a", "b");
        assert!(r.resolve("FR 001").is_err());
    }

    #[test]
    fn ix_uri_resolver_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<IxUriResolver>();
    }
}
