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
the clean case — 17 bound test cases, all in `tests/it/obligations.rs`, and nothing connects it to
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
   (`TestFunction`/`Benchmark`/`FuzzTarget`/`EntryPoint`); `implements` binds only production symbols
   (`Function`/`Container`).

Point 3 is what makes a mis-declared pattern harmless: an `implements` marker on a test binds
nothing, and a `trace` marker on a production function binds nothing. Getting it wrong yields *no*
relation rather than the *wrong* one.

Neither `ImplementsRelation` nor `VerifiesRelation` carries a `provenance` field: a relation does not
record how it was written (FR-051-CON-3).

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

When a guarded form's pattern matches a proper substring of an eligible
attribute but not its complete source text, the binder SHALL produce one
`partial-implements-attribute-match` finding for that form/candidate pair
in `CoverageReport.diagnostics`, with FR-057 severity `warning`. The finding
SHALL name the configured form, file, 1-based candidate start line,
qualified owning symbol and verbatim complete attribute text. The partial
match SHALL create no relation, even when another candidate matches fully.
A pattern with no match at all SHALL produce no finding. In particular,
unrelated `derive`, `inline` and `must_use` attributes without an implements
match SHALL produce neither a guarded relation nor this finding. The binder
SHALL NOT fall back to attached-source matching after a rejected partial
match; these findings are distinct from module-load refusals.

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

## Declared Python and TypeScript syntax positions (PLAT-866)

Rust production code declares scope with a language construct, `#[implements]`
(`ix-trace-rs`), read from the syntax tree by the guard above. Python and
TypeScript production code declares it only as a comment, which the binder matches
as text over the attached annotation block, so a wrapped line, a changed separator
or an example in prose binds or fails to bind by page position. This section gives
both languages a structural construct, read the way the Rust attribute is.

A module MAY add a `syntax` to a form in `trace_tags.implements`. Three values are
added to the existing `rust_attribute`:

| `syntax` | Language | Position the binder reads |
|---|---|---|
| `python_decorator` | Python | a `decorator` node directly attached to a `def`, `async def` or `class` |
| `typescript_decorator` | TypeScript | a decorator directly attached to a class declaration or a method definition |
| `typescript_wrapper` | TypeScript | the callee call of a wrapper chain that is the initializer of a `const`, `let` or `var` declarator |

A form that declares no `syntax` SHALL keep its declared-text behaviour. The
`syntax` field SHALL select a syntax position only: it SHALL NOT select, replace or
broaden the module's `pattern` or `template`, and the engine SHALL NOT supply an
`implements` pattern (FR-062-CON-2).

**Candidate text.** The candidate SHALL be the exact source text of the node, with
its internal whitespace, line breaks and argument spelling, and the binder SHALL
apply the form's pattern to that text and to nothing else. For `python_decorator`
and `typescript_decorator` the candidate SHALL be the complete decorator node
including its `@`. For `typescript_wrapper` it SHALL be the callee call of the
wrapper, `implementing("FR-007-AC-2")`, and not the call that applies it nor the
wrapped function. A comment, a docstring, a string or template literal and an
undecorated line are never nodes of these kinds, so text in them SHALL bind
nothing, and the binder SHALL NOT scan them with a second lexer.

**Spellings.** The binder SHALL resolve no import and no alias: which spelling of
the call binds is exactly what the module's declared `pattern` matches against the
candidate text. A renamed import (`import ix_trace_py as t`, `import {
implementing as impl }`) therefore binds nothing unless a module pattern names it,
and yields no finding because it matches no pattern; a module pattern SHALL be anchored at the start of the candidate, because an unanchored one would find `implementing("X")` inside `t.implementing("X")` and raise a partial-match finding. The shipped patterns accept
the unqualified spelling and the package-qualified one (`ix_trace_py.implements`,
`ixTrace.implementing`), as `spec-artifacts-process` FR-004 pins them.

