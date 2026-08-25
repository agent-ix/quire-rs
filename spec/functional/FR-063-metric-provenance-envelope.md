---
id: FR-063
title: "The metric provenance envelope"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/FR-050"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/FR-055"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/StR-001"
    type: "traces_to"
---

# FR-063: The metric provenance envelope

## Description

Every number this engine emits shall state what it counts, over what population, by what method, and
how much of its input the measurement actually read.

### The number that started it

`quire coverage` reported this over `agent-ix/filament-ide-rs`:

```
Coverage: 555/2389 rows backed (23%)
```

**Correct arithmetic. Meaningless number.** The declared tag patterns matched **0 of 1,292** Rust
evidence symbols in that repository, so the ratio was computed over a corpus the binder could not
read. Nothing in the payload distinguished that from a repository with 1,766 missing tests — the two
render identically — and three published SpecReviews (SR-150, SR-151, SR-152) were built on the
figure.

`FR-050-AC-27` closed that specific case by naming the one premise the backed/total ratio happens to
rest on. This requirement generalizes it: **stating the premise becomes a property of every ratio the
engine publishes**, so the hollow shape is structurally visible rather than something a reader has to
already suspect and then go looking for.

### What an envelope carries

| field | answers |
|---|---|
| `name` | which number this is — a stable dotted token a consumer joins on |
| `unit` | what **one** of `value` is: a matrix row, an acceptance criterion, a production symbol |
| `method` | how it was arrived at, and what `matched` counts for it |
| `value` / `population` | the numerator and denominator |
| `examined` / `matched` | how much input was offered, and how much of it was read |

`examined` and `matched` are the pair that does the work, and they are a pair on purpose:

- `matched` 0 of `examined` 0 — there was nothing to read. A repository with no tests reports 0%
  honestly.
- `matched` 0 of `examined` 1,292 — the measurement was handed its input and could not read any of
  it. That is the `filament-ide-rs` shape, and it is a defect in the measurement, not in the corpus.

A single "did it match anything" flag cannot tell those apart, which is why the count is carried
rather than a boolean.

### Not computed is a value, not a zero

An empty list and an uncomputed one serialize identically — the key is absent either way — so a
consumer could not tell *"the engine looked and found none"* from *"the engine never looked"*
(`agent-ix/quire-rs#226`). `no_symbol_rows` is absent both when the reconciliation found no exempt
row and when the module declares no `no_source_symbol` vocabulary so no row could ever be exempt.
`implements` is absent both when no production symbol carries a marker and when the module declares
no `implements` forms at all.

**This does not belong in the warning stream.** Two diagnostics for it were written and run against
the suite before being rejected: both fired on healthy fixtures, because not adopting an optional
declaration is not a defect. It is a fact about *what was asked*, so the envelope carries it as a
state of the measurement — with no numbers at all attached, so there is no zero to be mistaken for an
answer.

### A hollow denominator fails loudly

A metric whose measurement was offered input, read none of it, and published a ratio anyway is
reported as a diagnostic. Deliberately the absolute case only: `matched` low-but-non-zero is a
judgement — the tail of a migration looks the same from here — and is reported with both counts where
the engine has the context to say so (`FR-050-AC-27`), never collapsed to a boolean.

