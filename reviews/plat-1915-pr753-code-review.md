---
id: SR-967175026302197768900474181941935084898
title: "code-review of quire-rs PR 753"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PLAT-1915; PR 753; tests/it/corpus_cases.rs; FR-065"
review_set: subset
---

## Summary

Reviewed ab927d82f78f9b29d4c7343086889dc19c2fdf6d. Thirteen requested controlled-corpus bindings were inspected.

## Verdict

CONDITIONAL — two coverage claims need correction before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-2 is claimed without executing a mutating case: the snapshot surrounds metadata-loader rejection checks only. A runner that mutates checked-in input instead of a copy would still pass these checks. Exercise a real mutating case on its copied input and assert the checked-in input is byte-unchanged, or leave AC-2 open. | tests/it/corpus_cases.rs:1202-1207,1519-1524 |
| FND-002 | medium | AC-14 is claimed by differential grading rather than control silence. A control emitting the partner diagnostic at a different path, line or message can still fail the partner expectation and satisfy this test. Assert absence of the partner finding on actual healthy output for the applicable finding channels, or leave AC-14 open. | tests/it/corpus_cases.rs:2136,2330-2405 |

## Assurance Context

Read AP-201 detection/minting. This test-only change does not alter engine collection/minting, but its warning about shared bounds oracles applies: calls into qa_corpus test that producer, not an independent second derivation. No new measurement or independence claim is made. Selected external evidence unavailable: separate retained coder stdout; the PR records the focused result.

## Coverage

Reviewed the complete one-file diff and all 13 requested criterion bindings, plus the loader, grading and execution helpers and the pinned corpus. AC-4 rejects missing control_for with identity; AC-5 checks omitted backed is unconstrained against a real nonzero payload; AC-7 rejects whitespace reasons with cell identity; AC-8 rejects a derived state absent from the declaration; AC-16 loads a real ecosystem case without variant ticket metadata; AC-24 rejects a partner in another language. AC-6 checks enum states and complete cell counts; AC-9/10/CON-2 inspect the canonical derived bounds count as an integer; CON-1 reads actual checked-in case input directories. The AC-2 and AC-14 bindings are relevant but incomplete, as detailed in code-review findings; they are not counted as fully verified bindings here.

No production functions or dependency pins changed, so no new implements markers or dependency comparison are required. Branch fix/controlled-corpus-contract-traces and title test: cover controlled corpus contract are neutral; body contains Refs PLAT-1915. The code lead explicitly assigned that ticket. No workflow changes, ignored tests, mocks replacing the operation or new production stubs occur. New helpers are test-only; searched the corpus helper modules for existing tree/snapshot helpers and reused the existing tree copier. The before/after byte comparison is a legitimate functional oracle, but the operation it surrounds is the AC-2 problem.

## Examined units and bindings

```yaml
scope:
- id: FR-065-AC-2
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: A mutating case operates on a copy, leaving its checked-in `input/` tree
    byte-unchanged.
- id: FR-065-AC-4
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: A control case declaring no `control_for` is rejected.
- id: FR-065-AC-5
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: An `expect` field a case omits is asserted on by nothing; the omitted field
    is not defaulted.
- id: FR-065-AC-6
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: Every declared cell in `corpus.yaml` reads as exactly one of `covered`,
    `out-of-scope` or `GAP`.
- id: FR-065-AC-7
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: An `out-of-scope` cell carries a non-empty reason; one with an empty reason
    is rejected.
- id: FR-065-AC-8
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: A cell in none of the three states is rejected, naming the case and the
    language.
- id: FR-065-AC-9
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: '`bounds.gap_count` is an integer count on every surface this crate emits.'
- id: FR-065-AC-10
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: No payload this crate emits renders `bounds.gap_count` as a ratio or a
    percentage.
- id: FR-065-AC-14
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: A control case over healthy input produces no finding for the mode its
    partner asserts.
- id: FR-065-AC-16
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: A case binding the ecosystem declaration loads without naming a ticket.
- id: FR-065-AC-24
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: '`control_for` resolves against failure cases only, in the control''s own
    language; a control whose partner is absent is rejected.'
- id: FR-065-CON-1
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: 'Case data SHALL NOT be embedded in runner code or generated at runtime.
    A case that cannot be read without executing something is not data. The one exception
    is stated in the Behavior section: a mutating case operates on a copy and never
    writes the checked-in tree.'
- id: FR-065-CON-2
  path: spec/functional/FR-065-controlled-corpus-contract.md
  role: examined
  excerpt: Every surface reporting `bounds.gap_count` SHALL render it as an absolute
    count, never normalised into a ratio or a percentage. This is FR-063-AC-6 applied
    to this metric, not a second rule.
bindings:
- test_id: a_control_without_control_for_is_rejected
  ac_id: FR-065-AC-4
  trace: correct
- test_id: an_omitted_expectation_field_is_not_defaulted
  ac_id: FR-065-AC-5
  trace: correct
- test_id: tc1021_the_vocabularies_come_from_the_corpus_not_from_this_file
  ac_id: FR-065-AC-6
  trace: correct
- test_id: an_out_of_scope_cell_with_an_empty_reason_is_rejected
  ac_id: FR-065-AC-7
  trace: correct
- test_id: a_cell_state_missing_from_the_declared_bounds_states_is_rejected
  ac_id: FR-065-AC-8
  trace: correct
- test_id: tc1021_the_vocabularies_come_from_the_corpus_not_from_this_file
  ac_id: FR-065-AC-9
  trace: correct
- test_id: tc1021_the_vocabularies_come_from_the_corpus_not_from_this_file
  ac_id: FR-065-AC-10
  trace: correct
- test_id: an_ecosystem_bound_case_loads_without_a_variant_ticket
  ac_id: FR-065-AC-16
  trace: correct
- test_id: a_control_partner_must_be_a_failure_in_the_same_language
  ac_id: FR-065-AC-24
  trace: correct
- test_id: corpus_cases_hold
  ac_id: FR-065-CON-1
  trace: correct
- test_id: tc1021_the_vocabularies_come_from_the_corpus_not_from_this_file
  ac_id: FR-065-CON-2
  trace: correct
```

