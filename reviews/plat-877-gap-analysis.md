---
id: SR-16327
title: "Gap analysis — PLAT-877 report cfg-gated duplicate symbol ids (quire-rs PR 607)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (AC-31), spec/tests.md (TC-2082 rows), src/symbols/mod.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
---

## Summary

Ticket: PLAT-877. PR head: `1e404e9`. Plan completion: not assessed (planless). This analysis checks the FR-051-AC-31 clauses against TC-2082's three tagged adapter tests and the implementation.

## Verdict

**PASS with one low finding.** All three tests carry `#[trace("TC-2082", "FR-051-AC-31")]`, are feature-gated to their own adapter, and pass. The tests cover these clauses:

- **One diagnostic per colliding id.** Covered by `diagnostics.len() == 1` in all three tests.
- **Reason `symbol-id-collision`.** Covered by the Rust test.
- **Qualified name.** Covered by the Rust test (`` `rec` ``).
- **Every declaration by `path:line` with its outer attributes.** Covered by Rust (both cfg texts), Python (`a.py:2 and a.py:5`) and TypeScript (`a.test.ts:1 and a.test.ts:2`).
- **Ids unchanged.** Covered by the Rust test, which asserts the two ids are equal.
- **No diagnostic without a collision.** Rust and Python each have a separate clean control file. TypeScript has an inline distinct title (`'other'`) under the `len == 1` guard.

The implementation is a single shared function, so the language-agnostic clauses only need one adapter to bind them. TC-2082 is in the Test Matrix and in the AC audit mapping. No stub was found, no coverage is inflated, and there is no untraced production code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The AC-31 clause "naming the kind" is asserted by no test. None of the three tests checks for `function` (or any kind) in the reason, so dropping `g[0].kind.as_str()` from the message would still pass TC-2082. | src/symbols/mod.rs:788-806 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-051-AC-31 | examined | All clauses bound except the kind clause (FND-001) |
| TC-2082 (spec/tests.md row and audit row) | examined | Present and mapped |
| tc2082_rust_cfg_gated_duplicates_are_reported | examined | Passes; has a control |
| tc2082_python_redefinition_is_reported | examined | Passes; has a control |
| tc2082_typescript_duplicate_title_is_reported | examined | Passes; has an inline control |
| id_collision_diagnostics | examined | Traced to AC-31 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 6b9c5af: tc2082_rust asserts `d.reason.contains("function \`rec\`")`. It also pins `severity == Warning`, checks that `require_available()` is ok, and checks the warning severity on the symbol-table report. |
