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
functions, test functions, entry points (shell, below), and containers (Rust `struct`/`enum`/`trait`/`mod`,
Python classes and modules, TypeScript classes and modules). Each symbol SHALL
carry a **stable identity** composed of language, repo-relative file path,
qualified symbol path, and kind. Symbol identity SHALL NOT incorporate line
numbers, byte offsets, or formatting, so reformatting a file leaves every
identity unchanged; the current line number SHALL be carried as a non-identity
attribute. Record ids SHALL be the identity itself, per the
[FR-045](./FR-045-filament-core-extraction-engine.md) record-id convention.

## Source walk

Which files the extractor reads SHALL NOT depend on the machine it runs on. The walk under
a code root SHALL apply exactly these rules, in this order; a path removed by an earlier rule
is never judged by a later one:

1. **Non-configurable prunes.** The document root (a symlinked document root such as
   `spec -> real_spec` included, as it resolves) and a cookiecutter template tree
   (`{{...}}`, whose placeholder ids must never bind) are not walked. Source files under a
   template tree remain counted in `excluded_source_files`
   ([FR-050](./FR-050-declarative-coverage-computation.md)-AC-24).
2. **Hidden entries** (a name starting with `.`), directories and files alike, are not source.
3. **Ignore files.** Entries matched by a `.gitignore` or `.ignore` file are not source, with
   or without a `.git` directory. Those files are read at or under the code root and in
   each ancestor directory up to and including the nearest ancestor holding a `.git` entry
   (the repository root), so a committed repo-root `.gitignore` still applies when the code
   root is a subdirectory. An ignore file above that repository root, a user-global ignore
   file and `.git/info/exclude` are never consulted. Without any `.git` ancestor only the
   files at or under the code root apply.
4. **Symlinks.** A symlink is recognised from its own link metadata alone, never from its
   target, and is never followed or read, so a dangling link, a link to a file and a link to a
   directory are one case. Each symlink that reaches this rule is reported (AC-54).
5. **Declared `source_exclude`** (FR-050-AC-22) removes matching files, symlinks included.

Only languages the extractor supports (Rust, Python, TypeScript, shell, QSL) are in scope:
an unsupported extension is not reported file by file, and a supported language the build
cannot read is recorded as unavailable (AC-29). What rule 3 removes is made visible in
aggregate (AC-55); what rule 4 removes is reported per path.

## Language adapters

