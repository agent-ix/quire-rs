---
id: SR-191
title: "spec-review of the FR-050 require_clean input rule (quire-rs PR 545)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@08b9b605; spec/functional/FR-050-declarative-coverage-computation.md; bench/manifest.json; tools/quire-bench/tests/benchmark.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-050
    type: references
---

# Spec review of the FR-050 require_clean input rule (quire-rs PR 545)

## Summary

Ticket: STD-362 (spec gap behind QSL-942). One spec-review pass over
`git diff origin/main...08b9b605` (3 files, +7/-6). origin/main is an ancestor
of the head. Oracles: `tools/quire-bench/src/collection.rs`
(`require_clean`, `resolve_default_module`, `collect`) and
`tools/quire-bench/src/export.rs` (`validate_repository_against_stack`, the
governed export) at the head.

Gates run by the reviewer: `quire validate` on FR-050 reports no failure.

Examined, clean:

- `make bench` collection matches AC-52's main path. It runs
  `git status --porcelain=v1 --untracked-files=all` at the entry path, uses
  the exact dirty and absent messages, writes `skip <name>: …` on a normal
  run, fails a strict run naming the entry, and fails the whole run on a dirty
  `module_source` when an entry reads the default module.
- The trace tag `#[trace("FR-050-AC-30", "FR-050-AC-52")]` on
  `declared_clean_corpus_refuses_uncommitted_input` is correct for the
  clauses it asserts: clean is accepted, an untracked file is refused.
- FR-050-AC-52 is unused on main and in every open quire-rs PR.
- The lead default is labelled in the PR body.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | [scope-deferral] The PR removes a capability that the spec stated. AC-30's "A tier-2 entry declares a pinned SHA and a run against a different tree is refused" goes, and AC-52 now says "at whatever commit the checkout holds: no commit, SHA or revision is declared or compared". After this change a `make bench` ratchet can compare baselines taken on different corpus revisions, and no rule refuses that. The standing rule of 2026-10-22 makes any drop or narrowing a blocking HIGH finding, even when it is flagged as a lead default. It conflicts with Peter's guidance that pins belong only in package managers. This is an owner question that the lead cannot answer by default: drop the clause, or specify how a run against a different tree is refused (the governed export already does this, see FND-002). | spec/functional/FR-050-declarative-coverage-computation.md:236-237 | wrong-requirement |
| FND-002 | high | [soundness] The code breaks AC-52's statement that "no commit, SHA or revision is declared or compared". AC-30 cites `measurement_export.rs` as its verification. The governed measurement export (`export.rs`) calls `validate_repository_against_stack` for every entry with `require_clean` and for the `module_source`. That function compares the checkout's HEAD with the verification stack's attested revision. It refuses `<source> checkout does not equal the attested revision`, and a dirty tree with its own message, `<source> checkout is dirty`, not AC-52's. So revisions are declared (in the stack) and compared on that path. Fix: scope AC-52 to the `make bench` collection, and state the export's rule (attested revision plus clean tree, with its messages) in its own clause or AC. | spec/functional/FR-050-declarative-coverage-computation.md:237; tools/quire-bench/src/export.rs:297-335, 585-611 | wrong-requirement |
| FND-003 | medium | [coverage] Most of AC-52 is untested. Its only tagged test creates an empty repository, accepts it, adds one untracked file and expects "dirty". Nothing tests these clauses: a modified tracked file; the absent-path message; a path that is not a Git checkout; normal-run `skip` against strict-run failure; a `module_source` without `require_clean` being refused; and a dirty `module_source` refusing the whole run. The PR body says the existing test "covers the refusal and the clean case", but the AC states six more behaviours. Fix: add tests over `collect` and `resolve_default_module` for each of them, tagged FR-050-AC-52. | spec/functional/FR-050-declarative-coverage-computation.md:237; tools/quire-bench/tests/benchmark.rs:185-200 | correct-requirement-no-evidence |
| FND-004 | low | [soundness] Two of AC-52's statements are stronger than the code. (a) "The `module_source` must declare `require_clean: true`, a declaration without it is refused": `collect` checks this only when some entry has no `module` of its own, through `resolve_default_module`. (b) "A path that is not a Git checkout is refused": `git -C <path> status` in a directory nested inside another checkout reports the enclosing checkout, so that directory is not refused, and the enclosing tree's cleanliness decides. Fix: state the condition in (a), and say that the enclosing checkout decides in (b), or make the code refuse it. | spec/functional/FR-050-declarative-coverage-computation.md:237; tools/quire-bench/src/collection.rs:41-71, 164-172 | wrong-requirement |
| FND-005 | low | [wording] The manifest `$comment` ("each held fixed by a clean checkout") and the STD-228 note ("A corpus input is held fixed by a clean checkout, not by a recorded revision") say that a clean checkout holds the input fixed. It does not: HEAD can move between runs. The note also uses the "X, not Y" form. Fix: say "scored only from a clean checkout". | bench/manifest.json:2; spec/functional/FR-050-declarative-coverage-computation.md:420-422 | wrong-requirement |

