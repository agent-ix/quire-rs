---
id: SR-001
title: "Gap analysis — canonical Filament core-data extraction (FR-045/FR-046/NFR-020)"
type: SpecReview
analysis: gap-analysis
scope: "spec/functional/FR-045-filament-core-extraction-engine.md, spec/functional/FR-046-filament-extraction-bindings.md, spec/non-functional/NFR-020-filament-extraction-boundary.md, spec/usecase/US-016-canonical-filament-extraction.md, spec/functional/FR-006-frontmatter-with-fallback.md, spec/tests.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-045", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-046", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: references }
---

## Summary

Code-review + gap-analysis of the **rebased-onto-main** canonical Filament extraction
feature (originally reviewed on a stale branch under FR-040/FR-041; re-verified here under
the landed IDs FR-045/FR-046/US-016/NFR-020 + FR-006-AC-7). The engine core is well
covered; review surfaced that several binding/static Test Cases were marked ✅ without
in-repo backing — feasible ones were backed, the rest honestly re-marked.

## Verdict

**CONDITIONAL** — no incorrect behavior; the engine core (FR-045) is backed by 20 unit +
fixture tests. All matrix over-claims found in review were resolved: two were genuinely
backed but untagged (fixed), two gained real in-repo tests (fixed), and three
downstream/CI/inspection TCs were honestly downgraded to 🚧 to match main's convention.

## Findings

| ID      | Severity | Summary                                                                                                      | Refs                          |
| ------- | -------- | ------------------------------------------------------------------------------------------------------------ | ----------------------------- |
| FND-001 | medium   | RESOLVED — TC-703/704 (corpus isolation + determinism) were backed by the harness tests but carried no tracking tag; added `// TC-703`/`// TC-704` tags. | TC-703, TC-704                |
| FND-002 | medium   | RESOLVED — TC-686 (Python binding parity) and TC-690 (NFR-020-AC-1 static boundary) had no in-repo test; added `tests/python/test_bindings.py` filament cases (TC-686, CI-run) and `tests/filament_boundary_audit.rs` (TC-690). | TC-686, TC-690, NFR-020-AC-1  |
| FND-003 | low      | ACCEPTED — TC-687 (`@agent-ix/quire-wasm` exports), TC-688 (binding no-policy inspection), TC-689 (wasm-target compile) have no in-repo test; verified downstream / by CI wasm-target / by inspection. Downgraded ✅→🚧 with method notes to match main's convention for Static/Compile TCs. | TC-687, TC-688, TC-689        |
| FND-004 | low      | NOTE — a pre-existing `useless_borrows_in_formatting` failure in `tests/parser_real_documents.rs` (surfaced by CI's toolchain bump to rust 1.97) was fixed incidentally so the gate passes; not part of the feature. | tests/parser_real_documents.rs |

## Coverage

- **Engine (FR-045):** TC-681..685 (unit), TC-691..702 + TC-705 (fixtures), TC-703/704
  (harness isolation + determinism), TC-706 (FR-006-AC-7 frontmatter status) — all backed
  by tagged tests, verified under rust 1.97 (387 tests, clippy `-D warnings`, fmt clean).
- **Bindings (FR-046):** TC-686 backed by a new Python binding test (CI `python.yml`);
  TC-687/688/689 verified downstream/CI/inspection (🚧).
- **Boundary (NFR-020):** TC-690 backed by a new static-inspection test; TC-704 (determinism)
  and TC-686 (native-value parity) backed.
- **Reverse gap:** 0 untraced code — all new code owns a requirement (FR-045/FR-046/NFR-020,
  FR-006-AC-7 via CR-011). No stale FR-040/FR-041/US-015/CR-010/TC-63x IDs leaked into code
  or the new spec files (verified by diff scan).
- **Semantic review:** not run (mechanical gates only); engine edge-harvesting confirmed
  architecturally independent of main's FR-040 object-edge vocabulary (standalone boundary,
  no registry).
