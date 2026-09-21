# Rule-zero baselines for the three Jev lens candidates (PLAT-837 / PLAT-838 / PLAT-839)

- Date: `2026-09-20`
- Scope: **measurement only.** No lens code was written, nothing was integrated, and nothing under `src/` was touched. This report and its three data files are the whole change.
- Why now: all three tickets carry the same clause — *"Measure the current behaviour before any Jev code lands. Once the existing pass is replaced, the before-number is unrecoverable and no improvement can be proven."* A PLAT-837 implementation worktree appeared while these runs were in flight, so this was the last moment the before-number was available.
- The second question this answers, independent of whether Jev is ever adopted: **nobody had ever measured whether our current review passes are reproducible.** They are not. That is a finding about the tool we ship today and it stands on its own.

> **Revision note (post-review).** This report was reviewed and 15 findings were raised, nearly all confirmed by independent recomputation. Every published figure below has been re-derived. Four changed: the corpus digest recipe was path-dependent and has been replaced (§Corpus), the spec byte count was wrong, the M5 accounting for `spec-review` covered 9 of 12 runs and now covers all 12, and the M8 skip rate now leads with its floor over a frozen population. One claim that was a prediction is now a measurement (§Method). Changes are marked **[rev]**.

## Corpus: `agent-ix/ix-trace-rs` @ `2ce4ebf47f726b9d76388220545cd0abda8a5cfb`

Chosen because it is small enough to run 20 times per pass, and complete enough to be real: a full spec bundle, a Test Matrix, a source tree, three test suites, and two rows the repository itself honestly marks pending.

| | |
|---|---:|
| FR / NFR / StR / US / TestMatrix documents | 3 / 1 / 1 / 1 / 1 |
| Acceptance + stakeholder-validation criteria | 21 |
| Requirement-bearing statements (EARS scope) | 23 |
| Test Case rows in the matrix | 11 |
| `spec/` bytes, all 8 files **[rev]** | 35,343 |
| Source + test bytes (`src/lib.rs` + the 3 suites named to the runs) | 18,807 |

**[rev] The `spec/` figure was published as 32,596 in the first version. That was wrong** — no subset of the 8 spec files sums to it. The correct total over all 8 is **35,343**, reproducible with `git archive <commit> | tar -x` then `find spec -type f | sort | xargs wc -c`. The per-file breakdown is in the JSON. The source+test figure (18,807) was correct and is unchanged. Neither figure is an input to any other number in this report.

### [rev] Corpus integrity: what was claimed, what was wrong, and what replaces it

The first version published a digest of `a1d4fa2d…` described as a content digest. **It does not reproduce from the commit, and the review was right to say so.** The recipe was:

```
find "$SCRATCH/corpus" -type f | sort | xargs sha256sum | sha256sum
```

`find` was given an **absolute** path, so every line fed to the outer `sha256sum` carried the full scratch-directory path alongside the file hash. The digest is therefore a function of *where the tree was extracted*, not only of its contents, and nobody else could ever have reproduced it. That defeats the entire purpose of publishing a content digest instead of a commit id.

Root cause confirmed by re-deriving it: re-extracting the same commit to the same path reproduces `a1d4fa2d…` exactly. **That re-derivation also answers the second question the review asked** — whether the scratch tree held anything beyond `git archive` output. A tree built by `git archive` alone reproduces the original digest bit-for-bit, and `find -type f` enumerates every regular file, so an extra file would have changed the inner listing and therefore the outer hash. The prompt files and the statement inventory lived in the parent scratch directory, never inside the corpus tree.

**[rev] Two limits on that inference, since it is the one claim here a reader cannot check.**

- `find -type f` sees only **regular files**. "The measured tree held no extra *file*" is exactly what the evidence supports; "held nothing beyond `git archive` output" is a shade broader than that, and is not claimed.
- This link is **author-only and not externally reproducible**, because the scratch path is deliberately unpublished (it is a developer path, and this is a public repository). What keeps it from being circular is that **`a1d4fa2d…` was committed before the question was asked** — it appears in the first revision of this file, timestamped in this repository's history, so the target was fixed in advance and could not have been chosen to fit. A reader who declines to take it on trust loses only this inference; the path-independent digest below is checkable without it, and the review verified it from two directories that the author has never seen.

