---
id: FR-099
title: "Change impact analysis from source diffs"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-062"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-077"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-022-review-source-change-impact"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-007"
    type: "traces_to"
---
# FR-099: Change impact analysis from source diffs

## Description

When a consumer supplies a source change between two revisions, the Quire-RS
impact engine SHALL report the changed Rust symbols, tests selected by the
change, related implementations, and any part of the change that it could not
connect to requirement links or tests.

The engine reuses the existing source-symbol extraction and the
[`implements` relation](./FR-062-implements-relation.md) and trace search
([FR-077](./FR-077-trace-search-index.md)). It does not define another marker
grammar or reinterpret the binder's result. Each returned test and related
symbol names the chain of diff evidence, symbol, and exact trace id that led to
it. Tests added or edited by the diff are selected directly. Tests that claim
an id implemented by changed code are selected by exact id. Other code
implementing those ids is listed for review.

The engine treats ids as complete identifiers, not as a language hierarchy.
An id such as `FR-099` does not imply `FR-099-AC-1`; code that implements
individual criteria must declare those criterion ids. An id with a repository
prefix such as `FSpec-FR-…` remains distinct from a similarly numbered local
`FR-…` id.

The first supported impact analysis is for Rust source, using structural symbol
extraction. Other source languages are not covered by this requirement. If a
change includes unsupported or unavailable source, the report identifies it as
uncovered rather than treating it as a change with no affected symbols.
The source locations and reason chains make the findings interpretable by
assurance owners and reviewers ([StR-007](../stakeholder/StR-007-source-grounded-assurance-data.md)).

Out of scope: choosing or serializing the CLI command's human and machine
formats; running selected tests; building or executing the changed project;
inferring requirement-to-criterion hierarchies; type resolution; and remote
fetching of source or specification repositories that were not supplied to the
engine.

## Inputs

- The repository source at a base revision and a head revision, together with
  the change between them.
- The traceability model and specification artifacts available to the analysis.
- An optional finite caller-search depth for following calls from unlinked
  changed symbols.
- For a dependency revision move, the old and new revision identifiers and
  the corresponding upstream source snapshots when available.

## Outputs

- Changed symbols and changed lines, each identified by repository-relative
  path and revision side.
- Selected tests, related implementing symbols, and a reason chain for every
  result.
- Link and extraction findings, including unresolved or uncovered changes.
- Dependency-move evidence and caller-search evidence when those analyses are
  requested.

## Behavior

- When the engine cannot read a supplied base or head revision, the engine SHALL
  return an error that identifies the unavailable or invalid revision.
- The engine SHALL NOT return an empty impact report for a change it could not
  read.
- The engine SHALL map changed lines in both revisions to the Rust symbols
  whose source spans contain them. When a changed symbol is nested in a
  container, the engine SHALL also include enclosing extracted containers
  that carry requirement links relevant to that change.
- The engine SHALL represent deleted symbols from the base revision.
- The engine SHALL represent added symbols from the head revision.
- The engine SHALL retain each changed line that maps to no extracted symbol
  as an unmapped-change record naming its path, revision side, and line range.
- The engine SHALL select every test symbol added or edited by the diff.
- For each selected Rust test, the engine SHALL include a test-runner filter.
- The engine SHALL NOT present a deleted test as a test to run.
- When a changed symbol has an `implements` relation, the engine SHALL select
  every test whose `verifies` relation carries the same exact id.
- When a changed symbol has an `implements` relation, the engine SHALL list
  every other symbol whose `implements` relation carries that exact id.
- Every selected test and related symbol SHALL carry at least one complete
  reason chain. A chain SHALL identify the originating changed path and line
  range, the changed or directly edited test symbol, the implementing symbol
  when applicable, and the exact relation id that connects them.
- The engine SHALL use the identifier normalization defined by FR-077 when
  matching relation ids.
- The engine SHALL compare the entire normalized id.
- The engine SHALL NOT expand a requirement id into its acceptance-criterion
  ids or infer equivalence from a numeric suffix.
- The engine SHALL preserve the full spelling of an id from another repository.
- The engine SHALL NOT collapse an id from another repository into a local id.
- When a changed production symbol has no bound `implements` relation, the
  engine SHALL report it as unlinked.
- The engine SHALL distinguish an id that fails to resolve in supplied
  specification artifacts, an id whose specification is unavailable, and a
  source marker that does not bind to the symbol.
- The engine SHALL report a path-qualified or otherwise non-binding Rust
  attribute containing a requirement id as non-binding.
- The engine SHALL NOT report a non-binding Rust attribute as a successful link.
- The engine SHALL use FR-062 binding results and FR-077 mention records for
  this check.
- The engine SHALL NOT implement a second trace-marker grammar.
- When source extraction or specification lookup is unavailable for a changed
  file or id, the engine SHALL report the affected scope as unavailable or
  unverifiable.
- The engine SHALL NOT present an incomplete analysis as an empty selection or
  as evidence that remaining tests are unaffected.
- The engine SHALL follow extracted caller edges to linked symbols no farther
  than the supplied finite depth when caller following is requested for an
  unlinked changed symbol.
