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
whose manifest heading always matches cannot exhibit the section defect at all** — the
one whose candidate census is 3,514 unminted TC ids across 88 repositories. (That census
is a candidate population, not the defect's causal size: CR-118 measured the section fix
at +83 rows, because the population is confounded with id-column mismatch —
`agent-ix/quire-rs#318`.) Tier 1 has never caught this failure mode because tier 1 was
built where the mode cannot occur.

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

### The witness channel: which output constitutes detection

A failure case's live block, graded against its control's payload, must produce a
mismatch (AC-42). That closes empty blocks and assertions true of every input, and it is
a **floor rather than closure**: it proves an assertion distinguishes *these two
payloads*, not that the assertion concerns *the seeded defect*. Measured by running every
failure case and every control and comparing `totals.total`, over the whole controlled
population and no sample, **14 of the 35 (case, control) pairs differ in `total`** —
equivalently 14 of the 34 controlled cases — an incidental global row count satisfying
the whole rule while saying nothing about the family the case is named for.

`corpus.yaml` SHALL therefore declare, per mode, the **witness channels**: the output
this family's detection is *observable in*. A block SHALL be graded twice — once whole,
and once **restricted** to its mode's channels — and the restriction SHALL still
mismatch.

Restriction rather than mismatch inspection, because the two claims differ. A list of
which mismatches fired says something fired somewhere, which is what AC-42 already
asserts. Dropping every non-witness key and re-grading says **the witness channel itself
discriminates**, which is the claim with content.

`total` is a witness for `minting` and for nothing else: a minted-row count *is* the
minting channel, and everywhere else it is the incidental scalar. `backed` is barred from
`minting` for the mirror reason — a row's minting and its backing are different facts.

A channel a reader cannot restrict on SHALL be rejected rather than dropped, or the rule
weakens silently for exactly the mode that declared it (AC-47). **By EVERY reader**, and
that clause is here because the rule shipped in one of them. CR-130 wrote AC-42's row with
"by every reader" and AC-46's and AC-47's without it, and AC-47 then landed in the Rust
harness alone — reproduced by changing `witness_channels.disposition`'s `unbacked_rows` to
`unbacked_rowz`, one character, whereupon `verify.py` reported 77/77 over 35 pairs with 0
mismatches and exit 0 while `cargo test tc1028` panicked. `bounds.py` does not read
`witness_channels`, `schema_selftest` does not cover it and `parity_selftest` still passed
6/6, so the corpus's whole `make ci` was green on a corpus where AC-46 had been weakened
for a mode. The asymmetry in the criteria table was the drafting error underneath it and
is corrected with the code (CR-132). A `witness_channels` value that is not a LIST, and a
declared `mode_family` with no entry at all, are rejected under the same criterion for the
same reason: both make the restriction quietly empty rather than loudly wrong.

**One weakening is stated rather than left to be discovered.** `validate_contains` /
`validate_absent` are a witness in every mode, because `quire validate` is a second
*oracle* rather than a coverage channel: a defect visible only to it produces a coverage
payload byte-identical to a healthy tree's. So within one mode a case can satisfy AC-46
through the validate oracle alone.

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
IX_FILAMENT_MODULES_PATH=modules/ecosystem \
  quire coverage --scope cases/<mode>/<case>/input --json