**The path-independent digest, which anyone can check:**

```
git archive 2ce4ebf47f726b9d76388220545cd0abda8a5cfb | tar -x -C <dir>
cd <dir> && find . -type f | sort | xargs sha256sum | sha256sum
→ 2beba8e8bbf691db0dabbd5f0bb320a27675b415a3127a46cbc559352233102c
```

**What survives from the original claim, and what does not.** The *internal* claim survives intact: the same recipe over the same tree produced the same digest before the first run and after the last, so the corpus provably did not drift mid-measurement, and the source repository's `HEAD` and clean working tree were re-checked at the end. The *external* claim — that a reader could verify the corpus independently — did not survive, and is now restored by the digest above.

### Deterministic engine output over that tree (the half that *is* reproducible)

`quire 0.32.2 (engine 0.46.0@acd1be63)`:

- `quire validate --scope . "spec/**/*.md" --summary` → `6/8 docs grammar-clean (75%)`, exactly two EARS findings: `ears:non-singular` at `spec/functional/FR-003-argument-grammar.md:51` and `ears:vague-response` (`be able to`) at `spec/stakeholder/StR-001-compiler-checked-trace-markers.md:16`. One structural error: `spec/tests.md:41`, the `functional_coverage` table's `Coverage Status` column against an asserted `Status`.
- `quire coverage --scope . --json` → backed 26 / 32; three unbacked rows (`FR-003-AC-7` method Demonstration, `TC-013` type Manual, `NFR-001-AC-3` type Inspection); zero status lies; zero untracked tags; binding census rust 10/10.

These numbers were identical on every invocation. Everything unstable in this report is on the semantic side.

## Method, stated before the numbers

Each of the three passes was executed **N times over the byte-identical corpus, one Claude Code subagent per run**, each given the same prompt file by path and nothing else. The prompt reproduces the skill's own instruction text verbatim and hands the run the already-computed deterministic engine output so that only the semantic judgment varies. Runs were independent: no run saw another's output.

- Runner model: `claude-opus-5[1m]`, stated because the number is a property of the model as much as of the skill.
- For `spec-ears-analysis` and `spec-review` the unit of judgment is fixed in advance (a numbered inventory of the 23 statements; the 21 named criteria), so a run's disagreement is about the *verdict*, not about which items it chose to look at. For `gap-analysis` the finding set is open, which is the real behaviour of that pass.

A "finding changes" per the tickets' definition: it appears in one run and not another, OR changes severity, OR changes its label. Disagreement is reported **pairwise** — over all `C(N,2)` run pairs, the share of judged items on which the two runs differ.

### [rev] Denominators — the three passes do not share one, and the first version did not say so

This is the single most important methodological caveat in the report, and it was missing.

`gap-analysis` has an **open** finding set, so its published 43.5% uses a **per-pair union denominator**: for each run pair, the classes raised by either run. The other two passes judge a **fixed inventory**, so every item counts in every pair. Those denominators are not interchangeable, and the first version put all three in one column and then compared them.

Both figures are given below and **the cross-pass comparison is made only on the common denominator.** On the fixed 16-class inventory the same gap-analysis data gives **33.4%** — recomputed and confirmed.

### [rev] The clustering step, and the collapse rule the review asked for

`gap-analysis` finding keys are free text, so they were mapped by hand into 16 canonical defect classes. Two things the first version should have stated:

