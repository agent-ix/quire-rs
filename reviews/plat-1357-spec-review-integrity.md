---
id: SR-20261103118134
title: "spec-review (integrity) — PLAT-1357 FR-038-AC-10, TC-2151 (quire-rs PR 620)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs PR620 head 65437a4; spec/functional/FR-038-okf-bundle-validation.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-038
    type: references
---

## Summary

Ticket: PLAT-1357. Integrity check of the PR's new ids and rows. Id uniqueness: `TC-2151` and `FR-038-AC-10` do not occur in origin/main's spec (highest TC there is TC-2141; `FR-038-AC-10` appears only inside committed benchmark output under `spec/evidence/measurements/`, as an id of a different measured corpus, so it is not a collision). Neither id is added by open PRs 611, 615, 616, 617, 618, 619 or 521 (619 adds TC-2150; 616 TC-2095..2100; 611 TC-2040/2043; 615 TC-2056..2090; 617 TC-2055). PR 621 is stacked on this PR's commit and carries the same ids by design. AC row, TC row and mapping row agree. `make validate` (process/iso roots = fresh origin/main worktrees): 196 documents, 0 failed, 44 warnings, identical line for line to origin/main; none is in a touched file.

## Verdict

Pass with low findings. No id collision, no broken row.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-10 restates AC-9's rule: the case fold is already in AC-9, and AC-10's "with no package declared every absent target stays `dangling-reference`" repeats AC-9's no-package clause. One clause, two ACs, two tests. Keep the fold rule in one AC and give AC-10 only the case vectors. | spec/functional/FR-038-okf-bundle-validation.md:105-106 |
| FND-002 | low | If this PR merges without PR 621, AC-9 now asserts case-insensitive comparison while its verification TC-1992 stays ✅ and the code on main is byte-exact. The ✅ is then false until 621 lands. Land 620 and 621 together, or merge 621 alone (it contains this commit). | spec/tests.md:1196 |
| FND-003 | low | The "post-audit explicit mapping" table in `spec/tests.md` is a hand-kept AC-to-TC ledger. It duplicates the AC table's Verification column and the TC table's AC column, and the PR adds a row to it. The computed matrix (`quire matrix`) is the source; the ledger is pre-existing ceremony and should be deleted repo-wide in its own change. | spec/tests.md:1820 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-038-AC-10 | examined | Unique; FND-001 |
| TC-2151 | examined | Unique; rows agree |
| FR-038-AC-9 | examined | FND-002 |
