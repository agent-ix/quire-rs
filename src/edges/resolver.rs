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

use std::collections::BTreeMap;

use crate::error::QuireError;

/// Resolve a bare relationship target to a fully-qualified URI per
/// FR-018.
///
/// The full signature carries caller hints so a single resolver can
/// serve documents from any module:
///
/// - `org_hint` / `repo_hint`: optional hints from the calling
///   document's own ix:// URI. Implementations MAY use them as the
///   org/repo when normalizing a bare ID; resolvers carrying their
///   own defaults (e.g. [`IxUriResolver`]) override them only when
///   the hints are `None`.
/// - `bare_id`: the target as it appeared in the source document.
///   May be a bare ID (`FR-002`), a fully-qualified `ix://` URI, or
///   anything else.
///
/// Returns `Ok(uri)` on a successful normalize / pass-through, or
/// `Err(QuireError::UnresolvedTarget { target, reason })` when the
/// input can't be turned into a canonical URI. The harvester catches
/// `UnresolvedTarget`, preserves the bare value, and emits a
/// `Diagnostic::UnresolvableEdgeTarget`.
///
/// Implementations MUST be pure and panic-free.
pub trait RelationshipResolver: Send + Sync {
    fn resolve(
        &self,
        org_hint: Option<&str>,
        repo_hint: Option<&str>,
        bare_id: &str,
    ) -> Result<String, QuireError>;
}

/// Pass-through resolver — every bare ID stays bare. Useful when the
/// caller doesn't care about ix:// URIs (e.g. local-only graphs).
#[derive(Debug, Default, Clone, Copy)]
pub struct IdentityResolver;

impl RelationshipResolver for IdentityResolver {
    fn resolve(
        &self,
        _org_hint: Option<&str>,
        _repo_hint: Option<&str>,
        bare_id: &str,
    ) -> Result<String, QuireError> {
        Ok(bare_id.to_string())
    }
}

