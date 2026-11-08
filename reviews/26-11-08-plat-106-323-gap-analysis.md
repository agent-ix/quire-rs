---
id: SR-16306
title: "gap-analysis of module-path union and named-manifest failure (quire-rs PR 589)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@126f813; src/loader/paths.rs; src/registry.rs; spec/functional/FR-013-archetype-loader.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-013
    type: references
---

# Gap analysis of PR 589 (PLAT-106, PLAT-323)

## Summary

Tickets: PLAT-106, PLAT-323. Planless gap analysis scoped to
`git diff origin/main...126f813`. Plan completion: not assessed. Requirement
owner: FR-013 (search path, the named-module constructors, error model). The
reviewer read each AC in FR-013 that the diff touches. FR-013-AC-1, AC-2,
AC-8, AC-13 and AC-17 were checked against the changed code, and their
behaviour holds. No stub, `todo!` or placeholder return was found in the
changed code.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [trace] The new behaviour has no owning AC, and the new tests carry no `#[trace]` tag. `env_roots_keep_default_root` (the PLAT-106 union and default root) and `named_module_with_future_manifest_key_fails_loud` (PLAT-323) are both untagged. FR-013 has no AC for 'both variables are unioned and the default root is kept', and none for 'a named module whose manifest fails to parse is an error naming the key and module version'. So `quire matrix` cannot see either test. Add FR-013 ACs for both behaviours and tag the tests. | src/loader/paths.rs:256-282; src/registry.rs:1710-1728; spec/functional/FR-013-archetype-loader.md (Acceptance Criteria) | correct-requirement-no-evidence |
| FND-002 | medium | [coverage] PLAT-323 asks for a version-skew test (a manifest with a future key fails loudly). The one test added uses an unknown key nested under `traceability:`, which already failed at parse on main because `TraceabilityModel` is `deny_unknown_fields`. Two cases have no test: an unknown top-level key, and a future key in a module found through a search root (`load_from` or `from_env`, the shared install root of the ticket). Both are silent today (SR-16305 FND-002, FND-003), and no test would fail if they stayed that way. | src/registry.rs:1710-1728 | correct-requirement-no-evidence |

## Verdict

Not complete. The changed behaviour has no requirement owner and no trace
tags, and the version-skew test does not cover the two cases that are still
silent. Both close with the FR-013 amendment that SR-16305 FND-006 calls for,
together with the tests that come with the PLAT-323 fix.
