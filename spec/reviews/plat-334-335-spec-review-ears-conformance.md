---
id: SR-16371
title: "EARS conformance review of FR-079 and FR-080 (quire-rs PR 613)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs; spec/functional/FR-079-pytest-trace-marker-plugin.md, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md (PR 613 diff only)"
review_set: subset
---

## Summary

Tickets: PLAT-334, PLAT-335. This review checks the requirement statements of FR-079 and FR-080, and the `While ...` prefixes of their ACs, against the EARS grammar. `quire validate` raises no `ears:` or `quality:` warning on either file. Each statement has one SHALL, but it joins a ubiquitous response and a state-driven response.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The FR-079 statement joins two responses with different triggers: a ubiquitous one (`SHALL register the trace marker`) and a state-driven one (`while emission is enabled, write the ids`). The FR's own Registration section says registration is independent of emission. Split the statement into a ubiquitous registration statement and a `While QUIRE_TRACE_EMIT=1, the plugin SHALL write ...` statement. | spec/functional/FR-079-pytest-trace-marker-plugin.md:18-23 |
| FND-002 | low | The FR-080 statement has the same mix. `SHALL provide a typed trace(...ids) helper` is ubiquitous, and `a reporter ... that, while emission is enabled, writes` puts the state clause inside a relative clause. Split it into a ubiquitous helper statement and a `While emission is enabled, the reporter SHALL write ...` statement. | spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:18-22 |

## Verdict

Examined: the FR-079 and FR-080 statements, the `Emission` and `Limit` paragraphs, and the AC prefixes. The ACs read `While enabled` / `While QUIRE_TRACE_EMIT is unset or not 1` as state preconditions, and FR-079-AC-1 states its result `whether or not emission is enabled`. Those forms are correct. Both findings are about wording only. Neither blocks merge.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Commit eaf5dbd splits FR-079 into a ubiquitous `The pytest plugin SHALL register the trace marker.` and a `While emission is enabled, the pytest plugin SHALL write ...` statement (FR-079:18-24). |
| FND-002 | fixed | Commit eaf5dbd splits FR-080 into a ubiquitous helper statement and a `While emission is enabled, the package's vitest reporter SHALL write ...` statement (FR-080:18-23). |
