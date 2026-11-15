---
id: SR-20261115183660
title: "Integrity review of PLAT-1441 FR-069, FR-081, FR-084 amendments"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/functional/FR-081-record-sections-extraction.md, spec/functional/FR-084-two-way-relation-coverage-report.md (PR #688)"
review_set: all
---

## Summary

Ticket: PLAT-1441. Checks cross-requirement consistency, including with the quoin FR-121
amendment (agent-ix/quoin PR #927) that FR-081 claims to follow ("Per `agent-ix/quoin`
FR-121"). One normative contradiction and one traceability gap.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-081 says a frontmatter-owned section adds "no `availability.recordKeys` value for that key", but quoin FR-121 (PR #927) requires `availability.recordKeys` `unavailable` in the section-only error case (FR-121-AC-10); the two specs that must agree disagree. | spec/functional/FR-081-record-sections-extraction.md:107-124 |
| FND-002 | low | FR-081 and FR-084 now depend normatively on FR-069 (and FR-081 on quoin FR-121), but neither adds a frontmatter `relationships` edge; only body links exist. | spec/functional/FR-081-record-sections-extraction.md:9-20; spec/functional/FR-084-two-way-relation-coverage-report.md:5-13 |

### FND-001 detail

quoin PR #927 states: "If a section heading names a key owned by a `frontmatter_field`
locator and the frontmatter holds no value for the field, then Quire SHALL emit the error
... report `availability.recordKeys` for the key `unavailable`, and extract no value for it."
quire-rs FR-081 (this PR) states: "The section adds no `recordSources` entry and no
`availability.recordKeys` value for that key." An implementation cannot satisfy both
FR-121-AC-10 and FR-081-AC-18 + this clause. Align on one rule (`unavailable` with a reason
is the more informative, and matches every other FR-081 refusal), and state the
section-plus-frontmatter case's availability explicitly in both specs, since quoin is silent
on it there.

## Verdict

Changes requested on FND-001. Also examined, with no defect found: FR-069's frontmatter-owned
definition against quoin FR-121's ownership rule ("A `frontmatter_field` locator declared for
a record key SHALL own that key, whether or not the artifact's frontmatter holds the field").
Both are implicit, declaration-driven, and agree. The FR-081-AC-6 amendment is consistent with
the new AC-18. FR-084's new `record` placement (after `criteria`) is consistent with the
serialization ordering rules. AC ids are unique.

## New findings (disposition pass 1)

Pass 1 compared FR-081 at quire-rs#688 head `5731e42` with quoin FR-121 at quoin#927 head
`ab5356be`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | FR-081's absent-section rule ("The engine SHALL omit a key whose section is absent, reporting it `not_applicable`") and its Outputs line (`availability.recordKeys` is present when "a section names, or fails to name, a key") still cover frontmatter-owned keys. So a frontmatter-owned key with no section is reported `not_applicable`, and read literally its frontmatter value is "omitted". FR-121 at `ab5356be` lines 90-91 say the frontmatter value is kept, and that there is "no `recordSources` or `availability.recordKeys` entry for the key". No FR-081 AC covers the no-section case: AC-19's control asserts only "no diagnostic". | spec/functional/FR-081-record-sections-extraction.md:144-145 |
| FND-004 | medium | FR-081's general Availability rule is not limited to keys that are not frontmatter-owned: "If any ... section of a key carries an error ... `unavailable` ... Otherwise the key is `available` and not lossy". In the section-only case it contradicts FR-081's own line 121 (no `availability.recordKeys` entry). In the both-forms case the section carries only a warning, so "Otherwise ... `available`" applies, while FR-121 lines 92 and 123-124 say there is no `availability.recordKeys` entry. The both-forms bullet says only "no `recordSources` entry", and AC-19 asserts nothing about availability. FR-121 limited its own rule this way in round 2. | spec/functional/FR-081-record-sections-extraction.md:192-195 |

## New findings (disposition pass 2)

Pass 2 re-read FR-081 at head `329b6b9`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The Availability rule's "Otherwise the key is `available` and not lossy" still covers frontmatter-owned keys, because the round-1 fix limited only the error branch. Read literally, it reports a frontmatter-owned key `available`, against lines 77, 133 and 149 and AC-19/AC-20. | spec/functional/FR-081-record-sections-extraction.md:196-199 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the section-only case, FR-081 and FR-121 at `ab5356be` now agree: an error at the heading with `reason` `frontmatter-owned`, no `record`, no `recordSources` and no `availability.recordKeys` entry (FR-081-AC-18, FR-121-AC-10). FR-121 dropped its `unavailable` wording to match. The remaining availability differences are the no-section and both-forms cases, which are new findings FND-003 and FND-004. |
| FND-002 | fixed | FR-081 adds `requires` FR-011, FR-084 adds `requires` FR-069, and FR-058 adds `requires` FR-069. FR-081 → FR-069 was correctly not added, because FR-069 already `requires` FR-081 and the edge would make a cycle. FR-081 reaches FR-069 through the body link, and quoin FR-121 stays a prose Upstream with no typed edge back, as quoin#927 settled. A two-hop walk of the new edges' targets (FR-011, FR-069 and their targets) reaches neither FR-058 nor FR-084, so the new edges add no cycle. |
| FND-003 | still-open | New this pass; the fix round has not run yet. |
| FND-004 | still-open | New this pass; the fix round has not run yet. |
| FND-003 | fixed | Pass 2 (`329b6b9`): the absent-section rule and the Outputs line are limited to keys that are not frontmatter-owned. New AC-20 asserts the no-section case. |
| FND-004 | fixed | Pass 2 (`329b6b9`): the Availability error branch is limited. The both-forms bullet adds no `availability.recordKeys` entry, and AC-19 asserts it. |
| FND-005 | still-open | Pass 2 (`329b6b9`): new this pass; no fix round has run for it. |
| FND-005 | fixed | Pass 3 (`2e3cd47`): the `Otherwise` clause now reads "Otherwise a key that is not frontmatter-owned is `available` and not lossy", so frontmatter-owned keys get no availability entry on any path. |

- FND-001 after_excerpt (`spec/functional/FR-081-record-sections-extraction.md:117-122`): `If a heading names a frontmatter-owned key and the frontmatter does not carry the key's value, then the engine SHALL emit the error semantic.feature-not-extractable at the heading with reason frontmatter-owned, read no value from the section, and produce no record entry, no recordSources entry and no availability.recordKeys entry for the key.`
- FND-002 after_excerpt (`spec/functional/FR-084-two-way-relation-coverage-report.md:10-11`; `spec/functional/FR-081-record-sections-extraction.md:13-15`): `- target: "ix://agent-ix/quire-rs/spec/functional/FR-069"` / `type: "requires"`; `- target: "ix://agent-ix/quire-rs/spec/functional/FR-011"` / `type: "requires"`
- FND-003 after_excerpt (`spec/functional/FR-081-record-sections-extraction.md:146-149`): `The engine SHALL omit a key that is not frontmatter-owned and whose section is absent, reporting it not_applicable. A frontmatter-owned key with no section keeps its frontmatter value in record and has no recordSources entry and no availability.recordKeys entry.`
- FND-004 after_excerpt (`spec/functional/FR-081-record-sections-extraction.md:132-133, 196`): `If an artifact carries both, then the section SHALL add no recordSources entry and no availability.recordKeys entry.` / `If any header, row, block or section of a key that is not frontmatter-owned carries an error, ...`
- FND-005 after_excerpt (`spec/functional/FR-081-record-sections-extraction.md:196-200`): `If any header, row, block or section of a key that is not frontmatter-owned carries an error, then the engine SHALL report the key unavailable ... Otherwise a key that is not frontmatter-owned is available and not lossy.`

Suggested fix for FND-003 and FND-004, which needs no new mechanism: limit FR-081 lines 144-145
and 192-195 to keys that are not frontmatter-owned, as FR-121 lines 83 and 123 do. Add "and no
`availability.recordKeys` entry" to the both-forms bullet. Then extend AC-19 to assert, for the
both-forms document and the no-section control, that there is no `availability.recordKeys`
entry for `K` and that the no-section `record` holds the frontmatter value. quoin#927 moved to
`8b7a934e` during this pass. It deleted line 91's "report the section mapping `not_applicable`"
clause and added "no `recordSources` or `availability.recordKeys` entry" to FR-121-AC-12 and
AC-13. That makes FND-003 sharper: the quoin fixtures now assert the no-entry outcome that FR-081
line 144 contradicts.

Pass-1 verdict: changes requested, on FND-003 and FND-004 (medium).

Pass-2 verdict: approved; FND-005 is low and does not block.

Pass-3 verdict: approved; every finding is fixed and nothing new was found. The frontmatter-owned
rules at FR-081 lines 77, 117-133, 146-149 and 196-200 now agree with each other and with
AC-18..AC-21.
