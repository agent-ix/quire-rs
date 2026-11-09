---
id: SR-16420
title: "spec-review (base) of CR-215 only marker forms bind, one advisory (quire-rs PR 623)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@9901e1f3; spec/functional/FR-050-declarative-coverage-computation.md; spec/functional/FR-051-source-symbol-extraction.md; spec/functional/FR-068-source-grounded-assurance-projection.md; spec/functional/FR-077-trace-search-index.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
  - target: ix://agent-ix/quire-rs/FR-050
    type: references
---

# Spec review (base) of CR-215 (quire-rs PR 623)

## Summary

Ticket: PLAT-847. Base checklist over `git diff origin/main...9901e1f3` (two commits, spec-only, 5 files). Each reworded AC was compared with its origin/main text: FR-051 narrative, CON-3, new CON-4, AC-6, AC-11, AC-16, AC-19, AC-22, AC-24, AC-28, AC-42, AC-43, new AC-49 and AC-50; FR-050-AC-44 (replaced), AC-50 and AC-62 (trimmed); FR-068 and FR-077 edits. Decided rulings were not re-litigated: one advisory regex rather than moving the code into a module, no deprecation window, retag before deletion, the coverage delta published with the change, and the separate `/// Implements:` warning. Assurance profile AP-201 (`spec/assurance/AP-201-detection-minting.md`, review_policy `require` / `spec-review`) applies, because the change alters trace binding. The enforced ix-flow run needs a human acknowledgement, which a reviewer subagent cannot give, so this is a direct review and that run remains open. Of three findings rated high, two are contradictions between requirements: CON-3 against the retained `implements` list, and CON-4's release order against AC-43. The third is AC-49 suggesting a `#[trace]` rewrite on symbols that cannot bind it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-051-CON-3 and the narrative ("Text that is not a marker form binds nothing") are unscoped: "the module `trace_tags` schema ... SHALL carry no field, variant, list, template or branch for any other trace-text form" contradicts the retained FR-062 `trace_tags.implements` textual forms (`rust-implements-line`, and `python-implements-line`, the only Python implements form) and FR-050-AC-62, which keeps a no-`syntax` implements use binding. Scope CON-3 to `verifies` binding (`trace_tags.markers`, `VerifiesRelation`, the census) | spec/functional/FR-051-source-symbol-extraction.md:188 |
| FND-002 | high | FR-051-CON-4 says the engine release that stops reading non-marker forms precedes the module release that removes them, but AC-43 has the engine reject a `trace_tags.legacy` key (and `TraceTagGrammar` is `deny_unknown_fields`), so the new engine refuses to load every consumer's current spec-artifacts-process module during that window. The module release must come first (an old engine loads a module with no `legacy` list) or ship in the same release | spec/functional/FR-051-source-symbol-extraction.md:189 |
| FND-003 | high | FR-051-AC-49 fires on a comment id attached to any declaration (AC-22 routes modules, containers and production functions to it) and renders "the exact text to write instead" from the first `markers` template (`#[trace("...")]`). FR-062 says a `trace` marker on a production function binds nothing, so following the advice clears the warning (a marker is not a comment) and backs nothing. Production doc-comment citations, which FR-077 treats as mentions, become a worklist that never empties. Limit the replacement text to evidence-kind symbols; for non-binding kinds keep AC-22's report naming the channel the kind can carry | spec/functional/FR-051-source-symbol-extraction.md:241 |
| FND-004 | medium | FR-051-AC-28 (and FR-050-AC-50) narrowed "any source trace tag, marker or legacy alike" to "a marker's argument". The retained no-`syntax` implements textual form (`/// Implements: FR-001..FR-003`, `# Implements:`) is not a marker under the new wording, so the range rule for it is now unspecified, while `bind_implements` (trace.rs:1278) applies `marker_range` to it today. TC-1936 covers only `#[implements(...)]` | spec/functional/FR-051-source-symbol-extraction.md:222 |
| FND-005 | medium | FR-051-AC-22 dropped the explicit rule that a canonical marker on a non-binding symbol is not reported, and TC-1047, the only test of that negative control, was removed. The narrative says the advisory is "the generic id-shaped-token detection that already feeds `unmatched_tags`", and that detection (`generic_tags`, trace.rs:1178) reads every line from `leading_line` to the declaration, attribute and decorator lines included, so a straight reuse reports `#[trace("X")]` on a production function. Restate the exclusion and keep a control test | spec/functional/FR-051-source-symbol-extraction.md:215 |
| FND-006 | medium | FR-066's `marker-form-mismatch` disposition was left untouched (author decision 2). After CR-215, every row backed only by a comment or docstring id (192 in quire-rs per PLAT-912) classifies there, with owner "module declaration" and evidence "legacy comma-list forms". The module grammar is correct and the owner is the repository (retag), so the census sends the work to the wrong owner. Update the owner and the evidence text, or route these rows to `authoring-absent` | spec/functional/FR-066-gap-disposition-census.md:47 |
| FND-007 | low | FR-062's rationale "`ImplementsRelation` carries no `provenance`, because there is no legacy `implements` form to migrate from" contradicts FR-050-AC-62 ("a legacy textual implements use") and is stale now that `VerifiesRelation` drops provenance too. Reword it to: neither relation carries provenance (FR-051-CON-3) (author question 3) | spec/functional/FR-062-implements-relation.md:58 |
| FND-008 | low | FR-051-AC-43 names `trace_tags.legacy` in a canonical AC ("rejects a `trace_tags.legacy` key as unknown, naming it"). `deny_unknown_fields` already rejects any undeclared key and names it, so the clause can be stated generically and the canonical design keeps no mention of legacy forms (containment rule 2) | spec/functional/FR-051-source-symbol-extraction.md:237 |

