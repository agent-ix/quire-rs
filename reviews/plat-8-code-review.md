---
id: SR-16307
title: "code-review and rust-review of the FR-075 single population extent (quire-rs PR 591)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@62c592a9; src/semantic/model.rs; src/semantic/mod.rs; schemas/output/semantic-v1.schema.json; tests/it/semantic_model.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-075
    type: references
---

# Code review of the FR-075 single population extent (quire-rs PR 591)

## Summary

Ticket: PLAT-8. One code-review pass, with the rust-review lane, over
`git diff origin/main...62c592a9` (6 files, +138/-26). The reviewer ran
`cargo test --test it` at the head: 524 passed. A throwaway probe test, not
part of the PR and since removed, drove `extract_semantic_json` with the
manifest shape that every current consumer still declares.

Examined, clean:

- `PopulationExtent` is a closed `Copy` enum with lowercase serde names.
  Unknown values are refused, not defaulted.
- `read_population` refuses a population that has no frontmatter `extent`,
  and reports the error at the table line. Members no longer carry
  `extent`, and the member reader drops `map_multiplicity`.
- The schema edit agrees with the serialized `PopulationDecl`
  (`extent`, `extentSpan`, `members`), and every test record is validated
  against `semantic-v1.schema.json` in the `extract` helper.
- Versioning: `semantic-v1` keeps `formatVersion: 1` and is edited in
  place. That matches the repo's precedent for breaking output edits
  (CHANGELOG `assurance-v1 edited in place (no v2)`, `semanticCore`
  removal). No v2 is needed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | [soundness] The old shape is dropped silently, not refused, for every consumer that has not yet changed. spec-objects-business `manifest.yaml:504` (local checkout and upstream main) and filament-core-data `fixture-domain/manifest.yaml:444` both declare the population locator with `columns: [Type, Extent]`. `DeclaredTables::from_dsl` now skips that locator because `spec.reads` fails on `Extent`. The `## Members` table then has no owner, and `TableSpec::for_columns(["Type","Extent"])` matches no spec, so nothing refuses it. Reproduced: a population artifact with the old table under the old manifest yields no `model`, no `availability.model` and no diagnostic. The PR body and FR-075 say the old table "is refused (`semantic.feature-not-extractable`)". That holds only when the manifest has been changed and the artifact has not, which is the case the test covers. Fix: refuse a population locator or table whose header names `Extent` (keep `Extent` recognisable for refusal in `TableSpec`, or emit a load-time diagnostic for a model-table locator that `from_dsl` drops), and test it under the old manifest. | src/semantic/model.rs:139-143, 341-343, 1036-1045 | correct-requirement-no-evidence |
| FND-002 | medium | [soundness] Frontmatter `extent` is read on every artifact. Nothing ties it to a declared population table, and it does not set `out.declared`. An entity with `extent: 5` gets `semantic.invalid-model-cell` while `availability.model` is absent, which FR-075 General does not allow (an error from a declaring source makes the kind `unavailable`). A valid `extent: open` on an artifact whose type declares no population table is ignored silently. The `abstract:` precedent this design cites is gated: undeclared, it is refused with `semantic.feature-not-extractable`. Fix: read `extent` only when the object type declares a population table, and refuse it with `semantic.feature-not-extractable` (section `frontmatter`) otherwise. Set `declared` when it is read. | src/semantic/model.rs:932-942 | wrong-requirement |
| FND-003 | low | [other] No CHANGELOG entry. The change breaks the public Rust API (`PopulationMemberDecl.extent` removed, `PopulationDecl` gains `extent`/`extent_span`, new `PopulationExtent`) and the `semantic-v1` output (`population.extent`/`extentSpan` required, member `extent` refused by `additionalProperties: false`). This repo records such edits under `[Unreleased]` as "Breaking, ... edited in place (no v2)". Fix: add that entry. | CHANGELOG.md:8-30, 151-153 | correct-requirement-no-evidence |

## Verdict

Not mergeable as is. FND-001 makes the intended breaking change silent for
spec-objects-business and the FCD fixture module until they change. FND-002
leaves `extent` ungated, unlike the `abstract:` precedent it follows. The
rest of the Rust change is idiomatic and small.

## Dispositions

Round 1, at head 2337e140 (fix commit checked against 62c592a9). `cargo test --test it`: 526 passed. The reviewer ran a throwaway example, not committed, that feeds the spec-objects-business population skeleton and fixtures through `extract_semantic_json` with that repo's own manifest. It ran on a main build (2a3f56b) and a head build, each in its own target dir.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `TableSpec::for_columns` falls back to `is_legacy_population_header`, and `refuse_table` refuses a `Type \| Extent` table as `population` with a message that names the `Type` table and frontmatter `extent`. That covers an unowned section, a non-conforming owned table and the preamble. It is tested under both a `[Type, Extent]` and a `[Type]` manifest (`legacy_population_table_is_refused_under_either_manifest`). Against upstream spec-objects-business 8ae0482, all three population files are refused loudly at head (`feature-not-extractable`, `availability.model` `unavailable`), where main extracted them. |
| FND-002 | fixed | `extent` sets `declared`. It is refused with `feature-not-extractable` (section `frontmatter`) unless `declared_tables.declares(Population)`. An `extent` with no `Type` table is `invalid-model-cell` at the `extent` line. Both cases are tested (`extent_without_a_declared_population_is_refused`). |
| FND-003 | fixed | CHANGELOG `[Unreleased]` / `### Changed` has a "Breaking, `semantic-v1` edited in place (no v2)" entry that lists the API and record changes. |
