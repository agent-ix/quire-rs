---
id: SR-967175026302197768900474181941935084876
title: "code-review of quire-rs PR 745"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PLAT-1915; src/registry.rs; src/loader/mod.rs; src/loader/paths.rs; src/diagnostic.rs; FR-013"
review_set: subset
---

## Summary

Reviewed PR #745 at dd8c0c3c4d109cb0c182747cb88c33a0c27d2fb1. The lookup change preserves exact names and refuses ambiguous case-folded names; three claimed test-method closures need fuller evidence.

## Verdict

CONDITIONAL — three coverage findings require fixes before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-2 is marked covered by a synthetic name-only manifest rather than a copy of spec-artifacts-iso. The test declares eight names without schemas or other real module content, so a loader that fails to load the actual ISO module still passes this test. Use the specified module fixture and assert it loads without failures, or keep AC-2 open. | src/loader/mod.rs:2460 |
| FND-002 | medium | AC-8 is marked covered by a path-resolution-only test: no IX_SCHEMA_PATH is set, no module exists under the duplicated root, and no registry is loaded. A regression that loads the same module twice after resolution still passes. Exercise module loading through the duplicate tilde environment path and assert one load. Build the path list with join_paths; the literal colon also makes the added test fail on Windows. | src/loader/paths.rs:334 |
| FND-003 | medium | AC-14 requires both human and JSON rendering for all three PathTraversalReason values. The newly tagged test checks Display only for DotDotSegment; SymlinkEscape and EscapesModuleRoot only check the JSON reason field. A wrong human rendering for either latter reason still passes. Assert the required variant, argument, path and reason in both formats for each value. | src/diagnostic.rs:659 |

## Assurance Context

Inspected AP-201 at `spec/assurance/AP-201-detection-minting.md`. Its concern is a plausible backed-row population with missing bindings or minted rows. This review preserves the difference between static tags and behavioral discharge. Its required operations list is spec-review; this PR changes no spec. Unavailable selected context: none. Active exceptions: none.

## Coverage

The production change is limited to Registry::archetype: exact names win, and a unique ASCII case-insensitive match is accepted; ambiguous fallback returns None. The implements marker names AC-2 and is read by the trace tool. block_type delegates to archetype; validate_document and extract consumers use archetype. schema_for and archetype_in_module retain exact-name contracts and were inspected. No dependency revisions, public interface types, unsafe operations, or CI workflows change. Exact and ambiguous lookup behavior is exercised, though the AC-2 actual-module test remains incomplete.

Test tags point at relevant criteria, but AC-2, AC-8 and AC-14 only assert part of their full statements; `bindings.trace: correct` records the relevant link, not full criterion discharge. Their missing branches are the coverage findings above.

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

## Dispositions

Round 1 at 0b71d9724372aa77a32b21661a10ce5091efe83d. The three original findings are fixed. PR mergeable: no, because the new FND-004 has no resolution.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The synthetic ISO evidence was removed; matrix AC-2 is untagged with zero binders and the implementation marker remains separate. AC-2 is explicitly open, rather than incorrectly discharged. |
| FND-002 | fixed | The test now joins platform-separated duplicate tilde roots, sets IX_SCHEMA_PATH, loads the schema-bearing fixture via Registry::from_env and asserts one module/archetype. The former path-only test was removed. |
| FND-003 | fixed | A three-case loop checks variant, argument, path and reason in Display and JSON for every PathTraversalReason. |

## New findings — round 1

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | with_search_env now changes process-global HOME, but SEARCH_ENV_LOCK protects only the loader tests. Existing loader::paths tests expand_tilde_root and expand_tilde_subpath read HOME once to build their expected path and again inside expand_tilde without this lock. If a loader test changes or restores HOME between those reads, the otherwise-correct path test fails nondeterministically under the parallel test harness. Isolate environment-mutating tests in a subprocess, or use one shared guard for every affected environment reader/writer in the test binary. | src/loader/mod.rs:2407 |

Disposition pass checks: cargo fmt --all --check and git diff --check passed. Fresh quoin matrix has 1,136 tagged Test criteria, 154 untagged Test criteria and four untagged Unit Test criteria. AC-2 and AC-4 each have zero binders and remain open; AC-8 and AC-14 are tagged. Fresh registry trace lists the AC-2 implements claim and no AC-2 verifies claim. The coder reports all six named locked-build filters passing at this head; those individual filters do not exercise the new HOME race with the paths tests. The prior independent rerun was cancelled while queued (exit 130 before cargo); no reviewer test pass is claimed and no new reviewer test queue remains. The coder's build emitted the non-failing spec-objects-architecture-fixture warning: npm pack could not fetch @agent-ix/spec-objects-architecture@0.10.3 and the two dependent tests skip at runtime. No coverage is credited from those skipped tests.

## Disposition round 2

Rechecked fbafdd322800592ac749ef8ac573aef339fe663d. FND-004 is fixed and the three earlier findings remain fixed. No new findings. PR mergeable: yes.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | HOME_ENV_LOCK is shared by with_search_env and the affected paths tests. Sensitive HOME/IX_HOME reads and expected-versus-observed assertions stay inside the guard; restoration occurs before unlock. Pure injected-value tests and absolute/mid-path no-op assertions do not depend on HOME. The HOME reader in parser_real_documents is in a separate integration-test process. |

Round-2 checks: inspected the whole src/tests HOME and IX_HOME reader/writer search and the complete diff from round 1; no trace tags or production behavior changed. The code keeps AC-2 untagged and explicitly open, AC-8 exercises duplicated join_paths tilde roots through a schema-bearing fixture, and AC-14 checks the required fields in both formats for all three reasons. FND-001–003 remain fixed. cargo fmt --all --check and git diff --check passed independently. Coder evidence supplied for this head: locked-build loader::paths namespace 13/13 passed, plus six named filters each 1/1. The spec-objects-architecture-fixture package-fetch warning for @agent-ix/spec-objects-architecture@0.10.3 remains non-failing; the two dependent skipped tests are not credited as evidence. No new independent reviewer test run was queued. The prior reviewer rerun was cancelled before cargo started and no reviewer test pass is claimed.
