---
id: SR-16438
title: "Gap analysis — PLAT-847 CR-215 marker-only binding against FR-051 (quire-rs PR 627)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@6f257a9; FR-051 AC-1..50, CON-3, CON-4; FR-050-AC-44, FR-050-AC-62; FR-066; FR-068-AC-5; TC-1044, TC-1055..1057, TC-1081, TC-2110, TC-2160..2166; PR body"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-068", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-066", type: references}
---

## Summary

Ticket: PLAT-847. PR head: `6f257a9`. This is a planless gap analysis. Plan completion: not assessed. Each criterion in scope was traced to a test carrying its marker, and the test body was read. The installed `quoin matrix` was not used, because it links engine 0.50.2, which still reads legacy forms, so its matrix would describe the old binding rules rather than this branch.

## Verdict

**FAIL: two medium and one low finding.** Code and spec otherwise agree.

- **Tagged tests found and read.** TC-1044, TC-1046 and TC-1081 (AC-22); TC-1055, TC-1056 and TC-1057 (AC-24); TC-2160 (AC-11); TC-2161 and TC-2162 (AC-49); TC-2163 (AC-50); TC-2164 (FR-050-AC-44); TC-2165 (AC-22, AC-49); TC-2166 (AC-28). Each body asserts the clause it names. Positive and control halves are both present: a marker on a production symbol yields nothing, a body comment and a string yield nothing, and an all-marker tree yields nothing.
- **FR-051-CON-3.** Inspection holds. `VerifiesRelation` carries symbol, id, form, path, line, column, kind and ignored, with no provenance. `SymbolGraph` has no rewrite list. `BindingCensus` has no non-marker field, and `trace_tags` has only `markers` and `implements`.
- **FR-051-CON-4.** Step 1 (the quire-rs retag) is in the PR. Steps 2 and 3 are ordered in the PR body: the spec-artifacts-process module must drop its `trace_tags.legacy` list before or with the engine release, because this engine refuses that key. That is a release-order obligation and does not block merging into `main`. The published delta is incomplete (FND-003).
- **FR-050-AC-62.** Unchanged. `legacy-implements-form` still fires, and `an_id_an_implements_form_read_is_left_to_its_own_warning` shows the advisory defers to it.
- **FR-066.** No code change was needed. Its `marker-form-mismatch` row is a census disposition, not the deleted coverage reason, and `scripts/` and `tools/` hold no reference to the removed names.
- **Shell (point 6).** Conformant. FR-051-AC-49 limits the advisory to Rust, Python and TypeScript and says "a shell comment is reported only under AC-44". `annotation_channel` returns `None` for shell. Consequence of the spec, not a code gap: a bare id in a shell comment that is not marker-shaped is reported by nothing, because AC-44 reports only declared-marker matches.
- **Advisory noise (point 3). Data from the coder's measurement, which this review did not re-measure.** On this tree there are 2045 findings. About 1314 are plausible trace ids: FR 996, TC 220, NFR 75, StR 14, US 6, IT 3. About 730 are not: CR 400, PLAT 197, AC 60, CON 13, TASK 9, TIER 8, PLAN 8, MP 8, EV 8, UTF 6, ADR 3, STD 2, SR 1, AGPL 1 and others. The pattern is `(?i)\b[A-Z]{2,4}-[0-9]+(?:-[A-Z]+-[0-9]+)?\b`. It is case-insensitive, so `utf-8`-style words match too. This is the shared generic pattern that FR-051's Advisory section and AC-49 require ("is not specialised"), so the code is spec-conformant. **Judgement:** acceptable for V1. It is a `warning`, it binds nothing and moves no total, and the spec mandates it. However, about 36% of findings advise writing a marker such as `#[trace("CR-215")]` for an id that no trace target mints, and following that advice would create an untracked binding. Under the repository's own CLAUDE.md rule this is a "bad rule" class, not a "bad corpus" one, and the reason to narrow it does not depend on the count. Whether to restrict the token set to prefixes the loaded module's trace targets mint is **a decision for Peter**. It should be taken before quire-cli (about 196 symbols) and quoin (about 1282) are retagged against it, where the noise scales.
- **Coverage delta (point 5).** The PR body has a "Coverage delta" section: backed 1419/1910 before, 1348/1885 after. See FND-003.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-2110 claims "refuses a `legacy` key as unknown naming it" (the AC-43 behaviour that CON-4's release order rests on), but `tc2110_load_refusals` asserts only a `bogus` key. No test loads a manifest declaring `trace_tags.legacy` and checks that the error names `legacy`. Add that case to `tc2110_load_refusals`. | tests/it/shell_symbols.rs:406-425; spec/tests.md TC-2110 |
| FND-002 | medium | FR-068-AC-5 was amended to say "a test binding names the marker form that bound it", but TC-1095 (`tc1094_tc1097_relation_variants_preserve_graph_meaning`) dropped its `provenance == "canonical"` assertion and asserts nothing about `form` in its place. The amended clause has no test that could fail. Assert `form == "rust-trace"` on the `Verifies` record. | tests/it/assurance_export.rs:394-400; spec/functional/FR-068-source-grounded-assurance-projection.md:132 |
| FND-003 | low | FR-051-CON-4 requires the published delta to state rows backed before and after "for every repository retagged", plus "the symbols that were tagged only through their name, which the change leaves unbacked and unreported". The PR body gives one combined figure for this repository and the corpus submodule. It does not say which part of the 71-row backed drop and the 25-row denominator drop belongs to quire-rs and which to the qa-corpus#49 bump, and it gives no name-only count (0 is a valid answer if the retag covered every `rust-test-name-id` binding). Add both to the PR body. | PR #627 body, "Coverage delta" |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-051-AC-11 | examined | TC-2160 binds and asserts per language; clean |
| FR-051-AC-22 | examined | TC-1044, TC-1046, TC-1081, TC-2165; language defect in SR-16437 FND-001 |
| FR-051-AC-24 | examined | TC-1055, TC-1056, TC-1057; clean |
| FR-051-AC-28 | examined | TC-2166; clean |
| FR-051-AC-43 | examined | FND-001 |
| FR-051-AC-49 | examined | TC-2161, TC-2162, TC-2165; clean |
| FR-051-AC-50 | examined | TC-2163; clean |
| FR-051-CON-3 | examined | Inspection holds |
| FR-051-CON-4 | examined | FND-003 |
| FR-050-AC-44 | examined | TC-2164; clean |
| FR-050-AC-62 | examined | Unchanged; deferral tested |
| FR-066 | context_only | No code dependency on removed names |
| FR-068-AC-5 | examined | FND-002 |
| FR-051-AC-1..10, 12..21, 23, 25..27, 29..42, 44..48 | context_only | Not changed by this PR |

## New findings (disposition pass 1)

Pass 1 reviewed fix head `2648663` against the amended FR-051 Advisory narrative and CR-215 kind-test note, AC-22, AC-49, FR-050-AC-39, FR-077-AC-2 and FR-077-CON-1, and TC-2183, TC-2184 and TC-2185. TC-2183 asserts that the advisory reports exactly `FR-001-AC-1` and `TC-001` from a comment that also names `CR-215`, `PLAT-847` and `AC-3`; that is a real control. TC-2184 covers `unmatched_tags` and the fallback that filters nothing when no id was minted. The coder measured the advisory falling from 2045 to 1345 findings on quire-rs; this review did not re-measure it.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | FR-077-AC-2 as amended says `SymbolGraph.mentions` records only tokens "of a minted kind", as an unconditional property. The code applies the kind test to mentions only when a caller opts in through `SymbolGraph::retain_id_kinds`. `bind()` has no corpus and returns every token, and no production path calls the filter: `trace_search::search` and quire-cli's `quire trace` read `graph.mentions` unfiltered. TC-2184 itself states "after `retain_id_kinds`". **Judgement: the opt-in design is acceptable.** `bind` cannot know the minted ids, and a citation index queried by id (`quire trace --id CR-215`) arguably should not hide a CR id. The AC text, however, claims a property the shipped graph does not have. Fix: reword FR-077-AC-2 so the kind test applies when the consumer supplies the corpus's minted kinds through `retain_id_kinds`, which matches TC-2184. Wiring quire-cli is a separate decision and is not required for this PR. | spec/functional/FR-077-trace-search-index.md (AC-2); src/symbols/trace.rs:514-518; src/symbols/trace_search.rs:196-248 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `tc2110_load_refusals` now loads `trace_tags.legacy` and asserts that the error names `legacy` (shell_symbols.rs:424-428); it passes. |
| FND-002 | fixed | TC-1095 now asserts `AssuranceRelation::Verifies { form, .. } if form == "rust-trace"` (assurance_export.rs:394-404); it passes. |
| FND-003 | deferred | The PR body edit for the per-repository coverage delta and the name-only count is not yet applied. The coordinator asked that it not be judged this round; recheck it before merge. |
| FND-004 | fixed | Round 2 (head `54940b4`): FR-077-AC-2 now reads "`bind()` records every token; a consumer that supplies the corpus's minted kinds through `SymbolGraph::retain_id_kinds` (FR-051-AC-49, CR-215) keeps only tokens of those kinds". That matches the code and TC-2184. |
| FND-003 | fixed | Round 2: the PR body "Coverage delta" section now gives quire-rs alone at 1410/1921 after (1411/1921 with the corpus bump, the corpus tree contributing one backed row) and the name-only count: 40 symbols (42 `rust-test-name-id` relations) of the 197 retagged. Caveat, not blocking: the "before" figure of 1419/1910 includes the corpus and predates the #626 merge, so the before/after comparison is approximate. |
