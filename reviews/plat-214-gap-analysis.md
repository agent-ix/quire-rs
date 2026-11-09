---
id: SR-20261103118143
title: "Gap analysis — PLAT-214 duplicate semantic export (quire-rs PR 629)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md (AC-15), spec/tests.md (TC-2190), src/loader/mod.rs, tests/it/semantic_contract.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-214. PR head `accd773`. `quoin matrix --repo .` (quoin 0.29.1) reports FR-069-AC-15 as `tagged`, bound to `duplicate_export_refuses_the_later_module`. TC-2190 has a row in the test table and in the audit mapping. Plan completion: not assessed.

The implementation has an owning requirement (FR-069-AC-15). It contains no stub and no unowned code.

## Verdict

**Covered, with test gaps.** The test checks the AC's main clauses: the later root is refused with `semantic.duplicate-export`, the type and first module are named, the first module loads, and a disjoint pair loads. The gaps are cases the test does not exercise. One of them hides the wrong result recorded as FND-001 in the code-review SR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test covers a refused module whose other exports are contested by a third module, so the cascade in code-review FND-001 passes the suite. Add a three-module case. The assertion: a-first `[entity]` and b-second `[entity, enumeration]`, then c-third `[enumeration]` loads. | tests/it/semantic_contract.rs:644-670 |
| FND-002 | low | The PR says that no `DuplicateArchetype` follows the refusal, but the test does not assert it. A probe confirms that none is emitted today. Assert that `registry.diagnostics()` has no `DuplicateArchetype` in the refusal case. | tests/it/semantic_contract.rs:644-659 |
| FND-003 | low | The disjoint-exports control sets b-second's exports to `[]`. That passes even if the name comparison is wrong, because there is nothing to compare. Use `[enumeration]`, which the fixture declares. A probe confirms that both modules load with no failures. | tests/it/semantic_contract.rs:661-669 |
| FND-004 | low | Load-order independence ("later sorted root", "no load order selects a definition") is never exercised: both modules sit in one temp dir, and the test has only one `load_from` order. A probe with two directories passed in both orders gave the same outcome. Add it so a later change to `order` is caught. | tests/it/semantic_contract.rs:646-650 |

## Dispositions
