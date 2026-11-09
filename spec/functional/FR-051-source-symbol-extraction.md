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
`TC-041`) in the comment or docstring block attached to a declaration. It is
detection only: it reconstructs no id, is not specialised per language, and
produces no binding and no rewrite. For **every** occurrence, from the first
release carrying it and whether or not any other form also bound that id, it
emits one warning naming `file:line` and the canonical text to write, rendered
from the loaded module's declared marker `template` for that file's language
(AC-49). The warnings are the worklist for a repository that has not yet
written markers; the work is done when they stop.

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
| FR-051-CON-1 | The extractor SHALL classify syntax from a syntax tree that a grammar-driven parser library produces over the source text, and performs no network I/O, no service I/O, no execution of extracted code, no compilation of the analysed tree, no dependency installation, and no type resolution. This holds for every adapter, shell included (the `tree-sitter-bash` grammar), and the shell adapter never executes or sources the script it reads. | Architecture | Test |
| FR-051-CON-2 | Adapters SHALL degrade per file: one unparseable file never aborts the tree extraction. | Operational | Test |
| FR-051-CON-3 | Only marker forms bind a `verifies` relation: the `verifies` binding path, the `VerifiesRelation` and symbol-graph types, the `trace_tags.markers` schema and the binding census SHALL carry no field, variant, list, template, branch or compatibility path for any other trace-text form (CR-215, PLAT-847). The `trace_tags.implements` textual forms of FR-062 are outside this constraint. | Architecture | Inspection |
| FR-051-CON-4 | The order of work that removes non-marker binding is a precondition of the change, not a follow-up: first, every symbol that binds only through a non-marker form is retagged with its canonical marker (the `quire-rs` symbols first, then `quire-cli`, then `quoin`); second, the module `trace_tags` entries that declare non-marker forms are removed from `spec-artifacts-process` in a cross-repo change, and that module release precedes the engine release that deletes the binding machinery or ships with it, because the new engine rejects an undeclared `trace_tags` key (AC-43) and so could not load a module that still carries them, while an earlier engine loads a module without them; third, the binding machinery is deleted. The per-repository coverage delta (rows backed before and after, for every repository retagged, and the symbols that were tagged only through their name, which the change leaves unbacked and unreported) is measured and published with the change, stating that the numbers moved because the evidence standard changed and not because tests were lost (CR-215). | Operational | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-051-AC-1 | Each adapter extracts functions, test functions, and containers from a fixture tree (the shell adapter: functions and the `entry_point`), and each symbol carries language, repo-relative path, qualified path, kind, and a line attribute. | Test (TC-741) |
| FR-051-AC-2 | Reformatting a fixture file (whitespace and line-number changes only) leaves every symbol id unchanged; renaming a symbol changes only that symbol's id. | Test (TC-742) |
| FR-051-AC-3 | Rust `#[test]`-family functions, Python `test_` functions and direct imported unittest.TestCase methods in the bounded forms above, and TypeScript `test`/`it` registrations classify as test symbols; sibling non-test symbols do not. **Since CR-187:** TypeScript `xtest`/`xit` base names classify as test symbols exactly as `test`/`it` do (they mint with `ignored: true`, FR-051-AC-27) — no new registration shape, only the admitted base-name set widens. | Test (TC-743, TC-1803, TC-1940) |
| FR-051-AC-4 | Each canonical marker form binds statically: a Python `@pytest.mark.trace(...)` decorator, a Rust `#[trace(...)]` attribute, and a TypeScript `trace(...)` helper each mint one `verifies` relation per attached trace id, with no code executed. | Test (TC-744) |
| FR-051-AC-5 | Marker and tag forms are module data: a fixture model declaring different marker names/patterns binds by its own declaration, and with no declared forms the extractor mints zero `verifies` relations. | Test (TC-745) |
| FR-051-AC-6 | A trace id attached more than once to one symbol (repeated marker) mints one `verifies` relation and one diagnostic. | Test (TC-746) |
| FR-051-AC-7 | The emitted records match the FR-045 graph-record shapes with normalized `ref` values, and filament-core ingestion fixtures accept them unchanged. | Test (TC-747) |
| FR-051-AC-8 | `defined_in` edges link every symbol to its file and `contains` edges link containers to members, deterministically ordered. | Test (TC-748) |
| FR-051-AC-9 | An unparseable fixture file yields a per-file diagnostic naming that file and the line the error sits at, while the rest of the tree extracts normally. | Test (TC-749, TC-1884) |
| FR-051-AC-10 | Repeated extraction over an identical fixture tree, at a pinned grammar version (`tree-sitter-rust`/`tree-sitter-python`/`tree-sitter-bash`, locked in `Cargo.lock`) and a fixed Cargo feature set (`rust-symbols`/`python-symbols`/`shell-symbols`, which change whether Rust/Python extract at all — see CR-177, CR-179), emits byte-identical JSON and identical record ids. | Test (TC-750) |
| FR-051-AC-11 | Only a marker form binds (CR-215, PLAT-847): a fixture whose tests carry a bare id in a docstring, a doc comment or a line comment, a `Trace:` line, or an id in the test's own name, and no marker, mints zero `verifies` relations and backs no row, in every language; the same tests with a marker bind. A `verifies` relation carries its symbol, trace id, form, path and line and no provenance field, the symbol graph carries no rewrite list, and the binding census carries no field for a non-marker form. | Test (TC-2160) |
| FR-051-AC-12 | Comment recognition is string-aware, and template-literal state carries across lines: a `//` or `/*` inside a string or template literal is content, not a comment opener, whether it sits on the literal's opening line or a continuation line. | Test (TC-798, TC-799) |
| FR-051-AC-13 | A declaration whose signature spans lines binds tags in its docstring: a `def` wrapped by a formatter has the same span as the unwrapped form. | Test (TC-800) |
| FR-051-AC-14 | One reading of a file decides what is code in it, and every consumer reads that one answer — whether the file is accepted at all, and the span a symbol ends at — so two consumers of the same file never disagree about whether a given byte is code, a comment, or the inside of a string. A comment trailing on the same line as the statement before a declaration is that statement's own trailing note, not the following declaration's leading annotation (PLAT-882). A leading comment reaches a TypeScript registration through an `await` wrapper the same as without one — `await` is not a boundary a leading annotation search stops at (PLAT-882). | Test (TC-803, TC-1924, TC-1926) |
| FR-051-AC-15 | The Rust adapter reads raw strings, lifetimes, character literals and nested block comments as the language defines them, so a brace or a quote inside any of them is content: the file is accepted and yields its symbols. String state carries across lines: a raw string or an ordinary string opened on one line keeps its interior as content on the lines that follow, until its own closing delimiter — for a raw string, the matching hash count — so a `//`, a brace or a quote is content whether it sits on the literal's opening line or a continuation line, and code after the closer is code again. | Test (TC-804) |
| FR-051-AC-16 | A marker form mints one `verifies` relation per trace id its match carries, so a marker naming several ids binds every id rather than only the first, with irregular spacing and a trailing comma yielding the ids and never an empty one. | Test (TC-806) |
| FR-051-AC-17 | A Rust benchmark — an attribute-marked one, or a function a `criterion_group!` registers in either invocation form, whether or not the registration line carries a trailing comment — classifies as a benchmark symbol, and a `fuzz_target!` invocation mints one fuzz-target symbol per file whose span is its whole file. Both bind trace ids; a container and a plain function still bind none. Each kind's stable label (`benchmark`, `fuzz_target`) is part of the symbol identity and of the FR-045 record's `kind` field. | Test (TC-827, TC-828) |
| FR-051-AC-18 | A `test`/`it`/`xtest`/`xit` registration (CR-187 widens this AC's own base-name set to match AC-3's) whose modifier chain is curried (`it.skipIf(cond)(…)`, `it.each([…])(…)`, `xit.each([…])(…)`), whose chain carries whitespace anywhere — before `(`, between curried groups, or before a `.` modifier (PLAT-882) — or whose title literal begins on a later line, registers a test symbol named by that title, with the span and leading block any other registration gets: the tree names a call's own first argument directly, so there is no longer a scan or a window bounding how far down the file that argument may sit (PLAT-882, superseding CR-084's own bounded-scan mechanism — see CR-180). A registration carries a callback as its second argument or it registers nothing: a title held in a variable, an identifier merely beginning with `it`, and a call with no second argument each register nothing rather than something wrong. A title inside a multi-line template literal is out of scope and registers nothing (CR-084). | Test (TC-943, TC-948, TC-958, TC-960, TC-961, TC-1920, TC-1921, TC-1940) |
| FR-051-AC-19 | Binding reports a per-language census of what it examined: `candidates` counts evidence symbols whose kind admits a trace tag; `tagged` counts candidates whose attached annotation block carries a generic id-shaped token or whose declared form bound; `bound` counts candidates that minted at least one `verifies` relation; and `forms` names every declared marker form consulted. The hard invariant is `bound <= tagged <= candidates`. An unbound candidate carries `unbound_example`; a tagged-but-unbound candidate additionally carries `unmatched_example`, each the deterministic lowest `path:line`. Generic tag detection is independent of the declared grammar and does not inspect the test body. Counts are symbols rather than relations, containers and production functions are not candidates, rows are language-ordered, and the candidate population does not depend on declared patterns (CR-093, CR-142). | Test (TC-982, TC-1060) |
| FR-051-AC-20 | A Python triple-quoted string is content wherever its delimiter sits: a body opened mid-line (`FIXTURE = """`) is never read as code, and a declaration following an embedded string keeps its true container rather than resuming the scope that was open when the string began. Both delimiter kinds and every string prefix (`f`, `r`, `b`, `rb`, `u`) are recognised as openers; a triple delimiter written inside a single-quoted string, escaped, or after a `#` comment marker opens nothing, and a `#` inside a triple-quoted string ends nothing (CR-115). | Test (TC-1029, TC-1030, TC-1031) |
| FR-051-AC-22 | An id-shaped token the advisory finds in the annotation block of a symbol whose kind cannot bind a trace id (a module, a container, a production function) is reported by AC-49 naming the id and the symbol's kind, with no `#[trace(...)]` replacement text, because a `trace` marker on that kind binds nothing (FR-062); where the kind is a production function or container the finding names instead the text rendered, as FR-050-AC-62 renders it, from the `template` of the first declared `syntax: rust_attribute` `implements` form that declares one (`#[implements("FR-007-AC-2")]`; never the textual `rust-implements-line` form), and where no such form declares a template it names the kind alone, as it does for a module. It binds nothing, and no count in the payload moves, so the census `candidates` denominator is unchanged. A canonical marker on such a symbol is not reported: the advisory reads comment and docstring lines, never an attribute or decorator line, so `#[trace("X")]` on a production function yields no finding. Where several symbols span one token the **innermost** is named, because the fix a reader needs is the symbol the token sits on and not the module that contains it; at equal `leading_line` a container loses. A token is reported once however many symbols span it. | Test (TC-1044, TC-1046, TC-1081, TC-2165) |
| FR-051-AC-23 | The extracted symbol table is reported on its own surface, as the engine built it: per symbol its path, qualified name, kind, language, declaration line, **declaration column** (CR-187: the 1-based UTF-8 byte column of the declaration's own start, as tree-sitter itself reports it — all four adapters parse through tree-sitter at this head), annotation-block line, end line, container, whether that **kind** can bind a trace id, whether it can carry an `implements` marker, whether AC-27 marks it (or its container) ignored, and — when a module is supplied — the ids it bound. A scanner defect must be sizeable **without reimplementing the scanner**: three ports of `symbols/python.rs` gave 386, 490 and 5,263 lost declarations over one tree, disagreeing precisely where the original is wrong. Binding is reported as **not asked** rather than as zero when no module is supplied, because an unbound run and a repository nobody tagged produce the same empty list. Per-language totals carry both denominators — symbols examined and symbols of a **binding kind** — since a rate over the wrong one reads a tree of containers as untagged. | Test (TC-1052, TC-1053, TC-1054) |
| FR-051-AC-24 | String-literal contents are masked before the advisory (AC-49) matches, in **every** language rather than in Rust alone — a trace id a file carries as data is not an authored tag, and reporting it sends a reader to rewrite data. The mask preserves each language's **annotation channel**: comments everywhere, and additionally a Python **docstring**, which is a string literal by syntax. Byte length and line count are preserved so a reported `file:line` is the line in the original. Markers are never masked against: they put ids inside string literals by design. | Test (TC-1055, TC-1056, TC-1057) |
| FR-051-AC-21 | A TypeScript `describe(...)` / `suite(...)` registration mints one **container** symbol named by its registered title, spanning its block and its leading annotation block, and the registrations written inside it carry it as their container rather than the file's module — a `describe(` whose `{` falls on a later line (a callback wrapped for width, like any other registration under AC-18) still parents its members to it: the tree, not a same-line text check, decides what is inside a suite's callback (PLAT-882, superseding the pre-tree-sitter scanner's own same-line requirement — see CR-180). A suite does not *name* its members: a registration's qualified name is its own registered title whether or not a suite encloses it, while a class inside a suite still qualifies its own. A suite is a grouping and not evidence, so a trace tag on a suite header mints **no** `verifies` relation and the suite is **not** a `binding_census` candidate — and since `agent-ix/quire-rs#312` that tag is **reported** rather than dropped, under AC-22. The tag naming a test is the declarative form; a tag on the group would make coverage an inference about which test inside is meant, and the report says where the tag is instead of guessing. A registration whose name chain names a suite **anywhere along it** is a suite: `test.describe(...)` and `it.describe.only(...)` classify exactly as `describe(...)` does, because a harness spells its suite as a member of its test namespace and reading only the first identifier gave one construct two classifications (CR-121). `context` is not a suite name. **Since CR-187:** `xdescribe` classifies as a suite base name exactly as `describe`/`suite` do (it mints with `ignored: true`, FR-051-AC-27, inherited by every member the tree still parents to it) — no new container shape, only the admitted base-name set widens. | Test (TC-1039, TC-1040, TC-1042, TC-1940) |
| FR-051-AC-25 | A symbol and its trace ids are read from the declaration's own node in the syntax tree, so attachment rather than page position decides them: a Rust test whose attribute block carries `#[test]` in any position, a Rust declaration whose attribute block a formatter has split across lines, a Rust declaration preceded by a block doc comment (`/** ... */`), a Python marker a formatter has wrapped across lines, a marker separated from its `def` by a second wrapped decorator, and a declaration whose signature spans lines each classify and bind exactly as their single-line, adjacent spelling does (`agent-ix/quire-rs#387`, `agent-ix/quire-rs#395`, `agent-ix/quire-rs#459`, `agent-ix/quire-rs#843`). A brace inside a regular-expression literal is content — the ground AC-12 and AC-15 hold for the other literal forms — so a file carrying a `{n,m}` quantifier yields its symbols (`agent-ix/quire-rs#424`). | Test (TC-1879, TC-1880, TC-1881, TC-1885, TC-1886) |
| FR-051-AC-26 | Extraction over a fixture tree that has never been compiled and whose declared dependencies are not installed yields that tree's symbols and relations, and the extraction path invokes no compiler, package manager, or build artifact of the analysed tree and resolves no type. | Test (TC-1882, TC-1883) |
| FR-051-AC-27 | A test-kind or suite-kind symbol carries `ignored: bool`, true exactly for a **statically decidable** language form (CR-187, PLAT-1077; SR-117 FND-003/FND-004 narrowed this from the first draft): **Rust** — a `#[ignore]` attribute (with or without a reason string), before or after `#[test]`, on any function AC-3 already classifies as a test (which already includes the `#[tokio::test]`/`#[rstest]`/`#[wasm_bindgen_test]` family; no new attribute vocabulary is added for `ignored` itself). **TypeScript** — a `.skip` modifier anywhere in a registration's chain (`it.skip(...)`, `describe.skip(...)`), or a registration whose base name is `xit`/`xtest`/`xdescribe` (AC-3, AC-21). **Python** — `@unittest.skip(...)`/`@pytest.mark.skip(...)` on a function or method, the same decorator at class level on a `unittest.TestCase` subclass or a pytest test class, or a module carrying `pytestmark = pytest.mark.skip(...)` (bare `@pytest.mark.skip` with no call, and the aliased-import forms AC-3's own unittest binding already resolves, count identically). A suite's or class's `ignored` is inherited by every test-kind or suite-kind member nested in it, transitively, via the same `contains` edges AC-8 already emits — a test's own marking is not required when its container is ignored. Inheritance follows the enclosing container at the member's own declaration site, never a qualified-name lookup, because AC-21 lets two TypeScript suites share a title (CR-189). **Explicitly excluded, because each may still run:** Rust `#[cfg_attr(..., ignore)]` (conditional); TypeScript `.skipIf(cond)`, `.todo`, `test.fixme`; Python `skipif`/`skipIf`/`xfail`. A form the extractor does not otherwise classify as a test attribute at all — for example a bare `#[should_panic]` — is out of scope for `ignored` the same way it is out of scope for AC-3. A symbol carrying none of its language's ignored forms, and contained in no ignored container, has `ignored: false`. Marking a symbol ignored manufactures no new *relation* or *diagnostic* — a tag on an ignored symbol still binds a `verifies`/`implements` relation exactly as it would on a running one — but it is not silent on *symbols*: `xit`/`xtest`/`xdescribe` are registrations AC-3/AC-21 now mint (where they minted nothing before CR-187), each carrying `ignored: true` from the moment it exists. `ignored` is carried on the FR-051-AC-23 symbol-table record and on `VerifiesRelation`, serialized only when `true` (`skip_serializing_if`), so a payload over a corpus with no ignored symbol — including every existing fixture — stays byte-identical to TC-750's pre-CR-187 baseline. | Test (TC-1939, TC-1940, TC-1941, TC-1943) |
| FR-051-AC-28 | A range — two id-shaped tokens joined by a bare `..`, whether same-prefix, differing-prefix, or short-suffix — inside a marker's argument or inside a textual `implements` use (`/// Implements: FR-001..FR-003`, `# Implements: FR-001..FR-003`), binds no id and mints no `verifies`/`implements` relation for any id it names, including its endpoints. One finding is reported per occurrence, in `CoverageReport.diagnostics` under reason `range-in-trace-tag` at FR-057 severity `warning` exactly as FR-050-AC-50 states it, naming the file, the 1-based line the range text sits on, the qualified symbol, and the literal range text verbatim (CR-189); the occurrence contributes to neither `verifies`/`implements` nor the generic id-shaped-token channels (`unmatched_tags`, `mentions`) — the range finding is the only record of it. A marker range used to mint one relation keyed on the whole literal string; that stops, so `verifies`/`implements` counts over a corpus carrying one can change (CR-187, PLAT-1077). Only the range token is inert: a marker that continues past a range (`#[trace("A..B", "C")]`) still binds every other id it names (CR-189). | Test (TC-1936, TC-1937, TC-2166) |
| FR-051-AC-29 | A build that has no symbol adapter for a language whose source is in the tree (the `wasm` build drops all four adapter features) records that language as unavailable on the extraction, the symbol graph and the symbol-table report, with no symbols and no per-file diagnostic for those files; a tree with no file in an unavailable language is unaffected. Satisfied only when the unavailability is recorded: a zero-symbol extraction with no unavailable-language fact for a tree holding source the build cannot read fails this criterion (CR-201, PLAT-874). | Test (TC-2060) |
| FR-051-AC-30 | Emitting FR-045 graph records from an extraction that records an unavailable language is refused with an error naming the language, not returned as an empty node and edge set (CR-201). | Test (TC-2064) |
| FR-051-AC-31 | When two or more symbols extracted from one file share a symbol id, extraction reports one per-file diagnostic per colliding id, reason `symbol-id-collision`, at severity `warning` (as AC-28 does): it is advisory, appears on the symbol-table report with its severity, and never changes an exit code or refuses a result. The diagnostic names the kind, the qualified name, and every colliding declaration by `path:line`, followed by that declaration's `#[cfg(...)]` / `#[cfg_attr(...)]` outer attributes, whitespace-collapsed to one line, where the adapter records them (other attributes are not listed; adapters without such conditions list none). It reports shared identity and does not claim a defect: causes include mutually exclusive `#[cfg]`-gated variants, Rust trait-impl methods differing only by generics (`From<A>` / `From<B>`), a Python function defined in both branches of an `if` or a nested helper repeated across scopes, and TypeScript registrations with one title. Symbol ids are unchanged, and a file with no collision yields no such diagnostic (CR-206, PLAT-877). | Test (TC-2082) |
| FR-051-AC-32 | A Rust test symbol's `qualified_name` is the `::`-joined chain of its enclosing `mod`, `struct`, `enum`, `trait`, the `impl` target type (never the trait) and an enclosing `fn`, then the function; neither the crate name nor the file stem appears, and the extractor walks each file from an empty container. For a fixture crate, `quire coverage --json` reports exactly these `(path, qualified_name)` pairs for its four tagged tests: `(tests/x.rs, top_level)` for a top-level test, `(tests/x.rs, inner::nested)` for a test in `mod inner`, `(src/foo/bar.rs, tests::t)` for a test in `mod tests` of the non-root source file `src/foo/bar.rs`, and `(tests/x.rs, param)` for an `#[rstest]` function `param` written with `#[trace]` above `#[rstest]` (the only order `ix-trace-rs` FR-005 supports; the source function, not its generated cases) (CR-211, PLAT-333; `ix://agent-ix/quoin/FR-120`, emitter rule E-2). | Test (TC-2091) |
| FR-051-AC-33 | One emitter chain over a fixture crate demonstrates `ix://agent-ix/quoin/FR-120` (E-10) with the `ix-trace-rs` `emit` feature and `ix-trace-run` runner (`ix-trace-rs` FR-005, FR-006): a `#[trace]` marker in source; a run writing JUnit whose `<testcase>` for each of the four tests of FR-051-AC-32 carries `name` and `classname` equal to that test's `qualified_name`, a `file` attribute equal to its `path`, and the `trace` property with its ids; `quoin evidence record` over that file; and an obligation bound by the recorded ids. A **second run** of the same crate, in which one tagged test panics (the test fails when the environment variable `IX_TRACE_E2E_FAIL` is set), records that test as `fail` into a fresh evidence store and binds no obligation. No other test of the fixture carries the panicking test's ids (CR-211, PLAT-333). | Test (TC-2092) |
| FR-051-AC-34 | When the extractor reads a tree holding `.sh` files, it opens each `.sh` file and every symbol it mints carries language `shell`; `SourceLanguage::Shell` serializes as `shell`; a `.bash` file and an extension-less file with a `#!/bin/sh` line are not read. | Test (TC-2101) |
| FR-051-AC-35 | The shell adapter mints one `function` symbol, named by the function alone and spanning its definition, for each spelling `name() { … }`, `function name { … }` and `function name() { … }`, for a `{` on the line after the header, for a subshell body `name() ( … )`, for a name containing `-`, `.` or `:`, and for a definition nested in another function's body. | Test (TC-2102) |
| FR-051-AC-36 | The shell adapter mints no function, and no false unterminated-structure diagnostic, for a definition-shaped line inside a single-quoted, double-quoted, `$'…'`, backtick or `$( )` string, a here-string `<<<`, a heredoc body (`<<EOF`, `<<'EOF'`, `<<-EOF`), a comment, or a `name)` `case` pattern, and for a brace group `{ …; }`, `${x:-}}`, `$(( ))` or CRLF line endings; each real function in the same file is still minted. | Test (TC-2103) |
| FR-051-AC-37 | The shell adapter mints exactly one `entry_point` symbol for every `.sh` file, qualified name `entry_point`, spanning the whole file, with and without a module supplied; its kind label `entry_point` is part of the symbol identity and of the FR-045 record `kind`, and the symbol-table report lists it as a kind that binds a trace id and carries no `implements`. | Test (TC-2104) |
| FR-051-AC-38 | A shell `entry_point` binds a `verifies` id only when its repo-relative path matches an `entry_paths` glob of a loaded shell marker form; an `entry_point` matching none binds nothing, and a shell function binds no `verifies` whether or not its file matches. | Test (TC-2105) |
| FR-051-AC-39 | The `entry_paths` globs use the dialect and compile check of `source_exclude` (FR-050-AC-22) and match a path relative to the code root; the globs of all shell marker forms across all loaded modules are one union; `source_exclude` is applied first, so an excluded `.sh` file mints no symbol; a pattern that does not compile is rejected at module load with an error naming `entry_paths` as the key at fault, and an extraction handed uncompiled globs applies none and reports a diagnostic naming the pattern (FR-050-AC-25). | Test (TC-2106) |
| FR-051-AC-40 | A shell function reads its tag channel from the contiguous `#` comment lines immediately above its definition (a blank line ends the block), and an `entry_point` reads its from the file's first run of consecutive `#` comment lines that precedes the first command, skipping a `#!` line and blank lines; a comment after the first command, such as one following `set -euo pipefail`, is not in the header channel. | Test (TC-2107) |
| FR-051-AC-41 | Only a syntax-tree comment node opens a shell comment: a `#` inside a quoted string, a heredoc body, `$#`, `${#x}`, `${x#pat}`, or a word (`a#b`, a URL fragment) is content and carries no tag. | Test (TC-2108) |
| FR-051-AC-42 | Shell tags are read only through a declared `markers` form with `language: shell` and `syntax: comment`; a shell file under a model that declares no such form mints zero `verifies` relations. | Test (TC-2109) |
| FR-051-AC-43 | Module load rejects `syntax: comment` on a form that is not `language: shell`, rejects any undeclared `trace_tags` key, naming it, and rejects `entry_paths` on a form that is not a `language: shell` form in `trace_tags.markers`, each with an error naming the key at fault; a `language: shell` form in `trace_tags.implements` is accepted, and neither carries `entry_paths` nor binds an `entry_point`. | Test (TC-2110) |
| FR-051-AC-44 | A declared shell marker that matches in a comment but binds nothing is reported in `CoverageReport.diagnostics`, naming the id, the repo-relative path and the line, with the reason `outside-entry-paths` when it sits in an `entry_point` header whose file matches no glob, `function-not-evidence` when it sits in a shell function's comment block (a shell function binds no `verifies`, AC-38), and `not-in-tag-channel` when it sits in a comment that is neither a function's block nor a header; no census count moves. | Test (TC-2111) |
| FR-051-AC-45 | An unparseable `.sh` file (a tree carrying an error over its declaration structure) yields a per-file diagnostic naming the file and the line the error sits at, while the rest of the tree extracts normally. | Test (TC-2112) |
| FR-051-AC-46 | A fixture audit script matching a declared `entry_paths` glob whose header comment carries a declared shell marker backs the `Static` matrix row it names in the coverage report, with the `verifies` relation on its `entry_point`; the same marker in a script matching no glob backs nothing and is reported by AC-44. | Test (TC-2113) |
| FR-051-AC-47 | A Python test symbol's `qualified_name` is its bare function name for a module-level function and `Class.method` (nested classes dot-joined) for a method, the file stem never a prefix; a parametrized test is one symbol named by its source function; a TypeScript test symbol's `qualified_name` is the registered title exactly as written in source, an enclosing `describe` never qualifying it, and an `it.each`/`test.each` registration is one symbol named by its template title as written. For a fixture tree, `quire coverage --json` reports exactly these `(path, qualified_name)` pairs for its tagged tests: `(tests/test_x.py, test_top)` for a module-level pytest function, `(tests/test_x.py, TestC.test_m)` for a method of class `TestC`, `(tests/test_x.py, TestOuter.TestInner.test_n)` for a method of class `TestInner` nested in class `TestOuter` (both classes `Test`-prefixed, so the extractor and pytest both treat their methods as tests), `(tests/test_x.py, test_param)` for a `@pytest.mark.parametrize` function, `(tests/x.test.ts, adds)` for an `it('adds', …)` inside two nested `describe` blocks, and `(tests/x.test.ts, adds %s)` for an `it.each([…])('adds %s', …)` registration (PLAT-334, PLAT-335; the emitters' name equality with these pairs is `ix://agent-ix/quoin/FR-120` emitter rule E-2, and the emitter requirements are owned by `ix-trace-py` FR-001 and `ix-trace-ts` FR-001). | Test (TC-2154) |
| FR-051-AC-48 | One emitter chain per language over the fixture tree of FR-051-AC-47 demonstrates `ix://agent-ix/quoin/FR-120` (E-10) with the `ix-trace-py` pytest plugin and the `ix-trace-ts` vitest reporter (`FR-001` in each of those repositories, which now own the emitter requirements): a run with `QUIRE_TRACE_EMIT=1` writes JUnit whose `<testcase>` for each tagged test carries `name` and `classname` equal to that test's `qualified_name`, a `file` attribute equal to its `path`, and the `trace` property with its ids; `quoin evidence record` over that file; and an obligation bound by the recorded ids. A **second run** of the same suite, required for each language, in which one tagged test fails (the pytest test fails when the environment variable `IX_TRACE_E2E_FAIL` is set, and the vitest test throws under the same variable) records that test as `fail` into a fresh evidence store and binds no obligation. The failing test calls `trace()` (vitest) or carries its marker (pytest) before it fails, so its recorded entry carries its ids and the unbound result is the failure's effect rather than the absence of ids. No other test of the fixture carries the failing test's ids (PLAT-334, PLAT-335). | Test (TC-2155) |
| FR-051-AC-49 | An id-shaped token found by the advisory in a Rust, Python or TypeScript comment or docstring attached to a declaration yields one finding in `CoverageReport.diagnostics` under reason `trace-id-outside-marker` at FR-057 severity `warning`, naming the file, the 1-based line the token sits on, the qualified symbol, the id verbatim, and, for a symbol of an evidence kind, the exact text to write instead (AC-22 states what a non-binding kind receives). That text is rendered from the `template` of the first declared `markers` form for the file's language, with the id as a quoted literal (`#[trace("FR-007-AC-1")]`); where that form declares no template the finding names the form instead, and where the module declares no marker form for the language no finding is emitted because there is nothing to write. One finding per occurrence, an occurrence being one id on one line: a block naming three ids on three lines yields three findings, never a count and never a sampled subset, and the same id written twice on one line yields one. The finding is additive to the FR-050-AC-39 `unmatched_tags` record for the same token: it replaces neither that record nor the `mentions` entry, and neither suppresses it. The finding is emitted from the first release carrying it, behind no flag and with no quiet period. The advisory binds nothing and moves no total. An id that an `implements` form read is reported only under FR-050-AC-62, and a shell comment is reported only under AC-44. | Test (TC-2161, TC-2162) |
| FR-051-AC-50 | The advisory fires whether or not the id is also backed: a token in a comment on a symbol that also carries a marker naming the same id, or a different id, still yields its finding, and a token inside a symbol's body or inside a string literal outside the annotation channel (AC-24) yields none. A tree whose every tag is a marker yields no `trace-id-outside-marker` finding, and fixing every reported occurrence silences the diagnostic. | Test (TC-2163) |

> **CR-215 note (2026-11-08, PLAT-847, PLAT-912):** only marker forms bind.
> The ruling is to take one advisory pattern rather than quarantine the
> textual-form handling in a module: that handling is called from five or more
> binder functions and reaches the canonical types (a provenance field on
> `VerifiesRelation`, a rewrite list on `SymbolGraph`, census fields), so
> isolating it is a rewrite and fails the ticket's own containment test. The
> narrative, CON-3, AC-6, AC-11, AC-16, AC-19, AC-22, AC-24, AC-28, AC-42 and
> AC-43 now describe the marker path alone, and AC-49 and AC-50 are new. The
> machinery removed is: `TraceLegacyForm` and the model's `legacy` list,
> `id_format` reconstruction from captures, `rewrite_to`, `RewriteSuggestion`
> and the mechanical rewrite path, `TraceProvenance` and the provenance field,
> `SymbolGraph.rewrites`, and the census fields for non-marker forms; FR-050-AC-44
> (the declaration-name census) goes with them. `TraceMarkerForm` and the
> FR-062 `implements` list are unchanged, and the `/// Implements:` doc-comment
> warning of FR-050-AC-62 (CR-209) is unaffected: it is the `implements` list,
> not this one. There is no deprecation window and no compatibility layer.
> The measured input (PLAT-912, `quire-rs` main, 2026-11-08): 1681 canonical
> relations, 291 non-marker relations (`rust-comment-id` 218, `rust-test-name-id`
> 42, `python-docstring-id` 24, `rust-trace-line` 5, `python-comment-id` 2), 192
> of 1478 backed ids (13%) losing all backing (115 `TC` rows, 72 ACs, 5 other);
> `quire-cli` about 196 and `quoin` about 1282 symbols have no marker. CON-4
> states the order of work this implies. Coverage will drop for a repository
> not yet retagged; that is the evidence standard changing and is published
> per repository with the change.
> The id-embedded-in-name shape has no advisory: detecting it would need the
> id reconstruction this change removes.

> **CR-206 note (2026-11-08, PLAT-877):** AC-31 is new. The id hashes
> language, path, qualified name and kind and nothing evaluates `#[cfg]`, so
> two cfg-gated variants of one item mint one id and consumers saw one symbol
> where two declarations exist. The same collision also arises from Rust
> trait-impl methods that differ only by generics (`From<A>` / `From<B>`)
> and from Python nested helpers repeated across scopes; the diagnostic
> reports shared identity and does not claim a defect. The ruling is to keep identity and report the
> collision instead of resolving it; per-build-configuration extraction is
> not decided here.

> **CR-211 note (PLAT-333):** FR-051-AC-32 pins, as a requirement, the Rust
> `qualified_name` rule that `ix://agent-ix/quoin/FR-120` binds the Rust emitter
> to (`ix-trace-rs` FR-005 and FR-006). It states what the extractor already
> reports; a divergence found by the fixture is a defect in whichever side
> disagrees with the rule. FR-051-AC-33 is the end-to-end demonstration (E-10)
> that quoin's emitter contract requires one repository to host, and it is where
> the runner's output is compared with the extractor's.

> **PLAT-334 / PLAT-335 note:** the pytest plugin and the vitest reporter are
> specified in their own repositories, `agent-ix/ix-trace-py` and
> `agent-ix/ix-trace-ts`, as `ix-trace-rs` is for Rust. FR-051-AC-47 pins, as a
> requirement, the Python and TypeScript `qualified_name` rules that
> `ix://agent-ix/quoin/FR-120` binds those emitters to; FR-051-AC-48 is the
> end-to-end demonstration (E-10) in which each emitter's output is compared with
> the extractor's. A divergence found by the fixture is a defect in whichever side
> disagrees with the rule.

> **CR-201 note (2026-11-08, PLAT-874):** the build-configuration diagnostic
> `rust-symbols`-off builds emitted per Rust file (and its Python and
> TypeScript twins) is removed. It reached coverage as a successful report
> of roughly 0% backing for a repository whose evidence the binary could not
> read, and AC-9 and AC-10 both read satisfied in that state. Incapacity is a
> property of the binary, so it is now a first-class fact (`unavailable_languages`)
> and measurement refuses: AC-29. AC-9's diagnostic channel is again only for
> files whose own content fails to parse. The CR-177 sentence that an off
> `rust-symbols` yields "a build-configuration diagnostic" is superseded by
> this note. Coverage, trace search and the assurance export refuse the same way: FR-050-AC-59, FR-077-AC-7 and FR-067-AC-8 own those behaviours; AC-29 and AC-30 own the extraction and graph-record halves, and the statement above now says a build may lack an adapter instead of promising all three unconditionally.

> **CR-204 note (2026-11-08, PLAT-876):** the boundary crate
> `crates/quire-rust-extraction` is renamed `crates/quire-code-extraction`
> (lib `quire_code_extraction`): it has wrapped all three grammars since
> PLAT-851 and is no longer Rust-specific. Every reference in this document,
> ADR-0013 and the boundary gates follows the new name; no behaviour changes.

> **CR-189 note (2026-09-28):** three corrections to AC-27 and AC-28, found
> by the code review of CR-187's implementation. None changes what an AC
> requires of a correct engine; each states a clause precisely enough that
> the review's failing probes become tests.
>
> - **AC-28's diagnostic shape.** It said "`TraceDiagnostic`-shaped", while
>   FR-050-AC-50 routes the same finding to `CoverageReport.diagnostics`.
>   There is one finding with one home, so AC-28 now names FR-050-AC-50's
>   shape and says which line it reports: the line the range text itself
>   sits on, which differs from the marker's first line only when a marker
>   is wrapped.
> - **AC-28 and a list continuing past a range.** `// Trace: A..B, C` binds
>   `C`. The range is the inert token, not the rest of the list. Dropping
>   `C` would read as "range reported" while an id the author named
>   explicitly failed to bind.
> - **AC-27's inheritance.** It is resolved at the declaration site. A
>   title-keyed lookup let a running `describe('s')` and a skipped
>   `describe.skip('s')` in one file swap their members' state, in either
>   order. Inheritance reaches test-kind and suite-kind members only; a
>   helper function inside a skipped suite or class stays `ignored: false`,
>   as the AC's first sentence already scopes the flag.

> **CR-176 note (2026-09-20):** CON-1 now carries the method as well as the
> boundary — the extractor classifies syntax from a syntax tree a
> grammar-driven parser library produces — and keeps every exclusion it
> already held, adding three the parse makes worth stating. It stays one row
> so TC-756 and the `FR-051-CON-1` tags in `src/coverage.rs`,
> `src/symbols/trace.rs` and `tests/it/coverage_rollup.rs` keep binding what they
> already bind; splitting the boundary onto its own id would have renamed a
> constraint four live tags name. AC-25 and AC-26 are new. `PLAT-842`; the
> implementation change is `PLAT-843`.
>
> **The question the extractor answers is a syntax question.** "Is this marker
> attached to a test declaration, and to which one" is decided by the shape of
> the declaration, not by the arrangement of its lines — so the answer must be
> read from a tree, and a parser dependency is admitted to read one. The
> ecosystem already holds the rule for a narrower case:
> `ix://agent-ix/quire-cli/FR-025`'s CON-1 admits no grep, regex-only
> classification or hand-written lexer for the Rust-source gates, and
> `quire-qualify` accordingly classifies that repository's own imports and
> `unsafe` loci from `syn`'s tree. That scope is one language in one
> repository, against this FR's three languages in an arbitrary target tree —
> so it is precedent for the reading, not a rule that already covers this
> extraction. This repairs an under-reading rule, not the authored tests.
>
> **Four measurements, one class.** Each is tracked in Linear and mirrored on
> GitHub, and AC-25 cites the GitHub half. `PLAT-14`
> (`agent-ix/quire-rs#459`): a polyglot repository reported
> **zero** `.py` entries in `criteria[].sources` while 509 Python tests passed,
> so its headline `status_lies` and `unbacked_rows` described two of its three
> languages. `PLAT-163` (`agent-ix/quire-rs#424`): a `{n,m}` quantifier in a
> regex literal moved brace
> depth, the file was abandoned, and **6 of 9** reported status lies in that
> audit were manufactured against correct matrix rows. `PLAT-305`
> (`agent-ix/quire-rs#387`): with
> `#[test]` written above `#[ignore]` — the order rustfmt-formatted code writes
> — a tagged test contributed no candidate at all: backed 83/198 with 4 lies
> against 87/198 with 0, nothing else changed. `PLAT-234`
> (`agent-ix/quire-rs#395`): `black` wrapping a
> `@pytest.mark.trace` at the repo's own 88-column limit moved a criterion to
> unbacked, so the formatter changed the coverage number.
>
> **The properties the old CON-1 protected are unchanged and each is stated.**
> Determinism stays AC-10, byte-identical output over an identical tree: a
> grammar decides the same bytes the same way wherever they sit, and all four
> measurements above are cases where the same bytes read differently depending
> on their neighbours. No build stays AC-26, and is why the parser is a grammar
> library: a grammar takes source text and returns a tree, for every language
> the FR names, with the analysed repository's toolchain absent. No type
> resolution and no execution of extracted code stay in CON-1: what a symbol
> *is* comes from the tree; what it *means* is out of scope. Per-file
> degradation stays CON-2 and AC-9, and the **Language adapters** section now
> states what makes a file unparseable, because a grammar returns a tree for
> every input and the trigger is a property of that tree. The same sentence
> requires the diagnostic to name the file and the line the error sits at, and
> AC-9 says so: a criterion that asked only for "a diagnostic" would be
> discharged by one that names nothing, which is the same hollowness as a
> trigger that can never fire.
>
> **Four criteria named the reader rather than the reading, and three are
> restated here.** AC-12 states what a reader of the file observes — a `//`
> inside a string or template literal is content — so it is unchanged and its
> cases still discriminate. AC-14, AC-15 and AC-20 named the machinery
> instead: "the balance check, brace depth, and block-end spans", "a brace …
> never moves the depth", and an opener that "enters string state" which a
> closing delimiter "closes rather than re-opens". A criterion that names a
> mechanism stops being verifiable when the mechanism changes, and TC-1030
> says as much of AC-20 in its own row — it asserts "on the state machine
> directly". Each is reworded to the property its cases actually pin: AC-14
> that one reading decides what is code and every consumer reads that one
> answer, AC-15 that a brace or quote inside a raw string, lifetime, character
> literal or nested block comment is content and the file is accepted, AC-20
> that a triple-quoted body is content wherever its delimiter sits and a
> declaration after it keeps its true container — the half TC-1031 already
> pins. TC-803, TC-804, TC-1029, TC-1030 and TC-1031 keep their ids and their
> bindings — CR-039 and CR-040 measured
> 78 of 140 reported status lies against a single such reader disagreement, and
> that measurement is what these rows exist to keep from recurring.
>
> **AC-15 also gains the property AC-12 already states for TypeScript.** A
> string opened on one line carries its interior to the lines that follow, so
> a `//`, a brace or a quote on a continuation line is content — asserted for
> Rust by `tc804_string_state_carries_across_lines` and, until now, written
> down nowhere. A multi-line `r#"…"#` is the exact shape of CR-040's incident:
> the 33 files that yielded zero symbols each held a JSON fixture, and a JSON
> fixture spans lines. The carry **is** what broke. It stays on TC-804 rather
> than taking a new id, because a Rust test binds here by its name under
> `rust-test-name-id` — a new id would need a renamed test function, which is
> a `src/` edit this change does not make, and the row would otherwise bind to
> nothing and read as a status lie.
>
> **Two criteria bound a scan to a window, and both stand as written.** AC-18
> stops its title scan at the first non-blank text, so a literal beyond that
> window registers nothing; AC-21 mints a container for a `describe(` whose
> `{` falls on a later line and parents nothing to it. **Both are TypeScript
> criteria** — a `test`/`it` registration and a `describe(...)`/`suite(...)`
> one — so they belong to the TypeScript adapter and sit outside the Rust
> scope `PLAT-843` implements. A tree answers both questions without a window,
> so each carve-out becomes a choice rather than a limit — and widening either
> one **changes what a repository's coverage number is**, for every
> repository, which is a measured decision of the kind CR-119 recorded for the
> suite question rather than a consequence of admitting the parser. They are
> therefore unchanged here and named so the reader porting the TypeScript
> adapter finds them: any widening is its own change with its own **[RAN]**
> numbers.
>
> **Where the old statement still lives.** `src/symbols/mod.rs` opens with
> "They are line-structural rather than full parsers", which is the module
> restating this constraint back when the constraint said that. `PLAT-843`
> carries that line with the code it describes; this CR moves the requirement,
> not the implementation.
>
> The parser `PLAT-843` adopts is `tree-sitter`: one grammar per language,
> source text in, tree out, no toolchain and no dependency resolution for the
> analysed repository.

> **CR-177 note (2026-09-20, `PLAT-843`):** AC-10's identity is amended, not
> widened. Raised in `PLAT-842`'s review and deferred here deliberately — a
> criterion written there would have pinned a dependency the repo did not yet
> have.
>
> AC-10 pins **run-to-run** byte identity over an identical tree, and that is
> unchanged. **Cross-grammar-version identity is deliberately not claimed —
> that scope stays closed.** CR-176 made an external grammar library an input
> to the output; a `tree-sitter-rust` bump that changes a node kind, a field
> name, or error recovery moves every downstream number, with no source
> change and no spec change. AC-10's amendment does not promise output stays
> the same across such a bump — it promises the opposite reads as a *tracked*
> event rather than a silent one, by naming the grammar as part of the
> identity byte-identity is claimed *at*: "over an identical tree, at a
> pinned grammar version."
>
> **The dependency path is single-sourced, not per-consumer.**
> `quire-code-parse` re-exports `tree_sitter` itself rather than wrapping it,
> so every consumer (`quire-code-rs`, `quire-rs`, `filament-ide-rs`, and
> later a daemon) reaches the grammar through the same crate rather than
> vendoring their own. This repository reaches that path through exactly one
> dependency, `crates/quire-code-extraction`'s `quire-code-parse` git-rev
> dependency (`ADR-0013`), and `tests/dependency_boundary.rs`'s first test is
> what keeps a second, independently-versioned grammar from reappearing
> anywhere else in this workspace. One path does not by itself mean one
> *version* across every consumer — each consumer resolves the same caret
> ranges against its own `Cargo.lock`, which is exactly why this repo asserts
> its own resolved version rather than trusting the shared path alone. A
> criterion requiring every consumer to resolve the identical grammar version
> would need a cross-repo mechanism this ticket does not build; none is
> written here.
>
> **The feature set is part of the identity too, not only the grammar.**
> `rust-symbols` (on by default, off for `wasm`) decides whether Rust
> extracts *at all* — off, the build records Rust as unavailable and coverage
> refuses (CR-201), a change of orders of magnitude larger than
> any grammar bump. `AC-10`'s row now names this feature alongside the
> grammar for the same reason: an identity claim silent about a variable that
> moves the output this much is not stating its own scope honestly (spec
> review F4).

> **CR-179 note (2026-09-20, `PLAT-868`):** Phase 2's Python leg of `PLAT-851`
> lands — `src/symbols/python.rs` parses over the same `tree-sitter`/
> `quire-code-parse` boundary CR-176/CR-177 adopted for Rust, through the
> same `crates/quire-code-extraction` crate. `CON-1`'s grammar-driven-parser
> clause now holds for two of the FR's three languages; TypeScript's own
> port (`PLAT-869`) is unaffected by this change and lands separately, under
> serial merge windows to keep coverage movement attributable to one
> language port at a time.
>
> **AC-25's Python shape is now bound.** `TC-1880` — a `black`-wrapped
> multi-line `@pytest.mark.trace` decorator, one separated from its `def` by
> a second wrapped decorator, and a wrapped signature — moves from `🚧` to
> `✅`. `decorated_definition`'s own tree-sitter span already includes every
> decorator regardless of how any one of them wraps, closing `PLAT-234`
> (a `black`-wrapped decorator silently moved a criterion to unbacked) the
> same way `PLAT-69`/`PLAT-846` closed the equivalent Rust span defects.
>
> **TC-1029, TC-1030 and TC-1031 keep their ids and bindings, as CR-176
> already said they would** — AC-20's text was reworded there to the
> property these rows pin ("a triple-quoted body is content wherever its
> delimiter sits and a declaration after it keeps its true container"),
> not to the hand-rolled `Quoting`/`scan_line` state machine that property
> used to run through. That machine — the source of the three real defects
> these rows were written against (#274) — is deleted outright by this port,
> as a consequence of parsing correctly rather than a deliberate
> optimisation; each row's own test body is rewritten to prove the same
> property against the tree instead, confirmed red first against a naive
> stand-in that does read a string's embedded content as a declaration (see
> `src/symbols/python.rs`'s own test module).
>
> **Qualified-name construction is unchanged, byte-for-byte** — a
> compatibility surface (`Symbol::compute_id` composes `(language, path,
> qualified_name, kind)`), not a style choice this port is free to clean up.
> In particular: only a `class` is a scope (a `def` never becomes a
> container for its own nested `def`s, unlike Rust's own `PLAT-845`); a
> qualified name never carries the module prefix, only class nesting joins
> with `.`; and the `#407` unittest-`TestCase`-base detection stays bounded
> to a single-line, top-level class header with module-top-level-only import
> tracking — widening either is an identity change reserved for its own
> ticket, not a free byproduct of the tree-sitter port.
>
> `python-symbols` is promoted to the default Cargo feature set by this
> change (it was wiring-only and off by default under `PLAT-851`, since no
> adapter used it yet) — `cargo build`/`cargo test` with no flags keeps
> extracting Python symbols exactly as the pre-port scanner always did,
> mirroring `rust-symbols`. It stays gated (rather than unconditional) so the
> `wasm` build can still opt out: `tree-sitter-python` carries the identical
> C-toolchain cross-compilation problem `rust-symbols`'s own `ADR-0013`
> documents.

> **CR-119 note (2026-08-24):** AC-21 is new. `agent-ix/quire-rs#273`, epic
> `agent-ix/quire-rs#264`.
>
> `describe(…)` blocks group tests and registered **no symbol themselves**, so a
> trace tag on a suite header had nothing to attach to. The nearest enclosing
> symbol was the file's own module container — a container spanning line 1 to
> EOF, which localises nothing. The header now mints a `Container` named by its
> registered title, and the registrations inside it are parented to it.
>
> **A suite parents without naming.** A registration's qualified name is its
> registered title — stated in this FR's body, not in AC-3, which is about
> classification — and qualifying it through the enclosing suite would
> change the identity of every test in the ecosystem that sits inside a
> `describe(`. The scope stack therefore carries two axes — what a scope is
> *called*, and whether members are named *through* it — and a class still
> qualifies its own members inside a suite.
>
> **`context` is refused on measurement.** Across `~/dev`: 1,699 `describe(`
> registrations with a quoted title in 103 repository roots, **zero** `suite(`,
> **zero** `context(`, and five lines beginning `context.` that are method calls
> on an object of that name. Mocha's alias would put the `.modifier` chain in
> front of the one shape it cannot distinguish from a suite, for no occurrence
> it would recover. `suite` is admitted at zero occurrences because it collides
> with nothing.
>
> **The bind semantics are declared as UNCHANGED, and that is the load-bearing
> half.** A suite is a grouping, not evidence, so a tag on a suite header mints
> no `verifies` relation and the suite is not a `binding_census` candidate.
> #273 recommended the opposite — that a suite-level tag bind on behalf of its
> contained tests — so the recommendation was **run**. **[RAN]** over the 241
> repositories `scripts/corpus.py` enumerates, with `Container` moved into
> `binds_trace_ids()`: rows backed 5,039 → 5,526 (+487), `untracked_symbols`
> 1,085 → **3,961** (+2,876), census candidates 25,358 → 43,260, the rust
> binding ratio 44.41% → **28.05%**, and `low-symbol-binding` firing on 13 more
> repositories that did nothing wrong. Six ids bound to nothing per row
> recovered. The widening cannot reach a suite without reaching **every**
> container, the file's module included — CR-061's "manufacture backing out of
> prose", measured. A suite-level tag that binds needs a kind that is neither
> `Container` nor `TestFunction`, which is a change to the
> `binds_trace_ids`/`carries_implements` complement, and that belongs to
> `agent-ix/quire-rs#312` along with its census question.
>
> **What this change moves, ecosystem-wide, is one ratio and it moves the wrong
> way.** **[RAN]** same 241 repositories, same binary but for this change: rows
> minted 20,023 → 20,023, rows backed 5,039 → 5,039, every diagnostic count and
> every `binding_census` identical — and `coverage.implements examined`
> 49,855 → **51,340** (+1,485 suite containers across 93 repositories), a
> denominator growing while its value does not.
>
> **#273's two worked examples are not both this defect, and one is not this
> defect at all.** `agent-ix/agent-cli-daemon` reads **0 backed of 119** here,
> under this change, and under the `Container`-binds experiment as well: its
> census is 127 candidates / 0 bound and it fires `no-symbol-bound` plus six
> `section-matches-nothing`, so its matrix does not mint the ids its tags name
> and no attachment fix can reach it. Under the experiment it gains 24 bound
> symbols and **29 `untracked_symbols`** — every recovered tag names a row the
> declaration never minted. `agent-ix/ix-ui` does move under the experiment
> (0 → 57 of 181, with 20 untracked) and not under this change. Recorded because
> the ticket cites both as the same defect.

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
> every `.py` under the ecosystem corpus (excluding `worktrees/`, `.venv*`,
> `site-packages/`, `node_modules/`; files `ast` cannot read are skipped).
> Comparison is by multiset, and both
> namings are reported because the engine mints the qualified one: `bare` is the
> declaration's own name, `qualified` its dotted path through enclosing classes.
>
> The gap between the two axes is the misattribution mode: declarations present
> under the wrong container, which a bare-name comparison cannot see.
> Named example from the ticket, verified: `test_update_simple_version`
> (`py-project/tests/test_deps.py:464`) really lives in `TestTomlModification`
> and was reported under `TestTomlParsing`.
>
> **Measured against the engine, not a reimplementation.** #274's own figures
> moved three times because each was produced by a port of the state machine,
> and the ports disagree exactly where the original is wrong (#309). The
> comparison uses `quire_rs::symbols::extract_file` itself, run twice over one
> file list — once before the change, once after.
>
> **What did not change.** Scope is popped by indentation alone, so a
> declaration nested inside a block that is itself nested keeps the wrong
> qualifier. The one remaining instance is:
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
> the Criterion perf gate does. It does not: that gate is not in
> `make ci`, and the CI `perf` job runs `--bench parse --bench load`, neither of
> which touches `src/symbols/`. No bench exercises the adapter at all.

> **STD-228 implementation note (2026-10-21):** The Criterion regression band
> is checked by the Rust `quire-perf-gate` tool. Its bench scope is unchanged.
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
> **What the number counts:** evidence symbols in `agent-ix/filament-ide-rs` at
> the time of the measurement — **1,292 Rust candidates, 0 bound**,
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
> (Allocation note: TC-959 is skipped.)

> **CR-180 note (2026-09-20, `PLAT-882`):** CR-176's own note named two
> window carve-outs — AC-18's bounded title scan and AC-21's same-line `{`
> requirement — as decisions belonging to the TypeScript adapter's own port,
> to be widened only as "its own change with its own **[RAN]** numbers," not
> as an incidental consequence of adopting a parser. This closes both,
> measured, not assumed:
>
> - **AC-18's window is gone**, because there is no scan left to bound: a
>   tree-sitter `call_expression`'s `arguments` field names the true first
>   argument directly, regardless of how many lines the call spans. What
>   replaces it is a different, narrower requirement — a registration must
>   carry a second (callback) argument, or it registers nothing — which is
>   what every real registration in this adapter's own fixtures already has,
>   and is what the old window accidentally also enforced for a title with no
>   callback written far enough down the file to exceed it.
> - **AC-21's same-line requirement is gone**, for the identical reason: the
>   tree, not a same-physical-line text check, decides whether a
>   `describe(...)`'s callback is a suite's own body.
> - **A third edge widens as a direct structural consequence of the same
>   change**: whitespace before a modifier chain's `.` (`it .skip(...)`) is
>   now admitted, because tree-sitter reads it identically to `it.skip(...)`
>   — real TypeScript does too — where the old regex's own boundary
>   coincidentally rejected it (see `src/symbols/typescript.rs`'s own docs for
>   the mechanism).
>
> **[RAN]**, over the two corpora carrying this ecosystem's TypeScript
> (`quire-rs`, `filament-ide-rs`, per the PLAT-882 differential report): all
> three widenings recover **zero** real registrations or re-parented
> containers — no title beyond the old window, no wrapped `describe(` call,
> and no whitespace-before-`.` chain exists anywhere in the measured corpus.
> The decision is made and stated, not deferred, because tree-sitter cannot
> selectively reintroduce these three specific limitations without hand-rolled
> checks that would themselves be exactly the kind of textual heuristic this
> port exists to retire — but the measurement is recorded here precisely so a
> future widening elsewhere is not mistaken for this one having been assumed
> safe rather than checked.
>
> Two **narrowings** ride along with the same change (PLAT-882 PR #481 review
> finding F8), stated here for the same reason rather than left implicit in
> code:
>
> - The callback-argument requirement above is stricter than CR-084's own
>   window in one direction: `it.todo('x')` and `test.skip('name')` — a
>   registration call with only a title and no callback — registered under
>   the old regex (whose own scan had no notion of "how many arguments",
>   only "is there a quote within N lines") and register nothing here.
> - `mint_arrow_const_declarators` mints every arrow-valued declarator in a
>   comma-separated `const`/`let`/`var` statement (`const a = () => {}, b =
>   () => {}`), where the old regex's single capture group matched only the
>   first.
>
> **[RAN]**, same two corpora, by a targeted pattern search rather than the
> full symbol-level reconciliation the three widenings above got (that
> reconciliation already accounts for every symbol either way, so a
> non-zero count here would already have surfaced as an unexplained delta
> in the differential and did not): the no-callback shape matched **zero**
> times across both repositories including `quire-rs`'s own fixtures; the
> comma-separated multi-declarator-arrow shape matched **zero** times
> single-line, which is the only form the pattern search covers — a
> multi-line instance of that specific shape (a genuinely uncommon way to
> write it) cannot be ruled out by this search the same way the callback
> check can, so that half is reported at lower confidence than the rest of
> this note.

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
> **[RAN]** Across the ecosystem corpus, legacy comment lines carrying a list
> bound only their first id, spanning every declared legacy shape and all three
> languages; `quoin`'s status lies had this one cause (agent-ix/quoin#65).
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

> **CR-213 note (2026-11-08, PLAT-362):** `language_of` read three
> extensions, so `scripts/audits/check_no_shared_mutable.sh` (TC-502), a shell
> audit that enforces a `Static` row, was never opened and could mint no symbol.
> The tree holds four `.sh` files (three in `scripts/audits/`, plus
> `scripts/check_property_purity.sh`). CR-061 widened what binds but could not
> reach a language the extractor does not read. Ruled (PLAT-362 option 1;
> research on PLAT-390): add the language rather than declare `Static` in
> `no_source_symbol`, which would change verdicts for every repository on the
> module and withdraw an accusation the code does not earn.
>
> **Parser.** The shell adapter parses with `tree-sitter-bash` (MIT, within
> `deny.toml`'s allowed licences), not a hand-written line lexer: shell's
> quoting, heredocs, `$( )` and `case` patterns are exactly the lexer class
> CR-039 and CR-040 paid for and PLAT-843/868/882 removed. The grammar enters
> through the PLAT-911 boundary: a `bash` feature of `quire-code-parse`
> (`quire-code-rs`), forwarded by `quire-code-extraction` under
> `shell-symbols`, which `check_tree_sitter_boundary.sh` already polices. No
> CON-1 exception is needed.
>
> **Entry point.** The file is the entry point (AC-37) because a shell audit
> declares no function that *is* the test. It is minted for every `.sh` file so
> that extraction depends on no module data (AC-19, AC-23); the module's
> `entry_paths` glob decides only whether it may bind (AC-38), so a production
> script never becomes evidence, for the reason CR-061 kept plain functions out.
> A marker that does not bind is reported (AC-44) because a misplaced audit or a
> glob typo would otherwise leave its `Static` row unbacked in silence. The tag
> is a comment, so it is declared as a `markers` entry: FR-051-CON-3 admits
> marker forms only, and a shell marker is one.
>
> **Cross-repo.** The implementation also needs a change in `quire-code-rs`:
> `quire-code-parse` must add the `bash` feature (tree-sitter-bash) before
> `quire-code-extraction` can forward it under `shell-symbols`.
>
> **Ordering.** `SourceLanguage` is closed and the grammar is
> `deny_unknown_fields`, so an engine that predates this change rejects a module
> declaring a shell form at load; the engine release must precede the module
> release. The qualified-name spelling here (function name; `entry_point`) is the
> shell row for quoin FR-120's per-language table.
>
> **TC-502 flips** when the engine and module ship **and** this repo's
> `check_no_shared_mutable.sh` header carries the declared marker (it carries
> prose today) under a declared `entry_paths` glob covering `scripts/audits/`.
> TC-050 and TC-203 were removed in the same change: the scripts they name
> (`check_unsafe_comments.sh`, `verify_cookiecutter_inheritance.sh`) no longer
> exist, and TC-203 was a byte-equality check against an external baseline.

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the declared trace-tag grammar), [FR-045](./FR-045-filament-core-extraction-engine.md) (record shapes, id and dedup conventions), [NFR-006](../non-functional/NFR-006-determinism.md) (determinism discipline)
- **Downstream**: the coverage rollup ([FR-050](./FR-050-declarative-coverage-computation.md)), the `gap-analysis` semantic review, and filament-core knowledge-graph ingestion consume the symbol graph; [FR-065](./FR-065-controlled-corpus-contract.md) (the corpus's L2 localisation level reads `binding_census.unbound_example` from this extraction)
- **Companion deliverables (outside quire-rs core)**: the canonical markers imply per-language support packages — a pytest plugin registering the `trace` marker, a lightweight Rust no-op proc-macro crate, and an npm helper for vitest/jest — separate deliverables following the separate-workspace-crate pattern (ADR 0010 placement precedent); this FR specifies only the static parsing contract the extractor holds them to
- **Emitter owners**: `FR-001` in `agent-ix/ix-trace-py` (pytest plugin) and in `agent-ix/ix-trace-ts` (vitest reporter) now own the emitter requirements, as `ix-trace-rs` does for Rust; FR-051-AC-47 and FR-051-AC-48 are this repository's extractor-side companions to them (`ix://agent-ix/quoin/FR-120`).
