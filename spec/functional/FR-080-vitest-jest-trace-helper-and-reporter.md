---
id: FR-080
title: "vitest/jest trace() helper and reporter emit the trace mark into JUnit output"
type: FR
relationships:
  - target: "ix://agent-ix/quoin/FR-120"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-007"
    type: "traces_to"
---

# FR-080: vitest/jest trace() helper and reporter emit the trace mark into JUnit output

## Description

A TypeScript package SHALL provide a typed `trace(...ids)` helper, and a reporter
for vitest and for jest that, while emission is enabled, writes the ids a test
declared into the JUnit XML as the mark of
[Quoin FR-120](ix://agent-ix/quoin/FR-120), under its Emitter contract
(E-1..E-10). This requirement is the vitest/jest binding of that contract.

The static side is unchanged: [FR-051](./FR-051-source-symbol-extraction.md) binds
`trace("FR-001-AC-1")` from the syntax tree with no code executed.

### The helper

`trace(...ids: string[]): void` is a real exported, type-checked function, so the
canonical form is not an undefined symbol. Called inside a test body, it declares
the ids for the running test; it does nothing else. While emission is not enabled
it returns immediately, records nothing and touches no runner state. The ids reach
the reporter through the runner's own per-test result channel (for vitest the
task's `meta`), not through a file.

### The reporter

The reporter is a custom reporter for each runner. While emission is enabled it
writes the JUnit file itself, one `<testcase>` per test, with the runner's own
outcome, and adds `<property name="trace" value="…"/>` to each test that declared
ids.

Emission is enabled by the environment variable `QUIRE_TRACE_EMIT=1` and by nothing
else.

### The symbol

For each case the reporter writes as `name` and again as `classname` the
`qualified_name` that `quire coverage --json` reports for the test, and as `file`
the repo-relative `/`-separated path:

| Registration shape | `name` and `classname` |
| --- | --- |
| `it`/`test` | The registered title exactly as written in source. An enclosing `describe` does not qualify it. |
| Class method | `Class.method`. |
| `it.each`/`test.each` | The template title as written in source (for example `adds %s`), never the interpolated title; each case is its own `<testcase>`. |
| Modifier shapes (`it.skipIf(c)(…)`, `it.only`, `it.concurrent`, `it.skip`) | The registered title, once, whatever the modifier. |

The reporter sees every registered test, so a registration the static extractor
mis-shapes (`agent-ix/quire-rs#189`) still reports its title at run time.

### Limit

A test that never reaches its `trace(...)` call (skipped by a modifier, or throwing
before the call) has no declared ids at run time. It is reported with the runner's
outcome and no `trace` property, and so discharges nothing. A test that reaches
the call and then throws is recorded `fail` with its ids. Declaring ids at
registration, so that a skipped test carries them, is outside this requirement.

## Rationale

The JUnit reader exists and nothing writes the mark. A runtime reporter is the one
place that sees a test however it was declared. The runner's default JUnit naming
joins `describe` titles into a name the extractor does not use, so the reporter
renames every case.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-080-AC-1 | `trace("FR-001-AC-1", "TC-041")` type-checks as an exported function accepting string ids, and a call with a non-string argument is a compile-time error. | Test |
| FR-080-AC-2 | While `QUIRE_TRACE_EMIT` is unset or not `1`, the helper records nothing, the reporter writes no `trace` property, the suite's runner results equal those of a run without the package, and the package has no runtime dependency beyond the runner. | Test |
| FR-080-AC-3 | While enabled, a test that calls `trace` with several ids, or calls it more than once, carries the ids in call order in `trace` properties on its case; a test that did not call it carries none. | Test |
| FR-080-AC-4 | While enabled, every id written was an argument of a `trace` call in that test; an id is never inferred from a title or file name. | Test |
| FR-080-AC-5 | While enabled, for vitest and for jest, an `it` inside two nested `describe` blocks reports its bare title, a class method reports `Class.method`, and an `it.each` with two cases yields two `<testcase>` elements both named by the template title; for each, `name` equals `classname`, `file` is the repo-relative path, and all three equal what `quire coverage --json` reports for the same fixture. | Test |
| FR-080-AC-6 | While enabled, `it.skipIf(true)(…)`, `it.only`, `it.concurrent` and `it.skip` registrations each report their registered title once with the runner's outcome (`skip` for a skipped one), and a skipped test carries no ids and discharges nothing. | Test |
| FR-080-AC-7 | A test that calls `trace` and then throws or fails an assertion is recorded `fail` with its ids; the outcome written is always the runner's. | Test |
| FR-080-AC-8 | Under the runner's default parallel execution, each tagged test appears once with exactly its own ids, none mixed with another test's. | Test |
| FR-080-AC-9 | The result file is written by the reporter during the run; no sidecar file is merged and no step after the runner exits rewrites names or ids. | Test |
| FR-080-AC-10 | While enabled, a 500-test tagged suite adds under 5% to the median wall-clock time of the same suite with emission disabled, over at least five runs each, for each runner. | Test |
| FR-080-AC-11 | A real vitest suite and a real jest suite, run with emission enabled, each produce a result file that `quoin evidence record` accepts and whose obligation is bound; the same suite with its tagged test made to fail yields a failing entry and binds nothing; editing the requirement text marks the binding stale. | Test |

## Constraints

| ID | Constraint | Type | Validation |
| --- | --- | --- | --- |
| FR-080-CON-1 | The package SHALL NOT merge, patch or rewrite a result file after the runner has written it (Quoin FR-120 E-8). | Design | Inspection |

## Dependencies

- **Upstream**: [Quoin FR-120](ix://agent-ix/quoin/FR-120) (the mark and the Emitter contract), [FR-051](./FR-051-source-symbol-extraction.md) (the marker form and the `qualified_name` the reporter must equal).
- **Package home**: the repository and package name are open (`plan/Plan-001-ac-grammar-coverage/plan.md`, EXT-4c, owner: user decision). The criteria bind the package wherever it lives; they are tested against this repository's extractor.
- **Sibling**: [FR-079](./FR-079-pytest-trace-marker-plugin.md), the pytest binding of the same contract.