```

Naming the module is not optional decoration: without it no traceability model loads,
the run reports `0/0 rows backed`, and the case cannot exhibit the declaration defect it
exists for. If the only way to see a case fail is to run the runner, the corpus is code
again and the contract has bought nothing.

**The ecosystem declaration is a module PATH, not one module** (CR-108). It is
`spec-artifacts-process` *and* `spec-artifacts-iso`: the first carries the traceability
model, the second declares `FR`, `NFR` and `TestMatrix`. `--module` takes a single
directory, so a path is selected with `IX_FILAMENT_MODULES_PATH`. A case binding one
module still uses `--module`, and the runner reads whichever shape the case declares.

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
  static files, `expect.yaml`, and — for a case declaring `pending` —
  `expect-pending.yaml`.
- `modules/ecosystem/`: the real declaration, vendored **whole** — both
  `spec-artifacts-process` and `spec-artifacts-iso`, each with its own pinned SHA.
  Whole directories rather than manifests, because archetypes reference their schema
  files relative to the module root. `modules/variants/<id>/`: relaxation variants.
- `labels/`, `config/metrics.json`, `baselines/{quire-rs,quoin}.json`: ground truth,
  the metric dictionary, and per-runner baselines, versioned with the corpus.

## Outputs

- Per failing case, the highest grading level reached and the first level lost, with the
  reasons and loci compared (AC-11, AC-12). This is what a runner **prints**; it is
  deliberately not a declared wire record — see CR-124.
- The engine report each case's `reproduce` invocation produces, byte-identical across
  two runs over unchanged input (AC-17).
- `bounds.gap_count`, published beside every score derived from the corpus.

## Behavior

The corpus SHALL declare its case-metadata schema in `corpus.yaml` as `case_schema`.

That declaration SHALL carry the required field set, the optional field set, each field's
type, the per-`kind` required, forbidden and constrained-value rules, the
conditionally-required fields, the fields a language-set variant is forbidden to declare,
and the fields that are unique across the corpus.

Every corpus reader SHALL be held to that declaration rather than to a list of its own.
This requirement used to name the fields here, which made this paragraph a second
declaration free to disagree with both readers — and it did: it named `case` as required
where no control declares one, and omitted `tags`, which TC-1021 has always required to
carry a `TC-` id. **The prose is not the schema. `corpus.yaml` is** (CR-126).

A reader SHALL reject a case that omits a required field, carries one empty, carries a
field the schema declares neither required nor optional, violates a per-`kind` rule, or
whose derived `id` collides with another case's. A reader finding no `case_schema` SHALL
fail rather than skip: a reader that quietly does nothing when its rules are absent is
indistinguishable from one that checked and found nothing.

`kind` is one of `failure`, `control` or `regression` (CR-116), from `case_kinds`.

### Where a language set declares each field (CR-109)

A **language set** splits that declaration across two files, and the split is a contract
rather than a convention two readers happen to agree on:

| file | carries |
|---|---|
| `<case>/case.yaml` | everything shared: `id`, `case`, `issue_ref`, `mode`, `module`, `kind`, `findable`, `control_for` |
| `<case>/<language>/case.yaml` | only what varies: `reproduce`, and any per-language `expect` override |

The runner SHALL take `language` from the **directory name**, never from a declared
field — a shared `language:` is dead config, and a variant declaring one that disagrees
with its directory would be two claims about one fact.

The runner SHALL derive a variant's `id` as `<shared id>-<language>`, so one fixture has
one identity in every reader. A variant SHALL NOT override `case`, `mode`, `module`,
`kind` or `pending`: those declare *which case this is*, and varying them re-points the
cell the fixture credits — measured, a one-line override moved a covered cell to a
different inventory row while `gap_count` did not change.

A case directory carrying both an `input/` and a `<language>/` SHALL be rejected rather
than silently read as one or the other.

`control_for` names the partner's **`case`**, and resolution is per language: a control
pairs with the failure case in its own language, which is the only pairing that means
anything. The runner SHALL resolve it against **failure cases only** — including
controls puts a control's own `case` in the namespace, so `control_for` resolves against
itself and the check becomes self-satisfying. The corpus loader SHALL additionally require
`control_for` on a case of kind `control`; `relaxation_ticket` on a case binding a
variant module; and `pending_reason` on a case declaring `pending`.

`control_for` names a LIST of partners' `case` values, never an `id` (CR-110). A list
because one control can legitimately serve several failure cases — the healthy repair of
two single-cell defects in one document is the same document — and measured, two controls
authored separately for two such cases were byte-identical expectations over input trees
differing by one blank line, with swapping their `control_for` leaving every gate green. An earlier
wording said `id` two paragraphs after saying `case`, and for a language set those
are different strings — a set's variant `id` is `<shared id>-<language>`, which a
*shared* field cannot name. An author following the `id` sentence would have written
`control_for: rows-across-many-headings-rust` and had it resolve in Rust and be
rejected in the other two.

A case MAY declare `pending`, naming the ticket that will make it pass. A case
declaring `pending` SHALL ship an `expect-pending.yaml`, and a case shipping one
SHALL declare `pending`; the corpus loader SHALL reject either alone.

A case SHALL carry **two expectation blocks**, and every runner SHALL grade both
against one run of the payload (CR-110):

| File | Contract | Required outcome |
| --- | --- | --- |
| `expect.yaml` | what holds **today** | MUST hold, for every case, pending or not |
| `expect-pending.yaml` | what the named ticket will make hold | MUST NOT hold yet |

The runner SHALL count and report every pending case, and SHALL fail the run when a
case's `expect-pending.yaml` starts holding — the fix landed, and the marker is now
lying about the engine.

This is what makes *case red before fix* workable: a defect gets its regression the
day it is found, the fixture fails honestly, and the suite still goes green.

The split replaces an earlier rule that a pending case *"SHALL assert only the
behaviour that is pending — anything already true belongs in its control"*. That rule
was a consequence of `pending:` excusing a case's whole expectation block, and it cost
more than it bought: with the live facts pushed into the control, a **failure** fixture
asserted nothing about the payload it was named for. Measured on the two minting rows,
both fixtures could have regressed to minting nothing at all, in all three languages,
and the suite would have stayed green — including the one field that distinguishes them
from each other. A control cannot hold a failure case's live facts, because it does not
have them: its input is healthy.

`corpus.yaml` SHALL declare every diagnostic reason a fixture may name, in two parts:
`emitted`, the tokens the engine produces today, and `forward`, a map from a token to
the ticket that will introduce it. Both runners SHALL reject a token declared in
neither.

A live `expect.yaml` SHALL NOT REQUIRE a `forward` token — it must hold today. A forward
`expect-pending.yaml` SHALL require only `forward` tokens whose ticket is the case's own
`pending`, so a fixture cannot wait on one ticket while asserting another's behaviour.

A forward `expect-pending.yaml` SHALL require **at least one** `forward` token whose
ticket is the case's own `pending`. Being merely FALSE is not enough: a block asserting
`backed: 99` is false today and false after the fix, so the case stays pending forever
and no gate ever says the fixture went stale. A forward block has to be ABOUT its ticket.

`corpus.yaml`'s `emitted` and `forward` lists SHALL be checked against the engine rather
than maintained by hand: a token declared `emitted` that the engine does not produce, or
a token declared `forward` that it already does, SHALL fail. Two hand-written lists drift
from each other the same way one list drifts from the code.

A case of kind `control` SHALL bind the same `mode` and `module` as each partner it names.
A control is the healthy version of ITS partner, not any case that happens to resolve.

A case of kind `control` SHALL NOT declare `case`. A control credits no cell, and one
control may serve several inventory rows through its partners while `case` can name only
one.

`corpus.yaml` MAY declare `known_gaps`, and every reader SHALL enforce it in both
directions: a departure listed there is permitted, one not listed fails the run, and an
entry naming no case in the corpus fails it too. A declaration nobody reads is how a
block came to list three uncontrolled failure cases where eleven were true. Every `known_gaps`
entry SHALL name a ticket and at least one case: `pending` requires a `pending_reason`
and a variant `module` requires a `relaxation_ticket`, but an exemption from the contract
itself required neither, so one appended line was permanent by default.

EVERY expectation block SHALL assert something, live and forward alike. This was enforced
on the forward block by both readers and on `expect.yaml` by neither, so truncating any
failure case's `expect.yaml` to zero bytes left every gate green with its cell still
`covered` — a case asserting nothing about its own payload, counted as covering the mode
it is named for.

A case MAY declare `kind: regression`. Such a case has no control, is not `findable`, and
asserts behaviour a LANDED ticket established rather than a defect that still reproduces.

This kind was added when the first Wave 3 fix landed, because the corpus had no way to say
*"this used to break and must not again"*. `agent-ix/quire-rs#274` is the reason: its
fixture's control was the same content written the way the broken parser could handle, so
once the parser was fixed **both spellings parsed identically** and the pair had nothing
left to separate — AC-42 rejected it, correctly. The input is still the shape that used to
break, and asserting it parses correctly is worth keeping; it is simply no longer a defect.

