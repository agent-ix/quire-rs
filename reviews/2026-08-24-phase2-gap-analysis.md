---
id: SR-056
title: "Gap analysis — EPIC #264 session 2: unstated requirements behind the phase's findings"
type: SpecReview
analysis: gap-analysis
scope: "FR-050, FR-063, FR-065, spec/tests.md, qa-corpus corpus.yaml"
review_set: subset
---

## Summary

Discovery pass over the same range as SR-055, asking what requirement *should*
have existed such that each finding could not have shipped. Four unstated
requirements, three of them mechanically checkable today. The unifying shape:
**this codebase reliably detects a declaration that is wrong and reliably fails
to detect a declaration that is inert.**

## Verdict

**FAIL** — GAP-001 and GAP-002 each describe a defect class with three
independent instances already on the record, and no requirement covering the
class.

## Findings

| ID      | Severity | Summary                                                                        | Refs                                     | Escape Cause        |
| ------- | -------- | ------------------------------------------------------------------------------ | ---------------------------------------- | ------------------- |
| FND-001 | high     | No requirement that a declared key have a consumer; three instances so far      | FR-065, qa-corpus `corpus.yaml`          | missing-requirement |
| FND-002 | high     | A declared selector matching nothing is a silent skip; FR-063's rule was never generalised past metrics | FR-063, `src/coverage.rs:1229`           | missing-requirement |
| FND-003 | medium   | No requirement that a rule widened in response to a failing case record that case | FR-065-AC-46                             | missing-requirement |
| FND-004 | medium   | `spec/tests.md` carries two hand-maintained status projections over one fact, with no cross-check | `spec/tests.md:30,41,61,148`             | missing-requirement |
| FND-005 | low      | Plan scope narrowed from five structural gates to two with no ticket recording the other three | `scripts/audits/check_spec_structure.sh` | missing-requirement |

## Discovery

### FND-001 — the declaration nobody reads

| instance | declared | consumers | how it was found |
|---|---|---|---|
| `result_record` | `corpus.yaml`, 9 fields | 0 | outside review, #279 |
| `variant_forbidden` | `corpus.yaml:171`, 5 entries | 0 | this review, SR-055 FND-001 |
| `case_schema.by_kind.forbidden` / `.values` | `corpus.yaml` | 1 reader, 0 mutations | this review, SR-055 FND-005 |

Three instances, two of them in the file whose whole purpose is to be the
single declaration two readers are held to. `result_record` survived long
enough for FR-065-AC-17 to be marked implemented over a subject that did not
exist.

**Unstated requirement.** *Every key `corpus.yaml` declares is read by at least
one reader, and the check that says so is itself mutation-verified.* This is
mechanically decidable — a key set differenced against a symbol search across
`bounds.py`, `verify.py` and `tests/corpus_case/` — and it is strictly cheaper
than the reviews that found the first two by hand.

The obvious weaker form (*"a key is read"* by grep) is not enough: a key can be
read into a variable and never consulted. The check worth having pairs the
inventory with the existing mutation discipline — a declared key whose mutation
changes no gate's verdict is inert whatever the grep says.

### FND-002 — a selector that matches nothing

FR-063 states the principle already, for metrics:

> An empty list and an uncomputed one serialize identically — the key is absent
> either way — so a consumer could not tell *"the engine looked and found none"*
> from *"the engine never looked"*.

The same distinction is unmade for **selectors**, and each time it has cost a
release:

| selector | matches nothing → | detected? |
|---|---|---|
| `section:` | table stranded, smaller `total`, plausible % | **yes**, since #272/#320 — `section-matches-nothing` |
| `id_column:` | row minted with null identity | partly — #318 open |
| `status.column` | status-lie check silently skipped | **no** |

SR-055 FND-002 measures the third: renaming three header cells in
`quire-rs/spec/tests.md` takes `status_lies` from 0 to 6. The canonical quoin
matrix template ships the non-matching column name, and 206 files under the dev
root carry it, so the status-lie check has never fired on a stakeholder,
user-story or functional-coverage matrix anywhere.

**Unstated requirement.** *A declared column or section that selects a document
but matches nothing in it is reported, and the check it gates is never silently
skipped.* Written once at the level of "declared selector", not a third time as
a special case for `status`.

This is the highest-value item in the analysis: it converts an entire class from
undetectable to detected, it has a fixture-shaped answer in the controlled
corpus (`minting/section-name-mismatch` is the same defect one column over), and
the census it would produce is measurable across all 206 files.

### FND-003 — the rule that moves when a case fails it

`witness_channels` is authored in the same file, by the same hand, as the cases
it grades. AC-47 constrains it to *restrictable* channel names — a type check.
Nothing constrains a mode's channel set to channels that are semantically about
that mode.

Measured (SR-055 FND-006): 5 of 35 pairs discriminate **only** through channels
added to `witness_channels` after those pairs failed. The additions were
justified from the fixtures' own comments and the justification holds. But the
procedure that produced them — widen the rule until the corpus passes — is
indistinguishable from the one this repository's `CLAUDE.md` forbids, and the
difference lives only in a commit message.

**Unstated requirement.** *A rule widened in response to a failing case records
the case that forced the widening, in the declaration, and a standing check
reports when the set of cases depending on that widening changes.* The
measurement already exists as a script; making it a gate makes the next
widening visible as a diff rather than as a judgement nobody re-reads.

### FND-004 — two projections, one fact

`spec/tests.md` (1,698 lines) states each test's state twice: once per TC in the
ledger's `Status` column, once per FR in the coverage table's `Coverage Status`.
Nothing compares them. **22 rows disagree**, and the error runs both ways —
`TC-636` carries a live `#[trace]` in `src/registry.rs` while its ledger row
reads 🚧, so there the ledger is stale, not the summary.

This is Wave 5's thesis, now with a number attached. It does not need the
migration Wave 5 defers; it needs the comparison.

**Unstated requirement.** *A document that projects one fact into two tables
derives the second, or a gate compares them.* The cheap form is a script in
`audit-static`; the durable form is FR-058's upward-trace machinery applied
within a document rather than across the tree.

### FND-005 — scope narrowed, unrecorded

Wave 1.2 specified five engine-independent gates. Two shipped. CR-124 says so
plainly, so nothing is over-claimed — but the three that did not ship
(duplicate artifact ids, unresolved local links, frontmatter schema validation)
exist in no ticket, so the only record that they were ever intended is a plan
file in a scratch directory.

**Unstated requirement.** *Deferring a planned gate files the ticket that
carries it.* This is the ticket-policy rule the epic already runs on, applied to
scope removed rather than scope added.

## Formalization

Proposed, in the order that pays:

1. **FR-050 or FR-063 amendment** — a declared selector matching nothing is
   reported (FND-002). One requirement, one diagnostic reason,
   `status-column-matches-nothing`, and a controlled-corpus pair in
   `minting/`. Governs the 206-file census.
2. **FR-065 amendment** — every declared key has a mutation-verified consumer
   (FND-001), closing the class that produced `result_record` and
   `variant_forbidden`.
3. **A `tests.md` cross-check** in `audit-static` (FND-004), read-only, no
   migration.
4. **A witness-dependency gate** in `qa-corpus`'s `make ci` (FND-003), pinning
   the 5-pair dependency measured here.

## Note on method

Every count in this document was produced by running something, and the command
is in SR-055's Evidence section for each. Two figures that a first pass got
wrong and re-measurement corrected: the initial mutation probe was run against
`quire-cli`, whose matrices already use `Status` and whose one unbacked row is
correctly marked ⛔, and reported no change — the finding only reproduces on a
repository that carries the template's column name.
