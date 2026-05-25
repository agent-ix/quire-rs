# Task 030: Intra-Spec Reference Resolution

Status: complete — `src/corpus/resolve.rs` (7 unit tests). Edge/Resolution types; frontmatter `relationships` + `ix://` body-link harvest; BTreeSet dedup; O(edges) hash-join → Resolved/Dangling; forward+reverse indices; DanglingReference diagnostic. Wired into Spec::from_repo.

## Scope

At corpus construction, resolve each document's reference stubs against the loaded set, producing a unified resolved/dangling edge set. Bounded to the loaded corpus — never reaches outside it.

## Subtasks
- [ ] **Harvest stubs.** From frontmatter `relationships` (`{target, type, cardinality}`) and `ix://` body links. Both feed one edge set.
- [ ] **Target-id extraction.** Lexical: `ix://<org>/<repo>/spec/<class>/<ID>` → `<ID>`; bare `<ID>` → itself. No URI fetch/authority validation.
- [ ] **Resolve.** Hash lookup of target in the corpus id index → `Resolved` (in set) or `Dangling { target_id }` (absent, incl. targets that live only in another spec). Dangling is queryable, non-fatal.
- [ ] **Dedup.** Identical `(source, target, type)` from both frontmatter and body → one edge. Same-pair different-type → kept as distinct edges.
- [ ] **Bidirectional index.** Each `Resolved` edge indexed in both source-outgoing and target-incoming maps (substrate for 031).
- [ ] **Determinism + cost.** O(edges), one lookup per stub; classification independent of doc/thread ordering (NFR-006).

## Owns
- FR-026

## Dependencies
- 029 (`Spec` + id index), FR-006 frontmatter (complete)

## Unblocks
- 031 (query API), 033 (resolution fuzz)

## Deliverables
- `Edge` type + `Resolution` enum; resolved + reverse edge indices on `Spec`
- Resolution diagnostics (dangling) surfaced via `Spec::diagnostics()`

## Primary Tests
- TC-486 (frontmatter edge), TC-487 (ix:// edge), TC-488 (dangling), TC-489 (cross-spec dangling, no IO), TC-490 (bidirectional), TC-491 (target-id extraction), TC-492 (O(edges) proptest), TC-501 (dedup)

## Notes
- The previously-removed FR-015 edge-harvesting is NOT being resurrected: that was unbounded relationship harvesting; this is bounded join within one loaded spec (StR-006 / ADR-0002). Keep the boundary tight.
- Reuse the body-link parsing already present for `ix://` if any exists; otherwise a small lexical scan — do not pull in a URI crate.
