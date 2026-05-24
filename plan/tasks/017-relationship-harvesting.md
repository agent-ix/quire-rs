# Task 017: Relationship Harvesting + Edge Dedup

Status: blocked on Task 016

## Scope

Implement `harvest_edges(doc, source_ref, extraction, resolver) -> EdgeHarvest` per FR-015: read frontmatter sugar fields + structured `relationships:` block + DSL `emit_edges`; normalize via resolver; dedup by `(source, type, target)`.

## Subtasks

- [ ] **Sugar field set.** `depends_on`, `parent`, `parent_process` (alias→parent), `template_for`, `archetype_for`, `replaced_by`. Defined as a static list.
- [ ] **Structured `relationships:` block.** Each entry → edge with metadata.
- [ ] **emit_edges union.** If `extraction` is `Some`, union its edges.
- [ ] **RelationshipResolver trait.** Pure + panic-free (documented).
- [ ] **Target normalization.** Bare ID → `ix://<org>/<repo>/<name>` via resolver. Unresolvable → diagnostic + preserve bare.
- [ ] **Dedup.** By `(source_ref, type, target_ref)`. First-encountered metadata wins; dropped reported in diagnostic.
- [ ] **Determinism.** Order: structured block → sugar fields (declared order) → emit_edges (DSL order). Across threads → byte-identical (TC-141).

## Owns

FR-015 (7 ACs).

## Dependencies

Task 016.

## Unblocks

(Track C complete after Task 018.)

## Deliverables

- `src/edges/{mod,resolver,harvest}.rs`

## Primary Tests

TC-100, TC-101, TC-102, TC-103, TC-104, TC-140, TC-141.

## Notes

- Reference: `~/dev/filament-parser-lib/filament_parser/relationships.py`. Parity sweep TC-104 against this.
- A reference test resolver is provided in `tests/utils/test_resolver.rs` — maps bare IDs to fixture URIs.
