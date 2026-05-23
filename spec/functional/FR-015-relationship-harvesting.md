---
id: FR-015
title: "Relationship Harvesting + Edge Deduplication"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/filament-parser-lib"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL harvest relationship edges from both **frontmatter sugar fields** and **declarative `emit_edges` entries** (FR-011), normalize their targets, and deduplicate them by `(source_ref, type, target_ref)`. This mirrors FR-115 in `filament-parser-lib`.

### Frontmatter sugar fields

When a parsed `QuireDocument`'s frontmatter contains any of the canonical sugar keys below, the harvester emits one edge per declared target:

| Sugar field | Edge type | Shape |
|---|---|---|
| `depends_on` | `depends_on` | list of refs |
| `parent` | `parent` | scalar ref |
| `parent_process` | `parent` | scalar ref (alias) |
| `template_for` | `template_for` | list of refs |
| `archetype_for` | `archetype_for` | list of refs |
| `replaced_by` | `replaced_by` | scalar ref |

In addition, a structured `relationships:` block (as in the existing iso archetype frontmatter) emits edges directly: each entry's `{ target, type, cardinality }` becomes one edge with metadata.

### Target normalization

Targets may be either:

- Bare IDs (e.g. `"FR-014"`) — normalized to `ix://<org>/<repo>/<name>` using a caller-supplied `RelationshipResolver`. The resolver receives `(org_hint, repo_hint, bare_id)` and returns a canonical `ix://` URI.
- Full `ix://...` URIs — passed through the resolver for validation (it may reject unknown hosts or emit a diagnostic).

Unresolvable bare IDs emit `Diagnostic::UnresolvedRelationshipTarget`. The edge is still emitted with the bare-ID target preserved so downstream code can decide whether to drop or repair.

### Edge deduplication

After collection from all sources (frontmatter sugar + `relationships:` block + `emit_edges` from FR-011 DSL evaluation), edges are deduplicated by the tuple `(source_ref, type, target_ref)`. Duplicates are removed and a `Diagnostic::DuplicateEdge { source, type, target, sources: [a, b, ...] }` is emitted listing the original origins.

### Public API

```rust
pub trait RelationshipResolver {
    fn resolve(&self, org_hint: Option<&str>, repo_hint: Option<&str>, bare_id: &str)
        -> Result<String, QuireError>;
}

pub fn harvest_edges(
    doc: &QuireDocument,
    source_ref: &str,
    extraction: Option<&ExtractionResult>,
    resolver: &dyn RelationshipResolver,
) -> EdgeHarvest;

pub struct EdgeHarvest {
    pub edges: Vec<HarvestedEdge>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct HarvestedEdge {
    pub source_ref: String,
    pub edge_type: String,
    pub target_ref: String,
    pub metadata: serde_json::Map<String, Value>,
}
```

When `extraction` is `None`, only frontmatter sources are harvested. When `Some`, edges from `emit_edges` are unioned in and then deduplicated.

## Acceptance

- **FR-015-AC-1**: A document with frontmatter `depends_on: [FR-012, FR-013]` and a `RelationshipResolver` that maps bare IDs to `ix://agent-ix/this-repo/<id>` emits exactly two edges with type `depends_on` and the canonical URIs.
- **FR-015-AC-2**: A document with both `depends_on: [FR-014]` in frontmatter AND a `relationships:` block entry `{ target: "ix://agent-ix/this-repo/FR-014", type: "depends_on" }` emits one edge (deduped) and one `Diagnostic::DuplicateEdge` listing both sources.
- **FR-015-AC-3**: A `parent_process: "X"` sugar field emits an edge with `edge_type: "parent"` (aliased), not `"parent_process"`.
- **FR-015-AC-4**: A bare ID that the resolver cannot map emits a `Diagnostic::UnresolvedRelationshipTarget` and still produces an edge with the bare ID as target.
- **FR-015-AC-5**: A parity test against filament-parser-lib's `relationships.py` module asserts equivalent edge sets on a corpus of real artifacts.