1. **No many-to-one collapse occurred.** Checked programmatically across all 194 raw findings: **zero** runs produced two findings mapping to the same class. The mapping is injective within every run, and combined findings (one finding naming two defects) were counted against both classes at that severity — one-to-many only. **No severity-collapse rule was ever exercised**, so the counterexample the review constructed (two findings collapsing to one class, taking a pair from 50% to 100% under a first-listed rule) does not arise in this data. The general claim is therefore narrowed to what was shown: *clustering lowered disagreement in every case observed here, and is guaranteed to do so only under a monotone severity collapse such as max-severity.*
2. **The raw keys are published**, in the TSV's `raw_key` column, so the mapping can be independently re-derived and redone. **[rev]** The first version offered as evidence that "the reconstructed raw-key table reproduces the published class table exactly" — which checks nothing, since `raw_key` and `unit` are two columns of the same rows and cannot disagree.

   The substantive property, and the one that was actually checked: **of 105 distinct keys, 99 map to exactly one class in every run they appear in.** The other 6 are the combined findings. **Two of those 6 are not stable across runs**, and that is worth stating rather than smoothing:

   | Key | Class set per run |
   |---|---|
   | `fr-001-con-1-declares-test-validation-with-no-test` | A15 → D10; **A16 → D10+D11** |
   | `nfr-001-ac-2-substring-assertion-is-weak` | A05 → D5; A08 → D5; **A14 → D5+D15** |

   In both cases the slug was reused for findings of *different scope* — A16's named both constraints where A15's named one; A14's added the manifest-table clause where A05's and A08's did not. **So the raw slug is not a perfect identity for a finding, and 90.6% slightly understates disagreement at that granularity**, because it scores those pairs as agreeing. Reported because it cuts against this report's own figure.

**Counting note for anyone recomputing from the TSV.** The gap-analysis section has **200 rows** but represents **194 raw findings**: the TSV carries one row per (run, class), so each of the 6 combined findings — one finding naming two defects — appears twice under the same `raw_key`. Deduplicate on `(run, raw_key)` to recover the 194. The six are `A03`/`A16` `fr-001-con-…` (D10+D11 — `…-and-fr-002-con-1-untraced` in A03, `…-1-declares-test-validation-with-no-test` in A16) and `A14`/`A16`/`A17`/`A20` `nfr-001-ac-…` (D5+D15). The 90.6% raw-key figure is computed over the deduplicated 194, the 33.4% and 43.5% over the 200 class rows.

**[rev] "Raw string-identity disagreement would be higher" was a prediction. It is now measured: 90.6%.** The 20 runs produced **105 distinct free-text keys** for 16 underlying defects. Two runs of this pass agree on the literal wording of a finding roughly one time in ten.

**[rev] Both figures are granularity-dependent, and neither is an absolute bound.** 33.4% is the lower bound **at this clustering granularity** — a coarser clustering would push it lower, and 16 classes is a choice, not a fact. 90.6% is disagreement at raw-slug granularity, and per the table above it slightly *understates* even that, since two slugs were reused for findings of differing scope. The honest statement is that how unstable this pass looks depends on how finely you read it, across a range from 33.4% to somewhat above 90.6%, and that every point in that range is bad.

## Verbatim check against the ticket text

All three tickets describe the skills accurately as of today.

- `gap-analysis/SKILL.md`, in the section headed **"The optional semantic review"**, calls it *"an expensive, judgment-heavy LLM pass"*, gates it behind an explicit yes/no, and instructs: *"If yes, fan the work out (one subagent per FR or per area) for thoroughness."* **[rev]** — the first version truncated this quotation to "one subagent per FR", dropping "or per area", in a section whose whole purpose is a verbatim check.
- `spec-ears-analysis/SKILL.md` step 2 is headed, verbatim, *"what the engine cannot check deterministically"*.
- `spec-review/SKILL.md`'s Common Issues carries *"Vague Criteria: 'Fast', 'User-friendly'. -> Make measurable."* and nothing else on criterion strength.

**No discrepancy between ticket text and skill behaviour to report.**

---

## Headline

| Pass | N | Pairwise disagreement, **fixed-inventory denominator** | Same, union denominator | Verdict / label stability |
|---|---:|---:|---:|---|
| `gap-analysis` step 5 (PLAT-839) | 20 | **33.4%** | 43.5% (open finding set) | Verdict changed **0/20 — all FAIL, forced by one unanimous `high` finding; not a general stability result** |
| `spec-ears-analysis` step 2 (PLAT-838) | 20 | **28.7%** | n/a — inventory is fixed | `ears_pattern` choice label alone differed on **16.7%** of statement-pairs |
| `spec-review` Vague Criteria (PLAT-837) | 12 | **17.0%** | n/a — inventory is fixed | 1 of 21 criteria flagged by every run — severity still split 8 medium / 4 low |

