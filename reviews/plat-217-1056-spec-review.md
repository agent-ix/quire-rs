---
id: SR-20261103118119
title: "PLAT-217 and PLAT-1056 FR-033-AC-17 and FR-038-AC-9 spec-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs PR590 head 677dbd60; spec/functional/FR-033-locator-assert-facet.md; spec/functional/FR-038-okf-bundle-validation.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-033
    type: references
  - target: ix://agent-ix/quire-rs/FR-038
    type: references
---

## Summary

Tickets: PLAT-217, PLAT-1056. One spec-review pass over the spec part of `git diff origin/main...677dbd60`: new FR-033-AC-17 and FR-038-AC-9, TC-1991 and TC-1992, and the two coverage-ledger rows. Ids FR-033-AC-17, FR-038-AC-9, TC-1991 and TC-1992 are new and unique in the tree. Each AC has one test-case row and one ledger row, and they agree. The consistency (integrity) check was folded into this base pass; no separate sub-analysis artifact. FR-038 was read in full for context: its statement (resolvable `ix://` references, a dangling one is an error) and FR-038-AC-3.

## Verdict

Changes requested. FR-033-AC-17 is testable and matches the implementation, but it leaves the cell-counting rule unstated. FR-038-AC-9's "(or no package is declared)" clause contradicts FR-038-AC-3 and the FR-038 statement on the path where no package is declared, which is the CLI's only path.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-9 excuses an unresolved `ix://` target when "no package is declared". `validate_bundle_at` and the `quire validate --okf` CLI path never declare one. On that path no `ix://` miss is ever dangling. That contradicts FR-038-AC-3 ("a dangling `ix://` reference yields ... `dangling-reference`") and the FR-038 statement ("a dangling reference is an error"), and lets a wrong own-repo `ix://` id pass. Fix: delete the parenthetical and state that with no declared package an unresolved `ix://` target stays `dangling-reference`. Cross-repo tolerance then depends on the caller declaring the package (the CLI wiring is a quire-cli change). | spec/functional/FR-038-okf-bundle-validation.md:105 | wrong-requirement |
| FND-002 | medium | FR-038-AC-9 does not say how `<org>/<repo>` is taken from a two-segment `ix://<org>/<id>` URI, which FR-026 accepts, or how authorities compare: exact bytes or case-insensitive, given that GitHub org and repo names are case-insensitive and the package comes from a git remote. Two implementers would classify `ix://Agent-IX/quire-rs/X` against package `agent-ix/quire-rs` differently. Fix: define the extraction (three or more segments) and the comparison. | spec/functional/FR-038-okf-bundle-validation.md:105 | wrong-requirement |
| FND-003 | medium | FR-033-AC-17 says "authored cell count" without defining how cells are counted. It does not say that an escaped `\|` is one cell, that outer delimiter pipes are optional, or whether an unescaped pipe in a code span splits (GFM: it does). PLAT-217 asks for the escaped-pipe rule explicitly. Fix: state the GFM counting rule in the AC and add an escaped-pipe vector to TC-1991. | spec/functional/FR-033-locator-assert-facet.md:86 | wrong-requirement |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038-AC-9 now says any absent target when no package is declared is still `dangling-reference`. |
| FND-002 | fixed | FR-038-AC-9 defines the authority as the first two of at least three path segments, compared by exact string, and says that a two-segment `ix://<org>/<id>` has no authority. |
| FND-003 | fixed | FR-033-AC-17 states the GFM counting rule: outer pipes are not cells, `\|` is cell text, an unescaped pipe in a code span is a delimiter, and empty cells count. |