- When caller following reaches one or more linked symbols, the engine SHALL
  return linked callers at the nearest reached depth.
- The engine SHALL derive caller edges from Rust call-site syntax and the
  extracted Rust symbol identities.
- The engine SHALL resolve a call site only when its statically named callee
  path matches exactly one extracted Rust symbol path.
- The engine SHALL report a call site as unresolved when its callee is dynamic,
  ambiguous, aliased, or does not match one extracted Rust symbol path exactly.
- The engine SHALL NOT use type resolution or infer call edges through macros,
  function pointers, or dynamic dispatch.
- The engine SHALL return a symbol no more than once during caller following,
  including when the call graph contains a cycle.
- The engine SHALL add tests verifying each reached caller's implemented ids
  to the selection with the full call path in each reason chain.
- The engine SHALL add tests selected by caller following without removing
  tests selected by another rule.
- The caller-search result SHALL list unresolved call sites encountered during
  traversal.
- The engine SHALL mark caller-search results as truncated when further caller
  edges remain beyond the supplied depth limit.
- When the engine analyzes a dependency revision move, it SHALL compare the
  supplied source snapshots and report changed upstream files and extracted
  items.
- The engine SHALL report local files that refer to those upstream changes and
  tests selected through those local references.
- When dependency-move analysis selects a test, the engine SHALL include the
  upstream-to-local-to-test reason chain.
- The engine SHALL list unmatched upstream files or items as uncovered.
- The engine SHALL identify each unavailable upstream revision or source
  snapshot in the report.
- The engine SHALL NOT imply that no local tests are affected when an upstream
  revision or source snapshot is unavailable.
- Given the same base and head trees, change, traceability model, available
  specifications, and caller-search depth, the engine SHALL produce the same
  report and ordering.
- The engine SHALL NOT include wall-clock time, temporary paths, or
  host-specific values in that report.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-099-CON-1 | Impact selection SHALL be additive. | Design | Inspection |
| FR-099-CON-2 | The engine SHALL inspect source snapshots without building, executing, or type-resolving the changed project. | Design | Inspection |
| FR-099-CON-3 | The engine SHALL read `implements` and `verifies` relations from the existing binder. | Design | Inspection |
| FR-099-CON-4 | The engine SHALL NOT define or match another trace-marker grammar. | Design | grammar-based-fuzzing |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-099-AC-1 | Given readable base and head revisions, the report maps changed Rust lines on both sides to the containing symbols, includes enclosing containers, and reports deleted base-side and added head-side symbols. An unavailable revision is an explicit error, not an empty report. | Test |
| FR-099-AC-2 | Every changed line outside an extracted symbol is returned as an unmapped-change record with repository-relative path, revision side, and line range. | Test |
| FR-099-AC-3 | Every test symbol added or edited by the diff is selected with a Rust test-runner filter and a direct-change reason; a deleted test is not selected. | Test |
| FR-099-AC-4 | For a changed symbol implementing an exact id, the report selects every test verifying that id and lists every other symbol implementing it. A test or symbol with only a different id is not included by this rule. | Test |
| FR-099-AC-5 | Every selected test and related symbol carries a complete reason chain from a changed path and line range through the relevant symbol and exact relation id; a directly edited test carries its direct-change chain. | Test |
| FR-099-AC-6 | Requirement-level ids match only the same normalized requirement-level id and do not expand to criterion ids; criterion ids match only the same normalized criterion id; repository-prefixed ids remain distinct from local ids. | Test |
| FR-099-AC-7 | A changed production symbol with no bound relation is reported as unlinked. An id that is absent from supplied specs, a spec that is unavailable, and a source marker that fails to bind produce distinct findings; a path-qualified non-binding Rust attribute is not reported as a successful implementation link. | Test |
| FR-099-AC-8 | When extraction, traceability forms, or a referenced specification are unavailable, the affected files or ids are identified as unavailable or unverifiable, and the report does not claim an empty selection or unaffected tests. | Test |
| FR-099-AC-9 | Caller following resolves only statically named paths that exactly match one extracted Rust symbol path; dynamic, ambiguous, aliased, or unmatched calls are unresolved. Following stops at its finite depth, returns only nearest linked callers once even in a cycle, adds their verified tests without removing earlier selections, reports unresolved call sites, and marks depth truncation when more caller edges remain. | Test |
| FR-099-AC-10 | A dependency revision change reports changed upstream files and extracted items, local references, selected tests with upstream-to-local-to-test reason chains, and every unmatched upstream change as uncovered. An unavailable upstream tree is not reported as having no impact. | Test |
| FR-099-AC-11 | Identical analysis inputs produce byte-identical report output and stable result ordering, without timestamps, temporary paths, or host-specific values. | Test |

## Dependencies

- **Upstream**: [FR-062](./FR-062-implements-relation.md) supplies the
  requirement-to-production-code relation; [FR-077](./FR-077-trace-search-index.md)
  supplies exact forward and inverse lookup over the existing relations and
  mentions; [FR-051](./FR-051-source-symbol-extraction.md) supplies structural
  Rust symbols and source spans.
- **Downstream**: the quire-cli impact command presents this report; the
  engine and CLI contracts are specified separately.
