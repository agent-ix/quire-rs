---
id: Task-010
title: "Corpus cleanup — legacy trace-tag migration (user-gated removal)"
type: Task
status: not_started
track: C
priority: P2
relationships:
  - target: ix://agent-ix/quire-rs/Task-007
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
  - target: ix://agent-ix/quire-rs/TC-753
    type: verifies
---
# Task-010: Corpus cleanup — legacy trace-tag migration (user-gated removal)

## Scope

**Corpus/quality work class — own branch (`chore/trace-tag-migration`), never
mixed with feature branches.** After Gate G2: run the Task-006 extractor over
this repo's test tree, apply its mechanical marker-rewrite suggestions
(legacy `// TC-041` comments, `Trace:` lines, `tc657_*` name embeddings →
`#[trace(...)]` markers), and verify via `quire coverage` that the rewritten
tree yields identical `verifies` relations with zero remaining `legacy`
provenance in this repo.

## Subtasks
- [ ] **External prerequisite.** The Rust no-op `#[trace]` proc-macro support
  crate (companion deliverable, separate workspace/repo) must exist and be
  consumable — flag and stop if unpublished.
- [ ] **Mechanical rewrite.** Apply suggestions; no test-behavior changes;
  `make ci` green; relation-set equality before/after (coverage report diff).
- [ ] **Removal proposal.** When this repo (and any agreed consumers) show
  zero `legacy` relations, propose removing legacy textual-tag recognition
  entirely — no compat path retained. **STOP: removal is user-gated
  (FR-051-CON-3) — do not remove recognition without explicit sign-off.**

## Deliverables
- Migrated test tree on the cleanup branch; before/after coverage-relation
  diff; a removal proposal awaiting user decision.

## Notes
- Python/TS migrations (pytest plugin, npm helper) apply to other repos and
  ship with those external packages — out of scope here.
- Removal, once approved, is a Track A-class engine change (delete the legacy
  parser + its TCs via CR note), not part of this cleanup branch.
