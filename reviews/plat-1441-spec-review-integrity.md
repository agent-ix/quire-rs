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
