---
id: SR-20261103118145
title: "Code review — PLAT-277 F4 presentation-only archetypes rows (quire-rs PR 631)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/loader/manifest.rs, src/loader/mod.rs, CHANGELOG.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-013", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-031", type: references}
---

## Summary

Ticket: PLAT-277 (finding F4). Frozen PR head: `91cdf225b0ceaf29646f4f59a88e9ecad2043ea4`. The review covered the full diff, including the Rust lane (rust-review). `Manifest::all_archetypes` now skips any `archetypes:` row for which the new `Archetype::declares_validation_part` returns false. That predicate is true when the row has `frontmatter_schema_ref`, `data_schema`, `body_extraction` or `grammar_ref`.

Every field of `Archetype` and how the loader, registry and validator use it:

- `name`: the registry key. It is identity, not a validation part.
- `frontmatter_schema_ref`: the frontmatter validator. It is a validation part, and the predicate includes it.
- `data_schema`: the extracted-record validator, including the FR-069 reference form. It is a validation part, and the predicate includes it.
- `body_extraction`: the locator DSL and assert facets. It drives extraction and structural validation, so it is a validation part, and the predicate includes it.
- `extras.grammar_ref`: `validate_document` and `coverage/declarations` use it to run grammar checks. It is a validation part, and the predicate includes it. TC-981 already loads a grammar_ref-only `archetypes:` row.
- `extras.allowed_links`: `Registry::resolve_allowed_links` uses it to build the edge vocabulary. The loader's FR-040 pass reports UnknownEdgeType/UnknownRole from it, and `semantic/relations` reads it for typed relation keys. It constrains documents, and the predicate omits it (FND-002).
- `extras.roles`: `target_satisfies` uses it to resolve role target tokens in other types' `allowed_links`. The predicate omits it (FND-002).
- `extras.defaults.id_pattern`, `extras.has_plugin`, `extras.construct`: FR-031-AC-3/AC-7 carry-over that downstream tools read (filament-core-data reads `construct()`). The predicate omits them (FND-002).
- Retired `extras.required_sections` / `variants` / `template_ref`: FR-031-AC-5 says any of these must be rejected with ArchetypeLoadFailure. The predicate omits them, so such rows are now silently dropped (FND-001).
- Filament keys (`kind`, `doc_backed`, `supports_membership`, `composition`, `description`, `icon`, `has_service`, `state_schema_ref`): no quire consumer reads them. A row with only these keys is presentation-only.

Other consumers checked:

- `ManifestOrigins` still records `archetypes:` rows. It is first-wins, and `artifact_types` comes first, so this has no effect.
- `read_module_semantic` now passes only the kept names to the `semantic.exports` check. Every real module's exports that overlap presentation names are also `artifact_types`, so nothing breaks.
- filament-core-service (origin/main) reads `manifest["archetypes"]` straight from the YAML dict in `module_service.py`. `quire.validate_manifest` is plain JSON-Schema validation and never goes through `all_archetypes`. That consumer is unaffected.

## Verdict

**FAIL (fixable in-PR)**. The fix is right for the real corpus. Built from this PR, the fresh origin/main spec-artifacts-process plus spec-artifacts-iso module set gives 0 DuplicateArchetype diagnostics, against 5 on the origin/main loader (ADR, Plan, Review, SpecReview, Standard). `NoSuchType`, `Test Matrix` and `Spec` are refused as unknown types, and ADR and the other real types still validate. One high defect remains. The predicate is an allowlist of four validation parts, so an `archetypes:` row with a retired field and no validation part is now silently dropped, where the FR-031-AC-5 no-compat rule requires a refusal. Rows that carry only carry-over semantics are also silently dropped (FND-002). The CHANGELOG calls the change a behaviour change but not breaking, although real documents that validated before now fail (FND-004).

