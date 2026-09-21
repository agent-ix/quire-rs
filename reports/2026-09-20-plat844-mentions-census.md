# PLAT-844: `SymbolGraph.mentions` size census, pre-hardening

- Date: `2026-09-20`
- Purpose: the coordinator's own condition before treating the `mentions` design as final — "when the pass first runs, count total `mentions` across the six-repo corpus and tell me, along with the worst single id." Not a filtering exercise: fixtures are counted, not excluded, because silently dropping them would be exactly the failure mode FR-077 exists to avoid.
- Harness: `examples/plat844_mentions_census.rs`, run once, `cargo run --release --example plat844_mentions_census`, over the same six repos and shared module (`spec-artifacts-process`) `plat840_rust_baseline_sweep` used, at whatever commit each repo's local `~/dev` checkout was pinned to at run time (this is a size measurement, not a differential — unlike the PLAT-843 report, no cross-engine identity matching is needed, so exact pinning is not load-bearing here).
- Population: `extract_tree` (unscoped — no `source_exclude` filtering), so fixture trees are included on purpose.

## Headline

| Repo | `mentions` |
|---|---:|
| quire-rs | 4,652 |
| quire-code-rs | 328 |
| quire-contract-ir | 300 |
| quire-protocol | 611 |
| filament-ide-rs | 10,189 |
| ecaz | 1,319 |
| **total** | **17,399** |

By bucket, all six repos combined: `mention` (generic, no declared form matched) 14,688; `evidence_near_miss` 2,557; `production_orphan_tag` 154.

**Against the coordinator's own stated thresholds: a few thousand, or tens of thousands.** 17,399 is neither cleanly — closer to "a few thousand" scaled across six repos and every fixture tree than to the 50k+ that would clearly demand an engine-side change, but large enough that a CLI consuming this raw is not free to print every citation for a common id without grouping. Reported as the actual number rather than rounded toward either reading; the CLI-side grouping/path-filter decision this was meant to inform is the follow-up ticket's to make, not this PR's.

## Worst 10 single ids (repo, id, count)

| Count | Repo | Id |
|---:|---|---|
| 573 | filament-ide-rs | `AGPL-3` |
| 278 | filament-ide-rs | `FR-001` |
| 278 | quire-rs | `FR-001` |
| 214 | quire-rs | `TC-001` |
| 155 | quire-rs | `FR-001-AC-1` |
| 129 | filament-ide-rs | `Plan-009` |
| 124 | filament-ide-rs | `FR-031` |
| 105 | filament-ide-rs | `FR-030` |
| 90 | quire-rs | `TC-002` |
| 78 | filament-ide-rs | `FR-020` |

## A real finding, not a guess: `AGPL-3` is a false-positive class, not a trace id

The single worst offender, `AGPL-3` (573 hits, filament-ide-rs), is not a mention of any trace id at
all. `[A-Z]{2,4}-[0-9]+` — the generic, form-agnostic pattern `find_mentions` deliberately reuses from
`generic_tags` rather than declaring a second one (FR-077-CON-1) — matches `AGPL-3` inside the SPDX
license header (`AGPL-3.0-or-later`) that this ecosystem's source files carry. The pattern was never
scoped to a known id-prefix vocabulary (the engine "knows nothing of FR/AC/TC," per `traceability.rs`,
so it cannot know license identifiers either), so a license header repeated across hundreds of files
produces hundreds of `mention` rows that are honestly classified — they genuinely are not claims — but
are not trace-id citations in the sense an agent asking "what cites FR-047" cares about.

**Not filtered here**, and not proposed as an engine-side filter: doing so from a guess at what "looks
like" a license string is exactly the kind of judgment call FR-077 exists to keep out of the engine
(`traceability.rs`'s own stated boundary — the engine has no vocabulary to draw that line correctly for
every repo). It is named so the follow-up `quire-cli` ticket has a concrete, measured example to design
its grouping/filter UX against, rather than an abstract "some noise is expected."

The rest of the top 10 is the second, unsurprising and already-anticipated cause: low-numbered
placeholder/example ids (`FR-001`, `TC-001`, `FR-001-AC-1`, `TC-002`) used pervasively across fixtures,
templates and this very report's own kind of measurement scaffolding, plus one repo-specific outlier
(`Plan-009`, filament-ide-rs) that is a real, repeated planning-document reference, not a defect in the
pattern.

## What this does and does not decide

This measurement answers the one question it was commissioned to answer — the raw size and the worst
offender, named rather than estimated. It does not decide whether `quire-cli`'s follow-up `trace`
subcommand needs a `--group-by-id`/path-filter flag by default; that is explicitly the follow-up
ticket's call, informed by this number.