**Attachment.** The binder SHALL attach a decorator to the one declaration it
decorates by syntax node and not by line span. A decorator SHALL bind its own
declaration only; a class decorator SHALL bind the class container and none of its
methods; a decorated nested declaration SHALL bind neither its parent nor its
siblings. The tree-sitter TypeScript grammar places a method's decorators as
siblings that precede the `method_definition` inside the class body and not as its
children, so the binder SHALL read the run of decorators immediately preceding the
method, and SHALL read a class declaration's own decorators. The production/evidence
complement SHALL stay in force: a form of any of these syntaxes on a test,
benchmark, fuzz target or entry point binds nothing.

**What a TypeScript decorator reaches.** A decorator is eligible only on a class
declaration and on a method definition, the two TypeScript constructs the adapter
mints as production symbols that admit one. A decorator on a class expression, a
property, an accessor or a parameter, and on an `abstract` method signature (which
has no body, takes no decorator in the language and mints no symbol), SHALL bind
nothing and SHALL yield no finding: the extractor mints no symbol for it, so no
candidate exists to report. A TypeScript `function` declaration admits no decorator
and has no structural construct; its scope is declared by writing the function as a
binding that the wrapper form wraps. This is a stated limit and not a gap the
binder papers over.

**The wrapper and the symbol it binds.** A **wrapper chain** is a curried call
`callee(...)(argument)` whose single argument is an arrow function, a function
expression, or another wrapper chain. FR-051-AC-51 states that a `const`, `let` or
`var` declarator whose initializer is a wrapper chain mints a Function symbol named
by the declarator, which is the symbol a `typescript_wrapper` form binds; `export
const` and a declarator inside a multi-declarator statement mint it the same way.
The binder SHALL apply the form's pattern to the callee call of every link of the
chain, so `implementing(A)(implementing(B)(fn))` binds the ids of both links to the
one symbol.

**A wrapper that attaches to nothing.** A call whose callee call matches a
`typescript_wrapper` form SHALL bind nothing, and the binder SHALL yield one
`implements-syntax-unattached` finding at FR-057 severity `warning`, naming the form,
the file, the 1-based line of the candidate and its verbatim text, when that call is
not a link of a wrapper chain that initializes a declarator. That covers a bare
expression statement, an argument of another call
(`memoize(implementing(A)(fn))`), a default export, a chain whose argument is
neither function-valued nor another chain (`implementing(A)(memoize(fn))`), and a
declarator whose initializer is not a chain. The same call written as a decorator,
which the grammar attaches to a class or method or drops as above, SHALL yield no
`implements-syntax-unattached` finding.

**Partial matches.** When a form's pattern matches a proper substring of a candidate
but not all of it, the binder SHALL produce one `partial-implements-syntax-match`
finding at `warning` severity, naming the form, file, 1-based candidate line,
qualified owning symbol and the verbatim candidate, and SHALL create no relation,
with no fallback to attached-source matching. A candidate the pattern does not match
at all, such as an unrelated decorator (`@staticmethod`, `@dataclass`,
`@Injectable()`), SHALL yield neither a relation nor this finding.

**Load.** The module loader SHALL reject an unknown `syntax` value, a syntax on a
form of another language, and any of these three outside `trace_tags.implements`, as
invalid module input naming the offending form, and SHALL NOT drop the guard and
fall back to text matching.

**The comment forms are replaced, not kept beside.** Both languages are prerelease,
so the end state is one construct per language. A module whose Python or TypeScript
`implements` forms declare only structural syntaxes has no textual form to read, so
`# Implements:` and `// Implements:` SHALL bind nothing. The FR-051 advisory SHALL
still report such a comment id on a production function or container, naming the
replacement text of FR-051-AC-22 (the structural form of the symbol's language and
position). The engine SHALL accept the new syntax values in a release before the
module that declares them is released.

**Runtime constructs are another repository's.** The decorator a Python author writes
and the function a TypeScript author calls exist at run time and are specified by
`ix-trace-py` and `ix-trace-ts`; this requirement specifies the static parse
contract the binder holds them to.

