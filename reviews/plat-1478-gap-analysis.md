---
id: SR-20261112155216
title: "gap-analysis of located engine findings against FR-033/FR-049/FR-069 (quire-rs PR 655)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@1f5017d5; spec/functional/FR-033-locator-assert-facet.md; spec/functional/FR-049-verification-reference-integrity.md; spec/functional/FR-069-semantic-module-contract-at-load.md; spec/functional/FR-011-body-extraction-dsl.md; spec/functional/FR-081-record-sections-extraction.md; spec/tests.md; tests/it/located_findings.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-033
    type: references
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
---

# Gap analysis of located engine findings (quire-rs PR 655)

## Summary

Ticket: PLAT-1478. This is a planless gap analysis scoped to the PR diff, with the FRs read in full on origin/main. Plan completion: not assessed. Examined: FR-033-AC-18, FR-049-AC-19..22, FR-069-AC-21..28, and the TC-2300..2312 rows of `spec/tests.md`.

Fifteen hand mutants were run against `tests/it/located_findings.rs`. Ten were killed:

- the element index ignored
- the line-1 fallback changed
- one-defect suppression removed
- the placeholder drop removed from the lines
- the trace line set to `None`
- the bridged error line dropped
- the `section_body` line shifted
- the object-archetype record check removed
- `key_line` returning the first key of the path
- flow continuation lines disabled

Three survived and are reported below:

- the bridged *warning* line dropped
- `line` removed from the trace sort/dedup key
- `Concerns.absent` flipped (an equivalent mutant, in SR-20261112155215)

The fourth survivor is the joined-sections `Scope::body_line` part lookup. A probe showed it equivalent on contiguous sections, and it is correct on nested sections. One mutant failed to apply.

The AC-27 fixture's `yield_pattern` is byte-identical in its keys and locators to the real Anomaly entry in spec-artifacts-process origin/main `spec_artifacts_process/manifest.yaml`, plus `closed_by`. A valid closed anomaly passes (`is_valid`), and both defect classes report at the `status:` line. The `spec/tests.md` statuses flip to ✅ only for TC-2300..2312 and the three FR rows, and every listed TC has a passing traced test.

The coder's interpretations were judged as follows:

