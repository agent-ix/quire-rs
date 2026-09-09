---
id: SR-099
title: "Independent re-review of the oracle scan remediation at dd2920b"
type: SpecReview
analysis: code-review
scope: "PR #415 at dd2920b; src/skeptic.rs; NFR-023-AC-1..AC-4; TC-1810..TC-1812"
review_set: subset
---

## Summary

Both SR-092 high findings are closed, and closed by measurement rather than by
assertion. The counters now sit inside `observed_captures_iter`, the one place a
traversal can start, so re-nesting the assertion scan under the binding loop
makes TC-1810 report 129 assertion passes against an expected 1 and fails
TC-1811 on traversal ordering. The eager line index and eager assertion pass are
gone and TC-1810 asserts their absence directly. One residual escape remains: a
traversal reached through a helper defined outside the audited source slice is
invisible to both gates.

## Verdict

**CONDITIONAL** — no high findings. One medium, closable by one line that
mirrors a check the same test already performs for the line index.

## Gates run at `dd2920b`

Branch worktree, submodule initialised, isolated target directory, on the
toolchain `rust-toolchain.toml` selects (1.94.1).

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | pass (nightly-only `imports_granularity`/`group_imports` warnings) |
| `cargo clippy --locked --all-targets -- -D warnings` | pass |
| `cargo test --locked` | pass — **612 library tests** plus every integration and doc suite, 0 failures |

## Mutation experiments

Three mutants, each applied to `dd2920b` and reverted afterwards. The worktree
was clean before and after.

| # | Mutant | TC-1810 | TC-1811 | TC-1812 |
| --- | --- | --- | --- | --- |
| M1 | Assertion scan re-nested inside the binding loop, still through `observed_captures_iter` | **FAIL** | **FAIL** | ok |
| M2 | Same rescan via a raw `pattern.captures_iter(span)` helper placed *inside* the audited slice | ok | **FAIL** | ok |
| M3 | Same rescan via that helper placed *before* `fn oracle_candidate_observed` | ok | **ok** | ok |

M1 is the defect NFR-023-AC-1 exists to prevent, and it now fails loudly:

```
assertion `left == right` failed
  left: 129
 right: 1
```

That is the counter observing 129 real traversals over the 129-binding span. At
`3ce2849` the identical mutant passed TC-1810. The AC has a gate.

M2 is the evasion that defeated the previous TC-1811 (`let rescan = assertion_re;`).
It is now caught by `!direct_scan.contains(".captures_iter(")`.

M3 is FND-001.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | A traversal reached through a helper defined outside the audited slice evades both the counter and the static audit | src/skeptic.rs:1558, src/skeptic.rs:1571 | correct-requirement-no-evidence |
| FND-002 | low | The Rust helper traversal runs on every Rust span, before the direct result is known, and no counter observes it | src/skeptic.rs:679, src/skeptic.rs:701 | missing-requirement |
| FND-003 | low | TC-1811 still reads its own source file and delimits the audited region by function-name text, and #413 rewrites this file | src/skeptic.rs:1533, src/skeptic.rs:1541 | correct-requirement-no-evidence |

## Finding detail

### FND-001 — the audit's blind spot is its own slice boundary

`direct_scan` is the text between `fn oracle_candidate_observed` and
`fn line_offset_at`. Both of TC-1811's traversal checks —
`!direct_scan.contains(".captures_iter(")` and
`direct_scan.matches("observed_captures_iter(").count() == 2` — are evaluated
only inside that window. So is the counter, because `observe` fires only from
inside `observed_captures_iter`.

Adding

```rust
fn assertion_census(pattern: &Regex, span: &str) -> usize {
    pattern.captures_iter(span).count()
}
```

*before* `fn oracle_candidate_observed`, and calling it once per assignment
inside the binding loop, restores `O(a·b)` work and passes TC-1810, TC-1811
**and** TC-1812. The same helper placed one function later — still in
production, but inside the slice — fails TC-1811 (M2). The gate's reach is a
source-position accident, not a property of the code.

