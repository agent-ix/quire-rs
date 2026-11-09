---
id: SR-20261103118139
title: "Code review — PLAT-1370 record-sections extraction (quire-rs PR 626)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/semantic/record.rs, src/semantic/surface.rs, src/semantic/resolver.rs, src/semantic/context.rs, src/semantic/python_entry.rs, src/loader/, src/filament.rs, src/validate_document.rs, schemas/output/semantic-v1.schema.json, crates/quire-fixtures/fixtures/record-sections/, tests/it/semantic_record_sections.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-081", type: references}
---

## Summary

Ticket: PLAT-1370. Frozen PR head `fe44f0280d59113c74f701b085542f82abc25634`, diff
`origin/main...HEAD` (49 files, +6029/-25). This review covers the code lane and the
Rust lane (`/rust-review`), measured against quoin FR-121, FR-122 and FR-123 on quoin
`origin/main` (`03a417cb`) as the normative source.

Examined and clean:

- Token gating. With no `record-sections` token, the record is byte-identical to the
  record of a request with no record schema (FR-081-CON-2, TC-2171).
- Heading normalization, `key-ambiguous`, `second-section`, the object, object-array,
  scalar and scalar-array forms, `no-row`, `second-row`, `record-key-no-block`,
  `column-unknown` and `column-duplicate`, cell typing, `nested-object-cell`,
  first-failing-check, and the `entry-errors` availability.
- FR-104 `Target` resolution for `SemanticId` cells, including `target-not-id`,
  the no-index advisory and `no-bundle-package`.
- The FR-122 forms: `provider`/`exposes`, the four policy tables, `routes` with
  `route-operation-unknown`, `associated_types` read from the table and never from
  the fence, the `## Layout` lowering with `size-unknown-field`/`size-ambiguous`,
  and `thresholds` with `status-required`.
- The FR-123 copy of `values`, `states`, `transitions` and `steps`, with
  the clause-id `Guard` turned into a `ClauseRef`, the exclusion of locator-filled
  keys, and process `## States` left in `model`.
- The `semantic-v1` schema additions, `declaration_record` flattening, and the
  resolver refactor (`module_schema_documents` / `compile_documents`).
- Rust lane: the change has no `unsafe`, no `HashMap`, and no `unwrap` in production
  paths. One `expect` guards a constant regex. The new public API is
  `KeyOwner`, `RecordSource`, `record_key_owners`, `with_record_schema` and
  `compile_documents`; the CHANGELOG records the breaking struct-literal change.

Reviewer gates: `cargo test --test it semantic_record_sections` passed 14/14.
TC-2179 ran against the fetched architecture module; it was not skipped.
`cargo clippy --all-targets -D warnings` passed. Four throwaway
probe tests were run in the review worktree and then removed. They confirmed
FND-002 and FND-003 and the TypeRef silence in FND-007.

## Verdict

