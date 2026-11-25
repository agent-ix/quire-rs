---
id: SR-20261103118160
title: dependency review of quire-rs source-diff impact requirements
type: SpecReview
analysis: dependency
scope: agent-ix/quire-rs; spec/functional/FR-099-diff-impact-analysis.md, spec/usecase/US-022-review-source-change-impact.md;
  PLAT-1888
review_set: subset
relationships:
- target: ix://agent-ix/quire-rs/FR-099
  type: reviews
---

## Summary

The initial dependency review found caller following relied on unspecified extracted call edges. The later specification defines bounded edge derivation from Rust call-site syntax and extracted symbol identities; no dependency gap remains at the current head.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-099-AC-9 requires following extracted caller edges, but no upstream requirement defines or supplies those edges. Specify their source and matching boundary, or make edge data an explicit input. | spec/functional/FR-099-diff-impact-analysis.md:167 | missing-requirement |

## Verdict

PASS on the amended head after the recorded disposition. The initial finding is preserved in its original text and fixed; the 11 FR acceptance criteria remain untagged pending the implementation ticket.

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
- id: FR-051
  path: spec/functional/FR-051-source-symbol-extraction.md
  role: context_only
  excerpt: The extractor returns tree symbols with stable identities and typed relations;
    the stated Rust extraction covers symbols and source spans, not caller edges.
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The requirement now derives and bounds caller edges from Rust syntax and extracted symbols. |
| FND-001 | fixed | Rechecked at head 3ab7beaf; caller-edge sourcing and limits remain, with no regression. |

### After excerpt

```text
The engine SHALL derive caller edges from Rust call-site syntax and the
extracted Rust symbol identities. The engine SHALL resolve a call site only
when its statically named callee path matches exactly one extracted Rust
symbol path.
```

### Round 2 after excerpt

```text
The engine SHALL derive caller edges from Rust call-site syntax and the
extracted Rust symbol identities. The engine SHALL resolve a call site only
when its statically named callee path matches exactly one extracted Rust
symbol path.
```
