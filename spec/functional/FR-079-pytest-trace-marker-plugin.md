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

The pytest plugin SHALL register the `trace` marker and, while emission is enabled,
write the ids of each `@pytest.mark.trace(...)` marker into the JUnit XML
that pytest itself writes, as the mark of
[Quoin FR-120](ix://agent-ix/quoin/FR-120) and under its Emitter contract
(E-1..E-10). This requirement is the pytest binding of that contract; the contract
text is not restated beyond what a criterion below needs.

The static side is unchanged: [FR-051](./FR-051-source-symbol-extraction.md) binds the
marker from the syntax tree with no code executed. This requirement adds the
execution side, so that `quoin evidence record` can bind the same ids to a result.

### Registration

The plugin registers `trace` as a marker with a description, so
`@pytest.mark.trace("FR-001-AC-1")` raises no `PytestUnknownMarkWarning` and passes
under `--strict-markers`. Registration is independent of emission: it changes no
result file.

### Emission

Emission is enabled by the environment variable `QUIRE_TRACE_EMIT=1`, the one enable name shared by every trace emitter (the Rust emitter uses it too), and by
nothing else; installing the plugin or writing the marker never enables it.
While enabled, each collected test whose item carries `trace` markers (on the
function, its class, or its module) gets one JUnit `<property name="trace"
value="…"/>` child on its `<testcase>`, holding every id from every marker argument
in declaration order. Ids are carried to the result through pytest's own
per-item user properties, so pytest's `--junit-xml` writer emits them.

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

An id argument that is not a string literal, or a case for which the active
`junit_family` cannot carry the `trace` property, `file`, and the `name`/`classname`
pair above, makes an enabled run fail with an error that names the test or the
family. It never writes a result file that silently lacks a mark.

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
| FR-079-AC-3 | While enabled, a case with markers on its function, class and module carries one `trace` property holding every id of every marker, in order, split exactly as written; a case with no marker carries none. | Test |
| FR-079-AC-4 | While enabled, every id written is an argument of a marker on that test; an id is never inferred from a test or file name, and a marker argument that is not a string literal fails the run naming the test. | Test |
| FR-079-AC-5 | While enabled, a module-level test reports its bare name, a method reports `Class.method`, and a test parametrized with two cases yields two `<testcase>` elements both named for the source function with no `[case]` suffix; for each, `name` equals `classname`, `file` is the repo-relative path, and all three equal what `quire coverage --json` reports for the same fixture. | Test |
| FR-079-AC-6 | A marked test that fails, raises in setup or teardown, is skipped, or is marked `xfail` is recorded with the outcome pytest's own `<failure>`, `<error>` or `<skipped>` child reports, and with its ids; no outcome is written that pytest did not report. | Test |
| FR-079-AC-7 | Under `pytest-xdist` with two or more workers, each marked test appears once with exactly its own ids, none mixed with another test's. | Test |
| FR-079-AC-8 | The result file is written by pytest's own `--junit-xml` writer; the plugin writes no sidecar file and runs no step after pytest exits. A `junit_family` that cannot carry the mark fails the enabled run naming the family. | Test |
| FR-079-AC-9 | While enabled, a 500-test marked suite adds under 5% to the median wall-clock time of the same suite with emission disabled, over at least five runs each. | Test |
| FR-079-AC-10 | A real pytest suite run with emission enabled produces a result file that `quoin evidence record` accepts and whose obligation is bound; the same suite with its tagged test made to fail yields a failing entry and binds nothing; editing the requirement text marks the binding stale. | Test |

## Constraints

| ID | Constraint | Type | Validation |
| --- | --- | --- | --- |
| FR-079-CON-1 | The plugin SHALL NOT merge, patch or rewrite a result file after pytest has written it (Quoin FR-120 E-8). | Design | Inspection |

## Dependencies

- **Upstream**: [Quoin FR-120](ix://agent-ix/quoin/FR-120) (the mark and the Emitter contract), [FR-051](./FR-051-source-symbol-extraction.md) (the marker form and the `qualified_name` the plugin must equal).
- **Package home**: the repository and package name of the plugin are open (`plan/Plan-001-ac-grammar-coverage/plan.md`, EXT-4a, owner: user decision). The criteria bind the package wherever it lives; they are tested against this repository's extractor.
- **Sibling**: [FR-080](./FR-080-vitest-jest-trace-helper-and-reporter.md), the vitest/jest binding of the same contract.
