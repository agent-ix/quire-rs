---
id: SR-20261103118147
title: "Spec review — FR-013-AC-20/AC-21 presentation rows (quire-rs PR 631)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-013-archetype-loader.md, spec/functional/FR-031-unified-archetype-shape.md, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-013", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-031", type: references}
---

## Summary

Ticket: PLAT-277 (finding F4). Frozen PR head: `91cdf225b0ceaf29646f4f59a88e9ecad2043ea4`.

The review examined FR-013-AC-20, FR-013-AC-21, the TC-2211 and TC-2212 rows, and the two new AC-to-TC mapping rows in spec/tests.md. It read FR-031 (Description, AC-3, AC-5, AC-7) to check the new criteria for consistency with it.

`spec_validate` (`make validate`) over the fresh origin/main spec-artifacts-process and spec-artifacts-iso roots reports 199 documents and 0 failed. The PR's spec edits are therefore structurally valid. The known `make validate` failure on main comes from a stale sibling checkout and does not reproduce with fresh roots.

## Verdict

**FAIL (fixable in-PR).** AC-20 is atomic, testable and states the cross-module control. AC-21 defines "validation part" as a closed list of four fields. That definition contradicts FR-031-AC-5, under which every archetype that declares a retired field is rejected. It also leaves out the FR-031 carry-over parts. The AC-20 text does not say what happens when the same-module `archetypes:` row carries its own validation part, and FR-013 has no CR note for the change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-013-AC-21 says an `archetypes:` entry with none of four named fields is not registered. That contradicts FR-031-AC-5, which says any manifest archetype that declares `required_sections`/`variants`/`template_ref` is rejected with ArchetypeLoadFailure. Under AC-21, such a row is now silently ignored. AC-21 also leaves out the FR-031 carry-over parts (`allowed_links`, `roles`, `defaults.id_pattern`, `has_plugin`, `construct`). | spec/functional/FR-013-archetype-loader.md:248 |
| FND-002 | low | FR-013-AC-20 does not state the outcome when the same module's `archetypes:` row also declares a validation part that differs from its `artifact_types:` entry. Today that gives DuplicateArchetype naming the module twice, and the `artifact_types` entry wins (first-wins). Reporting it is right, but the AC should say so. | spec/functional/FR-013-archetype-loader.md:247 |
| FND-003 | low | FR-013 adds AC-20/AC-21 and changes what the `archetypes:` section means, with no CR note and no Description change. The repo's CLAUDE.md asks for a CR note on any spec change, and AC-18/AC-19 got one. FR-031's Description ("compile every manifest archetype") is also left unqualified. | spec/functional/FR-013-archetype-loader.md:52-66 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-013-AC-20 | examined | FND-002 |
| FR-013-AC-21 | examined | FND-001 |
| TC-2211 row (spec/tests.md) | examined | Clean |
| TC-2212 row (spec/tests.md) | examined | Clean |
| FR-031 Description, AC-3, AC-5, AC-7 | context_only | Used to establish FND-001 |
| FR-013 CR notes | context_only | FND-003 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (25a0495): FR-013-AC-21 now defines quire-owned keys as validation parts plus FR-031 carry-over keys plus retired fields, and says a retired-field row is still refused (FR-031-AC-5). It no longer contradicts FR-031. |
| FND-002 | fixed | Round 1 (25a0495): FR-013-AC-20 now states that a same-named `archetypes:` row with a quire-owned key emits DuplicateArchetype naming the module twice, with the `artifact_types:`/`object_types:` entry active. |
| FND-003 | fixed | Round 1 (25a0495): CR note "presentation rows are not types — 2026-11-09, PLAT-277 F4" added to FR-013. FR-031's Description is still unqualified, but AC-21 now refers back to FR-031-AC-5, so the two do not conflict. |
