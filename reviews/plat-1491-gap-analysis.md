---
id: SR-20261112205306
title: "gap-analysis of PLAT-1484 / PLAT-1491 against FR-049, FR-050, FR-069 and FR-076 (quire-rs PR 660)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@a1203093; spec/functional/FR-049-verification-reference-integrity.md; spec/functional/FR-069-semantic-module-contract-at-load.md; spec/functional/FR-050-declarative-coverage-computation.md; spec/functional/FR-076-relationships-extraction.md; spec/tests.md; tests/it/located_findings.rs; tests/it/coverage_rollup.rs; src/loader/manifest.rs; src/loader/record_validation.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-049", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-069", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-050", type: references}
---

## Summary

Tickets: PLAT-1484 and PLAT-1491. The ticket text was read as data. Plan completion: not assessed. Checked: FR-049-AC-23..26 against TC-2319..2322, FR-069-AC-29 against TC-2318, the FR-050-AC-58 AC→TC audit rows, and the PR's FR-069 claim about the other optional objects of an archetype entry. That claim was checked against the authoritative module-manifest schema (`filament-core-data` main, `schema/semantic/v1/module-manifest.schema.json`, `ArtifactTypeEntry`/`ObjectTypeEntry`).

## Verdict

**FAIL.** Each new AC has exactly one TC row, and each TC is tagged by a real, discriminating test. The mutations are recorded in the code-review SR. `where: null` is refused as claimed: `RawRequiredRelation.selector` uses `deserialize_present`, and `tests/it/required_relations_where.rs:341` covers it under FR-058-AC-19. But the new FR-069 sentence saying no other optional object of an archetype entry has the null-as-absent reading is false (FND-001). FR-076 is used only as a contrast in FR-049, and correctly so: its `## Relationships` lowering still refuses a second table, and this PR does not touch it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-069 now states that "the remaining optional objects of an archetype entry carry no null-as-absent reading". The code violates this. Probed on the PR head with the TC-2318 fixture: `data_schema: null`, `body_extraction: null` and `frontmatter_schema_ref: null` each load the archetype with zero failures, read as absent. A top-level `semantic: null` also loads as "no semantic block". The authoritative schema types `data_schema` as an object (oneOf), `body_extraction` as `$ref BodyExtraction` (an object), and `frontmatter_schema_ref` as a string that `ArtifactTypeEntry` requires. So an authored `data_schema: null` silently disables record validation, which is the same defect PLAT-1484 fixes for `record_validation`. Fix: keep explicit null as present for these keys and refuse it naming the archetype, with a test per key. Otherwise remove the false sentence and track the remaining keys explicitly. | spec/functional/FR-069-semantic-module-contract-at-load.md:333-340; src/loader/manifest.rs:472-477 |
| FND-002 | low | FR-049-AC-26 says the id "counts as referenced in the coverage matrix". TC-2322 asserts only that the second-table row appears in `unbacked_rows` with the target id. `coverage_matrix` is the FR-050-AC-47 obligation population, which this test never reads. Fix: reword the AC to the observable the test checks (a referencing row in the rollup), or assert on the field the AC names. | spec/functional/FR-049-verification-reference-integrity.md:186; tests/it/coverage_rollup.rs:4217-4242 |
| FND-003 | low | FR-049-AC-24's second clause says a first-table-only bundle "reports the findings, order and lines it reported before this criterion". TC-2320 compares two inputs under the new engine (a single table against a single table plus a legend), not old against new behaviour, so the test cannot fail on a change that alters both alike. The corpus sweep in the code-review SR (291 repos, identical) is the real evidence. Fix: phrase the clause as the two-input equivalence the test checks, or pin a known single-table finding list. | spec/functional/FR-049-verification-reference-integrity.md:184; tests/it/located_findings.rs:404-437 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The list reads as exhaustive but is not. Five carry-over keys the engine reads in Archetype::carry_over are typed non-null by the authoritative schema: grammar_ref (string), defaults (object), allowed_links (array/object), roles (array) and construct (object). Authored as null, each still loads as absent with zero failures (probe on 356294a). Fix: add them to NULL_REFUSED_KEYS (note_explicit_nulls already reads every key generically) with a TC-2323 case, or scope the sentence to the keys it names. | spec/functional/FR-069-semantic-module-contract-at-load.md:339-345 |
| FND-005 | low | The refusal in compile_archetype, which is the Registry::load_module path every on-disk module takes, is untested. With it disabled, TC-2318, TC-2323 and TC-2324 still pass, because they all use load_inline_module. A probe shows the path works today. Fix: add a directory-module case to TC-2323. | src/loader/mod.rs:924-926 |
| FND-006 | low | An `archetypes:` entry whose only owned key is authored as null (for example `data_schema: null`) is now dropped silently as a presentation row, with no failure. declares_quire_key does not count null_key. In round 0, `record_validation: null` there was refused. Fix: treat null_key.is_some() as declaring an owned key. | src/loader/manifest.rs:505-534 |

## Dispositions

Round 1, PR head `356294a`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
