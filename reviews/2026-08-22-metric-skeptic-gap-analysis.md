---
id: SR-054
title: "Gap analysis — the metric-integrity and skeptic work (quoin#197)"
type: SpecReview
analysis: gap-analysis
scope: "spec/functional/FR-050, FR-051, FR-052, FR-055, FR-063, FR-064; src/metric.rs, src/skeptic.rs, src/coverage.rs; tests/, bench/"
review_set: subset
---

# SR-054: Gap analysis — the metric-integrity and skeptic work (quoin#197)

## Summary

Verified the engine half of `agent-ix/quoin#197` by running `quire coverage`
against four repositories rather than against this one. Both new checks the
programme shipped — the vacuity detector and the `hollow-denominator`
diagnostic — produce **false positives on corpora outside this crate**, and
neither was measured outside it. The matrix itself is clean: 0 status lies, 0
untracked tests, 929 of 1276 rows backed.

## Verdict

**FAIL** — two `high` findings. Both are live in `quire-rs v0.44.0`, both are
in checks whose stated purpose is to make numbers trustworthy, and both were
invisible because the only corpus either was measured against is this crate.

## Findings

| ID      | Severity | Summary                                                                 | Refs                            |
| ------- | -------- | ----------------------------------------------------------------------- | ------------------------------- |
| FND-001 | high     | The vacuity detector's `"=> {"` guard matches every TypeScript arrow function, producing 549 suspicions from 551 candidates on `quoin` | src/skeptic.rs:90               |
| FND-002 | high     | `hollow-denominator` fires on an honest zero for count-shaped metrics; FR-063-AC-1 encodes the defect, so no test can catch it | src/metric.rs (`is_hollow`)     |
| FND-003 | medium   | The whole FR-064 suite is Rust-only — `extract()` hardcodes `SourceLanguage::Rust` — for a check that runs on every bound language | src/skeptic.rs:278              |
| FND-004 | medium   | FR-064's justification for deleting the `no-assertion` class is Rust-specific ("a test fails on panic") but the deletion is global | spec/functional/FR-064-skeptic-layer.md |
| FND-005 | low      | The `make bench` ratchet measures one repository, so a check that is correct on this crate and wrong elsewhere ratchets green | bench/manifest.json             |

## Detail

### FND-001 — the vacuity detector on TypeScript (high)

`NARROWING_GUARDS` is `["if let ", "while let ", "=> {"]`. The third entry is
meant to be a **`match` arm**. In TypeScript `=> {` opens an **arrow
function**, and every `vitest` test body is `() => {`.

Measured against `agent-ix/quoin`:

```
suspicions            549
typescript candidates 551
```

Sampled three, all rule, none real:

```
tests/advise-command.test.ts:62  2 of 2 assertions guarded, 0 unguarded
tests/advise-command.test.ts:77  3 of 3 assertions guarded, 0 unguarded
tests/advise-command.test.ts:103 1 of 1 assertions guarded, 0 unguarded
```

Each is an ordinary `it("…", async () => { … })` with unconditional
`expect()` calls. Rust is unaffected — 2 suspicions from 883 candidates, both
true positives.

**Failure scenario.** Any TypeScript consumer running `quire coverage` sees
essentially every test flagged. Per FR-064's own reasoning, *"a check that can
fail somebody's build over a heuristic about their assertions will be switched
off within a week"* — a check that flags 99.6% of a corpus is switched off on
day one, at which point it detects nothing at all, including the Rust cases
where it works.

This is the failure `CLAUDE.md` names directly. The rule was measured against
this crate's 921 evidence symbols and never against a TypeScript corpus, so
the census that would have settled bad-rule-versus-bad-corpus was never taken
on the corpus where the rule is bad.

**Fix.** Make the guard vocabulary per-language: `=> {` is a match arm in Rust
and must not be a guard in TypeScript. Add a TypeScript case to TC-997.

### FND-002 — `hollow-denominator` on an honest zero (high)

`is_hollow()` is true when `population > 0 && examined > 0 && matched == 0`.
That is the right test for a **ratio**. It is wrong for a **count**, where the
value *is* the match count and zero is the answer rather than a failure to
read.

`coverage.implements` is count-shaped, and its own `method` string says so:

> *"`matched` and the value coincide because every bound symbol is a matched
> one, and `population` is the production symbols examined"*

Measured on `spec-artifacts-process`:

```
coverage.implements  measured  value 0  population 42  examined 42  matched 0
→ diagnostic: hollow-denominator
```

Nothing is hollow. All 42 production symbols were read; none carries an
`Implements:` marker. The honest report is *"0 of 42"*.

