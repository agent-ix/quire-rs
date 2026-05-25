# Task 031: Whole-Spec Query API + Gate G5

Status: complete — `src/corpus/query.rs` (6 unit tests) + `tests/spec_dogfood.rs` (5 tests). by_id/by_type/referencing/outgoing/orphans/dangling; untyped→UntypedArtifact; sorted/deterministic. **Gate G5 PASS** — corpus loads quire-rs's own spec/ (≥50 artifacts), FR-023..027 each resolve `implements`→StR, reverse lookup + dangling all correct.

## Scope

Expose read-only whole-spec queries over the resolved corpus, then run the G5 dogfood gate against `quire-rs`'s own `spec/` tree.

## Subtasks
- [ ] **Direct lookups.** `by_id` (O(1)), `by_type` (frontmatter `type`/`artifact_type` string match).
- [ ] **Untyped handling.** Doc with no type field → never in `by_type`, reachable by `by_id`, emits `Diagnostic::UntypedArtifact` (FR-027-AC-9).
- [ ] **Edge navigation.** `outgoing(id)` (incl. dangling), `referencing(id)` (resolved reverse only), `dangling()`.
- [ ] **Coverage/traceability.** `orphans(of_type, missing_edge_type, toward_type)` — docs of a type lacking a resolved outgoing edge of a kind (optionally toward a target type).
- [ ] **Determinism.** All iterators yield sorted-by-id; two runs identical (NFR-006).
- [ ] **Scope guard.** Read-only; no traversal/query DSL; no transitive-closure precompute. Callers compose `outgoing`/`referencing` for reachability.
- [ ] **Query bench.** `benches/corpus_query.rs`: by_id/referencing/orphans sub-ms over 200-artifact corpus (TC-458).
- [ ] **G5 dogfood test.** Load `spec/` into a `Spec`; assert real-spec facts (every FR-023..027 has resolved `implements`→StR; FR-021 body mention is not a false orphan; no in-set dangling).

## Owns
- FR-027

## Dependencies
- 030 (resolved edge set + reverse index)

## Unblocks
- 032 (bindings) — **via Gate G5**

## Deliverables
- Query methods on `Spec`; `benches/corpus_query.rs`; `tests/spec_dogfood.rs` (G5)

## Primary Tests
- TC-493 (by_type), TC-494 (referencing), TC-495 (orphans), TC-496 (coverage), TC-497 (dangling agreement), TC-498 (sorted determinism), TC-499 (no-IO), TC-500 (untyped), TC-458 (query bench)

## Gate G5: Corpus correctness (dogfood)
- **Measures:** resolution + queries correct on the real `quire-rs/spec/` corpus.
- **Pass criteria:** TC-480..501 green + `tests/spec_dogfood.rs` assertions pass.
- **If fails:** stop before 032. Root-cause in id derivation (028) or target-id extraction (030) against the real spec diff.

## Notes
- The dogfood test doubles as living documentation of what a "healthy" spec looks like and will catch resolution regressions cheaply on every CI run.
