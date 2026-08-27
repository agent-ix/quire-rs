---
id: FR-066
title: "Gap disposition census"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-063"
    type: "requires"
---
# FR-066: Gap disposition census

## Description

`quire-rs` SHALL provide a deterministic ecosystem census that partitions every
authored obligation row into exactly one actionable outcome. The census is an
analytical report, not a per-change gate: the controlled corpus remains the
gate, while the census sizes failure modes and routes each one to an owner.

### Four populations

The report SHALL keep these populations distinct and SHALL NOT calculate a
ratio by treating any two as interchangeable:

| Population | Unit | Meaning |
|---|---|---|
| P1 | evidence symbol | A symbol whose kind can carry a trace binding. |
| P2 | tag-shaped token | An authored id-shaped annotation, independent of whether the declared grammar binds it. |
| P3 | authored obligation row | A deduplicated row found by the engine-independent Markdown scan, excluding rows selected only by a target whose module-declared evidence posture is `reference-only`. |
| P4 | minted row | A row the active traceability declarations reached and minted. |

The disposition denominator SHALL be P3, not P4. P4 is instrument output; using
it as the denominator hides declaration failures inside the denominator. The
measured corpus made that error visible as roughly 26,357 authored rows against
20,148 minted rows: the difference is evidence to classify, not input to omit.

### Dispositions

Strict precedence applies and the first matching disposition wins:

1. `instrument-unread` — the row's language surface is covered by a
   `no-symbol-bound` or `low-symbol-binding` diagnostic. Owner: engine.
2. `declaration-unreached` — the row is in P3 but not P4, its id class has an
   active trace-target declaration, and that declaration did not reach the
   row's archetype, section, or id column. Owner: declaration or repository, as
   named by the diagnostic.
3. `marker-form-mismatch` — the row is minted and unbacked, and its id occurs in
   P2 but no declared tag form bound it. Owner: module declaration.
4. `id-class-unminted` — the row's id class has no active trace-target
   declaration. A matching `untracked_symbols` record supplies a source locus
   when one exists, but is not required: requiring a tag would classify the
   same declaration defect differently before and after a repository authored
   evidence. Owner: module declaration.
5. `method-exempt` — the row's declared verification method mints no source
   symbol and the payload reports it in `no_symbol_rows`. Owner: nobody; this is
   an explained absence rather than a gap.
6. `authoring-absent` — none of the prior conditions matches and the applicable
   binder read rate is above the declared observation floor. Owner: repository.

`status-lie` is an orthogonal overlay on a classified row. It SHALL NOT be a
seventh disposition because doing so would double-count rows and stop the
partition summing.

### Hard invariant and refusal

For every repository and for the aggregate, the census SHALL enforce:

```text
backed + instrument-unread + declaration-unreached + marker-form-mismatch
       + id-class-unminted + method-exempt + authoring-absent == authored_rows
```

A row matching no outcome, matching more than one outcome after precedence, or
violating the sum SHALL make the census exit non-zero. There is no residual or
`other` bucket.

The census SHALL also abort before publishing results when the engine provenance
envelope is absent, changes during a run, or omits any capability required by
the classification. The diagnostic SHALL name the missing capability token.

### Artifact and execution posture

One run SHALL emit byte-stable `reports/<date>-gap-census.json` and a human
rendering at `reports/<date>-gap-census.md`. Both headers SHALL carry the CLI and
engine versions, capability list, module commit, repository population
enumerated and scanned, and every exclusion. These values come from payloads
and resolved inputs, never from operator-authored version strings.

`make census` SHALL be invoked only by a scheduled or manually dispatched
workflow. It SHALL NOT be a push or pull-request prerequisite.

### Worked measured examples

| Disposition | Observed corpus shape | Routing |
|---|---|---|
| `instrument-unread` | `filament-ide-rs` offered 1,292 Rust evidence symbols and bound zero before the binding-census repair. | Engine: the binder could not read its input. |
| `declaration-unreached` | The pre-CR-117 scan found 3,514 TC-shaped ids behind unmatched matrix sections; changing the declaration later realised only +83 rows, proving the candidate population must remain visible and separately diagnosed. | Declaration or repository, according to the section/id-column diagnostic. |
| `marker-form-mismatch` | Legacy comma-list forms dropped 205 ids across 17 repositories while the rows were minted. | Module trace-tag grammar. |
| `id-class-unminted` | The #69 sweep found 572 untracked entries in id classes the model did not mint. | Module trace-target declaration. |
| `method-exempt` | Quoin's review found 40 of 55 apparent status lies were Eval/Manual/Inspection/Analysis rows whose method cannot mint a source symbol. | Nobody; report as explained absence. |
| `authoring-absent` | The measured corpus contained 82 repositories with 4,248 real tests and no authored trace-tag line. | Repository authoring backlog, never an instrument bucket. |

The vocabulary parallels the project-owned disposition shape in
`engineering-assurance/docs/structural-coverage.md`. This requirement does not
change that module's enum; convergence would require a separately specified
compatibility change in its owning repository.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-066-AC-1 | The report states P1, P2, P3 and P4 with distinct units and never substitutes one population for another. | Test (TC-1062) |
| FR-066-AC-2 | P3 authored rows are the denominator, scanned independently of engine minting and deduplicated by the shared corpus rules; rows selected only by a module-declared `reference-only` target are excluded because referenceability is not a source-evidence obligation. P4 and each row's backed state come from FR-050's `minted_targets` records and are cross-checked against the totals rather than inferred from `unbacked_rows`. | Test (TC-1063, TC-1073, TC-1075) |
| FR-066-AC-3 | Every row is assigned by strict first-match precedence to exactly one of the six named dispositions, and every disposition carries its declared owner. | Test (TC-1064) |
| FR-066-AC-4 | `backed + Σ dispositions == authored_rows` holds per repository and in aggregate; an unclassified row or sum mismatch exits non-zero and no residual bucket is emitted. | Test (TC-1065) |
| FR-066-AC-5 | `status-lie` is emitted only as an orthogonal overlay and never changes the partition sum. | Test (TC-1066) |
| FR-066-AC-6 | A repository with evidence symbols but no authored tags is classified as `authoring-absent` when binder readability is above the floor; the measured 82-repository class never enters `instrument-unread`. | Test (TC-1067) |
| FR-066-AC-7 | Missing provenance, an engine change during the run, or a missing required capability aborts the census; the error names the missing token. | Test (TC-1068) |
| FR-066-AC-8 | JSON and Markdown reports carry measured CLI/engine versions, capabilities, module commit, enumerated/scanned populations, and exclusions, and identical inputs produce byte-identical bytes. | Test (TC-1069) |
| FR-066-AC-9 | `make census` is exposed as a scheduled/manual operation and no push or pull-request workflow requires it. | Test (TC-1070) |
| FR-066-AC-10 | Each disposition has a worked example tied to the measured ecosystem corpus, and the human report names the repository, row/id when available, owner, reason and next action. | Test (TC-1071) |
| FR-066-AC-11 | The relationship to `structural-coverage.md` is recorded without changing the owning module's disposition enum. | Inspection (TC-1072) |

## Dependencies

- **Upstream:** [FR-050](./FR-050-declarative-coverage-computation.md) supplies
  coverage facts; [FR-063](./FR-063-metric-provenance-envelope.md) supplies the
  premise and provenance envelope.
- **Downstream:** `scripts/gap_census.py`, the scheduled/manual census workflow,
  and `agent-ix/quire-rs#277` implement this requirement.
