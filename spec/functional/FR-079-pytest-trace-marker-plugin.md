---
id: FR-079
title: "pytest plugin registers the trace marker and emits it into JUnit output"
type: FR
relationships:
  - target: "ix://agent-ix/quoin/FR-120"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-007"
    type: "traces_to"
---

# FR-079: pytest plugin registers the trace marker and emits it into JUnit output

## Description

The pytest plugin SHALL register the `trace` marker.

While emission is enabled, the pytest plugin SHALL write the ids of each
`@pytest.mark.trace(...)` marker on a test function into the JUnit XML that pytest
itself writes, as the mark of [Quoin FR-120](ix://agent-ix/quoin/FR-120) and under
its Emitter contract (E-1..E-10). This requirement is the pytest binding of that
contract; the contract text is not restated beyond what a criterion below needs.

The static side is unchanged: [FR-051](./FR-051-source-symbol-extraction.md) binds the
marker from the syntax tree with no code executed. This requirement adds the
execution side, so that `quoin evidence record` can bind the same ids to a result.

### Registration

The plugin registers `trace` as a marker with a description, so
`@pytest.mark.trace("FR-001-AC-1")` raises no `PytestUnknownMarkWarning` and passes
under `--strict-markers`. Registration is independent of emission: it changes no
result file.

### Emission

Emission is enabled by the environment variable `QUIRE_TRACE_EMIT=1` and by
nothing else; installing the plugin or writing the marker never enables it.
`QUIRE_TRACE_EMIT` is the enable name for the pytest and TypeScript emitters only.
The Rust emitter is enabled by a cargo feature, not by this variable.

While enabled, each collected test whose function carries `trace` markers gets one
JUnit `<property name="trace" value="…"/>` child on its `<testcase>`, holding every
id from every marker argument on that function, each once. The order of ids in the
value carries no meaning. Ids are carried to the result through pytest's own
per-item user properties, so pytest's `--junit-xml` writer emits them.

Only markers on the test function itself are emitted. A marker on a class or a
module-level `pytestmark` is not emitted, because the static side
([FR-051](./FR-051-source-symbol-extraction.md)) binds a decorator only to the
`def` it decorates and a class binds no trace ids, so the two views would disagree.

### The symbol

The plugin reports for each case, as `name` and again as `classname`, the
`qualified_name` that `quire coverage --json` reports for that test, and as `file`
the repo-relative `/`-separated path:

| Test shape | `name` and `classname` |
| --- | --- |
| Module-level function | The bare function name. The module is never a prefix. |
| Method of a class | `Class.method` (nested classes dot-joined). |
| Parametrized test | The source function name, with the `[case]` suffix removed; each case is its own `<testcase>` carrying that symbol. |

pytest's default `classname` (the dotted module path) and `name` (with the
parametrize suffix) are replaced for every tagged case.

### Failure is loud

A marker argument that is not a `str` makes an enabled run fail with an error that
names the test. A string held in a variable is a `str` at run time and is emitted;
the static side does not bind it.

`file` is required on every tagged case, and pytest's default `junit_family`
(`xunit2`) drops it, where `xunit1` keeps it (measured on pytest 9.0.2). An enabled
run under any family other than `xunit1` therefore fails with an error that names
the family and the remedy `-o junit_family=xunit1`. The plugin never writes a
result file that silently lacks a mark or a `file`.

## Rationale

The reader exists (the JUnit adapter reads a `trace` property) and the writer does
not. pytest already exposes the hook and the property mechanism, so the writer is a
plugin rather than a harness. A pytest case name is not the extractor's symbol, so
without the rename the static and execution views never join.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-079-AC-1 | With the plugin installed, a test marked `@pytest.mark.trace("FR-001-AC-1")` collects with no `PytestUnknownMarkWarning`, and the suite passes under `--strict-markers`, whether or not emission is enabled. | Test |
| FR-079-AC-2 | While `QUIRE_TRACE_EMIT` is unset or not `1`, the JUnit file of a marked suite is identical, apart from timing attributes, to the file of the same run without the plugin installed, contains no `trace` property, and the plugin imports nothing beyond pytest and the standard library. | Test |
| FR-079-AC-3 | While enabled, a test function carrying several markers, or one marker with several ids, has one `trace` property holding each id once; a function with no marker carries none, and a marker on its class or module adds nothing. | Test |
| FR-079-AC-4 | While enabled, every id written is an argument of a marker on that test function; an id is never inferred from a test or file name, and a marker argument that is not a `str` fails the run naming the test. | Test |
| FR-079-AC-5 | While enabled, a module-level test reports its bare name, a method reports `Class.method`, and a test parametrized with two cases yields two `<testcase>` elements both named for the source function with no `[case]` suffix; for each, `name` equals `classname`, `file` is the repo-relative path, and all three equal what `quire coverage --json` reports for the same fixture. | Test |
| FR-079-AC-6 | A marked test that fails, raises in setup or teardown, is skipped, or is marked `xfail` is recorded with the outcome pytest's own `<failure>`, `<error>` or `<skipped>` child reports, and with its ids; no outcome is written that pytest did not report. | Test |
| FR-079-AC-7 | Under `pytest-xdist` with two or more workers, each marked test appears once with exactly its own ids, none mixed with another test's. | Test |
| FR-079-AC-8 | The result file is written by pytest's own `--junit-xml` writer; the plugin writes no sidecar file and runs no step after pytest exits. | Test |
| FR-079-AC-9 | An enabled run under `junit_family=xunit2` fails with an error naming the family and the remedy `-o junit_family=xunit1`, and writes no result file lacking `file`; under `xunit1` every tagged case carries `trace`, `name`, `classname` and `file`. | Test |
| FR-079-AC-10 | While enabled, a 500-test marked suite adds under 5% to the median wall-clock time of the same suite with emission disabled, over at least five runs each. | Test |
| FR-079-AC-11 | A real pytest suite run with emission enabled produces a result file that `quoin evidence record` accepts and whose obligation is bound; the same suite with its tagged test made to fail yields a failing entry and binds nothing. | Test |

## Constraints

| ID | Constraint | Type | Validation |
| --- | --- | --- | --- |
| FR-079-CON-1 | The plugin SHALL NOT merge, patch or rewrite a result file after pytest has written it (Quoin FR-120 E-8). | Design | Inspection |

## Dependencies

- **Upstream**: [Quoin FR-120](ix://agent-ix/quoin/FR-120) (the mark and the Emitter contract), [FR-051](./FR-051-source-symbol-extraction.md) (the marker form and the `qualified_name` the plugin must equal).
- **Package home**: the `agent-ix/ix-trace-py` repository, a sibling of `ix-trace-rs`. This requirement is held in quire-rs only until that repository exists and carries its spec, and is then to relocate to it (as the Rust emitter's spec does in `ix-trace-rs`); quire-rs keeps only the symbol-equality criterion (FR-079-AC-5) and the end-to-end demonstration (FR-079-AC-11).
- **Sibling**: [FR-080](./FR-080-vitest-jest-trace-helper-and-reporter.md), the vitest binding of the same contract.
