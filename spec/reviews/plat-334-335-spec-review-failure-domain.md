---
id: SR-16374
title: "Failure-domain review of FR-079 and FR-080 runner feasibility (quire-rs PR 613)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs; spec/functional/FR-079-pytest-trace-marker-plugin.md, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md (PR 613 diff only)"
review_set: subset
---

## Summary

Tickets: PLAT-334, PLAT-335. This review looks for unstated failure modes and runner behaviour the FRs assume. pytest was measured directly: pytest 9.0.2 was run with a probe plugin, `--junit-xml` and `junit_family` set to xunit1 and then xunit2. vitest was read from the source of `@vitest/runner` 4.1.9 installed under `~/dev/cli-agent-evals`. jest was not installed locally, and its findings rest on its documented reporter API. The pytest property mechanism works. The other runner assumptions have gaps the FRs do not record.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-080 names a per-test channel only for vitest (`for vitest the task's meta`). For jest it names none, and it rules out a file. Jest runs test bodies in worker processes. The reporter receives `AssertionResult` (title, fullName, ancestorTitles, status, location, ...), which has no user metadata field, so there is no documented way for a `trace()` call in a test body to reach the reporter. The FR neither names a mechanism nor records the limit, yet FR-080-AC-5/8/11 require jest. Name the jest mechanism (for example a custom test environment, and how its data reaches the reporter), or record jest as unsupported until one exists. | spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:30-34 |
| FND-002 | medium | pytest's default `junit_family` is `xunit2`, and it cannot carry `file`. Measured on pytest 9.0.2: with `name`/`classname`/`file` set through the node reporter, xunit2 kept the `trace` property and the renamed `name`/`classname` but dropped `file`, while xunit1 kept all of them. Under the FR's own `Failure is loud` rule, every enabled run on a default-configured project therefore fails. Neither the FR nor FR-079-AC-8 says so. State that an enabled run needs `junit_family=xunit1`, and name xunit2 as the failing family in AC-8. | spec/functional/FR-079-pytest-trace-marker-plugin.md:61-66, spec/functional/FR-079-pytest-trace-marker-plugin.md:86 |
| FND-003 | medium | The `it.each` template title is not available at run time. In `@vitest/runner` 4.1.9, `taskFn.each` registers each case as `test(formatTitle(_name, items, idx), ...)` and keeps only `each: true`, not the template. jest-each also interpolates before registering. The reporter therefore cannot report `adds %s` from runner data. It would have to map the task location back to source, or wrap `it.each`. The FR says neither, and its line 59 implies runtime titles. Record the mechanism, or record the limit. | spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:56, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:85 |
| FND-004 | medium | `trace(...ids)` takes no test context, so it must find the running test through a global. In vitest, `getCurrentTest()` reads one module-level `_test` (`@vitest/runner` 4.1.9, `setCurrentTest`). Under `it.concurrent`, which FR-080-AC-6 names, a `trace()` after an `await` can attach ids to another test, breaking E-5. State how the helper binds to its own test under concurrency, or exclude concurrent tests explicitly. | spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:28-34, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:86 |
| FND-005 | medium | FR-079 emits ids from markers `on the function, its class, or its module` (FR-079-AC-3). The static side does not bind those. FR-051-AC-4 binds a `@pytest.mark.trace` decorator to the def it decorates. A class is a container that binds no trace ids (CR-061), and a module-level `pytestmark = pytest.mark.trace(...)` is outside every test's span. The execution view would therefore bind ids that `quire coverage` reports as unbound or misplaced, and the two views would disagree. Either emit only function-level markers, or add the static binding (FR-051) in the same change. | spec/functional/FR-079-pytest-trace-marker-plugin.md:40-43, spec/functional/FR-079-pytest-trace-marker-plugin.md:81 |

## Verdict

Measured clean: the pytest probe put the accumulated `trace` property on every case, including a failing parametrized case, an xfail (`<skipped type="pytest.xfail">`) and a skip. One `<testcase>` was written per parametrized case, `name` and `classname` overrode pytest's dotted module default, and `--strict-markers` passed with the marker registered. These results back FR-079-AC-1/3/5/6. The rename must run where `LogXML` lives, which under xdist is the controller. FR-079-AC-7 covers that. FND-001 blocks merge, and FND-002 to FND-005 should be fixed in this PR.
