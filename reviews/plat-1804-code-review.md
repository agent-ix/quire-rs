---
id: SR-2026112316341704
title: "code-review of the per-declaration load drop (quire-rs PR 715)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PR #715 at b098f417; src/loader/declarations.rs, src/loader/mod.rs, src/loader/manifest.rs, src/loader/flow_rules.rs, src/traceability.rs, src/traceability/prune.rs, src/traceability/field_rules.rs, src/traceability/flow_rules.rs, src/corpus/validate.rs, src/coverage.rs, src/registry.rs, src/error.rs, src/grammar/mod.rs, src/obligation/inventory.rs, CHANGELOG.md, tests/it/*.rs (21 files)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
  - target: ix://agent-ix/quire-rs/FR-096
    type: references
  - target: ix://agent-ix/quire-rs/FR-090
    type: references
---

# Code review of the per-declaration load drop (quire-rs PR 715)

## Summary

Ticket: PLAT-1804. Code review with the rust-review lane (rust-style loaded) of
`git diff origin/main...b098f417` (37 files, +3200/-1314). The PR replaces every
reset of the merged traceability model with per-declaration drops, adds the
structured `ArchetypeLoadFailure.declaration`, the non-registrable
`trace:declared-model-dropped` bundle finding and `CoverageError::DeclarationsDropped`.

Evidence: focused runs through the shared locked build (toolchain 1.98.1):
`cargo test --test it -- declaration_drop field_rules flow_rules required_relations design_join obligations id_column_lists trace_target_sources trace_reference_kinds qualified_trace_ids coverage_rollup::retire implements_relation::present`
179 passed, 0 failed; `cargo test --lib -- loader:: registry:: traceability coverage:: obligation::inventory grammar::`
186 passed, 0 failed. The coder's ambiguous filter runs are now settled: all pass.

## Examined scope

- Reset sites at origin/main `src/loader/mod.rs` (retirement, qualifiers, bound target,
  carried-field conflict, record_keys, flow declarations): all six
  `traceability = TraceabilityModel::default()` assignments are deleted. The model is
  now built once by `merge_traceability` after `declarations::resolve`, and no code path
  in the loader assigns a default model. (examined)
- AC-38 / AC-42: the regression test was committed first (f9a9423) and is red at
  origin/main by construction: rule `bad` makes `validate_flow_declarations` return true
  and the model resets, so `registry.traceability().expect(..)` panics. A reset mutation
  fails the same assertion. (examined)
- Structured failure: `ArchetypeLoadFailure.declaration: Option<Box<DroppedDeclaration>>`;
  `Registry::dropped_declarations` and the bundle finding filter on that field only,
  never on reason text. (examined)
- Non-registrable finding: emitted through `BundleReport::bridged` with no pack, error
  under both postures; `grammar_severity` manifest keys and the `--severity` parser both
  refuse `trace:declared-model-dropped`. Raised once in `validate_corpus`. (examined)
- Cross-module conflicts (relations with different bodies, carried keys, flow-rule and
  remediation names, qualifier prefixes, retirement): every party dropped, failures name
  the other modules in load order, identical relations and identical qualifiers merge.
  Retirement has a gap (FND-005). (examined)
- Cascades: remediation to flow rule, trace target to obligation and document reference,
  status to document reference, relation to design relation, carried field to vocabulary,
  `sets` per rule. Implemented. (examined)
- `coverage::compute` / `compute_with_evidence` return `DeclarationsDropped` before
  touching the model. (examined)
- Archetype with an unresolvable schema reference is now removed from the registry
  (`drop_unresolvable_archetypes`); previously it stayed loaded without a validator and
  the model was reset. Spec-mandated by FR-058 drop units; consequences in FND-003. (examined)
- Public API break: `TraceabilityModel::validate` now returns `Vec<DeclarationFailure>`,
  `ArchetypeLoadFailure` gains a field, `CoverageError` gains a variant. CHANGELOG
  entry present. Downstream: quoin `quoin-quire/src/coverage.rs` matches `CoverageError`
  exhaustively and will not compile on the bump until it maps the new variant; quire-cli
  and quoin construct no `ArchetypeLoadFailure` and call no `TraceabilityModel::validate`. (examined)
