---
id: SR-20261115183659
title: "Base spec review of PLAT-1441 FR-069, FR-081, FR-084 amendments"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/functional/FR-081-record-sections-extraction.md, spec/functional/FR-084-two-way-relation-coverage-report.md (origin/main...7b43672, PR #688)"
review_set: all
---

## Summary

Ticket: PLAT-1441. Spec-only PR. The FR-069 frontmatter projection and the FR-081
frontmatter-owned section rules are sound in intent: projection before `data_schema`
validation, no defaulting on absence, empty, duplicate and scalar refused, and no key name
held by the engine. The FR-084 amendment carries each document's whole composed `record`
and leaves key selection to the consumer. That contradicts the downstream requirement it
exists to serve (`agent-ix/spec-objects-architecture` FR-013), which forbids the report
from showing the confidentiality `level`. FR-084 needs a generic, declared selected-field
mechanism.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-084 carries the whole composed `record` with no key selection, so the report exposes `level` (and every other record key), contradicting spec-objects-architecture FR-013 "The report SHALL NOT include the confidentiality `level`" and FR-013-AC-5; a generic declared selected-field mechanism is needed. | spec/functional/FR-084-two-way-relation-coverage-report.md:79-87 |

### FND-001 detail

`DataClassification.json` declares `level` (`ConfidentialityLevel`) as a record property.
`spec-objects-security` enables `record-sections`. Under FR-069-AC-34, every authored
frontmatter key except the metadata keys is projected, so an author writing `level:` in
frontmatter puts `level` in the record. Under FR-081, a `## Level` section is read into
`level`. The record also carries `fields` (the full `FieldDecl[]`, including the `level`
field declaration), `clauses` and `values`. FR-084 states "The engine selects no key: a
consumer reads the values it needs", and quire-cli FR-030 (PR #204) passes the record
through unfiltered. No layer then enforces FR-013's no-`level` rule. FR-013-AC-5 ("contains
no `level` value") can pass only by accident of authoring. FR-013 itself says the change is
"a carried-field entry on `linked` rows that reads `regulated_categories` from the target".

Recommended: a generic, declared selection on the `required_relations` entry. For example,
`carry: [<record key>, ...]` names the record keys of the counterpart copied onto each
resolved `linked` entry as `record`. When `carry` is absent, nothing is carried. A key the
target did not state stays absent and is never defaulted. Unknown keys are refused at load,
against the counterpart kinds' `data_schema`. This keeps the engine free of domain names
because the module declares the key, satisfies FR-013 by construction, and bounds report size.
It also needs a quoin/manifest-schema change for the new field, and an AC asserting that an
unselected key (`level`) is absent.

## Verdict

Changes requested. FND-001 blocks: the FR-084 design does not meet its consumer requirement.
Also examined, with no defect found: the FR-069 projection-before-validation rule, absence
without default (AC-41), the refusal of invalid, empty, duplicate and scalar values (AC-40),
the FR-081 error and warning split, determinism of the reverse view ordering (consistent with
existing FR-084 ordering rules), and the absence of domain hardcoding in the engine statements.

### Independent checks

- `quire validate` and `quoin validate` were denied by the permission layer in this review
  session as well as in authoring. The changed files have **not** been validated. The AC
  tables were checked by hand: ids are sequential and unique for the new rows.
- Cross-repo inputs read: spec-objects-security FR-023, manifest.yaml, DataClassification.json,
  skeleton; spec-objects-architecture FR-013; quoin PR #927 (FR-121); quire-cli PR #204 (FR-030).
