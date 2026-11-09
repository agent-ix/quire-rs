---
id: SR-20261103118133
title: "Code review — PLAT-1363 semantic spec cleanup, TC-1638 oracle swap (quire-rs PR 617)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; tests/it/semantic_surface.rs, tests/fixtures/semantic/semantic-v1.json (deleted)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-072", type: references}
---

## Summary

Ticket: PLAT-1363 (child of PLAT-287). PR head `6abef71`, diff `git diff origin/main...HEAD`.
Rust lane (rust-review with rust-style). The one Rust change is in
`schema_and_compatibility_fixture` (TC-1638, FR-072-AC-8). The test used to read
`tests/fixtures/semantic/semantic-v1.json`. It now reads `cases.expected.json`. The test's
`UPDATE_SEMANTIC_BASELINES` mint block and its "fixture equals the engine" loop are gone.

The oracle is just as strong. The reviewer checked this directly. Each of the five records in
the deleted fixture (`golden-table-available`, `operations-available`, `required-section-missing`,
`legacy-bullets-unavailable`, `unresolved-type-lossy`) equals its entry in `cases.expected.json`
once `$case` is removed. So the deleted file was a strict subset of `cases.expected.json`.
The dropped mint block only regenerated that subset from `extract_semantic_json`, which
`case_suite` (TC-1630, `tests/it/semantic_surface.rs:190-211`) already does for all 57 cases
through `write_or_compare`. The dropped drift loop compared engine output to the pinned records.
`case_suite` makes the same comparison byte for byte over every case, so engine drift still
fails a test. Dropping both was right, because they duplicated TC-1630.

The new test keeps every assertion it had before:

- each record validates against the schema
- all four availability state tokens appear
- the ten named top-level keys are present
- removing `availability` or renaming `diagnostics` to `findings` fails schema validation

It now checks these over 57 records instead of 5. `records[0]` is `both-forms-unavailable`,
because serde_json without `preserve_order` uses BTreeMap order. That record carries both
`availability` and `diagnostics`, so the `remove("diagnostics").unwrap()` is sound.

Gates run by the reviewer:

- `cargo test --test it -- semantic_surface:: semantic_fixtures::`, through `locked-build.sh` with
  `CARGO_PROFILE_DEV_DEBUG=0` and `--jobs 4` — 15 passed, 0 failed
- `rustfmt --check` on the touched file — clean

## Verdict

**PASS**. No defects in the Rust change. The test is no weaker. The deleted fixture duplicated
`cases.expected.json`, and the dropped mint and drift blocks duplicated `case_suite`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| tests/it/semantic_surface.rs `schema_and_compatibility_fixture` | examined | Oracle is just as strong; clean |
| tests/it/semantic_surface.rs `case_suite` / `write_or_compare` | context_only | Carries the engine-drift oracle the deleted loop duplicated |
| tests/fixtures/semantic/semantic-v1.json (deleted) | examined | Five records, each equal to its `cases.expected.json` entry |
| Remaining references to semantic-v1.json | examined | Only plan/ (see gap-analysis) and historical reviews/ |