- Rust idioms: no `unwrap` in production paths, no `HashMap` in banned modules, typed
  `DeclarationKind` enum, `thiserror` variants. (examined)

## Verdict

Not mergeable yet. The core change is sound and well tested: the resets are gone,
AC-38/41/42/45 have real oracles, and the focused runs are green. FND-001 is high:
a test tagged FR-013-AC-19 was rewritten to assert the opposite of an unamended AC.
FND-002 and FND-003 are the same pattern at medium. Each needs either a spec CR note
amending the older AC or a code change, in this PR. FND-004 and FND-005 are real gaps
in the FR-058 statement. The rest are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-013-AC-19 still requires an unknown key nested under `traceability:` to make every constructor, strict and tolerant, return `ManifestError` naming module, version and key. `deserialize_manifest` now drops the section instead, and the test tagged FR-013-AC-19 was rewritten to assert that the module loads; the strict `load_module_set_strict(nested)` check was deleted. The matrix will show AC-19 covered while the code violates it. Needs a spec CR note amending FR-013-AC-19 to defer nested traceability keys to FR-058-AC-45, or the old refusal kept | src/loader/manifest.rs:642-679 |
| FND-002 | medium | `screen(.., unique=true, ..)` merges identical duplicate entries silently for every kind. FR-086-AC-10 (duplicate `design_relations` entry fails load naming it), FR-091-AC-10 (duplicate name fails naming the entry) and FR-096-AC-6 (duplicate rule name fails) are unamended, and FR-058 states merge-identical only for `required_relations` and `carried_fields`. Tests tagged FR-086-AC-10 and FR-091-AC-10 now assert the merge | src/traceability/prune.rs:176-215 |
| FND-003 | medium | FR-096-AC-27 requires one diagnostic naming the schema reference location and the flow rule and field. The archetype `sink` is now dropped first, so the rule fails only as "subject names an unknown archetype" (no field, no schema location, and misleading: the archetype was declared, then dropped). The tagged test dropped the rule and field assertions. The rule's failure should name the dropped archetype it depended on, as the FR-058 cascade rule requires | tests/it/flow_rules.rs:240-300 |
| FND-004 | medium | FR-058 says `quire coverage` and every other consumer that computes over the merged model SHALL NOT compute silently over dropped declarations. Only `coverage::compute` and `compute_with_evidence` refuse; `corpus::design_matrix`, `corpus::relation_report`, `symbols::trace_search` and `obligation::inventory` compute over the pruned model with no signal | src/coverage.rs:943-956 |
| FND-005 | medium | Retirement conflicts are built per retiring module. With two modules retiring `x` and one declaring `x` in `no_source_symbol`, the declaring module gets two failures for one declaration (two `declared-model-dropped` findings), each naming only one retiring module, against "one failure per declaration ... names every other declaring module". Within one module the failure reads "declared by modules " with an empty list. No three-module test for retirement, qualifiers, flow rules or remediations | src/loader/declarations.rs:282-335 |
| FND-006 | low | Public `TraceabilityModel::validate()` no longer reports a model whose retired values are also in `no_source_symbol`, nor overlapping qualifier prefixes inside one model; both checks moved to the loader's cross-module pass. The in-model overlap assertion in `qualified_trace_ids.rs` was deleted, so within-module overlap is now untested | src/traceability/prune.rs:445-460 |
| FND-007 | low | `CoverageError::DeclarationsDropped(Vec<String>)` carries pre-formatted strings, so quoin must re-parse to map module, kind and item; a `Vec<(module, DeclarationKind, item)>` is the typed form. `order_declaration_failures` swaps in dummy failures with `mem::replace` and clones item strings per comparison; extracting the kinded failures and a stable `sort_by_key` is simpler | src/coverage.rs:61 |