The fix is one line, and TC-1811 already uses it for the other traversal it
governs:

```rust
assert_eq!(
    production.matches("crate::parser::line_offsets(span)").count(),
    1,
    "raw line-index construction is confined to its measured wrapper"
);
```

`production` currently contains exactly one `.captures_iter(` — the one inside
`observed_captures_iter` — so

```rust
assert_eq!(production.matches(".captures_iter(").count(), 1);
```

passes today, kills M3, and states the invariant NFR-023's Verification section
actually claims: "a static source audit rejects **every** direct capture
traversal outside that measured entry point".

### FND-002 — one traversal is still outside the measured set

`helper_oracle_candidate_at_byte` runs `rust_helper_assertion_pattern().captures(span)`
for every Rust span, before `direct` is consulted, and emits no
`OracleScanEvent`. It is a single `captures` rather than a `captures_iter`, so
it is bounded and cheap, and NFR-023's measurement table scopes traversal
counting to "per direct candidate extraction" — so this is outside the
requirement rather than in violation of it.

It is recorded because the requirement's own Rationale says the cost is paid
"on every test span in a corpus run", and this is the one span-proportional
traversal that neither the table nor TC-1810 accounts for. Either bring it into
`observed_captures_iter` with its own event, or say in AC-1 that the helper
probe is deliberately excluded and why.

### FND-003 — the audit still reads itself

`include_str!("skeptic.rs")` with `rfind("\nmod tests {")` is an improvement on
the previous `split("#[cfg(test)]\nmod tests")`, and dropping the
`matches(r"(expected|oracle)").count() == 6` assertion removes the most brittle
pin SR-092 recorded. What remains is that the audited regions are located by
searching for `fn span_oracle_candidates_observed`, `fn oracle_candidate_observed`,
`fn observed_captures_iter` and `fn line_offset_at`. Renaming or reordering any
of them silently moves the window rather than failing.

Issue #413 rewrites `strip_source_comments` in this same file and #403 adds a
third scanner state to `src/symbols/typescript.rs`; neither touches these
functions, so nothing is broken today. Adopting FND-001's `production`-scoped
check reduces how much of this test depends on slicing at all.

## What is correct

- **The counter is now where the work is.** `observed_captures_iter` is the only
  path to `captures_iter` in production, and every `OracleScanEvent` is emitted
  at the point of traversal rather than at a fixed position in the source. That
  is precisely the correction SR-092 FND-001 asked for, and M1 proves it took.
- **The eager work is gone and the absence is asserted.** `span_oracle_candidates_observed`
  returns `SpanOracleCandidates::default()` before building any line index when
  neither path selected a candidate, and `oracle_candidate_observed` peeks the
  binding iterator before running the assertion pass. TC-1810 now varies six
  spans — including no-binding, helper-only and direct-and-helper — and asserts
  `line_indexes == 0` and `assertion_passes == 0` where no work is owed. SR-092
  FND-003 is closed.
- **`OracleAssertions` replaces the `BTreeMap`.** The join state is two
  `Option<&str>` fields keyed by a closed `OracleBinding` enum, so AC-4's
  "bounded independently of `a` and `b`" is now structural rather than argued,
  and no output can depend on container iteration order.
- **The record is honest.** SR-091 carries an explicit supersession note naming
  what it got wrong; `spec/tests.md` records "SR-092 findings remediated in
  SR-096/SR-097" rather than the previous "reviewed in SR-088..SR-091"; NFR-023's
  measurement table and AC text were rewritten to describe what is measured
  instead of what was hoped. SR-092 FND-005 is closed.
- **The toolchain claim is corrected.** The PR body now names both Rust 1.98.1
  and the repository-selected 1.94.1, and states that the 1.94.1 pin and the
  1.75 consumer MSRV are defects owned by #417 which this PR neither preserves
  nor justifies. SR-092 FND-004 is closed.
