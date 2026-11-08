---
id: SR-16321
title: "code-review and rust-review of trace targets from list items and headings (quire-rs PR 603)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@ab57de1; src/traceability.rs; src/corpus/declared_tables.rs; src/corpus/trace_refs.rs; src/coverage/reconcile.rs; src/coverage/row_binding.rs; src/evidence.rs; src/obligation.rs; src/query.rs; src/registry.rs; tests/it/trace_target_sources.rs; tests/fixtures/corpus_cases/trace-target-sources.json; spec/tests.md; reviews/plat-316-spec-review-*.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-050
    type: references
---

# Code review of trace targets from list items and headings (quire-rs PR 603)

## Summary

Ticket: PLAT-316. One code-review pass, with the rust-review lane, over
implementation commit `ab57de1` in `git diff origin/main...HEAD`. Spec commit
`9c55759` already passed spec review and was not re-reviewed. The reviewer ran
these at the head: `cargo test --test it trace_target_sources` (3 passed);
focused lib tests (traceability, declared_tables, trace_refs, reconcile,
evidence, row_binding, registry, query: 114 passed); focused integration tests
(trace, coverage, declared, evidence, obligation, corpus: 186 passed); and
`cargo clippy --all-targets --all-features -D warnings` (clean). A throwaway
probe test, not part of the PR and since removed, drove `coverage::compute`
over nested-fence and nested-heading documents.

Examined, clean:

- `TraceTargetWire` keeps `deny_unknown_fields`. `TraceTarget` goes through it
  with `try_from`/`into`. Omitted `from` serialises away, and the new unit test
  round-trips all three sources.
- `declared_tables::mint` is the only path that mints trace-target ids.
  `coverage/reconcile.rs`, `corpus/trace_refs.rs` and `evidence.rs` all call it.
  The remaining `declared_tables::scan` callers read document-reference rows or
  obligation statement rows, not target ids.
- Load-time rejections are in place for: an invalid regex, a pattern with no
  group 1, `id_column` on a non-table source, `pattern` on a table source, a
  missing selector, and an obligation bound to a non-table target. Each one
  has a fixture case.
- Table targets keep their output. Omitted and explicit `from: table_row`
  serialise to byte-identical reports, and the existing table tests pass.
- Section scope: list items are read from the matched section's own content,
  the same as `table_in` reads tables. Headings are read from nested sections.
  Both follow the accepted decisions.
- The change adds no hash, pin or other ceremony. No downstream crate in
  `~/dev` reads the removed `TraceTarget.id_column` field.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [code-bug] `from: heading` mints the same id twice when a heading nested under a matched section also matches the declared section name. `scan_text` calls `collect_nested` for every matched section, and `query::sections` returns nested matches too, so the inner subtree is walked twice. Reproduced: `## Examples` / `### Examples` / `#### [US-1-EX-1]` gives two `minted_targets` records for `US-1-EX-1` at line 10. Table and list-item sources read only a section's own content, so they cannot double-count this way. Fix: skip a matched section that is a descendant of another matched section, or dedupe by (path, line). Add a test. | src/corpus/declared_tables.rs:705-720 | implementation-bug-despite-evidence |
| FND-002 | medium | [code-bug] `list_item_lines` closes a fence on any line that starts with the same fence character run, whatever its length. A shorter inner fence therefore closes a longer outer one. Inside a fenced Markdown example (```` ````md ```` … ```` ``` ```` …), the list items that follow are minted as live ids. Reproduced: `- IT-1-SC-3` inside a four-backtick fence minted at line 12. CommonMark needs a closing fence of the same character and at least the opening length, with no info string. Fix: record the opening run's length and close only on a run at least that long with nothing after it. Add a test. | src/query.rs:404-423 | implementation-bug-despite-evidence |
| FND-003 | low | [other] No CHANGELOG entry. The public Rust API breaks: `TraceTarget.id_column` is removed and replaced by `source: TraceTargetSource` plus `id_column()`. The module manifest gains `from` and `pattern` on `trace_targets`. The repo records such changes under `[Unreleased]`. Fix: add an Added/Changed entry. | CHANGELOG.md:8-20 | correct-requirement-no-evidence |
| FND-004 | low | [other] Report only, per the brief; the reviewers' files are the record and are not to be edited. `reviews/plat-316-spec-review-base.md` and `reviews/plat-316-spec-review-scope-boundary.md` both declare `id: SR-316`. All three spec review files still cite `FR-050-AC-58` / `TC-1991`, but the spec landed `FR-050-AC-60` / `TC-2080`. AC-60 is consistent with #596 (AC-59) and #598 (AC-58). | reviews/plat-316-spec-review-base.md:2; reviews/plat-316-spec-review-scope-boundary.md:2 | wrong-requirement |

## Verdict

Mergeable after FND-001 and FND-002 are fixed. Both are edge cases on the new
sources that mint wrong ids, and both reproduced. The wire contract, the single
minting path, the load-time validation and table-target behaviour are sound.
The Rust is idiomatic and clippy is clean.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-005 | low | [coverage] The regression case added for FND-002 passes with the old fence logic, so it does not guard the fix. In the fixture, `- IT-002-SC-97` comes after the inner fence pair (```` ````markdown ```` / ```` ``` ```` / `inner` / ```` ``` ````), so under the old rule the inner opener closes the outer fence and the inner closer reopens one, and SC-97 stays fenced either way. Confirmed: with `src/query.rs` and `src/corpus/declared_tables.rs` reverted to ab57de1, the only fixture mismatch is the duplicate `US-033-EX-3`; SC-97 is not minted. The fix itself is correct (probe: an item between an inner opener and closer inside a longer fence is not minted at d7534d5). Fix: put the list item between the inner fence lines, e.g. ```` ````markdown ```` / ```` ``` ```` / `- IT-002-SC-97` / ```` ``` ```` / ```` ```` ````. | tests/fixtures/corpus_cases/trace-target-sources.json:8 | correct-requirement-no-evidence |

## Dispositions

Round 1, at head d7534d5, checking fix commit d7534d5 against ab57de1. The merge commit eab5dc8 was checked only for conflict markers: none remain, and FR-050 AC-58 and AC-59 are unchanged. Tests run: `trace_target_sources` (3 passed), focused lib tests (69 passed), and `clippy --all-targets --all-features -D warnings` (clean). A throwaway probe, since removed, re-ran the round-0 repro inputs plus tilde, info-string and trailing-text fence cases.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `scan_text` keeps a per-document `minted_headings: BTreeSet` keyed on `child.start_line` and skips a heading it has already read. Probe: `## Examples / ### Examples / #### [US-1-EX-1]` mints once. The fixture case `US-033-EX-3` fails on the ab57de1 source with a duplicate record, so the test guards the fix. |
| FND-002 | fixed | `list_item_lines` tracks the opening fence as `(char, run)` and closes only on the same character, a run at least as long, and nothing after it (`fence_run`). Probe: items inside a four-backtick fence, a `~~~`/`~~~~` fence, and a ```` ``` trailing ```` non-closer stay fenced, and items after a valid close mint. The fixture's regression case does not discriminate (FND-005). |
| FND-003 | fixed | CHANGELOG `[Unreleased]` adds an Added entry for `from`/`pattern` and a "Breaking (Rust API)" Changed entry for `TraceTarget.source` and `id_column()`. |
| FND-004 | accepted-no-change | The brief made this report only, and the reviewers' files are the record. The PR body now notes that the landed ids are AC-60/TC-2080 and that SR-316 is shared. |
