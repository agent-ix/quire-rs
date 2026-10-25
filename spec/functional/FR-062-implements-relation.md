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

## Declared Rust attribute position (CR-191, PLAT-1181)

A module MAY add `syntax: rust_attribute` to a Rust form in
`trace_tags.implements`. When the field is absent, the form SHALL retain
its existing declared-text matching behavior. The field selects a syntax
position; it SHALL NOT select, replace or broaden the module's `pattern`
or `template`, and the engine SHALL NOT supply an `implements` pattern.

When a form selects `rust_attribute`, the binder SHALL apply that form's
pattern only to real Rust `attribute_item` syntax attached to the selected
production declaration. It SHALL use the source adapter's Rust syntax
interpretation, not a second comment/string lexer. The candidate text is
that attribute node's exact source text, including its internal whitespace
and argument spelling, without the declaration or following comment. A
successful guarded match SHALL cover the complete attribute candidate;
matching a substring inside a different attribute's payload SHALL produce
no relation. The binder SHALL present the attribute at the start of its candidate
text, including when the declaration shares the attribute's source line. The engine SHALL
retain the attribute's actual source location when reporting a finding.

The binder SHALL treat an attribute's comment or string-shaped contents as
data within that attribute, not additional candidate attributes. A matching example inside
a doc, line or block comment, an ordinary string, a byte string, or a raw
string SHALL produce no attribute-form relation. This includes a multiline
raw string with an apparent attribute starting at column zero. Leading
attributes of a nested declaration SHALL belong only to that declaration;
they SHALL NOT bind to its enclosing declaration or siblings. The existing
production/evidence symbol-kind complement remains in force.

Only an outer attribute directly attached to an extracted production
Function or Container declaration SHALL be eligible. Attributes on struct
fields, enum variants, statements, expressions or closures SHALL NOT bind
to the enclosing symbol. Ownership SHALL use syntax-node association rather
than overlapping source-line spans, including same-line sibling declarations.
Inner `#![...]` attributes SHALL NOT be eligible. The binder SHALL NOT expand
macros or `cfg_attr`: attributes represented only as tokens inside a macro
invocation (including `proptest!`), and nested attributes in a `cfg_attr`
payload, SHALL NOT become candidates. An outer `cfg_attr` itself remains an
ordinary complete candidate for an explicitly matching module form. An
attribute in an adapter error-recovery node SHALL NOT bind; a file rejected
by the adapter SHALL retain the per-file failure rule below.

For each guarded form and source file with eligible attribute candidates,
the binder SHALL emit a diagnostic naming the form and candidate source
locations when no candidate matches completely. A partial match SHALL emit
a diagnostic at that candidate even if another candidate matches completely;
it SHALL create no relation. No eligible candidates SHALL produce no such
unmatched-form diagnostic. These diagnostics SHALL remain distinct from
module-load refusals and SHALL NOT fall back to attached-source matching.

The module loader SHALL reject an unknown `syntax` value and a
`rust_attribute` guard on a non-Rust form or outside
`trace_tags.implements` as invalid module input, naming
the offending form. It SHALL NOT drop the guard and fall back to text
matching. A source file whose declaration structure the Rust adapter
cannot resolve SHALL retain FR-051's per-file failure behavior, with no
synthetic attribute relation created from the rejected file.

The change SHALL preserve the existing symbol identities, input source,
declaration order, unguarded forms, marker id/range rules, and the
separation between `implements` scope and `verifies` evidence. In
particular, a syntax guard SHALL NOT turn a production relation into
coverage backing. Repeated runs over identical source and model inputs
SHALL preserve deterministic relation and diagnostic ordering.

> **CR-191 (syntax positions, not regex examples):** the former binder
> applied an `implements` form to the complete attached source span, so a
> module could not distinguish a real Rust attribute from an example in a
> block comment or column-zero raw-string line. PLAT-1181 adds an explicit,
> optional module-declared syntax guard. The absence default preserves
> existing comment forms; adopting the guard deliberately removes false
> scope relations. It changes no acceptance obligation or evidence-method
> semantics and does not itself qualify any release or consumer adoption.

## Acceptance Criteria

