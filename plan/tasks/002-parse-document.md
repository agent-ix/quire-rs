# Task 002: parse_document API + QuireDocument Types

Status: blocked on Task 001

## Scope

Define the public `QuireDocument` / `QuireSection` types and implement `parse_document(&str) -> QuireDocument` that orchestrates the primitives from Task 001 into the final shape.

## Subtasks

- [ ] **Types (FR-005).** `QuireSection { id, heading, level, content, children, start_line, end_line }` and `QuireDocument { preamble, sections, raw, frontmatter }`. `derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)`. `Send + Sync`.
- [ ] **parse_document.** Calls frontmatter::extract → walk::headings → slice::content → tree assembly via level-aware stack. Empty input → empty document; preamble-only doc → preamble set, sections empty.
- [ ] **Determinism (NFR-006).** Identical input → byte-identical output across threads. No observable HashMap.
- [ ] **No panic (FR-005-AC-4).** Proptest 10k random UTF-8 inputs; never panic.

## Owns

FR-005 (4 ACs).

## Dependencies

Task 001 (parser primitives).

## Unblocks

Task 003 (Query API), Task 004 (parser parity gate), Tasks 015-018 (DSL + edges + fallback).

## Deliverables

- `src/ast.rs` — public types
- `src/parser/mod.rs` — `parse_document` entry

## Primary Tests

TC-001, TC-002, TC-057 (parse determinism).

## Notes

- The TS heading-stack algorithm (parser.ts lines 109-130) handles level skips; mirror it exactly.
- Pure function: no IO, no `std::time`, no thread-locals.