> **CR note (agent-ix/quire-rs#407, 2026-09-06):** Python `test_` methods
> on a class with a direct `unittest.TestCase` base are evidence regardless
> of the class name. The bounded static forms are a single-line top-level class header
> referencing a preceding module-level `import unittest` (with optional alias),
> or `from unittest import TestCase` (with optional alias). Comma-separated
> imports and bases retain those same exact identities. Unrelated imports,
> including dotted imports that rebind their first component, invalidate a
> previous identity. An unaliased `import unittest.mock` retains the unittest
> root; an alias for that submodule does not become an alias for unittest.
> Similarly named bases, helper methods, nested helper functions, and ordinary classes do not gain
> evidence status. A simple single-name top-level assignment, deletion, import,
> or class/function declaration replacing a binding removes its imported identity.
> Dynamic factories, wildcard imports, local imports, multi-line import/base
> declarations, and transitive inheritance resolution are not inferred.
> Base matching stops at the class suite's colon or a comment: a TestCase
> spelling in comment text or class-body data never establishes inheritance.
> Existing pytest naming rules are unchanged. This repairs an under-reading
> rule, not the authored tests: native unittest runs four controlled methods
> that the old adapter reports as zero candidates. The corpus banks both
> spellings and sibling non-test controls (TC-1803).
> Authority: [Python unittest loading](https://docs.python.org/3/library/unittest.html#unittest.TestLoader.loadTestsFromTestCase).

The extractor SHALL ship per-language adapters for Rust, Python,
TypeScript, and shell, each behind its own Cargo feature (`rust-symbols`,
`python-symbols`, `typescript-symbols`, `shell-symbols`); a build without a feature SHALL record that language as
unavailable when the tree holds source in it, and every consumer of the
extraction SHALL refuse rather than report on a tree it only partly read
(AC-29, AC-30). Each adapter SHALL parse its file to a syntax tree with that
language's grammar and SHALL read every symbol, span and container relation
from the tree's nodes. Adapters SHALL read the source text of the analysed
tree alone: no build of it, no type resolution, no installation of its
dependencies. A file is **unparseable** where the tree the grammar yields
carries an error over the file's declaration structure — the term CON-2 and
AC-9 rest on, stated because a grammar returns a tree for every input, so the
trigger is a property of the tree rather than the absence of one. The
per-file diagnostic SHALL name the file and the line the error sits at, so a
reader is told where to look rather than that something somewhere failed.
Adapters SHALL classify test
functions by each language's convention: Rust functions under a `#[test]`-family
attribute, Python `test_`-prefixed functions and test-class methods, and
TypeScript `test(...)`/`it(...)` registrations (the registered title is the
symbol's qualified name). A TypeScript `describe(...)`/`suite(...)` registration
is a **container**, not a test: it groups tests rather than being one, and it is
the scope the registrations inside it are parented to. If an adapter cannot
parse a file, then the extractor SHALL emit a per-file diagnostic, skip the
file, and continue. Every grammar reaches the extractor through the single
tree-sitter boundary crate `quire-code-extraction`
(`scripts/audits/check_tree_sitter_boundary.sh`): the shell adapter SHALL
parse with the `tree-sitter-bash` grammar, enabled by a `bash` feature of
`quire-code-parse` (in `quire-code-rs`) that `quire-code-extraction`
forwards under the `shell-symbols` feature, as the other three adapters are.

### Shell adapter (CR-213, PLAT-362)

The extractor SHALL read a file whose extension is `.sh` through the shell
adapter and report it as language `shell`; no other spelling (`.bash`, an
extension-less file with a `#!` line) is read. Shell has no attribute syntax,
so its tag channel is the `#` comment, read from the syntax tree's comment
nodes. The adapter SHALL mint two kinds of symbol:

- a **function** symbol for every `function_definition` node the grammar
  yields, whatever the spelling (`name() { … }`, `function name { … }`,
  `function name() { … }`, `{` on a following line, a subshell or other compound
  body, a name containing `-`, `.` or `:`), whose qualified name is the
  function's own name. Shell functions are global, so a definition nested in
  another function's body is named by its own name alone;
- one **`entry_point`** symbol for **every** `.sh` file, spanning the whole file
  and named `entry_point` (the whole-file precedent of the Rust `fuzz_target`
  symbol). The extractor SHALL mint it without consulting module data, so the
  candidate population does not depend on declared patterns (AC-19) and the
  symbol table is the same with or without a module (AC-23).

An `entry_point` is evidence: it binds `verifies` ids, and it does not carry
`implements` (the FR-062 complement rule). A shell function is production: it
binds no `verifies`, and carries `implements` only through a `language: shell`
form the module declares in `trace_tags.implements`. Whether an `entry_point`
is **eligible** to bind is decided at the binding stage by the module's
`entry_paths` globs (AC-38, AC-39), not at extraction.

> **CR-213 note (2026-11-08, PLAT-362):** two clarifications. First, the
> function symbol is minted for every `function_definition` node the grammar
> yields *except* one inside a command substitution (`$( )` or backticks): that
> definition lives in a subshell and is not a global function, and AC-36 lists
> the substitution among the places a definition-shaped line mints nothing.
> Second, shell symbols are **excluded from the binding census of AC-19**:
> the `entry_point` is minted for every `.sh` file regardless of any declared
> pattern, so counting it would report every unglobbed script as an unbound
> candidate and fire `no-symbol-bound` on repositories that declare no shell
> form. A shell marker that binds nothing is reported by AC-44 instead.

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
  `trace("FR-007-AC-01")` wrapping or annotating the registration;
- shell — a `#` comment (`# trace: FR-007-AC-01`) in the tag channel of AC-40,
  declared as a `markers` entry with `language: shell` and `syntax: comment`.
  Shell has no attribute syntax, so a comment is the only construct a canonical
  shell marker can be; it is canonical because the module declares its form and
  the extractor reads it from the syntax tree's comment nodes, not because of
  where it sits on the page. It is a marker form, so FR-051-CON-3 does not
  retire it.

The extractor SHALL read markers **statically** from the syntax tree — the
decorator, attribute and call-argument nodes (and, for shell, the comment
nodes) the tree attaches to the test symbol's declaration — so a marker binds by its attachment rather than by its
position on the page, and coverage never requires a runtime. Runtime
queryability (running all tests for an FR, tagging JUnit reports) is a stated
benefit of the marker form, not a requirement of this FR.
Each trace id attached by a marker SHALL mint one `verifies` relation from the
test symbol to that id; a trace id attached more than once to one symbol
(repeated marker) SHALL mint one relation and a diagnostic, per the FR-045
edge-dedup convention. **A `verifies` relation is minted only from a marker
form.** Text that is not a marker form binds no `verifies` relation, and a
`verifies` relation carries no field recording how it was written. The
`implements` relation keeps its own declared textual forms
([FR-062](./FR-062-implements-relation.md)); this paragraph does not touch them.

### Advisory: an id-shaped token outside a marker

One advisory pattern, the generic id-shaped-token detection that already feeds
`unmatched_tags` and `mentions`, finds an id-shaped token (`FR-007-AC-01`,
`TC-041`) of a kind the corpus mints (`CR-215`, `PLAT-847` and a bare `AC-3` are
not trace ids and are not reported) in the comment or docstring block attached to a declaration. It is
detection only: it reconstructs no id, is not specialised per language, and
produces no binding and no rewrite. For **every** occurrence, from the first
release carrying it and whether or not any other form also bound that id, it
emits one warning naming `file:line` and the canonical text to write, rendered
from the loaded module's declared marker `template` for that file's language
(AC-49). The warnings are the worklist for a repository that has not yet
written markers; the work is done when they stop. A token counts only when its **kind** — the leading alphabetic prefix, compared without case (`FR` for `FR-007-AC-1`) — is the kind of at least one id the loaded corpus's declared trace targets minted; when the corpus minted no id, no kind is declared and every id-shaped token counts (CR-215).

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
| FR-051-CON-1 | The extractor SHALL classify syntax from a syntax tree that a grammar-driven parser library produces over the source text, and performs no network I/O, no service I/O, no execution of extracted code, no compilation of the analysed tree, no dependency installation, and no type resolution. This holds for every adapter except the QSL adapter, shell included (the `tree-sitter-bash` grammar), and the shell adapter never executes or sources the script it reads. The QSL adapter (FR-094) reads declaration heads line by line, uses no syntax-tree library, and performs no I/O, execution, compilation or type resolution either. | Architecture | Test |
| FR-051-CON-2 | Adapters SHALL degrade per file: one unparseable file never aborts the tree extraction. | Operational | Test |
| FR-051-CON-3 | Only marker forms bind a `verifies` relation: the `verifies` binding path, the `VerifiesRelation` and symbol-graph types, the `trace_tags.markers` schema and the binding census SHALL carry no field, variant, list, template, branch or compatibility path for any other trace-text form (CR-215, PLAT-847). The `trace_tags.implements` textual forms of FR-062 are outside this constraint. | Architecture | Inspection |
| FR-051-CON-4 | The order of work that removes non-marker binding is a precondition of the change, not a follow-up: first, every symbol that binds only through a non-marker form is retagged with its canonical marker (the `quire-rs` symbols first, then `quire-cli`, then `quoin`); second, the module `trace_tags` entries that declare non-marker forms are removed from `spec-artifacts-process` in a cross-repo change, and that module release precedes the engine release that deletes the binding machinery or ships with it, because the new engine rejects an undeclared `trace_tags` key (AC-43) and so could not load a module that still carries them, while an earlier engine loads a module without them; third, the binding machinery is deleted. The per-repository coverage delta (rows backed before and after, for every repository retagged, and the symbols that were tagged only through their name, which the change leaves unbacked and unreported) is measured and published with the change, stating that the numbers moved because the evidence standard changed and not because tests were lost (CR-215). | Operational | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-051-AC-1 | Each adapter extracts functions, test functions, and containers from a fixture tree (the shell adapter: functions and the `entry_point`), and each symbol carries language, repo-relative path, qualified path, kind, and a line attribute. | Test |
| FR-051-AC-2 | Reformatting a fixture file (whitespace and line-number changes only) leaves every symbol id unchanged; renaming a symbol changes only that symbol's id. | Test |
| FR-051-AC-3 | Rust `#[test]`-family functions, Python `test_` functions and direct imported unittest.TestCase methods in the bounded forms above, and TypeScript `test`/`it` registrations classify as test symbols; sibling non-test symbols do not. **Since CR-187:** TypeScript `xtest`/`xit` base names classify as test symbols exactly as `test`/`it` do (they mint with `ignored: true`, FR-051-AC-27) — no new registration shape, only the admitted base-name set widens. | Test |
| FR-051-AC-4 | Each canonical marker form binds statically: a Python `@pytest.mark.trace(...)` decorator, a Rust `#[trace(...)]` attribute, and a TypeScript `trace(...)` helper each mint one `verifies` relation per attached trace id, with no code executed. | Test |
| FR-051-AC-5 | Marker and tag forms are module data: a fixture model declaring different marker names/patterns binds by its own declaration, and with no declared forms the extractor mints zero `verifies` relations. | Test |
| FR-051-AC-6 | A trace id attached more than once to one symbol (repeated marker) mints one `verifies` relation and one diagnostic. | Test |
| FR-051-AC-7 | The emitted records match the FR-045 graph-record shapes with normalized `ref` values, and filament-core ingestion fixtures accept them unchanged. | Test |
| FR-051-AC-8 | `defined_in` edges link every symbol to its file and `contains` edges link containers to members, deterministically ordered. | Test |
| FR-051-AC-9 | An unparseable fixture file yields a per-file diagnostic naming that file and the line the error sits at, while the rest of the tree extracts normally. | Test |
| FR-051-AC-10 | Repeated extraction over an identical fixture tree, at a pinned grammar version (`tree-sitter-rust`/`tree-sitter-python`/`tree-sitter-bash`, locked in `Cargo.lock`) and a fixed Cargo feature set (`rust-symbols`/`python-symbols`/`shell-symbols`, which change whether Rust/Python extract at all — see CR-177, CR-179), emits byte-identical JSON and identical record ids. | Test |
| FR-051-AC-11 | Only a marker form binds (CR-215, PLAT-847): a fixture whose tests carry a bare id in a docstring, a doc comment or a line comment, a `Trace:` line, or an id in the test's own name, and no marker, mints zero `verifies` relations and backs no row, in every language; the same tests with a marker bind. A `verifies` relation carries its symbol, trace id, form, path and line and no provenance field, the symbol graph carries no rewrite list, and the binding census carries no field for a non-marker form. | Test |
| FR-051-AC-12 | Comment recognition is string-aware, and template-literal state carries across lines: a `//` or `/*` inside a string or template literal is content, not a comment opener, whether it sits on the literal's opening line or a continuation line. | Test |
| FR-051-AC-13 | A declaration whose signature spans lines binds tags in its docstring: a `def` wrapped by a formatter has the same span as the unwrapped form. | Test |
| FR-051-AC-14 | One reading of a file decides what is code in it, and every consumer reads that one answer — whether the file is accepted at all, and the span a symbol ends at — so two consumers of the same file never disagree about whether a given byte is code, a comment, or the inside of a string. A comment trailing on the same line as the statement before a declaration is that statement's own trailing note, not the following declaration's leading annotation (PLAT-882). A leading comment reaches a TypeScript registration through an `await` wrapper the same as without one — `await` is not a boundary a leading annotation search stops at (PLAT-882). | Test |
| FR-051-AC-15 | The Rust adapter reads raw strings, lifetimes, character literals and nested block comments as the language defines them, so a brace or a quote inside any of them is content: the file is accepted and yields its symbols. String state carries across lines: a raw string or an ordinary string opened on one line keeps its interior as content on the lines that follow, until its own closing delimiter — for a raw string, the matching hash count — so a `//`, a brace or a quote is content whether it sits on the literal's opening line or a continuation line, and code after the closer is code again. | Test |
| FR-051-AC-16 | A marker form mints one `verifies` relation per trace id its match carries, so a marker naming several ids binds every id rather than only the first, with irregular spacing and a trailing comma yielding the ids and never an empty one. | Test |
| FR-051-AC-17 | A Rust benchmark — an attribute-marked one, or a function a `criterion_group!` registers in either invocation form, whether or not the registration line carries a trailing comment — classifies as a benchmark symbol, and a `fuzz_target!` invocation mints one fuzz-target symbol per file whose span is its whole file. Both bind trace ids; a container and a plain function still bind none. Each kind's stable label (`benchmark`, `fuzz_target`) is part of the symbol identity and of the FR-045 record's `kind` field. | Test |
| FR-051-AC-18 | A `test`/`it`/`xtest`/`xit` registration (CR-187 widens this AC's own base-name set to match AC-3's) whose modifier chain is curried (`it.skipIf(cond)(…)`, `it.each([…])(…)`, `xit.each([…])(…)`), whose chain carries whitespace anywhere — before `(`, between curried groups, or before a `.` modifier (PLAT-882) — or whose title literal begins on a later line, registers a test symbol named by that title, with the span and leading block any other registration gets: the tree names a call's own first argument directly, so there is no longer a scan or a window bounding how far down the file that argument may sit (PLAT-882, superseding CR-084's own bounded-scan mechanism — see CR-180). A registration carries a callback as its second argument or it registers nothing: a title held in a variable, an identifier merely beginning with `it`, and a call with no second argument each register nothing rather than something wrong. A title inside a multi-line template literal is out of scope and registers nothing (CR-084). | Test |
| FR-051-AC-19 | Binding reports a per-language census of what it examined: `candidates` counts evidence symbols whose kind admits a trace tag; `tagged` counts candidates whose attached annotation block carries a generic id-shaped token or whose declared form bound; `bound` counts candidates that minted at least one `verifies` relation; and `forms` names every declared marker form consulted. The hard invariant is `bound <= tagged <= candidates`. An unbound candidate carries `unbound_example`; a tagged-but-unbound candidate additionally carries `unmatched_example`, each the deterministic lowest `path:line`. Generic tag detection is independent of the declared grammar and does not inspect the test body. The census counts every id-shaped token, whatever its kind (CR-215): unlike `unmatched_tags` (FR-050-AC-39) and the advisory (AC-49), it is not filtered to the kinds the corpus mints, so `tagged`, `tagged_not_bound` and `unmatched_example` can count a `CR-` or `PLAT-` token that has no `unmatched_tags` record. Counts are symbols rather than relations, containers and production functions are not candidates, rows are language-ordered, and the candidate population does not depend on declared patterns (CR-093, CR-142). | Test |
| FR-051-AC-20 | A Python triple-quoted string is content wherever its delimiter sits: a body opened mid-line (`FIXTURE = """`) is never read as code, and a declaration following an embedded string keeps its true container rather than resuming the scope that was open when the string began. Both delimiter kinds and every string prefix (`f`, `r`, `b`, `rb`, `u`) are recognised as openers; a triple delimiter written inside a single-quoted string, escaped, or after a `#` comment marker opens nothing, and a `#` inside a triple-quoted string ends nothing (CR-115). | Test |
| FR-051-AC-22 | An id-shaped token the advisory finds in the annotation block of a symbol whose kind cannot bind a trace id (a module, a container, a production function) is reported by AC-49 naming the id and the symbol's kind, with no `#[trace(...)]` replacement text, because a `trace` marker on that kind binds nothing (FR-062); where the kind is a production function or container the finding names instead the text rendered, as FR-050-AC-62 renders it, from the `template` of the first declared structural `implements` form (`syntax` `rust_attribute`, `python_decorator`, `typescript_decorator` or `typescript_wrapper`) whose `language` is the symbol's own, whose syntax position fits the symbol, and that declares one — `#[implements("FR-007-AC-2")]` for a Rust symbol, `@implements("FR-007-AC-2")` for a Python function, method or class, `@implementing("FR-007-AC-2")` for a TypeScript class or method, and `implementing("FR-007-AC-2")` for a `const`-, `let`- or `var`-bound TypeScript function (FR-062-AC-26); never a textual `implements` form — and where no such form declares a template, or none fits the symbol (a TypeScript `function` declaration), it names the kind alone, as it does for a module. It binds nothing, and no count in the payload moves, so the census `candidates` denominator is unchanged. A canonical marker on such a symbol is not reported: the advisory reads comment and docstring lines, never an attribute or decorator line, so `#[trace("X")]` on a production function yields no finding. Where several symbols span one token the **innermost** is named, because the fix a reader needs is the symbol the token sits on and not the module that contains it; at equal `leading_line` a container loses. A token is reported once however many symbols span it. | Test |
| FR-051-AC-23 | The extracted symbol table is reported on its own surface, as the engine built it: per symbol its path, qualified name, kind, language, declaration line, **declaration column** (CR-187: the 1-based UTF-8 byte column of the declaration's own start, as tree-sitter itself reports it — all four adapters parse through tree-sitter at this head), annotation-block line, end line, container, whether that **kind** can bind a trace id, whether it can carry an `implements` marker, whether AC-27 marks it (or its container) ignored, and — when a module is supplied — the ids it bound. A scanner defect must be sizeable **without reimplementing the scanner**: three ports of `symbols/python.rs` gave 386, 490 and 5,263 lost declarations over one tree, disagreeing precisely where the original is wrong. Binding is reported as **not asked** rather than as zero when no module is supplied, because an unbound run and a repository nobody tagged produce the same empty list. Per-language totals carry both denominators — symbols examined and symbols of a **binding kind** — since a rate over the wrong one reads a tree of containers as untagged. | Test |
| FR-051-AC-24 | String-literal contents are masked before the advisory (AC-49) matches, in **every** language rather than in Rust alone — a trace id a file carries as data is not an authored tag, and reporting it sends a reader to rewrite data. The mask preserves each language's **annotation channel**: comments everywhere, and additionally a Python **docstring**, which is a string literal by syntax. Byte length and line count are preserved so a reported `file:line` is the line in the original. Markers are never masked against: they put ids inside string literals by design. | Test |
| FR-051-AC-21 | A TypeScript `describe(...)` / `suite(...)` registration mints one **container** symbol named by its registered title, spanning its block and its leading annotation block, and the registrations written inside it carry it as their container rather than the file's module — a `describe(` whose `{` falls on a later line (a callback wrapped for width, like any other registration under AC-18) still parents its members to it: the tree, not a same-line text check, decides what is inside a suite's callback (PLAT-882, superseding the pre-tree-sitter scanner's own same-line requirement — see CR-180). A suite does not *name* its members: a registration's qualified name is its own registered title whether or not a suite encloses it, while a class inside a suite still qualifies its own. A suite is a grouping and not evidence, so a trace tag on a suite header mints **no** `verifies` relation and the suite is **not** a `binding_census` candidate — and since `agent-ix/quire-rs#312` that tag is **reported** rather than dropped, under AC-22. The tag naming a test is the declarative form; a tag on the group would make coverage an inference about which test inside is meant, and the report says where the tag is instead of guessing. A registration whose name chain names a suite **anywhere along it** is a suite: `test.describe(...)` and `it.describe.only(...)` classify exactly as `describe(...)` does, because a harness spells its suite as a member of its test namespace and reading only the first identifier gave one construct two classifications (CR-121). `context` is not a suite name. **Since CR-187:** `xdescribe` classifies as a suite base name exactly as `describe`/`suite` do (it mints with `ignored: true`, FR-051-AC-27, inherited by every member the tree still parents to it) — no new container shape, only the admitted base-name set widens. | Test |
| FR-051-AC-25 | A symbol and its trace ids are read from the declaration's own node in the syntax tree, so attachment rather than page position decides them: a Rust test whose attribute block carries `#[test]` in any position, a Rust declaration whose attribute block a formatter has split across lines, a Rust declaration preceded by a block doc comment (`/** ... */`), a Python marker a formatter has wrapped across lines, a marker separated from its `def` by a second wrapped decorator, and a declaration whose signature spans lines each classify and bind exactly as their single-line, adjacent spelling does (`agent-ix/quire-rs#387`, `agent-ix/quire-rs#395`, `agent-ix/quire-rs#459`, `agent-ix/quire-rs#843`). A brace inside a regular-expression literal is content — the ground AC-12 and AC-15 hold for the other literal forms — so a file carrying a `{n,m}` quantifier yields its symbols (`agent-ix/quire-rs#424`). | Test |
| FR-051-AC-26 | Extraction over a fixture tree that has never been compiled and whose declared dependencies are not installed yields that tree's symbols and relations, and the extraction path invokes no compiler, package manager, or build artifact of the analysed tree and resolves no type. | Test |
| FR-051-AC-27 | A test-kind or suite-kind symbol carries `ignored: bool`, true exactly for a **statically decidable** language form (CR-187, PLAT-1077; SR-117 FND-003/FND-004 narrowed this from the first draft): **Rust** — a `#[ignore]` attribute (with or without a reason string), before or after `#[test]`, on any function AC-3 already classifies as a test (which already includes the `#[tokio::test]`/`#[rstest]`/`#[wasm_bindgen_test]` family; no new attribute vocabulary is added for `ignored` itself). **TypeScript** — a `.skip` modifier anywhere in a registration's chain (`it.skip(...)`, `describe.skip(...)`), or a registration whose base name is `xit`/`xtest`/`xdescribe` (AC-3, AC-21). **Python** — `@unittest.skip(...)`/`@pytest.mark.skip(...)` on a function or method, the same decorator at class level on a `unittest.TestCase` subclass or a pytest test class, or a module carrying `pytestmark = pytest.mark.skip(...)` (bare `@pytest.mark.skip` with no call, and the aliased-import forms AC-3's own unittest binding already resolves, count identically). A suite's or class's `ignored` is inherited by every test-kind or suite-kind member nested in it, transitively, via the same `contains` edges AC-8 already emits — a test's own marking is not required when its container is ignored. Inheritance follows the enclosing container at the member's own declaration site, never a qualified-name lookup, because AC-21 lets two TypeScript suites share a title (CR-189). **Explicitly excluded, because each may still run:** Rust `#[cfg_attr(..., ignore)]` (conditional); TypeScript `.skipIf(cond)`, `.todo`, `test.fixme`; Python `skipif`/`skipIf`/`xfail`. A form the extractor does not otherwise classify as a test attribute at all — for example a bare `#[should_panic]` — is out of scope for `ignored` the same way it is out of scope for AC-3. A symbol carrying none of its language's ignored forms, and contained in no ignored container, has `ignored: false`. Marking a symbol ignored manufactures no new *relation* or *diagnostic* — a tag on an ignored symbol still binds a `verifies`/`implements` relation exactly as it would on a running one — but it is not silent on *symbols*: `xit`/`xtest`/`xdescribe` are registrations AC-3/AC-21 now mint (where they minted nothing before CR-187), each carrying `ignored: true` from the moment it exists. `ignored` is carried on the FR-051-AC-23 symbol-table record and on `VerifiesRelation`, serialized only when `true` (`skip_serializing_if`), so a payload over a corpus with no ignored symbol — including every existing fixture — stays byte-identical to TC-750's pre-CR-187 baseline. | Test |
| FR-051-AC-28 | A range — two id-shaped tokens joined by a bare `..`, whether same-prefix, differing-prefix, or short-suffix — inside a marker's argument or inside a textual `implements` use (`/// Implements: FR-001..FR-003`, `# Implements: FR-001..FR-003`), binds no id and mints no `verifies`/`implements` relation for any id it names, including its endpoints. One finding is reported per occurrence, in `CoverageReport.diagnostics` under reason `range-in-trace-tag` at FR-057 severity `warning` exactly as FR-050-AC-50 states it, naming the file, the 1-based line the range text sits on, the qualified symbol, and the literal range text verbatim (CR-189); the occurrence contributes to neither `verifies`/`implements` nor the generic id-shaped-token channels (`unmatched_tags`, `mentions`) — the range finding is the only record of it. A marker range used to mint one relation keyed on the whole literal string; that stops, so `verifies`/`implements` counts over a corpus carrying one can change (CR-187, PLAT-1077). Only the range token is inert: a marker that continues past a range (`#[trace("A..B", "C")]`) still binds every other id it names (CR-189). | Test |
| FR-051-AC-29 | A build that has no symbol adapter for a language whose source is in the tree (the `wasm` build drops all four adapter features) records that language as unavailable on the extraction, the symbol graph and the symbol-table report, with no symbols and no per-file diagnostic for those files; a tree with no file in an unavailable language is unaffected. Satisfied only when the unavailability is recorded: a zero-symbol extraction with no unavailable-language fact for a tree holding source the build cannot read fails this criterion (CR-201, PLAT-874). | Test |
| FR-051-AC-30 | Emitting FR-045 graph records from an extraction that records an unavailable language is refused with an error naming the language, not returned as an empty node and edge set (CR-201). | Test |
| FR-051-AC-31 | When two or more symbols extracted from one file share a symbol id, extraction reports one per-file diagnostic per colliding id, reason `symbol-id-collision`, at severity `warning` (as AC-28 does): it is advisory, appears on the symbol-table report with its severity, and never changes an exit code or refuses a result. The diagnostic names the kind, the qualified name, and every colliding declaration by `path:line`, followed by that declaration's `#[cfg(...)]` / `#[cfg_attr(...)]` outer attributes, whitespace-collapsed to one line, where the adapter records them (other attributes are not listed; adapters without such conditions list none). It reports shared identity and does not claim a defect: causes include mutually exclusive `#[cfg]`-gated variants, a Python function defined in both branches of an `if` or a nested helper repeated across scopes, and TypeScript registrations with one title. Rust trait-impl methods do not collide, because their ids are trait-qualified (AC-32; CR-410). Symbol ids are unchanged, and a file with no collision yields no such diagnostic (CR-206, PLAT-877). | Test |
| FR-051-AC-32 | A Rust test symbol's `qualified_name` is the `::`-joined chain of its enclosing `mod`, `struct`, `enum`, `trait`, the `impl` scope and an enclosing `fn`, then the function, where the `impl` scope is the target type for an inherent impl (`Type::method`) and `<Type as Trait>` for a trait impl (`<Type as Trait>::method`), with the type and the trait as written, whitespace-normalised, so distinct trait impls on one type never share a name (CR-410, PLAT-1393); neither the crate name nor the file stem appears, and the extractor walks each file from an empty container. For a fixture crate, `quire coverage --json` reports exactly these `(path, qualified_name)` pairs for its four tagged tests: `(tests/x.rs, top_level)` for a top-level test, `(tests/x.rs, inner::nested)` for a test in `mod inner`, `(src/foo/bar.rs, tests::t)` for a test in `mod tests` of the non-root source file `src/foo/bar.rs`, and `(tests/x.rs, param)` for an `#[rstest]` function `param` written with `#[trace]` above `#[rstest]` (the only order `ix-trace-rs` FR-005 supports; the source function, not its generated cases) (CR-211, PLAT-333; `ix://agent-ix/quoin/FR-120`, emitter rule E-2). | Test |
| FR-051-AC-33 | One emitter chain over a fixture crate demonstrates `ix://agent-ix/quoin/FR-120` (E-10) with the `ix-trace-rs` `emit` feature and `ix-trace-run` runner (`ix-trace-rs` FR-005, FR-006): a `#[trace]` marker in source; a run writing JUnit whose `<testcase>` for each of the four tests of FR-051-AC-32 carries `name` and `classname` equal to that test's `qualified_name`, a `file` attribute equal to its `path`, and the `trace` property with its ids; `quoin evidence record` over that file; and an obligation bound by the recorded ids. A **second run** of the same crate, in which one tagged test panics (the test fails when the environment variable `IX_TRACE_E2E_FAIL` is set), records that test as `fail` into a fresh evidence store and binds no obligation. No other test of the fixture carries the panicking test's ids (CR-211, PLAT-333). | Test |
| FR-051-AC-34 | When the extractor reads a tree holding `.sh` files, it opens each `.sh` file and every symbol it mints carries language `shell`; `SourceLanguage::Shell` serializes as `shell`; a `.bash` file and an extension-less file with a `#!/bin/sh` line are not read. | Test |
| FR-051-AC-35 | The shell adapter mints one `function` symbol, named by the function alone and spanning its definition, for each spelling `name() { … }`, `function name { … }` and `function name() { … }`, for a `{` on the line after the header, for a subshell body `name() ( … )`, for a name containing `-`, `.` or `:`, and for a definition nested in another function's body. | Test |
| FR-051-AC-36 | The shell adapter mints no function, and no false unterminated-structure diagnostic, for a definition-shaped line inside a single-quoted, double-quoted, `$'…'`, backtick or `$( )` string, a here-string `<<<`, a heredoc body (`<<EOF`, `<<'EOF'`, `<<-EOF`), a comment, or a `name)` `case` pattern, and for a brace group `{ …; }`, `${x:-}}`, `$(( ))` or CRLF line endings; each real function in the same file is still minted. | Test |
| FR-051-AC-37 | The shell adapter mints exactly one `entry_point` symbol for every `.sh` file, qualified name `entry_point`, spanning the whole file, with and without a module supplied; its kind label `entry_point` is part of the symbol identity and of the FR-045 record `kind`, and the symbol-table report lists it as a kind that binds a trace id and carries no `implements`. | Test |
| FR-051-AC-38 | A shell `entry_point` binds a `verifies` id only when its repo-relative path matches an `entry_paths` glob of a loaded shell marker form; an `entry_point` matching none binds nothing, and a shell function binds no `verifies` whether or not its file matches. | Test |
| FR-051-AC-39 | The `entry_paths` globs use the dialect and compile check of `source_exclude` (FR-050-AC-22) and match a path relative to the code root; the globs of all shell marker forms across all loaded modules are one union; `source_exclude` is applied first, so an excluded `.sh` file mints no symbol; a pattern that does not compile is rejected at module load with an error naming `entry_paths` as the key at fault, and an extraction handed uncompiled globs applies none and reports a diagnostic naming the pattern (FR-050-AC-25). | Test |
| FR-051-AC-40 | A shell function reads its tag channel from the contiguous `#` comment lines immediately above its definition (a blank line ends the block), and an `entry_point` reads its from the file's first run of consecutive `#` comment lines that precedes the first command, skipping a `#!` line and blank lines; a comment after the first command, such as one following `set -euo pipefail`, is not in the header channel. | Test |
| FR-051-AC-41 | Only a syntax-tree comment node opens a shell comment: a `#` inside a quoted string, a heredoc body, `$#`, `${#x}`, `${x#pat}`, or a word (`a#b`, a URL fragment) is content and carries no tag. | Test |
| FR-051-AC-42 | Shell tags are read only through a declared `markers` form with `language: shell` and `syntax: comment`; a shell file under a model that declares no such form mints zero `verifies` relations. | Test |
| FR-051-AC-43 | Module load rejects `syntax: comment` on a form that is not `language: shell`, rejects any undeclared `trace_tags` key, naming it, and rejects `entry_paths` on a form that is not a `language: shell` form in `trace_tags.markers`, each with an error naming the key at fault; a `language: shell` form in `trace_tags.implements` is accepted, and neither carries `entry_paths` nor binds an `entry_point`. | Test |
| FR-051-AC-44 | A declared shell marker that matches in a comment but binds nothing is reported in `CoverageReport.diagnostics`, naming the id, the repo-relative path and the line, with the reason `outside-entry-paths` when it sits in an `entry_point` header whose file matches no glob, `function-not-evidence` when it sits in a shell function's comment block (a shell function binds no `verifies`, AC-38), and `not-in-tag-channel` when it sits in a comment that is neither a function's block nor a header; no census count moves. | Test |
| FR-051-AC-45 | An unparseable `.sh` file (a tree carrying an error over its declaration structure) yields a per-file diagnostic naming the file and the line the error sits at, while the rest of the tree extracts normally. | Test |
| FR-051-AC-46 | A fixture audit script matching a declared `entry_paths` glob whose header comment carries a declared shell marker backs the `Static` matrix row it names in the coverage report, with the `verifies` relation on its `entry_point`; the same marker in a script matching no glob backs nothing and is reported by AC-44. | Test |
| FR-051-AC-47 | A Python test symbol's `qualified_name` is its bare function name for a module-level function and `Class.method` (nested classes dot-joined) for a method, the file stem never a prefix; a parametrized test is one symbol named by its source function; a TypeScript test symbol's `qualified_name` is the registered title exactly as written in source, an enclosing `describe` never qualifying it, and an `it.each`/`test.each` registration is one symbol named by its template title as written. For a fixture tree, `quire coverage --json` reports exactly these `(path, qualified_name)` pairs for its tagged tests: `(tests/test_x.py, test_top)` for a module-level pytest function, `(tests/test_x.py, TestC.test_m)` for a method of class `TestC`, `(tests/test_x.py, TestOuter.TestInner.test_n)` for a method of class `TestInner` nested in class `TestOuter` (both classes `Test`-prefixed, so the extractor and pytest both treat their methods as tests), `(tests/test_x.py, test_param)` for a `@pytest.mark.parametrize` function, `(tests/x.test.ts, adds)` for an `it('adds', …)` inside two nested `describe` blocks, and `(tests/x.test.ts, adds %s)` for an `it.each([…])('adds %s', …)` registration (PLAT-334, PLAT-335; the emitters' name equality with these pairs is `ix://agent-ix/quoin/FR-120` emitter rule E-2, and the emitter requirements are owned by `ix-trace-py` FR-001 and `ix-trace-ts` FR-001). | Test |
| FR-051-AC-48 | One emitter chain per language over the fixture tree of FR-051-AC-47 demonstrates `ix://agent-ix/quoin/FR-120` (E-10) with the `ix-trace-py` pytest plugin and the `ix-trace-ts` vitest reporter (`FR-001` in each of those repositories, which now own the emitter requirements): a run with `QUIRE_TRACE_EMIT=1` writes JUnit whose `<testcase>` for each tagged test carries `name` and `classname` equal to that test's `qualified_name`, a `file` attribute equal to its `path`, and the `trace` property with its ids; `quoin evidence record` over that file; and an obligation bound by the recorded ids. A **second run** of the same suite, required for each language, in which one tagged test fails (the pytest test fails when the environment variable `IX_TRACE_E2E_FAIL` is set, and the vitest test throws under the same variable) records that test as `fail` into a fresh evidence store and binds no obligation. The failing test calls `trace()` (vitest) or carries its marker (pytest) before it fails, so its recorded entry carries its ids and the unbound result is the failure's effect rather than the absence of ids. No other test of the fixture carries the failing test's ids (PLAT-334, PLAT-335). | Test |
| FR-051-AC-49 | An id-shaped token found by the advisory in a Rust, Python or TypeScript comment or docstring attached to a declaration yields one finding in `CoverageReport.diagnostics` under reason `trace-id-outside-marker` at FR-057 severity `warning`, naming the file, the 1-based line the token sits on, the qualified symbol, the id verbatim, and, for a symbol of an evidence kind, the exact text to write instead (AC-22 states what a non-binding kind receives). That text is rendered from the `template` of the first declared `markers` form for the file's language, with the id as a quoted literal (`#[trace("FR-007-AC-1")]`); where that form declares no template the finding names the form instead, and where the module declares no marker form for the language no finding is emitted because there is nothing to write. One finding per occurrence, an occurrence being one id on one line: a block naming three ids on three lines yields three findings, never a count and never a sampled subset, and the same id written twice on one line yields one. The finding is additive to the FR-050-AC-39 `unmatched_tags` record for the same token: it replaces neither that record nor the `mentions` entry, and neither suppresses it. The finding is emitted from the first release carrying it, behind no flag and with no quiet period. The advisory binds nothing and moves no total. An id that an `implements` form read is reported only under FR-050-AC-62, and a shell comment is reported only under AC-44. A token counts only when its **kind** — the leading alphabetic prefix, compared without case (`FR` for `FR-007-AC-1`) — is the kind of at least one id the loaded corpus's declared trace targets minted; when the corpus minted no id, no kind is declared and every id-shaped token counts (CR-215). The same kind test applies to `unmatched_tags` (FR-050-AC-39) and `mentions` (FR-077-AC-2); the census `tagged` count is unchanged. | Test |
| FR-051-AC-50 | The advisory fires whether or not the id is also backed: a token in a comment on a symbol that also carries a marker naming the same id, or a different id, still yields its finding, and a token inside a symbol's body or inside a string literal outside the annotation channel (AC-24) yields none. A tree whose every tag is a marker yields no `trace-id-outside-marker` finding, and fixing every reported occurrence silences the diagnostic. | Test |
| FR-051-AC-51 | A TypeScript `const`, `let` or `var` declarator, `export`ed or not and alone or inside a multi-declarator statement, whose initializer is a **wrapper chain** — a curried call `callee(...)(argument)` whose single argument is an arrow function, a function expression, or another wrapper chain — mints a Function symbol named by the declarator, with the identity and position of the arrow-valued declarator it parallels. An arrow-valued declarator keeps its own rule, a function-expression-valued declarator that is not wrapped still mints nothing, and a call that is not a curried call with one such argument (`memoize(fn)`, `callee(...)(memoize(fn))`) mints nothing. The rule reads only the shape of the call and names no function, so the engine carries no wrapper name; the module's `typescript_wrapper` form (FR-062) decides which chains bind a scope id. The symbol is a production Function, so the census `candidates` denominator is unchanged (PLAT-866). | Test |
| FR-051-AC-52 | The extractor reads a file whose extension is `.native` through the QSL adapter and reports it as language `qsl`, and `SourceLanguage::Qsl` serializes as `qsl`. Every symbol it mints has kind `declaration` and id `qsl:<repo-relative path>::<label>`; a declaration head is a column-0 line at brace depth 0 matching `^([a-z][a-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)` whose keyword is not `language`, `profile` or `model` (FR-094). A repository with no `.native` file reports no `qsl` language, and a `.native` file under a model declaring no `formalizes` form mints symbols that carry no relation. | Test |
| FR-051-AC-53 | The QSL adapter is line-level and needs no Cargo feature: a `.native` file whose braces do not balance yields a per-file diagnostic naming the file and the line, and the rest of the tree extracts normally (CON-2). | Test |
| FR-051-AC-54 | A symlink that survives rules 1-3 of the Source walk yields no symbol and one per-file diagnostic, reason `symlink-not-followed`, at severity `warning` (as AC-28 does), naming its repo-relative path, whatever its name and whether its target is a file, a directory, absent or outside the root. A symlink pruned or ignored by rules 1-3, or matching a `source_exclude` glob, is not reported; a symlink named like source and matching a glob counts in `excluded_source_files` like any such file. A symlinked file and its in-tree target mint one set of symbols, not two. A tree with no surviving symlink yields no such diagnostic, and one tree extracts identically with its link targets present or deleted (CR-423, PLAT-1793). | Test |
| FR-051-AC-55 | A source file removed by rule 3 yields no symbols and no trace bindings, identically with and without a `.git` directory; un-ignoring the file yields its symbols. The coverage report carries `ignored_source_files`, the count of supported-extension, non-hidden regular files removed by rule 3 (including those under an ignored directory), omitted at zero, where the report is byte-identical to one from an engine predating it, exactly as `excluded_source_files` is (FR-050-AC-24); a `.gitignore` line that removes a test directory is therefore visible in the report. A file matched only by a user-global ignore file, by `.git/info/exclude` or by an ignore file above the repository root is extracted. A repo-root `.gitignore` entry such as `packages/x/node_modules/` still removes those files for the code root `packages/x`. Hidden controls: a hidden-named directory holding a `.rs` file and a hidden-named source file (`.gen.rs`) each yield no symbols, and their non-hidden twins yield theirs. Extraction of one fixture tree is byte-identical under any `HOME` and `XDG_CONFIG_HOME` (CR-423, PLAT-1793). | Test |
| FR-051-AC-56 | When AC-31 reports colliding symbol ids, the symbol-table report (AC-23) attributes each declaration's own ids: a symbol record's `trace_ids` and `implements` hold exactly the relations bound on that declaration, so a second `#[cfg]` variant carrying a marker lists its ids on its own record rather than an empty list and the first variant never lists both. The set of relations in the graph is identical under any order the colliding declarations are discovered in; a coverage criterion backed only by the second variant is backed; and an inverse query by the shared `path#qualified_name` ref (FR-077-AC-4) returns the claims of all colliding declarations. A control file with no collision attributes exactly as before (CR-423, PLAT-1793). | Test |
| FR-051-AC-57 | Symbol **identity** (id, kind, qualified name, language, path), the relations and mentions each symbol carries and the counts are unchanged by an edit that touches no part of that symbol, and by presenting the same tree in a different file-discovery order: for every file not edited the extraction is byte-identical, and in the edited file every symbol the edit does not touch keeps its identity and relations. Position attributes (`line`, `end_line`, leading line, `column`) are non-identity (Symbols section) and are asserted separately: they are unchanged in files not edited and for symbols above an edit, and a symbol below an insertion in the same file keeps its identity while its lines shift by the inserted line count. A marker naming an id that no declared target mints still binds its `verifies` relation, because the id is matched exactly with no hierarchy expansion (FR-077-AC-6) and is returned by a forward query (FR-077-AC-1); it is reported in `untracked_symbols` (FR-050-AC-5). FR-050-AC-39 does not apply to it: that record is for generic id-shaped tokens no declared form bound. Correcting or removing the stale marker changes only that symbol's relations and its `untracked_symbols` entry (CR-423, PLAT-1793). | Test |

> **CR-423 note (2026-11-22, PLAT-1793):** Spec-only; no code or corpus changed here. The
> extractor's walk (`src/symbols/mod.rs`, `extract_tree_scoped`) uses the `ignore` crate with
> its defaults, so behaviour depended on the machine: `.gitignore` is honoured only inside a
> git work tree (`require_git` is on), the user's global git ignore file is read, and a
> symlinked directory is silently not entered. A symlinked file is read today
> (`Path::is_file` follows it), so AC-54 *changes* that case. None of this was reported, so
> extraction could differ per machine with no diagnostic. The corpus walk
> (`src/corpus/walk.rs`) does **not** match the new rules: it sets `require_git(false)` and
> `follow_links(false)` (FR-024-AC-5) but also enables `git_exclude` and `parents` and leaves
> the global ignore file at its default, so it consults `.git/info/exclude`, ancestor files at
> any depth and the user-global file. Aligning the corpus walk is not decided here.
>
> **Rule 3 changes repository behaviour.** The code walk reads ancestor ignore files today
> without bound; bounding them at the repository root keeps committed in-repo ignores for a
> subdirectory scope and drops only files outside the repository and the user's global file.
> The non-configurable template-tree prune (rule 1) already exists and is preserved, not
> added. Silent loss to an ignore file is closed by the `ignored_source_files` count, on the
> existing `quire coverage` report; no new consumer or API is introduced. Symlinks are
> diagnosed from link metadata alone, because classifying by target would make the diagnostic
> depend on whether the target exists.
>
> AC-31 already reports colliding ids, including Rust `#[cfg]` variants, and is sufficient for
> reporting. The unstated part was attribution: the symbol table hands every id of a
> collision to the first record (`symbol_table.rs`, `verifies.remove(symbol.id)`) and an empty
> list to the later ones; AC-56 states the per-declaration outcome that fails it. AC-57 keeps
> identity and position separate (the Symbols section makes line a non-identity attribute) and
> states what a stale marker actually does. Exact versus parent ids (FR-077-AC-6), mention
> versus claim (FR-077-AC-1/2, AC-49/50), ignored tests (AC-27) and duplicate status ids
> (FR-050-AC-23) are already covered and not repeated. Per project rule, each of AC-54..57
> lands with a declarative corpus case, `issue_ref` `PLAT-1793`, a healthy control and absence
> assertions; that is implementation work.

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the declared trace-tag grammar), [FR-045](./FR-045-filament-core-extraction-engine.md) (record shapes, id and dedup conventions), [NFR-006](../non-functional/NFR-006-determinism.md) (determinism discipline)
- **Downstream**: the coverage rollup ([FR-050](./FR-050-declarative-coverage-computation.md)), the `gap-analysis` semantic review, and filament-core knowledge-graph ingestion consume the symbol graph; [FR-065](./FR-065-controlled-corpus-contract.md) (the corpus's L2 localisation level reads `binding_census.unbound_example` from this extraction)
- **Companion deliverables (outside quire-rs core)**: the canonical markers imply per-language support packages — a pytest plugin registering the `trace` marker, a lightweight Rust no-op proc-macro crate, and an npm helper for vitest/jest — separate deliverables following the separate-workspace-crate pattern (ADR 0010 placement precedent); this FR specifies only the static parsing contract the extractor holds them to
- **Emitter owners**: `FR-001` in `agent-ix/ix-trace-py` (pytest plugin) and in `agent-ix/ix-trace-ts` (vitest reporter) now own the emitter requirements, as `ix-trace-rs` does for Rust; FR-051-AC-47 and FR-051-AC-48 are this repository's extractor-side companions to them (`ix://agent-ix/quoin/FR-120`).
