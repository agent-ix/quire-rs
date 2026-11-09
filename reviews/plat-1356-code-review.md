---
id: SR-20261103118133
title: "Code review — PLAT-1356 Clock, Quantity<u>, unit tokens, importedArtifacts (quire-rs PR 611)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/semantic/properties.rs, src/semantic/units.rs, src/semantic/context.rs, src/semantic/model.rs, src/semantic/mod.rs, schemas/output/semantic-v1.schema.json, spec/tests.md, tests/it/semantic_model.rs, tests/it/semantic_properties.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-070", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-075", type: references}
---

## Summary

Ticket: PLAT-1356. Frozen PR head: `e4f73ec`. Code review with the Rust lane (rust-review + rust-style) folded in. The PR adds the native `Clock` Type cell, the `Quantity<u>` token, a unit/dimension token resolver (`src/semantic/units.rs`) over the own bundle and the new `BundleIndex.importedArtifacts`, widens `semantic-v1` `typeRef` with a `clockTypeRef` alternative, and recognises `Base`, `Dimension | Exponent` and `Dimension | Target | Scale | Offset` tables only to refuse them when undeclared (FR-075-AC-19).

Gates run on the frozen head (worktree target dir, `locked-build.sh`): `cargo fmt --check` clean; `cargo clippy --locked --all-targets -- -D warnings` clean (python-feature clippy lane not run; the PR touches no `src/python`); `cargo test --lib semantic::units` 6/6 pass; `cargo test --test it -- semantic_properties semantic_model semantic_clauses semantic_systems semantic_surface semantic_relations` 84 pass. Aggregate gate not run (per brief).

Two behaviours were reproduced with a scratch probe crate calling `python_entry::extract_semantic_json` against the PR head: an operation `Param` or `Returns` typed `Clock` yields `semantic.internal-invalid-decl` ("engine defect") with `operations` unavailable, while the same table typed `String` extracts cleanly; and a module that declares a `Base` locator gets `model` `available` with no `dimension` member and no diagnostic.

Clock wire shape. quire-rs FR-070 (Behavior, "Native names and quantities") says `Clock` "maps to the target `Clock`, beside the kernel scalar names", is validated against `FieldDecl.json` "with its `target` read as the `SemanticId` `ix://quire/native/Clock`", and that `semantic-v1.schema.json` "admits the target `Clock`". The PR's `{kind: KernelScalar, value: Clock}` plus a validation-time rewrite and a `semantic-v1` `anyOf` is the literal reading of that text. The authoritative contract disagrees: filament-core-data semantic-core 0.3.3 (embedded at `6f3da0f`, `packages/semantic-core/main.tsp`) defines `TypeRefTarget` as a tagged union whose `KernelScalar` branch is a closed enum of eleven names with no `Clock`, and FCD FR-092/FR-093 lower every native type to the `SemanticId` `ix://quire/native/<Name>`. Nothing in FCD names `Clock` as a kernel scalar. The contract-correct wire shape is therefore `{kind: SemanticId, value: "ix://quire/native/Clock"}`, which validates against `FieldDecl.json` and `OperationDecl.json` unmodified and needs neither the rewrite nor the `semantic-v1` widening. FND-002 records this as a spec/contract divergence to resolve by CR; FND-001 is its concrete fallout.

No AC/TC id is added, so there is no id collision with #606 (TC-2081) or #607 (TC-2082), both now merged. The PR does conflict textually with main in `spec/tests.md` (the FR-069 row changed by #606 sits next to the FR-070 row changed here) and needs a rebase.

## Verdict

