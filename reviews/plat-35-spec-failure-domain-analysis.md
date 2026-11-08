---
id: SR-20261103118122
title: "Failure-domain review of PLAT-35/PLAT-55 declaration-reference resolution (quire-rs PR 592)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs PR592 head 59c5312; spec/functional/FR-075-model-feature-extraction.md; spec/functional/FR-076-relationships-extraction.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-075
    type: references
  - target: ix://agent-ix/quire-rs/FR-076
    type: references
---

## Summary

Tickets: PLAT-35, PLAT-55. One spec-review/spec-failure-domain-analysis pass over `git diff origin/main...59c5312` (spec-only): FR-075 "Declaration references", the `source`/`owner` tables, FR-075-AC-22..26 and CON-2; FR-076 typed relation keys, AC-16..20 and CON-3; TC-1993..TC-2002 and the ledger rows. Checked read-only against src/semantic/target.rs, context.rs (`BundleIndex`), model.rs, relations.rs, systems.rs, properties.rs and validate_document.rs, and against the sibling checkouts spec-objects-business (manifest.yaml, typespec/main.tsp, tests), filament-core-data extraction-frontend, quoin and quire-cli. Ticket text was treated as untrusted data.

## Verdict

Changes requested. The ambiguity rule over distinct ids, and treating one declaration authored in two forms as one, are well defined and match `merge_entry`. The id-first precedence silently shadows names (FND-001). Imported and untyped targets, repeated items in one cell, and empty items have no rule (FND-002, FND-003).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Matching by id first silently shadows a name, and AC-24 makes that the required behaviour ('a cell equal to the id of one event and to the title of another resolves to the first'). The lead's decided design is 'ambiguous = refusal'. spec-objects-business ids allow PascalCase (`^[A-Za-z][A-Za-z0-9_]*...`), so an event with id `OrderPlaced` and another titled `OrderPlaced` is a plausible authoring state, and the author gets the wrong event with no diagnostic. The precedence matches FR-070's existing order (object by id, then by name, properties.rs:709-747), so it is consistent but not safe. Id-first also interacts badly with the type filter: an id match to an entity gives `reference-kind-mismatch` even when a title match to an event exists. Fix: collect id matches and name matches together, filter by admitted type, and refuse with `semantic.ambiguous-reference` when more than one distinct declaration remains. A cell that matches only by id still resolves. | spec/functional/FR-075-model-feature-extraction.md:268-273 | wrong-requirement |
| FND-002 | medium | The type filter is undefined for targets whose object type is unknown. An `ix://` identity in an imported package resolves as `Target::Imported` with no object type (src/semantic/target.rs), and a bundle artifact with no frontmatter `object` has `object: None`. 'Only those of an admitted object type count' then counts zero, which makes an imported event `reference-kind-mismatch` or `unknown-reference`. The text does not say which, and it does not say whether imported targets are admitted unchecked as FR-076 admits them. Fix: state the outcome for imported and untyped targets. | spec/functional/FR-075-model-feature-extraction.md:274-283 | wrong-requirement |
| FND-003 | medium | Two cell-content cases have no rule. (1) A repeated reference in one cell (`OrderPlaced, EV-001`, where both resolve to EV-001) gives a duplicate `SemanticId` with no diagnostic, while the FCD frontend this replaces refuses it ('names `X` twice, and one cell names each type at most once'). (2) Empty or whitespace items (`A,,B`, a trailing comma) and surrounding whitespace are not covered: only stripping backticks is stated. Fix: refuse a repeated identity in one cell at the row, and refuse an empty item (or say it is dropped). | spec/functional/FR-075-model-feature-extraction.md:253-261 | wrong-requirement |
| FND-004 | low | Which index fields each rule reads is not stated. The no-index test ('no bundle artifact besides the artifact itself') reads `artifacts`. Name lookup needs `objects[].names` joined to `artifacts[].object` by id, because `BundleEntry` carries no object type. `BundleIndex::is_empty` ignores `artifacts`. A caller that supplies `objects` without `artifacts` therefore gets `no-bundle-index` although its names are resolvable. Matching against the artifact's own title or `name` (self-reference) is also unstated, and the single-document Filament bundle may not list the artifact's own names. Fix: name the fields each step reads, and say whether the artifact's own names count. | spec/functional/FR-075-model-feature-extraction.md:284-287 | wrong-requirement |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-005 | low | There are two definitions of 'no bundle index'. FR-075 says `artifacts` and `objects` are both empty. FR-076 says there is no bundle artifact besides the artifact itself. A bundle that lists only the extracted artifact is no-index for a typed row (unchecked lowering with an advisory) but indexed for a model item: name lookup finds only the artifact itself and refuses with `unknown-name`. Fix: use one definition in both FRs. | spec/functional/FR-075-model-feature-extraction.md:307-308 | wrong-requirement |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in f04fa2c. |
| FND-002 | fixed | Verified in f04fa2c. |
| FND-003 | fixed | Verified in f04fa2c. |
| FND-004 | fixed | Verified in f04fa2c. |