An **ecosystem-bound** `regression` case SHALL credit its inventory cell. The mode is
exercised whether or not the behaviour is currently broken, and a cell that reverted to
GAP when its defect was fixed would make `gap_count` count *unfixed* defects rather than
*unmeasured* modes.

A case binding a **relaxation variant** SHALL credit no cell, whatever its `kind`, and the
loader SHALL mark that cell `GAP` with a reason naming the case's `relaxation_ticket`.
CON-3 outranks the paragraph above and the two rules are about different things: `kind`
answers *is this behaviour currently broken*, and the binding answers *can this fixture
exhibit an ecosystem defect at all*. A corpus whose manifest always matches cannot, so
crediting it would restate the defect this corpus exists to end. AC-44 exists so a cell
does not revert to GAP the moment its defect is **fixed**; a variant-bound cell never
credited in the first place, which is a different thing from one that lost its credit.

This is not hypothetical and it is what `agent-ix/quire-rs#285` left behind. Eleven
fixtures bound a relaxation variant — nine `modules/variants/bench-legacy/` and two
`modules/variants/bench-no-symbol-vocab/`, counted from `bounds.py --json` at corpus
`db55b05`. **Ten** migrated onto the vendored declaration and both variants were deleted.
The eleventh, `cases/provenance/implements-never-asked`, asserts `coverage.implements` is
`state: not_computed` — the `#226` distinction between *computed and found none* and
*never asked* — which the engine reaches only when the loaded module declares no
`implements` forms, and the vendored ecosystem declares three. It is a `regression` pin
(`#226` landed, nothing reproduces) bound to a variant relaxing exactly that one axis, so
it is a `regression` case whose cell reads `GAP`. Under the unqualified rule it was a
counterexample to this document.

