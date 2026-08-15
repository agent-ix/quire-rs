---
id: SR-005
title: "Gap analysis — CR-044 type-driven corpus membership"
type: SpecReview
analysis: gap-analysis
scope: "src/corpus/walk.rs, src/corpus/validate.rs, src/corpus/glossary.rs, tests/spec_dogfood.rs"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-024"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-038"
    type: "references"
    cardinality: "1:1"
---

# SR-005: Gap analysis — CR-044 type-driven corpus membership

## Summary

Verification gate over the CR-044 slice on branch
`feat/73-type-driven-corpus-membership` (PR #87), closing agent-ix/quire-rs#63,
#73, #76 and #77. The change deletes `DEFAULT_SKIP` and
`WalkOptions::skip_names`, makes corpus membership depend on the presence of a
frontmatter block, reduces `NON_ARTIFACT_FILES` to `{index.md, log.md}`, and
retires the hand-rolled markdown walker in `tests/spec_dogfood.rs`.

There is **no plan bundle** for this slice — it was driven from the issue set
rather than from `plan/`, so Step 1 (plan completion) does not apply and is
recorded as not-applicable rather than passed.

Three defects were found by the review and fixed on the branch before this
document was written. All three were **silent-agreement failures**: places
where a second consumer of the membership rule, or a second statement of it,
did not move with the first.

## Verdict

**CONDITIONAL** — no unbacked new matrix row and no `high` finding survives, but
two `medium` findings record pre-existing gaps this slice sits on top of and
does not close.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | high | `glossary_terms_from_path` inherited the `{README.md, tests.md}` skip through `discover_files`; deleting the skip would have silently widened its scan to every stray `.md`, letting a non-document define a repository's ubiquitous language. Fixed: the rule is extracted to `walk::is_document` and applied by both consumers, pinned by FR-024-AC-11 / TC-808. | FR-024-AC-11 |
| FND-002 | high | FR-024-AC-10 as first written described a bundle holding two files named `tests.md` in one directory — unsatisfiable — and TC-807 built a different tree than the criterion stated. Fixed: the criterion now names a sibling directory and TC-807 asserts exactly that tree. | FR-024-AC-10 |
| FND-003 | high | FR-038's index-completeness prose still excluded `README.md` and `tests.md`, contradicting the reduced `NON_ARTIFACT_FILES` — the code changed and its owning requirement did not. Fixed: FR-038 prose and FR-038-AC-5 updated with a CR-044 reference. | FR-038-AC-5 |
| FND-004 | medium | FR-024-AC-1..AC-9 are unbacked: their `Verification` cells read a bare `Test` with no test id, and no symbol carries the criterion id. The two criteria added here are backed, taking the FR-024 group from 0/9 to 2/11, but the pre-existing nine are untouched and out of scope for this slice. | FR-024 |
| FND-005 | medium | `tests/coverage_rollup.rs` still asserts that `tests.md` is not a corpus document. It passes for a **different reason** after this change — those fixture matrices carry no frontmatter rather than being skipped by name — and it inverts when agent-ix/quire-rs#74 gives them `type: TestMatrix`. Comments corrected; the assertion is left for #74. | FR-050-AC-15 |
| FND-006 | low | 13 untracked symbols and 7 status lies exist repo-wide and are unchanged by this slice (13 → 13, 7 → 7). None involve the changed files. | - |

## Coverage

**[RAN]** `quire coverage --scope . --json` against a branch-built `quire`
(quire-cli v0.15.0 compiled against this working tree; the installed CLI is
0.14.0 and predates the model), module `spec-artifacts-process` at v0.13.1.
Baseline is a `main` worktree measured with the same binary.

| | main | branch | delta |
|---|---|---|---|
| criteria backed | 361 | 365 | +4 |
| criteria total | 928 | 932 | +4 |
| status lies | 7 | 7 | 0 |
| untracked symbols | 13 | 13 | 0 |

Every row added by this slice is backed by a real tagged test: TC-807
(`corpus::walk::tests::tc807_membership_is_type_driven_not_filename_driven`) and
TC-808 (`corpus::glossary::tests::tc808_from_path_applies_the_corpus_membership_rule`),
each carrying both its `TC-` id and its owning `FR-024-AC-` id. The +4 is
exactly those two test cases plus the two criteria they back — the slice adds no
unbacked row.

Gates: `make ci` (fmt-check, clippy `-D warnings`, 19 test binaries, cargo-deny,
unsafe/property/static audits) and `make ci-python` (maturin wheel + 35 PyO3
binding tests) both green.

**Semantic review (Step 4) was not run** — the mechanical steps found and closed
three real defects, and the slice is one rule with two consumers rather than a
broad requirement surface.

### Ecosystem measurement

**[RAN]** `scripts/classify_matrices.py` over `~/dev`, worktrees and `-task<N>`
copies deduped: of 184 matrices at a bound path, **0 carry no frontmatter
block** — the regression this change could have caused does not exist in the
corpus. 170 are typed `TestMatrix` and are unaffected; 14 are mis-typed and
would go invisible, of which 6 mint rows today and need a one-line frontmatter
fix. 20 real matrices across 9 repos become visible for the first time, 12 of
them minting rows.

**[RAN]** 4 of 180 repos with a `spec/tests.md` already list it in
`spec/index.md`, so the `NON_ARTIFACT_FILES` reduction raises `index-incomplete`
in 172 repos. That is authoring debt the suppression was hiding; it is reported,
not absorbed.
