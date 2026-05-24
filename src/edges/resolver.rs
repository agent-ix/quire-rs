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
}