## Verdict

Not mergeable until FND-001..003 are fixed. Fixing FND-004..006 in the same round is recommended.

Confirmed clean:
- AC-6 and AC-16 are still testable. AC-16 now overlaps AC-4 but still fails on a multi-id marker that binds only its first id.
- AC-11/TC-2160 is a strict negative test that can fail, and AC-19 keeps its invariant.
- AC-24 now protects the advisory instead of binding, keeping byte and line preservation.
- AC-42 and AC-43 lose only clauses tied to the removed machinery.
- FR-050-AC-50 and FR-050-AC-62 lose only the legacy endpoint case and the `RewriteSuggestion` clause (FND-004 aside).
- FR-050-AC-44 is replaced by a negative AC that can fail: current code emits `marker-form-mismatch` for a name subpopulation.

The CON-3 wording otherwise passes containment rule 2 (no legacy named), and the CR-215 note records the measurement and the ruling.

FR-051-AC-49/50 satisfy the brief's points:
- severity `warning`, reason `trace-id-outside-marker`
- one finding per occurrence
- silent when the language declares no marker form
- fires whether or not the id is also backed
- not for body or non-annotation string ids

Author decisions:
1. No advisory for an id embedded in a test name (including a TS registration title, now masked by AC-24): acceptable, because detection needs the removed reconstruction and CON-4 retags the three in-scope repositories first. The published coverage delta should list symbols tagged only through their name.
2. FR-066 untouched: not acceptable as-is (FND-006).
3. FR-062:58: flagged (FND-007).

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | medium | FR-051-AC-22 (fixed for FND-003) does not say which template is named when a kind "can carry an `implements` form that declares a `template`". spec-artifacts-process main declares `rust-implements-line` (template `/// Implements: {ids}`) before `rust-implements-attribute` (`syntax: rust_attribute`). Taking the first declared form tells a Rust author to write the textual form, which FR-050-AC-62 then warns about as `legacy-implements-form`. Name the same form FR-050-AC-62 renders (the first `syntax: rust_attribute` form with a template), and have TC-2165 assert the implements text | spec/functional/FR-051-source-symbol-extraction.md:217 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 4d4877e: CON-3 is scoped to `verifies`/`trace_tags.markers` and the census, and excludes the FR-062 implements textual forms; the narrative says the same |
| FND-002 | fixed | 4d4877e: CON-4 puts the module release before, or with, the engine release, and explains why via AC-43 |
| FND-003 | fixed | 4d4877e: AC-49 renders `#[trace]` text only for evidence kinds; AC-22 gives non-binding kinds no `#[trace]` text (residual template choice in FND-009) |
| FND-004 | fixed | 4d4877e: AC-28 and FR-050-AC-50 cover a range in a textual `implements` use; TC-2166 added |
| FND-005 | fixed | 4d4877e: AC-22 states that a canonical marker on a non-binding symbol is not reported (comment and docstring lines only); TC-2165 replaces TC-1047 |
| FND-006 | fixed | 4d4877e: FR-066 disposition 3 and the evidence table name the repository (retag) as owner, and the module only for an undeclared marker form |
| FND-007 | fixed | 4d4877e: FR-062 now reads "Neither `ImplementsRelation` nor `VerifiesRelation` carries a `provenance` field ... (FR-051-CON-3)" |
| FND-008 | fixed | 4d4877e: AC-43 rejects "any undeclared `trace_tags` key, naming it" |
| FND-009 | fixed | 8952e9a: AC-22 names the text rendered, as FR-050-AC-62 renders it, from the first `syntax: rust_attribute` implements form with a template (never the textual form), or the kind alone when none declares one; TC-2165 asserts it |
