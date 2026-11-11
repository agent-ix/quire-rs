---
id: SR-20273
title: "spec-review of the PLAT-1393 note in FR-051 (quire-rs PR 643)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@3837b815; spec/functional/FR-051-source-symbol-extraction.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
---

# Spec review of the PLAT-1393 note in FR-051 (quire-rs PR 643)

## Summary

Ticket: PLAT-1393. Base spec-review of the one spec hunk, which adds a blockquote 'PLAT-1393 note' after the CR-211 note. Sub-analyses: none beyond base. The hunk adds no requirement statement, no relationship and no domain object. Under the repo CLAUDE.md, 'a spec error or change requires a CR note (see CR-002 in FR-024 for the pattern)'. CR-002 rewrites the normative text and explains the change in a note. The highest CR id in `spec/` is CR-409.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The change is not a CR. It is filed as a ticket-labelled note, and the AC rows stay as they were. It also reverses the CR-206 note's ruling ('keep identity and report the collision instead of resolving it') without naming that reversal. Fix: allocate the next free CR id and rewrite the AC-32 chain to inherent-impl target, or `<Type as Trait>` for a trait impl, with the normalisation rule. Remove the `From<A>`/`From<B>` cause and 'Symbol ids are unchanged' from AC-31. Turn the note into a `CR-NNN note (PLAT-1393)` that names the CR-206 reversal. | spec/functional/FR-051-source-symbol-extraction.md:316-322 | wrong-requirement |
| FND-002 | low | 'ix-trace-rs FR-005 must follow the same rule for a `#[trace]` test declared inside a trait impl' creates a cross-repo obligation that can never apply. rustc rejects `#[test]` on an associated function, so no test is ever declared inside a trait impl. Fix: drop the sentence, or say that only production symbols are re-keyed. | spec/functional/FR-051-source-symbol-extraction.md:321-322 | wrong-requirement |

## Verdict

Not mergeable: FND-001 (medium) is to be fixed in the same round as SR-20272 FND-001. FND-002 is low.
