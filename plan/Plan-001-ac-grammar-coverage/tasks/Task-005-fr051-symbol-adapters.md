---
id: Task-005
title: "FR-051 — language adapters + stable symbol identities"
type: Task
status: not_started
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
  - target: ix://agent-ix/quire-rs/TC-741
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-742
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-743
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-749
    type: verifies
---
# Task-005: FR-051 — language adapters + stable symbol identities

## Scope

Syntax-level source-symbol extraction for Rust, Python, and TypeScript: per
file, functions, test functions, and containers, each with a stable identity
(language, repo-relative path, qualified symbol path, kind — no line numbers
or formatting in the identity; line carried as a non-identity attribute) and a
SHA-256 record id per the FR-045 convention. Test classification per language
convention (`#[test]`-family, `test_` prefix/test-class methods, `test()`/
`it()` registrations with the registered title as qualified name).
Per-file degradation: unparseable file → diagnostic, skip, continue
(FR-051-CON-2).

## Subtasks
- [ ] **Parser choice + adapters.** Static parsing only — no build, no type
  resolution, no dependency installation, no execution (FR-051-CON-1).
- [ ] **Identity + record ids.** Reformat-stable, rename-local (TC-742).
- [ ] **Test classification.** All three conventions (TC-743).
- [ ] **Extraction walk.** Deterministic file order; per-file diagnostics
  (TC-741, TC-749).

## Deliverables
- `src/symbols` (working name) adapters + identity model; fixture trees for
  all three languages; tests tagged TC-741..743, TC-749.

## Notes
- Parallel-ready: no dependency on Task-004 (marker binding is Task-006).
- Adapter parser crates must clear `deny.toml` license/registry gates.
- Unblocks: Task-006.
