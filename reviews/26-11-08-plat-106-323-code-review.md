---
id: SR-16305
title: "code-review of module-path union and named-manifest failure (quire-rs PR 589)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@126f813; src/loader/paths.rs; src/registry.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-013
    type: references
---

# Code review of PR 589 (PLAT-106, PLAT-323)

## Summary

Tickets: PLAT-106, PLAT-323. One code-review pass, with the Rust lane folded
in, over `git diff origin/main...126f813` (2 files, +115/-24). The reviewer ran
these gates on the head: `cargo fmt --check` is clean. `cargo clippy --lib
--tests -D warnings` (default features) is clean. The focused lib tests (17)
pass. `cargo test --test it` fails 3 of 520 tests.

Examined units:

- `module_path_env` and `resolve_search_paths` (PLAT-106). With both
  variables set, the root order is `[IX_FILAMENT_MODULES_PATH roots,
  IX_SCHEMA_PATH roots, default root]`. That is the order quire-cli's
  `scoped_registry_roots` uses (`ix-cli-kit` `SearchPath::push_envs`, then
  `default_module_root`), less the CLI-only scope roots. Canonical dedup is
  unchanged. On Linux this matches quire-cli.
- `split_search_paths` now calls `std::env::split_paths`, as quire-cli does.
- `Registry::fail_on_unreadable_manifest`, called from `load_module` and
  `load_module_set` (PLAT-323). The `is_file()` guard keeps FR-013-AC-13
  intact: an absent manifest still yields a tolerant registry with one failure.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | [code-bug] The PR leaves `tests/it` red. `load_module` and `load_module_set` now return `Err` when a manifest fails to parse, but three integration tests still expect the old tolerant `Ok` from `load_module`. The reviewer reproduced all three with `cargo test --test it`: `obligations::tc833_ambiguous_or_originless_source_fails_load` (FR-053-AC-3; panics at obligations.rs:151, `expect("tolerant load")`), `coverage_rollup::retired_declarations_keep_live_gaps_and_historical_identity` (coverage_rollup.rs:3320, `expect("tolerant registry reports invalid module failures")`), and `implements_relation::present_invalid_syntax_values_never_become_an_unguarded_form` (implements_relation.rs:777). The PR updated only the one lib test it touched. | src/registry.rs:142,178; tests/it/obligations.rs:151; tests/it/coverage_rollup.rs:3320; tests/it/implements_relation.rs:777 | correct-requirement-no-evidence |
| FND-002 | high | [soundness] PLAT-323 is fixed only for explicitly named modules. `load_from`, `from_env` and `from_default` still go through `finish_tolerant` alone. On those paths a module in a search root whose manifest fails to parse (including an unknown nested key) is dropped silently. The failure shows up only on the opt-in `failures()` surface, with no diagnostic and no version. That is the ticket's own scenario: a version-skewed module in the shared `~/.ix/filament/modules`. quire-cli's scoped `validate` loads through `load_from` and reports `failures()` only when zero modules load (validate.rs:481-484). So one skewed module among several still degrades the model with no error. | src/registry.rs:190-200; src/registry.rs:271-274 | wrong-requirement |
| FND-003 | high | [soundness] An unknown top-level manifest key is still silently ignored. `Manifest` (src/loader/manifest.rs:295) has no `deny_unknown_fields`, so a future or misspelled top-level key loads as if it were absent, with no failure, diagnostic or record of any kind. PLAT-323 requires that 'module load hard-fails with an error NAMING the unknown key and the module version' for the next unknown key. Its 2026-08-21 reproduction (Variant B) names the top-level case as a separate silence that the fix must decide. The PR's test covers only an unknown key nested under `traceability:`, which already failed at parse on main. | src/loader/manifest.rs:295 | wrong-requirement |
| FND-004 | medium | [code-bug] The strict constructors are now laxer than the tolerant ones. `load_module_strict` and `load_module_set_strict` do not call `fail_on_unreadable_manifest`, and `flatten_into_registry_strict` escalates only collision and vocabulary diagnostics. A named module with an unparseable manifest therefore makes `load_module` fail but makes `load_module_strict` return `Ok` with an empty registry. The check belongs where every constructor goes through it: in `flatten_into_registry` or the loader, not in two constructors. | src/registry.rs:148-151,186-189; src/loader/mod.rs:1427 | correct-requirement-no-evidence |
| FND-005 | medium | [code-bug] `module_path_env` joins the two variables with a literal `:` and then splits them with `std::env::split_paths`, which splits on `;` on Windows. With both variables set on Windows (`C:\a` and `C:\b`), the result is one bogus entry `C:\a:C:\b`, reported as Missing, so both roots are lost. That is worse than main, and the switch to `split_paths` was made for Windows. Fix: have `module_path_env` return `Vec<PathBuf>` (split each variable, then chain), or use `std::env::join_paths`. | src/loader/paths.rs:155-165 | correct-requirement-no-evidence |
| FND-006 | medium | [soundness] The code now contradicts the text of FR-013, which the PR does not amend. FR-013's Search-path item 2 and its 2026-06-15 CR note say `IX_SCHEMA_PATH` is 'read only when the new var is unset'. Path-handling says entries are 'taken literally after splitting on `:`'. The module doc in paths.rs:3-8 still gives the old priority order. Only the 2026-09-06 CR ('additive at every level … the default remains reachable') agrees with the new code. FR-013 must be updated to the union-plus-default rule, and the stale module doc corrected. | spec/functional/FR-013-archetype-loader.md (Search path; Path-handling edge cases); src/loader/paths.rs:3-8 | wrong-requirement |
| FND-007 | low | [other] `fail_on_unreadable_manifest` reads and parses `manifest.yaml` a second time, as an untyped `Value`, to recover `version`. It also reports only the first such failure in a module set. This creates a second source of truth for the version, and later failures in the set stay hidden behind the first. The raw `version` could be captured when the loader first reads the manifest and carried on the failure. | src/registry.rs:262-266 | correct-requirement-no-evidence |