A `regression` case SHALL name the ticket whose fix it pins, in `issue_ref`, so a reader
can tell a pin from a fixture nobody finished.

Not every fix produces one. Where a fix makes a diagnostic fire, the control stays silent
and the pair keeps discriminating — those fixtures fold their forward block into their live
block and remain `failure` cases. Only a fix that makes the case and its control
*equivalent* leaves a regression pin behind.

A failure case that has a control SHALL **discriminate**: its `expect.yaml`, graded
against its control's payload, SHALL produce at least one mismatch.

**EVERY corpus reader SHALL implement this** (CR-128). It is the strongest rule in this
requirement and it existed in one of the two readers: the Rust harness graded it from the
commit TC-1028 landed, while `verify.py` ran each case once against its own payload and
never cross-graded anything, and the Python loader checked only that a control was
*named* — a predicate on shape, which is the class of defect this rule exists to close.
So the two-reader independence this contract rests on stopped immediately before the
check that carries it. Proved by construction: a fixture blinded to `total: 4` — an
assertion true of the defective tree and of the repaired one — was accepted by
`verify.py` at `mismatches: 0`, exit 0, and rejected by `cargo test --test corpus_cases`
at the same corpus revision.

Every reader SHALL resolve `control_for` by ONE rule: a name resolves to the failure case
whose **`id`** it is, and only failing that to the case whose **`case:` alias** it is; an
alias SHALL NOT displace a real id. A failure case SHALL be graded against **every**
control that names it, not one of them — two controls legitimately name one case, and any
rule selecting one selects it by iteration order. The two readers disagreed on both
points, and the resolutions are now asserted against each other rather than assumed to
agree.

The rule is *"assert at least one fact that differs between the two trees"*, which is
weaker than *"assert a fact about the defect"*. Measured on this corpus at `776a6b3`: ten
of eleven controlled fixtures are satisfiable by one incidental scalar — `total: 1` passes
for a case whose control totals 2 — and only `no-symbol-method-in-the-verification-column`,
whose control matches it on every count, is forced onto a defect-specific field. So this
raises the floor rather than closing the question, and it is stated that way because an
earlier draft of this clause claimed more.

