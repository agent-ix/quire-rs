---
id: SR-20261103118123
title: "code-review of declaration references and typed relation keys (quire-rs PR 593)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@fbf580d; src/semantic/{model,target,relations,context,surface,python_entry}.rs; src/loader/mod.rs; src/filament.rs; src/validate_document.rs; schemas/output/semantic-v1.schema.json; tests/it/semantic_*.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-075
    type: references
  - target: ix://agent-ix/quire-rs/FR-076
    type: references
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
  - target: ix://agent-ix/quire-rs/FR-072
    type: references
---

# Code review of PR 593 (PLAT-35, PLAT-55)

## Summary

Tickets: PLAT-35, PLAT-55. One code-review pass, with the Rust lane folded in,
over `git diff origin/main...fbf580d` (19 files, +6143/-173). The reviewer ran
these on the head, in a scratchpad checkout: `cargo fmt --check` is clean.
`cargo clippy --all-targets -D warnings` (default features) is clean.
`cargo test --test it` passes 540 of 540. A release wheel was built with
`--features python` and `pytest tests/python/test_semantic.py` passes 4 of 4,
which covers Python and Rust JSON parity for all 36 cases, the 18 new ones
included. `test_speedup.py` fails because `yaml` is missing from the throwaway
venv. That is an environment problem, not this PR.

Examined units:

- FR-075 Declaration references: `ReferenceScope`, `ReferenceColumn::admission`,
  `TableRead::references` and `settle_references`, plus `target::resolve_reference`,
  `Declarations` and `Admission`. The candidate join, the admission precedence
  (construct, then `allowed_links`, then any), the outcome codes, the
  repeated, empty and too-many cell rules, the no-index and no-package states,
  and `model_availability` with `unresolved` all match the spec text.
- FR-076 typed relation keys: `RelationSchema`, `TypedShape`,
  `ExtractedRelation::split`, the `VerbNotTypedKey` check after
  `verb-not-allowed`, and `TypedKeyInFrontmatter`. They match FR-076 Outputs
  and "Typed relation keys".
- FR-069 duplicate authority: `loader::duplicate_authority` on both the inline
  and filesystem load paths.
- Surfaces: the Filament snapshot `constructReferences` and `with_data_schema`,
  the `validate_document` construct and data_schema wiring, and the Python
  `constructReferences` and `dataSchema` request members.
- `semantic-v1.schema.json`: transition `emits` is now a `SemanticId`, step
  `consumes`/`emits` are `semanticIdList`, `relationSource` gains
  `verb`/`target`/`multiplicity`, and the root `additionalProperties` changed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [soundness] The root `additionalProperties` of `semantic-v1.schema.json` changed from `false` to an array-of-`semanticId` schema. FR-072 Behavior and FR-072-AC-8 still require `additionalProperties: false` on every object, and spec PR 592 did not amend FR-072. TC-1638's `every_object_closed` only checks that the key is present, so it passes without checking the value. Blast radius is narrow. Only an unknown top-level key whose value is an array of `ix://` strings now passes semantic-v1. Any other unknown key still fails. The per-type data schemas still close the declaration record (spec-objects-business `Process.json` uses `unevaluatedProperties: {not: {}}`). PLAT-323 (manifest keys at load) is untouched. Fix: amend FR-072 to name the root typed-key exception, and make TC-1638 require `false` everywhere except that documented root schema. | schemas/output/semantic-v1.schema.json:7; spec/functional/FR-072-semantic-extraction-surface.md:124,146; tests/it/semantic_surface.rs:499-515 | wrong-requirement |
| FND-002 | low | [exceeds] `duplicate_authority` refuses a `table_row` locator keyed on a typed key under `yield_pattern.per_match` as well as under `match`. FR-069 names only the locator's match key. So a module the spec loads (a per_match `emits` locator) is refused. Either drop `per_match` from the scan, or extend FR-069 to cover it. | src/loader/mod.rs:926-936; spec/functional/FR-069-semantic-module-contract-at-load.md:125-128 | implementation-bug-despite-evidence |

## Verdict

The approach is sound. Admission, candidate resolution, typed-key lowering and
the duplicate-authority refusal follow the merged spec. Fmt, clippy, the `it`
suite and the Python parity tests are green. The coder's judgement calls,
assessed:

1. Typed mode engages only when a type has at least one typed key: accept. This is
   consistent with FR-076 Inputs ("a verb in the type's `allowed_links`"),
   AC-20 (no typed key means byte-identical) and the CR-193 note. The
   `verb-not-typed-key` rule sits under "Typed relation keys", so it applies
   only to typed types.
2. `SemanticId[]` is detected by an `items.$ref` prefix and suffix: accept. Real
   spec-objects-business schemas use the absolute
   `https://schemas.agent-ix.org/semantic-core/0.3.3/SemanticId.json`, and
   FR-069 forbids rewriting refs at load. A `$defs`-local or relative ref would
   not be detected, but no module uses one.
3. Construct references with no relation vocabulary admit any declaration:
   accept. Roles come only from the vocabulary, and FR-075 says "a context
   without them admits every declaration". As a consequence, the new Filament
   `constructReferences` field has no effect until filament-core-service#32
   supplies a vocabulary.
4. no-bundle-package wins over no-bundle-index, and kind-mismatch/ambiguous
   carry no `reason`: accept. FR-075 says "with neither package known … the
   no-bundle-package state". The spec states a reason only for unknown-name,
   repeated-reference, empty-item and too-many-references.
5. Test-harness changes: accept. Main had 0 cases with `bodyExtraction`, so
   the TC-1846 filter drops no prior coverage. AC-21 and AC-27 require the
   `PLAT-35`/`PLAT-55` issue_ref, which the tracker-key acceptor admits.
6. Root `additionalProperties`: a defect, FND-001.