**CHANGES REQUIRED** — one high finding. The `Quantity<u>` grammar, token resolution precedence, reason precedence and owner-qualified identities match FR-070 rules 1–5; the `Clock` property path, fence parity and refusals match FR-070-AC-11; FR-075-AC-19 refusals carry the right reason, section and span. FR-075-AC-16..18/20/21 stay `🚧` in `spec/tests.md`, carry no trace tag in `src/` or `tests/`, and the PR body says the readers are not part of this PR, so nothing claims them. Not mergeable until FND-001 is fixed and the `spec/tests.md` conflict is rebased.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | An operation `Param` row or `Returns:` typed `Clock` maps through the shared `map_type` to `KernelScalar("Clock")`, and `clauses.rs` `validate_decls` checks the `OperationDecl` against semantic-core with no `Clock` read-as rewrite, so it emits `semantic.internal-invalid-decl` (engine defect) and drops every operation of the artifact (`operations` unavailable, `entry-errors`). Reproduced; a `String` control extracts. Only the Properties path got the rewrite. | src/semantic/properties.rs:703-724, src/semantic/properties.rs:1271-1272, src/semantic/clauses.rs:935-954 |
| FND-002 | medium | `Clock` is emitted as `{kind: KernelScalar, value: Clock}`, a value the authoritative semantic-core 0.3.3 `TypeRefTarget` rejects (its `KernelScalar` enum has no `Clock`); any consumer validating or deserializing with FCD's generated types fails on every `Clock` field. FCD's native-type convention (FR-092, FR-093) is the `SemanticId` `ix://quire/native/<Name>`. FR-070's Clock bullet should be amended by CR to emit that shape, which removes the validation rewrite and the `clockTypeRef` widening. | src/semantic/properties.rs:720-723, schemas/output/semantic-v1.schema.json:79-101, spec/functional/FR-070-typed-properties-extraction.md:122-130 |
| FND-003 | medium | A module that declares a `dimension`, `terms` or `unit` `table_row` locator gets the table claimed (`declared = true`, pushed to `read`) and then `read: None` skips it silently: `model` is `available` with no member and no diagnostic. Reproduced with a declared `Base` locator. Until AC-16..18 land, a declared locator should refuse visibly instead of dropping the table. | src/semantic/model.rs:233-248, src/semantic/model.rs:1403-1410 |
| FND-004 | low | `BundleIndex::is_empty` now also counts `artifacts` and `importedArtifacts`, which changes the reason for an unresolved plain `Type` cell from `no-bundle-index` to `unknown-token` under an index that has only artifacts. No test covers that Type-cell change, and FR-070 rule 5 defines emptiness as "no object, enumeration or imported artifact" (it names neither `artifacts` nor `imports`). | src/semantic/context.rs:91-97, src/semantic/properties.rs:819-820 |
| FND-005 | low | The resolver uses strings for closed sets: the object kind is passed as `&str` ("unit"/"dimension") and `Resolution::Unresolved` carries `reason: &'static str`. CLAUDE.md "Design taste" asks for enums over stringly-typed values; a typo in a kind or reason compiles and silently resolves nothing. | src/semantic/units.rs:57, src/semantic/units.rs:156-159 |
| FND-006 | low | `CHANGELOG.md` `[Unreleased]` has no entry for the `semantic-v1` `typeRef` widening, the `Clock`/`Quantity<u>` targets, or the new `BundleIndex.importedArtifacts` input, all of which are user-visible wire changes. | CHANGELOG.md:8 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-070-AC-11 `Clock` Type cell (properties.rs `map_type`, `validate_decls`) | examined | Property path correct; operation path FND-001; wire shape FND-002 |
| FR-070-AC-12/13 `map_quantity` grammar and refusals | examined | Clean |
| FR-070-AC-14 `units.rs` `UnitToken::parse`, `resolve_token`, `settle` | examined | Precedence and reasons match rules 1–5; FND-005 style |
| `BundleIndex.importedArtifacts`, `is_empty` (context.rs) | examined | FND-004 |
| FR-075-AC-19 `TABLE_SPECS` entries, `table_features` | examined | Undeclared refusal correct; declared path FND-003 |
| `schemas/output/semantic-v1.schema.json` `typeRef`/`clockTypeRef` | examined | Admits the emitted shape; see FND-002 |
| FR-075-AC-16..18, AC-20, AC-21 | context_only | Unimplemented, `🚧`, untagged, not claimed |
| Open PRs #606/#607 | context_only | Merged; no id collision; `spec/tests.md` text conflict |
