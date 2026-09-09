---
id: SR-092
title: "Independent code review of the issue 412 oracle scan remediation"
type: SpecReview
analysis: code-review
scope: "PR #415; src/skeptic.rs; NFR-023; TC-1810..TC-1812; tests/fixtures/corpus_cases/issue_412_oracle_selection.json"
review_set: subset
---

## Summary

The optimisation itself is correct: the indexed binding-to-assertion join
reproduces the former first-binding/first-assertion selection exactly, TC-1812
proves it against a frozen copy of the previous algorithm, and every gate this
PR names reproduces green here. The complexity evidence does not hold. TC-1810
was executed against a deliberately re-nested (quadratic) implementation and
**passed**, and TC-1811's two anti-nesting assertions passed with it, so
NFR-023-AC-1 is currently backed by counters that cannot move.

## Verdict

**FAIL** — the requirement this PR exists to satisfy is guarded by a test that
does not fail when the requirement is violated; measured, not inferred.

## Gates run

Run in the branch worktree at `3ce2849` on the toolchain `rust-toolchain.toml`
actually selects (1.94.1), with an isolated `CARGO_TARGET_DIR`:

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | pass (nightly-only `imports_granularity`/`group_imports` warnings, non-blocking) |
| `cargo clippy --locked --all-targets -- -D warnings` | pass |
| `cargo test --locked` | pass — every library, integration and doc suite |

## Mutation experiment

`oracle_candidate_observed` was rewritten to restore the pre-#412 nested scan —
`assertion_re` bound to a local, then `.captures_iter(span).find(...)` inside
the binding loop — leaving every `observe(...)` call exactly where this PR puts
it. Result on that quadratic implementation:

| Test | Result against the re-nested implementation |
| --- | --- |
| `tc1810_oracle_candidate_scan_counts_are_bounded_by_matches_not_nesting` | **ok** |
| `tc1812_oracle_candidates_match_the_frozen_reference_across_edge_cases` | **ok** |
| `tc1811_oracle_scan_source_shape_prevents_rescans_and_unbounded_join_state` | FAILED at `assert!(direct_scan.contains("assertions.get(binding)"))` |

The file was restored to `3ce2849` afterwards; the worktree is clean.

Two things follow. TC-1810 does not discriminate. And TC-1811 caught the mutant
only through a *positive* assertion that a particular literal still appears —
its two negative assertions, the ones written to reject nesting, both passed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | TC-1810 passes on the quadratic implementation it governs, so NFR-023-AC-1 has no gate | src/skeptic.rs:1346, src/skeptic.rs:675 | correct-requirement-no-evidence |
| FND-002 | high | TC-1811's two anti-nesting assertions are evaded by binding the regex to a local first | src/skeptic.rs:1432, src/skeptic.rs:1434 | correct-requirement-no-evidence |
| FND-003 | medium | Every inspected span now pays a line index, an allocation and a full assertion pass it did not pay before, and the no-candidate path is unmeasured | src/skeptic.rs:571 | missing-requirement |
| FND-004 | medium | The PR and SR-091 report gates on Rust 1.98.1; the repository selects 1.94.1 and declares 1.75, and this PR changes neither | rust-toolchain.toml:2, Cargo.toml:5 | correct-requirement-no-evidence |
| FND-005 | medium | SR-091 records a mutation-sensitivity result for TC-1810 that does not reproduce | spec/reviews/SR-091-issue-412-rust-review.md | correct-requirement-no-evidence |
| FND-006 | low | TC-1811 pins an exact occurrence count of a regex literal in its own source file | src/skeptic.rs:1437, src/skeptic.rs:1403 | correct-requirement-no-evidence |

## Finding detail

### FND-001 — the operation-count harness counts its own instrumentation

`observe(OracleScanEvent::AssertionPass)` is emitted once, unconditionally, at
`src/skeptic.rs:675`, before any traversal happens; `LineIndex` and
`BindingPass` are the same shape. `Join` fires once per capture in whichever
loops happen to carry it. So `assertion_passes == 1`, `binding_passes == 1`,
`line_indexes == 1` and `joins <= matches` are properties of where the four
`observe` calls sit in the source, not of how much scanning occurs — which is
why the re-nested implementation above satisfies all four.

NFR-023's Verification section asks for something stronger: "a deterministic
operation-count harness varies span length and the numbers of bindings and
assertions independently, then proves that capture traversals and line-index
construction do not grow with the number of candidates." TC-1810 varies the
inputs correctly (0, 1, and 129 bindings) but measures the wrong quantity.