## Cross-check: does this false-positive class already contaminate `unmatched_tags` (PLAT-840's published `tagged_not_bound`)?

Asked directly by the coordinator once `AGPL-3` surfaced above, because `find_mentions` reuses the
same generic pattern `generic_tags()` uses to produce `unmatched_tags` — the population PLAT-840
published as `tagged_not_bound`, that PLAT-867 characterises for ecaz, and that PLAT-875 is filed to
give an enumerable accessor. **`generic_tags()` and `unmatched_tags` are untouched by this PR**; this
section only measures whether a pre-existing false-positive class was already present in them.

Methodology matched to PLAT-840's own (`extract_tree_scoped` with the module's declared
`source_exclude` globs, not the unscoped `extract_tree` the headline census above deliberately used) —
confirmed correct by exact reproduction on two of the six repos: quire-rs `tagged_not_bound` = **20**
(published: 20) and ecaz = **116** (published: 116), both bit-for-bit. quire-contract-ir (0 vs
published 1) and filament-ide-rs (20 vs published 24) differ slightly, attributable to each repo's
local checkout having moved since PLAT-840's measurement rather than to methodology — the two exact
matches confirm the method itself reproduces the published figures.

**Answer: non-zero, confined to filament-ide-rs, 2 of the current 20 `tagged_not_bound` rows.**

| Repo | `unmatched_tags` entries with a license-shaped `trace_id` |
|---|---:|
| quire-rs | 0 |
| quire-code-rs | 0 |
| quire-contract-ir | 0 |
| quire-protocol | 0 |
| filament-ide-rs | **3** (2 of which inflate `tagged_not_bound`; see below) |
| ecaz | 0 |

All three hits carry the exact id `AGPL-3` (from an `AGPL-3.0-only`/`AGPL-3.0-or-later` SPDX header);
no other license-prefix shape (`GPL`, `LGPL`, `MPL`, `BSD`, `MIT`, `ISC`, `Apache`) appeared anywhere in
any of the six repos' `unmatched_tags`.

**Two of the three are confirmed, concrete contamination of `tagged_not_bound`:**

- `fuzz/fuzz_targets/deep_link_url.rs:2` — `// SPDX-License-Identifier: AGPL-3.0-only`, the entire
  11-line file's only id-shaped text. The Rust extractor gives a `fuzz_target!` invocation a symbol
  whose span is the **whole file** ("the macro invocation declares no `fn`" — `rust.rs`'s own
  documented behaviour), so this license line sits inside the `fuzz_target` symbol's leading annotation
  block with nothing else to bind it. This symbol carries no other trace tag at all, so it is
  genuinely untagged and unbound — its `tagged=true` status in the census is caused *entirely* by the
  SPDX header, confirmed directly (`symbol_is_bound_by_something_else=false`).
- `fuzz/fuzz_targets/settings_document.rs:2` — byte-identical shape, same file structure, same
  conclusion.

**The third is not contamination**, confirmed the same direct way:
`crates/filament-cutover/src/license.rs:135`, the doc comment on
`tests::tc_484_missing_corresponding_source_mechanism_fails_the_gate`, reads "the license gate fails
when the packaged distribution lacks the AGPL-3.0 Corresponding Source ... mechanism" — legitimate
prose describing what the test covers, on a test that itself binds via its own real `TC-484` tag
(`symbol_is_bound_by_something_else=true`). A bound symbol exits `BindingCensus::observe` before any
stray token in its span is ever considered for `tagged_not_bound`, so this row sits in
`unmatched_tags` (correctly — `AGPL-3` itself never bound) but does not inflate `tagged_not_bound`.

**So: filament-ide-rs's `tagged_not_bound` is 2 rows smaller than a reader would assume it to be** —
18 genuine near-miss/untagged evidence symbols, not 20, in the current tree (18 of the published 24,
by the same reasoning, if the same two fuzz targets existed unchanged at PLAT-840's measurement
commit — plausible, since the mechanism is structural rather than transient, but not independently
re-verified at that exact historical commit here). **The other five repos are clean: zero
license-shaped tokens anywhere in their `unmatched_tags`, so PLAT-867's ecaz characterisation (116,
exact match above) and quire-rs's 20 are unaffected by this class.** quire-contract-ir's published `1`
could not be re-verified either way (current tree measures 0 `tagged_not_bound` for that repo,
consistent with repo drift, not with a corrected/re-classified figure).

This is a correction to report to whoever owns PLAT-840/867/875's numbers, not something fixed here:
`generic_tags()`, `unmatched_tags` and `BindingCensus` are all unmodified by this PR, per the
coordinator's explicit instruction.
