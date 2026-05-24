# Task 023: Reference IxUriResolver (FR-018)

Status: blocked on Task 017 (relationship harvesting)

## Scope

Ship `IxUriResolver` as a reference `RelationshipResolver` implementation. Removes the burden from common-case consumers who would otherwise need to author their own resolver before they can call `harvest_edges`.

## Subtasks

- [ ] **Type.** `IxUriResolver { org_hint: String, repo_hint: String }`. `Send + Sync`.
- [ ] **Constructor.** `new(org, repo) -> Self`.
- [ ] **Convenience constructor.** `from_archetype_module(&Registry, &str)` for the common case.
- [ ] **resolve impl.** Bare ID → canonical `ix://`. Full URI → pass-through after structural validation. Garbage → `UnresolvedTarget`.
- [ ] **Purity.** No I/O. Pure function. Panic-free per FR-015.

## Owns

FR-018 (6 ACs).

## Dependencies

Task 017 (relationship harvesting; this is the sibling reference impl).

## Unblocks

Common-case consumer integration without custom resolver authoring.

## Deliverables

- `src/edges/resolver.rs` (extension of FR-015 module)
- Reference example in crate-level docs

## Primary Tests

TC-310, TC-311, TC-312, TC-313, TC-314, TC-315.

## Notes

- This is small. Could be folded into Task 017 if convenient.
