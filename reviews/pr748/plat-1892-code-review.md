---
id: SR-967175026302197768900474181941935084878
title: "PLAT-1892 Rust code review (quire-rs PR 748)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@a03ab3608cf97cf939e65dad57078d3e7eb12209; git diff origin/main...HEAD: Cargo.toml, src/impact.rs, src/loader/mod.rs; FR-099-AC-9"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-099
    type: references
---

## Summary

Reviewed the frozen PR #748 head and its Rust implementation, tests, changed dependency declaration/import cleanup, repository guidance, and changed-file call sites. FR-099-AC-9 links on the changed implementation and all nine new tests are present. `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, and the focused 25-test `impact::tests::` suite pass. Build output warns that the optional `@agent-ix/spec-objects-architecture@0.10.3` fixture could not be fetched; its two dependent tests skip. No dependency version move or CI workflow change is present. One behavior defect remains in static call resolution.

## Verdict

**CHANGES REQUIRED** — FND-001 can cause a local closure call to be treated as a call to a different extracted function, producing an incorrect caller edge and test selection.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `resolve_callee` treats a bare identifier as the extracted function with the same qualified path without checking lexical bindings. If `fn changed() {}` exists and another function contains a local binding named `changed` shadows the function before a call to that name, `call_path` returns `changed` and resolution matches the unrelated function symbol. The call is to a local closure and must remain unresolved; instead the impact report can add unrelated linked callers/tests and claim the traversal is complete. Track local bindings/shadowing conservatively or classify shadowed names as unresolved, and add an AC-9 regression test. | src/impact.rs:1410-1448 |

## Review Notes

- Changed behavior is owned by FR-099-AC-9. The `implements` markers name that criterion, and quoin's computed matrix binds all nine new tests to it.
- The changed code uses tree-sitter parsing, bounded depth, a visited set, and deterministic output ordering. The focused tests exercise direct and multi-step callers, nearest linked callers, depth truncation, cycles, traits, dynamic methods, imported aliases, and duplicate extracted paths.
- `Cargo.toml` retains the existing exact `ix-trace-rs` v0.1.2 pin; the duplicate declaration and import cleanup are minimal compile fixes.
- No applicable AssuranceProfile was found for this impact-selection behavior. No workflow file changed.
