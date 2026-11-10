---
id: SR-20261103118151
title: "Gap analysis — PLAT-1369 remove the semantic_core strip (quire-rs PR 636)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; src/semantic/contract.rs, tests/it/semantic_contract.rs, spec/functional/FR-069-semantic-module-contract-at-load.md, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-1369. PR head `3e3bf39`. Plan completion: not assessed (planless).

- FR-069-AC-1 is backed by TC-1599 (`#[trace("TC-1599", "FR-069-AC-1")]`, `valid_block_and_reference_schema_load`), which loads module-ok with no mutation and asserts the record and resolved schema. FR-069-AC-2 is backed by TC-1600 (`unsupported_versions_are_refused_first`). Both trace tags are unchanged and both tests pass.
- The FR-069 "unknown key" refusal that a leftover `semantic_core` would now hit is already covered by TC-1601 (`block_shape_refusals_name_the_offender`); no key-specific test is wanted, per the owner's ruling.
- No production code is left without an owning requirement: the change only deletes code.
- No stub, no coverage inflation, no orphaned criterion.

## Verdict

**Clean.** The deleted behaviour had no remaining requirement after the FR-069 edit, and the remaining criteria keep their tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-069-AC-1 / TC-1599 | examined | Traced, passes |
| FR-069-AC-2 / TC-1600 | examined | Traced, passes |
| FR-069-AC-3 / TC-1601 | context_only | Covers the generic unknown-key refusal |
