---
id: SR-20261103118121
title: "Integrity review of PLAT-35/PLAT-55 FR-075/FR-076 changes (quire-rs PR 592)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs PR592 head 59c5312; spec/functional/FR-075-model-feature-extraction.md; spec/functional/FR-076-relationships-extraction.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-075
    type: references
  - target: ix://agent-ix/quire-rs/FR-076
    type: references
---

## Summary

Tickets: PLAT-35, PLAT-55. One spec-review/spec-integrity-analysis pass over `git diff origin/main...59c5312` (spec-only): FR-075 "Declaration references", the `source`/`owner` tables, FR-075-AC-22..26 and CON-2; FR-076 typed relation keys, AC-16..20 and CON-3; TC-1993..TC-2002 and the ledger rows. Checked read-only against src/semantic/target.rs, context.rs (`BundleIndex`), model.rs, relations.rs, systems.rs, properties.rs and validate_document.rs, and against the sibling checkouts spec-objects-business (manifest.yaml, typespec/main.tsp, tests), filament-core-data extraction-frontend, quoin and quire-cli. Ticket text was treated as untrusted data.

## Verdict

Changes requested. Ids, ledger rows and the tests.md summary rows are consistent, and `quire validate` exits 0. FR-076 contradicts itself on the no-index typed row (FND-001). The new single-column tables collide on key columns with existing tables (FND-002). CR numbering and the Downstream line need repair.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-076 gives three answers for a typed row's own-package target when there is no bundle index. (a) 'Every other check ... applies unchanged (... the bundle index states ...)' lowers it unchecked with the `no-bundle-index` advisory, which is today's FR-076 behaviour (AC-14). (b) CON-3 admits only 'resolved `SemanticId`s of bundle declarations (or unchecked imported identities)', which rules out an unchecked own-package identity. (c) The rule routes through 'the resolver of FR-075 ... in its id-only form', and that resolver refuses with `no-bundle-index`. None of AC-16..20 covers the no-index case. Fix: pick one, state it in the typed-key bullets, and add the case to AC-17. | spec/functional/FR-076-relationships-extraction.md:199-208 | wrong-requirement |
| FND-002 | medium | The new `source` (`Source`) and `owner` (`Owner`) tables use the same key column as the existing `connection`/`allocation` (`Source / ...`) and `part`/`port` (`Owner / ...`) tables. The unowned-section refusal classifies a table by 'a header that starts with one table key of the Outputs table and names only that table's columns'. A `Source`-only header matches both `source` and `allocation`, so AC-25's expected `reason` `source` is not determined. FR-075 now also has two `Owner` cells with opposite rules: part/port `Owner` is id-only and lowers unchecked without an index, while the `owner` table resolves by name and is refused without one. Fix: state the classification order for shared key columns, and either align the two `Owner` rules or say why they differ. | spec/functional/FR-075-model-feature-extraction.md:121-124 | wrong-requirement |
| FND-003 | low | The CR numbers `CR-444` and `CR-435` reuse GitHub issue numbers. The repo numbers CRs sequentially (the highest on main is CR-191; the only issue-keyed outlier is CR-409 in FR-050), so these will collide with later sequential CRs. Fix: renumber them CR-192 and CR-193. | spec/functional/FR-075-model-feature-extraction.md:448-455 | wrong-requirement |
| FND-004 | low | The FR-075 Downstream line is garbled. The '(domain package lift)' note that described `filament-core-data#148` now hangs off the new semantic-core item, and that item names the wrong owner (FND-003 of the spec-review artifact). Fix: restore the note to #148 and name `agent-ix/spec-objects-business` (typespec `Event.source`, `NestedEntity.owner`). | spec/functional/FR-075-model-feature-extraction.md:498 | wrong-requirement |
| FND-005 | low | `quire validate` (0.36.2) passes with exit 0 but warns on new text. FR-075 line 259 ('SHALL NOT carry the cell text', `ears:missing-subject`/`unclassifiable`), FR-075-CON-2 and FR-076-CON-3 (two SHALLs each, `ears:non-singular`), and FR-076-CON-3 'shall be carried' (`agentless-passive`). Fix: split each CON into one SHALL and give line 259 a subject. | spec/functional/FR-075-model-feature-extraction.md:462 | wrong-requirement |