AC-6..AC-11 are intentionally unused here: historical measurement artifacts
already mention those ids for another corpus. The coordinated PLAT-1181
reservation starts at AC-12 to avoid ambiguous reuse; it does not retire or
waive any normative criterion.

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-062-AC-1 | A production symbol carrying a declared `implements` marker mints an `implements` relation, and that relation backs no trace id. | Test (TC-936) |
| FR-062-AC-2 | An `implements` marker on an evidence symbol, and a `verifies` marker on a production symbol, each bind nothing — the kinds are complements. | Test (TC-937) |
| FR-062-AC-3 | A requirement named by several markers yields one relation, ordered deterministically. | Test (TC-938) |
| FR-062-AC-4 | The relation reaches the `coverage --json` contract, and moves no coverage number: not `backed`, not `totals`, not `untracked_symbols`. | Test (TC-939) |
| FR-062-AC-5 | Marker forms declared in a module manifest survive the load: the merged model carries them, a model declaring only them does not read as empty, and `bind` mints from them. | Test (TC-940) |
| FR-062-AC-12 | A module-loaded Rust form retains `syntax: rust_attribute` through the traceability model and serialization; omission preserves the old declared-text behavior. For eligible candidates, partial matches diagnose the candidate without binding, and zero complete matches diagnose the form and candidate locations; a file with no candidates produces no unmatched-form diagnostic. An unknown syntax value or this guard on a non-Rust form or outside `trace_tags.implements` is refused at module load, identifying the form, with no text-matching fallback. | Test (TC-1983, TC-1989) |
| FR-062-AC-13 | A real production attribute carrying two declared ids produces exactly those two `implements` relations and no `verifies` relation. Valid leading, multiline, trailing-comment and same-line-item attributes are eligible through the same module-loaded form; each form matches only the exact attribute-node text. | Test (TC-1984, TC-1986) |
| FR-062-AC-14 | Attribute-shaped text in doc, line, block or nested block comments, ordinary or byte strings, and single-line or multiline raw strings produces no guarded attribute-form relation, including a column-zero raw-string line; the matching real attribute control remains bound. A substring inside another attribute payload creates no relation. | Test (TC-1985) |
| FR-062-AC-15 | Guarded attributes bind only to their actual production declaration: a nested declaration's attribute binds neither parent nor sibling, and an attribute on an evidence symbol produces no `implements` relation. Field, variant, statement, expression and closure attributes do not bind to their enclosing symbol. Inner attributes, macro-token attributes and recovered-error attributes do not bind; cfg_attr payloads remain unexpanded data rather than nested candidates. Same-line siblings retain distinct owners. | Test (TC-1987) |
| FR-062-AC-16 | Unguarded Rust comment forms and Python/TypeScript declared comment forms retain their existing relations; syntax-guard adoption supplies no implicit pattern and does not change the module's declared ids or templates. | Test (TC-1988) |
| FR-062-AC-17 | Guarded binding preserves symbol identity, actual finding source locations, existing range refusal and per-file syntax-failure behavior, and deterministic relation/diagnostic ordering. It changes no `verifies` edge or coverage backing count. | Test (TC-1990) |

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
>
> **Review follow-up (SR).** The first draft applied the `src/` partition to the
> `verifies` half only, so an `Implements:` marker on a function in `tests/` was
> handed straight to `cargo mutants --file` and the test code mutated. The
> engine cannot prevent it — `implements` binds any `Function`/`Container`, and a
> helper in `tests/` is a `Function` — so the partition applies to **both**
> edges: a `tests/` file is never a mutable target however it was reached. The
> mis-annotation lands in `evidence-only` with a note naming the mistake instead
> of being silently obeyed. A second defect in the same block: the evidence-only
> report indexed the `verifies` map directly, so a declared-only path there
> panicked. Neither changes the 55-of-58 figure.

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
| FR-062-CON-4 | The binder SHALL obtain guarded attribute positions and declaration association exclusively through the source adapter's Rust syntax interpretation, without a second comment/string lexer or macro expansion pipeline. | Design | Inspection |
| FR-062-CON-3 | `implements` SHALL NOT reuse `VerifiesRelation`. Two questions, two types, and no field a typo can flip between them. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (the marker grammar this extends), [FR-045](./FR-045-filament-core-extraction-engine.md) (symbol identity)
- **Downstream**: `agent-ix/quoin` FR-039 — mutation scoping, which needs a requirement's production files to be more than an accident of test placement
