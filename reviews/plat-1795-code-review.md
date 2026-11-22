---
id: SR-2026111921333400000002
title: "Code review — quire-rs PLAT-1795 FR-068 obligation export fidelity"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; tests/it/assurance_export.rs; FR-053; FR-068-AC-2, FR-068-AC-8, FR-068-CON-4; AP-201"
review_set: subset
---

## Summary

Reviewed PR #705's test-only obligation export oracle against FR-053 and FR-068. The test asserts every exported obligation field and locator against independent expected records, then checks round-trip equality, stable ordering, and invariance under an unrelated document edit; no defect was found.

## Verdict

**PASS** — the focused regression test and formatting check pass, and the diff preserves the existing exporter and fixtures.

## Assurance Context

AP-201 applies because the changed test exercises minted obligation records in the published assurance projection. The profile's relevant impact scenario is a plausible export that omits or changes source-backed records. The added fixture and independent expected vector address that scenario for the fields, locator, identity, order, and unrelated-edit behaviors in scope. The focused test runs through the real quire-rs exporter and reader.

Available context: AP-201 and its impact scenario; FR-053 and FR-068 contracts; the focused test result. No architecture description, change-impact assessment, producer-reliance record, or declared exception applies to this test-only slice. The profile's controlled-corpus, mutation, and full-gate evidence were not run; the dispatch requires name-filtered focused tests only. No broader assurance claim is made.

## Review Scope

| Unit | Role | Evidence |
| --- | --- | --- |
| FR-053 | context_only | Obligation records are derived from declared source rows and carry source identity, statement, method, parameters, and criticality. |
| FR-068-AC-2 | examined | Every obligation record preserves FR-053's id, document, statement, method, parameters, criticality, and target ids exactly, and its row locator selects the source statement. |
| FR-068-AC-8 | examined | Two projections over identical authoritative inputs serialize byte-identically, and changing an unrelated document does not change any other artifact, obligation, symbol, or relation identity. |
| FR-068-CON-4 | examined | The projection SHALL sort records by their stable identity tuple and encode every map with observable order deterministically. |
| AP-201 | context_only | Scope includes row minting and published schemas; its relevant impact scenario is a plausible coverage/export population with missing records. |
| tests/it/assurance_export.rs | examined | `obligations_preserve_every_field_across_rendered_and_configured_identities` constructs repeated statements across multiple source documents, checks each `AssuranceObligation` field and locator, round-trips the export, reverses document creation order, and edits an unrelated document. |

## Review Evidence

- Frozen head: `9fa7dc23239ce0809c15b4ac4e4ac79693e65643`.
- Focused test: `/home/peter/dev/worktrees/locked-build.sh /home/peter/dev/worktrees/plat-1795-review-quire-rs-target cargo test --locked --test it obligations_preserve_every_field_across_rendered_and_configured_identities -- --nocapture` — exit 0, 1 passed, 955 filtered.
- The focused test build emitted an unrelated optional test-module fetch warning for `@agent-ix/spec-objects-architecture@0.10.3`; the named test did not use that module. No missing dependency is counted as evidence for this change.
- `rustfmt --check tests/it/assurance_export.rs` — exit 0.
- `quire matrix --scope . --format tsv` showed the new test bound to FR-068-AC-2 and FR-068-AC-8. No repository-wide strict matrix or aggregate gate was run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
