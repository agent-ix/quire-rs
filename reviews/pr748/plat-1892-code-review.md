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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | `shadows_local_binding` recognizes prior `let` statements, function and closure parameters, and `for` patterns, but it does not inspect bindings in `if let` conditions or `match` arms. For example, with a top-level `fn changed() {}`, `if let Some(changed) = value { changed(); }` calls the local binding, not the top-level function. `collect_calls` therefore leaves the bare path eligible for `resolve_callee`, creating a false caller edge and potentially selecting unrelated tests as complete. Treat those pattern bindings as unresolved and add an FR-099-AC-9 regression test. | src/impact.rs:1435-1484 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At head `0a9e114f12022bc83c27ecec1ca3a7f2d08c7652`, `collect_calls` calls `shadows_local_binding` for bare identifiers and emits `local-binding-shadows-symbol` when matched. The new `local_closure_shadowing_is_unresolved_and_does_not_select_unrelated_test` is tagged FR-099-AC-9 and passed in a focused run. |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | `condition_binds_name` scans the entire `if`/`while` condition for a matching let-condition pattern, then marks every same-named call anywhere in that condition as shadowed. In `if let Some(changed) = changed() { changed(); }`, the initializer call occurs before the pattern binding enters scope and resolves to the top-level function, but it is marked unresolved. The engine then misses this caller edge and its tests while reporting incomplete selection. Check binding scope relative to each call (or inspect only calls after the binding), and add an FR-099-AC-9 regression test. | src/impact.rs:1486-1518 |

### Dispositions — round 2

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | At head `966318b69dc5f2c9788469b40925ecddf6a4f2ba`, `shadows_local_binding` inspects `if_expression` and `while_expression` conditions plus `match_arm` patterns. The new `if_let_binding_shadowing_is_unresolved_and_does_not_select_unrelated_test` and `match_arm_binding_shadowing_is_unresolved_and_does_not_select_unrelated_test` tests are each tagged FR-099-AC-9 and passed in focused runs. |

### Dispositions — round 3

Checked at `f9326fb6d47a6459aabda59b01fb9ce47c75ae5c`. FND-003 is fixed: conditional binding detection is limited to the `if` consequence or `while` body, and match-arm binding applies only after the pattern. The `if_let_initializer_resolves_before_binding_then_body_is_unresolved` regression verifies the initializer call still forms the caller edge while the body call remains unresolved. The existing if-let and match-arm shadowing regressions also pass individually. The current matrix reports FR-099-AC-9 as tagged with 13 tests. The PR body still says nine focused tests; it should be updated to describe the current 13. The ticket's required 24-change replay remains unrun, so overall PR readiness is still blocked by that acceptance item even though no code-review finding remains open.

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | The implementation checks whether a call is in the `if` consequence or `while` body before applying condition bindings, and requires a match-arm pattern to precede the call. The new initializer regression confirms the initializer call resolves, the bound body call is unresolved, and the caller's verified test remains selected. |

### Review note update — round 4

At unchanged head `f9326fb6d47a6459aabda59b01fb9ce47c75ae5c`, the PR description now reports 13 FR-099-AC-9 test binders and describes the current dependency state accurately: the v0.1.2 dependency appears once on current main and this PR only removes the duplicate loader import. This closes the stale-description note from round 3. No source review or gate was run for this documentation correction. The required 24-change replay remains outstanding.

### Replay evidence — round 5

The 24-pair caller-following selection-only replay is complete at frozen implementation head `f9326fb6d47a6459aabda59b01fb9ce47c75ae5c`, using `spec-artifacts-process@0.3.0` and caller depth 3. Per-PR results and replay methodology are recorded in [plat-1892-replay-summary.tsv](plat-1892-replay-summary.tsv); full JSON selection reports remain in `/private/tmp/plat1892-selection-output/`. The 24 selections were saved before opening the canonical `broken/<PR>.txt` files; no historical selector results, `why`, or EVIDENCE inputs were used, and no FSL builds or tests were run.

The replay selected 2,080 tests. Twenty-eight selected tests carried caller-following reasons, but all also had direct-change or implemented-ID reasons, leaving zero caller-only additions and zero of 82 broken rows caught by newly added tests. All 24 reports have `selection_complete=false`. There were 6,984 unresolved roots among 7,991 caller roots (87.4% root-level share) and 5,763,910 unresolved-call records. The report API does not expose a total call-site denominator, so no call-site percentage is claimed.

### Rebase scope update — round 5

At current PR head `b823835cbe30e9792945e00585f7e06ab8d99f93`, `src/impact.rs` is byte-for-byte identical to reviewed head `f9326fb6d47a6459aabda59b01fb9ce47c75ae5c`. The current PR diff contains `src/impact.rs` and review evidence (`reviews/pr748/plat-1892-code-review.md`, `reviews/pr748/plat-1892-gap-analysis.md`, and `reviews/pr748/plat-1892-replay-summary.tsv`); it no longer includes the loader import cleanup now on main. The existing impact-engine source review and finding dispositions therefore still apply unchanged. No new source review or gate was run for this rebase note.

### Fix-round update — round 6

Reviewed fix commit `36de6110858bc6c40357af745114b424ef5f1663` at the current PR head. The Rust tree-sitter imports, `ParsedCall`, and parser/call-resolution helpers are gated by `rust-symbols`; the caller-graph builder is gated the same way. Without `rust-symbols`, `analyze` uses `caller_call_graph_unavailable`, which returns an empty graph. The existing extraction path emits `LanguagesUnavailable`, and `selection_complete` is computed false from report findings; the no-Rust path does not infer caller edges or tests. The `disabled_rust_symbols_keeps_caller_selection_incomplete` regression is gated with `not(feature = "rust-symbols")`, tagged FR-099-AC-9, and passed under `--no-default-features --features python-symbols`. The computed matrix reports 14 tagged FR-099-AC-9 test binders. The change adds no new code-review finding. The default-feature impact engine is otherwise unchanged from the previously reviewed implementation. No aggregate gate was run.
