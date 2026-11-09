---
id: SR-16499
title: "scope-boundary review of FR-062 Python and TypeScript structural implements syntax positions (quire-rs PR 628)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-rs@8db9e04; spec/functional/FR-062-implements-relation.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-062
    type: references
---

# Scope-boundary review of FR-062 Python and TypeScript syntax positions (quire-rs PR 628)

## Summary

Ticket: PLAT-866. Checked the split between the engine (this FR: static parse contract), the module (`spec-artifacts-process` FR-004: declares the forms and drops the textual ones) and the runtime libraries (`ix-trace-py` FR-002, `ix-trace-ts` FR-002: what the decorator and wrapper do at run time). The allocation is clean: the engine supplies no pattern (CON-2), the module owns the forms, and the runtime repositories own run-time behaviour, with no code executed by the binder. The FR-051 advisory boundary is recorded in SR-16496 FND-002.

The replacement position (no compatibility period; FR-051 advisory names the new construct) is the no-compat choice and is stated consistently. A consumer retagged late loses `implements` scope relations for its comment-tagged symbols and gets one advisory per comment naming what to write; `implements` is scope, so no coverage number moves. That is an acceptable end state and not a hole.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | "The order of work is a precondition and not a follow-up: every production symbol that binds only through a textual `Implements:` comment is retagged first, then the module's textual forms are removed" is a cross-repository work order with no AC and no mechanism. While both forms are declared, a Python/TypeScript comment still bound by a textual form yields no finding (FR-051-AC-49 defers an implements-read id to FR-050-AC-62, which is Rust-only), so nothing finds the symbols to retag. After removal the advisory already names each one. Fix: delete the retag-first sentence and keep the one real constraint (the engine must accept the new `syntax` values before a module declaring them is released, because unknown values are refused at load). | spec/functional/FR-062-implements-relation.md (The comment forms are replaced, not kept beside) |

## Verdict

Approve with one low finding.

## Dispositions

Checked at c7eadc1 (via 592e7ad, d8a3c7a). Line numbers refer to that head.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | L242-251: the structural forms are declared and the textual forms removed in one change that ships with the engine release accepting the syntax values. There is no coexistence window and "no precondition measurement gates it". This is identical in substance to spec-artifacts-process#133 at fac4e2e (FR-004, Replacement of the comment forms). |
