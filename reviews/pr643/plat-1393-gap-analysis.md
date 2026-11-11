---
id: SR-20272
title: "gap-analysis of trait-qualified Rust symbol ids against FR-051 (quire-rs PR 643)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@3837b815; spec/functional/FR-051-source-symbol-extraction.md; src/symbols/rust.rs; tests/it/implements_relation.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
---

# Gap analysis of trait-qualified Rust symbol ids (quire-rs PR 643)

## Summary

Ticket: PLAT-1393. Planless gap analysis scoped to the PR diff. Plan completion: not assessed. Criteria examined: FR-051-AC-31 and FR-051-AC-32, which the new test `trait_impl_methods_get_trait_qualified_ids` traces, and FR-051-AC-1, which `impl_blocks_are_scopes_not_symbols` traces. The test fails on pre-fix code, and its cfg control case still asserts a genuine collision. No stub or coverage inflation was found. The defect: the code now contradicts the normative text of AC-31 and AC-32, which are unchanged.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The code violates FR-051-AC-32 and FR-051-AC-31 as written. AC-32 says the qualified-name chain uses 'the `impl` target type (never the trait)'. AC-31 lists 'Rust trait-impl methods differing only by generics (`From<A>` / `From<B>`)' as a collision cause and says 'Symbol ids are unchanged'. The new test is traced to both ACs while asserting `<Halt as From<Finish>>::from`, the opposite of AC-32's text. Only a blockquote note says otherwise. Fix: amend the AC-31 and AC-32 rows themselves (see SR-20273). | spec/functional/FR-051-source-symbol-extraction.md:239-240 | wrong-requirement |

## Verdict

Not mergeable until AC-31 and AC-32 are amended to the behaviour the code implements. The code and the test are otherwise sound.