Comparing only the fixed-inventory column, which is the one comparison the denominators support: **`gap-analysis` is the least reproducible of the three passes and `spec-review` the most, and `spec-review` is most reproducible because it finds least.**

**The answer to "is our current semantic review a fact or an opinion?" is: an opinion, with a reproducible core.** Between two runs over identical bytes, a third of `gap-analysis`'s defect classes differ — and nine in ten of its finding *texts* do. Nothing in the pipeline notices.

## M1 / M6 — `gap-analysis` step 5 and Verdict stability (PLAT-839)

N = 20. Every run returned a **FAIL** verdict, so the measured **verdict change rate is 0%**.

**That stability is real but it is not a property of the pass.** Every one of the 20 runs raised `D1` at `high` severity, and one `high` finding is sufficient for FAIL under the skill's own verdict rule. A corpus with no unanimous `high` finding would have no such protection. That is a mechanism claim derived from the verdict rule, not a second measurement — it was not tested here, and testing it needs a second corpus near the PASS/CONDITIONAL boundary. **The honest reading: the Verdict was reproducible on this corpus, and this measurement does not establish that the Verdict is reproducible in general.**

**[rev] There is a second, stronger reason the 0/20 says little about semantic stability.** The deterministic input handed to every run reports `TC-013` as an unbacked row, and the skill's verdict rule fails on *"any matrix Test Case with no backing tagged test"* — a clause with no severity qualifier. Whether a row whose declared type is `Manual` counts as "no backing tagged test" is genuinely arguable, and the runs split on it: 18 of 20 raised it at `medium`, 2 at `high`. **If that clause fires, the Verdict was pinned FAIL by the deterministic half before the semantic pass ran at all, and the 0/20 is evidence about neither semantic stability nor semantic instability.** Either way the figure cannot be read as "the Verdict is reproducible".

**[rev] Class rows per run ranged 7 to 14 (mean 10.0); raw findings per run ranged 7 to 12 (mean 9.7)** — 200 class rows, 194 raw findings, 105 distinct keys, over the identical tree. The two differ because the 6 combined findings each occupy two class rows; the first version gave only the class-row figures while labelling them "findings", so dividing 194 by 20 contradicted the stated mean.

| Class | What it is | Raised in | Severities |
|---|---|---:|---|
| D1 | `FR-001-AC-5` trailing-comma criterion has no fixture that carries a trailing comma | 20/20 | high ×20 |
| D2 | `TC-007` row claims `FR-AC-1` is rejected; `is_id_shaped` accepts it | 20/20 | medium ×20 |
| D3 | FR-001's invisible-group SHALL has no criterion, no row, no test | 20/20 | medium ×20 |
| D5 | `NFR-001-AC-2` verified by a `use <crate>` substring scan | 20/20 | medium ×18, low ×2 |
| D7 | `FR-003-AC-7` / `TC-013` has no automated trace | 20/20 | medium ×18, **high ×2** |
| D8 | `is_id_shaped` doc comment contradicts the implemented rule | 16/20 | low ×15, medium ×1 |
| D10 | `FR-001-CON-1` declares Test validation with no tagged test | 16/20 | medium ×16 |
| D6 | "item still emitted on rejection" is not gated by any ui fixture | 12/20 | medium ×12 |
| D9 | `TC-744` carried by two symbols, against the matrix's own Rule 2 | 12/20 | medium ×8, low ×4 |
| D4 | the "below `#[test]`" half of FR-001's ordering is untested | 9/20 | medium ×9 |
| D12 | `NFR-001-AC-3` inspection-only with no recorded evidence | 9/20 | low ×8, medium ×1 |
| D15 | `NFR-001-AC-1` manifest check misses `[dependencies.x]` / target tables | 9/20 | medium ×8, low ×1 |
| D14 | invisible-group arm accepts a group with no separating comma | 7/20 | low ×4, medium ×3 |
| D13 | `spec/tests.md:41` column-header mismatch | 5/20 | low ×5 |
| D11 | `FR-002-CON-1` declares Test validation with no tagged test | 4/20 | low ×1, medium ×3 |
| D16 | `r"TC-707"` (raw string literal) is rejected as a non-literal | **1/20** | low ×1 |

