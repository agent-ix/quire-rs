---
id: SR-20261103118141
title: "Spec review — FR-081 record-sections extraction vs quoin FR-121/122/123 (quire-rs PR 626)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-081-record-sections-extraction.md, spec/tests.md, spec/spec.md, spec/functional/index.md, spec/functional/FR-075-model-feature-extraction.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-081", type: references}
---

## Summary

Ticket: PLAT-1370. PR head `fe44f02`. This review compares FR-081 (AC-1..15, CON-1..3)
and its TC-2167..TC-2180 rows with quoin FR-121, FR-122 and FR-123 on quoin
`origin/main` `03a417cb`. `quire validate` on FR-081 and `spec/tests.md` reports no
errors. It reports EARS warnings on lines 29, 86, 97, 99, 113, 117 and 179.

Examined and clean:

- FR-081 token, normalization, form, column, cell and availability statements match
  quoin FR-121 Behavior, including `key-ambiguous` at the heading, `no-row` for an
  object key and `[]` for an array key.
- The FR-122 forms and refusals match FR-122 Behavior one for one.
- The FR-123 copy rules for `values`, `states`, `transitions` and `steps` (no
  `sourceSpan`, `ClauseRef` guard, schema-declared keys only, locator-filled keys
  never read as sections) match FR-123.
- FR-081-AC-14 restates quoin FR-122-AC-7.
- FR-081-CON-1/CON-2 restate FR-121-CON-1/CON-3.
- Every FR-081 AC and CON-1/2 has a TC.
- The relationships, index and `spec.md` entries are consistent.

## Verdict

**FAIL (fix in PR or record follow-up).** FR-081 is a faithful but partial restatement
of quoin FR-123. Its criteria bind validation to fixture stand-ins rather than the
module schemas. The FR-075 CR-192 note contradicts FR-081.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-081-AC-1 validates against "the fixture module's `Hazard.json`" (and AC-13 inherits the case schemas), where quoin FR-121-AC-1/AC-4, FR-122-AC-1..5 and FR-123-AC-1/AC-3 name the module's own schemas (`HazardAssessment.json`, `Hazard.json`, `Threshold.json`, `EnumValue.json`, ...). The criterion builds the vendored stand-ins of code-review FND-001 into the spec. | spec/functional/FR-081-record-sections-extraction.md:209 |
| FND-002 | medium | FR-081-AC-15 covers only quoin FR-123-AC-1/2(list)/3/5/9. FR-081 has no criterion for FR-123-AC-2's duplicate and invalid `Value`, AC-4, AC-6, AC-7 or AC-8. Dependencies names the unimplemented upstream items 1 (members/vocabulary), 3, 5 and 6, but not that AC-6, AC-7 and AC-8 have no FR-081 criterion, although the owned-keys statement claims the AC-7 behaviour. | spec/functional/FR-081-record-sections-extraction.md:223 |
| FND-003 | medium | The FR-075 CR-192 note still says `owner` "is a `RelationDecl` read from the required `## Parent`". FR-081's owned-keys statement and quoin FR-123 say `owner` is never extracted (FR-123 upstream item 6, not done in this PR). | spec/functional/FR-075-model-feature-extraction.md:549 |
| FND-004 | low | FR-081 Behavior gives a copied model-table key a `recordSources` span "from the first entry to the last". quoin FR-121 Outputs define `sourceSpan` as "a `SourceLocus` over the key's section", which the header-only branch already uses. The two spans differ for the same kind of key. | spec/functional/FR-081-record-sections-extraction.md:190 |
| FND-005 | low | FR-081's owned-keys statement scopes the unmapped `members` to the whole `agent-ix/spec-objects-business` package. quoin FR-123 and FR-121 scope it to the `members` of `domain` and `population`. The code follows FR-081 (`record.rs:316-320`). | spec/functional/FR-081-record-sections-extraction.md:93 |
| FND-006 | low | The statement on line 99 holds two `shall`s (`ears:non-singular`). Lines 29, 86, 97, 113, 117 and 179 are `ears:unclassifiable` / `missing-subject` per `quire validate`. | spec/functional/FR-081-record-sections-extraction.md:99 |
| FND-007 | low | Ceremony: the PR adds 17 hand-kept rows to the `spec/tests.md` AC→TC audit table. They restate the Verification column of the TC rows it also adds, along with a hand-set ✅ status per TC. The computed matrix (`quire matrix`) already derives both. This is a repo-wide convention, so delete the table and status column repo-wide rather than extending them. | spec/tests.md:2268-2284 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | Not a copy any more. FR-081-AC-1 names a synthetic root and plainly states that the `spec-objects-safety` criteria are not run because that module cannot be fetched and its schemas are never copied. Business-schema validations are stated as not run in Dependencies. The architecture criteria now validate against the fetched module (`validates`). The lead accepted this limitation. |
| FND-002 | fixed | AC-15 is extended (duplicate and invalid `Value`, `Emits`/`consumes` references, document order). The new AC-16 covers quoin FR-123-AC-4/6/7 and the new AC-17 covers fragment, unreadable, TypeRef and Status-once. Dependencies maps every FR-123 AC and states that AC-8 is partial. |
| FND-003 | fixed | The FR-075 CR-192 note now reads "Superseded by quoin FR-123 and quire-rs FR-081 (PLAT-1370): ... `owner` is not extracted at all ... no `## Parent` section is read as its source." |
| FND-004 | fixed | FR-081 now says "Its `recordSources` span is the key's whole section, from its heading to its last non-blank line, as for any other key". The code uses `section_span` and falls back to entries only when no section is found. |
| FND-005 | fixed | The owned-keys statement now names "`owner` of a `nested_entity`, `ubiquitous_language` of a `domain`, and `members` of a `domain` or `population`". The code matches on `(object_type_name, name)`. |
| FND-006 | still-open | The token statement was split, but `quire validate` still flags `ears:non-singular` on line 111 ("SHALL NOT read a record key from frontmatter, and SHALL omit a key whose section is absent") and `unclassifiable`/`missing-subject` on lines 29, 87, 109, 125 and 129. |
| FND-007 | fixed | The 17 FR-081 AC→TC audit rows are removed from `spec/tests.md`, and the new TC-2181/TC-2182 add no audit rows. The per-TC ✅ column remains, as a repo-wide convention. |
