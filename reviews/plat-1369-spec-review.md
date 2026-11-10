---
id: SR-20261103118152
title: "Spec review — PLAT-1369 FR-069 semantic_core wording removal (quire-rs PR 636)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-1369. PR head `3e3bf39`. Integrity and consistency check of the wording change; no new requirement statement, so EARS and other sub-analyses do not apply.

- FR-069 Behavior: "SHALL NOT refuse a module over a `semantic_core` value" is deleted; the bullet now reads "A module declares no semantic-core version; the loader SHALL use the one embedded bundle." This no longer promises tolerance of the key, so it agrees with the code, which now refuses it through the generic `semantic.unknown-key` rule in the next bullet.
- FR-069-AC-2: the parenthetical about the removed `semantic_core` clause is gone; the criterion text is otherwise unchanged.
- spec/tests.md TC-1600: the matching parenthetical is gone; title, kind, priority and AC link unchanged. TC-1599 needed no edit (its row never named `semantic_core`).
- `git grep semantic_core -- spec/` on the head: no hits. US-019 ("names no semantic-core version") stays consistent.
- `quire validate` on both changed spec files: clean.

## Verdict

**Clean.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-069 Behavior semantic-core bullet | examined | Consistent with code |
| FR-069-AC-2 | examined | Clean |
| spec/tests.md TC-1600 row | examined | Clean, matches AC-2 |
| US-019 | context_only | Consistent |