Five of 16 classes were unanimous; **only three were unanimous *and* severity-identical**. Eleven classes were raised by fewer than 20 runs — **a reader of any single `gap-analysis` report is seeing a sample, not an inventory.**

Two results deserve naming:

- **D7 was raised at `high` by 2 of 20 runs and `medium` by 18.** Under the verdict rule a `high` finding forces FAIL. Here the Verdict was already FAIL, so the flip changed nothing — but severity assignment, which the verdict rule depends on, is itself unstable.
- **D16 appeared exactly once in 20 runs, and is a genuine code-behaviour finding** at `src/lib.rs:122-125`: acceptance strips a leading and trailing `"` from the literal's text, so `r"TC-707"` — a well-formed string literal containing a valid id — is rejected. A real defect with a 5% chance of being reported is the clearest single argument in this report for a reproducible lens.

## M1 / M6 — `spec-ears-analysis` step 2 (PLAT-838)

N = 20 over the fixed 23-statement inventory (published in the TSV; scope is FR Description/Behavior/Constraints, NFR Statement, StR Stakeholder Need, per the skill).

- **Combined disagreement (semantic finding OR `ears_pattern` label): 28.7%** of statement-pairs.
- Finding presence/severity/kind alone: **20.1%**.
- `ears_pattern_actual` choice label alone: **16.7%**.

The label figure matters because PLAT-838 proposes to make that label the finding: *"The finding is the delta between the pattern the engine parsed from the keyword and the pattern Jev says the sentence means."* Today, on identical text:

- `ST-02` and `ST-08` (`... whenever the argument list is well formed`) were read `state_driven` by 15 runs and `event_driven` by 5.
- `ST-12`, `ST-14`..`ST-17` (the rejection bullets) were read `unwanted_behaviour` by 11–12 runs and `ubiquitous` by 8–9.
- `ST-19` split 12 `unwanted_behaviour` / 8 `ubiquitous`.

**The delta this lens is built on is, today, as likely to come from run-to-run variance as from a real keyword/intent mismatch on a third of the statements that carry one.**

### M6 — the number that decides whether PLAT-838 ships

The ticket is blunt: *"if the delta is near zero, this lens adds nothing over the grammar engine and should not ship."*

Denominator: the **21 statements the deterministic engine passes clean** (all but `ST-20` and `ST-23`).

| | Statements flagged | % of the 21 |
|---|---:|---:|
| Lowest-flagging run | 2 | 9.5% |
| **Mean over 20 runs** | **4.0** | **19.0%** |
| Highest-flagging run | 9 | 42.9% |
| Union over all 20 runs | 12 | 57.1% |
| Flagged by **every** run | **0** | **0.0%** |

**The delta is not near zero — a single run flags about 19% of engine-clean statements — so on this evidence the lens is not redundant against the grammar engine.** Two caveats belong next to that number, not below it:

1. **Not one statement was flagged by all 20 runs.** The `gap-analysis` pass had five unanimous classes; this pass has none. The 19% is a mean over a distribution running 9.5%–42.9%, so "19% of statements have a semantic defect the engine misses" would overstate what was measured. What was measured is that *a run* flags about 19%, and which ones it picks is substantially a coin flip.
2. The most-flagged statement, `ST-10` (`The two attributes SHALL be usable together ...`), was raised by 18/20 as an unmeasurable capability phrase — exactly the class the ticket predicts the `vague-response` **denylist** will miss, since `usable together` is not in `support`/`handle`/`manage`/`process`/`provide`/`enable`/`be able to`. One concrete, repeatable instance of the lens earning its place.

**Inverse direction** (engine findings the semantic pass judged acceptable — candidate engine false positives, which the ticket asks to route back to quire-rs): `ST-20` was flagged by 0/20 semantic runs, `ST-23` by 1/20. Neither engine finding was *contradicted* either — the prompt asked for additional semantic findings, not review of the engine's, so **this measurement cannot distinguish "the semantic pass agreed" from "the semantic pass was not asked". The engine false-positive rate is not measured here** (see the unmeasured table).

