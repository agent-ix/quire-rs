---
id: SR-967175026302197768900474181941935084877
title: "gap-analysis of quire-rs PR 745"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; PLAT-1915; src/registry.rs; src/loader/mod.rs; src/loader/paths.rs; src/diagnostic.rs; FR-013"
review_set: subset
---

## Summary

Reviewed PR #745 at dd8c0c3c4d109cb0c182747cb88c33a0c27d2fb1. The lookup change preserves exact names and refuses ambiguous case-folded names; three claimed test-method closures need fuller evidence.

## Verdict

FAIL for repository assurance — 153 Test and four Unit Test criteria remain untagged; AC-2, AC-8 and AC-14 have partial evidence as recorded in the code review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Assurance Context

Inspected AP-201 at `spec/assurance/AP-201-detection-minting.md`. Its concern is a plausible backed-row population with missing bindings or minted rows. This review preserves the difference between static tags and behavioral discharge. Its required operations list is spec-review; this PR changes no spec. Unavailable selected context: none. Active exceptions: none.

## Coverage

Plan completion: not assessed

The computed repository matrix has 1,137 tagged Test criteria, 153 untagged Test criteria and four untagged Unit Test criteria. FR-013-AC-4 remains untagged. Its full 17-archetype canonical corpus is not vendored or pinned here; benches/load.rs explicitly measures only the demo module. The PR correctly leaves AC-4 open and does not claim its benchmark exists. Tagged status for AC-2, AC-8 and AC-14 overstates full behavioral coverage; see the separate code-review findings.

Reverse and stub procedure: inventoried 113 source Rust files and the Registry public surface, loader roots and diagnostic variants; searched the whole source for todo!/unimplemented! (zero matches). In the changed area, lookup is owned by FR-013, Send+Sync by AC-9, environment loading by AC-1/2, duplicate roots by AC-8 and path diagnostics by AC-10/14. No new unowned production behavior or hollow production function was found. No mocks, skipped assertions, or empty test bodies occur in this diff. Full manual ownership adjudication outside the changed area remains incomplete; no repository PASS is claimed. Optional semantic audit was not requested and did not run.

## Examined units and bindings

```yaml
scope:
- id: FR-013-AC-1
  path: spec/functional/FR-013-archetype-loader.md
  role: examined
  excerpt: '`Registry::from_env()` with `IX_FILAMENT_MODULES_PATH` and `IX_SCHEMA_PATH`
    unset and no `~/.ix/filament/modules/` returns a registry with zero archetypes
    and no error.'
- id: FR-013-AC-2
  path: spec/functional/FR-013-archetype-loader.md
  role: examined
  excerpt: Pointing `IX_SCHEMA_PATH` at a path containing a copy of `spec-artifacts-iso/spec_artifacts_iso/`
    produces a registry with the 8 ISO archetypes (FR, NFR, StR, US, IT, TC, AC, CON)
    all loaded and `archetype("fr")` returns Some.
- id: FR-013-AC-4
  path: spec/functional/FR-013-archetype-loader.md
  role: examined
  excerpt: 'A criterion bench measures `Registry::load_from(&[corpus_path])` for the
    full 17-archetype baseline corpus and reports a one-time cost (target: under 50
    ms median on baseline hardware).'
- id: FR-013-AC-8
  path: spec/functional/FR-013-archetype-loader.md
  role: examined
  excerpt: A test sets `IX_SCHEMA_PATH="~/foo:~/foo"` (duplicate canonical path) and
    asserts modules under `~/foo/` are loaded exactly once.
- id: FR-013-AC-9
  path: spec/functional/FR-013-archetype-loader.md
  role: examined
  excerpt: 'A test confirms `Registry: Send + Sync` (compile-time bound assertion
    via a generic helper function).'
- id: FR-013-AC-10
  path: spec/functional/FR-013-archetype-loader.md
  role: examined
  excerpt: A test sets a search-path entry that points to a regular file (not a directory)
    and asserts the loader emits a warning and processes remaining entries normally.
- id: FR-013-AC-14
  path: spec/functional/FR-013-archetype-loader.md
  role: examined
  excerpt: '`Diagnostic::PathTraversal { argument, path, reason }` is a defined variant
    of the (internal) `Diagnostic` enum. A unit test constructs the variant and asserts
    both human (`Display`) and JSON (`to_json`) renderings carry the variant name,
    argument, path, and reason discriminator, covering all three `PathTraversalReason`
    values.'
bindings:
- test_id: from_env_without_search_roots_returns_an_empty_registry
  ac_id: FR-013-AC-1
  trace: correct
- test_id: schema_path_loads_the_eight_iso_archetypes
  ac_id: FR-013-AC-2
  trace: correct
- test_id: case_folded_lookup_does_not_choose_between_ambiguous_names
  ac_id: FR-013-AC-2
  trace: correct
- test_id: duplicate_tilde_paths_dedupe_after_expansion
  ac_id: FR-013-AC-8
  trace: correct
- test_id: registry_is_send_sync_and_cheap_to_clone
  ac_id: FR-013-AC-9
  trace: correct
- test_id: file_search_root_warns_and_other_roots_still_load
  ac_id: FR-013-AC-10
  trace: correct
- test_id: path_traversal_variant_renders_human_and_json
  ac_id: FR-013-AC-14
  trace: correct
```

