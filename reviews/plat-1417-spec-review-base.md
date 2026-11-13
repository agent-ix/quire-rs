---
id: SR-20261113045746
title: "spec-review (base) of FR-086 design-element join and FR-058 CR-416 (quire-rs PR 668)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@8f9a5b9; spec/functional/FR-086-design-element-join.md; spec/functional/FR-058-upward-trace-completeness.md; spec/functional/index.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-086
    type: references
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
---

# Spec review (base) of FR-086 and FR-058 CR-416 (quire-rs PR 668)

## Summary

Ticket: PLAT-1417. Base checklist over `git diff origin/main...8f9a5b9` (3 files: new FR-086, FR-058 CR-416 section with AC-2 amended and AC-23/AC-24 added, index entry). `quire validate --scope .` on the changed files raises no error; the two `ac:non-canonical-shape` warnings are on the existing FR-058-AC-10/AC-11. Ids FR-086-AC-1..14, FR-086-CON-1..2 and FR-058-AC-23..24 are well-formed and unique. Assurance profile `spec/assurance/AP-201-detection-minting.md` requires `spec-review`; this run is a direct review (no ix-flow run). Clean on: engine holds no archetype, verb or inverse name (FR-086 text, AC-12, AC-13); no compatibility layer, fallback or deprecation window; no paywalled standard text; no hash, digest, pin or tracking record. Findings below are AC-level testability defects; cross-FR consistency findings are in SR-20261113045747 and statement-grammar findings in SR-20261113045748.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-086-AC-6: with member `o/b` not loaded, an edge authored on the design element in `o/b` does not exist in the corpus at all, so nothing can be "in `dangling` as authored". The dangling half only holds if the requirement in `o/a` authors the inverse toward the element, and then it also needs a non-empty `to` (with `to: []` a dangling edge satisfies the relation and the row is `covered`, FR-058/FR-084-AC-16). Fix: state which end authors the edge in each half and that `to` is non-empty. | spec/functional/FR-086-design-element-join.md (AC-6) | wrong-requirement |
| FND-002 | medium | FR-086-AC-10 says the join errors naming an entry that names no declared relation. The Description says only that such an entry reports `trace:undeclared-relation-vocabulary`, and justifies not failing because the relation may come from another module. Failing the whole join in that case is stated only in the AC and works against that reasoning. Fix: either state the join error in the body or have the join skip the entry and leave it to the finding. | spec/functional/FR-086-design-element-join.md (AC-10) | wrong-requirement |
| FND-003 | medium | FR-086-AC-5 and FR-058-AC-24 say a pair authored both ways gives one `linked` entry. But FR-084 `linked` entries carry `verb` as authored (the CR-416 note keeps this), and relation_report.rs keys entries on (repo, id, verb), so `V` and `W` are two entries. Which verb the single entry carries is not stated. Fix: name the verb (the listed `V`, for example) or allow one entry per authored verb. | spec/functional/FR-086-design-element-join.md (AC-5); src/corpus/relation_report.rs:474-481 | missing-requirement |
