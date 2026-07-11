---
id: SR-001
title: "Gap analysis — issue #127 malformed-frontmatter parse failure"
type: SpecReview
analysis: gap-analysis
scope: "spec/functional/FR-040-filament-core-extraction-engine.md, spec/functional/FR-006-frontmatter-with-fallback.md, spec/tests.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-040", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-006", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: references }
---

## Summary

Post-implementation gap analysis of the issue #127 change, which makes the Filament
extraction engine emit a `parse_failed` error when a document has a complete-but-unparsable
frontmatter fence block (so the failure reaches Filament's `index_errors` via
`MarkStale → apply_stale`). The change spans the parser (a new typed `FrontmatterStatus` on
`extract_frontmatter`, FR-006/CR-010) and the Filament boundary (`FR-040-AC-6`). Mechanical
gates (matrix backing, reverse-gap) pass; two low-severity process observations are recorded.
The optional semantic review was skipped at the user's request.

## Verdict

**CONDITIONAL** — no incomplete tasks, no unbacked matrix Test Cases, and no high/medium
findings. FND-002 (traceability) has been **resolved** post-review; FND-001 remains as an
accepted, non-blocking process note (no plan bundle for a goal-driven bugfix, consistent with
how FR-040/FR-041 themselves landed).

## Findings

| ID      | Severity | Summary                                                                                                   | Refs                              |
| ------- | -------- | --------------------------------------------------------------------------------------------------------- | --------------------------------- |
| FND-001 | low      | The #127 change has no owning `plan/tasks/NNN` bundle entry (goal-driven fix); FR-040/FR-041 also landed without plan-task tracking. Accepted — change is complete and verified regardless. | plan/tasks/README.md, FR-040      |
| FND-002 | low      | RESOLVED — the CR-010 empty/null→`Absent` classification now has a dedicated `FR-006-AC-7 → TC-658`, and the parser test carries the `FR-006-AC-7 / TC-658` tracking tag. Coverage updated 340 → 341. | FR-006-AC-7, TC-658               |

## Coverage

- **Tasks done:** N/A — no formal `plan/tasks/` bundle owns FR-040/FR-041 or the #127 fix
  (see FND-001). No task is incomplete or blocked.
- **Matrix Test Cases backed by tagged test:** 2 / 2 new rows. `TC-657` is backed by the
  `tc657_malformed_frontmatter_is_a_parse_failure_but_absent_is_clean` unit test
  (`src/filament.rs`) and the `["TC-657"]`-tagged `frontmatter-unparsable-error` fixture in
  `tests/fixtures/filament_core/graph_cases.json`, exercised by the corpus harness (value +
  determinism). `TC-658` (added to resolve FND-002) is backed by the
  `empty_or_comment_only_frontmatter_is_absent_not_malformed` parser test carrying the
  `FR-006-AC-7 / TC-658` tag. Both `FR-040-AC-6 → TC-657` and `FR-006-AC-7 → TC-658` mappings
  are present in the AC→TC audit table; the coverage count is consistently updated
  (339 → 341, integrity note aligned). No existing matrix row was disturbed.
- **Untraced behaviors / stubs:** 0. All new code has an owning requirement — the
  `FrontmatterStatus` enum and `status` field trace to FR-006 §5 (CR-010); the
  `malformed_frontmatter_result` / `absent_frontmatter_result` helpers and the
  `parse_failed` / `frontmatter_unparsable` emission trace to FR-040-AC-6 and the split
  FR-040 behavior bullet. The two incidental clippy fixes (`Tier2Extraction` type alias,
  `stable_id` borrow) are behavior-preserving and carry no spec surface.
- **Semantic review:** skipped (user opted out). Intent↔test↔code for the FR-040-AC-6 triple
  was nonetheless spot-checked during code-review: `TC-657` asserts the exact AC behavior
  (empty nodes/edges, `parse_failed` in `errors`, `frontmatter_unparsable` error-severity
  diagnostic; absent stays clean) against the real `extract_filament_core` path (no mocks).
