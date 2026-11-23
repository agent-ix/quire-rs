---
id: SR-318
title: "integrity review of FR-051 source walk amendment (PR #708)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (Source walk, FR-051-AC-54..57, CR-423 note) @ ab0fa9958126956ee82d21b061ace08e73b8710c"
review_set: subset
---

## Summary

Ticket: PLAT-1793. Consistency of AC-54..57 and the CR-423 note against FR-051's own
identity rule, FR-050 (AC-5, AC-22, AC-39), FR-077 and the code at the frozen head.
Two criteria contradict existing contracts (AC-57 twice), AC-56 cannot fail against
current code while the first-wins it targets survives elsewhere, the "only these rules"
walk list omits a rule the code already applies, and the CR note misstates the corpus
walk. Exact-id vs hierarchy (FR-077-AC-6) and claim vs citation (FR-077-AC-1/2) are
not altered by the diff (examined, clean).

## Scope examined

- FR-051 Source walk section — examined
- FR-051-AC-56, FR-051-AC-57 — examined (findings)
- FR-051-AC-54, FR-051-AC-55 — examined for cross-reference consistency
- CR-423 note — examined (finding)
- FR-051 identity statement (lines 28-34), FR-050-AC-5, FR-050-AC-22, FR-050-AC-39,
  FR-077-AC-1, FR-077-AC-4, FR-077-AC-6 — context_only
- Code: src/symbols/mod.rs:376-500, src/corpus/walk.rs:293-321,
  src/symbol_table.rs:171-225, src/symbols/trace_search.rs:147-330,
  src/coverage/reconcile.rs:436-456 — context_only

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | high | AC-57 says an unminted marker id "is reported by FR-050-AC-39 and binds nothing"; a marker binds a `verifies` relation independent of the corpus and the unknown id is reported under FR-050-AC-5 `untracked_symbols` (reconcile.rs:436-456), and FR-077-AC-1 forward query returns it. FR-050-AC-39 covers only generic tokens no declared form bound, so it never fires for a marker. | spec/functional/FR-051-source-symbol-extraction.md:284 |
| FND-002 | high | AC-57 requires adding/removing/reformatting a declaration to leave "every other symbol's ... report position unchanged"; a declaration inserted above another in the same file shifts that symbol's line, which FR-051 (lines 31-33) carries as a non-identity attribute. No implementation can pass the same-file case as written. | spec/functional/FR-051-source-symbol-extraction.md:284 |
| FND-003 | medium | AC-56 already holds at the frozen head (bind walks every symbol; search_symbol filters by path+qualified_name), so it cannot fail; the real first-wins is symbol_table.rs:220-221, where `verifies.remove(symbol.id)` gives the first colliding record every id and later colliders empty `trace_ids`/`implements`. AC-56 names no FR-051-AC-23 symbol-table outcome, so the collision-binding risk stays open. | spec/functional/FR-051-source-symbol-extraction.md:283; src/symbol_table.rs:220 |
| FND-004 | medium | Source walk says the walk applies "only these rules", omitting the non-configurable cookiecutter `{{...}}` template-tree prune the code walk applies (symbols/mod.rs:476-481) and counts in `excluded_source_files`. Existing behaviour now violates the SHALL, or an implementer removes the prune and placeholder ids bind. | spec/functional/FR-051-source-symbol-extraction.md:38-49 |
| FND-005 | low | CR-423 note says the corpus walk "already pins the same choices"; walk.rs:307-315 sets git_exclude(true) and parents(true) and leaves git_global at its default, i.e. it consults exactly the `.git/info/exclude`, ancestor and user-global files AC-55 forbids for the code walk. The record is false and the two walks under one scope now disagree. | spec/functional/FR-051-source-symbol-extraction.md:1116; src/corpus/walk.rs:307 |

## Verdict

FAIL. FND-001 and FND-002 must be fixed before AC-57 is implementable; FND-003 must
retarget AC-56 at the symbol-table attribution (per-declaration `trace_ids`) so it can
fail today.