## Verdict

Not mergeable. FND-001 leaves the integration suite red. FND-002 and FND-003
leave PLAT-323's acceptance unmet: the shared install root and unknown
top-level keys still degrade silently. PLAT-106's union and default-root
order matches quire-cli's `scoped_registry_roots` on Linux, which answers the
parity question. It still needs FND-005 (Windows join) and FND-006 (the
FR-013 text) fixed.

Minimal design for FND-002/FND-003, to close PLAT-323:

1. Add `#[serde(deny_unknown_fields)]` on `Manifest`, and declare as opaque
   (`Option<serde_yaml::Value>`) fields the top-level keys owned by other
   consumers. A sweep of 510 manifests under ~/dev found exactly four such
   keys: `manifest_version`, `grammars`, `doc_kinds`, `nav`. The FCD semantic
   `module-manifest.schema.json` (additionalProperties: false) adds two more,
   `depends_on` and `state_schemas`. serde's error then names the key and the
   expected set.
2. Make a manifest-level parse failure fatal in one place, the
   loader/flatten step that every filesystem constructor goes through
   (search roots, named, strict, inline). The error names module, version and
   key. Carry `version` from the first read.
3. Update the three FND-001 tests and FR-013 (plus any AC that states the
   tolerant behaviour) in the same PR.

## Dispositions

Round 1, at head 683b0fa. The reviewer checked the fix diff `126f813..683b0fa`.
On that head, `cargo test --test it` passes 520/520, `cargo test --lib` passes
709/709, `cargo clippy --workspace --all-targets -D warnings` is clean, and
`cargo fmt --check` is clean. The reviewer also loaded every one of the 1316
`manifest.yaml` directories under ~/dev and ~/.ix through `load_module` on this
head. No installed or real module fails. The 85 failures are deliberately
malformed test fixtures (`catalog/bad-key`, `catalog/empty-field`,
`traceability/obligations-both`, `traceability/obligations-neither`, across
worktree copies) and unrendered cookiecutter templates. All of them failed
before this change too. No real manifest carries a top-level key outside the
six opaque ones.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The three tests now assert the error (`expect_err`, message contains the cause). `plain_language` was updated too. `tests/it` passes 520/520. |
| FND-002 | fixed | `finish_tolerant` now calls `refuse_unreadable_manifests` for every constructor (`load_from`, `from_env`, `from_default`, `load_module`, `load_module_set`, inline). `walk_search_root` records `manifest_errors`. The test covers `load_from` over a root with one skewed module among good ones. |
| FND-003 | fixed | `#[serde(deny_unknown_fields)]` is now on `Manifest`, with `manifest_version`, `grammars`, `doc_kinds`, `nav`, `depends_on` and `state_schemas` as opaque `Option<serde_yaml::Value>`. `a_top_future_key` fails, naming key and version. A scan of 1316 manifests finds no other real top-level key. |
| FND-004 | fixed | `finish_strict` calls `refuse_unreadable_manifests` first. The test covers `load_module_strict`, `load_module_set_strict` and `load_strict`. |
| FND-005 | fixed | `module_path_env` splits each variable with `split_paths` and rejoins with `std::env::join_paths`. The multi-root test uses `join_paths` on both sides. |
| FND-006 | fixed | FR-013 has a 2026-11-08 CR note, Search-path items 2 and 3 are rewritten, the Path-handling text names the platform separator, the 06-15 note is marked superseded, and AC-18 and AC-19 are added. The paths.rs module doc is updated. |
| FND-007 | fixed | `load_manifest_with_origins` returns `ManifestReadError { reason, version }`, with `declared_version` taken from the same bytes. `refuse_unreadable_manifests` joins every `manifest_errors` message. The test asserts both keys from a two-module set. |