**Failure scenario.** Any repository that declares `implements` marker forms
and has not yet annotated production code gets a spurious alarm on the single
diagnostic this programme added to mean *"this number is arithmetic over
nothing"*. `spec-artifacts-process` is that repository today, at
`v0.24.0`.

**FR-063-AC-1 encodes the defect.** It states hollowness as exactly
`population != 0 && examined != 0 && matched == 0`, with no exception for
count-shaped metrics. TC-985 tests precisely that, using `coverage.backed` — a
ratio — for all four of its cases, and passes. The test validates the
criterion faithfully; the criterion is wrong. No test in the suite could have
caught this, which is why the code review did not.

**Coupled fix — do not fix this one alone.** `quoin`'s FR-043-AC-6
silent-zero sentinel gates on *"`matched = 0` over a non-zero population **and
no accompanying diagnostic**"*. Today the spurious `hollow-denominator`
satisfies the "accompanying diagnostic" clause, so the sentinel stays green by
accident. Removing the false positive here makes the sentinel fire on the same
honest zero unless AC-6 is corrected in the same change.

**Fix.** Give `Metric` an explicit shape (`Ratio` / `Count`) and apply
`is_hollow` only to ratios, amending FR-063-AC-1 by CR note. Correct
`quoin` FR-043-AC-6 in the same release.

### FND-003 — the skeptic suite tests one language (medium)

```rust
fn extract(body: &str) -> SymbolExtraction {
    extract_file("src/lib.rs", SourceLanguage::Rust, body)
}
```

Every FR-064 test routes through this helper. The strings `typescript`,
`python` and `.ts` appear **zero** times in `src/skeptic.rs`. The check runs
wherever the binder binds, which is Rust, TypeScript and Python.

FND-001 is the consequence, and would have been caught at authoring time by a
single TypeScript case.

**Fix.** Parameterise TC-997/998 over the bound languages.

### FND-004 — a Rust-specific rationale for a global deletion (medium)

FR-064 removes the `no-assertion` suspicion class, justified as:

> *"In Rust a test fails on panic, so absence of an assertion macro is not
> absence of an oracle."*

True in Rust. **False in TypeScript**: a `vitest` test whose body contains no
`expect()` passes silently, which is exactly the shape the class existed to
catch. The measurement behind the deletion — 12 of 12 sampled were rule, 0
real — was taken on this crate's Rust corpus only.

The class was rightly deleted for Rust. The spec states the reason as though
it were language-independent, and the code deletes it for every language.

**Fix.** Record the scope in FR-064 and re-open the question for TypeScript as
its own ticket rather than leaving the rationale reading as universal.

### FND-005 — the ratchet's corpus is one repository (low)

`bench/manifest.json` scores against `self`. A check correct on this crate and
wrong on every other ratchets green: `coverage.implements` is 15 of 1412 here,
so FND-002 cannot trip it, and there is no TypeScript corpus at all, so
FND-001 cannot either. `scripts/overfit_check.py` exists to catch fitting to
the corpus and cannot see past a corpus of one.

**Fix.** Add a second repository to the manifest — `quoin` covers TypeScript
and `spec-artifacts-process` covers the count-shaped-metric case, and each
would have failed today.

## Coverage

Matrix verification ran `quire coverage --scope <root> --module
spec-artifacts-process --json` per repository, with the `v0.44.0` engine:

| Repository | Backed | Status lies | Untracked | Suspicions | Candidates bound |
| ---------- | ------ | ----------- | --------- | ---------- | ---------------- |
| `quire-rs` | 929 / 1276 | 0 | 0 | 2 | rust 591/883, python 18/77 |
| `quire-cli` | 252 / 293 | 0 | 0 | 0 | rust 145/164 |
| `spec-artifacts-process` | 56 / 128 | 0 | 0 | 0 | python 72/79 |
| `quoin` | 298 / 671 | 10 | 1 | 549 | typescript 374/551 |

`quoin`'s rows are reported in that repository's SR-015.

**Plan completion** — `agent-ix/quoin#197` was executed ticket-first with no
`plan/` bundle, so step 1 resolves against the 34 board-18 items, all `Done`.

**Semantic review** — run, at the user's request, over FR-050, FR-051,
FR-052, FR-055, FR-063 and FR-064. FND-002 and FND-004 are its findings:
both are cases where the test faithfully validates a criterion that is itself
wrong, which is the class a code review cannot reach. The `criteria` and
`specific_shaped` metrics report `not_computed` in the runs above because the
invocation loaded only the `spec-artifacts-process` module, not `-iso`; that
is an artifact of the measurement, not of the corpora.

## Note on the reviewer

Both `high` findings are against code written in the same session that is now
reviewing it, and both are the same mistake: a rule measured on one corpus and
shipped to all of them. That is the mistake `agent-ix/quoin#197` was opened to
stop, committed by `agent-ix/quoin#197`.
