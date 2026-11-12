---
id: SR-20261112034128
title: "spec-integrity-analysis of PLAT-1476 located engine findings (quire-rs PR 653)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@cc37f57; spec/functional/FR-033-locator-assert-facet.md; spec/functional/FR-049-verification-reference-integrity.md; spec/functional/FR-069-semantic-module-contract-at-load.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
---

## Summary

Ticket: PLAT-1476. This SR is the cross-document consistency lens over the PR 653 diff. It covers the new text against FR-011 (the existing extract record), against FR-049's own ordering ACs and the reference-scan code, against quoin FR-073 and filament-core-data's `module-manifest.schema.json` (both read-only at `origin/main`), and against open PR 652 (FR-058), which also edits `spec/tests.md`.

Checked clean:

- The FR-049 ordering key (path, id, reason, message, line) adds the line as the last key, so the FR-049-AC-8 and AC-17 order is preserved.
- No existing trace-reference test relies on cross-row dedup.
- The filament-core-data follow-up is correct, because `ArtifactTypeEntry` is `additionalProperties: false` (schema line 396) and would reject `record_validation`. quire-rs's own loader keeps archetype extras in a flattened map, so it would not.
- The TC ids do not collide with PR 652, which uses TC-2248..TC-2255.
- No unreleased roadmap or private-repo detail appears in the diff.
- `spec_validate` reports 0 failed.

## Verdict

**FAIL.** Two contradictions with existing requirements. The PR also claims a quoin requirement is satisfied when it is not.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The "yield record" FR-069 defines contradicts the record FR-011 already builds from the same `yield_pattern.match`. In the engine, a `table_row` yields strings (the `column` cell, or the row joined by tabs; locator.rs:618-663), collapsed to the first value unless `multiple: true` (FR-011-AC-21, extract/mod.rs:128-137). FR-069 says a `table_row` "gives an array, a row being an object keyed by the table's headers". FR-069 also drops the required-miss abort, `regex` capture rewriting and placeholder dropping. The result is two different records under one name. Fix: define the yield record as FR-011's `extract` record, or state each departure as a CR on FR-011 | spec/functional/FR-069-semantic-module-contract-at-load.md:273-281; spec/functional/FR-011-body-extraction-dsl.md:221 |
| FND-002 | medium | The interface paragraph puts normative SHALLs on another repository ("quoin SHALL NOT run a second record validator and SHALL surface Quire's located findings unchanged"). It also says quoin FR-073's last bullet "is satisfied" by this subsection. That bullet validates the extracted record against the referenced `<Name>.json`, which for Anomaly is the emitted-record schema the yield record cannot meet (spec-review SR FND-001), so the claim is false. Keep only what quire-rs provides, and file the quoin and filament-core-data asks as tickets in those lanes | spec/functional/FR-069-semantic-module-contract-at-load.md:309-321 |
| FND-003 | medium | FR-049 makes `line` "a field of the finding" and says "the rendered diagnostic leads with `<path>: line <N>:` in the form the command uses". `BundleFinding` has no `line` field (corpus/validate.rs:82-102), and quire-rs has no command or `Display` that renders bundle findings, because quire-cli does that. The spec must say whether the line goes into `message` (testable here) or is a renderer obligation (a quire-cli ask). It must also say what `line` is for the other bundle findings (`Option`, absent) | spec/functional/FR-049-verification-reference-integrity.md:116-130 |
| FND-004 | low | Census change not stated. `validate_trace_references` runs `findings.sort(); findings.dedup()` over (path, id, reason, message) (trace_refs.rs:172-173). Today the same id dangling on two rows of a declaration with no `row_id_column` collapses to one finding. With the line in the tuple, the prose should say these now count separately, so a jump in corpus counts is not read as a regression | spec/functional/FR-049-verification-reference-integrity.md:124-127 |
| FND-005 | low | PR 652 adds its TC rows at the same insertion point (after TC-2247, hunk `@@ -1221`) and audit rows in the same table. Whichever lands second needs a textual rebase. The ids do not collide | spec/tests.md:1221 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-069 Located record validation prose | examined | FND-001, FND-002 |
| FR-069-AC-23 | examined | FND-001 |
| FR-049 Located findings prose | examined | FND-003, FND-004 |
| FR-049-AC-19 | examined | FND-003 |
| FR-049-AC-21 vs FR-049-AC-8, FR-049-AC-17 | examined | Consistent |
| spec/tests.md summary, TC and audit rows | examined | FND-005 |
| FR-011-AC-21 | context_only | Existing extract-record contract |
| quoin FR-073 (origin/main) | context_only | Last bullet names `<Name>.json` |
| filament-core-data module-manifest.schema.json | context_only | ArtifactTypeEntry closed, so the follow-up is valid |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | FR-049 says `line` is `None` for every bundle finding other than the two trace findings. `validate_bundle` bridges each document `ValidationError` through `BundleFinding::bridged(reason, path, message)` (corpus/validate.rs:543-562) and drops `err.line`. FR-069 names the bundle path as an entry point, so its located `semantic.record-invalid` findings, and every other located document finding, reach `validate_bundle` callers with no line. A bridged finding should carry `line: err.line` | spec/functional/FR-049-verification-reference-integrity.md:120-124 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | ed5519b: the yield record is defined as `extract::extract` over `yield_pattern.match`, "a `table_row` is its `column` cell, or the row's cells joined by tabs; the first value unless `multiple: true`". This matches locator.rs:618-663 and extract/mod.rs:128-137. The required-miss abort is FR-011's (extract/mod.rs:99-104), and FR-069 now defers to it. |
| FND-002 | fixed | ed5519b: the interface paragraph with SHALLs on quoin is removed from FR-069. The follow-ups live only in the PR body, non-normatively. |
| FND-003 | fixed | ed5519b: `BundleFinding` gains `line: Option<usize>`, and the render clause is dropped ("how a surface renders the line is that surface's concern"). The struct is constructed only in-crate (`in_pack`, `bridged`), so the new field breaks no caller. Residual: FND-006. |
| FND-004 | fixed | ed5519b: the line joins the sort and dedup key, and the prose says counts may rise for declarations without `row_id_column`. |
| FND-005 | fixed | PR 652 landed (e1bca82) and was merged into the branch (42d24a9). TC-2248..TC-2257 and TC-2300..TC-2311 are disjoint, with no duplicate TC rows in spec/tests.md. |
