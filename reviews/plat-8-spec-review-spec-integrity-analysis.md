---
id: SR-16309
title: "spec-review (integrity) of the FR-075 single population extent (quire-rs PR 591)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@62c592a9; spec/functional/FR-075-model-feature-extraction.md; spec/tests.md; schemas/output/semantic-v1.schema.json"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-075
    type: references
---

# Spec review (integrity) of the FR-075 single population extent (quire-rs PR 591)

## Summary

Ticket: PLAT-8. Scoped to the PR's FR-075, `spec/tests.md` and schema
edits. `quire validate` on FR-075 and `spec/tests.md` exits 0.

Examined, clean:

- The new shape matches QSpec. In quire-specification, FR-153 says "the
  population declaration ... lists its member types and its extent", with
  object closure holding exactly when the extent is `closed`. FR-208 defines
  `quire.meaning.model.population/v1` as "a membership view over listed
  member types with a `closed` or `open` extent". FCD semantic-ir already
  checks one population-level `extent` in {closed, open}. The CR note's
  citations are correct.
- QSpec does not say where the extent is authored. Frontmatter `extent:` is a
  sound choice for one scalar per artifact, and it matches `abstract:`.
- The Inputs row, the Behavior rule, AC-4 and TC-1843 agree with each other
  and with the schema.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [untestable-ac] "A `Type \| Extent` table is not a population table" states no observable outcome: no code, no locus, no `availability`. No AC covers it. The implementation's outcome depends on the manifest: refused under a `[Type]` manifest, dropped silently under a `[Type, Extent]` manifest (SR-16307 FND-001). Fix: state the refusal, `semantic.feature-not-extractable` at the header line with `reason` `population`, whatever the manifest declares, and add it to AC-4 or AC-6. | spec/functional/FR-075-model-feature-extraction.md:244-248 | wrong-requirement |
| FND-002 | medium | [ambiguous] FR-075 does not say what happens to frontmatter `extent` on an artifact whose object type declares no population table. AC-6 reads as requiring `semantic.feature-not-extractable` for any feature the manifest does not declare. The code instead ignores a valid value and reports an invalid one as `semantic.invalid-model-cell` without making `model` unavailable. Fix: add a rule that `extent` is a population feature, declared by the population table locator and refused in section `frontmatter` otherwise, and extend AC-6 to cover it. | spec/functional/FR-075-model-feature-extraction.md:118, 244-248, 422 | wrong-requirement |
| FND-003 | low | [other] The CR note blockquote sits inside the "Tables:" bullet list and splits it in two. The rules after it ("A second entry with the same ...") render as a separate list. Fix: move the note below the list. | spec/functional/FR-075-model-feature-extraction.md:250-253 | wrong-requirement |

## Verdict

The shape is right and matches QSpec and FCD. FND-001 and FND-002 should be
fixed with the code findings, so that the spec states the refusal the code
then enforces.
