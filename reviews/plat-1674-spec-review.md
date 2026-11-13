---
id: SR-20261113073340
title: "Spec review — PLAT-1674 FR-058 'Narrowing the obligation by an edge' (quire-rs PR 667)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@18c3bb9; spec/functional/FR-058-upward-trace-completeness.md diff (table row, section 'Narrowing the obligation by an edge', AC-23..28, CON-4); context: spec-objects-safety FR-302 (origin/main), filament-core-data FR-149 and schema/semantic/v1/module-manifest.schema.json (origin/main), quire-rs FR-068"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-058", type: references}
---

## Summary

Ticket: PLAT-1674. This spec review covers the FR-058 text diff. The new section and AC-23..28 were checked against safety FR-302's declared `when_any` shape, and FR-058's dependencies against the manifest contract.

## Verdict

**PASS with one medium finding.** The text matches FR-302 exactly:

- the condition shape `{verb, direction: outgoing|incoming, other: [kinds]}`
- any-of scope over a resolved edge
- dangling and wrong-kind edges hold nothing
- intersection with `where`/`exclude`
- load-time rejection naming the relation
- `undeclared-relation-vocabulary` for a dead kind

The ACs are atomic and testable, and CON-4 restates CON-2/CON-3 for the new key.

The gap is in dependencies, not in the text. The module-manifest contract that mirrors the typed readers does not admit `when_any`, and nothing owns that change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-058's Dependencies name neither the manifest contract nor the downstream consumer. The authoritative module-manifest schema is filament-core-data `schema/semantic/v1/module-manifest.schema.json`. spec-artifacts-iso no longer ships one (PLAT-902, iso `dffc449`). In that schema `RequiredRelation` sets `additionalProperties: false` and has no `when_any`. FR-149 states the schema mirrors quire-rs's typed readers. The `where` precedent was FR-149-AC-10 / PLAT-1482. No ticket exists for `when_any`; Linear searches found none. Measured: neither quire-rs's loader (`src/semantic/contract.rs` validates only the `semantic` block) nor quoin's installer (`quoin-semantic` `SemanticValidators::load` compiles only `/properties/semantic`) applies the full schema. So a module can declare `when_any` today without either one refusing it. The contract is out of step with the reader, though, and any full-schema validation rejects the safety manifest. Fix: name filament-core-data FR-149 (upstream contract) and spec-objects-safety FR-302 (downstream) in Dependencies, and open a filament-core-data ticket mirroring PLAT-1482 for `RequiredRelation.when_any`. | spec/functional/FR-058-upward-trace-completeness.md (Dependencies) |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-058 relation-key table row `when_any` | examined | Clean |
| FR-058 section 'Narrowing the obligation by an edge' | examined | Clean; matches FR-302 |
| FR-058-AC-23..AC-28 | examined | Atomic, testable; test-binding defects are in the gap-analysis SR |
| FR-058-CON-4 | examined | Clean |
| FR-058 Dependencies | examined | FND-001 |
| spec-objects-safety FR-302 Scope | context_only | Source of the shape |
| filament-core-data FR-149 / module-manifest schema RequiredRelation | context_only | No `when_any` |
| quire-rs FR-068 Behavior (required-relation observation) | context_only | Applicability rule the projection violates (code-review SR FND-001) |