`FR-050-AC-27`'s `no-symbol-bound` diagnostic **stays**. It knows it is about the trace binder and can
name the declared forms to check, and a finding that says what to look at beats one that says a number
is wrong. This one is the backstop for every metric, including ones added later with no bespoke check
of their own — which is what makes it a schema invariant rather than another special case.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-063-AC-1 | A metric carries a name, a unit, a method and a **shape**, and there is no constructor that can omit them. A **ratio** is hollow when its population is non-zero, its `examined` count is non-zero, and `matched` is zero; a zero `examined`, a zero `population`, and a low-but-non-zero `matched` are each not hollow. | Test (TC-985, TC-987) |
| FR-063-AC-6 | A **count** is never hollow: `matched` and `value` are the same fact for a tally, so `matched: 0` over a non-zero examined reports that none was found, not that none was read. The identical numbers carried as a ratio are still hollow, so the shape decides and not the arithmetic. | Test (TC-985) |
| FR-063-AC-2 | A measurement that did not run serializes with a distinct state and a stated reason, carries no `value`, `population`, `examined` or `matched`, is unequal to the same metric measured as zero, and round-trips through the contract. | Test (TC-986) |
| FR-063-AC-3 | The coverage payload carries one enveloped metric for every headline number it emits, each with its unit, method, population, `examined` and `matched` counts. | Test (TC-988) |
| FR-063-AC-4 | `coverage.implements` draws its population from the production symbols examined, so the relation count is never a bare number; a module declaring no `implements` forms reports it as not computed with the condition named. | Test (TC-988) |
| FR-063-AC-5 | A metric whose measurement was offered input and read none of it is reported in `CoverageReport.diagnostics` under `hollow-denominator`, naming the metric; the same corpus read cleanly, and a corpus offering no input at all, each report nothing. | Test (TC-988) |
| FR-063-AC-7 | The payload carries `minting.section_hit_rate` — the documents whose declared minting section was found, over the documents a trace target's `archetype:` selected and its `exclude:` kept — as a **ratio**, so a corpus none of whose minting sections were read is hollow and reported by name, while a partial read is not. A model declaring no `trace_targets` reports it as not computed with the condition named, never as zero. | Test (TC-1036) |

> **CR-117 note (2026-08-24):** AC-7 is new — `minting.section_hit_rate`.
> `agent-ix/quire-rs#270`; the metric half of the FR-050-AC-33 note.
>
> Every existing metric describes what happened to the rows the engine
> **found**. None of them describes whether the rows were found at all, and
> that is the premise 88 of 239 repositories fail: the archetype matches, the
> declared heading does not, and those TC ids never enter `coverage.backed`'s
> denominator — a **candidate census** of 3,514, which CR-118 later showed is
> not the causal size of the section defect (the section fix realised +83 rows;
> the population is confounded with id-column mismatch, `agent-ix/quire-rs#318`).
> Either way the premise fails silently, which is what this envelope is for.
> The repository reports a smaller `total` and a plausible
> percentage — the exact shape the binding census (FR-050-AC-27) was built for
> one layer down.
>
> **A ratio, and the shape is the decision.** `examined` and `matched` are the
> same pair of counts as `population` and `value` here, because the
> measurement's input **is** its population: one document offered to one
> minting declaration. That makes `is_hollow` fire exactly when no declaration
> found any of its sections — a ratio published over a corpus none of whose
> minting tables were read — and stay quiet on both honest zeroes: a bundle
> with no documents of any minting archetype examines nothing, and a partial
> read is a judgement this metric does not make (it is reported per document,
> by name, under `section-matches-nothing`). Carrying it as a `Count` would
> have made the hollow case unreportable, which is the CR-102 mistake with the
> sign flipped.

