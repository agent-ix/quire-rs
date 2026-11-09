---
id: SR-20261103118142
title: "Code review — PLAT-214 duplicate semantic export refused at load (quire-rs PR 629)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/loader/mod.rs (semantic_cross_module_checks), tests/it/semantic_contract.rs, CHANGELOG.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-214. PR head `accd773`, base `origin/main` `4b744c6`. This is a code review with the rust-review lane folded in. The diff adds an export pass to `semantic_cross_module_checks`. It runs after the duplicate-package pass and before the import pass. The diff also adds one integration test, a one-line change to the import-cycle fixture, and a CHANGELOG entry.

## Verdict

**Changes requested (one medium finding).** The main path is correct:

- Modules are visited in sorted module-root order, so the outcome does not depend on load order. A probe that called `Registry::load_from` with the two roots in reversed argument order gave the same refusal.
- The reason is `semantic.duplicate-export: semantic.exports.<type>: type <type> is also exported by module <first> (<root>); later sorted root refused`. It names the type and the first module. The refused module is the failure's `module` field. This is the same pattern as `semantic.duplicate-package`.
- The refused module's archetypes are drained and its `semantic` block is set to `None` before `flatten_into_registry` builds `active_archetypes`. A probe confirmed that the two-module refusal case emits no `DuplicateArchetype`.
- The import-cycle fixture change is justified. Both cycle modules are copies of `module-ok`, which exports `entity`. Without the change, `y-cycle` would be refused for the duplicate export and dropped from `providers`. The import from `x` would then be unresolved, the cycle would never form, and the FR-069-AC-10 assertion would fail for the wrong reason.

The defect is that a refused module's non-clashing exports still claim their names (FND-001).

Gates run on the PR head: `cargo fmt --check` (clean), `cargo clippy --locked --all-targets -- -D warnings` (clean), and `cargo test --test it` (623 passed, 1 failed). The one failure is `corpus_cases::tc1026_the_declared_vocabulary_matches_the_engine`, which fails the same way on `origin/main` `4b744c6`. The cause is the corpus submodule pointer, which PR #626 moved back from `3e0686c` to `e230217`. It is not caused by this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A module that is refused for one duplicate export still inserts its other exports into `exported`. The clash list is collected, but the `None => insert` arm has already run for every non-clashing name. A third module that exports one of those names is then refused with a reason that cites a module that was itself refused. Reproduced: a-first `[entity]`, b-second `[entity, enumeration]`, c-third `[enumeration]` gives a-first loaded, b-second refused (entity), and c-third refused with "type enumeration is also exported by module b-second". `enumeration` then has no exporter. Fix: decide the clashes for module i before inserting any of its names, and insert only when there is no clash. Add the three-module case to TC-2190. | src/loader/mod.rs:1819-1828 |
| FND-002 | low | The CHANGELOG entry is under `### Changed` with no **Breaking** marker. A module set that loaded before now fails every archetype of one module, and the repository marks new load refusals as **Breaking** (for example, FR-075 `semantic.feature-not-extractable`). No released module pair is known to trip the check, because the released app/iso pair spells the type differently. | CHANGELOG.md:10-14 |
| FND-003 | low | The doc comment on `semantic_cross_module_checks` still lists only the duplicate-package, unresolved-import and import-cycle checks. It does not mention the new duplicate-export refusal. | src/loader/mod.rs:1769-1773 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/loader/mod.rs export pass (1806-1840) | examined | FND-001 |
| src/loader/mod.rs refusal application (1947-1960) | examined | Clean: drains archetypes, clears semantic |
| src/loader/mod.rs flatten_into_registry DuplicateArchetype (1137-1162) | context_only | Runs after refusal; no stray diagnostic |
| src/semantic/contract.rs exports parsing / export-undeclared | context_only | Exact-string names |
| tests/it/semantic_contract.rs duplicate_export_refuses_the_later_module | examined | Passes; gaps recorded in gap-analysis SR |
| tests/it/semantic_contract.rs cross_module_checks y-cycle change | examined | Justified |
| CHANGELOG.md | examined | FND-002 |

## Verification

The probes ran as uncommitted tests in a review worktree, with `CARGO_PROFILE_DEV_DEBUG=0` and through `locked-build.sh`:

- No `DuplicateArchetype` on refusal: confirmed.
- Reversed `load_from` argument order: same outcome.
- Three-module export cascade: reproduces FND-001.
- Non-empty disjoint control (b exports `[enumeration]`): both modules load and there are no failures.

`tc1026` was run on a separate `origin/main` worktree with its own target directory and the submodule initialised. It failed with the same three missing ids: `function-not-evidence`, `not-in-tag-channel` and `outside-entry-paths`.