## M1 — `spec-review` Vague Criteria (PLAT-837)

N = **12**, not 20. Stated plainly: the session's subagent concurrency cap was 20 and the gap-analysis and EARS batches had priority for the full N. A smaller honest N is reported rather than a claimed 20.

Over the 21 criteria: **17.0% pairwise disagreement.** Findings per run ranged 2 to 7 (mean 3.7).

| Criterion | Flagged in | Severities |
|---|---:|---|
| FR-003-AC-7 | **12/12** | medium ×8, **low ×4** |
| NFR-001-AC-3 | 8/12 | low ×8 |
| FR-002-AC-2 | 6/12 | low ×6 |
| NFR-001-AC-2 | 6/12 | low ×6 |
| FR-002-AC-1 | 4/12 | low ×4 |
| FR-003-AC-1 | 4/12 | low ×3, medium ×1 |
| StR-001-VC-1 | 4/12 | low ×4 |

14 of 21 criteria were never flagged by any run. Exactly one was flagged by every run — and **even that one's severity was not reproducible**, splitting 8 medium / 4 low. Five of the seven flagged criteria sit near a coin flip.

This is the pass with the lowest disagreement rate of the three, and that is not a compliment: it is low because the pass finds little. The by-eye check is stable mostly where it is silent.

## M5 — cost and latency per full pass

**[rev]** Measured from the harness's own per-subagent accounting. The first version's `spec-review` row reported a token total covering **9** of the 12 runs against an N of 12 — the three runs that reported last were omitted from the accounting but counted in N, so the implied mean fell below the stated minimum. **All 12 are now included**, and every row now satisfies the check that total ÷ N lies within [min, max].

Runs were executed concurrently in batches, so **wall-clock is per-run duration under concurrency, not the latency of a single isolated pass**; treat it as an order-of-magnitude figure.

| Pass | N | Wall-clock median (min–max) | Tokens/run median (min–max) | Tokens total |
|---|---:|---|---|---:|
| `gap-analysis` step 5 | 20 | 119.1 s (91.1–144.4) | 73,209 (70,537–77,442) | 1,471,906 |
| `spec-ears-analysis` step 2 | 20 | 80.7 s (59.6–97.5) | 60,587 (49,311–65,157) | 1,218,260 |
| `spec-review` vague criteria | **12** | 39.5 s (28.2–47.0) | 53,353 (52,311–54,036) | **640,066** |

52 runs, 3,330,232 tokens in total.

**The dollar figure is not reported, because it was not measured.** The harness reports one total token count per subagent and does not expose the input / output / cache-read split, and those price differently. Anyone with a rate card can apply it to the token column; inventing the split to produce a dollar number would be exactly the thing this report exists to stop. The *input-token* figure PLAT-839's M5 asks for is therefore also **not measured**.

Note the scale against PLAT-839's own estimate. That ticket projects *"roughly 300 triples for a ~100-FR repo"* at *"about $0.03 per full audit"*. This corpus is 3 FRs, and one pass cost ~73k tokens. Whatever the Jev figure turns out to be, **the baseline it must beat is an LLM pass whose cost scales with the whole repository, measured here only at the 3-FR end.** Extrapolating to 100 FRs was not attempted and is not offered.

## M8 — the historical skip rate (PLAT-839)

PLAT-839 claims *"the most valuable check stopped being the one that gets skipped."* **This is measurable from existing run history, and was measured.**

Population: every Markdown document in the local multi-repo checkout whose frontmatter carries `type: SpecReview` and `analysis: gap-analysis`, **deduplicated by content hash**. The raw file count is 3,930, reduced to 483 distinct documents — an **8.1× ratio [rev]**, explained by **duplicate checkouts (worktrees and full mirror clones) holding identical copies [rev]**; the original version attributed this to worktrees alone, which is only part of the mechanism. An independent sweep on a later day with a looser predicate found 4,573 → 492, a 9.3× ratio, confirming the magnitude on a different population.