> **PLAT-866 reconciliation (FR-051-AC-22).** quire-rs#627 narrows FR-051-AC-22 to
> the symbol's own language and renders the replacement from a `rust_attribute`
> form only, so a Python or TypeScript symbol is named by kind alone. This change
> keeps that language filter and widens the rendering to the structural form of the
> symbol's language and position.

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
| FR-062-AC-12 | A module-loaded Rust form retains `syntax: rust_attribute` through the traceability model and serialization; omission preserves the old declared-text behavior. A partial attribute match produces no relation and one `partial-implements-attribute-match` warning in `CoverageReport.diagnostics`, naming the form, file, 1-based candidate start line, qualified symbol and complete attribute text; no-match and unrelated-attribute cases produce no such finding. An unknown syntax value or this guard on a non-Rust form or outside `trace_tags.implements` is refused at module load, identifying the form, with no text-matching fallback. | Test (TC-1983, TC-1989) |
| FR-062-AC-13 | A real production attribute carrying two declared ids produces exactly those two `implements` relations and no `verifies` relation. Valid leading, multiline, trailing-comment and same-line-item attributes are eligible through the same module-loaded form; each form matches only the exact attribute-node text. | Test (TC-1984, TC-1986) |
| FR-062-AC-14 | Attribute-shaped text in doc, line, block or nested block comments, ordinary or byte strings, and single-line or multiline raw strings produces no guarded attribute-form relation, including a column-zero raw-string line; the matching real attribute control remains bound. A substring inside another attribute payload creates no relation. | Test (TC-1985) |
| FR-062-AC-15 | Guarded attributes bind only to their actual production declaration: a nested declaration's attribute binds neither parent nor sibling, and an attribute on an evidence symbol produces no `implements` relation. Field, variant, statement, expression and closure attributes do not bind to their enclosing symbol. Inner attributes, macro-token attributes and recovered-error attributes do not bind; cfg_attr payloads remain unexpanded data rather than nested candidates. Same-line siblings retain distinct owners. | Test (TC-1987) |
| FR-062-AC-16 | Unguarded Rust comment forms and Python/TypeScript declared comment forms retain their existing relations; syntax-guard adoption supplies no implicit pattern and does not change the module's declared ids or templates. | Test (TC-1988) |
| FR-062-AC-17 | Guarded binding preserves symbol identity, actual finding source locations, existing range refusal and per-file syntax-failure behavior, and deterministic relation/diagnostic ordering. It changes no `verifies` edge or coverage backing count. | Test (TC-1990) |
| FR-062-AC-18 | A module-loaded Python or TypeScript `implements` form retains `syntax: python_decorator`, `typescript_decorator` or `typescript_wrapper` through the traceability model and serialization; omission preserves the declared-text behaviour. An unknown syntax value, a syntax on a form of another language, and any of the three on a `trace_tags.markers` form are refused at module load, each identifying the form, with no text-matching fallback. | Test (TC-2200) |
| FR-062-AC-19 | A Python production function, method or class carrying a decorator that a declared `python_decorator` form matches, with two declared ids, mints exactly those two `implements` relations and no `verifies` relation; a decorator a formatter has wrapped across lines, one stacked with other decorators in any order, and one on an `async def` each bind exactly as the single-line adjacent spelling does. The candidate is the exact decorator node text, so a pattern matching only a substring of a decorator's payload binds nothing, and a renamed import binds nothing unless the module pattern names it. | Test (TC-2201) |
| FR-062-AC-20 | Decorator-shaped text in a Python comment, docstring or string literal binds nothing and the real decorator control binds. A class decorator binds the class container and none of its methods; a decorated nested function binds neither its parent nor its siblings; the same decorator on a `test_` function binds nothing, and on a `@pytest.fixture` function binds, because a fixture is a production Function symbol. | Test (TC-2202) |
| FR-062-AC-21 | A TypeScript class declaration and a method definition (including `static` and `async`), each carrying a decorator that a declared `typescript_decorator` form matches, mint the declared `implements` relations and no `verifies` relation, including a decorator written before or after `export` and one wrapped across lines. A decorator on a class expression, a property, an accessor, a parameter or an `abstract` method signature binds nothing and yields no finding. | Test (TC-2203) |
| FR-062-AC-22 | Decorator-shaped text in a TypeScript comment, string literal or template literal binds nothing, and the real decorator control binds; a class decorator binds the class and none of its methods, and a decorator on a method binds only that method. A class declared inside a `describe` callback binds its decorators as any class does, and the `describe` container itself binds nothing. | Test (TC-2204) |
| FR-062-AC-23 | A `const`, `let` or `var` declarator whose initializer is a wrapper chain mints a Function symbol (FR-051-AC-51), and a declared `typescript_wrapper` form binds that symbol, as `export const` and a declarator of a multi-declarator statement do; the candidate is the callee call of each link and not the wrapped function. `implementing(A)(implementing(B)(fn))` binds the ids of both links to the one symbol. A `function` declaration binds through no TypeScript structural form. | Test (TC-2205) |
| FR-062-AC-24 | A call whose callee call matches a `typescript_wrapper` form but which is a bare statement, an argument of another call (`memoize(implementing(A)(fn))`), a default export, a chain whose argument is neither function-valued nor a chain (`implementing(A)(memoize(fn))`), or the initializer of a declarator that is not a chain, binds nothing and yields one `implements-syntax-unattached` warning in `CoverageReport.diagnostics` naming the form, file, 1-based line and verbatim candidate; the attached control yields none, and a decorator form yields none. | Test (TC-2206) |
| FR-062-AC-25 | A pattern matching a proper substring of a decorator or wrapper candidate yields no relation and one `partial-implements-syntax-match` warning naming the form, file, 1-based candidate line, qualified symbol and verbatim candidate, even where another candidate matches fully; a candidate the pattern does not match at all, including `@staticmethod`, `@dataclass` and `@Injectable()`, yields neither a relation nor this finding. | Test (TC-2207) |
| FR-062-AC-26 | Under a module whose Python and TypeScript `implements` forms are structural only, `# Implements: FR-007-AC-2` and `// Implements: FR-007-AC-2` on a production function bind nothing, and the FR-051 advisory reports each under FR-051-AC-22 naming the text rendered from the declared structural form of that symbol's language and position that declares a `template`: `@implements("FR-007-AC-2")` for a Python function, method or class, `@implementing("FR-007-AC-2")` for a TypeScript class or method, and `implementing("FR-007-AC-2")` for a `const`-bound TypeScript function. A TypeScript `function` declaration, and a symbol whose language declares no structural templated form, is named by kind alone. A module that still declares a textual form for the language binds that comment as before. | Test (TC-2208) |
| FR-062-AC-27 | Structural forms preserve symbol identities, the existing range refusal for an argument carrying a `..` range, per-file parse-failure isolation and deterministic relation and diagnostic ordering, and move no `verifies` edge and no coverage backing count. | Test (TC-2209) |

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
| FR-062-CON-5 | The binder SHALL obtain decorator, wrapper and declaration association for Python and TypeScript exclusively through those source adapters' syntax interpretation, without a second comment/string lexer, import resolution or code execution. | Design | Inspection (TC-2209) |
| FR-062-CON-3 | `implements` SHALL NOT reuse `VerifiesRelation`. Two questions, two types, and no field a typo can flip between them. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (the marker grammar this extends), [FR-045](./FR-045-filament-core-extraction-engine.md) (symbol identity)
- **Downstream**: `agent-ix/ix-trace-py` FR-002 and `agent-ix/ix-trace-ts` FR-002 — the runtime constructs the Python and TypeScript forms bind; `agent-ix/spec-artifacts-process` FR-004 — the module declaration of the forms
- **Downstream**: `agent-ix/quoin` FR-039 — mutation scoping, which needs a requirement's production files to be more than an accident of test placement