**That audit has not been repeated and its denominator has moved** (CR-123). Controlled
failure cases were 11 at `776a6b3`, 33 at `db55b05` and **34** at `3ff72c0`. "Ten of
eleven" therefore describes eleven of the thirty-four that exist, and the twenty-three
authored since were never assessed against it. Re-running it is
`agent-ix/quire-rs#301`. The figure is left in place rather than deleted because it was
true when measured and deleting it would lose the finding; it is annotated rather than
extrapolated because a fraction restated over a population it was not measured on is the
fabrication this bundle keeps catching.

**THE `3ff72c0` FIGURE READ 35 UNTIL CR-128, AND ITS STATED METHOD WAS WRONG TWICE.** It
was attributed to `bounds.py --json`, which emits `bounds` and `cases` and no count of
controlled failure cases at all — the number came from the Rust harness. Re-measured at
`3ff72c0` by calling each reader's own resolution: `bounds.controlled_cases()` returns
**34**, the harness's returned **35**, and the extra was `marker-mismatch`, which this
corpus DECLARES under `known_gaps.uncontrolled_failure_cases` and which the harness
reached through another case's `case:` alias. So a published figure was one reader's
count of a defect, cited to the other reader's tool. Both now return 34 cases, over which
**35 (case, control) pairs** are graded, because two controls name
`marker-form-mismatch`.

A separate measurement of how low the floor is. Method: run every failure case and every
control and compare `totals.total`. Population: the whole controlled set, no sample, at
corpus `801afd5` (fixtures byte-identical at `2bc486d`) with CLI 0.30.2 / engine 0.33.0.

| unit | share an identical `total` | differ | n |
|---|---|---|---|
| per **case** (34 controlled failure cases) | **20** | **14** | 34 |
| per **(case, control) pair** | **21** | **14** | 35 |

Where they share it the incidental scalar is not available as an evasion; for the 14 that
differ it is. **This paragraph published "20 pairs" until CR-132**, which is 20 CASES —
over pairs it is 21. The 14 is right under both units, so only the noun was wrong, and it
was wrong in permanent requirement text. This supersedes nothing above: it counts a
different thing over a different population than the `776a6b3` audit, which is why it is
stated separately rather than as an update.

The `validate_*` assertions SHALL be graded over the OTHER case's tree when a case is
graded differentially. `quire validate` reads a spec TREE rather than a coverage payload,
so recomputing it from the case's own tree makes those keys contribute no discrimination
— and `wrong-type-cell`, whose coverage payload is byte-identical to a healthy tree's by
design and whose entire claim is structural, would be rejected as blind the moment it
gained a control.

**THAT RULE HAS REACH 0 OVER THE CORPUS TODAY, AND IT IS KEPT ANYWAY** (CR-132). Measured
at corpus `2bc486d` over all 77 discovered fixtures: exactly **two** declare a `validate_*`
key — `wrong-type-cell`, a `failure` declared under
`known_gaps.uncontrolled_failure_cases`, which no control names and both readers skip,
and `clean-control`, a `control`, which the differential does not iterate — so **zero of
the 35 graded (case, control) pairs carry one**, in either reader. `git log -S
'wrong-type-cell' -- cases/` returns one commit, the fixture's own introduction, so no
`control_for` has ever named it. The rule is correct and has no current subject; those
are different things, and the commentary around this requirement conflated them until
CR-132 (`verify.py`, `scripts/parity_selftest.py`, `tests/corpus_case/mod.rs`,
`spec/log.md`, all corrected). `agent-ix/quire-rs#286` giving `wrong-type-cell` its
control is what raises the reach above zero.

A pending case's `expect-pending.yaml` is NOT held to this rule. AC-36 requires it to
require a `forward` token and AC-35 guarantees no engine emits one, so it cannot hold
against any payload — grading it differentially is a theorem restated as a test, and
TC-1023 already makes the claim that has content.

It is still different in kind from everything else here. Every other gate is a predicate
on the SHAPE of a declaration — a block is non-empty, a token is
declared, a ticket is named, a control exists — and shape has an unbounded supply of
forms that are non-empty and mean nothing. Five review rounds each removed one such form
and the next round found another: an excused block, then a block naming any token, then
one naming a real token with a false companion, then a truncated file, then a block
asserting only a true row count. Discrimination is a predicate on MEANING, and it closes
the class rather than an instance.

