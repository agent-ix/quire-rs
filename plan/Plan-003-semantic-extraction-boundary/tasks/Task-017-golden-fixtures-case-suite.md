---
id: Task-017
title: "Vendored golden fixtures and case-suite skeleton"
type: Task
status: todo
track: B
priority: P1
relationships:

  - target: ix://agent-ix/quire-rs/FR-070
    type: references
  - target: ix://agent-ix/quire-rs/FR-071
    type: references
  - target: ix://agent-ix/quire-rs/FR-072
    type: references

---
# Task-017: Vendored golden fixtures and case-suite skeleton

## Scope

Pin the quoin mapping fixtures read-only and lay out the declarative semantic case suite so Tasks 018..020 have their oracles before they start.

## Subtasks

- [ ] Copy `tests/fixtures/semantic-module/mapping/` and `corpus/config-service/` from agent-ix/quoin @ 3e842ce into `tests/fixtures/semantic/quoin/` with a `PROVENANCE.json` (repository, revision, per-file sha256) and a test that recomputes it.
- [ ] Author `config-version.bundle.json` (FR-006 → `ConfigVersion`, FR-005 → `ConfigOverlay`).
- [ ] Create `tests/fixtures/semantic/cases.json` skeleton (`name`, `issue_ref`, `tags`, `input`, `expect`) and extend the attribution test to cover it.

## Deliverables

- `tests/fixtures/semantic/quoin/**`, `tests/fixtures/semantic/quoin/PROVENANCE.json`
- `tests/fixtures/semantic/config-version.bundle.json`, `tests/fixtures/semantic/cases.json`

## Notes

- Fixtures are never edited; a needed change goes back to agent-ix/quoin and is re-vendored.
