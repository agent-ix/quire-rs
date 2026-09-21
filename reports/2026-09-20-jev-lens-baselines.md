# Rule-zero baselines for the three Jev lens candidates (PLAT-837 / PLAT-838 / PLAT-839)

- Date: `2026-09-20`
- Scope: **measurement only.** No lens code was written, nothing was integrated, and nothing under `src/` was touched. This report and its two data files are the whole change.
- Why now: all three tickets carry the same clause — *"Measure the current behaviour before any Jev code lands. Once the existing pass is replaced, the before-number is unrecoverable and no improvement can be proven."* A PLAT-837 implementation worktree appeared while these runs were in flight, so this is the last moment the before-number was available.
- The second question this answers, independent of whether Jev is ever adopted: **nobody had ever measured whether our current review passes are reproducible.** They are not. That is a finding about the tool we ship today and it stands on its own.

## Corpus: `agent-ix/ix-trace-rs` @ `2ce4ebf47f726b9d76388220545cd0abda8a5cfb`

Chosen because it is small enough to run 20 times per pass, and complete enough to be real: a full spec bundle, a Test Matrix, a source tree, three test suites, and two rows the repository itself honestly marks pending.

| | |
|---|---:|
| FR / NFR / StR / US / TestMatrix documents | 3 / 1 / 1 / 1 / 1 |
| Acceptance + stakeholder-validation criteria | 21 |
| Requirement-bearing statements (EARS scope) | 23 |
| Test Case rows in the matrix | 11 |
| Spec bytes / source+test bytes | 32,596 / 18,807 |

The corpus was extracted with `git archive` into a read-only scratch directory before the first run. Its digest — the SHA-256 of the sorted `sha256sum` listing of every file — was `a1d4fa2de1ab454968890cf134275145381c21369bd5e27ca462dbff474ab6c9` **before the first run and again after the last**, and the source repository's `HEAD` and clean working tree were re-checked at the end. The corpus did not drift mid-measurement.

### Deterministic engine output over that tree (the half that *is* reproducible)

`quire 0.32.2 (engine 0.46.0@acd1be63)`:

- `quire validate --scope . "spec/**/*.md" --summary` → `6/8 docs grammar-clean (75%)`, exactly two EARS findings: `ears:non-singular` at `spec/functional/FR-003-argument-grammar.md:51` and `ears:vague-response` (`be able to`) at `spec/stakeholder/StR-001-compiler-checked-trace-markers.md:16`. One structural error: `spec/tests.md:41`, the `functional_coverage` table's `Coverage Status` column against an asserted `Status`.
- `quire coverage --scope . --json` → backed 26 / 32; three unbacked rows (`FR-003-AC-7` method Demonstration, `TC-013` type Manual, `NFR-001-AC-3` type Inspection); zero status lies; zero untracked tags; binding census rust 10/10.

These numbers were identical on every invocation. Everything unstable in this report is on the semantic side.

## Method, stated before the numbers

Each of the three passes was executed **N times over the byte-identical corpus, one Claude Code subagent per run**, each given the same prompt file by path and nothing else. The prompt reproduces the skill's own instruction text verbatim (step 5 of `gap-analysis`; step 2 of `spec-ears-analysis`; the `Vague Criteria` line from `spec-review`'s Common Issues) and hands the run the already-computed deterministic engine output so that only the semantic judgment varies. Runs were independent: no run saw another's output.

- Runner model: `claude-opus-5[1m]`, stated because the number is a property of the model as much as of the skill.
- For `spec-ears-analysis` and `spec-review` the unit of judgment is fixed in advance (a numbered inventory of the 23 statements; the 21 named criteria), so a run's disagreement is about the *verdict*, not about which items it chose to look at. For `gap-analysis` the finding set is open, which is the real behaviour of that pass, so its disagreement figure includes disagreement about what to look at.
- **`gap-analysis` finding clustering is the one judged step in this report.** Free-text finding keys were mapped by hand into 16 canonical defect classes (`D1`..`D16`) so that two runs describing the same defect count as agreeing. That mapping is an author judgment, not a measurement; the full per-run assignment is published in the TSV so it can be checked and redone. Where a run raised one finding covering two classes, it was counted against both at that severity. **This clustering can only lower the measured disagreement**, since it merges differently-worded descriptions of the same defect — the raw string-identity disagreement would be higher.