A failure case with no control cannot be held to it — there is nothing to discriminate
against — which is what makes an uncontrolled failure case a declared gap rather than a
matter of taste.

A case declaring `findable` SHALL require at least one finding, in one of its blocks. The
flag tells a recall-scoring consumer to expect a finding on that input, and a case that
names nothing which finds it counts its cell covered for a mode nothing measures.
Measured: three fixtures shipped byte-identical live blocks true of any healthy corpus,
and swapping one's whole `input/` tree for another's left every gate green.

A case of kind `control` MAY assert a `forward` token ABSENT in its `expect.yaml`: that
claim is vacuous today and load-bearing the day the ticket lands, which is what a control
is for. A case of kind `failure` SHALL NOT — its live block would assert the absence of
the very behaviour it waits for, and would therefore FAIL on the fix rather than pass it.

When a ticket lands, its token MOVES from `forward` to `emitted`. That single edit is
what makes every fixture in its group go green at once.

An `expect-pending.yaml` asserting nothing SHALL be rejected. A block grading zero
assertions trivially holds, and every runner then reports its ticket as landed — measured
with a 0-byte file and with `{}`, both runners announced a fix that had not happened, and
a reader following that instruction would delete the marker and turn the regression
fixture into a green case asserting nothing.

The corpus loader SHALL **derive** `bounds.gap_count` and the per-cell states from the
inventory and the fixtures present, and SHALL NOT read them from a stored value. A stored count is a number that can go stale; a
derived one cannot disagree with the tree it describes, and adding a fixture then moves
the count with no edit to any central file.

Only a case of kind `failure` or `regression`, bound to the vendored ecosystem module,
SHALL mark a cell `covered`. A control asserts that healthy input stays silent and
measures nothing about the mode; a variant-bound case exercises no ecosystem mode at all.
This sentence read "only a case of kind `failure`" until CR-123 and had been false since
`regression` was added — `bounds.py` credited both kinds from the commit that introduced
the second one, and AC-44 said so while this paragraph still said otherwise.

The corpus loader SHALL reject a case omitting any required field, naming the case and
the field.

The runner SHALL read the case's `input/` tree in place as static files, generating and
copying nothing. Where a case is **mutating** — exercising a command that writes, such as
`quire fix` — the runner SHALL operate on a copy, leaving the checked-in input
unmodified. This is the only case in which the runner copies anything.

The runner SHALL assert against both expectation blocks as data. The runner SHALL accept
a newly added case directory without being edited.

The runner SHALL treat every `expect` field as optional, and an OMITTED field is
asserted on by nothing. A corpus where each case pins the whole envelope fails forty
cases on one unrelated change and is then relaxed wholesale — which is the failure mode
that ends a corpus.

An omitted field and an EMPTY one are different claims. `unbacked_rows: []` says the
declaration minted nothing; omitting the key says this case is not about what minted.
The distinction is what lets one fixture assert the absence of a row another asserts
the presence of, which is the only thing separating the two minting cases.

A case SHALL bind to the vendored ecosystem module by default. Where a case binds a
variant module, `case.yaml` SHALL name the relaxation ticket that variant sizes.

`corpus.yaml` SHALL be the single declaration of the bounds enum (`covered`,
`out-of-scope`, `GAP`), of the grading-ladder level names, and of the mode families —
which is what makes single-definition a property a test in either repository can check,
rather than an agreement between two codebases nobody can verify from one of them.

**What each runner does with those three is not the same, and this section claimed it
was** (CR-129).

The **mode families** and the **bounds enum** are vocabularies: a runner SHALL read them
and SHALL NOT carry its own copy. `bounds.py` derives its per-state counters, its sum
invariant and its rejection of an undeclared state from `bounds_states`, so a state added
to the declaration is counted and reported with no code edit. Assignment of a cell to a
state is the derivation rule and remains code.