- **Nested `required` located at the property line.** Matches FR-069 ("a value with no line of its own takes the line of its property's value").
- **Refusal at `<section>[<type>].record_validation`.** Satisfies "at its manifest path".
- **Top-level-only reads for `list_item` and `code_block` with no `under_section`.** This is existing FR-011 behaviour, and the probe confirmed it is preserved.
- **Empty record validated as `{}`.** Recorded below.
- **`ValidationReason::Semantic` plus a code prefix.** Recorded below.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-033-AC-18's last clause is unmet: "The line of an assert failure and the line exposed for the same value are the same number." Value-scoped asserts still take their line from the section, or none at all. A probe of `choices` on a `frontmatter_field` `status: bad` gave an assert line of `None` against an exposed line of 3. On a `list_item` item `nope` it gave 6 (the section heading) against 9. On a `heading` level 3 it gave `None` against 11. Scalar `id_pattern` is always `None`. TC-2300's parity test checks only `section_body` and `table_row` `column_choices`, the two kinds where the numbers already coincide. Fix: give value-scoped failures (`choices`, scalar `id_pattern`, `matches`) the line of the value from `eval_located`, and extend the parity test to frontmatter_field, list_item and heading. Or amend AC-18. | src/extract/assert_eval.rs:529-553,594-624; tests/it/located_findings.rs:209-230 | implementation-bug-despite-evidence |
| FND-002 | medium | FR-049-AC-19 says "the same id dangling on two rows of one document yields two findings ... (one finding before this change)". This clause is untested. The `kinds` fixture declares `row_id_column: Link`, so the rows' messages already differed and gave separate findings before the change. A mutant that drops `line` from the sort/dedup key passes every test. Fix: add a dangling-twice case under a declaration without `row_id_column` (for example the `refs-alt` module of TC-2303) and assert two findings at the two rows' lines. | tests/it/located_findings.rs:286-324,340-390; src/corpus/trace_refs.rs:176-177 | correct-requirement-no-evidence |
| FND-003 | medium | The warning half of FR-049-AC-22 ("a document-level error or warning bridged into the bundle report keeps its line") is untested. TC-2312 checks only `report.errors`, and a mutant setting `.at_line(None)` on the bridged warnings passes. Fix: add a document whose registry result carries a located warning (for example an unknown `object:` or a disallowed edge type) and assert that its bridged warning has the same line. | src/corpus/validate.rs:569-577; tests/it/located_findings.rs:979-1049 | correct-requirement-no-evidence |
| FND-004 | low | When every `match` locator is optional and none located anything, FR-011 yields no record. The code then validates `{}`, so absences are still reported. FR-069 does not say what happens here, and no test reaches `unwrap_or_default()`. The interpretation is sensible. Fix: add one sentence to FR-069 ("no located value validates the empty record") and one test. | src/record_check.rs:47-50 | missing-requirement |
| FND-005 | low | FR-069 says each error is "one finding with reason `semantic.record-invalid`". The code emits `ValidationReason::Semantic` with the code as the message prefix, so the bundle reason token is `semantic`. This matches the existing declaration-record `semantic.record-invalid` finding and the FR-072 convention, so it is not a code defect. The spec wording reads as a distinct reason value. Fix: reword FR-069 to "reason `semantic`, message code `semantic.record-invalid`" as a spec follow-up. | spec/functional/FR-069-semantic-module-contract-at-load.md:299-301; src/record_check.rs:78-90 | wrong-requirement |

## Verdict

Not mergeable until FND-001 is resolved, either by a code fix with an extended test or by an AC-18 amendment. FND-002 and FND-003 are missing tests that belong in this PR. FND-004 and FND-005 are spec follow-ups. Everything else traced cleanly. The record comes from the unmodified `extract::extract`, the engine holds no archetype or field names, the load refusals cover the AC-22 list (including iterate_over-only and no-semantic modules), the type-only `validate_document` runs no record check, and a module without `record_validation` is unchanged.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (f9de3f2, verified at e662590): `AssertFailure.doc_line` is set from `eval_located` for `choices`, `matches` and scalar `id_pattern`, and `validate_required_locator` prefers it. The parity test now covers frontmatter_field `choices` and `id_pattern`, section_body `choices` and `matches`, list_item, heading and table_row. Mutants that set `doc_line: None` (on `choices` or `id_pattern`) or prefer `line` over `doc_line` fail. A probe puts an Anomaly-style `status: bogus` assert at the status key's line 4, and per_match list_item failures at their item lines (10 and 14). The full `it` suite (711) and lib suite (740) pass unchanged, so section-scoped lines are unchanged. |
| FND-002 | fixed | Round 1: TC-2303's `refs-alt` memo (no `row_id_column`) cites `G7` on two rows and expects two findings at the two row lines. The mutant dropping `line` from the sort/dedup key fails. |
| FND-003 | fixed | Round 1: `tc2312_a_bridged_warning_keeps_its_line` bridges a grammar `adequate` warning and compares its lines with `validate_document_in_registry`'s. The mutant `.at_line(None)` on warnings fails. |
| FND-004 | fixed | Round 1: FR-069 "When it runs" now states that an all-optional record with nothing located validates the empty record, with the absence at line 1. `tc2307_no_located_value_validates_the_empty_record` checks it, and the mutant returning no findings for a missing record fails. |
| FND-005 | fixed | Round 1: FR-069 "Every error" now says the reason is `semantic` and the message begins with the code `semantic.record-invalid`, and that a bundle reports the same reason token. No stale "reason `semantic.record-invalid`" text remains, and spec_validate reports 0 failed. |
