---
id: FR-065
title: "The controlled-corpus contract"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/FR-050"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/FR-051"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/FR-055"
    type: "references"
  - target: "ix://agent-ix/quire-rs/FR-063"
    type: "references"
  - target: "ix://agent-ix/quire-rs/StR-001"
    type: "traces_to"
---

# FR-065: The controlled-corpus contract

## Description

The engine's checks are gated today by cases embedded in code — mini-repositories as
strings inside one JSON blob, `include_str!`-bound at `tests/corpus_cases.rs:20`. A
sibling toolchain does the same in JavaScript, materialising template literals to a
tmpdir and returning `labels.json` as the value of a function.

Neither corpus can be read without running it, and one of them declares its own
manifest. That last fact is the load-bearing one: the generator declares
`section: Test Cases` where the ecosystem declares `Test Case Summary`, so **a corpus
whose manifest heading always matches cannot exhibit the defect accounting for 3,514
unminted TC ids across 88 repositories**. Tier 1 has never caught the dominant failure
mode because tier 1 was built where that mode cannot occur.

`quire-rs` SHALL define the contract for a **controlled corpus**: static case data on
disk, language-neutral, read in place, with declared bounds and a graded detection
ladder. The corpus artifact itself lives in its own repository and is consumed as a
submodule; what is specified here is the contract both consumers conform to.

### Why the contract lives here and the data does not

Two runners in two languages read the same cases. If each owned its own reading of what
a case means, the corpus would be two corpora that happen to share a directory — which
is the state this replaces, one layer up. The contract is specified once, in the crate
that owns the checks being gated, and the artifact is data that conforms to it.

### A mode is a failure family, and applicability is per-case

A **mode** is a named family of failure the toolchain can exhibit, not one defect and
not one check. The declared families are `minting`, `detection`, `attachment`, `parser`,
`join`, `disposition` and `provenance`; `corpus.yaml` owns the list, and adding a family
is a change to that file.

Applicability is a property of the **case**, not of the family. A triple-quote scope
desync is Python by construction; a `describe()` header is TypeScript by construction; a
section-name mismatch applies to all three. So the matrix is not *families × languages*,
and the cell count is not the product of two round numbers — it is the sum, over cases,
of the languages that case's row declares applicable. Stating it as a product is how the
first draft of this requirement arrived at a number no implementer could reproduce.

### The corpus is the bounds of the use case, not a score

A case list answers "what did we try". A **bounds matrix** answers "what did we never
try", and only the second is a statement about the tool. Every declared cell is
`covered`, `out-of-scope` with a written reason, or `GAP`.