The **grading ladder** is not a vocabulary. `Level`'s variants and the assignment of each
mismatch to one of them ARE the grading rule; a fourth declared level would have no
variant to carry it and no mismatch would ever be filed under it. So the runner SHALL NOT
be required to accept a ladder change without a code edit — a claim this requirement made
and no implementation ever satisfied. What it SHALL do is **agree**: the compiled ladder
and the declared ladder SHALL match in name and in order, and a gate SHALL fail on
disagreement. Order is normative because the first level lost is a MINIMUM over the
ladder. TC-1021 asserted the declaration equalled a literal `["L1","L2","L3"]` written in
the test — a third copy checked against the second while the enum that grades was checked
against neither; it now renders `Level::ALL` and compares that.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-065-CON-1 | Case data SHALL NOT be embedded in runner code or generated at runtime. A case that cannot be read without executing something is not data. The one exception is stated in the Behavior section: a mutating case operates on a copy and never writes the checked-in tree. | Architecture | Test |
| FR-065-CON-2 | Every surface reporting `bounds.gap_count` SHALL render it as an absolute count, never normalised into a ratio or a percentage. This is FR-063-AC-6 applied to this metric, not a second rule. | Architecture | Test |
| FR-065-CON-3 | A case SHALL bind the vendored ecosystem module unless it names a relaxation ticket. A corpus whose manifest always matches cannot exhibit a declaration defect. | Architecture | Test |
| FR-065-CON-4 | The vendored `modules/ecosystem/` SHALL be refreshed from every declaring module by a recorded ritual that copies whole module directories and moves each pinned SHA, so the declaration a case binds is a reviewable event rather than a silent copy. | Process | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-065-AC-1 | A case is read from disk in place: no file under `input/` is generated, copied or materialised during a non-mutating run. | Test (TC-1011) |
| FR-065-AC-2 | A mutating case operates on a copy, leaving its checked-in `input/` tree byte-unchanged. | Test (TC-1011) |
| FR-065-AC-3 | A case omitting any field `case_schema.required` names is rejected by every reader, and the rejection names the case and the field. | Test (TC-1012, TC-1043) |
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
| FR-065-AC-17 | Two runs of one case over unchanged input produce a byte-identical engine report. | Test (TC-1019) |
| FR-065-AC-18 | Each case's `case.yaml` carries the invocation that reproduces it, and that invocation names a module. | Test (TC-1020) |
| FR-065-AC-19 | `bounds.py` derives its per-state counters and its sum invariant from `corpus.yaml`'s `bounds_states`, and rejects a cell graded into a state the declaration does not name. | Test (TC-1021, `bounds.py`) |
| FR-065-AC-20 | The compiled grading ladder and `corpus.yaml`'s `grading_levels` agree in name and in order, and a gate fails on disagreement. The runner is not required to accept a ladder change without a code edit: the ladder is the grading rule, not a vocabulary (CR-129). | Test (TC-1021) |
| FR-065-AC-21 | The runner reads the mode families from `corpus.yaml`, and a case naming an undeclared family is rejected. | Test (TC-1021) |
| FR-065-AC-22 | A language set's variant declares only what varies; a variant DECLARING `case`, `mode`, `module`, `kind` or `pending` is rejected naming the field, whether or not the shared file also declares it. | Test (TC-1022) |
| FR-065-AC-23 | Every reader derives the same `id` for one variant, `<shared id>-<language>`, so a record keyed on `id` joins across runners. | Test (TC-1022) |
| FR-065-AC-24 | `control_for` resolves against failure cases only, in the control's own language; a control whose partner is absent is rejected. | Test (TC-1017) |
| FR-065-AC-25 | Every case's `expect.yaml` is graded and MUST hold, whether or not the case declares `pending`. | Test (TC-1023) |
| FR-065-AC-26 | A case declaring `pending` and shipping no `expect-pending.yaml` is rejected, and so is one shipping `expect-pending.yaml` and declaring no `pending`. | Test (TC-1023) |
| FR-065-AC-27 | A case whose `expect-pending.yaml` holds fails the run, naming the ticket in its `pending`. | Test (TC-1023) |
| FR-065-AC-28 | A case asserting `unbacked_rows` or `groups` fails on a payload carrying an extra entry, a missing entry, a changed field or a different order; a case asserting an empty list fails on a payload carrying any entry. | Test (TC-1024) |
| FR-065-AC-29 | A case naming several substrings for one reason fails when the message carries any proper subset of them. | Test (TC-1024) |
| FR-065-AC-30 | A block naming a reason token that `corpus.yaml` declares neither emitted nor forward is rejected, naming the token. | Test (TC-1025) |
| FR-065-AC-31 | A live block requiring a `forward` token is rejected; a forward block requiring a token whose ticket is not the case's own `pending` is rejected. | Test (TC-1025) |
| FR-065-AC-32 | A failure case asserting a `forward` token ABSENT is rejected; a control asserting one in its live block is accepted. | Test (TC-1025) |
| FR-065-AC-33 | An `expect-pending.yaml` grading zero assertions is rejected rather than read as its ticket having landed. | Test (TC-1025) |
| FR-065-AC-34 | A token declared `emitted` that the engine does not produce is rejected. | Test (TC-1026) |
| FR-065-AC-35 | A token declared `forward` that the engine already produces is rejected, naming the ticket that appears to have landed. | Test (TC-1026) |
| FR-065-AC-36 | An `expect-pending.yaml` requiring no token its own `pending` ticket introduces is rejected, and one requiring an already-emitted token is rejected. | Test (TC-1025) |
| FR-065-AC-37 | A control whose `mode` or `module` differs from a partner's is rejected unless declared in `known_gaps`; an entry in `known_gaps` naming no case is rejected. | Test (TC-1027) |
| FR-065-AC-38 | A failure case named by no control is rejected unless declared in `known_gaps`. | Test (TC-1027) |
| FR-065-AC-39 | An `expect.yaml` grading zero assertions is rejected, for every case. | Test (TC-1025) |
| FR-065-AC-40 | A case declaring `findable` and requiring no finding in any block is rejected unless declared in `known_gaps`. | Test (TC-1027) |
| FR-065-AC-41 | A `known_gaps` entry naming no ticket, or naming no case, is rejected. | Test (TC-1027) |
| FR-065-AC-42 | A failure case with a control whose `expect.yaml` holds against that control's payload is rejected **by every reader**, with `validate_*` graded over the control's tree, against every control that names the case. | Test (TC-1028; `qa-corpus` `scripts/parity_selftest.py`) |
| FR-065-AC-43 | A case of kind `regression` is accepted with no control and is not held to AC-42; one declaring `findable`, `control_for` or `pending` is rejected. | Test (TC-1032) |
| FR-065-AC-44 | An ecosystem-bound `regression` case credits its inventory cell, so a cell does not revert to GAP when its defect is fixed. | Test (TC-1032) |
| FR-065-AC-45 | A case binding a relaxation variant credits no cell whatever its `kind`, and that cell reads `GAP` with a reason naming its `relaxation_ticket`. | Test (TC-1032) |
| FR-065-AC-46 | A failure case whose `expect.yaml`, restricted to the `witness_channels` its mode declares, holds against its control's payload is rejected **by every reader** — as is one naming no witness channel at all. | Test (TC-1028; `qa-corpus` `scripts/parity_selftest.py`) |
| FR-065-AC-47 | A `witness_channels` entry naming a channel a reader cannot restrict on is rejected **by every reader**, rather than dropped; so is one that is not a list, and a declared `mode_family` with no entry at all. | Test (TC-1028; `qa-corpus` `verify.py check_witness_channels`) |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the coverage payload cases assert against), [FR-051](./FR-051-source-symbol-extraction.md) (the binding census the L2 level reads), [FR-055](./FR-055-published-output-contract.md) (the payload shape a case's `expect` is written against), [FR-063](./FR-063-metric-provenance-envelope.md) (the envelope and the count-vs-ratio rule `bounds.gap_count` is an instance of)
- **Downstream**: agent-ix/quire-rs#266 (the `qa-corpus` artifact), #267 (this crate's runner), #268 (the case inventory), agent-ix/quoin#227 (the sibling runner)
