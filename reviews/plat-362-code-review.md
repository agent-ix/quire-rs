---
id: SR-16434
title: "Code review of agent-ix/quire-rs PR 624 (PLAT-362 part 2): shell source-symbol adapter"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/symbols/shell.rs, src/symbols/shell_trace.rs, src/symbols/trace.rs, src/symbols/mod.rs, src/traceability.rs, src/coverage/*, src/skeptic.rs, tests/it/shell_symbols.rs, Cargo.toml, Makefile, schemas/output/assurance-v1.schema.json, tests/fixtures/coverage_baseline/expected.json (PR 624 diff only, head 1b3bafd)"
review_set: subset
---

## Summary

Ticket: PLAT-362. PR 624, head `1b3bafd`, scoped to `git diff origin/main...HEAD` (26 files, +1315/-40). This covers the code-review method with its Rust lane (`/rust-review`, quire-rs CLAUDE.md conventions).

Evidence gathered by the reviewer at the head:

- `cargo test --test it -- shell_symbols::` passes 13 of 13 (TC-2101..TC-2113).
- `cargo test --test it -- coverage_baseline:: corpus_recall:: corpus_cases:: output_contract:: implements_relation::` passes 51 of 51. That run includes the qa-corpus cases `outside-entry-paths`, `function-not-evidence` and `not-in-tag-channel`, each with its control.
- `cargo test --lib -- symbols:: skeptic:: traceability::` passes 170 of 170.
- `quoin matrix --json` reports FR-051-AC-34..46 each `tagged` by its TC-2101..TC-2113 test.

Examined against FR-051-AC-34..46, AC-19 and AC-28: the tree-sitter walk, the comment-node tag channel, `block_above`, `header_comments`, `ShellBinder` (binding, `entry_paths` union, unbound reporting), the `TraceMarkerForm` deserializer and `validate` changes, the coverage reason registration and guidance, the skeptic exclusions, and the feature gating (`shell-symbols`, `check-shell-symbols`).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A shell `markers` form that omits `syntax: comment` still binds. FR-051-AC-42 says shell tags are read only through a `markers` form with `language: shell` and `syntax: comment`. `ShellBinder::new` filters forms on `form.language == SourceLanguage::Shell` alone, and its module doc says shell forms read comment nodes "whatever `syntax` they declare". `validate` refuses only `rust_attribute` on a markers form, so `{language: shell, pattern: ...}` with no `syntax` loads and mints `verifies` on an eligible `entry_point`. TC-2109 tests only "no form" and "a Rust form", so this path has no test. Fix: either filter markers on `form.syntax == Some(TraceMarkerSyntax::Comment)`, or refuse a `language: shell` markers form without `syntax: comment` at load, naming the key. Then add that case to TC-2109. | src/symbols/shell_trace.rs:66-81; src/traceability.rs:1772-1775 |
| FND-002 | medium | Shell entry points are left out of the binding census, and the spec gives no carve-out for that. The `bind` loop takes the shell branch and runs `continue` before any census accumulation. So `binding_census` has no `shell` row, and shell `verifies` never count toward `bound`. FR-051-AC-19 asks for a per-language census of the evidence symbols binding examined. `entry_point` is an evidence kind (`binds_trace_ids()` is true). AC-44's "no census count moves" covers only the reported non-binding marker. Keeping shell out is defensible, because ineligible entry points would otherwise show up as unbound candidates in every repository that has scripts. But the spec has to say so. Fix: add a CR note amending AC-19 (shell entry points are not census candidates, and AC-44 is their channel), or count shell entry points in the census. | src/symbols/trace.rs:641-645; spec/functional/FR-051-source-symbol-extraction.md:215 |
| FND-003 | medium | A range in a shell marker that does not bind is dropped with no record at all. `report_unbound` runs `continue` on any range-shaped id. `bind_entry_point` reads only eligible headers, and `bind_function` reads only `implements` forms. So `# trace: FR-001..FR-003` in an ineligible header, in a function's comment block, or in any other comment produces neither a `range-in-trace-tag` finding (FR-051-AC-28: one finding per occurrence, wherever it sits) nor an AC-44 finding. Fix: in `report_unbound`, push a `RangeInTraceTag` for a range id instead of skipping it, and add a TC-2111 row for it. | src/symbols/shell_trace.rs:217-231 |
| FND-004 | low | The diagnostic for an uncompiled `entry_paths` list stores a glob in `TraceDiagnostic.path`, with empty `symbol` and `trace_id`. It locates the pattern by probing `globset::Glob::new` per pattern, which can disagree with the builder `ExcludeSet::compile` uses and then fall back to `""`. `graph.diagnostics` has no consumer outside tests: `SymbolGraph` is not serialized and is not merged into `CoverageReport`. So only a library caller that skipped `validate` ever sees it. That satisfies AC-39's wording, and module load already refuses the pattern. Fix (optional): take the failing pattern from `ExcludeSet::compile`'s own error, not from a second probe. | src/symbols/shell_trace.rs:92-108 |
| FND-005 | low | The module doc line 14 of `shell.rs` is about 170 columns. rustfmt does not wrap comments, and the repo style is 100 columns. Rewrap it. | src/symbols/shell.rs:14 |

## Verdict

Mergeable once FND-001 is fixed. FND-002 and FND-003 can land in the same fix round: each is small, and FND-002 is one CR note if the census exclusion stays. There is no high finding.

On the lead's points:

1. **`$( )` and backtick functions mint nothing.** This matches FR-051-AC-36, which lists a definition-shaped line inside a backtick or `$( )` string as a decoy that mints no function. It is also semantically right, because such a function lives in a subshell and is not global. The FR-051 narrative ("every `function_definition` node the grammar yields") reads the other way, and that is recorded as a spec low in SR-16435. `in_command_substitution` walks every ancestor, so a function nested in a function inside `$( )` is also skipped, and a `"$(f() …)"` inside a double-quoted string is covered by the same ancestor test.
2. **`entry-paths-not-applied` removed.** The spec and the corpus agree. AC-39 asks only that an extraction handed uncompiled globs "applies none and reports a diagnostic naming the pattern"; it names no `CoverageReport` reason. Module load refuses the pattern, so the CLI path cannot reach it. qa-corpus `3e0686c` drops the token from `emitted`, and `COVERAGE_DIAGNOSTIC_REASONS` and `expected.json` do not list it. The `graph.diagnostics` entry (asserted by TC-2106) is the library-facing report (FND-004 is a low on its shape only).
3. **Shell left out of the binding census.** This is a reasonable design, but it is not what AC-19 says as written (FND-002).
4. **Corpus submodule bump to qa-corpus `3e0686c`.** Expected, and not vendoring.
5. **Shared-file edits.** These are small and additive. In `src/symbols/trace.rs`, the PR adds one `SymbolGraph` field, a `ShellBinder::new` and `report_unbound` call pair, a shell early-`continue` in the bind loop, `Shell` arms in `legacy_match_span` and `mask_source_string_contents`, a `Some(Comment)` arm in `bind_implements`, `pub(crate)` on `range_pattern` and `marker_ids`, and `entry_paths: None` in three test literals. A PLAT-847 change that deletes the legacy machinery will touch `legacy_match_span`, the `bind_implements` candidate match and possibly those test literals; each conflict is a one-line resolution. The `expected.json` change is three lines in a sorted reason list. The shell binder keeps its own state outside the main loop, so neither PR changes the other's binding semantics.

Also checked, with no finding: the walk uses an explicit cursor (no recursion), comment classification works from nodes with CRLF trimming, and `end_line` handles a node that ends at column 0. Feature gating is complete: an `allow(unused_mut)` under no-shell, `record_unavailable` is extended, and `check-shell-symbols` is in `ci`. The skeptic scans skip shell symbols, an `unreachable!()` was replaced by `return None`, and no new `unwrap` or `expect` appears outside tests. The PR adds no pin, digest, tracking record, vendored file or compatibility layer; the `quire-code-parse` rev is the repeatable-build dependency pin the cross-repo change requires.