**A scenario with no case is undefined behaviour, not assumed-working.** Today the
corpus holds 10 cases and every one is Rust; the Python and TypeScript columns are
`GAP`, and v0.44.0 shipped two `high` defects (#250, #251) straight through them.

`bounds.gap_count` is an [FR-063](./FR-063-metric-provenance-envelope.md) metric of shape
`count`, and FR-063-AC-6 already governs it: a count is never hollow, and the identical
numbers carried as a ratio would be. CON-2 below is that rule applied to this metric
rather than a new rule — a ratio falls as easy cases are added, so a corpus could improve
its number while the hard missing case stayed missing. Converting a `GAP` to
`out-of-scope` moves the count, which makes declaring something out of scope a visible
act rather than a quiet one.

A `GAP` cell is a measurement that did not run, and FR-063-AC-2's shape applies: it
carries its state and, for `out-of-scope`, its stated reason. Nothing here re-derives
that rule.

### Detection is graded, not boolean

| Level | Question | Asserted by |
|---|---|---|
| L1 detected | Did anything fire? | `diagnostic_reasons` |
| L2 localised | Did it name the right `path:line`? | `diagnostic_paths`, `binding_census.unbound_example` |
| L3 actionable | Did the message name the thing to change? | `diagnostic_message_contains` |

L1 without L2 is an alert nobody acts on, and scoring it as a pass is how a detector
earns credit for noise. A failing case SHALL report **which level was lost**, because
"the case failed" and "the message stopped naming the row" are different repairs.

**Reconciliation with the sibling dictionary, because the numbers are not interchangeable.**
`agent-ix/quoin`'s `bench/metrics.json` already publishes `actionability_rate` — findings
naming a row id **or a document line** — and `finding_localisation_rate` separately.
Quoin's "actionable" therefore spans this ladder's L2 **and** L3, which this ladder holds
apart. Both definitions stand: quoin measures **finding quality** over its own
population, this measures **detection depth** over the corpus, and #264 is explicit that
folding the two programmes together is how a fourth conflated number gets created. What
is forbidden is a **cross-runner comparison** treating them as one quantity. The 3.02%
figure (15 of 496 findings carrying a row id) that motivates splitting L2 from L3 is
quoin's, computed under quoin's merged definition; it is cited as the observation that
prompted the split and SHALL NOT be reported as this ladder's L3 rate.

### Every failure case ships its control

A detector that fires on everything scores perfect recall. That is not hypothetical
here: #250 shipped a check that produced **549 suspicions from 551 candidates** on a
TypeScript corpus, and recall alone called it excellent. A failure case without healthy
input that must stay silent measures nothing.

The pairing is declared, not inferred. Mode and language cannot identify a control's
partner, because a mode carries several failure cases — so a control names the case it
controls, by id.

### A mode is authored in every language its own row marks applicable

A case authored only in Rust is a claim about Rust. Where a case's row does not mark a
language applicable, that cell is `out-of-scope` with a reason, or it is `GAP` and the
bounds say so.

### Reproducible by hand, or it has regressed to what it replaced

A case reproduces with the invocation recorded in its own `case.yaml`, with no harness
and no generator. It runs **from the corpus root**:

```
quire coverage --scope cases/<mode>/<case>/input --module modules/ecosystem --json
```

The `--module` argument is not optional decoration: without it no traceability model
loads, the run reports `0/0 rows backed`, and the case cannot exhibit the declaration
defect it exists for. If the only way to see a case fail is to run the runner, the
corpus is code again and the contract has bought nothing.

**Why from the root and not from inside `input/`.** The first draft of this requirement
documented `cd input && … --module ../../../../modules/ecosystem`, which **the CLI
rejects**: `--module` refuses a path containing `..` under
[FR-005](./FR-005-path-safety.md) path safety, and the refusal is correct — a module
argument that can climb out of the tree it was given is the traversal that guard exists
for. Written that way, no case could bind a shared module at all, AC-16 was unreachable
by construction rather than deferred, and the vendored declaration was decorative. Found
by running it.

## Inputs

- `corpus.yaml`: schema version, the case index, the declared mode families, and the
  bounds matrix.
- A case directory `cases/<mode>/<case>/[<language>/]`: `case.yaml`, an `input/` tree of
  static files, `expect.yaml`.
- `modules/ecosystem/`: the real declaration, vendored from `spec-artifacts-process` and
  pinned by SHA. `modules/variants/<id>/`: relaxation variants.
- `labels/`, `config/metrics.json`, `baselines/{quire-rs,quoin}.json`: ground truth,
  the metric dictionary, and per-runner baselines, versioned with the corpus.

## Outputs

- A per-case **result record** carrying the case id, the highest grading level reached,
  the first level lost where the case failed, and the reasons and loci compared. Its
  field set and ordering are declared in `corpus.yaml`'s schema, so AC-15's byte-identity
  has a defined shape to be identical about.
- `bounds.gap_count`, published beside every score derived from the corpus.

## Behavior

The corpus loader SHALL require each case directory to declare, in `case.yaml`: `id`,
`issue_ref`, `mode`, `language`, `module`, `findable`, `reproduce`, and `kind` — one of
`failure` or `control`. The corpus loader SHALL additionally require `control_for` on a
case of kind `control`, naming the failure case's `id`; `relaxation_ticket` on a case
binding a variant module; and `pending_reason` on a case declaring `pending`.

A case MAY declare `pending`, naming the ticket that will make it pass.

The runner SHALL treat a case declaring `pending` as **expected to fail**.

The runner SHALL count and report every pending case.

The runner SHALL fail the run when a case declaring `pending` passes.

This is what makes *case red before fix* workable: a defect
gets its regression the day it is found, the fixture fails honestly, and the suite still
goes green. A pending case SHALL assert only the behaviour that is pending — anything
already true belongs in its control, or the marker hides a live assertion.

The corpus loader SHALL **derive** `bounds.gap_count` and the per-cell states from the
inventory and the fixtures present, and SHALL NOT read them from a stored value. A stored count is a number that can go stale; a
derived one cannot disagree with the tree it describes, and adding a fixture then moves
the count with no edit to any central file.

Only a case of kind `failure` SHALL mark a cell `covered`. A control asserts that
healthy input stays silent and measures nothing about the mode.

The corpus loader SHALL reject a case omitting any required field, naming the case and
the field.

The runner SHALL read the case's `input/` tree in place as static files, generating and
copying nothing. Where a case is **mutating** — exercising a command that writes, such as
`quire fix` — the runner SHALL operate on a copy, leaving the checked-in input
unmodified. This is the only case in which the runner copies anything.

The runner SHALL assert against `expect.yaml` as data. The runner SHALL accept a newly
added case directory without being edited.

The runner SHALL treat every `expect` field as optional. A corpus where each case pins
the whole envelope fails forty cases on one unrelated change and is then relaxed
wholesale — which is the failure mode that ends a corpus.

A case SHALL bind to the vendored ecosystem module by default. Where a case binds a
variant module, `case.yaml` SHALL name the relaxation ticket that variant sizes.

`corpus.yaml` SHALL be the single declaration of the bounds enum (`covered`,
`out-of-scope`, `GAP`), of the grading-ladder level names, and of the mode families. Each
runner SHALL read all three from `corpus.yaml` rather than carrying its own copy — which
is what makes single-definition a property a test in either repository can check, rather
than an agreement between two codebases nobody can verify from one of them.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-065-CON-1 | Case data SHALL NOT be embedded in runner code or generated at runtime. A case that cannot be read without executing something is not data. The one exception is stated in the Behavior section: a mutating case operates on a copy and never writes the checked-in tree. | Architecture | Test |
| FR-065-CON-2 | Every surface reporting `bounds.gap_count` SHALL render it as an absolute count, never normalised into a ratio or a percentage. This is FR-063-AC-6 applied to this metric, not a second rule. | Architecture | Test |
| FR-065-CON-3 | A case SHALL bind the vendored ecosystem module unless it names a relaxation ticket. A corpus whose manifest always matches cannot exhibit a declaration defect. | Architecture | Test |
| FR-065-CON-4 | The vendored `modules/ecosystem/` SHALL be refreshed from `spec-artifacts-process` by a recorded ritual that moves a pinned SHA, so the declaration a case binds is a reviewable event rather than a silent copy. | Process | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-065-AC-1 | A case is read from disk in place: no file under `input/` is generated, copied or materialised during a non-mutating run. | Test (TC-1011) |
| FR-065-AC-2 | A mutating case operates on a copy, leaving its checked-in `input/` tree byte-unchanged. | Test (TC-1011) |
| FR-065-AC-3 | A case omitting any required `case.yaml` field is rejected, and the rejection names the case and the field. | Test (TC-1012) |
| FR-065-AC-4 | A control case declaring no `control_for` is rejected. | Test (TC-1012) |
| FR-065-AC-5 | An `expect` field a case omits is asserted on by nothing; the omitted field is not defaulted. | Test (TC-1013) |
| FR-065-AC-6 | Every declared cell in `corpus.yaml` reads as exactly one of `covered`, `out-of-scope` or `GAP`. | Test (TC-1014) |
| FR-065-AC-7 | An `out-of-scope` cell carries a non-empty reason; one with an empty reason is rejected. | Test (TC-1014) |
| FR-065-AC-8 | A cell in none of the three states is rejected, naming the case and the language. | Test (TC-1014) |
| FR-065-AC-9 | `bounds.gap_count` is an integer count on every surface this crate emits. | Test (TC-1015) |
| FR-065-AC-10 | No payload this crate emits renders `bounds.gap_count` as a ratio or a percentage. | Test (TC-1015) |
| FR-065-AC-11 | A failing case reports the highest detection level it reached, distinguishing L1, L2 and L3. | Test (TC-1016) |
| FR-065-AC-12 | A failing case reports the first level it lost. | Test (TC-1016) |
| FR-065-AC-13 | A failure case whose `control_for` partner is absent is rejected, naming the missing control. | Test (TC-1017) |
| FR-065-AC-14 | A control case over healthy input produces no finding for the mode its partner asserts. | Test (TC-1017) |
| FR-065-AC-15 | A case binding a variant module without naming a relaxation ticket is rejected. | Test (TC-1018) |
| FR-065-AC-16 | A case binding the vendored ecosystem module loads without naming a ticket. | Test (TC-1018) |
| FR-065-AC-17 | Two runs of one case over unchanged input produce byte-identical result records. | Test (TC-1019) |
| FR-065-AC-18 | Each case's `case.yaml` carries the invocation that reproduces it, and that invocation names a module. | Test (TC-1020) |
| FR-065-AC-19 | The runner reads the bounds enum from `corpus.yaml` rather than a compiled-in list, so an enum value added there is accepted without a code change. | Test (TC-1021) |
| FR-065-AC-20 | The runner reads the grading-ladder level names from `corpus.yaml` rather than a compiled-in list. | Test (TC-1021) |
| FR-065-AC-21 | The runner reads the mode families from `corpus.yaml`, and a case naming an undeclared family is rejected. | Test (TC-1021) |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the coverage payload cases assert against), [FR-051](./FR-051-source-symbol-extraction.md) (the binding census the L2 level reads), [FR-055](./FR-055-published-output-contract.md) (the payload shape a case's `expect` is written against), [FR-063](./FR-063-metric-provenance-envelope.md) (the envelope and the count-vs-ratio rule `bounds.gap_count` is an instance of)
- **Downstream**: agent-ix/quire-rs#266 (the `qa-corpus` artifact), #267 (this crate's runner), #268 (the case inventory), agent-ix/quoin#227 (the sibling runner)
