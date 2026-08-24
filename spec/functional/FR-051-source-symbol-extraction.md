---
id: FR-051
title: "Source Symbol Extraction with Relations"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-045"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-016"
    type: "references"
---
# FR-051: Source Symbol Extraction with Relations

## Description

`quire-rs` SHALL ship a deterministic **source-symbol extractor**: given a
source tree, it returns the tree's symbols with stable identities and typed
**relations**, so the same extraction feeds three consumers — the coverage
rollup ([FR-050](./FR-050-declarative-coverage-computation.md)), semantic
review, and knowledge-graph ingestion.

## Symbols

The extractor SHALL extract, per file, the symbols visible at syntax level:
functions, test functions, and containers (Rust `struct`/`enum`/`trait`/`mod`,
Python classes and modules, TypeScript classes and modules). Each symbol SHALL
carry a **stable identity** composed of language, repo-relative file path,
qualified symbol path, and kind. Symbol identity SHALL NOT incorporate line
numbers, byte offsets, or formatting, so reformatting a file leaves every
identity unchanged; the current line number SHALL be carried as a non-identity
attribute. Record ids SHALL be stable SHA-256 digests of the identity, per the
[FR-045](./FR-045-filament-core-extraction-engine.md) record-id convention.

## Language adapters

The extractor SHALL ship per-language adapters for Rust, Python, and
TypeScript. Adapters SHALL operate at syntax level: no build, no type
resolution, no dependency installation. Adapters SHALL classify test functions
by each language's convention: Rust functions under a `#[test]`-family
attribute, Python `test_`-prefixed functions and test-class methods, and
TypeScript `test(...)`/`it(...)` registrations (the registered title is the
symbol's qualified name). If an adapter cannot parse a file, then the extractor
SHALL emit a per-file diagnostic, skip the file, and continue.

## Trace-tag grammar

The extractor SHALL bind symbols to spec trace ids via a **trace-tag grammar**
declared as module data in the `traceability:` model
([FR-050](./FR-050-declarative-coverage-computation.md)) — the engine SHALL
carry no hardcoded tag forms.

**Framework-native markers are the canonical trace form.** A canonical marker
is a per-language, statically parseable construct attached to the test symbol
that carries one or more trace ids; the exact names and syntax are
module-declared data, with these forms as the intended ISO declaration
(a follow-up change in `spec-artifacts-iso`):

- Python — a pytest marker: `@pytest.mark.trace("FR-007-AC-01", "TC-041")`;
- Rust — a no-op proc-macro attribute from a lightweight support crate:
  `#[trace("FR-007-AC-01")]`;
- TypeScript — a vitest/jest helper or tag-metadata form:
  `trace("FR-007-AC-01")` wrapping or annotating the registration.

The extractor SHALL parse markers **statically** from source (decorators,
attributes, and call metadata on the test symbol) — coverage never requires a
runtime. Runtime queryability (running all tests for an FR, tagging JUnit
reports) is a stated benefit of the marker form, not a requirement of this FR.
Each trace id attached by a marker SHALL mint one `verifies` relation from the
test symbol to that id; a trace id attached more than once to one symbol
(repeated marker, or marker plus legacy tag) SHALL mint one relation and a
diagnostic, per the FR-045 edge-dedup convention.

The textual forms the `gap-analysis` workflow greps today are a recognized
**legacy class**, read only during migration: a bare trace id in a doc comment
or docstring (`FR-007-AC-01`, `TC-041`), a `Trace:` line (`Trace: FR-001`), a
trace id in a line comment (`# TC-041`, `// TC-041`), and a test name embedding
a trace id (`tc657_classification`). A legacy binding SHALL be marked `legacy`
in the relation's provenance metadata, and the extractor SHALL emit a
mechanical marker-rewrite suggestion (`quire fix`-style) where the equivalent
marker is derivable.

## Outputs

The extractor SHALL emit each symbol and relation as records aligned with the
Filament extraction contract ([FR-045](./FR-045-filament-core-extraction-engine.md)):
symbols as graph-node records and relations as graph-edge records
(`verifies` symbol→trace-id, `defined_in` symbol→file, `contains`
container→member), with `ref` values normalized under the caller-supplied
org/repo per FR-045-CON-4, so filament-core can ingest the symbol graph
through its existing pipeline. The extractor SHALL also expose the compact
in-process form [FR-050](./FR-050-declarative-coverage-computation.md)
consumes. Repeated extraction over an identical tree SHALL produce
byte-identical JSON ordering and stable record ids.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-051-CON-1 | The extractor SHALL NOT perform network I/O, service I/O, or extracted-code execution. | Architecture | Test |
| FR-051-CON-2 | Adapters SHALL degrade per file: one unparseable file never aborts the tree extraction. | Operational | Test |
| FR-051-CON-3 | Legacy textual-tag recognition SHALL be removed once the marker-normalization sweep lands (sweep gated on explicit user sign-off) — no deprecated compat path is retained after it. | Operational | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-051-AC-1 | Each adapter extracts functions, test functions, and containers from a fixture tree, and each symbol carries language, repo-relative path, qualified path, kind, and a line attribute. | Test (TC-741) |
| FR-051-AC-2 | Reformatting a fixture file (whitespace and line-number changes only) leaves every symbol id unchanged; renaming a symbol changes only that symbol's id. | Test (TC-742) |
| FR-051-AC-3 | Rust `#[test]`-family functions, Python `test_` functions, and TypeScript `test`/`it` registrations classify as test symbols; sibling non-test symbols do not. | Test (TC-743) |
| FR-051-AC-4 | Each canonical marker form binds statically: a Python `@pytest.mark.trace(...)` decorator, a Rust `#[trace(...)]` attribute, and a TypeScript `trace(...)` helper each mint one `verifies` relation per attached trace id, with no code executed. | Test (TC-744) |
| FR-051-AC-5 | Marker and tag forms are module data: a fixture model declaring different marker names/patterns binds by its own declaration, and with no declared forms the extractor mints zero `verifies` relations. | Test (TC-745) |
| FR-051-AC-6 | A trace id attached more than once to one symbol (repeated marker, or marker plus legacy tag) mints one `verifies` relation and one diagnostic. | Test (TC-746) |
| FR-051-AC-7 | The emitted records match the FR-045 graph-record shapes with normalized `ref` values, and filament-core ingestion fixtures accept them unchanged. | Test (TC-747) |
| FR-051-AC-8 | `defined_in` edges link every symbol to its file and `contains` edges link containers to members, deterministically ordered. | Test (TC-748) |
| FR-051-AC-9 | An unparseable fixture file yields a per-file diagnostic while the rest of the tree extracts normally. | Test (TC-749) |
| FR-051-AC-10 | Repeated extraction over an identical fixture tree emits byte-identical JSON and identical record ids. | Test (TC-750) |
| FR-051-AC-11 | The legacy textual forms (docstring bare id, `Trace:` line, line-comment id, trace-embedding test name) still bind during migration, carry `legacy` provenance on the minted relation, and yield a mechanical marker-rewrite suggestion where derivable. | Test (TC-753) |
| FR-051-AC-12 | Comment recognition is string-aware, and template-literal state carries across lines: a `//` or `/*` inside a string or template literal is content, not a comment opener, whether it sits on the literal's opening line or a continuation line. | Test (TC-798, TC-799) |
| FR-051-AC-13 | A declaration whose signature spans lines binds tags in its docstring: a `def` wrapped by a formatter has the same span as the unwrapped form. | Test (TC-800) |
| FR-051-AC-14 | Comment, string and template state is derived once per file and read by every consumer — the balance check, brace depth, and block-end spans — rather than re-derived per consumer. | Test (TC-803) |
| FR-051-AC-15 | The Rust adapter's lexer recognizes raw strings, lifetimes, character literals and nested block comments, so a brace inside any of them never moves the depth and never rejects the file. | Test (TC-804) |
| FR-051-AC-16 | A legacy textual form mints one `verifies` relation per trace id its match carries, so a comma-separated list binds every id rather than only the first, and one authored line yields one rewrite suggestion naming all of them; a form declaring `id_format` renders a single id and is not split. | Test (TC-806) |
| FR-051-AC-17 | A Rust benchmark — an attribute-marked one, or a function a `criterion_group!` registers in either invocation form, whether or not the registration line carries a trailing comment — classifies as a benchmark symbol, and a `fuzz_target!` invocation mints one fuzz-target symbol per file whose span is its whole file. Both bind trace ids; a container and a plain function still bind none. Each kind's stable label (`benchmark`, `fuzz_target`) is part of the symbol identity and of the FR-045 record's `kind` field. | Test (TC-827, TC-828) |
| FR-051-AC-18 | A `test`/`it` registration whose modifier chain is curried (`it.skipIf(cond)(…)`, `it.each([…])(…)`), or whose title literal begins on a later line, registers a test symbol named by that title, with the span and leading block any other registration gets. The scan is bounded and stops at the first non-blank text: a title held in a variable, an identifier merely beginning with `it`, and a literal beyond the window each register nothing rather than something wrong. A title inside a multi-line template literal is out of scope and registers nothing (CR-084). | Test (TC-943, TC-948, TC-958, TC-960, TC-961) |
| FR-051-AC-19 | Binding reports a per-language census of what it examined: `candidates` counts the evidence symbols whose kind admits a trace tag, `bound` counts those that minted at least one `verifies` relation, and `forms` names every declared form consulted for that language, markers before legacy. `bound` counts symbols, not relations, so a test carrying five ids counts once. A container or a production function is never a candidate. The census is ordered by language label, is present for every language the walk saw an evidence symbol in, and its `candidates` count does not depend on the declared patterns — the same tree bound against a grammar that matches nothing reports the same candidates and zero bound (CR-093). | Test (TC-982) |
| FR-051-AC-20 | The Python adapter tracks a triple-quoted string by where its delimiter *is*, not by where the line starts: an opener anywhere on a line (`FIXTURE = """`) enters string state, the body is never read as code, and the closing delimiter closes rather than re-opens. A string opened and closed on one line leaves the state unchanged; both delimiter kinds and every string prefix (`f`, `r`, `b`, `rb`, `u`) are recognised; a triple delimiter inside a single-quoted string, escaped, or after a `#` comment marker toggles nothing, and a `#` inside a triple-quoted string does not end it. A declaration following an embedded string therefore keeps its true container rather than resuming a stale scope (CR-115). | Test (TC-1029, TC-1030, TC-1031) |

> **CR-115 note (2026-08-24):** AC-20 is new. `agent-ix/quire-rs#274`, epic
> `agent-ix/quire-rs#264`.
>
> The Python adapter entered string state only when a triple-quote **started**
> the trimmed line. Two failures then compounded on the same literal, in
> opposite directions. `FIXTURE = """` does not start its line, so the scanner
> never entered string state and read the literal's **body** as code, inventing
> declarations the program does not contain. The **closing** `"""`, which does
> start its line, was then read as an **opener** — and everything after it
> swallowed into a string that never ended, losing every real declaration in the
> rest of the file. Because the state oscillated, the scope stack resumed stale,
> so declarations after a desync were attributed to whichever class was open
> when it began.
>
> **What the number counts.** Declarations — `class`, `def`, `async def` — that
> the shipped adapter's own symbol table holds, diffed against `ast.parse` over
> every `.py` under `~/dev` (excluding `worktrees/`, `.venv*`, `site-packages/`,
> `node_modules/`), 3,652 files parsed, 18 unreadable by `ast` and skipped,
> 30,569 declarations of ground truth. Comparison is by multiset, and both
> namings are reported because the engine mints the qualified one: `bare` is the
> declaration's own name, `qualified` its dotted path through enclosing classes.
>
> | | files | declarations |
> |---|---|---|
> | lost, bare, before | 42 | 300 |
> | lost, bare, after | 0 | 0 |
> | lost, qualified, before | 43 | 404 |
> | lost, qualified, after | 1 | 1 |
> | invented, qualified, before | 19 | 111 |
> | invented, qualified, after | 1 | 1 |
>
> The gap between the two axes is the misattribution mode: 104 declarations were
> present under the wrong container, which a bare-name comparison cannot see.
> Named example from the ticket, verified: `test_update_simple_version`
> (`py-project/tests/test_deps.py:464`) really lives in `TestTomlModification`
> and was reported under `TestTomlParsing`.
>
> **Measured against the engine, not a reimplementation.** #274's own figures
> moved three times because each was produced by a port of the state machine,
> and the ports disagree exactly where the original is wrong (#309). These
> numbers come from `quire_rs::symbols::extract_file` itself, run twice over one
> file list — once on the parent commit, once on this one.
>
> **What did not change.** Scope is popped by indentation alone, so a
> declaration nested inside a block that is itself nested keeps the wrong
> qualifier. The residual row above is exactly one instance:
> `workflow-plugin-sdk/tests/test_schema.py:69-75`, a function-local
> `class EmptyModel` at column 8 inside a test method, capturing a nested
> `async def handler` at column 12 — reported as
> `TestSchemaGeneration.EmptyModel.handler` where `ast` says
> `TestSchemaGeneration.handler`.
>
> An earlier draft of this note named `if not TYPE_CHECKING:` at column 0 as
> the instance. That is the same defect CLASS and a real one, but it is not
> what the residual row counts. Both old and new scanners produce it
> identically, so it predates this change.
> It is not folded in here, because a fix that widened this one until the
> residue vanished would stop being a statement about triple quotes.
>
> **Not a tokenizer.** The adapter is line-oriented by design (FR-051's
> indentation-structural model) and stays so: one left-to-right byte pass per
> line, no allocation, no lookbehind, carrying a single `Quoting` value across
> the line boundary.
>
> **No gate measures it**, and an earlier draft of this note claimed
> `scripts/check_perf_regression.sh` does. It does not: that script is not in
> `make ci`, and the CI `perf` job runs `--bench parse --bench load`, neither of
> which touches `src/symbols/`. No bench exercises the adapter at all.
>
> Measured by hand instead — 199,800 lines through `extract_file`, twenty
> passes: 0.85/0.90/0.93s before, 0.94/0.94/0.97s after, **about +6%**. Inside
> the 10% band the retired claim invoked, but stated as a measurement rather
> than as a gate that would catch a regression, because none would.

> **CR-093 note (2026-08-22):** AC-19 is new — the binder says what it looked
> at. `agent-ix/quire-rs#227`, epic `agent-ix/quoin#197`.
>
> The engine already had both numbers. It walks every candidate in order to
> match it, so it knows how many it examined and how many bound, and it reported
> neither. On a corpus whose tag convention matches no declared pattern that
> silence is the whole defect: the rows those symbols verify come back unbacked,
> and unbacked is exactly what a missing test looks like.
>
> **What the number counts:** evidence symbols in `agent-ix/filament-ide-rs` @
> `fc5d644`, under `quire 0.29.0` / engine `v0.42.0` /
> `spec-artifacts-process v0.23.0` — **1,292 Rust candidates, 0 bound**,
> reported as `Coverage: 555/2389 rows backed (23%)` and nothing else. The two
> declared conventions both missed: `fn tc_NNN_` against a pattern with no
> separator (fixed in `spec-artifacts-process#59`), and 643 `/// Tracing:` lines
> against a declared `Trace:` keyword. Establishing that took a controlled
> experiment — copy the module, widen one regex, re-run, diff the census — to
> learn something the engine had observed and discarded.
>
> **Three published SpecReviews were built on it.** SR-150, SR-151 and SR-152 in
> that repository cite coverage figures computed under the broken match, and one
> carried a finding — "TC-441/TC-446 are status lies" — as an open corpus-policy
> question across all three. They were not lying; the tool could not read the
> line their proof sat on.
>
> **`bound` counts symbols, not relations**, and the distinction is load-bearing.
> The question is "could the binder read this repository's convention at all",
> and one test carrying five ids is no more evidence of that than one carrying a
> single id. Counting relations would let a handful of heavily-tagged tests hide
> a thousand unreadable ones.
>
> **A container and a production function are not candidates.** CR-061 settled
> that only leaf evidence binds; counting the symbols that were never eligible
> would put every repository's census at a fraction of 1 and make the number
> meaningless in the ordinary case — which is the failure mode this AC exists to
> end, one layer over.
>
> **Not `untracked_symbols`.** That reports symbols that bound to an id no row
> declares — symbols that *matched a pattern*. A symbol matching no pattern was
> invisible to every output surface, which is why the count had to be new rather
> than derived.

> **CR-084 note (2026-08-20):** AC-18 is new. The TypeScript adapter registered
> **no symbol at all** for a curried registration — `it.skipIf(cond)(…)` and
> `it.each([…])(…)`, the conditional and parametrised forms both vitest and jest
> ship — and for any registration whose title wrapped onto a later line, which is
> simply how either is formatted past the line width.
>
> AC-3 named "TypeScript `test`/`it` registrations" and said nothing about
> modifiers, currying, or multi-line calls. There was no acceptance criterion to
> violate, which is why this survived: the regex matched the shape its author had
> in mind, and the shapes it did not match were indistinguishable from a file
> containing no tests.
>
> **The consequence is worse than a missed tag.** With no symbol, there is
> nothing for a legacy comment id *or* a canonical `trace(…)` call in the body to
> attach to — so migrating a repo to the canonical form would not have fixed it.
> The test runs, passes, and binds nothing, silently and always in the direction
> that loses coverage. Same class as CR-036/037, CR-040 and #68.
>
> **Measured before deciding the shape**, per this repo's rule that a count
> decides whether a finding is a rule problem or a corpus problem: across the 239
> `~/dev` repositories, curried registrations whose title carries a trace id number
> **one** — `typesetter/tests/ToleranceTaxonomy.test.ts:69`. This is a
> latent-authoring-trap fix, not coverage recovery, and it is scoped accordingly:
> a bounded forward scan over the lex `parse` already holds, no new abstraction,
> and no fixture change. TC-943 embeds that one real line verbatim.
>
> The scan **stops at the first non-blank text** rather than hunting for a quote.
> A scan that hunted would name a test after an unrelated string further down the
> argument list, and a wrong symbol name is worse than none: it binds a tag to the
> wrong requirement instead of visibly binding nothing.
>
> One limitation is deliberate and asserted rather than chased: `lex_line` drops
> content carried in from an unterminated template literal, so a title written
> inside a multi-line template registers nothing — the pre-CR-084 outcome, and
> preferable to registering a wrong name.

> **CR-090 note (2026-08-21):** the three gaps CR-084 shipped with are closed
> (`agent-ix/quire-rs#214`). The widened grammar — whitespace between the
> identifier/modifier chain and `(`, and an unbounded `.modifier` chain, where
> the old regex allowed exactly one `\.\w+` and no whitespace — was unpinned;
> TC-961 pins each admitted edge and each boundary that stays closed. CR-084's
> "no fixture change" left the scanner verified only through the crate-private
> `parse()`; the fixture `tests/fixtures/symbols/typescript/registration.test.ts`
> now carries every widened form plus the negative shapes, and TC-958/TC-960
> exercise them through `extract_tree` — the path every consumer uses. The
> CR-084 pair's legacy doc-comment tags migrate to `#[trace]` (TC-798's too),
> per FR-051-CON-3's direction of travel. No criterion changes.
> (Allocation note: TC-959 is skipped — the string already occurs as quoted
> foreign census data in `reports/2026-08-20-slash-trace-sweep.json`, and the
> all-refs collision grep must stay clean.)

> **CR-061 note (2026-08-16):** AC-17 is new. `trace::bind` skipped every
> symbol that was not a `TestFunction`, so a trace tag attached to anything else
> bound nothing — and three whole verification *methods* could never back a
> matrix row however they were tagged. CR-058 measured this by tagging them and
> re-running: TC-577 (a criterion bench), TC-579 (a `fuzz_target!`) and TC-502
> (a shell audit) stayed unbacked, and were marked 🚧 with the reason inline as
> the least-wrong option and explicitly not a resolution.
>
> The guard's real rule is **leaf evidence**, and the two things it must exclude
> are excluded for two different reasons. A **container** would let a `mod tests`
> block inherit every marker nested inside it — the original FR-051 reason, and
> still right. A plain **function** is production code, whose doc comments in
> this repository routinely cite the acceptance criteria they implement; binding
> those would manufacture backing out of prose, which is a far larger error than
> the one being fixed. A bench and a fuzz target are neither: each is a leaf
> artifact that exists to verify something and runs in CI.
>
> This was preferred to declaring `Benchmark` and `Fuzz` in the module's
> CR-041 `no_source_symbol` vocabulary. That mechanism exists for methods that
> **cannot** produce a symbol — an agent eval, a person reading code — and
> withdraws the accusation while leaving the row unbacked. A bench *is* a
> symbol; saying otherwise to quiet the report would be false, and would change
> verdicts for every repository on `spec-artifacts-process`.
>
> Two adapter facts fall out of it. A criterion bench carries no attribute — it
> is an ordinary `fn` that `criterion_group!` *registers* — so the registrations
> are collected in the same pass and the named top-level functions are promoted
> afterwards. And `fuzz_target!` declares no `fn` at all, so those files
> previously yielded **no symbol whatsoever**; the invocation now mints one whose
> span starts at line 1, because a `#![no_main]` fuzz-target file declares
> exactly one entry point and its header is that entry point's annotation block.
>
> Two constraints worth recording, because "tag the test harder" is the obvious
> wrong first move for anyone who hits this:
>
> - A `//!` module header binds nothing. The declared legacy forms match `//`
>   and `///`; `//!` matches neither. TC-579's tag was written there and bound
>   nothing even after the span reached it.
> - `/// NFR-002-AC-4 / TC-577` binds `NFR-002-AC-4` only. The declared list
>   separator is a comma, so a `/`-separated pair drops its second id in
>   silence. Filed separately against `spec-artifacts-process`.
>
> **TC-502 is not resolved by this.** A shell audit is not a Rust symbol:
> `language_of` reads `.rs`, `.py` and `.ts`/`.tsx`, so `.sh` is never opened
> and no widening of the binder reaches it. It stays 🚧 with its note corrected
> to name the actual blocker, and the shell-language question is filed
> separately. Closes agent-ix/quire-rs#126.

> **CR-043 note (2026-08-14):** Canonical markers and legacy forms read their
> ids by different rules. `marker_ids` comma-splits a marker's argument list, so
> `#[trace("TC-001", "FR-007-AC-1")]` binds both ids; `legacy_id` returned
> capture group 1 whole, so `// Trace: FR-001-AC-1, FR-001-AC-2` bound the first
> and silently dropped the rest. Nothing was lost by a bug — the ids were never
> *read*.
>
> **[RAN]** Across `~/dev`, worktrees and `-task<N>` copies excluded: **98
> legacy comment lines carrying a list, 205 ids binding to nothing, 17 repos** —
> spanning every declared legacy shape and all three languages. `quoin`'s 24
> dropped ids were about a tenth of the ecosystem total, and all 15 of its status
> lies had this one cause (agent-ix/quoin#65).
>
> **The engine alone could not fix it, contrary to the filing.**
> agent-ix/quire-rs#68 stated that no module needs to re-declare anything.
> Verified against real input, that is false: `Trace:\s*(ID)` matches once and
> stops at the comma, so capture group 1 is *already* a single id and splitting
> it moves nothing. Both halves are required — the declared patterns widen their
> id group to a list, and the engine splits it where `marker_ids` already does.
> AC-16 states the engine half; the module half lands in
> `spec-artifacts-process`.
>
> **`id_format` is deliberately excluded.** `rust-test-name-id` renders `TC-{1}`
> over a function name, which cannot carry a list. Splitting a rendered id would
> be dead code with future risk, so the template path is unchanged.
>
> Fixing the grammar rather than splitting the comments is the point:
> agent-ix/quoin#73 split quoin's 24 onto their own lines and that was the right
> repo-local fix, but the form reads naturally and the corpus keeps writing it. A
> grammar that silently means less than it says is the same failure shape as
> CR-040 and the `describe(` binding.
>
> Closes agent-ix/quire-rs#68.

> **CR-040 note (2026-08-13):** The Rust adapter carried the whole class of
> defect CR-039 had just removed from the TypeScript one, and it was **live**.
> `brace_delta` modelled a string as "text between unescaped quotes" and a char
> literal as "text between apostrophes". Rust breaks both: `r#"…"#` is a raw
> string where `\` escapes nothing and `"` is content, `&'a str` is a lifetime
> and not an open quote, and block comments **nest**.
>
> **[RAN]** 33 of this repo's own source files — every one holding a `r#"…"#`
> JSON fixture — were rejected as `unbalanced braces` and yielded **zero**
> symbols, so every trace tag in them bound to nothing. Measured against the
> repo's own matrix, that alone accounted for **78 of the 140 reported status
> lies** (agent-ix/quire-rs#60): backed rows went 144/907 → 306/907 and lies
> 140 → 62 with no matrix edit at all. The matrix was not overclaiming; the
> adapter could not see the tests.
>
> AC-15 gives the Rust adapter the same single-pass lexer AC-14 gives the
> TypeScript one, taught the four Rust-specific forms. A lifetime is
> distinguished from a character literal by a closing quote in one of the only
> two positions a literal can put one — decidable without parsing, because a
> lifetime is never followed by a quote that soon.
>
> Recorded as a measurement, not a triage: the remaining 62 lies are what
> agent-ix/quire-rs#60 is actually about.

> **CR-039 note (2026-08-13):** CR-036 and CR-037 each fixed one place where a
> line-structural adapter met a multi-line construct and silently produced
> **zero** symbols for the whole file. Both were found by measurement rather
> than by reading, because the failure is invisible from the source: the file
> parses, its tests pass, its trace tags are present and greppable, and every
> one of them binds to nothing.
>
> The cause was structural, not local. Three functions each derived "am I inside
> a comment, a string, a template?" by slightly different rules —
> `check_balanced` and the declaration scan carried state across lines,
> `brace_delta` re-derived quote state per line, and `block_end` restarted from
> `ScanState::default()` at the declaration index. AC-14 collapses them into one
> lexer pass per file whose per-line output — code text plus brace delta — every
> consumer reads. Agreement between them is now structural instead of
> incidental.
>
> **Measured while doing it:** the `block_end` restart is **not reachable**
> through `parse`. Every declaration matcher is `^`-anchored, so a line whose
> code begins mid-construct never presents a declaration to match, and the
> restart therefore always began in the state the carried lex would have given
> it. The refactor removes the hazard rather than a live defect — recorded so
> the next reader does not go looking for the failing corpus case.
>
> Closes agent-ix/quire-rs#62.

> **CR-037 note (2026-08-13):** Found by running `gap-analysis` over
> `spec-artifacts-process` with the new coverage path — two tests differing only
> in signature wrapping bound differently, and the matrix reported a status lie
> for the wrapped one.
>
> The Python adapter ends a suite at the first line indented no deeper than the
> declaration. Black wraps any `def` over the line limit, and the closing
> `) -> None:` sits at the declaration's **own column**, so the span ended there
> — one line short of the docstring, which is exactly where the trace tag lives.
> The suite is consumed by parenthesis depth first, then by indentation.
>
> This is the same defect class as CR-036 in the TypeScript adapter: a
> line-structural reader meeting a formatter-produced multi-line construct. Both
> fail silently and in the direction that loses coverage, which is why each is a
> stated acceptance criterion now rather than a property of whichever shapes a
> corpus happened to contain.

> **CR-036 note (2026-08-13):** The TypeScript adapter's comment stripper scanned
> raw characters, so a `/*` inside a literal opened a block comment that never
> closed. Every line after it was stripped, the braces could not balance, and
> `check_balanced` rejected the file — which under FR-051-CON-2 means the file
> yields **zero** symbols and every trace tag in it binds to nothing.
>
> The failure is silent by construction. The file is valid TypeScript, its tests
> run and pass, and its tags are present and greppable; only the symbol graph
> knows they attached to nothing. It was found in `quoin` by a coverage rollup
> that scored a correctly-tagged file 0/2, after one git refspec —
> `` `fetch = +refs/heads/*:refs/remotes/origin/*` `` — in a template literal.
> AC-12 makes string-awareness a stated property rather than an accident of
> which characters a corpus happened to contain.
>
> **Carried across lines, and that is not a detail.** The first fix tracked quote
> state per line, which handles a single-line literal and *not* the form the
> corpus actually writes — the refspec sits on a continuation line of a multi-line
> template, where a per-line scanner has already forgotten it is inside a literal
> and re-opens the block comment one line later. The file was rejected exactly as
> before. A template literal is the only TS/JS string form that can span lines, so
> it is the only one carried; a `'` or `"` left open at end of line is a malformed
> line, not a continuation. TC-799 covers the continuation form.

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the declared trace-tag grammar), [FR-045](./FR-045-filament-core-extraction-engine.md) (record shapes, id and dedup conventions), [NFR-006](../non-functional/NFR-006-determinism.md) (determinism discipline)
- **Downstream**: the coverage rollup ([FR-050](./FR-050-declarative-coverage-computation.md)), the `gap-analysis` semantic review, and filament-core knowledge-graph ingestion consume the symbol graph; [FR-065](./FR-065-controlled-corpus-contract.md) (the corpus's L2 localisation level reads `binding_census.unbound_example` from this extraction)
- **Companion deliverables (outside quire-rs core)**: the canonical markers imply per-language support packages — a pytest plugin registering the `trace` marker, a lightweight Rust no-op proc-macro crate, and an npm helper for vitest/jest — separate deliverables following the separate-workspace-crate pattern (ADR 0010 placement precedent); this FR specifies only the static parsing contract the extractor holds them to
