---
id: SR-20261115183662
title: "EARS conformance review of PLAT-1441 FR-069, FR-081, FR-084 amendments"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/functional/FR-081-record-sections-extraction.md, spec/functional/FR-084-two-way-relation-coverage-report.md (PR #688)"
review_set: all
---

## Summary

Ticket: PLAT-1441. The FR-069 and FR-081 additions use SHALL statements and EARS "If ...
then" unwanted-behaviour forms correctly. The FR-084 addition has no SHALL statement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-084 "Carried record values and the reverse view" section is descriptive prose ("carries", "is omitted", "The engine selects no key") with no SHALL, so the new report obligation lives only in AC-20..AC-22. | spec/functional/FR-084-two-way-relation-coverage-report.md:79-103 |

## Verdict

Changes requested (medium). Restate as, for example, "When a document entry's record is
non-empty, the engine SHALL serialize it as `record` ...". Restate it after the FR-084
selected-field redesign (base review FND-001). Also examined, with no defect found: FR-069
bullets 1–5, the FR-081 frontmatter-owned bullets, and FR-081-AC-18.

## New findings (disposition pass 1)

Pass 1 re-read the new statements in FR-058 (`record_keys` load refusal), FR-069
(frontmatter-owned record keys), FR-081 (frontmatter-owned bullets) and FR-084 (carried record
values) at head `5731e42`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The first FR-081 both-forms statement reads "Where an artifact carries both ... and a section with that key, then the engine SHALL emit the warning". In EARS, `Where` is the optional-feature keyword and takes no `then`. The trigger is a document state, so it belongs in the `If ..., then` form that the sibling section-only bullet uses. The four "Where an artifact carries both, the engine SHALL" statements after it have the same keyword issue but no `then`. | spec/functional/FR-081-record-sections-extraction.md:123-131 |

## New findings (disposition pass 2)

Pass 2 re-read FR-081 at head `329b6b9`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The first FR-081 both-forms bullet reads "When a document carries ... and has a section headed `K`, then the engine SHALL". EARS `When` takes no `then`, and the trigger is a state. Use "If ..., then", like its siblings. | spec/functional/FR-081-record-sections-extraction.md:123-126 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The FR-084 section is now five SHALL statements: serialize the selected keys, serialize no other key, order lexicographically, omit `record` in three named cases, and no `record` on a dangling entry. Each has one response. |
| FND-002 | still-open | New this pass; the fix round has not run yet. |
| FND-002 | fixed | Pass 2 (`329b6b9`): `Where` is gone. The follow-on statements read "If an artifact carries both, then the engine SHALL ...", each with one response. |
| FND-003 | still-open | Pass 2 (`329b6b9`): new this pass; no fix round has run for it. |
| FND-003 | fixed | Pass 3 (`2e3cd47`): the bullet now reads "If a document carries a frontmatter-owned value for key `K` and has a section headed `K`, then the engine SHALL emit the warning ...", matching its siblings. |

- FND-001 after_excerpt (`spec/functional/FR-084-two-way-relation-coverage-report.md:92-96`): `The engine SHALL order the keys of record lexicographically by key name, regardless of the order of record_keys or of authoring.` / `The engine SHALL omit record (the key, not an empty object) when no selected key is stated, when the document's archetype has no resolved data_schema, or when record validation of the document fails (any semantic.record-invalid or semantic.duplicate-record-key error).`
- FND-002 after_excerpt (`spec/functional/FR-081-record-sections-extraction.md:127-133`): `If an artifact carries both, then the engine SHALL ignore the section.` / `... SHALL keep only the frontmatter value in record.` / `... SHALL NOT emit semantic.duplicate-record-key for the pair.`
- FND-003 after_excerpt (`spec/functional/FR-081-record-sections-extraction.md:123-126`): `If a document carries a frontmatter-owned value for key K and has a section headed K, then the engine SHALL emit the warning semantic.feature-not-extractable at the heading with reason frontmatter-owned. The explicit null of FR-069 counts as a value.`

Pass-1 verdict: approved for this method. FND-002 is low and does not block. Also examined, with
no defect found: the FR-058 `record_keys` refusal statement, the FR-069 projection, absence,
null and duplicate statements, and the FR-081 section-only statement.

Pass-2 verdict: approved; FND-003 is low and does not block.

Pass-3 verdict: approved; every finding is fixed and nothing new was found. This copy replaces the
`## Disposition pass 2` section committed in `2e3cd47`. That section recorded FND-003 as fixed in
pass 2, had no finding row for it, and said "no new findings".