> **[rev] How this sentence got a wrong number, recorded because it is this report's own thesis.** The second revision published "9.3×" here while keeping 3,930 and 483 either side of it — 3,930 / 483 is 8.1. The 9.3 was the reviewer's ratio, from their own numerator and denominator, adopted while fixing a *different* clause of the same sentence. The first version's "roughly eightfold" had been right. A correct number was replaced with a wrong one in the act of correcting the paragraph around it, which is exactly the failure mode this report exists to document. Every figure touched in this revision was re-derived from this report's own data before being written, including the ones handed over by the review. No content group contains differing basenames, so the deduplication cannot have merged genuinely distinct reviews.

**[rev] Both classifiers now run over one frozen population** (469 documents, snapshotted and held in memory before either ran). The first version ran them over 483 and 469 documents respectively — other agents were active in these trees and 14 files disappeared between the sweeps — which conflated a broader regex with a changed corpus. Freezing removes that confound entirely, at the cost of 14 documents. This is the same byte-identity discipline applied to the spec corpus, applied here.

| Classifier | Docs | skipped | ran | ambiguous | not stated | Skip rate over docs that state it |
|---|---:|---:|---:|---:|---:|---:|
| Narrow — lines matching `semantic review` | 469 | 142 | 42 | 8 | 277 | 77.2% |
| Broad — `semantic (review\|pass\|mode)` | 469 | 176 | 43 | 8 | 242 | 80.4% |

On one population the classifier choice is worth **3.2 points**, not the 2.2 the first version implied.

### [rev] The headline number is the floor, and here is why

**At least 142 of 469 distinct audits — 30.3% — explicitly recorded that they skipped the semantic review.** Under the broad classifier the floor is 37.5%. At most **9.0% (narrow; 9.2% broad) [rev]** explicitly recorded that they ran it. *(The first version quoted a floor of 31.3% from the pre-freeze population of 483; freezing moved it to 30.3%.)*

**The 77.2% / 80.4% figures are biased upward by an unbounded amount, and the direction is structural.** `gap-analysis/SKILL.md` requires recording the **skip** only: *"If yes, fan the work out … If no, note in the SpecReview's Coverage section that semantic review was skipped."* A run that performed the review is under no obligation to say so. So "documents that record the decision" systematically over-samples skips, and the ratio computed over it cannot be read as the skip rate. The first version named the opposite-direction hypothesis (that unrecorded runs are more likely skips) and missed this one, which cuts against its own conclusion.

The floor is the number that survives an adversarial reading: it counts only explicit statements, is unaffected by the recording asymmetry, and does not require assigning the 242–277 silent documents in either direction.

The largest single contributor is `filament-ide-rs` at **69 of the frozen 469 — 14.7% [rev]** (the second revision gave "69 of 483, 14%", mixing the pre-freeze population into the sentence right after freezing it). Recomputed without it on that same frozen population, the over-stating rate is **77.1%** (narrow) and **80.9%** (broad) — the finding does not depend on any one repository.

**PLAT-839's premise is supported: at minimum three in ten recorded audits explicitly skipped the semantic review, and explicit confirmations that it ran are outnumbered better than three to one.**

## What could not be measured, and why