## Checks

Independent cargo fmt --all -- --check and git diff --check passed. Current-head coder evidence reports cargo test --locked --test it corpus_cases:: through locked-build: 27 passed, zero failed/ignored, 984 filtered, 415.64s. The reviewer attempted a compiled test artifact listing through locked-build, remained queued behind older waiters and cancelled before launch (exit 130); no independent test pass or rebuild is claimed. Optional spec-objects-architecture@0.10.3 fetch warning concerns two unrelated runtime-skipped tests. Static trace claims do not prove these particular behavioral assertions. CI analysis jobs remain running as reported by the lead.

The pinned corpus was initialized at bfe97c7cddd4060cf50ba107d29c0e5f123f2d28 for this review; no tracked changes were made. Quoin 0.37.0 is the review tool release; spec-artifacts-process is v0.36.0. Existing trace warnings are retained in /tmp/review753-matrix.stderr.

## Artifact retention

Commit as reviews/plat-1915-pr753-code-review.md after the finding fixes and dispositions. Preserve original finding text. Reviewer makes no branch edits.

## Dispositions

Round 1 reviewed 89b1161f28879bce8e32acaeaf8f2f75d19e3f3d. Mergeable: no.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Removed unsupported AC-2 tag and snapshot helper/assertions. PR explicitly leaves AC-2 open because this runner is read-only and no mutating case is present. |
| FND-002 | still-open | The new helper compares finding.reason to the entire expected string. Six existing variants use declaration-qualified test-case/section-holds-no-table or test-case/archetype-matches-nothing; the real reason omits test-case/ and declaration is a separate field. Those checks always report absence even when the matching diagnostic is present. Reuse find_diagnostic declaration/reason semantics from corpus_case/grading.rs. |

AC-2 after excerpt (PR body):

> FR-065-AC-2 remains open. This repository's `corpus_case::run` only reads an input tree for coverage, and the initialized pinned corpus has no mutating case or command (such as `quire fix`).

The new helper does directly check unqualified diagnostic reasons, suspicion kinds and validation tokens on healthy output; the declaration-qualified diagnostic branch is the remaining gap. The original findings above remain unchanged. Current body accurately claims 12 closures and the older cohort 149 to 137 (Test145 to133; Unit Test4 unchanged). No new production files or pins changed.

Round-1 verification: independent current compiled test artifact tc1028 exact filter passed 1/1 through locked-build (47.04s; 1,010 filtered). This confirms the focused test remains green despite the inspected qualified-reason hole; no fresh reviewer rebuild is claimed. Independent formatter and diff checks passed. Fresh matrix has 140 untagged exact-Test IDs, with AC-2 untagged and AC-14 tagged; the supplied older cohort is 137 including four Unit Test gaps.

## Dispositions (round 2)

Reviewed 914a0bf00f19204f683ddeabd0b87b25be5643c5. Review mergeable: yes.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Actual healthy output is checked through the existing declaration/reason matcher, including keys from diagnostic locus/message maps. A new traced regression constructs the qualified diagnostic and verifies the silence checker detects it. |

After excerpt:

```rust
for key in diagnostic_keys {
        if corpus_case::find_diagnostic(healthy, key).is_some() {
            present.push(format!("diagnostic `{key}`"));
        }
    }
```

FND-001 remains resolved from round 1: AC-2 is explicitly open and no unsupported trace tag remains. No new findings. The crate-visible export is test-harness-only and reuses the existing matcher rather than defining another matching rule. No production API, pin or workflow change. PR title and branch remain neutral, with Refs PLAT-1915 in the body. Source changes are in three test-harness files.

Coder current-head focused evidence: qualified diagnostic regression 1/1; TC-1028 1/1 (44.80s); complete corpus filter 28/28 (391.90s), zero failed/ignored. Optional architecture fixture warning remains non-failing and unrelated. Repository assurance remains FAIL for remaining gaps; this is not a blocker to the accurately scoped 12-criterion closure PR.

Round-2 independent verification: formatter/diff checks pass. Fresh matrix confirms AC-2 untagged, AC-14 bound to TC-1028 and the new regression; 140 exact-Test IDs remain untagged repository-wide. Reviewer compiled-artifact regression attempt stayed queued behind older waiters and was cancelled before launch (exit130); no independent round-2 test pass is claimed. Completed coder current-head runs above supply execution evidence.
