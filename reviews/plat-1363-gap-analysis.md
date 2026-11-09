---
id: SR-20261103118134
title: "Gap analysis — PLAT-1363 semantic spec cleanup (quire-rs PR 617)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-069..FR-072, spec/tests.md, tests/it/semantic_surface.rs, tests/it/output_contract.rs, tests/it/assurance_export.rs, tests/it/assurance_boundary.rs, tests/it/coverage_baseline.rs, plan/Plan-003-semantic-extraction-boundary"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-072", type: references}
---

## Summary

Ticket: PLAT-1363. PR head `6abef71`. This is a planless gap analysis scoped to the PR diff and
the ticket's listed deliverables. Plan completion was not assessed.

The ticket lists four deliverables. Three are done:

- the prose cleanup of FR-069..072 and the listed TCs
- the deletion of `semantic-v1.json`
- the removal of the FCD#36 deferral

The fourth, "tag TC-2055", is not done. `grep TC-2055` hits only `spec/tests.md:1129` and
`:2169`, and no test carries the tag.

The reviewer checked the existing tests against FR-072-AC-9's three clauses:

- **coverage-v1 byte equality** is covered by `tests/it/coverage_baseline.rs:59`
  `tc824_coverage_report_matches_the_checked_in_baseline`. It compares the rendered report
  byte for byte with `tests/fixtures/coverage_baseline/expected.json`. Tag it TC-2055 /
  FR-072-AC-9.
- **assurance-v1 byte equality** has no test. `assurance_export.rs:254` (TC-1086/1088) and `:439`
  (TC-1098) only compare two exports from the same run with each other. `:175` (TC-1084) only checks
  that the hand-written `tests/fixtures/assurance/v1.json` validates against the schema. No test
  compares a produced export with that file. `assurance_boundary.rs:49` (TC-1090) checks that the
  coverage and properties schemas declare no assurance version keys. That is not byte equality.
  Do not tag any of them for this clause. It needs a new golden test, or the AC needs restating to
  "byte-deterministic across runs", which TC-1098 already proves.
- **properties-v1 byte equality** has no test. quire-rs has no checked-in properties-v1 output
  fixture. The envelope is built in quire-cli (`output_contract.rs:479-482` says so).
  `output_contract.rs:442` (TC-857) only checks that emitted criteria match the schema. The clause
  needs a new golden test over `classify_document_criteria` records, or it belongs to quire-cli.
- **severity set** (`CoreExtractionDiagnostic.severity` within `info`/`warning`/`error`) has no
  test. The field is a free `String` (`src/filament.rs:183`). The semantic mapping at
  `src/filament.rs:1194-1201` is a closed match, but other constructors pass any
  `impl Into<String>` (`src/filament.rs:1108-1114`). A new test should collect every diagnostic
  from the filament_core fixtures and the semantic case suite and assert each severity is in the
  set. Better still, make severity a closed enum so the type enforces it.

## Verdict

**FAIL (medium)**. The ticket's "tag TC-2055" deliverable is missing. Of AC-9's four clauses,
only the coverage byte clause has an existing test to tag. One low stale reference remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Ticket deliverable "tag TC-2055" is not done, and FR-072-AC-9 has no tagged test. Tag `coverage_baseline.rs:59` tc824 (coverage byte clause). The assurance and properties byte clauses have no existing test (TC-1086/1098 prove only run-to-run determinism; TC-857 proves only schema conformance). The severity-set clause needs a new test. TC-2055 stays 🚧 until then. | spec/tests.md:1129, spec/functional/FR-072-semantic-extraction-surface.md:164, tests/it/coverage_baseline.rs:59, tests/it/assurance_export.rs:254, src/filament.rs:183 |
| FND-002 | low | `plan/Plan-003-semantic-extraction-boundary/tasks/Task-020-surface-schema-bindings.md:41` still names the deleted `tests/fixtures/semantic/semantic-v1.json` as a deliverable. It is the only reference left outside historical `reviews/`. | plan/Plan-003-semantic-extraction-boundary/tasks/Task-020-surface-schema-bindings.md:41 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-072-AC-8 / TC-1638 → schema_and_compatibility_fixture | examined | Binding correct; oracle intact |
| FR-072-AC-9 / TC-2055 | examined | Untagged (FND-001) |
| FR-072-AC-5 / TC-1634 → validate_document_surface (legacy-free-table.md) | examined | Binding correct |
| FR-070-AC-8 / TC-1617 → semantic_properties.rs legacy_forms | examined | Reads legacy.expected.json, whose first case is legacy-free-table.md; correct |
| FR-070-AC-1/2, FR-071-AC-1 fixtures | examined | Read via quire_fixtures::mapping_fixture from crates/quire-fixtures/fixtures/mapping/; correct |
| References to semantic-v1.json (Makefile, docs, CHANGELOG, schemas, scripts, src, tests) | examined | None except plan Task-020 (FND-002) and historical reviews/2026-09-03-plan-003-… |