| Metric | Status | Why |
|---|---|---|
| M2 — precision / recall / false-positive rate | **Not measured** | Requires a human-labelled ground-truth fixture set with known-correct answers. None exists for these passes, and building one was outside this task. This is the metric PLAT-837 says decides *"whether the lens ships at all"*, and it is unanswered for the baseline as well as for Jev. |
| M3 — expected calibration error | **Not measurable for the baseline, by construction** | The current passes emit no probability; ECE is undefined over a lens that states no confidence. This is the asymmetry PLAT-837 is pointing at, and it means **M3 will have a post-change number with no before-number to compare against.** Say so when M3 is first published. |
| M4 — abstention rate and precision | **Not measured** | The current passes have no abstention mechanism — a run either raises a finding or does not. There is nothing to count. |
| M6 (PLAT-837) — spec defect escape rate **[rev]** | **Not measured, and the feedback path does not exist** | Requires recording which lens verdict preceded each weak AC found late. No such path exists today, so the metric cannot be computed even retrospectively. PLAT-837 puts building that path in its own scope. **Omitted entirely from the first version of this table — the silent omission this program exists to end, in the report about ending it.** |
| M7 (PLAT-837) — adverse-case coverage distribution | **Not measured** | The `adverse_case_coverage` rubric does not exist yet; nothing today produces the score. |
| M7 (PLAT-838) — gate-readiness threshold **[rev]** | **Not applicable to a baseline; it is a pre-registration obligation** | Not a quantity to measure now. The ticket requires the FP-rate, ECE and disagreement bounds to be written into the MeasurementPlan **before** the post-change numbers are taken. This report supplies the disagreement figure those bounds must be set against (28.7%); setting them is PLAT-838's job and must happen before its first measurement. |
| M7 (PLAT-839) — `assertion_vacuous` vs a mutation run | **Not measured** | Requires running mutations; PLAT-839 places that out of scope for itself. |
| M5 dollar cost, and input tokens specifically | **Not measured** | The harness exposes one total token count per subagent, not the input/output/cache split. Token totals are reported instead. |
| EARS engine false-positive rate (PLAT-838 inverse) | **Not measured** | The runs were asked for additional semantic findings, not adjudication of the engine's two. Silence about them is not agreement with them. |
| Verdict stability away from a unanimous `high` finding | **Not measured** | Needs a second corpus near the PASS/CONDITIONAL boundary. Compounded by the `TC-013` clause above, which may have pinned the Verdict deterministically. |

## What these numbers change

- **PLAT-839.** The premise holds: at least 30.3% of recorded audits explicitly skipped the semantic review, against at most 9.0% that explicitly ran it. When it does run, a third of its defect classes and nine in ten of its finding texts differ between two runs over identical bytes. The **Verdict** was stable at 0/20, but that was forced by a unanimous `high` finding and possibly by a deterministic unbacked row, so **a gap-analysis PASS is not demonstrated to be reproducible enough to serve as a merge gate.** The before-number now exists and is durable.
- **PLAT-838.** The justifying delta is **19.0% mean (9.5%–42.9%)** of engine-clean statements, not near zero, so the lens is not redundant against the grammar engine. But zero statements were flagged unanimously, and the `ears_pattern` label itself moved on 16.7% of pairs. **Per the ticket's own M7, the bounds must be written into the MeasurementPlan before the post-change number is taken, and 28.7% is the disagreement figure they have to beat.** Do not turn on `quire validate --strict` against anything that does not.
- **PLAT-837.** The by-eye Vague Criteria check flags 3.7 of 21 criteria per run, agrees with itself on 83.0% of judgments, and reaches unanimity on exactly one criterion whose severity it still cannot fix. There is effectively no criterion-strength baseline to beat — which makes the *stability* numbers, not the finding counts, the thing to compare against.

## Files

- `2026-09-20-jev-lens-baselines.json` — every rollup in this report, plus all ten unmeasured metrics with their reasons.
- `2026-09-20-jev-lens-baselines.tsv` — the per-run, per-item judgment record: 20 gap-analysis runs by defect class and severity **with the originating free-text `raw_key` [rev]** (200 rows = 194 raw findings; see the counting note above), 20 EARS runs by statement with both the finding and the `ears_pattern` label, 12 spec-review runs by criterion.
- `2026-09-20-jev-lens-baselines-m8.tsv` **[rev]** — the frozen M8 population: one row per distinct SpecReview with both classifiers' verdicts, so the skip-rate figures are recomputable too.

**[rev] What is recomputable from the data files:** every M1 figure, every M6 figure, the defect-class and criterion tables, and the M8 rates. The following are **not** derivable from the TSVs and are recorded here instead: the M5 cost and latency figures (harness accounting, in the JSON), and the `kind` component of the EARS 20.1% (the TSV carries severity and the `ears_pattern` label, not the finding `kind`). The first version claimed "every percentage above is recomputable from it", which was false for M8, for the 43.5% (whose union denominator was unstated), and for that `kind` component.
