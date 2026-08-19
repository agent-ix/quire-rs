---
id: FR-062
title: "The requirement-to-production-code relation"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/FR-051"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/StR-001"
    type: "traces_to"
---

# FR-062: The requirement-to-production-code relation

## Description

`FR-051` mints `verifies`, linking an **evidence symbol** — a test, a benchmark, a fuzz target — to
a spec trace id. Nothing linked a **requirement to the production code that implements it**.

That gap is measurable. Across this repository's 52 FRs, **38 have at least one mutable target and 14
have none** (CR-071). The fourteen fail for one reason: every symbol verifying them lives in
`tests/`, which holds test code only, so a computed file set names no production file. `FR-053` is
the clean case — 17 bound test cases, all in `tests/obligations.rs`, and nothing connects it to
`src/obligation.rs`.

**Reach therefore correlates with test placement, not with requirement quality**, and inversely with
recency: the FRs whose tests are `#[cfg(test)]` modules inside the production file work by accident,
while the newer integration-tested work does not. Mutation scoping over that set measures where tests
live.

### Two relations, and no path from one to the other

This is the constraint the whole requirement exists to preserve.

| Relation | Means | May back an acceptance criterion |
|---|---|---|
| `verifies` | *"this test would fail if the behaviour broke"* | **yes** — it is evidence |
| `implements` | *"this code is what the requirement is about"* | **never** — it is scope |

`CR-061` stopped `verifies` binding production symbols precisely because a doc comment in
`src/foo.rs` that merely *cites* `FR-053-AC-1` would otherwise count as evidence backing it, letting
unverified code claim coverage. **Widening `verifies` was the wrong fix, and so is a shared type with
a discriminator** — that puts one typo between scope and evidence.

### The separation is structural, not conventional

Three independent things keep them apart, and none of them is a naming convention:

1. **A separate declared list.** `trace_tags.implements`, not a flag on `trace_tags.markers`.
2. **A separate relation type.** `ImplementsRelation`, not `VerifiesRelation` with a field.
3. **Complementary symbol kinds.** `verifies` binds only evidence symbols
   (`TestFunction`/`Benchmark`/`FuzzTarget`); `implements` binds only production symbols
   (`Function`/`Container`).

Point 3 is what makes a mis-declared pattern harmless: an `implements` marker on a test binds
nothing, and a `trace` marker on a production function binds nothing. Getting it wrong yields *no*
relation rather than the *wrong* one.

`ImplementsRelation` carries no `provenance`, because there is no legacy `implements` form to migrate
from and inventing the field would suggest one exists.

### It answers more than mutation scoping

Mutation scoping is the case that surfaced it, but the same edge answers *"which requirements does
this diff touch"* — the input to scoped CI, impact analysis and review routing.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-062-AC-1 | A production symbol carrying a declared `implements` marker mints an `implements` relation, and that relation backs no trace id. | Test (TC-936) |
| FR-062-AC-2 | An `implements` marker on an evidence symbol, and a `verifies` marker on a production symbol, each bind nothing — the kinds are complements. | Test (TC-937) |
| FR-062-AC-3 | A requirement named by several markers yields one relation, ordered deterministically. | Test (TC-938) |
| FR-062-AC-4 | The relation reaches the `coverage --json` contract, and moves no coverage number: not `backed`, not `totals`, not `untracked_symbols`. | Test (TC-939) |
| FR-062-AC-5 | Marker forms declared in a module manifest survive the load: the merged model carries them, a model declaring only them does not read as empty, and `bind` mints from them. | Test (TC-940) |

