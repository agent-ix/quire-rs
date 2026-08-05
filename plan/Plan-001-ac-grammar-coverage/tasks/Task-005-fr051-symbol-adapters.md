---
id: Task-005
title: "FR-051 — language adapters + stable symbol identities"
type: Task
status: completed
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
- [x] **Parser choice + adapters.** Static parsing only — no build, no type
  resolution, no dependency installation, no execution (FR-051-CON-1).
- [x] **Identity + record ids.** Reformat-stable, rename-local (TC-742).
- [x] **Test classification.** All three conventions (TC-743).
- [x] **Extraction walk.** Deterministic file order; per-file diagnostics
  (TC-741, TC-749).

## Deliverables
- `src/symbols` (working name) adapters + identity model; fixture trees for
  all three languages; tests tagged TC-741..743, TC-749.

## Notes
- Parallel-ready: no dependency on Task-004 (marker binding is Task-006).
- Adapter parser crates must clear `deny.toml` license/registry gates.
- Unblocks: Task-006.

## Implementation record (2026-08-04)

- `src/symbols/` — `mod.rs` (identity, tree walk, per-file degradation) plus
  `rust.rs`, `python.rs`, `typescript.rs`. Adapters are **line-structural**
  (brace depth for Rust/TS, indentation for Python) rather than parser-backed:
  that is what "syntax level, no build, no type resolution" buys, and it keeps
  the extractor dependency-free and deterministic. No new crate dependency.
- Identity = `(language, repo-relative `/`-path, qualified name, kind)` hashed
  with the FR-045 `stable_id` convention (NUL-separated SHA-256). Line numbers
  and formatting are excluded; `line` is carried as a non-identity attribute
  alongside `leading_line`/`end_line`, the span the Task-006 trace binder reads
  via `Symbol::attached_source`.
- Test classification: Rust `#[test]`-family attributes (any attribute path
  whose last segment is `test`, so `#[tokio::test]` counts), Python `test_`
  functions plus `test_` methods of `Test*` classes, TypeScript `test`/`it`
  registrations **titled by their registration string**.
- `SymbolExtraction` retains each file's source so downstream tasks slice spans
  without re-reading the tree; symbols sort by `(path, line, qualified_name)`.
- Per-file degradation: an unreadable file or one whose braces do not balance
  yields a `SymbolDiagnostic` and is skipped; siblings extract normally.
- Fixtures: `tests/fixtures/symbols/{rust,python,typescript,broken}`.
- TC-741, TC-742, TC-743, TC-749 green; `make ci` green.
