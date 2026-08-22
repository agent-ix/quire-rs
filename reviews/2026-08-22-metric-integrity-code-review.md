---
id: SR-053
title: "Code review — the metric-integrity and skeptic work (quoin#197)"
type: SpecReview
analysis: code-review
scope: "src/metric.rs, src/skeptic.rs, src/coverage.rs, src/symbols/trace.rs, src/grammar/, src/extract/assert_eval.rs, scripts/bench.py, scripts/overfit_check.py, tests/"
review_set: subset
---

# SR-053: Code review — the metric-integrity and skeptic work (quoin#197)

## Summary

Reviewed the eight PRs this crate landed for `agent-ix/quoin#197` (#238, #239,
#240, #242, #243, #245, #246, #247, #248, #249) under the `rust-review` lane.
Every gate is green — `fmt --check`, `clippy -D warnings` across all targets and
features, 867 tests, 41 python tests, `cargo deny` — and two real defects
survive them, both in `src/skeptic.rs` and `src/coverage.rs` where the new
skeptic layer and its diagnostics live.

The same pass covered `quire-cli`, `spec-artifacts-process` and `ix-cli-core`
(gates green: 164, 88+1 xfail, 245+1 skip respectively) and found nothing in
them; their findings, had there been any, would live in their own repositories.

## Verdict

**FAIL** — FND-001 is a `high`: the vacuity detector has a blind spot that
silently admits the exact shape it exists to catch, and its doc comment claims
the opposite.

## Findings

| ID      | Severity | Summary                                                              | Refs                       |
| ------- | -------- | -------------------------------------------------------------------- | -------------------------- |
| FND-001 | high     | Single-line `if let … { assert!(…) }` evades the vacuity check, and the doc comment says it does not | src/skeptic.rs:150         |
| FND-002 | medium   | The `hollow-denominator` message is grammatically broken — template and substitution do not compose | src/coverage.rs:676        |
| FND-003 | low      | `tc1001` asserts an absent key by substring, the pattern this same programme tightened away in `tc788`/`tc805` | tests/coverage_rollup.rs:2034 |

## Detail

### FND-001 — the vacuity detector's blind spot (high)

`assertion_positions` checks each line for an assertion **before** it checks
whether that line opens a guard, and it only pushes a guard when
`opens > closes`. A guard that opens and closes on one line therefore pushes
nothing, and the assertion inside it is counted as **unguarded**.

Reproduced against the shipped code:

```rust
if let Some(v) = parse() { assert_eq!(v, 1); }
```

→ **0 suspicions**. The multi-line spelling of the identical construct is
correctly reported.

The doc comment on `assertion_positions` states the opposite:

> *"A guard that opens and closes on one line contributes nothing, which is
> correct — `if let Some(x) = y { assert!(x) }` on one line is still guarded,
> and is caught by the depth check below."*

It is not caught. Per `rust-review` §0b a doc comment describing an intention
the code does not implement is a finding in its own right; here it also
actively misleads the next reader into believing the case is covered.

**Failure scenario.** A `proptest!` block written as
`if let Ok(v) = parse(x) { prop_assert_eq!(v, y); }` on one line passes the
vacuity check silently — the TC-1596 class (green while checking 2.3% of its
samples) that `FR-064-AC-1` exists to catch. The corpus that motivated this
work writes single-line guards.

**Fix.** Evaluate the guard-open before the assertion on the same line, and
treat `opens > 0 && closes >= opens` on a guard line as a guard covering that
line only. Add the single-line case to TC-997, which currently exercises only
the multi-line spelling.

### FND-002 — the hollow-denominator message does not compose (medium)

`hollow_denominators` renders:

```
`coverage.backed` reports a ratio over a population of 2 but read none of the
1 input(s) it walked, so the number is arithmetic over nothing; …
```

The template says *"a population of {}"* while the substituted expression
carries its own clause (*"{population} but read none of the {examined} …"*).
The template was rewritten mid-change and the substitution was not.

**Failure scenario.** This is the diagnostic a reader acts on when a coverage
figure is arithmetic over nothing — the single most important message this
programme added. A garbled sentence is read as tool noise and scrolled past,
which is the failure `FR-050-AC-27` and `FR-063-AC-5` were written to end.
`(s)` on a computed count is a second, smaller instance of the same
carelessness.

**Fix.** Collapse to one template: `` `{name}` published a ratio over
{population} {unit}s but read none of the {examined} it walked; …``.

### FND-003 — substring absence assertion (low)

`tc1001` ends with `assert!(!clean.to_json().contains("suspicions"))`.

This is the exact pattern this programme argued against and rewrote in
`tc788_no_criteria_corpus_is_unchanged` and
`tc805_undeclared_vocabulary_changes_nothing`, where a substring test began
answering a different question from the one its AC asks once the metric
envelope started *naming* the keys.

It errs safe — it can false-fail, never false-pass — so the severity is low.
But applying a standard to two existing tests and not to the new one written in
the same change is the drift the standard exists to prevent.

**Fix.** Parse and assert `!object.contains_key("suspicions")`, as the two
rewritten tests now do.

## Checks that passed

- **§3/§4 completeness** — no `todo!`, `unimplemented!`, `TODO`, `dbg!` or
  stray `eprintln!` in the changed source; no `#[ignore]` without a lane; the
  temporary probe harnesses used during development were removed.
- **§5 integrity** — no new `#[allow]`, no weakening of `-D warnings`,
  `deny.toml` or the coverage baseline. The `coverage_baseline` regeneration
  was a reviewed diff each time.
- **§6 panic surface** — the only `expect()` calls in `metric.rs` are inside
  `#[cfg(test)] mod tests`; `"panic!"` in `skeptic.rs` is a string literal in
  the closed assertion vocabulary, not a panic.
- **§7 conversions** — `usize as u64` in `Metric::measured` is lossless on
  every supported target; the `as f64` casts are over token and symbol counts
  far below 2^53.
- **§10 wire contracts** — the new payload types (`Metric`, `Measurement`,
  `Suspicion`, `BindingCensus`, `GroundingCounts`) omit
  `deny_unknown_fields`, matching every sibling **output** type on
  `CoverageReport`; the crate applies that attribute to **input** types such as
  `TraceabilityModel`. Consistent with the repo idiom, which §0 ranks above the
  generic rule.
- **§12 gates** — run, not assumed. `cargo fmt --check` clean;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings` zero
  errors; `cargo test` 867 passed / 0 failed; `pytest scripts/tests` 41 passed;
  `cargo deny check` advisories/bans/licenses/sources all ok.
- **`make bench`** green against the checked-in ratchet.

## Note on the reviewer

Every finding here is against code written in the same session that is now
reviewing it, and FND-001 and FND-003 are both cases of a standard this
programme argued for at length and then did not apply to itself.
