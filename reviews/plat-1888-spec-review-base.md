---
id: SR-20261103118157
title: base review of quire-rs source-diff impact requirements
type: SpecReview
analysis: base
scope: agent-ix/quire-rs; spec/functional/FR-099-diff-impact-analysis.md, spec/usecase/US-022-review-source-change-impact.md;
  PLAT-1888
review_set: subset
relationships:
- target: ix://agent-ix/quire-rs/FR-099
  type: reviews
---

## Summary

Initial base verdict: FAIL (P0/high) because FR-099-AC-10 required a revision-reporting obligation. The requirement was fixed by removing revision identifiers from the output. A separate missing-user-story-criteria defect found during the first disposition pass was fixed in commit 3ab7beaf. All 11 FR acceptance criteria remain untagged pending the implementation ticket, as expected for this spec-only PR.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | High, base checklist: FR-099-AC-10 required reporting old and new dependency revision identifiers in the impact report. The active checklist forbids requirements that record or track reviewed revisions. Remove that reporting obligation; the comparison can still use the revisions as inputs. | spec/functional/FR-099-diff-impact-analysis.md:168 | wrong-requirement |

Priority: P0 under the mandatory hash/revision antipattern checklist; severity: high.

## Verdict

PASS on the amended head after the recorded dispositions. The initial findings are preserved in their original text, both are fixed, and the 11 FR acceptance criteria remain untagged pending the implementation ticket.

## Review Scope

```yaml
- id: FR-099
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: When a consumer supplies a source change between two revisions, the Quire-RS
    impact engine SHALL report the changed Rust symbols, tests selected by the change,
    related implementations, and any part of the change that it could not connect
    to requirement links or tests.
- id: FR-099-AC-1
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: Given readable base and head revisions, the report maps changed Rust lines
    on both sides to the containing symbols, includes enclosing containers, and reports
    deleted base-side and added head-side symbols. An unavailable revision is an explicit
    error, not an empty report.
- id: FR-099-AC-2
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: Every changed line outside an extracted symbol is returned as an unmapped-change
    record with repository-relative path, revision side, and line range.
- id: FR-099-AC-3
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: Every test symbol added or edited by the diff is selected with a Rust test-runner
    filter and a direct-change reason; a deleted test is not selected.
- id: FR-099-AC-4
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: For a changed symbol implementing an exact id, the report selects every
    test verifying that id and lists every other symbol implementing it. A test or
    symbol with only a different id is not included by this rule.
- id: FR-099-AC-5
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: Every selected test and related symbol carries a complete reason chain
    from a changed path and line range through the relevant symbol and exact relation
    id; a directly edited test carries its direct-change chain.
- id: FR-099-AC-6
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: Requirement-level ids match only the same normalized requirement-level
    id and do not expand to criterion ids; criterion ids match only the same normalized
    criterion id; repository-prefixed ids remain distinct from local ids.
- id: FR-099-AC-7
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: A changed production symbol with no bound relation is reported as unlinked.
    An id that is absent from supplied specs, a spec that is unavailable, and a source
    marker that fails to bind produce distinct findings; a path-qualified non-binding
    Rust attribute is not reported as a successful implementation link.
- id: FR-099-AC-8
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: When extraction, traceability forms, or a referenced specification are
    unavailable, the affected files or ids are identified as unavailable or unverifiable,
    and the report does not claim an empty selection or unaffected tests.
- id: FR-099-AC-9
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: Caller following stops at its finite depth, returns only nearest linked
    callers once even in a cycle, adds their verified tests without removing earlier
    selections, reports unresolved call sites, and marks depth truncation when more
    caller edges remain.
- id: FR-099-AC-10
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: A dependency revision change reports the old and new revisions, changed
    upstream files and extracted items, local references, selected tests with upstream-to-local-to-test
    reason chains, and every unmatched upstream change as uncovered. An unavailable
    upstream tree is not reported as having no impact.
- id: FR-099-AC-11
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: Identical analysis inputs produce byte-identical report output and stable
    result ordering, without timestamps, temporary paths, or host-specific values.
- id: FR-099-CON-1
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: Impact selection SHALL be additive.
- id: FR-099-CON-2
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: The engine SHALL inspect source snapshots without building, executing,
    or type-resolving the changed project.
- id: FR-099-CON-3
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: The engine SHALL read implements and verifies relations from the existing
    binder.
- id: FR-099-CON-4
  path: spec/functional/FR-099-diff-impact-analysis.md
  role: examined
  excerpt: The engine SHALL NOT define or match another trace-marker grammar.
- id: FR-062
  path: spec/functional/FR-062-implements-relation.md
  role: context_only
  excerpt: The implements relation links requirements to production symbols and remains
    separate from verifies evidence relations.
- id: FR-077
  path: spec/functional/FR-077-trace-search-index.md
  role: context_only
  excerpt: Trace-id matching is exact after normalized_trace_id normalization, without
    implicit FR/AC/TC hierarchy expansion.
- id: US-022-AC-1
  path: spec/usecase/US-022-review-source-change-impact.md
  role: examined
  excerpt: Given a source change in a symbol linked to a requirement criterion and
    tests that declare the same criterion id, when the developer asks for change
    impact, then the result identifies those tests and shows the changed source,
    implementation, and criterion id that connect each test.
- id: US-022-AC-2
  path: spec/usecase/US-022-review-source-change-impact.md
  role: examined
  excerpt: Given a changed source area with no bound requirement link, or source or
    specification data that the analysis cannot inspect, when the developer asks
    for change impact, then the result identifies the unlinked or unavailable area
    without claiming that no tests are affected.
```

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-002 | medium | US-022 labels its two Given/When/Then entries “Acceptance Examples (Illustrative)” and says they add no verification criteria. The base checklist calls for at least two acceptance criteria per user story. Convert the examples to criteria or record why this story is exempt. | spec/usecase/US-022-review-source-change-impact.md:29-32 | missing-requirement |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The dependency-move output no longer reports old/new revision identifiers. |
| FND-002 | fixed | US-022 now has two explicitly numbered acceptance criteria. |

### After excerpts

#### FND-001

> A dependency revision change reports changed upstream files and extracted items, local references, selected tests with upstream-to-local-to-test reason chains, and every unmatched upstream change as uncovered. An unavailable upstream tree is not reported as having no impact.

#### FND-002

```text
**US-022-AC-1**: Given a source change in a symbol linked to a requirement
criterion and tests that declare the same criterion id, when the developer
asks for change impact, then the result identifies those tests and shows
the changed source, implementation, and criterion id that connect each test.
**US-022-AC-2**: Given a changed source area with no bound requirement link,
or source or specification data that the analysis cannot inspect, when the
developer asks for change impact, then the result identifies the unlinked or
unavailable area without claiming that no tests are affected.
```
