---
id: SR-20261113045743
title: "Gap analysis — PLAT-1556 FR-085 cross-repository edge resolution (quire-rs PR 665)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@fb44ef9; spec/functional/FR-085-cross-repository-edge-resolution.md, spec/functional/FR-027-whole-spec-query-api.md, spec/functional/FR-025-spec-corpus-model.md, src/corpus/, tests/it/cross_repo.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-085", type: references}
---

## Summary

Ticket: PLAT-1556. Planless gap analysis of PR head `fb44ef9` against FR-085 and the FR-025/FR-027/FR-058/StR-006 amendments on main (#664). Matrix: `quire matrix --scope . --format json` (quire 0.44.0, engine 0.59.1; quoin 0.30.1). All 21 criteria FR-085-AC-1..21 and FR-027-AC-10 are bound to a test in `tests/it/cross_repo.rs`. FR-025-AC-3 is bound to both its existing unit test and the new test. The #664 amendments to FR-025, FR-058 and StR-006 add no new criteria. No stubs, `todo!()` or placeholder returns. The optional semantic review was not run (opt-in, not requested), but each FR-085 test was read against its AC by hand.

Plan completion: not assessed

## Verdict

**FAIL (one high, two medium, one low).** The high finding is a contradiction in the spec, not in the code. FR-085-AC-16's second clause conflicts with AC-17 and FR-038-AC-9. The code follows AC-17, so the test tagged AC-16 asserts a substitute scenario. AC-21 names a model form that PLAT-1501 deleted. Both need a CR note amending FR-085 (per CLAUDE.md, never a silent spec edit), and this PR is the natural place for it. The relative-link half of the AC-8 test cannot fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-085-AC-16 says the corpus without B raises `dangling-reference` for A's edge into B. AC-17 (and FR-038-AC-9) say an `ix://o/b/...` reference whose authority names no loaded member is not reported. With B absent, `o/b` is exactly such an authority, so the two ACs contradict each other. The code correctly follows AC-17 and does not raise it. The test tagged AC-16 replaces the "without B" clause with "o/b loaded without the hazard", which is AC-17's first clause, so AC-16 as written is not tested. Fix: add a CR note amending AC-16's second clause to "the same corpus with B loaded but not holding the target raises it". | spec/functional/FR-085-cross-repository-edge-resolution.md (AC-16, AC-17), tests/it/cross_repo.rs:594-624 |
| FND-002 | medium | FR-085-AC-21 (and the Bundle-validation bullet "model-declared `document:` paths and `exclude:` globs") names the `document:` trace-target form that PLAT-1501 deleted. A manifest carrying it now fails to load (traceability.rs tc829). As written the AC cannot be satisfied. The tagged test substitutes `exclude:` globs resolved against each member's reference root. That is the right remaining surface, and its crossed-root control does fail. Fix: a CR note restating AC-21 and the bullet in terms of `exclude:` globs. | spec/functional/FR-085-cross-repository-edge-resolution.md (AC-21), tests/it/cross_repo.rs:721-764 |
| FND-003 | medium | The relative-path half of the FR-085-AC-8 test cannot fail. The link `../../b/spec/HAZ-001.md` from `<tmpA>/spec/REQ-001.md` normalizes to `/tmp/b/spec/HAZ-001.md`, but member b lives in a random `<tmpB>` dir. Under a single global path index (the defect AC-8 guards) the link would still not resolve, and `by_link("HAZ-001.md") == Dangling` would still pass. Only the bare-id half exercises member scoping. Fix: put both members under one parent TempDir (`<p>/a`, `<p>/b`) and link `../../b/spec/HAZ-001.md`, so a global index would resolve it. Better still, add a case where both members hold the same root-relative path. | tests/it/cross_repo.rs:352-388 |
| FND-004 | low | Reverse gap: the public `Spec::member_count` and `Spec::member_identity(usize)`, and `CorpusError::RootCount` from `validate_corpus`, have no owning AC. The test `validate_corpus_needs_one_root_pair_per_member` carries no trace tag. Fix: make the accessors `pub(crate)` unless PLAT-1554 needs them, and tag the root-count test to an FR-085 AC (add one stating the mismatch error). | src/corpus/spec.rs:293-300, src/corpus/spec.rs:59-66, tests/it/cross_repo.rs:766-785 |

## Coverage

- FR-085-AC-1..21: 21/21 tagged; bindings read: AC-1..7, 9..15, 17..20 correct; AC-8 partially vacuous (FND-003); AC-16 tests a substitute for its contradictory clause (FND-001); AC-21 tests the surviving `exclude:` form (FND-002).
- FR-027-AC-10, FR-025-AC-3: tagged and correct.
- Reverse gaps: FND-004. Stubs: none.
- Semantic review: skipped (opt-in); manual AC-to-test reading done.
- Plan completion: not assessed

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Commit 148abdc amends FR-085-AC-16 to "the same corpus with B loaded but not holding the target raises it". The existing AC-16 test now binds correctly to the AC as written. |
| FND-002 | fixed | Commit 148abdc rewords FR-085-AC-21 and the bundle-validation bullet in terms of model `exclude:` globs and drops `document:`. The existing AC-21 test (own root excludes, crossed root does not borrow) now matches the AC. |
| FND-003 | fixed | Commit 148abdc puts both members under one parent TempDir (`<p>/a`, `<p>/b`) and asserts `b/spec/HAZ-001.md` exists. A single global path index would now resolve the link to id `HAZ-001`, and the `ends_with("HAZ-001.md")` lookup would panic, so the test can fail. |
| FND-004 | fixed | Commit 148abdc adds FR-085-AC-22 (no members, root-count mismatch naming both counts, multi-member `with_bundle_package` error) and FR-085-AC-23 (`member_count`, `member_identity`). Both are tagged on tests that pass at 148abdc. |
