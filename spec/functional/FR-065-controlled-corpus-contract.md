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
  - target: "ix://agent-ix/quire-rs/StR-004"
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

### The corpus is the bounds of the use case, not a score

A case list answers "what did we try". A **bounds matrix** answers "what did we never
try", and only the second is a statement about the tool. Every mode × language cell is
`covered`, `out-of-scope` with a written reason, or `GAP`.

**A scenario with no case is undefined behaviour, not assumed-working.** Today the
matrix is 10 cells and every one is Rust; the Python and TypeScript columns are `GAP`,
and v0.44.0 shipped two `high` defects (#250, #251) straight through them.

`bounds.gap_count` is a **count**, never a ratio. A ratio falls as easy cases are added,
so a corpus can improve its number while the hard missing case stays missing. Converting
a `GAP` to `out-of-scope` moves the count, which makes declaring something out of scope a
visible act rather than a quiet one.

### Detection is graded, not boolean

| Level | Question | Asserted by |
|---|---|---|
| L1 detected | Did anything fire? | `diagnostic_reasons` |
| L2 localised | Did it name the right `path:line`? | `diagnostic_paths`, `binding_census.unbound_example` |
| L3 actionable | Did the message name the thing to change? | `diagnostic_message_contains` |

Ecosystem `actionability_rate` is **3.02%** — 15 of 496 findings carried a row id. L1
without L2 is an alert nobody acts on, and scoring it as a pass is how a detector earns
credit for noise. A failing case SHALL report **which level was lost**, because "the
case failed" and "the message stopped naming the row" are different repairs.

### Every failure case ships its control

A detector that fires on everything scores perfect recall. That is not hypothetical
here: #250 shipped a check that produced **549 suspicions from 551 candidates** on a
TypeScript corpus, and recall alone called it excellent. A failure case without healthy
input that must stay silent measures nothing.

### A mode is authored in every language it applies to

The 16-mode inventory times three languages is the shape of the claim this toolchain
makes. A mode authored only in Rust is a claim about Rust; if the other cells are not
`out-of-scope` with a reason, they are `GAP` and the bounds say so.

### Reproducible by hand, or it has regressed to what it replaced

A case reproduces with a documented invocation over its own directory — `cd
cases/<mode>/<case>/input && quire coverage --scope .` — with no harness and no
generator. If the only way to see a case fail is to run the runner, the corpus is code
again and the contract has bought nothing.

## Inputs

- A case directory: `case.yaml`, an `input/` tree of static files, `expect.yaml`.
- `corpus.yaml`: the mode × language bounds matrix.
- `modules/ecosystem/`: the real declaration, vendored and pinned by SHA.

## Outputs

- A per-case result naming the highest grading level reached and the first level lost.
- `bounds.gap_count`, published beside every score derived from the corpus.

## Behavior

The corpus loader SHALL require each case directory to declare `id`, `issue_ref`, `mode`,
`language`, `module` and whether the case is a failure case or a control, in `case.yaml`.
The runner SHALL read the case's `input/` tree in place as static files, generating and
copying nothing. The runner SHALL assert against `expect.yaml` as data. The runner SHALL
accept a newly added case directory without being edited.

A case SHALL bind to the **real** module declaration by default. Where a case binds a
variant module, `case.yaml` SHALL name the relaxation ticket that variant sizes.

The runner SHALL treat every `expect` field as optional. A corpus where each case pins
the whole envelope fails forty cases on one unrelated change and is then relaxed
wholesale — which is the failure mode that ends a corpus.

The corpus loader SHALL reject a case declaring no `issue_ref`. A fixture whose origin is
unrecorded becomes a fixture nobody dares change, which is how a corpus rots into a set
of assertions everybody works around.

This requirement SHALL be the single definition of the bounds enum (`covered`,
`out-of-scope`, `GAP`) and of the grading ladder (L1/L2/L3). A runner introducing a
second spelling of either SHALL fail the contract check.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-065-CON-1 | Case data SHALL NOT be embedded in runner code, generated at runtime, or materialised to a temporary directory. A case that cannot be read without executing something is not data. | Architecture | Test |
| FR-065-CON-2 | Every surface reporting `bounds.gap_count` SHALL render it as an absolute count, never normalised into a ratio or a percentage. | Architecture | Test |
| FR-065-CON-3 | A case SHALL NOT bind to a module whose declaration differs from the ecosystem's except as a named relaxation experiment. A corpus whose manifest always matches cannot exhibit a declaration defect. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-065-AC-1 | A case is read from disk in place: no file under `input/` is generated, copied or materialised during a run. | Test (TC-1011) |
| FR-065-AC-2 | A case declaring no `issue_ref` is rejected by the loader, and the rejection names the case. | Test (TC-1012) |
| FR-065-AC-3 | An `expect` field a case omits is asserted on by nothing; the omitted field is not defaulted. | Test (TC-1013) |
| FR-065-AC-4 | Every mode × language cell in `corpus.yaml` reads as exactly one of `covered`, `out-of-scope` or `GAP`. | Test (TC-1014) |
| FR-065-AC-5 | An `out-of-scope` cell carries a non-empty reason; one with an empty reason is rejected. | Test (TC-1014) |
| FR-065-AC-6 | A cell in none of the three states is rejected, naming the mode and the language. | Test (TC-1014) |
| FR-065-AC-7 | `bounds.gap_count` is an integer count on every surface that reports it. | Test (TC-1015) |
| FR-065-AC-8 | No surface renders `bounds.gap_count` as a ratio or a percentage. | Test (TC-1015) |
| FR-065-AC-9 | A failing case reports the highest detection level it reached, distinguishing L1, L2 and L3. | Test (TC-1016) |
| FR-065-AC-10 | A failing case reports the first level it lost. | Test (TC-1016) |
| FR-065-AC-11 | A failure case whose control is absent for the same mode and language is rejected. | Test (TC-1017) |
| FR-065-AC-12 | A control case over healthy input produces no finding for the mode its paired failure case asserts. | Test (TC-1017) |
| FR-065-AC-13 | A case binding a module other than the vendored ecosystem declaration names its relaxation ticket. | Test (TC-1018) |
| FR-065-AC-14 | A case binding a variant module without naming a relaxation ticket is rejected. | Test (TC-1018) |
| FR-065-AC-15 | Two runs of one case over unchanged input produce byte-identical result records. | Test (TC-1019) |
| FR-065-AC-16 | Each case carries the invocation that reproduces it over its own `input/` directory. | Test (TC-1020) |
| FR-065-AC-17 | That documented invocation reproduces the case's result without the runner. | Test (TC-1020) |
| FR-065-AC-18 | The bounds enum resolves to one definition across both runners. | Test (TC-1021) |
| FR-065-AC-19 | The grading-ladder level names resolve to one definition across both runners. | Test (TC-1021) |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the coverage payload cases assert against), [FR-051](./FR-051-source-symbol-extraction.md) (the binding census the L2 level reads), [FR-055](./FR-055-published-output-contract.md) (the payload shape a case's `expect` is written against)
- **Downstream**: agent-ix/quire-rs#266 (the `qa-corpus` artifact), #267 (this crate's runner), #268 (the 16-mode inventory), agent-ix/quoin#227 (the sibling runner)
