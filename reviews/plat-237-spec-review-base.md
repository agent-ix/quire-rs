---
id: SR-16305
title: "PLAT-237 FR-069 archetype wording spec-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md; PLAT-237; PR588"
review_set: base
---

## Summary

Ticket: PLAT-237. Reviewed PR588 head 8ed9846 (`git diff origin/main...HEAD`). The diff touches one file, FR-069: "object type" is reworded to "archetype" in the Description, Outputs, Behavior (advisory and Filament clauses), the allocation note and FR-069-AC-2, and a ticket-keyed CR note is added under the Description. `quire validate --scope . spec/functional/FR-069-semantic-module-contract-at-load.md` (quire 0.36.2, engine 0.50.2) exits 0 with only pre-existing module warnings.

Loader behaviour matches the new wording. `src/loader/mod.rs` fails every entry of `manifest.all_archetypes()` (which spans `artifact_types`, `object_types` and `archetypes`, `src/loader/manifest.rs:400`) when `read_module_semantic` refuses, on both the filesystem path (`load_one_module`) and the inline path (`load_inline_module`). It compiles per archetype through `compile_archetype` / `resolve_data_schema`, emits `semantic.inline-data-schema` per archetype, and resolves exports against the names of all archetypes. The CR note follows the CR-002 pattern in FR-024 (a bold-labelled blockquote that names the text it replaces, why that text was wrong and what replaced it). Its ticket-keyed label matches the existing `CR note (STD-228, ...)` form in FR-003 and FR-013.

## Assurance Context

AP-201 (spec/assurance/AP-201-detection-minting.md) governs coverage collection, trace binding, row minting, diagnostics and schemas. A wording-only change to the semantic module load contract alters none of them, so AP-201 does not apply. AP-201 also carries only the retired `review_selection` field and no `review_policy`, so it does not require an enforced spec-review run. This review ran directly, with the base checklist.

## Examined Units

- **FR-069** (examined): description, outputs and behavior statements reworded from object type to archetype.
- **FR-069-AC-2** (examined): `contract_version: 2.0.0` fails every archetype of the module with a reason starting `semantic.unsupported-contract-version` and no other `semantic.*` reason.
- **FR-069-AC-3** (context_only): exports resolved across `artifact_types`, `object_types` and `archetypes`.
- **FR-069-AC-6** (context_only): an inline `data_schema` under a `semantic` block warns `semantic.inline-data-schema`.
- **FR-069-AC-7** (context_only): a Filament snapshot with a reference-form `data_schema` is refused and yields no node.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The Filament extraction clause now says "produce no node for that archetype", but its subject is a `FilamentObjectType` snapshot (FR-045), not a manifest archetype from `all_archetypes()`. The CR note's rationale does not cover this occurrence. "That object type" was the accurate Filament term. | spec/functional/FR-069-semantic-module-contract-at-load.md:145-148 |
| FND-002 | low | FR-069-AC-2 now claims every archetype fails, but TC-1600 (`unsupported_versions_are_refused_first`) uses the module-ok fixture with a single `object_types` entry. No test shows an `artifact_types` or `archetypes` entry, or more than one archetype, failing on refusal. The loop in the code is uniform, so the risk is low. | tests/it/semantic_contract.rs:151-174 |

## Verdict

PASS with two low findings. The prose matches loader behaviour, the CR note follows the repository's CR-note pattern, and the document validates. Neither finding blocks merge. FND-001 is a one-word revert. FND-002 is an optional test strengthening.