A "finding changes" per the tickets' definition: it appears in one run and not another, OR changes severity, OR changes its label. Disagreement is reported **pairwise** — over all `C(N,2)` run pairs, the share of judged items on which the two runs differ.

## Verbatim check against the ticket text

All three tickets describe the skills accurately as of today. `gap-analysis/SKILL.md` calls step 5 *"an expensive, judgment-heavy LLM pass"*, gates it behind an explicit yes/no, and instructs *"one subagent per FR"*. `spec-ears-analysis/SKILL.md` step 2 is headed, verbatim, *"what the engine cannot check deterministically"*. `spec-review/SKILL.md`'s Common Issues carries *"Vague Criteria: 'Fast', 'User-friendly'. -> Make measurable."* and nothing else on criterion strength. **No discrepancy to report.**

---

## Headline

| Pass | N | Pairwise finding disagreement | Verdict / label stability |
|---|---:|---:|---|
| `gap-analysis` step 5 (PLAT-839) | 20 | **43.5%** | Verdict changed **0/20** — all FAIL |
| `spec-ears-analysis` step 2 (PLAT-838) | 20 | **28.7%** | `ears_pattern` choice label alone differed on **16.7%** of statement-pairs |
| `spec-review` Vague Criteria (PLAT-837) | 12 | **17.0%** | 1 of 21 criteria flagged by every run — and its severity still split 4 low / 8 medium |

**The answer to "is our current semantic review a fact or an opinion?" is: an opinion, with a reproducible core.** Between two runs of the same pass over the same bytes, roughly two in five `gap-analysis` findings differ. Nothing in the pipeline notices.

## M1 / M6 — `gap-analysis` step 5 and Verdict stability (PLAT-839)

N = 20. Every run returned a **FAIL** verdict, so the measured **verdict change rate is 0%**.

That stability is real but it is not a property of the pass; it is a property of this corpus. Every one of the 20 runs raised `D1` at `high` severity, and one `high` finding is sufficient for FAIL under the skill's own verdict rule. **A corpus with no unanimous `high` finding would have no such protection.** That last sentence is a mechanism claim derived from the verdict rule, not a second measurement — it was not tested here, and testing it needs a second corpus that sits near the PASS/CONDITIONAL boundary. **The honest reading is: the Verdict was reproducible on this corpus, and this measurement does not establish that the Verdict is reproducible in general.**

Findings per run ranged from **7 to 14** (mean 10.0) over the identical tree.

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

Five of 16 classes were unanimous; **only three were unanimous *and* severity-identical**. Eleven classes were raised by fewer than 20 runs — that is, **a reader of any single `gap-analysis` report is seeing a sample, not an inventory.**

Two results in that table deserve naming:

- **D7 was raised at `high` by 2 of 20 runs and `medium` by 18.** Under the verdict rule a `high` finding forces FAIL. Here the Verdict was already FAIL from D1, so the severity flip changed nothing — but it is a live demonstration that severity assignment, which the verdict rule depends on, is itself unstable.
- **D16 appeared exactly once in 20 runs, and is a genuine code-behaviour finding** (acceptance strips a leading and trailing `"` from the literal's text, so a raw string literal containing a valid id is rejected). A real defect with a 5% chance of being reported is the clearest single argument in this report for a reproducible lens.

## M1 / M6 — `spec-ears-analysis` step 2 (PLAT-838)

N = 20 over the fixed 23-statement inventory (published in the TSV; scope is FR Description/Behavior/Constraints, NFR Statement, StR Stakeholder Need, per the skill).

- **Combined disagreement (semantic finding OR `ears_pattern` label): 28.7%** of statement-pairs.
- Finding presence/severity/kind alone: **20.1%**.
- `ears_pattern_actual` choice label alone: **16.7%**.

The label figure is worth separating out because PLAT-838 proposes to make that label the finding: *"The finding is the delta between the pattern the engine parsed from the keyword and the pattern Jev says the sentence means."* Today, on identical text:

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

**The delta is not near zero — a single run flags about 19% of engine-clean statements — so on this evidence the lens does have something to add over the grammar engine.** Two caveats belong next to that number, not below it:

1. **Not one statement was flagged by all 20 runs.** The `gap-analysis` pass had five unanimous findings; this pass has none. The 19% is a mean over a distribution running from 9.5% to 42.9%, so "19% of statements have a semantic defect the engine misses" would be an overstatement of what was measured. What was measured is that *a run* flags about 19%, and which ones it picks is substantially a coin flip.
2. The most-flagged statement, `ST-10` (`The two attributes SHALL be usable together ...`), was raised by 18/20 as an unmeasurable capability phrase — and it is exactly the class the ticket predicts the `vague-response` **denylist** will miss, since `usable together` is not in `support`/`handle`/`manage`/`process`/`provide`/`enable`/`be able to`. That is one concrete, repeatable instance of the lens earning its place.

**Inverse direction** (engine findings the semantic pass judged acceptable — candidate engine false positives, which the ticket asks to route back to quire-rs): `ST-20` (`ears:non-singular`) was flagged by 0/20 semantic runs; `ST-23` (`ears:vague-response`) by 1/20. Neither engine finding was *contradicted* by a run either — the prompt asked for additional semantic findings, not for review of the engine's, so **this measurement cannot distinguish "the semantic pass agreed" from "the semantic pass was not asked". The engine-false-positive rate is not measured here.**

## M1 — `spec-review` Vague Criteria (PLAT-837)

N = **12**, not 20. Stated plainly: the concurrency cap on this session is 20 subagents and the gap-analysis and EARS batches had priority for the full N; 12 is what ran. A smaller honest N is reported rather than a claimed 20.

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

Measured from the harness's own per-subagent accounting over this corpus. Runs were executed concurrently in batches, so **wall-clock here is per-run duration under concurrency, not the latency of a single isolated pass**; treat it as an order-of-magnitude figure.

| Pass | N | Wall-clock median (min–max) | Tokens/run median (min–max) | Tokens total |
|---|---:|---|---|---:|
| `gap-analysis` step 5 | 20 | 119.1 s (91.1–144.4) | 73,209 (70,537–77,442) | 1,471,906 |
| `spec-ears-analysis` step 2 | 20 | 80.7 s (59.6–97.5) | 60,587 (49,311–65,157) | 1,218,260 |
| `spec-review` vague criteria | 12 | 40.2 s (28.2–47.0) | 53,439 (52,311–54,036) | 480,886 |

**The dollar figure is not reported, because it was not measured.** The harness reports one total token count per subagent and does not expose the input / output / cache-read split, and those price differently. Anyone with a rate card can apply it to the token column above; inventing the split to produce a dollar number would be exactly the thing this report exists to stop. The *input-token* figure PLAT-839's M5 asks for is therefore also **not measured**.

Note the scale against PLAT-839's own estimate. That ticket projects *"roughly 300 triples for a ~100-FR repo"* at *"about $0.03 per full audit"*. This corpus is 3 FRs, and one pass over it cost ~73k tokens. Whatever the Jev figure turns out to be, **the baseline it must be compared against is an LLM pass whose cost scales with the whole repository, measured here only at the 3-FR end.** Extrapolating this to 100 FRs was not attempted and is not offered.

## M8 — the historical skip rate (PLAT-839)

PLAT-839 claims *"the most valuable check stopped being the one that gets skipped."* **This is measurable from existing run history, and was measured.**

Population: every Markdown document under `/home/peter/dev` whose frontmatter carries `type: SpecReview` and `analysis: gap-analysis`, **deduplicated by content hash** (the raw file count is 3,930 — inflated roughly eightfold by sibling worktree checkouts holding identical copies; deduplicating gives 483 distinct documents). The skill requires a run that skips the semantic review to say so in `## Coverage`, which is what makes this countable at all.

Classification is by regex over the lines mentioning the semantic review, so it is a **derived** figure, not a read of 483 documents. Two classifiers are reported because a spot-check of the first found it under-counting skips (a document saying *"The optional gap-analysis semantic mode was not invoked"* has no line reading "semantic review"):

| Classifier | Docs | skipped | ran | ambiguous | not stated | Skip rate over docs that state it |
|---|---:|---:|---:|---:|---:|---:|
| Primary — lines matching `semantic review` | 483 | 151 | 42 | 9 | 281 | **78.2%** |
| Broader — `semantic (review\|pass\|mode)` | 469 | 176 | 43 | 8 | 242 | **80.4%** |

The largest single contributor is `filament-ide-rs` at 69 of 483 (14%). Recomputed without it the rate is **78.4%** and **80.9%** respectively — the finding does not depend on any one repository. (The broader classifier saw 14 fewer files: other agents were working in these trees and those paths disappeared between the two sweeps. Named rather than quietly dropped.)

**Roughly four out of five gap-analysis runs that recorded the decision skipped the semantic review.** The 242–281 documents that state nothing are genuinely unknown; the skill's own requirement means an unrecorded run is more likely a skipped one than a run one, but **that is a hypothesis about author behaviour, not a measurement, and those documents are excluded from the rate rather than assigned.** The floor that does not depend on it: at least 151 of 483 distinct audits (31.3%) explicitly skipped, and at most 42 (8.7%) explicitly ran.

PLAT-839's premise is supported by the record.

## What could not be measured, and why

| Metric | Status | Why |
|---|---|---|
| M2 — precision / recall / false-positive rate | **Not measured** | Requires a human-labelled ground-truth fixture set with known-correct answers. None exists for these passes, and building one was outside this task. This is the metric PLAT-837 says decides *"whether the lens ships at all"*, and it remains unanswered for the baseline as well as for Jev. |
| M3 — expected calibration error | **Not measurable for the baseline, by construction** | The current passes emit no probability. ECE is undefined over a lens that states no confidence. This is not a gap in the measurement; it is the asymmetry PLAT-837 is pointing at, and it means M3 will have a post-change number with no before-number to compare it to. Say so when M3 is first published. |
| M4 — abstention rate and precision | **Not measured** | The current passes have no abstention mechanism — a run either raises a finding or does not. There is nothing to count. |
| M7 (PLAT-839) — `assertion_vacuous` vs a mutation run | **Not measured** | Requires running mutations; PLAT-839 places that out of scope for itself. |
| M7 (PLAT-837) — adverse-case coverage distribution | **Not measured** | The `adverse_case_coverage` rubric does not exist yet; there is nothing today that produces the score. |
| M5 dollar cost, and input tokens specifically | **Not measured** | The harness exposes one total token count per subagent, not the input/output/cache split. Token totals are reported instead. |
| EARS engine false-positive rate (PLAT-838 inverse) | **Not measured** | The runs were asked for additional semantic findings, not for adjudication of the engine's two. Silence about them is not agreement with them. |
| Verdict stability away from a unanimous `high` finding | **Not measured** | Needs a second corpus near the PASS/CONDITIONAL boundary. |

## What these numbers change

- **PLAT-839.** The premise holds: the semantic review is skipped in about four of five recorded audits, and when it runs, 43.5% of its findings differ between two runs over identical bytes. The **Verdict** was stable at 0/20 here, but only because one `high` finding was unanimous, so a gap-analysis PASS is not yet demonstrated to be reproducible enough to serve as a merge gate. **The before-number now exists and is durable.**
- **PLAT-838.** The justifying delta is **19.0% mean (9.5%–42.9%)** of engine-clean statements, not near zero, so the lens is not redundant against the grammar engine. But zero statements were flagged unanimously, and the `ears_pattern` label itself moved on 16.7% of pairs. Against PLAT-838's own pre-registration requirement for M7 gate-readiness, **the disagreement bound must be written into the MeasurementPlan before the post-change number is taken**, and 28.7% is the figure it has to beat. Do not turn on `quire validate --strict` against anything that does not.
- **PLAT-837.** The by-eye Vague Criteria check flags 3.7 of 21 criteria per run, agrees with itself on 83.0% of judgments, and reaches unanimity on exactly one criterion whose severity it still cannot fix. There is effectively no criterion-strength baseline to beat — which makes the *stability* numbers, not the finding counts, the thing to compare against.

## Files

- `2026-09-20-jev-lens-baselines.json` — every rollup in this report, plus the metrics that could not be measured with their reasons.
- `2026-09-20-jev-lens-baselines.tsv` — the full per-run, per-item judgment record (912 rows): 20 gap-analysis runs by defect class and severity, 20 EARS runs by statement with both the finding and the `ears_pattern` label, 12 spec-review runs by criterion. Every percentage above is recomputable from it.