Suggested fix for FND-001 and FND-002: treat a row as presentation-only only when it declares no quire-owned key at all. That is, drop it only when none of the validation parts, the carry-over keys or the retired keys is present. A retired-field row then still reaches `compile_archetype` and is refused. Add a test that an `archetypes:` row carrying `required_sections` still yields an ArchetypeLoadFailure.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | An `archetypes:` row that declares a retired field (`required_sections`, `variants`, `template_ref`) and no validation part is now silently dropped instead of refused, which violates FR-031-AC-5's no-compat rule. Reproduced: a probe module with `archetypes: [{name: Retired, required_sections: [...]}, {name: Tmpl, template_ref: ...}]` gives 2 ArchetypeLoadFailures on the origin/main loader and 0 on this PR. | src/loader/manifest.rs:435-446, src/loader/manifest.rs:493-498 |
| FND-002 | medium | `declares_validation_part` leaves out the FR-031 carry-over keys that still constrain or describe a type: `allowed_links` (edge vocabulary), `roles` (role-target resolution), `defaults.id_pattern`, `has_plugin` and `construct`. An `archetypes:` row carrying only these was a registered type before; it is now silently gone. Reproduced with LinksOnly/RolesOnly/IdOnly/ConstructOnly rows: registered on main, unknown on this PR. No current ecosystem module has such a row. | src/loader/manifest.rs:493-498 |
| FND-003 | low | The new doc comment on `all_archetypes` cites FR-013-AC-16, which is the closed-module-set criterion. The criterion it implements is FR-013-AC-21. | src/loader/manifest.rs:430-433 |
| FND-004 | medium | The CHANGELOG entry is labelled "Behaviour change", but documents that validated before are now refused as unknown type. The same section marks such changes **Breaking** (see PLAT-214). Real affected documents include quire-wasm/spec/spec.md and filament-view-standard/spec/spec.md (`type: Spec`, an iso presentation-only row). Further affected types are `Test Matrix`, `Application Spec` and the five engineering-assurance display names. | CHANGELOG.md:12 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| `Manifest::all_archetypes` filter | examined | FND-001, FND-002 |
| `Archetype::declares_validation_part` against every `Archetype` field and consumer | examined | FND-001, FND-002 |
| Same-name `archetypes:` row WITH a validation part | examined | Still DuplicateArchetype `[m, m]`, first-wins `artifact_types`. A conflicting second contract in one module should be reported, so this is correct; the spec gap is recorded in the spec-review SR |
| filament-core-service `archetypes:` consumer | examined | Unaffected; it reads the YAML directly |
| `read_module_semantic` export check | examined | Clean for the real corpus |
| TC-2211 / TC-2212 test code | examined | Both fail on the origin/main loader and pass on this PR (verified) |
| CHANGELOG entry | examined | FND-004 |

## Verification

- `cargo test --locked --lib -- loader:: registry::`: 93 passed.
- `cargo test --locked --test it -- tc822 advisory_run_does_not_change vocabulary plat_validation_batch`: 42 passed.
- With `src/loader/manifest.rs` reverted to origin/main, TC-2211 fails (`DuplicateArchetype { name: "ADR", modules: ["only", "only"] }`) and TC-2212 fails (`Board` registered).
- A scratch probe example run over the fresh origin/main spec-artifacts-process (b935498) and spec-artifacts-iso (a6c09a2) gives 0 DuplicateArchetype on the PR and 5 on main.
- `spec_validate` (`make validate`) with those fresh roots: 199 documents, 0 failed.
- Builds used `locked-build.sh` with `CARGO_PROFILE_DEV_DEBUG=0`. The full aggregate gate was not run by this reviewer.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (25a0495): `all_archetypes` filters on `declares_quire_key`, which includes the retired fields, so a retired-field row reaches `compile_archetype` and is refused. Probe re-run: `Retired` (required_sections) and `Tmpl` (template_ref) each give an ArchetypeLoadFailure. TC-2213 fails on the round-0 predicate. |
| FND-002 | fixed | Round 1 (25a0495): `declares_quire_key` covers `allowed_links`, `roles`, `has_plugin`, `construct` and `defaults.id_pattern`. Probe re-run: the LinksOnly, RolesOnly, IdOnly, ConstructOnly and GrammarOnly rows stay registered; the kind/doc_backed-only row is dropped. |
| FND-003 | fixed | Round 1 (25a0495): the doc comment now cites FR-013-AC-21. |
| FND-004 | fixed | Round 1 (25a0495): the CHANGELOG entry is labelled **Breaking** and names the affected documents (`type: Spec` in quire-wasm and filament-view-standard, Test Matrix, Application Spec, the EA display names). |