The counter has to sit where the work is. One `captures_iter` wrapper that both
scans go through, incrementing a traversal counter per call, would make
`assertion_passes` track actual passes and would have failed the mutant.

### FND-002 — the static audit's negative assertions do not bind

```rust
assert_eq!(direct_scan.matches(".captures_iter(span)").count(), 2);
assert!(
    !direct_scan[binding_loop..].contains("assertion_re.captures_iter(span)"),
    "assertion traversal must not be nested below the binding loop"
);
```

Both passed against a mutant that calls `rescan.captures_iter(span)` inside the
binding loop after `let rescan = assertion_re;`. The count assertion is
satisfied because the mutant also has exactly two `.captures_iter(span)`
occurrences; the anti-nesting assertion is satisfied because it matches on the
variable name rather than on the call. NFR-023-AC-2/AC-4 are verified by
`Analysis`, so a source audit is the right instrument here — but it has to
recognise the construct, not one spelling of it.

### FND-003 — the common path got slower in a performance change

`span_oracle_candidates_observed` builds `crate::parser::line_offsets(span)` — a
full byte scan plus a `Vec<usize>` sized to the span's line count — and then
`oracle_candidate_observed` runs a complete `assertion_re` pass, both
unconditionally, for every evidence symbol. The Rust helper scan now also runs
even when the direct path already produced a suspicion, where before it was
reached only if the direct path produced nothing.

For a span with no `expected`/`oracle` binding — the overwhelming majority of
test spans in a corpus run — the previous code did one binding-regex pass, no
allocation and no newline counting. It now does three full-span traversals and
one allocation. NFR-023's own Rationale scopes the cost to "every test span in a
corpus run", but its measurement table counts traversals only "per direct
candidate extraction", so this path is outside the requirement it belongs to,
and no corpus timing is reported either way. Deferring the line index and the
assertion pass until a binding actually matches restores the old common-path
cost without giving up the join.

### FND-004 — the reported toolchain is not the repository's

`rust-toolchain.toml:2` is `channel = "1.94.1"` and `Cargo.toml:5` is
`rust-version = "1.75"`. This PR changes neither. The gates above reproduce
green on 1.94.1, so nothing is broken — but "Rust 1.98.1 formatting check and
clippy with warnings denied" describes an override, not what CI or a consumer
runs. The correction is #417/NFR-022, and stating the toolchain that was
actually selected costs nothing here.

### FND-005 — a recorded mutation result that does not reproduce

SR-091 lists "Changing pass counts or adding candidate-proportional traversals
fails TC-1810" under Mutation sensitivity. The experiment above adds a
candidate-proportional traversal and TC-1810 passes. The neighbouring bullet
("moving `assertion_re.captures_iter(span)` beneath the binding loop fails
TC-1811") is the one that holds, and only for that exact spelling.

### FND-006 — a test pinned to its own source text

`assert_eq!(production.matches(r"(expected|oracle)").count(), 6)` fails if a
fourth language pattern is added, or if the alternation is named in a doc
comment. `include_str!("skeptic.rs")` ties the test to the file's name and to
the exact string `#[cfg(test)]\nmod tests`. Issue #413 rewrites
`strip_source_comments` in this same file. This is brittleness rather than a
defect — TC-1811 fails loudly rather than silently — but it will be paid for by
whoever lands #413.

## What is correct

Recorded so the next reader does not re-derive it:

- The indexed join preserves selection exactly. The `BTreeMap` keeps the first
  assertion per binding name in span order, which is what the old inner `find`
  selected; `captures_iter` order plus `or_insert_with` gives the same answer
  for repeated bindings, unmatched bindings and assertions preceding bindings.
- `line_offset_at` is correct. `crate::parser::line_offsets` (`src/parser/slice.rs:25`)
  always pushes a leading `0`, so `partition_point(|s| s <= byte) - 1` returns
  the same count as `span[..byte].matches('\n').count()`, including for CRLF and
  multi-byte input.
- TC-1812 is a real differential. It compares against a frozen copy of the
  previous algorithm across three languages, both newline forms, three ordering
  scenarios and the helper path, and it fails when selection changes.
- The `observe` seam is clean: a generic no-op closure on the production path,
  no `cfg(test)` behaviour branch, no bypass feature, no clock assertion.