/// Lookup-based resolver for tests + simple cases. The `(org_hint,
/// repo_hint)` arguments are ignored — the map is the source of
/// truth.
#[derive(Debug, Default, Clone)]
pub struct MockResolver {
    map: BTreeMap<String, String>,
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
    fn resolve(
        &self,
        _org_hint: Option<&str>,
        _repo_hint: Option<&str>,
        bare_id: &str,
    ) -> Result<String, QuireError> {
        self.map
            .get(bare_id)
            .cloned()
            .ok_or_else(|| QuireError::UnresolvedTarget {
                target: bare_id.to_string(),
                reason: "no mapping in MockResolver".to_string(),
            })
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
    /// from the module that owns `archetype` in `registry`. Returns
    /// `Err(UnresolvedTarget)` when no such archetype is registered —
    /// callers can either fall back to [`new`] or fail the operation.
    pub fn from_archetype_module(
        registry: &crate::registry::Registry,
        archetype: &str,
    ) -> Result<Self, QuireError> {
        match registry.archetype(archetype) {
            Some(a) => Ok(Self::new("agent-ix", a.module.clone())),
            None => Err(QuireError::UnresolvedTarget {
                target: archetype.to_string(),
                reason: "archetype is not registered; cannot derive ix:// hints".to_string(),
            }),
        }
    }
}

impl RelationshipResolver for IxUriResolver {
    fn resolve(
        &self,
        org_hint: Option<&str>,
        repo_hint: Option<&str>,
        bare_id: &str,
    ) -> Result<String, QuireError> {
        let trimmed = bare_id.trim();
        if trimmed.is_empty() {
            return Err(QuireError::UnresolvedTarget {
                target: bare_id.to_string(),
                reason: "empty target".to_string(),
            });
        }
        if let Some(rest) = trimmed.strip_prefix("ix://") {
            if rest.split('/').any(|seg| seg.is_empty()) || rest.is_empty() {
                return Err(QuireError::UnresolvedTarget {
                    target: trimmed.to_string(),
                    reason: "malformed ix:// URI".to_string(),
                });
            }
            return Ok(trimmed.to_string());
        }
        if trimmed.contains("://") {
            return Err(QuireError::UnresolvedTarget {
                target: trimmed.to_string(),
                reason: format!("unsupported scheme in {trimmed}"),
            });
        }
        if trimmed.chars().any(|c| c.is_whitespace()) {
            return Err(QuireError::UnresolvedTarget {
                target: trimmed.to_string(),
                reason: "bare ID contains whitespace".to_string(),
            });
        }
        // FR-018-AC-2: caller-supplied hints override the resolver's
        // own defaults when present.
        let org = org_hint.unwrap_or(self.org_hint.as_str());
        let repo = repo_hint.unwrap_or(self.repo_hint.as_str());
        Ok(format!("ix://{org}/{repo}/{trimmed}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_returns_input() {
        let r = IdentityResolver;
        assert_eq!(r.resolve(None, None, "FR-001").unwrap(), "FR-001");
    }

    #[test]
    fn mock_round_trips_via_map() {
        let r = MockResolver::new().with("FR-001", "ix://agent-ix/spec-iso/FR-001");
        assert_eq!(
            r.resolve(None, None, "FR-001").unwrap(),
            "ix://agent-ix/spec-iso/FR-001"
        );
        assert!(matches!(
            r.resolve(None, None, "FR-999"),
            Err(QuireError::UnresolvedTarget { .. })
        ));
    }

    // FR-018-AC-1: bare ID → canonical ix:// URI using the resolver's
    // own org/repo when no caller hints are supplied.
    #[test]
    fn ix_uri_resolver_normalizes_bare_id_with_defaults() {
        let r = IxUriResolver::new("agent-ix", "spec-artifacts-iso");
        assert_eq!(
            r.resolve(None, None, "FR-001").unwrap(),
            "ix://agent-ix/spec-artifacts-iso/FR-001"
        );
    }

    // FR-018-AC-2: caller-supplied hints override resolver defaults.
    #[test]
    fn ix_uri_resolver_caller_hints_override_defaults() {
        let r = IxUriResolver::new("agent-ix", "spec-artifacts-iso");
        let out = r
            .resolve(Some("other-org"), Some("other-repo"), "X-1")
            .unwrap();
        assert_eq!(out, "ix://other-org/other-repo/X-1");
    }

    // FR-018-AC-3: full ix:// URI passes through unchanged.
    #[test]
    fn ix_uri_resolver_passes_through_full_uri() {
        let r = IxUriResolver::new("a", "b");
        assert_eq!(
            r.resolve(None, None, "ix://other-org/other-repo/X")
                .unwrap(),
            "ix://other-org/other-repo/X"
        );
    }

    // FR-018-AC-4: other schemes / malformed URIs → UnresolvedTarget.
    #[test]
    fn ix_uri_resolver_rejects_other_schemes_with_typed_error() {
        let r = IxUriResolver::new("a", "b");
        let err = r
            .resolve(None, None, "https://example.com/X")
            .expect_err("scheme");
        assert!(matches!(err, QuireError::UnresolvedTarget { .. }));
    }

    #[test]
    fn ix_uri_resolver_rejects_malformed_ix_uri() {
        let r = IxUriResolver::new("a", "b");
        assert!(matches!(
            r.resolve(None, None, "ix:///empty-segment/"),
            Err(QuireError::UnresolvedTarget { .. })
        ));
    }

    #[test]
    fn ix_uri_resolver_rejects_whitespace_in_bare_id() {
        let r = IxUriResolver::new("a", "b");
        assert!(matches!(
            r.resolve(None, None, "FR 001"),
            Err(QuireError::UnresolvedTarget { .. })
        ));
    }

    // FR-018-AC-5: Send + Sync verified at compile time.
    #[test]
    fn ix_uri_resolver_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<IxUriResolver>();
        // The trait object must also be Send + Sync.
        fn assert_dyn(_r: &(dyn RelationshipResolver + Send + Sync)) {}
        assert_dyn(&IxUriResolver::new("a", "b"));
    }

    // FR-018-AC-6: thread-safe concurrent resolve (proptest stand-in:
    // 64 threads × 256 distinct bare IDs each).
    #[test]
    fn ix_uri_resolver_concurrent_resolve_is_safe() {
        use std::sync::Arc;
        use std::thread;
        let r = Arc::new(IxUriResolver::new("agent-ix", "spec-iso"));
        let handles: Vec<_> = (0..64)
            .map(|t| {
                let r = Arc::clone(&r);
                thread::spawn(move || {
                    for i in 0..256 {
                        let bare = format!("FR-{:03}", t * 256 + i);
                        let uri = r.resolve(None, None, &bare).unwrap();
                        assert!(uri.starts_with("ix://agent-ix/spec-iso/"));
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    }
}