## Checks

`cargo fmt --all --check` and `git diff --check` passed. `quoin matrix --repo . --json` and `quire trace --scope . --file src/registry.rs --json` exited 0; the latter lists the AC-2 implementation and test claims. Coder reported six named focused tests, the ambiguity test and a final tilde rerun passing. Independent rerun of the four schema-path, ambiguity, duplicate-tilde and PathTraversal tests was attempted through locked-build.sh; it remained queued while load was 31.01 (threshold under 18), and was cancelled with exit 130 before cargo launched. No independent test pass is claimed. The source-established coverage findings do not depend on another green run. Full suite and broad lint/deny gates were excluded by the owner's focused-only campaign rule; no dependency or workflow changes occur.

## Disposition round 1

Rechecked 0b71d9724372aa77a32b21661a10ce5091efe83d. AC-2 no longer has a test binder, accurately leaving its full-module gap open; AC-4 remains open. AC-8 and AC-14 now exercise the requested behavior, but the new environment helper introduces the parallel HOME race documented as FND-004 in the code review. Repository assurance remains FAIL; PR mergeable: no.

Disposition pass checks: cargo fmt --all --check and git diff --check passed. Fresh quoin matrix has 1,136 tagged Test criteria, 154 untagged Test criteria and four untagged Unit Test criteria. AC-2 and AC-4 each have zero binders and remain open; AC-8 and AC-14 are tagged. Fresh registry trace lists the AC-2 implements claim and no AC-2 verifies claim. The coder reports all six named locked-build filters passing at this head; those individual filters do not exercise the new HOME race with the paths tests. The prior independent rerun was cancelled while queued (exit 130 before cargo); no reviewer test pass is claimed and no new reviewer test queue remains. The coder's build emitted the non-failing spec-objects-architecture-fixture warning: npm pack could not fetch @agent-ix/spec-objects-architecture@0.10.3 and the two dependent tests skip at runtime. No coverage is credited from those skipped tests.

## Disposition round 2

Rechecked fbafdd322800592ac749ef8ac573aef339fe663d. All code-review findings are resolved. Scoped PR mergeable: yes. The repository assurance verdict remains FAIL from the existing untagged criteria; AC-2 and AC-4 remain open.

Round-2 checks: inspected the whole src/tests HOME and IX_HOME reader/writer search and the complete diff from round 1; no trace tags or production behavior changed. The code keeps AC-2 untagged and explicitly open, AC-8 exercises duplicated join_paths tilde roots through a schema-bearing fixture, and AC-14 checks the required fields in both formats for all three reasons. FND-001–003 remain fixed. cargo fmt --all --check and git diff --check passed independently. Coder evidence supplied for this head: locked-build loader::paths namespace 13/13 passed, plus six named filters each 1/1. The spec-objects-architecture-fixture package-fetch warning for @agent-ix/spec-objects-architecture@0.10.3 remains non-failing; the two dependent skipped tests are not credited as evidence. No new independent reviewer test run was queued. The prior reviewer rerun was cancelled before cargo started and no reviewer test pass is claimed.
