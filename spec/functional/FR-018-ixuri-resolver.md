---
id: FR-018
title: "Reference IxUriResolver Implementation"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-015"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL ship a reference implementation of `RelationshipResolver` (FR-015) that handles the canonical `ix://<org>/<repo>/<id>` URI scheme. This removes the burden from common-case consumers who would otherwise need to author their own resolver before they can call `harvest_edges`.

### IxUriResolver

```rust
pub struct IxUriResolver {
    org_hint: String,
    repo_hint: String,
}

impl IxUriResolver {
    pub fn new(org: impl Into<String>, repo: impl Into<String>) -> Self;
}

impl RelationshipResolver for IxUriResolver {
    fn resolve(&self, org_hint: Option<&str>, repo_hint: Option<&str>, bare_id: &str)
        -> Result<String, QuireError>
    {
        // Bare ID → ix://<org>/<repo>/<id> using the resolver's defaults + caller hints
        // Full ix://... URI → pass through after structural validation
    }
}
```

### Behavior contract

1. **Bare ID input**: if `bare_id` matches `^[A-Z]{2,4}-[0-9]+$` (the canonical artifact ID pattern), return `ix://<org>/<repo>/<bare_id>` using `org_hint` arg if `Some`, else the resolver's `org_hint`, else error. Same for repo.
2. **Full `ix://` URI input**: validate the URI structure (3 path components after `ix://`); return unchanged.
3. **Unrecognized input** (neither bare ID matching pattern nor `ix://` URI): return `QuireError::UnresolvedTarget { input: bare_id.into() }`.
4. **Purity**: no I/O, no global state. Pure function of inputs.
5. **Panic-free**: as required by FR-015's resolver contract.

### Convenience constructor

```rust
impl IxUriResolver {
    pub fn from_archetype_module(registry: &Registry, archetype_name: &str)
        -> Result<Self, QuireError>
    {
        // Look up the archetype's module's `name` field;
        // derive (org, repo) from the module name if it follows the
        // "agent-ix/<repo>" pattern, else error.
    }
}
```

This is sugar; consumers can always construct the resolver explicitly.

## Acceptance

- **FR-018-AC-1**: `IxUriResolver::new("agent-ix", "this-repo").resolve(None, None, "FR-014")` returns `Ok("ix://agent-ix/this-repo/FR-014")`.
- **FR-018-AC-2**: `IxUriResolver::new("agent-ix", "this-repo").resolve(Some("other-org"), Some("other-repo"), "FR-014")` returns `Ok("ix://other-org/other-repo/FR-014")` (caller hints override defaults).
- **FR-018-AC-3**: `IxUriResolver::new("a", "b").resolve(None, None, "ix://x/y/Z-1")` returns `Ok("ix://x/y/Z-1")` (pass-through after validation).
- **FR-018-AC-4**: `IxUriResolver::new("a", "b").resolve(None, None, "garbage")` returns `Err(QuireError::UnresolvedTarget)`.
- **FR-018-AC-5**: `IxUriResolver` is `Send + Sync` and implementing `RelationshipResolver` (compile-time assertion).
- **FR-018-AC-6**: A proptest invokes `IxUriResolver::resolve` from 64 threads concurrently; no panic, byte-identical outputs.
