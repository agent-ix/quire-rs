---
id: SR-20261112155215
title: "code-review and rust-review of located engine findings (quire-rs PR 655)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@1f5017d5; src/extract/frontmatter_lines.rs; src/extract/locator.rs; src/extract/mod.rs; src/record_check.rs; src/loader/record_validation.rs; src/loader/*.rs; src/corpus/trace_refs.rs; src/corpus/validate.rs; src/query.rs; src/semantic/resolver.rs; src/validate_document.rs; tests/it/located_findings.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
---

# Code review of located engine findings (quire-rs PR 655)

## Summary

Ticket: PLAT-1478. One code-review pass with the rust-review lane over `git diff origin/main...1f5017d5`. Checks run in a review worktree: `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean, `cargo check --features python` clean, `cargo check --target wasm32-unknown-unknown --no-default-features --features wasm` clean, `cargo test --test it -- located_findings trace_reference bundle_validate` (55 passed), `cargo test --lib` over extract/query/loader/validate_document/corpus/semantic (274 passed). The record is built by the unmodified `extract::extract`; `match_value_lines` only re-evaluates the same locators for lines and applies the same placeholder drop, so record elements and lines stay aligned. No archetype, field or key name appears in `record_check.rs` or `loader/record_validation.rs`. `find_table` is behaviour-identical to the removed `table_from_section`/`parse_table` chain. A scratch probe confirmed `frontmatter_lines::key_line` on CRLF, quoted keys, comment lines, block scalars, lists of maps and nested or multi-line flow mappings. Line is the last sort key in `trace_refs`, and the existing AC-8/AC-17 ordering tests pass unchanged. Sibling PR 654 overlaps only textually (CHANGELOG `[Unreleased]`, `spec/tests.md`, `tests/it/main.rs`).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | `key_line` returns the first occurrence of a duplicated frontmatter key, while the parser keeps the last value. For `k: first` / `k: second` the probe returns value `second` at line 2, the line of `first`. Fix: take the last match at the scope level, or document the limit. | src/extract/frontmatter_lines.rs:83-84 | implementation-bug-despite-evidence |
| FND-002 | low | `Concerns.absent` is redundant. A key that a `required` error names has no `lines` entry, so `(Some(p), false)` already falls back to the absence line. Flipping `absent: true` to `false` passes every test (an equivalent mutant). Fix: drop the field and the match arm. | src/record_check.rs:65-77,103-127 | implementation-bug-despite-evidence |
| FND-003 | low | `value.find('{').filter(\|_\| value.starts_with('{'))` always yields `Some(0)` or `None`. It reads as a search. Fix: `value.starts_with('{').then_some(0)`, or compute the column directly. | src/extract/frontmatter_lines.rs:87 | implementation-bug-despite-evidence |
| FND-004 | low | `match_value_lines` (extract) and `resolve_reference_at` (semantic::resolver) are new `pub` items in public modules, but only in-crate code calls them. This widens the public API without need. Fix: `pub(crate)`. | src/extract/mod.rs:50; src/semantic/resolver.rs:167 | implementation-bug-despite-evidence |
| FND-005 | low | The record check evaluates every `match` locator twice (`extract`, then `match_value_lines`). For each key, `resolve_lines` re-runs `body_line_offset`, which re-parses the frontmatter, and for frontmatter keys it rescans the YAML. Only opted-in archetypes pay this, but it is repeated work. Fix: compute the offset once per document, for example by passing it into `resolve_lines`. | src/extract/locator.rs:516-540; src/record_check.rs:45-51 | implementation-bug-despite-evidence |

## Verdict

No correctness defect in the new code paths. All five findings are low (idiom, API surface, one edge-case line). The blocking issue is a spec gap, recorded in SR-20261112155216 (FR-033-AC-18 assert-line parity).

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (f9de3f2, verified at e662590): `find_key` searches `.rev()`, so a duplicated key takes its last occurrence's line. Unit test `a_duplicated_key_takes_the_line_of_its_last_occurrence` and integration test `tc2300_a_duplicated_frontmatter_key_follows_the_value_the_parser_keeps` check it; the mutant dropping `.rev()` fails. |
| FND-002 | fixed | Round 1: the `Concerns.absent` field and its match arm are removed, and `line` matches on `&at.property` alone. |
| FND-003 | fixed | Round 1: `if value.starts_with('{') { Scope::Flow { col: line.text.len() - value.len(), .. } }`. |
| FND-004 | fixed | Round 1: `match_value_lines` and `resolve_reference_at` are `pub(crate)`. |
| FND-005 | fixed | Round 1: `match_value_lines` computes `body_line_offset` once per document and passes it through `eval_locator_located_at`. The second evaluation of each locator remains because FR-069 requires the record from the unmodified `extract::extract`. A comment in `record_check.rs` states this. Accepted as the spec's cost. |