> **CR-081 (module-declared forms survive the load — 2026-08-19):** the relation
> was inert wherever it was actually used. `trace_tags.implements` was added to
> `TraceabilityModel` and to neither of the two **hand-maintained per-field
> functions** that model passes through — `merge_traceability`, which combines
> the per-module models, and `is_empty`, which decides whether a model counts as
> declared at all. Neither is an exhaustive `match`, so the compiler said
> nothing.
>
> The consequence, stated plainly: a repository whose production code was
> correctly annotated and whose module correctly declared the forms got an empty
> `implements` array from `quire coverage`, with no error and no diagnostic.
>
> **Every one of TC-936..TC-939 passed throughout**, because each builds the
> model with `TraceabilityModel::default()` and pushes forms onto it — the
> in-memory shape, never the load path. That is the same gap CR-076 and CR-080
> each closed one layer at a time: minted but not exposed, then exposed but not
> loadable.
>
> TC-905 exists to catch precisely this ("a field added to the model and to
> nothing else fails here") and did not, because its assertions are hand-listed
> too. It now names `trace_tags.implements`, and AC-5 adds the load-path test the
> other four bypass.

> **CR-082 (the relation is adopted, and what it bought — 2026-08-19):** the
> relation existing changes nothing until production code carries the marker, so
> this crate annotates its own.
>
> **What the number counts.** One row per functional requirement in
> `spec/functional/` — 58 of them — asking a single question: does
> `mutants_scope` name at least one file under `src/` that `cargo mutants --file`
> could mutate for that requirement? Not how many files, not how good the tests
> are. Just: is there anything to mutate at all.
>
> | | FRs with a mutable target |
> |---|---|
> | before, scoping by `verifies` alone | **40 of 58** |
> | after, 15 production symbols annotated | **55 of 58** |
>
> The earlier figure recorded on this programme was 38 of 52; the population has
> grown by six requirements since, which is why the baseline is re-derived here
> rather than quoted.
>
> **The three that remain are correct, not missed.** FR-004 (strict minijinja
> environment) and FR-012 (render-parity harness) describe the **render layer,
> which this crate removed** — there is no production code to mutate and
> inventing an annotation would point mutation testing at something else.
> FR-055's contract is **authored JSON schemas** under `schemas/`, and its own
> CON-1 requires they be authored rather than generated, so there is no Rust
> behind it either. A requirement about data has no mutable target and should
> not be given a fake one.
>
> **`mutants_scope` unions the two edges rather than replacing one with the
> other.** An annotated requirement whose tests are also co-located should mutate
> both files; dropping the inferred half the moment one marker appeared would
> make annotating a requirement *narrow* its own scope. The report names which
> edge produced each path, so a reader can tell a stated scope from an inferred
> one.
>
> Measured after annotating: 15 `implements` edges, `totals.backed` unchanged at
> **483** and `untracked_symbols` unchanged at **15** — CON-1 holds under real
> adoption, not only in the fixture.

### Minting it is not the same as exposing it

`FR-061` shipped a combinatorial branch that existed only on the single-document
path, so `quire coverage` — the surface every consumer reads — never carried it
(CR-076). A relation in `SymbolGraph` that no consumer can see is a capability
nothing reaches.

So the report carries `implements` as its own array, additive on the published
v1 contract and absent for a module declaring no marker forms. It is **not**
folded into `backed`, `totals` or `untracked_symbols`: an `implements` edge is
not a trace tag pointing at nothing, and counting it anywhere would let
production code that merely cites a requirement move a coverage number.

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-062-CON-1 | `implements` SHALL NOT contribute to `backed_trace_ids`, nor to any FR-050 rollup figure. It is scope, and counting it as evidence is the backdoor CR-061 closed. | Design | Test (TC-936) |
| FR-062-CON-2 | The engine SHALL NOT declare a marker pattern. The forms are module data, exactly as `verifies` forms are. | Design | Inspection |
| FR-062-CON-3 | `implements` SHALL NOT reuse `VerifiesRelation`. Two questions, two types, and no field a typo can flip between them. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (the marker grammar this extends), [FR-045](./FR-045-filament-core-extraction-engine.md) (symbol identity)
- **Downstream**: `agent-ix/quoin` FR-039 — mutation scoping, which needs a requirement's production files to be more than an accident of test placement
