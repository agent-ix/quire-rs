---
id: SR-2026111921333400000001
title: "code-review of QSL-2517 qualified trace ids in verifies markers (NFR-140)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PR #704 at f0518573; src/coverage/qualified.rs, src/coverage/reconcile.rs, src/coverage/diagnostics.rs, src/coverage.rs, src/traceability.rs, src/loader/mod.rs, tests/it/qualified_trace_ids.rs, tests/it/corpus_cases.rs, tests/fixtures/coverage_baseline/expected.json, tests/fixtures/traceability/iso-obligations-qualified/manifest.yaml, CHANGELOG.md; spec: agent-ix/filament-spec-language NFR-140 and TC-3033 at 019dadaf4"
review_set: subset
---

## Summary

Code review (rust-style and rust-review lane) and NFR-140-AC-1..3 gap check of quire-rs PR #704. Ticket: QSL-2517. The PR adds `traceability.trace_id_qualifiers`, resolves `verifies` tags written as `<prefix><id>` or `ix://<org>/<repo>/<id>` against the corpus member with that identity, and reports unresolved tags as `unresolved-qualified-trace-id`.

Evidence: `cargo test --test it qualified_trace_ids` 7/7 pass; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean. Control run: `cargo test --test it corpus_cases` fails the same 10 tests on origin/main 0c9dcc83 and on the PR head, and `loader::paths::tests::env_roots_keep_default_root` fails on both at src/loader/paths.rs:289. The author's pre-existing-failure claim holds. Three reviewer probe tests, run in a scratch clone at the PR head and not committed, confirmed FND-001, FND-002 and FND-003.

## Examined scope

- Matrix binders (NFR-140-AC-1, AC-2): `FSpec-` and `ix://` tags bind only the named member's criterion. The same bare id in FSL stays unbound. Ignored state is carried. Tested. (examined)
- Minted targets: `record.backed` ORs in the member-scoped qualified binders. Correct by reading. No test asserts it. (examined)
- Reference rows: the qualified backing uses an any-of rule that differs from the bare rule (FND-002). (examined)
- Unresolved tags: outside-corpus and not-minted causes are told apart in the message. Qualified tags are excluded from `untracked_symbols`. A document-level id resolves without binding and produces no report (FND-001). (examined)
- `trace_id_qualifiers` declaration: serde `deny_unknown_fields`, a blank or whitespace prefix is refused, a repository that is not `<org>/<repo>` is refused, and duplicate or overlapping prefixes in one module are refused with clear errors. Several prefixes may name one repository. The prefix match is case-sensitive and the repository match is ASCII-case-folded through `fold_identity`, consistent with FR-038-AC-10. The loader merges first-wins by prefix, but the merged model is not checked again for overlap, and the merge has no test (FND-004). (examined)
- Diagnostic registry: `COVERAGE_DIAGNOSTIC_REASONS`, `expected.json` and the `corpus_cases.rs` allowance each add exactly `unresolved-qualified-trace-id`. `guidance_for` has an arm for it. (examined)
- Bare-id behaviour: the existing bare paths are unchanged, because the PR only adds OR terms and the untracked exclusion. The pre-existing cross-member bare leak is FND-003. (examined)
- No Python, no shims, no pins outside Cargo. (examined)

## Verdict

Not mergeable yet. No high findings. FND-001 and FND-002 are defects in the new code and should be fixed in this PR, each with a test. FND-003 is pre-existing bare-id behaviour that the ticket keeps unchanged; it needs a follow-up ticket because NFR-140 Scope says a bare id names the FSL artifact. The declaration, the diagnostic registry change and the control-run claims are correct.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A qualified tag whose id resolves only through `spec.by_qualified_id`, such as a document-level id like `FSpec-FR-001`, counts as resolved and goes into `bound`. No matrix criterion, minted record or reference row reads that binding, so the tag binds nothing and still raises no `unresolved-qualified-trace-id` and no untracked symbol. The bare `FR-001` in the same probe is reported as untracked and `untracked-id-has-minted-children`. The qualified form silently hides what the bare form reports. This goes against NFR-140's metric "FSpec tags that bind no filament-specification artifact = 0" | src/coverage/qualified.rs:108-117 |
| FND-002 | medium | Reference-row backing for qualified tags is `answerable.iter().any(...)`. The bare rule requires the row's own obligation id when the row id is an obligation, and otherwise all referenced obligations. A probe with a TestMatrix row `TC-001` tracing to `FR-001-AC-1, FR-001-AC-2`, where only `FSpec-FR-001-AC-1` is tagged, leaves the fspec row backed. The bare twin tag `FR-001-AC-1` leaves it unbacked. The qualified form therefore turns rows green that the bare form reports, and can hide status lies | src/coverage/reconcile.rs:287-291 |
| FND-003 | medium | Pre-existing, not introduced by this PR. `binders_by_id` keys bare tags by id across all corpus members, so in the two-repository matrix a bare FSL tag `FR-001-AC-1` is listed as a binder of filament-specification `FR-001-AC-1` too (confirmed by probe). NFR-140 Scope says a bare id names the FSL artifact. Deferring to a follow-up ticket, since this ticket keeps bare-id behaviour unchanged | src/coverage/reconcile.rs:956-971 |
| FND-004 | low | Qualifier validation runs per module only. `merge_traceability` merges first-wins by prefix, so a prefix that two modules declare with different repositories is dropped silently, and prefixes that overlap across modules (e.g. `F-` and `F-X-`) survive the merge. `split_qualified` then resolves them by module order. Neither the merge nor this case has a test | src/loader/mod.rs:1733-1742 |
| FND-005 | low | Test evidence covers matrix binders only. No test asserts minted-target `backed` or reference-row backing for qualified tags (the gap that let FND-002 through), and none asserts that only the unresolved diagnostic fires over FSL alone (no other diagnostic for the same tag) | tests/it/qualified_trace_ids.rs:1-264 |
| FND-006 | low | NFR-140-AC-3 says that removing the declaration makes every `FSpec-` tag reported as "unresolved". The implementation and its test report such tags as `untracked_symbols`, not as `unresolved-qualified-trace-id`. The behaviour is reasonable, because without the declaration the engine cannot know the prefix. The AC wording and the outcome differ, so either the AC text or the test comment should say which report is meant | tests/it/qualified_trace_ids.rs:225-236 |
| FND-007 | low | Small nits: `"section-holds-no-table" =>FindingGuidance` lost its space and rustfmt skips this match, so `fmt --check` does not catch it. The new reason was added to the `FORMALIZES_REASONS` allowance, whose name and comment are about FR-094. `unresolved-qualified-trace-id` sits between `range-…` and `section-…` in the otherwise alphabetical registry | src/coverage/diagnostics.rs:125 |
