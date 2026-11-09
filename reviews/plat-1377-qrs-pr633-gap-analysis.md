---
id: SR-20261103118149
title: "Gap analysis — PLAT-1377 relock semantic-core 0.4.0 and repoint fixture refs (quire-rs PR 633)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; Cargo.lock, crates/quire-fixtures/fixtures/**, schemas/output/semantic-v1.schema.json, tests/fixtures/semantic/cases.json"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-1377. PR head: `bccc1ef`. Plan completion: not assessed (planless). The PR adds no requirement, criterion, test or production source. It relocks the embedded semantic-core bundle and repoints test-data and output-schema `$ref`s. The gap question is whether any spec text, criterion or test still pins the old version, and whether the changed fixtures are still exercised.

- Spec: no file under `spec/` names `0.3.3` or any semantic-core version. FR-069 resolves a semantic-core `$ref` "to the embedded bundle's file of that name whatever its version segment" and says "a module declares no semantic-core version", so no CR is needed. This matches the PR body's claim, which was re-measured.
- Tests that consume the changed fixtures: `tests/it/semantic_boundary.rs` reads `tests/fixtures/semantic/cases.json`; the record-sections and module-ok fixtures are read through the `quire-fixtures` crate. The embedded version is read at runtime (`embedded_semantic_core_version()`, used by `resolver.rs`, `properties.rs` and `clauses.rs`), so no test asserts a literal version.
- Independent validation against the 0.4.0 bundle (see SR-20261103118148 Summary) found 0 unresolved refs and 0 mismatching cases.

## Verdict

**Clean.** No untraced or orphaned criterion, no stub, no coverage inflation. The change is confined to data whose meaning FR-069 already makes version-agnostic.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-069 version-agnostic `$ref` resolution | examined | Holds; no spec change needed |
| FR-069 "a module declares no semantic-core version" | examined | Holds; TC-1599 still covers it |
| `tests/it/semantic_boundary.rs` over `cases.json` | examined | Consumes the repointed fixtures |
| `spec/**` grep for a semantic-core version | examined | None |