> **CR-102 note (2026-08-22):** `agent-ix/quire-rs#229`, reopened. The
> hollowness rule shipped in `v0.44.0` fires on an **honest zero** (SR-054).
>
> **`is_hollow` is a ratio test, and it was applied to counts.** For
> `coverage.implements` the value *is* the match count — the metric's own
> `method` string said so — so `matched: 0` means no production symbol carries
> an `Implements:` marker, which is the answer rather than a failure to read.
> Measured on `agent-ix/spec-artifacts-process` at `v0.24.0`: **0 of 42
> production symbols, reported as arithmetic over nothing.** `quire-cli` reads
> 0 of 214 and was the same. This crate reads 15 of 1,412, which is why a
> ratchet whose corpus is one repository never saw it.
>
> **AC-1 encoded the defect, so no test could catch it.** It stated hollowness
> as `population != 0 && examined != 0 && matched == 0` with no exception, and
> TC-985 tested exactly that using `coverage.backed` — a ratio — for all four
> cases. The criterion was wrong, not the test. `shape` is now a field rather
> than a convention, because a rule that depends on which kind of number this is
> cannot read that from the prose in `method`.
>
> **Coupled with `agent-ix/quoin#198`.** FR-043-AC-6's silent-zero sentinel
> excuses a metric with `matched = 0` when a diagnostic accompanies it, and the
> false positive here *was* that diagnostic. Removing it alone would make the
> sentinel fire on the same honest zero, so both ship together.
>
> **The `hollow-denominator` message was also rebuilt.** The template said "a
> ratio over a population of {}" while the substitution carried its own clause,
> and `(s)` was pluralising a computed count by hand. One template now, and the
> dead `NotComputed` branch is gone — `is_hollow` is false for every uncomputed
> metric, so rendering half a sentence for it was unreachable code.
>
> **CR-094 note (2026-08-22):** FR-063 is new. `agent-ix/quire-rs#229`, folding
> `agent-ix/quire-rs#226`; epic `agent-ix/quoin#197`.
>
> **`examined` was not in the original design and the tests found that.** The
> envelope began as `{value, population, matched}`, and the first full run put a
> `hollow-denominator` finding on `tc788_no_criteria_corpus_is_unchanged` — a
> fixture whose source tree is a single comment line. Zero symbols walked, zero
> bound, and the check called a genuine 0% "arithmetic over nothing". Without
> the offered-input count the invariant fires on every greenfield corpus, which
> would have made it worthless within a release. The pair is the requirement;
> `matched` alone is not.
>
> **`state` is a tagged enum rather than nullable numbers.** "Not computed" and
> "computed zero" are different facts, and a shape that can express only one of
> them is the defect this envelope exists to close. Three nullable integers
> would have re-created it one field down.
>
> **Two rejected diagnostics, recorded so they are not re-attempted.** An
> `implements-undeclared` warning fired on `tests/fixtures/coverage_baseline`,
> whose own test asserts `diagnostics.is_empty()` under the message *"the
> baseline model is healthy"*. Narrowing it to `no-source-symbol-undeclared`,
> gated on there also being unbacked rows the absence could explain, still fired
> on the `iso` fixture in two tests. Both were correct to fail: adding a check
> that fires on healthy input to close a ticket is the failure this repository's
> `CLAUDE.md` names directly, and it would have drowned the CR-093 signal
> shipped one release earlier.
>
> **Scope: the coverage payload only.** `#229` names the properties and validate
> payloads too. The `properties --json` envelope is assembled by `quire-cli`
> (see `schemas/output/properties-v1.schema.json`, whose own description records
> this), and `validate`'s human and JSON surfaces are the CLI's; both adopt this
> type rather than re-inventing it, tracked as `agent-ix/quire-cli#60` and
> `agent-ix/quire-rs#230`. The type is defined here because this crate owns the
> published output contract (`FR-055`).
>
> **Two assertions were tightened, not relaxed.**
> `tc788_no_criteria_corpus_is_unchanged` and
> `tc805_undeclared_vocabulary_changes_nothing` asserted an absent key by
> testing that a *substring* did not appear anywhere in the payload. The
> envelope names `coverage.property_shaped` and `coverage.no_symbol_rows` even
> when they were not computed — that naming is the entire point of the `#226`
> fold — so the substring test began answering a different question from the one
> its AC asks. Both now assert the absent **key** structurally, and additionally
> assert the metric's `not_computed` state, which is strictly more than they
> checked before.

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the payload the first metrics describe, and AC-27's binding census this generalizes), [FR-051](./FR-051-source-symbol-extraction.md) (the candidate/bound counts `coverage.backed` and `coverage.implements` draw `examined` and `matched` from), [FR-055](./FR-055-published-output-contract.md) (the published schema this shape is authored into)
- **Downstream**: `agent-ix/quire-cli` — the `properties --json` envelope and the `validate` surfaces adopt this type rather than re-inventing it (`agent-ix/quire-cli#60`); `agent-ix/quoin` — the quality benchmark scores against metrics that declare their own population (`agent-ix/quoin#198`)
