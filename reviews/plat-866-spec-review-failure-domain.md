---
id: SR-16500
title: "failure-domain review of FR-062 Python and TypeScript structural implements syntax positions (quire-rs PR 628)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs@8db9e04; spec/functional/FR-062-implements-relation.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-062
    type: references
---

# Failure-domain review of FR-062 Python and TypeScript syntax positions (quire-rs PR 628)

## Summary

Ticket: PLAT-866. Looked for unstated failure modes in the new binder contract: identity confusion of the construct (aliased and qualified imports, same-named decorators from other libraries), composition (stacked decorators, nested and composed wrappers), positions the tree does not attach as the section claims, and silent versus reported drops.

Covered and clean: stacked Python decorators in any order and formatter-wrapped decorators (AC-19); decorator-shaped text in comments, docstrings, strings and template literals (AC-20, AC-22); nested-def and class/method ownership (AC-20, AC-22); `@` before or after `export` (AC-21); multi-declarator and `export const` wrappers (AC-23); bare-statement, call-argument and default-export wrappers (AC-24); partial matches beside full matches and unrelated decorators (AC-25); `..` ranges, per-file parse failure and determinism (AC-27). The binder reads exact node text with no second lexer, consistent with FR-062-CON-2 (no engine pattern) and with the Rust path's CON-4.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Wrapper composition is unstated. Under AC-24 "an argument of another call" is unattached, so `implementing("A")(implementing("B")(fn))` binds A and warns for B, and `memoize(implementing("A")(fn))` binds nothing and warns, while `implementing("A")(memoize(fn))` is outside AC-23's arrow/function-expression list and its outcome is undefined. Stacked decorators are explicitly allowed (AC-19), so a reader expects nested wrappers to stack the same way. Fix: state the outcome for nested wrappers and for a wrapper over or under another higher-order call, with a TC-2205/TC-2206 case for each. | spec/functional/FR-062-implements-relation.md (AC-23, AC-24) |
| FND-002 | low | Import spelling is not addressed. The binder does no import resolution, so `from ix_trace_py import implements as impl` then `@impl(...)` binds only if the module pattern matches `@impl`, `@ix_trace_py.implements(...)` may raise `partial-implements-syntax-match` or nothing depending on where the pattern anchors, and any other library's `@implements(...)` binds. The behaviour follows from "the pattern is applied to that text and to nothing else", but CON-5 forbids only a second lexer, not import resolution or evaluation. Fix: add "and without import resolution or code execution" to CON-5 and one aliased and one qualified case to TC-2201/TC-2207. | spec/functional/FR-062-implements-relation.md (CON-5) |
| FND-003 | low | "The syntax tree attaches a decorator to the one declaration it decorates" and "the tree attaches every decorator to a declaration" are not true of tree-sitter-typescript 0.23.2: a method's decorators are `class_body` children preceding the `method_definition`, and decorators on a class expression, field or parameter attach to nodes that mint no symbol. Those drop silently, while an unattached wrapper warns. Fix: state that method decorators associate with the next member by sibling order, and state that a decorator on a non-minted node binds nothing with no finding (or give it the unattached finding). | spec/functional/FR-062-implements-relation.md (Attachment; A wrapper that attaches to nothing) |

## Verdict

Changes requested: FND-001 (medium) should be fixed this round; FND-002 and FND-003 (low) are wording fixes.

## Dispositions

Checked at c7eadc1 (via 592e7ad, d8a3c7a). Line numbers refer to that head.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The section (L207-227), AC-23 (L288) and AC-24 (L289) state composition: nested chains bind both links to one symbol; `memoize(implementing(A)(fn))` and `implementing(A)(memoize(fn))` are unattached with a warning. TC-2205, TC-2206 and TC-2210 cover these cases. |
| FND-002 | fixed | CON-5 (L387) now forbids import resolution and code execution. The Spellings paragraph (L178-183) requires start-anchored patterns. TC-2201 tests `@t.implements("X")` and TC-2207 tests `t.implementing("X")`, each binding nothing and yielding no finding. |
| FND-003 | fixed | Attachment (L185-195) states the sibling-run association of TypeScript method decorators. L196-206 states that a decorator on a node that mints no symbol binds nothing and yields no finding. Two cases in that list are wrong; see SR-16496 FND-007 and FND-008. |
