---
id: SR-20261103118140
title: "Gap analysis — PLAT-1370 record-sections extraction against quoin FR-121/122/123 (quire-rs PR 626)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-081-record-sections-extraction.md, src/semantic/record.rs, src/semantic/model.rs, tests/it/semantic_record_sections.rs, crates/quire-fixtures/fixtures/record-sections/cases.json"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-081", type: references}
---

## Summary

Ticket: PLAT-1370. PR head `fe44f02`. This is a planless gap analysis (plan completion
not assessed) over the PR diff only. It checks FR-081-AC-1..15 and their tagged tests
TC-2167..TC-2180, then checks every acceptance criterion of quoin FR-121, FR-122 and
FR-123 on quoin `origin/main` `03a417cb` against the code.

Code with an owning requirement but no test: the `Unmapped` owner path for
`agent-ix/spec-objects-business` (`record.rs:316-320`) and the `model`-unavailable
copy branch (`record.rs:1429`).

Traced and clean:

- Bindings: TC-2167 (FR-081-AC-1, -AC-2, -CON-1) through TC-2180 (FR-081-AC-15), with
  TC-2171 also backing FR-081-CON-2. Each test runs the `cases.json` cases tagged with
  its AC and fails when no case backs it.
- quoin FR-121-AC-1..9: covered. FR-121-AC-8's owned-key examples (`## Thresholds`,
  `## Endianness`) are covered through the architecture cases.
- quoin FR-122-AC-1..7: covered. AC-7 is TC-2179, which ran against the fetched module.
- quoin FR-123: AC-1, AC-3, AC-5 and AC-9 are implemented and covered. The status of
  the other ACs is recorded below.

### quoin FR-123 ACs not implemented (for the lead's fix-in-PR vs follow-up decision)

| quoin AC | Status at `fe44f02` |
| --- | --- |
| FR-123-AC-1 | Implemented and tested (`values-table`); validated against a fixture stand-in for `EnumValue.json`/`Enumeration.json` (see code-review FND-001) |
| FR-123-AC-2 | List refusal tested (`values-list-refused`). Duplicate `Value` and invalid `Value` come from existing FR-075 behaviour; under `record-sections` the copied key's `unavailable` state is untested (FND-006) |
| FR-123-AC-3 | Implemented; `consumes`/`emits` as `SemanticId[]` and transition `emits` are never exercised with values (FND-005) |
| FR-123-AC-4 | **Not implemented**: `members` copy as `RelationDecl[]`, `Member` resolution by declaration references with `construct.references.members` admission, composite verb choice, `member-verb`, `declared-in-members`, `vocabulary` copy as `Term[]`, and the empty-`Description` omission in `read_vocabulary` (`model.rs:1915` still emits `""`) |
| FR-123-AC-5 | Implemented; `## Members` is read once, by its locator, because the key is owned |
| FR-123-AC-6 | Mechanism present (typed relation keys are owned; `source` is a generic scalar `SemanticId`); `## Emits`/`## Persists` prose and the event `## Source` case are untested |
| FR-123-AC-7 | Code present (the `Unmapped` owner for the business package, with `owner` also a systems member); untested |
| FR-123-AC-8 | **Not implemented**: no ownership audit over `agent-ix/spec-objects-business` record schemas (TC-2179 audits only architecture) |
| FR-123-AC-9 | Implemented and tested (`values-header-only`) |
| Upstream item (6) | **Not done**: the FR-075 CR-192 note still says `owner` is read from `## Parent` (spec-review FND-003) |

## Verdict

**PARTIAL.** FR-081 as written is fully backed. Against the normative quoin set,
FR-123-AC-4 and FR-123-AC-8 are unimplemented, and FR-123-AC-6/AC-7 are implemented
but have no test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | quoin FR-123-AC-4 is unimplemented: there is no `members` RelationDecl copy, no `Member` resolution or composite verb, no `member-verb` or `declared-in-members` refusal, no `vocabulary` Term copy, and `read_vocabulary` still emits `doc: ""` for an empty Description. | src/semantic/record.rs:1352-1373, src/semantic/model.rs:1915 |
| FND-002 | medium | quoin FR-123-AC-8 is unimplemented: no test audits that every top-level property of every `agent-ix/spec-objects-business` record schema has exactly one owner. `record_key_owners` could serve it, but the business module is not fetched. | tests/it/semantic_record_sections.rs:350-388 |
| FND-003 | medium | quoin FR-123-AC-7 is untested: the `Unmapped` owner path (`nested_entity` `## Parent`/`## Owner` with no `owner` key and no diagnostic; `domain` with no `ubiquitous_language` key) has no case. | src/semantic/record.rs:316-320 |
| FND-004 | low | quoin FR-123-AC-6 is untested under `record-sections`: no case covers `## Emits`/`## Persists` as prose beside `emits`/`persists` typed relations, or an `event` `## Source` inline id extracting `source`. | crates/quire-fixtures/fixtures/record-sections/cases.json |
| FND-005 | low | quoin FR-123-AC-3: the `steps-copied-states-stay-in-model` case leaves `Consumes`/`Emits` empty and `states-and-transitions` has no `Emits` column, so the `SemanticId`/`SemanticId[]` copy of `emits`/`consumes` is never asserted. | crates/quire-fixtures/fixtures/record-sections/cases.json |
| FND-006 | low | The branch where a model-table record key is `unavailable` with the model's reason (for example a duplicate `Value`, quoin FR-123-AC-2) has no case. | src/semantic/record.rs:1428-1429 |
| FND-007 | low | TC-2179 prints `skipped` and passes when the architecture module cannot be fetched, so FR-081-AC-14 can pass without auditing anything. | tests/it/semantic_record_sections.rs:354-357 |
