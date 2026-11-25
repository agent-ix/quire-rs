---
id: SR-2026112016223475846355057
title: "Code review — PLAT-1874 owner frontmatter projection"
type: SpecReview
analysis: code-review
scope: "PLAT-1874 working-tree diff: src/semantic/surface.rs, tests/it/frontmatter_owned_keys.rs, tests/fixtures/corpus_cases/plat-1874.json; FR-069-AC-34, FR-069-AC-45, FR-069-AC-47; AP-201; live PLAT-1874 acceptance criteria"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Reviewed the generic frontmatter projection change, its registry and direct-extraction tests, the synthetic fixture, live PLAT-1874 acceptance criteria, and AP-201 context. The change preserves explicit `frontmatter_field` ownership while excluding top-level note `owner` metadata from generic record projection; no correctness or consistency finding remains.

## Assurance Context

- **Applicable profile:** AP-201, [spec/assurance/AP-201-detection-minting.md](../spec/assurance/AP-201-detection-minting.md). It applies because the change affects semantic extraction diagnostics and adds trace-bound test evidence. The profile's main population-coverage impact scenario is not activated: this change does not alter coverage population collection, row minting, or coverage metric calculation.
- **Changed behavior evaluated:** `src/semantic/surface.rs` reserves top-level `owner` only from generic record projection. The explicit-mapping path remains active. The test exercises a healthy nested typed owner, a scalar note-owner collision, and an invalid mapped leaf; its fixture contains only synthetic `example/owner-fixture`, `team-a`, and `team-b` data.
- **Evidence available:** live PLAT-1874 acceptance criteria; AP-201; targeted test results supplied for `cargo test --test it frontmatter_owned_keys` (12 passed) and `cargo test --test it tc2308` (1 passed); targeted `quire matrix` results show FR-069-AC-34, AC-45, and AC-47 tagged to tests. The strict repository-wide matrix remains non-green with 234 untagged criteria and 108 design gaps; those broad baseline results are not attributed to this change.
- **Unavailable or not run:** no AP-201-selected ArchitectureDescription, MeasurementPlan, or measurement record was available in the selected context. The controlled corpus and mutation/self-test evidence were not run for this review. The repository-wide final gate was explicitly deferred to the post-review-fix checkpoint. Quire test evidence is produced by the same engine and is not independent of it.
- **Exceptions:** AP-201 declares no active exception for this change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS** — the generic exclusion and explicit mapping remain distinct and are covered by trace-bound tests for the collision and invalid-value paths. The fixture contains only synthetic data; the deferred final gate is not represented as passed.
