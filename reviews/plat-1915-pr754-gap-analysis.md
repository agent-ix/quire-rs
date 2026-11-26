---
id: SR-967175026302197768900474181941935084905
title: "gap-analysis of quire-rs PR 754"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; PLAT-1915; PR754; src/registry.rs; FR-043"
review_set: subset
---

## Summary

Reviewed 999aaecbad6ea09ee0ab386d86e43adbc2d2088f. Two Unit Test constraints gain direct behavioral assertions and exact trace bindings.

## Verdict

FAIL for repository assurance — 140 exact-Test and two Unit Test criteria remain untagged repository-wide. No new gap in the two reviewed constraints.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Assurance Context

Read AP-201 detection/minting. Its stated engine measurement scope is not changed by these registry-load tests; no measurement or independent oracle claim is introduced. No applicable profile requires an additional gate for this test-only change.

## Coverage

Read the full one-file test-only diff, both FR-043 constraints and the adjacent FR-043 acceptance criteria. The malformed manifest has a real lexicon entry lacking required definition. Both direct load_module and parent search-root load_from must return ManifestError at the exact manifest path and name the missing definition; swallowing errors or returning an empty registry fails the assertions. The absent and explicit empty-map variants each execute load_module and inspect the returned empty lexicon. Existing valid-term loading and duplicate merge tests provide adjacent success/merge context. No mock substitutes the unit under test, no test is ignored, and no production cfg(test) bypass or new production function occurs.

Inspected Registry::load_module/load_from, fatal manifest error promotion, typed LexiconTermDef deserialization and lexicon merge/default behavior. Test setup reuses existing write_vocab_module/tmpdir helpers. No duplicate implementation/schema, dependency pin move, workflow change or new interface type. Every new test has the exact asserted constraint. No production code changed; implements backfill is outside this PR.

The branch fix/lexicon-module-load-constraints and title Test lexicon registry constraints are neutral. The PR body includes Refs PLAT-1915, lists the two criteria, related direct/search routes and focused tests. Ticket was explicitly assigned by the code lead rather than inferred from neutral branch/title.

Plan completion: not assessed

Fresh Quire 0.52.0/engine0.62.2 computed matrix with initialized pinned corpus resolves both new constraints to their expected test symbols. It reports 1,150 tagged exact-Test,140 untagged exact-Test, two tagged Unit Test and two untagged Unit Test IDs. The supplied older cohort is 136 Test+4 Unit to136 Test+2 Unit; its date denominator was not independently reconstructed. Matrix tags are static binding evidence, distinct from test execution. Reverse/stub checks inventoried the source/test tree and searched src for todo!/unimplemented!: none. Related loader/registry/lexicon behavior is owned by FR-013/FR-043. No new unowned behavior appears; full manual reverse ownership adjudication of the rest of the repository is incomplete, so no repository-wide PASS is claimed. Optional semantic audit was not separately requested.

## Examined units and bindings

```yaml
scope:
- id: FR-043-AC-1
  path: spec/functional/FR-043-module-concrete-lexicon.md
  role: context_only
  excerpt: A manifest declaring a `lexicon` entry loads, and `Registry::lexicon()`
    returns the merged map containing that term.
- id: FR-043-AC-2
  path: spec/functional/FR-043-module-concrete-lexicon.md
  role: context_only
  excerpt: Two modules declaring the same term with different definitions are first-wins
    and emit one `DuplicateLexiconTerm` diagnostic; identical redeclaration emits
    none.
- id: FR-043-AC-3
  path: spec/functional/FR-043-module-concrete-lexicon.md
  role: context_only
  excerpt: With a registry whose lexicon contains `pagination`, the statement `The
    system shall support pagination.` yields no `vague-response` finding; with the
    term removed, the same statement yields one.
- id: FR-043-AC-4
  path: spec/functional/FR-043-module-concrete-lexicon.md
  role: context_only
  excerpt: 'The engine contains no hardcoded concrete-noun list: a bare domain noun
    absent from the lexicon (and not backticked, with no mechanism/bound) yields a
    `vague-response` finding under an empty lexicon.'
- id: FR-043-AC-5
  path: spec/functional/FR-043-module-concrete-lexicon.md
  role: context_only
  excerpt: "Backticked-identifier, mechanism, and numeric-bound suppression (FR-042)\
    \ still hold under an empty lexicon \u2014 they do not depend on the lexicon."
- id: FR-043-AC-6
  path: spec/functional/FR-043-module-concrete-lexicon.md
  role: context_only
  excerpt: '`validate_document_in_registry` applies the registry''s merged lexicon;
    the type-only `validate_document` applies an empty lexicon (more findings) and
    never errors on the difference.'
- id: FR-043-AC-7
  path: spec/functional/FR-043-module-concrete-lexicon.md
  role: context_only
  excerpt: The `check_grammar` Python binding with a `module_root` applies that registry's
    lexicon; without one it applies an empty lexicon.
- id: FR-043-CON-1
  path: spec/functional/FR-043-module-concrete-lexicon.md
  role: examined
  excerpt: A malformed `lexicon` entry SHALL fail module load like any other manifest
    shape error
- id: FR-043-CON-2
  path: spec/functional/FR-043-module-concrete-lexicon.md
  role: examined
  excerpt: An empty or absent `lexicon` registry SHALL load without error
bindings:
- test_id: tests::malformed_lexicon_entry_fails_module_load
  ac_id: FR-043-CON-1
  trace: correct
- test_id: tests::absent_or_empty_lexicon_loads_empty_registry
  ac_id: FR-043-CON-2
  trace: correct
```

## Checks

Independent formatter/diff checks passed. Coder current-head locked cargo test --locked --lib lexicon passed13/13, zero failed/ignored,802 filtered. Optional spec-objects-architecture@0.10.3 fetch warning is unrelated; neither dependent skipped test matched this filter. Full workspace gates were not run under the focused-only campaign rule. Fresh matrix output is /tmp/review754-matrix.json with warnings retained separately. Quoin0.37.0 and spec-artifacts-process@v0.36.0 are the review artifact tooling release.

## Artifact retention

Commit as reviews/plat-1915-pr754-gap-analysis.md, then request final artifact-only disposition. Reviewer makes no branch edits.

Independent compiled-artifact lexicon rerun remained queued behind older waiters and was cancelled before launch (exit130). No independent test pass or rebuild is claimed; current-head coder13/13 supplies execution evidence. Source/test inventory343 files; no src todo!/unimplemented! matches.
