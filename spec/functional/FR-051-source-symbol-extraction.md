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
- **Downstream**: the coverage rollup ([FR-050](./FR-050-declarative-coverage-computation.md)), the `gap-analysis` semantic review, and filament-core knowledge-graph ingestion consume the symbol graph
- **Companion deliverables (outside quire-rs core)**: the canonical markers imply per-language support packages — a pytest plugin registering the `trace` marker, a lightweight Rust no-op proc-macro crate, and an npm helper for vitest/jest — separate deliverables following the separate-workspace-crate pattern (ADR 0010 placement precedent); this FR specifies only the static parsing contract the extractor holds them to
