---
id: SR-057
title: "Reconstructed: the second review pass CR-132 cites and nobody wrote (#352)"
type: SpecReview
analysis: code-review
scope: "spec/log.md CR-132 and CR-133, and the eight commits citing SR-055 FND-001..FND-013"
review_set: subset
---

## Summary

**This document is a reconstruction, not a review.** No findings were made
here. It exists because `spec/log.md` CR-132 and CR-133, and eight commits
across `quire-rs` and `qa-corpus`, cite `SR-055 FND-001` through `FND-013` — and
the SR-055 that exists, `reviews/2026-08-24-phase2-code-review.md`, has **twelve
findings, none of which are those**.

A second review pass was genuinely performed. Its findings are real, they were
all fixed, and they are described in full prose inside CR-132 itself. What
never existed is the numbered artifact those keys resolve against. So every
`SR-055 FND-*` citation in the log and in eight commit messages currently points
at a document that says something else.

This restores the referent for the ten keys that are recoverable and records the
three that are not.

## Verdict

**WITHDRAWN, IN PART.** Ten citation keys are re-pointed here as
`SR-057 FND-*`. Three — `FND-006`, `FND-010`, `FND-013` — are **not
reconstructable and are deliberately not guessed**. Inventing a mapping to close
a numbering gap would be the fabrication this programme has now spent three
rounds retracting; a missing referent that says it is missing is strictly better
than a plausible wrong one.

**No code changes.** Every finding below was fixed at the time, by the commit
named in its row. This document changes only what the citations resolve to.

## Method

Reconstructed from `spec/log.md` CR-132's prose and from `git log` subjects over
`quire-rs 049b840..816e187` and `qa-corpus 3e73db2..2bc486d`. A row appears here
only where the commit subject or CR-132's own text states the finding
unambiguously. Nothing was inferred from a diff alone.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|----|----------|---------|------|--------------|
| FND-001 | high | The AC-42 parity gate existed in one of the two readers: the Rust harness graded the differential, `verify.py` ran each case against its own payload and cross-graded nothing | `qa-corpus` `scripts/parity_selftest.py`, #337 | correct-requirement-no-evidence |
| FND-002 | high | FR-065-AC-47 existed in one of the two readers | `b24535c` | correct-requirement-no-evidence |
| FND-003 | medium | The `validate_*` control-tree rule has reach 0 over the real corpus, and the motivation it cited could not reach it | `84ffa30` | correct-requirement-no-evidence |
| FND-004 | medium | `Level::ALL` carries a hand-written length that nothing ties to the variant set; the doc comment claimed it was compile-enforced | `tests/corpus_case/mod.rs`, CR-129 | implementation-bug-despite-evidence |
| FND-005 | medium | A published count was missing its unit — "the 14" named neither pairs nor cases | `8722732` | correct-requirement-no-evidence |
| FND-007 | medium | "All 37 controlled failure blocks" is 37 of 41 | `8722732` | correct-requirement-no-evidence |
| FND-008 | medium | A guard with reach 0 | `a813ca0` | correct-requirement-no-evidence |
| FND-009 | medium | An AC-46 `continue` skipped a rule this reader alone has | `a813ca0` | implementation-bug-despite-evidence |
| FND-011 | low | The README's fixture-author section taught AC-42 and never mentioned the witness channel | `18aa35c` | correct-requirement-no-evidence |
| FND-012 | low | CR-127's claim superseded by CR-129 | `spec/log.md`, CR-129 | correct-requirement-no-evidence |


## The three that are not here

`FND-006`, `FND-010` and `FND-013` are **unrecoverable and deliberately not
guessed**. They are absent from the table above rather than recorded as empty
rows, because the `SpecReview` schema's `id` and `Severity` vocabularies
correctly refuse a row that records an absence — a finding table is for
findings, and a gap in a numbering is prose.

`278cfc5`'s subject says "the review's **thirteen** findings" against a
twelve-finding SR-055, which is how the whole discrepancy was first noticed.

## What this does not fix

The eight commit messages and the two log entries still read `SR-055 FND-*`.
Commit messages are immutable and rewriting merged history to correct a citation
would cost more than it buys. **The correct reading, stated once here and once in
CR-132: any `SR-055 FND-*` citation in `quire-rs` or `qa-corpus` history means
`SR-057 FND-*` in this document, and `SR-055` as published is a different review
with a different finding set.**

## Why it happened, so it does not again

A second review pass ran concurrently in another session, fixed all thirteen of
its findings, and never wrote the artifact its numbering indexed. The commits
carried keys; the document did not exist. Nothing checked that a cited review id
resolves to a review with that finding — which is the same class as the four
CR-133 names in one evening: `result_record` declared a record no runner emits,
`variant_forbidden` a rule no reader read, `check_spec_structure.sh` declared to
be in `make ci` and not, and this.

**A citation is not a gate.** The recurring lesson of this campaign is that a
claim about a thing has to be checked by executing the thing, and a finding id is
a claim about a document.
