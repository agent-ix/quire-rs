---
id: SR-16324
title: "Gap analysis — PLAT-876 rename quire-rust-extraction to quire-code-extraction (quire-rs PR 605)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (FR-051-CON-1), crates/quire-code-extraction/tests/dependency_boundary.rs, scripts/audits/check_tree_sitter_boundary.sh, src/symbols/*.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
---

## Summary

Ticket: PLAT-876. PR head: `9d8e4d5`. Plan completion: not assessed. The change adds or alters no requirement. The only obligation it touches is FR-051-CON-1, the single tree-sitter boundary. Its two enforcers are the `dependency_boundary` test and `check_tree_sitter_boundary.sh`, and both must follow the rename.

## Verdict

**PASS, clean.** Both enforcers name the new crate. Each passes at head and fails on a real violation: an injected direct `tree-sitter` dependency in the root manifest, or the exempt crate moved back to its old path. No production code is left without an owning requirement, and the diff adds no stubs and no new untested branches.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-051-CON-1 | examined | Enforced by both gates under the new name |
| crates/quire-code-extraction/tests/dependency_boundary.rs | examined | Passes; fails on injected dependency |
| scripts/audits/check_tree_sitter_boundary.sh | examined | Passes; fails on injected dependency and on the old path |
| src/symbols/*.rs | context_only | Import paths only |
