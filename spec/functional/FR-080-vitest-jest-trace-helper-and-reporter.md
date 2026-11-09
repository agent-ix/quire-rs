---
id: FR-080
title: "vitest trace() helper and reporter emit the trace mark into JUnit output"
type: FR
relationships:
  - target: "ix://agent-ix/quoin/FR-120"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-007"
    type: "traces_to"
---

# FR-080: vitest trace() helper and reporter emit the trace mark into JUnit output

## Description

The TypeScript package SHALL provide a typed `trace(...ids)` helper.

While emission is enabled, the package's vitest reporter SHALL write the ids a test
declared into the JUnit XML as the mark of [Quoin FR-120](ix://agent-ix/quoin/FR-120),
under its Emitter contract (E-1..E-10). This requirement is the vitest binding of
that contract. jest is recorded as unsupported (see Limits).

The static side is unchanged: [FR-051](./FR-051-source-symbol-extraction.md) binds
`trace("FR-001-AC-1")` from the syntax tree with no code executed.

### The helper

`trace(...ids: string[]): void` is a real exported, type-checked function, so the
canonical form is not an undefined symbol. Called inside a test body, it declares
the ids for the running test by writing them to that test's `meta`, which vitest
carries to the reporter; it does nothing else. While emission is not enabled it
returns immediately, records nothing and touches no runner state. No file is
involved.

### The reporter

While emission is enabled the reporter writes the JUnit file itself, one
`<testcase>` per test, with the runner's own outcome, and adds
`<property name="trace" value="…"/>` to each test that declared ids, holding each
distinct id once. The order of ids carries no meaning.

Emission is enabled by the environment variable `QUIRE_TRACE_EMIT=1` and by nothing
else. `QUIRE_TRACE_EMIT` is the enable name for the pytest and TypeScript emitters
only. The Rust emitter is enabled by a cargo feature, not by this variable.

### The symbol

For each case the reporter writes as `name` and again as `classname` the
`qualified_name` that `quire coverage --json` reports for the test, and as `file`
the repo-relative `/`-separated path. vitest and jest register tests only through
`it`/`test`, so there is no class-qualified form: the symbol is the title.

| Registration shape | `name` and `classname` |
| --- | --- |
| `it`/`test` | The registered title exactly as written in source. An enclosing `describe` does not qualify it. |
| `it.each`/`test.each` | The template title as written in source (for example `adds %s`), never the interpolated title; each case is its own `<testcase>`. |
| Modifier shapes (`it.skipIf(c)(…)`, `it.only`, `it.skip`) | The registered title, once, whatever the modifier. |

vitest registers each `each` case under its interpolated title and keeps no template.
The reporter therefore enables vitest's task location, reads the registration's
literal template from the source text at that location, and uses it as the symbol.
If the template cannot be recovered, the enabled run fails with an error naming the
test.

### Limits

These are known shortfalls, stated so that no conformance is claimed that the
package cannot meet.

- **Deviation from E-4 and FR-120-AC-5 (FR-120 amendment pending, PLAT-334).** A
  test that never reaches its `trace(...)` call (skipped by a modifier, or
  throwing before the call) has no declared ids at run time. It is reported with
  the runner's outcome and no `trace` property. E-4 requires a throwing test to be
  recorded `fail` with its ids, and FR-120-AC-5 requires a skipped tagged test to
  carry its ids; this package meets both only for a test that calls `trace` before
  it throws or skips. Such an entry has no ids known at record time and discharges
  nothing. Declaring ids at registration is outside this requirement.
- **jest is unsupported.** jest runs test bodies in worker processes and hands its
  reporter an `AssertionResult` that has no user metadata field, so no channel
  carries a `trace()` call to the reporter. The requirement is revisited by
  amendment when a channel is identified.
- **`it.concurrent`.** `trace()` finds the running test through a module-level
  current-test slot, so a call after the first `await` in a concurrent test can
  attach to another test. A `trace()` call is attributed correctly only when made
  before the first `await`; other calls in a concurrent test are unsupported.

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
| FR-080-AC-3 | While enabled, a test that calls `trace` with several ids, or calls it more than once, carries each distinct id once in its `trace` property; a test that did not call it carries none. | Test |
| FR-080-AC-4 | While enabled, every id written was an argument of a `trace` call in that test; an id is never inferred from a title or file name. | Test |
| FR-080-AC-5 | While enabled, an `it` inside two nested `describe` blocks reports its bare title, and an `it.each` with two cases yields two `<testcase>` elements both named by the template title; for each, `name` equals `classname`, `file` is the repo-relative path, and all three equal what `quire coverage --json` reports for the same fixture. | Test |
| FR-080-AC-6 | While enabled, `it.skipIf(true)(…)`, `it.only` and `it.skip` registrations each report their registered title once with the runner's outcome (`skip` for a skipped one), and a skipped test carries no ids and discharges nothing. | Test |
| FR-080-AC-7 | A test that calls `trace` and then throws or fails an assertion is recorded `fail` with its ids; the outcome written is always the runner's. | Test |
| FR-080-AC-8 | Under vitest's default parallel execution across files, each tagged test appears once with exactly its own ids, none mixed with another test's. | Test |
| FR-080-AC-9 | The result file is written by the reporter during the run; no sidecar file is merged and no step after vitest exits rewrites names or ids. | Test |
| FR-080-AC-10 | While enabled, a 500-test tagged suite adds under 5% to the median wall-clock time of the same suite with emission disabled, over at least five runs. | Test |
| FR-080-AC-11 | A real vitest suite run with emission enabled produces a result file that `quoin evidence record` accepts and whose obligation is bound; the same suite with its tagged test made to fail yields a failing entry and binds nothing. | Test |

## Constraints

| ID | Constraint | Type | Validation |
| --- | --- | --- | --- |
| FR-080-CON-1 | The package SHALL NOT merge, patch or rewrite a result file after vitest has written it (Quoin FR-120 E-8). | Design | Inspection |

## Dependencies

- **Upstream**: [Quoin FR-120](ix://agent-ix/quoin/FR-120) (the mark and the Emitter contract), [FR-051](./FR-051-source-symbol-extraction.md) (the marker form and the `qualified_name` the reporter must equal).
- **Package home**: the `agent-ix/ix-trace-ts` repository, a sibling of `ix-trace-rs`. This requirement is held in quire-rs only until that repository exists and carries its spec, and is then to relocate to it; quire-rs keeps only the symbol-equality criterion (FR-080-AC-5) and the end-to-end demonstration (FR-080-AC-11).
- **Sibling**: [FR-079](./FR-079-pytest-trace-marker-plugin.md), the pytest binding of the same contract.
