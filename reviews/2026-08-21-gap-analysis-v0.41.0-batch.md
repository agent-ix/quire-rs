---
id: SR-010
title: "Gap analysis — v0.41.0 batch: matrix integrity, trace-binding claims, sequencing and release ordering"
type: SpecReview
analysis: gap-analysis
scope: "spec/tests.md, spec/functional/FR-050-declarative-coverage-computation.md, spec/functional/FR-051-source-symbol-extraction.md, spec/functional/FR-024-parallel-repo-walk.md, spec/functional/FR-025-spec-corpus-model.md, spec/functional/FR-027-whole-spec-query-api.md, src/coverage.rs, src/symbols/, src/traceability.rs, scripts/slash_tag_sweep.py, reports/2026-08-20-slash-trace-sweep.md, reports/2026-08-20-slash-trace-sweep.json, CHANGELOG.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/log", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references }
---

## Summary

Companion to SR-009 (per-PR retroactive review). Where SR-009 reviews the seven
PRs as diffs, this artifact measures the **state they left behind**: the Test
Matrix after the #203/#204/#209 churn, whether the trace-binding claims of the
batch hold under the engine, and the two cross-repo debts the batch created —
the engine-before-contract sequencing pattern and the live release-ordering
violation. Measurements: installed `quire` CLI (self-reports `quire 0.23.0`,
quire-cli#52; engine quire-rs v0.41.0) over `main` at `954b315`, module
`spec-artifacts-process`.

## Verdict

CONDITIONAL

The matrix on `main` is coherent again — one row per test-case id in the Test
Case Summary, the AC→TC audit complete for FR-050-AC-21 and FR-051-AC-18, totals
838/1163 with zero status lies — but the *released* v0.41.0 matrix is corrupt,
one shipped fix has no matrix row at all, two test-case ids are each bound to
two symbols with no diagnostic, three of the sweep's 55 conversions are placebo,
and both cross-repo debts are open. Clears with #212–#219 and the WP2/WP3/WP5
release sequence (v0.42.0 + quire-cli v0.29.0).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Released-tag matrix integrity: v0.41.0 (`7278e98`) ships TC-827's row with `Traces To` = `FR-051-AC-18`, `Status` = `TC-943`, a stray `| FR-051-AC-17 | ✅ |` fragment, and no `FR-051-AC-18` audit-table row despite FR-051 declaring `AC-1..18` — so at the tag, the audit table's own completeness claim ("Every AC defined in the spec is listed") is false. Repaired on main by #209; remedied only by tagging v0.42.0. No local gate would have caught it: `make ci` never runs `quire validate` on the repo's own matrix (#212) | ix://agent-ix/quire-rs/spec/tests |
| FND-002 | medium | Duplicate test-case ids bound to multiple symbols: TC-943 tags two test fns (`src/symbols/typescript.rs:550,605`) and TC-944 tags two (`src/symbols/mod.rs:432,479`). The matrix lists each id once; the engine reports no diagnostic when one row id is backed by several distinct symbols, so the state is invisible to `quire coverage` and `quire validate` alike (#216). The Test Case Summary itself is clean — 615 rows, no duplicated row ids (TC-004/006/030/091 recur only in the separate Option Permutation Matrix table, by design) | ix://agent-ix/quire-rs/spec/tests |
| FND-003 | medium | Missing matrix row: #203's `implements` optional-key fix in tc859 (`tests/output_contract.rs`) is a real engine-contract correction — `implements` has carried `skip_serializing_if` since CR-080 and the contract test disagreed with the engine in the one direction it exists to catch — yet no TC row, CR note, or audit entry records it (#213). The fix is invisible to the matrix it corrects the contract for | ix://agent-ix/quire-rs/spec/tests |
| FND-004 | high | #208's binding claims, re-measured: aggregate holds — `totals.backed` 838/1163, `status_lies` 0, `undeclared_statuses` empty, matching #209's recorded post-repair numbers — but the claim was only ever verified manually (+46 backed ids; no pre-edit census committed, see FND-005). Cross-checking every id on the 55 edited lines against `unbacked_rows`: **52 edits are effective; 3 are placebo.** The four lines with an ` + <id>` tail (`// TC-473, FR-024-AC-4 + NFR-006: …`; TC-483/FR-025-AC-4; TC-485/FR-025-AC-6; TC-498/FR-027-AC-6) still bind only their first id — the comma-list grammar stops at the ` + ` — leaving FR-024-AC-4, FR-025-AC-4 and FR-027-AC-6 (verification rows, method Test) unbacked on main today; FR-025-AC-6 escapes only because its row is method Inspection. The GREEN classifier judged the slash-span, not whether the rewritten line is readable by the tag grammar (#217) | ix://agent-ix/quire-rs/spec/functional/FR-024 |
| FND-005 | medium | The sweep's census is not re-derivable: the committed JSON is the post-edit re-run only (`totals.green` 158, quire-rs `green: 0`, `write: false`), so the before-numbers the report and PR assert (214 GREEN, "quire-rs's 56", backed 793→839/1164) exist in prose alone — against the harness docstring's own rule that a reported number must be the census, not a reconstruction. The diff edited 55 lines, not 56; and the md attributes 238 repos to `scripts/corpus.py`, which enumerates 239 (238 is post-`ecaz`-exclusion, per the JSON's own `repos_scanned`/`excluded_repos`). All on #217; the count-discrepancy angle also on #219 | ix://agent-ix/quire-rs/reports/2026-08-20-slash-trace-sweep |
| FND-006 | medium | Engine-first-key sequencing, 4th occurrence: `source_exclude` shipped in the engine (CR-085, #206, in v0.41.0) before any contract knew the key. spec-artifacts-iso#28 then added it to the manifest schema, recording that its `additionalProperties: false` gate "has now caught **four** keys the engine accepts and this contract had never heard of" (after CR-005/007/008/009's keys) — "a good record for the gate and a poor one for the sequencing". The pattern is structural: nothing in this repo's process requires the contract change to precede or accompany the engine change | ix://agent-ix/quire-rs/spec/log |
| FND-007 | high | Live release-ordering violation (`deny_unknown_fields`): `TraceabilityModel` rejects unknown manifest keys at load, so a module declaring `source_exclude` hard-fails on any engine older than v0.41.0. Verified today: spec-artifacts-process **v0.23.0 is tagged** (tag → #55, which declares `source_exclude`) while the first tolerant CLI release, **quire-cli v0.28.0, is tagged but unpublished** — no GitHub release (latest v0.27.0, 2026-08-20), npm registry tops at 0.27.0; SAP additionally has zero GitHub releases for any tag. Any environment resolving SAP v0.23.0 against a published quire-cli gets a module-load failure on every command. Execution: WP2 (release scaffolding) + WP5 (single quire-cli v0.29.0); guardrails NS-1/NI-1 pin the invariant cross-repo | ix://agent-ix/quire-rs/spec/log |
| FND-008 | low | Residual validate debt on main: `quire validate spec/tests.md` fails on exactly two pre-existing line-145 `Traces To` cells (`FR-118 compatibility reference`, `Filament IDE FR-046 reference`), reproduced in this review — known to and left unticketed by #209, now #218 | ix://agent-ix/quire-rs/spec/tests |

## Matrix integrity after the churn

State of `spec/tests.md` on `main` (`954b315`), measured:

| Check | Result |
| --- | --- |
| Test Case Summary row ids | 615 rows, no duplicates (recurrences of TC-004/006/030/091 are the Option Permutation Matrix table) |
| TC-941, TC-942, TC-943, TC-944, TC-945 | present exactly once each; FR-050 row reads `AC-1..21`, FR-051 row `AC-1..18` |
| AC→TC audit | `FR-050-AC-21 → TC-941, TC-942` (#203) and `FR-051-AC-18 → TC-943` (#209's addition) both present |
| `quire coverage --json` | backed 838 / total 1163; `status_lies` 0; `undeclared_statuses` absent; `untracked_symbols` 1 |
| `quire validate spec/tests.md` | two failures, both FND-008's pre-existing cells — nothing from the batch |
| Same file at tag v0.41.0 | corrupt per FND-001 |

The one-row difference from #208's totals (839/1164 → 838/1163) is #209 removing
the spurious fragment row, exactly as its commit message records.

## The sequencing and ordering record

The dependency chain the batch created, with publication status verified
2026-08-21:

```
quire-rs v0.41.0 (published tag; carries FND-001)
  → quire-cli v0.28.0 (tag exists; NO GitHub release; npm tops at 0.27.0; binary self-reports 0.23.0 — quire-cli#52)
  → spec-artifacts-iso v0.18.0 (tag; schema key added by iso#28 AFTER the engine shipped — FND-006)
  → spec-artifacts-process v0.23.0 (tag = #55 declaring source_exclude; zero GitHub releases)
```

The contract-tolerance link (quire-cli) is the only unpublished link, and it is
the one `deny_unknown_fields` makes load-bearing (FND-007). The recorded plan:
v0.28.0 stays as a known-bad tag (tags are never deleted); all quire-cli fixes
batch into a single published v0.29.0 (WP5), with the version-stamp smoke check
added by WP2 so a binary can no longer report a version three tags old.

## Coverage

**Requirements in scope.** FR-050-AC-21 (backed, TC-941/TC-942), FR-050-AC-22
(backed, TC-944/TC-945 — with the duplicate-id caveat of FND-002),
FR-051-AC-18 (backed, TC-943 ×2 — same caveat). Regressed-in-place:
FR-024-AC-4, FR-025-AC-4, FR-027-AC-6 remain unbacked despite being targets of
#208's sweep (FND-004; they were unbacked before it too — the sweep failed to
deliver them, it did not break them).

**Underspecified code.** The `implements` serialization contract is asserted
only by an untracked test edit (FND-003). The excluded-file subtraction of
CR-085 has no owning observability criterion — `source_exclude` can remove
files with no count anywhere in the output (SR-009 FND-008, #215).

**Semantic review.** Not run — this pass is deliberately mechanical
(measurements against the engine); intent-level review of the batch is SR-009.

**Environment note.** Every `quire validate` run in this environment emits six
module-load diagnostics (`DuplicateArchetype` ×5, `DuplicateInverseEdge` ×1 —
the local module set registers `spec-artifacts-process` twice; quoin#174
territory). They are diagnostics, not failures: the exit code still reflects
document validity, and this document and SR-009 both validate clean (exit 0, no
failure lines).

**Follow-ups carried, not done here.** #212–#219 (NR-1..NR-8), #210 (grew:
`UndeclaredStatus` has no line field), #211 (sweep tail — blocked on #217, per
FND-004/FND-005 the harness must not re-run as-is), WP2/WP5 release ordering,
NS-1/NI-1 cross-repo guardrails.
