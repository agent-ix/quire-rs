---
id: SR-20261112163908
title: "spec-review of FR-083 clause-set publishability (quire-rs PR 657)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@b5651325; spec/functional/FR-083-clause-set-publishability.md; spec/functional/FR-073-generic-clause-sets.md; spec/functional/index.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-083
    type: references
  - target: ix://agent-ix/quire-rs/FR-073
    type: references
---

# Spec review of FR-083 clause-set publishability (quire-rs PR 657)

## Summary

Ticket: PLAT-1419. One spec-review pass over `git diff origin/main...b5651325` (integrity, EARS, criterion-strength and failure-domain checks done by hand; Jev not used). `quire validate` on the four changed files is clean apart from an unrelated module warning (`semantic.unused-import` in spec-artifacts-app). TC-2313..TC-2316 are new, unique and follow TC-2312. Checked against code: `ClauseSet::validate` (src/clauses.rs:185-222), `LoadedModule` (src/loader/mod.rs:50-60), `Manifest.clause_sets` (src/loader/manifest.rs:338), and the embedded manifest schema (src/semantic/embedded.rs:36, src/semantic/contract.rs:229). No real publication names or text and no roadmap content found.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The restricted rule keys on structure rights, but the hazard is licensed text. To load clause text, `validate` requires `text: original` or `explicitly_cleared`, so a restricted set can only be reached by declaring licensed text "original" or "cleared" (with a clearance_ref). A truthfully declared licensed set cannot load. A licensed set marked `structure: original` or `explicitly_cleared` is publishable. Cleared text on citation-only structure is refused, which contradicts "explicitly cleared". The gate is not vacuous mechanically, but it relies on a misdeclaration. Fix: add an enum value `TextRights::Licensed` (a new value, not a new field) that loads only into local use, and make restricted mean `text: licensed`. Otherwise, state the intended declaration for licensed text and justify the `explicitly_cleared` case. | spec/functional/FR-083-clause-set-publishability.md:22-35 | wrong-requirement |
| FND-002 | medium | `RestrictedClauseSet.path` (AC-4, AC-6 ordering) cannot be produced by a pure function of `&LoadedModule`. `LoadedModule.clause_sets` is `Vec<Arc<ClauseSet>>`, and neither it nor `ClauseSet` keeps the manifest-relative path. Fix: state that the loader keeps each set's manifest-relative path on `LoadedModule`, or drop `path`. | spec/functional/FR-083-clause-set-publishability.md:40-46 | missing-requirement |
| FND-003 | medium | AC-1 asks "the module-manifest schema" to declare `clause_sets`. quire-rs owns no top-level manifest schema. The embedded one comes from filament-core-data (`agent-ix-semantic-schema`) and validates only the `semantic` block. A non-list `clause_sets` already fails the serde parse (`Vec<PathBuf>`, `deny_unknown_fields`). As written, AC-1 is either another repo's work or already true. Fix: reword AC-1 as "manifest load rejects a non-list `clause_sets` with a load error naming the key", and move the schema entry to a filament-core-data ticket. | spec/functional/FR-083-clause-set-publishability.md:53 | wrong-requirement |
| FND-004 | medium | AC-8 says "opens no file and no socket". A unit test cannot observe that as stated. Fix: give an oracle, for example "after loading, the module directory is deleted, and `check_publishable` still returns the same vector". | spec/functional/FR-083-clause-set-publishability.md:60 | wrong-requirement |
| FND-005 | low | The publishable bullet list omits `structure: citation_only` with `text: original` or `text: explicitly_cleared` and no clause text present. It is publishable under "every other", but AC-3 has no vector for it. Fix: add the bullet and the vector. | spec/functional/FR-083-clause-set-publishability.md:24-28 | missing-requirement |
| FND-006 | low | AC-7 tests the "serialized form", but the spec never says `RestrictedClauseSet` is `Serialize`. Fix: state the derive, or drop "serialized form". | spec/functional/FR-083-clause-set-publishability.md:59 | wrong-requirement |
| FND-007 | low | The FR-083 coverage row is inserted between FR-077 and FR-078. The new TC rows are inserted between TC-2233 and TC-2240. Both are out of order. Fix: move them to their sorted positions. | spec/tests.md:117,1217-1220 | wrong-requirement |

## Verdict

Not mergeable: FND-001 (high) blocks the PR. FND-002, FND-003 and FND-004 (medium) are to be fixed in the same round. The purity statement, the no-clause-text output, ordering and determinism (AC-6), and the tests.md AC to TC mapping are otherwise sound. Every AC except AC-8 has a failing oracle.