## Verdict

Not mergeable. FND-001 is a scope-deferral under the standing rule and needs
an owner ruling on the pinned-SHA clause. FND-002 is a statement in AC-52
that the governed export violates. Both block. FND-003 leaves most of AC-52
without evidence.

## Dispositions

Round 1, at head 9a2df0d5 (fix diff checked against the original review head).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Narrowed, not closed. The revision check now lives in the governed export (`make measurement-collection`), which refuses a checkout at another commit than its attestation records, so the capability exists where measurements become evidence. But the ratchet run that AC-30 describes (`make bench`, and `make bench-update`, which rewrites the checked-in baselines) still accepts any revision of a corpus. That part of the dropped clause, 'a run against a different tree is refused', is gone from the ratchet. Under the standing rule this stays a HIGH scope-deferral. It closes when Peter rules the ratchet need not refuse, or when `bench` and `bench-update` check the same attestation when one is given. |
| FND-002 | fixed | fixed; after: **Governed export** (`make measurement-collection`, `quire-measurements`): for each corpus entry that declares `require_clean: true`, under its `source_name` when it has one, and for the `module_source` when it declares it, the export reads the source's attested revision and remote from the verifica… |
| FND-003 | fixed | fixed; after: tools/quire-bench/tests/clean_checkout.rs: modified_tracked_file_refuses_and_clean_checkout_is_accepted, absent_path_and_non_checkout_refuse, nested_directory_follows_its_enclosing_checkout, normal_run_skips_a_dirty_entry_and_strict_run_fails_naming_it, module_source_rules_apply_when_an_entry_reads_… |
| FND-004 | fixed | fixed; after: **Collection** (`make bench`, `quire-bench`): a corpus entry that declares `require_clean: true` is scored only when `git status --porcelain=v1 --untracked-files=all`, run at its declared `path`, reports nothing, at whatever commit that checkout holds. The status is the checkout's whose working tree… |
| FND-005 | fixed | fixed; after: > A corpus input declaring `require_clean` is scored only from a clean > checkout (FR-050-AC-52). The only revision a run compares is the one in the > governed export's verification-stack attestation; the tier says how an entry > is used. |

Round 2 (author).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The ratchet run now checks the same attestation as the governed export. `quire-bench` needs `--verification-stack` to collect, and `make bench` and `make bench-update` refuse without `VERIFICATION_STACK=`. `collection::collect_attested` refuses each `require_clean` entry that is off its attested revision or remote, using the export's messages: a normal run skips it loudly and a strict run fails. A `require_clean` `module_source` off its revision refuses the run. The export and the ratchet share `validate_repository_against_stack`, and the export's corpus loop moved to `export::validate_corpus_sources`. AC-30 and AC-52 state the rule for both runs. New test `ratchet_run_refuses_a_corpus_off_its_attested_revision` covers it. |