**FAIL (fix in PR).** One high finding: the fixture schemas are copies of other
repositories' module schemas. There are three medium correctness findings. The main
extraction path is correct and well tested.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Vendored schemas: 19 of the 24 files in `fixtures/record-sections/schemas/` are derived copies of `agent-ix/spec-objects-architecture`, `-safety` and `-business` schemas, re-`$id`'d under `quire-fixtures`. `Transition.json` differs from the business module's only in `$id` and the semantic-core version; `HazardAssessment.json` replaces the module's `anyOf` enum `$ref`s and `additionalProperties: false` with plain strings. The quoin ACs' "validates against HazardAssessment.json / Threshold.json / the module schema" is therefore proved against weakened stand-ins, and the module-local enum-`$ref` typing path is never exercised. The architecture module is already fetched by `spec-objects-architecture-fixture` (used by TC-2179). Only the synthetic roots (`Ambiguous`, `ArchitectureKeys`, `Evidence`, `EnumValue`, `Signature`) are legitimate fixtures. | crates/quire-fixtures/fixtures/record-sections/schemas/ |
| FND-002 | medium | A flow-style `## Layout` record such as `- {name: m, tag: 0x21, fields: [...]}` extracts `tag` as `"33"`, not `"0x21"` (reproduced). The pre-parse regex quotes only block-style `tag:` lines; `lower_records` then stringifies the YAML integer. This violates FR-122's rule that `tag` is taken as authored text. The fixture itself writes `fields` in flow style. | src/semantic/record.rs:1269-1286, src/semantic/record.rs:1300-1304 |
| FND-003 | medium | `Documents::classify` does not follow a fragment `$ref` (`#/$defs/X`, or the document's own `$id#...`): the URL is `""`, so `lookup` fails and the property becomes `Unreadable`. Its section is then silently prose, with no `availability.recordKeys` entry and no diagnostic (reproduced). The resolver accepts such fragments (`walk` skips them as in-document), so a schema that compiles and validates loses its keys without notice. | src/semantic/record.rs:171-179 |
| FND-004 | medium | `recordSources` is not in document order, as FR-081 Outputs requires. `copy_model_keys` appends the model-table keys after every FR-121 key, so `## Values` written before `## Status` is listed after it. | src/semantic/record.rs:1419-1422 |
| FND-005 | low | `members`/`vocabulary` show up in `availability.recordKeys` only when `model` is unavailable (first match arm). Otherwise they are skipped (`_ => continue`), so the key is present or absent depending on an unrelated table's errors. | src/semantic/record.rs:1428-1434 |
| FND-006 | low | A `thresholds` `Status` cell that fails `cell-type` also triggers `status-required` on the same row, so the cell gets two errors. FR-121 allows only the first failing check per cell. | src/semantic/record.rs:958-971 |
| FND-007 | low | A `TypeRef`/`TypeRef[]` property is `Unreadable` outside `agent-ix/spec-objects-architecture`, so `## Associated Types` in any other module is silently prose with no availability entry (reproduced). FR-121 says another module's property of that name "is read under this requirement". | src/semantic/record.rs:292, src/semantic/record.rs:335 |
| FND-008 | low | The `line_of` doc says `<text>#2` selects the second line, but the code splits on `@`. | tests/it/semantic_record_sections.rs:44-49 |
| FND-009 | low | The fixture README says `cases.json` backs FR-081-AC-1..AC-12, but it also holds the five FR-081-AC-15 (quoin FR-123) cases. | crates/quire-fixtures/fixtures/record-sections/README.md:10 |

## Verification

`cargo test --test it semantic_record_sections` passed (14 tests).
`cargo clippy --all-targets -- -D warnings` passed. Aggregate gates
(`make ci`, `make ci-python`) were not run by this reviewer.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | low | The fix round inserted `object_type` beneath the `schema_documents` doc comment, so that comment ("The documents the `data_schema` `$ref`s name...") now documents `object_type` and leaves `schema_documents` with no doc. | src/semantic/python_entry.rs:69-74 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 102cb00 deletes every copied module schema. The architecture cases load the fetched `spec-objects-architecture` schemas through `Schemas::of` and validate each key against the module schema named in `validates`. The remaining roots are synthetic, carry `$comment: not a module schema`, declare only the key under test, and reference only semantic-core. That safety and business modules are not validated is an accepted limitation, stated in FR-081-AC-1 and in Dependencies. |
| FND-002 | fixed | `parse_layout` deserializes into `RecordDoc { tag: Option<String> }`, so a plain scalar keeps its authored text in flow style too. `layout-fence` now authors `posting_tuple` in flow style and expects `"0x22"`. |
| FND-003 | fixed | `classify` follows a `#` fragment (and an own-`$id` fragment) by JSON pointer within the document. The `fragment-ref-property` case (`Fragment.json`, `#/$defs/policy`) extracts the key. An unreadable shape now yields `schema-unreadable` instead of being silent. |
| FND-004 | fixed | `copy_model_keys` ends with `outcome.sources.sort_by_key(\|s\| s.source_span.start_line)`. Two new cases (`sources-follow-the-document-order` and `-reversed`) assert the order. |
| FND-005 | fixed | The `None` arm is now `_ => KindAvailability::not_applicable()`, so `members` and `vocabulary` always report a state. |
| FND-006 | fixed | `row_rules` receives `clean` (no diagnostic in the row yet) and emits `status-required` only when it is set. `thresholds-bad-status-refused-once` asserts a single `cell-type`. |
| FND-007 | fixed | `Node::TypeRefArray => name == "associated_types"` reads the key in any module (case `type-ref-array-in-another-module`). Another `TypeRef[]` key named by a heading yields `schema-unreadable` (case `type-ref-array-other-key`). |
| FND-008 | still-open | `tests/it/semantic_record_sections.rs:80-81` still says ``<text>#2`` selects the second line, but the code splits on `@`. |
| FND-009 | fixed | The README row now reads "The FR-081 cases (quoin FR-121-AC-1..9, FR-122-AC-1..6, FR-123-AC-1..7 and AC-9); each case names its criterion in `ac`." |
