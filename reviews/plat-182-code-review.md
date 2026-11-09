---
id: SR-16330
title: "Code review (Rust lane) — PLAT-182 linear-time skeptic oracle detection (quire-rs PR 608)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/skeptic.rs (oracle_candidate, helper_oracle_candidate, newline_count_before, tests module)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-064", type: references}
---

## Summary

Ticket: PLAT-182. PR head: `70781bc` (one commit over origin/main, `src/skeptic.rs` +48/-8). Code review with the Rust lane (rust-review, rust-style loaded) over `git diff origin/main...HEAD`.

Gates run by the reviewer at head: `cargo fmt --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --lib skeptic` 12/12 pass.

Equivalence was checked by differential execution, not only by reading: the old loop and the new loop were run side by side inside `oracle_candidate` with an `assert_eq!` on their `Debug` output, over every skeptic unit test, six crafted Rust spans, two TypeScript and two Python spans (expected-before-oracle with only one asserted, assertion before binding, duplicate bindings, multibyte text before the binding, no assertion at all), and the full `cargo test --test it corpus` run (34 tests, which drive `oracle_copies_in` over the corpus). No divergence.

## Verdict

**Changes requested (two medium, one low).** The production change is correct and output-identical. First-asserted-binding order is unchanged, because the cached lookup per name is independent of the binding's position and the loop still returns at the first binding whose name has an assertion. `expected`/`oracle` precedence is still decided by binding order alone. `line_offset` is unchanged, because UTF-8 never carries a `0x0A` byte inside a multibyte sequence, so a byte count equals `matches('\n').count()`. The old `?` on `assertion.get(1)` and on `rsplit().next()` could never fire, since group 1 always participates in a match, so moving them into `and_then` changes nothing. The assertion scan now runs at most twice per span, so the quadratic rescan is gone.

The defects are in the test. Its insertion point takes over another test's trace attribute, and its wall-clock guard does not catch the old quadratic code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new test sits between `#[trace("TC-1000", "FR-064-AC-3")]` with its comment and the `#[test] fn tc1000_similarity_reads_tokens_not_layout`. The attribute now binds to `plat182_oracle_candidate_scales_and_picks_first_asserted_binding`, and `quire trace --id FR-064-AC-3` credits the PLAT-182 test as the sole verifier while the real TC-1000 test is untraced | src/skeptic.rs:1400-1423 |
| FND-002 | medium | The `< 2s` wall-clock guard does not discriminate. With the new test applied to the origin/main implementation, it passes in 1.68s (debug build, loaded machine); the new code takes ~0.05s. The bound sits just above the old cost, so it is machine-dependent both ways: it can miss a regression on fast hardware and fail on a slow CI host | src/skeptic.rs:1408-1414 |
| FND-003 | low | `newline_count_before` replaces `matches('\n').count()` with a byte filter of equal O(prefix) cost. Both `oracle_candidate` and `helper_oracle_candidate` compute at most one count per call, so the newline half was never quadratic. The helper only satisfies the ticket's literal AC and adds a function with no behavioural or performance value | src/skeptic.rs:618-624 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/skeptic.rs oracle_candidate | examined | Output-identical (differential run); at most two assertion scans per span |
| src/skeptic.rs helper_oracle_candidate | examined | Unchanged behaviour |
| src/skeptic.rs newline_count_before | examined | Equivalent to `matches('\n').count()`; FND-003 |
| src/skeptic.rs plat182 test | examined | FND-001, FND-002 |
| src/skeptic.rs oracle_copies_in (callers) | context_only | One call of each candidate function per test span |
| FR-064-AC-3 | context_only | Trace target displaced by FND-001 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The fix commit adds `reviews/plat-182-code-review.md` and `reviews/plat-182-gap-analysis.md` with ids SR-16326 and SR-16327. On origin/main those ids already belong to `reviews/plat-877-*` and `spec/reviews/plat-1170-*`. Free ids are SR-16330 and SR-16331, so this file becomes SR-16330 and the gap-analysis file SR-16331 | reviews/plat-182-code-review.md:2, reviews/plat-182-gap-analysis.md:2 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At 9b3103c the `#[trace("TC-1000", "FR-064-AC-3")]` block sits directly above `tc1000_similarity_reads_tokens_not_layout`; `quire trace --id FR-064-AC-3` lists only that test |
| FND-002 | fixed | The timing bound is gone. `plat182_oracle_candidate_scans_assertions_once_per_binding_name` asserts `scans == 2` through the private `oracle_candidate_counting_scans`. The reviewer disabled the per-name cache and the test failed with left 4001, right 2. The counter is a private `&mut usize` bumped once per scan; it adds no public surface and costs nothing |
| FND-003 | fixed | `newline_count_before` is deleted and `span[..start].matches('\n').count()` is back at both call sites |
