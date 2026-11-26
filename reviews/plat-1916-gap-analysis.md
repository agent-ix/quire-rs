---
id: SR-967175026302197768900474181941935084872
title: "Gap analysis — PLAT-1916 semantic type resolution"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; FR-070-AC-4 and its implementation/tests (PR #735 head 58ef3db4b3a9d102bdbf6634e29c9f9b8d92b5a7)"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-070", type: references }
---

## Summary

The PLAT-1916 change exercises FR-070-AC-4, but no passing test currently verifies the changed stable-ID resolution under that criterion. The prior name-based expected values fail, while the new test that asserts the stable ID is tagged to FR-070-AC-1.

## Verdict

**FAIL** — the changed identity behavior lacks a passing, correctly traced assertion on the reviewed head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-070-AC-4 has no passing, correctly traced assertion for stable sibling Type identity | FR-070-AC-4 |

## Coverage

- Reconciliation: `quoin matrix --repo <repo> --json` (`quoin 0.36.10`); FR-070-AC-4 is statically tagged by existing tests.
- `tests/it/semantic_properties.rs::type_cells` is tagged to FR-070-AC-4 but fails because it expects title-based identity. `generated_cells_validate_or_diagnose` passes but does not assert the resolved identity. The new exact identity assertion in `indexed_validation_resolves_same_repo_types_and_preserves_single_document_mode` is tagged to AC-1 (with AC-11 for Clock), not AC-4.
- Reverse-gap inspection of the changed production entry points found no new unowned public behavior or stubs. Plan completion: not assessed.

## Findings detail

The matrix's static binder presence is insufficient for this changed behavior: the directly relevant `type_cells` evidence fails on its old expected target, the passing generated test does not check identity, and the new exact target assertion is misbound. The full `cargo test -p quire-rs` run reproduced the failing `type_cells` assertion at the unchanged test path.
